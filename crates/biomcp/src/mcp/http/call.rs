//! Own bounded hosted tool workers through completion and return whole-call timeouts.
use super::Service;
use crate::{
    command::Context,
    mcp::{protocol, research, worker},
};
use biomcp_fetch::{Cancellation, Secrets};
use serde_json::Value;
#[expect(
    clippy::disallowed_types,
    reason = "The HTTP service owns its bounded active tool workers"
)]
use std::{
    sync::{Mutex, mpsc},
    thread::JoinHandle,
};

#[derive(Default)]
#[expect(
    clippy::disallowed_types,
    reason = "The HTTP service owns its bounded active tool workers"
)]
pub(super) struct Calls(Mutex<Vec<Active>>);
struct Active {
    worker: JoinHandle<()>,
    scope: Cancellation,
}
pub(super) struct HostedCall {
    pub(super) id: Value,
    pub(super) modern: bool,
    pub(super) call: research::Call,
    pub(super) secrets: Option<Secrets>,
    pub(super) excluded: std::collections::BTreeSet<biomcp_types::Source>,
    pub(super) declared_use: crate::command::DeclaredUse,
    pub(super) scope: Cancellation,
}

impl Calls {
    #[expect(
        clippy::disallowed_methods,
        reason = "Retain every hosted tool worker until it completes"
    )]
    pub(super) fn run(&self, service: &Service, work: HostedCall) -> Value {
        let HostedCall {
            id,
            modern,
            call,
            secrets,
            excluded,
            declared_use,
            scope,
        } = work;
        let mut active = self.0.lock().unwrap_or_else(|e| e.into_inner());
        for index in (0..active.len()).rev() {
            if active
                .get(index)
                .is_some_and(|call| call.worker.is_finished())
            {
                let _ = active.swap_remove(index).worker.join();
            }
        }
        if active.len() >= service.max_calls {
            return protocol::capacity(id, service.max_calls);
        }
        if scope.remaining().is_zero() {
            return protocol::timeout(id);
        }
        let globals = service.globals.clone();
        let root = service.root.clone();
        let settings = service.settings.clone();
        let fetch = service.fetch.clone();
        let recovery = service.recovery.clone();
        let worker_scope = scope.clone();
        let worker_id = id.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("mcp-http-tool".into())
            .spawn(move || {
                let context = Context {
                    globals: &globals,
                    root: &root,
                    settings: &settings,
                    fetch: &fetch,
                    recovery: recovery.as_deref(),
                    secrets: secrets.as_ref(),
                    excluded: Some(&excluded),
                    declared_use: Some(declared_use),
                    scope: Some(&worker_scope),
                };
                let outcome = worker::execute(|| {
                    research::execute(&call, context, true).map(|outcome| {
                        super::super::typed::tool_result(
                            worker_id,
                            &outcome,
                            modern,
                            context.settings,
                        )
                    })
                });
                let _ = sender.send(outcome);
            });
        let worker = match worker {
            Ok(worker) => worker,
            Err(error) => {
                super::connections::failure("tool thread start", error);
                return protocol::error(id, -32603, "Internal error");
            }
        };
        active.push(Active {
            worker,
            scope: scope.clone(),
        });
        drop(active);
        wait(&scope, receiver, id)
    }
    pub(super) fn cancel(&self) {
        for call in self.0.lock().unwrap_or_else(|e| e.into_inner()).iter() {
            call.scope.cancel();
        }
    }
}
impl Drop for Calls {
    fn drop(&mut self) {
        self.cancel();
        for call in self
            .0
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .drain(..)
        {
            let _ = call.worker.join();
        }
    }
}

fn wait(
    scope: &Cancellation,
    receiver: mpsc::Receiver<std::io::Result<Value>>,
    id: Value,
) -> Value {
    loop {
        let remaining = scope.remaining();
        if remaining.is_zero() {
            scope.cancel();
            return protocol::timeout(id);
        }
        match receiver.recv_timeout(remaining.min(super::ACCEPT_POLL)) {
            Ok(reply) => {
                if scope.remaining().is_zero() {
                    scope.cancel();
                    return protocol::timeout(id);
                }
                return reply.unwrap_or_else(|_| protocol::error(id, -32603, "Internal error"));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return protocol::error(id, -32603, "Internal error");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Active, Calls, HostedCall, Service};
    use crate::config::DEFAULT_HTTP_CAPACITY as MAX_CONNECTIONS;
    use crate::{
        config::StartupError,
        grammar::Globals,
        mcp::research::{Call, Projection},
    };
    use biomcp_fetch::Cancellation;
    use serde_json::json;
    use std::{sync::mpsc, time::Duration};

    pub(super) fn test_service() -> Service {
        Service {
            hosted_max_use: Default::default(),
            root: std::sync::Arc::new(crate::grammar::root().unwrap()),
            settings: std::sync::Arc::new(Ok(None)),
            calls: Calls::default(),
            connections: super::super::admission::Slots::new(MAX_CONNECTIONS),
            max_calls: MAX_CONNECTIONS,
            max_connections: MAX_CONNECTIONS,
            address: "127.0.0.1:1".parse().unwrap(),
            hosts: Vec::new(),
            globals: Globals::default(),
            fetch: Err(StartupError::Local),
            recovery: None,
        }
    }
    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Inject OS failures and observe recovery over a real local socket"
    )]
    fn accept_and_spawn_failures_leave_server_running() {
        use std::{
            io::{Read, Write},
            net::{TcpListener, TcpStream},
            sync::Arc,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let mut service = test_service();
        service.address = address;
        let (stop, stopped) = mpsc::sync_channel(1);
        let (failed, observed) = mpsc::channel();
        let server = std::thread::spawn(move || {
            let mut first_accept = true;
            let mut first_spawn = true;
            super::super::connections::run_with(
                listener,
                Arc::new(service),
                stopped,
                |listener| {
                    if std::mem::take(&mut first_accept) {
                        return Err(std::io::Error::other("forced accept failure"));
                    }
                    listener.accept().map(|(stream, _)| stream)
                },
                |work| {
                    if std::mem::take(&mut first_spawn) {
                        failed.send(()).unwrap();
                        return Err(std::io::Error::other("forced thread failure"));
                    }
                    std::thread::Builder::new().spawn(work)
                },
            )
        });
        let rejected = TcpStream::connect(address).unwrap();
        observed.recv_timeout(DEADLOCK_GUARD).unwrap();
        drop(rejected);
        let mut client = TcpStream::connect(address).unwrap();
        client.set_read_timeout(Some(DEADLOCK_GUARD)).unwrap();
        write!(client, "GET /health HTTP/1.1\r\nHost: {address}\r\n\r\n").unwrap();
        let mut reply = String::new();
        client.read_to_string(&mut reply).unwrap();
        assert!(reply.starts_with("HTTP/1.1 503 "), "{reply}");
        assert!(reply.contains("store"));
        stop.send(()).unwrap();
        server.join().unwrap().unwrap();
    }
    #[test]
    fn call_diagnostic_keeps_only_safe_fields() {
        let params =
            json!({"name":"get_gene", "arguments":{"id":"private-query", "key":"private-key"}});
        let reply = json!({"result":{"structuredContent":{
            "status":"partial", "query":"private-query", "sources":[
                {"source":"mygene", "state":"failed", "term":"private-key"},
                {"source":"private-key", "state":"failed"}
            ]
        }}});
        let line =
            super::super::logging::line(&params, Duration::from_millis(7), 200, Some(&reply));
        assert_eq!(
            line,
            json!({"event":"mcp_call", "tool":"get_gene", "duration_ms":7, "status":"partial", "failed_sources":["mygene"]})
        );
        let envelope = &reply["result"]["structuredContent"];
        for content in [
            json!({"content":[{"text":format!("private-query\n{envelope}")}]}),
            json!({"structuredContent":{"metadata":{"envelope":envelope}}}),
            json!({"structuredContent":{"metadata":envelope}}),
        ] {
            let reply = json!({"result":content});
            assert_eq!(
                super::super::logging::line(&params, Duration::from_millis(7), 200, Some(&reply)),
                line
            );
        }
        let unsafe_params = json!({"name":"private-query"});
        let line = super::super::logging::line(&unsafe_params, Duration::ZERO, 200, None);
        assert_eq!(line["tool"], "unknown");
        assert!(!line.to_string().contains("private"));
    }

    #[test]
    fn call_diagnostic_names_every_served_tool() {
        let service = test_service();
        let context = crate::command::Context::new(
            &service.globals,
            &service.fetch,
            &service.root,
            &service.settings,
        );
        let tools = crate::mcp::typed::tools(context, false).unwrap();
        let reference = tools
            .iter()
            .find(|tool| tool["name"] == "biomcp_reference")
            .unwrap();
        let grouped = reference["inputSchema"]["properties"]["command"]["enum"]
            .as_array()
            .unwrap();
        assert!(!grouped.is_empty());
        let log = |name: &serde_json::Value| {
            super::super::logging::line(&json!({"name":name}), Duration::ZERO, 200, None)
        };
        for name in tools.iter().map(|tool| &tool["name"]).chain(grouped) {
            assert_eq!(&log(name)["tool"], name);
        }
        for name in ["biomcp_private", "get_private", "reference", "BIOMCP", ""] {
            assert_eq!(log(&json!(name))["tool"], "unknown");
        }
    }

    const DEADLOCK_GUARD: Duration = Duration::from_secs(60);

    #[test]
    #[expect(
        clippy::disallowed_methods,
        reason = "Held worker exit events prove private HTTP capacity and cleanup ownership"
    )]
    fn expired_calls_keep_capacity_and_shutdown_waits_for_actual_worker_exit() {
        let service = test_service();
        let mut releases = Vec::new();
        let mut exits = Vec::new();
        let (leaving, observed) = mpsc::channel();
        for _ in 0..MAX_CONNECTIONS {
            let (release, released) = mpsc::channel::<()>();
            let (exit, exiting) = mpsc::channel::<()>();
            releases.push(release);
            exits.push(exit);
            let leaving = leaving.clone();
            let worker = std::thread::spawn(move || {
                let _ = released.recv();
                leaving.send(()).unwrap();
                let _ = exiting.recv();
            });
            service.calls.0.lock().unwrap().push(Active {
                worker,
                scope: Cancellation::with_budget(Duration::ZERO),
            });
        }
        let reply = service.calls.run(
            &service,
            HostedCall {
                id: json!(1),
                modern: false,
                call: Call {
                    args: Vec::new(),
                    projection: Projection::Cli,
                    catalog: None,
                },
                secrets: None,
                excluded: Default::default(),
                declared_use: crate::command::DeclaredUse::Academic,
                scope: Cancellation::with_budget(DEADLOCK_GUARD),
            },
        );
        assert_eq!(
            reply.pointer("/error/code"),
            Some(&json!(-31000)),
            "expired running workers must retain all server slots"
        );
        let (finished, finish) = mpsc::channel();
        let cleanup = std::thread::spawn(move || {
            drop(service);
            finished.send(()).unwrap();
        });
        drop(releases);
        for _ in 0..MAX_CONNECTIONS {
            observed.recv_timeout(DEADLOCK_GUARD).unwrap();
        }
        assert!(
            finish.try_recv().is_err(),
            "shutdown must own workers still held before actual exit"
        );
        drop(exits);
        finish.recv_timeout(DEADLOCK_GUARD).unwrap();
        cleanup.join().unwrap();
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "Measure explicitly invoked bounded HTTP load against one owned service and Fetch"
)]
#[path = "load_benchmark.rs"]
mod load_benchmark;
