//! Bounded blocking HTTP transport with independent request protocol state.
mod admission;
mod authority;
mod call;
mod chunked;
mod metadata;
mod wire;
use authority::Authority;
use call::{Calls, HostedCall};
use serde_json::{Value, json};
use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Arc,
    thread,
};
const ACCEPT_POLL: std::time::Duration = std::time::Duration::from_millis(20);
const HEALTH_BUDGET: std::time::Duration = std::time::Duration::from_secs(2);
pub(crate) enum StartError {
    Configuration(crate::config::StartupError),
    Usage,
    AllowHost,
    Local,
}
struct Service {
    calls: Calls,
    connections: admission::Slots,
    max_calls: usize,
    max_connections: usize,
    address: SocketAddr,
    hosts: Vec<Authority>,
    globals: crate::grammar::Globals,
    root: Arc<crate::grammar::Root>,
    settings: Arc<crate::config::Settings>,
    fetch: Result<Arc<biomcp_fetch::Fetch>, crate::config::StartupError>,
    recovery: Option<Arc<super::Recovery>>,
}
pub(crate) fn serve(
    address: &str,
    hosts: Vec<String>,
    globals: crate::grammar::Globals,
    root: Arc<crate::grammar::Root>,
    settings: Arc<crate::config::Settings>,
) -> Result<(), StartError> {
    let address: SocketAddr = address.parse().map_err(|_| StartError::Usage)?;
    let names = crate::config::http_hosts(&settings)
        .map_err(|error| StartError::Configuration(error.into()))?;
    let mut allowed = names
        .iter()
        .map(|host| {
            Authority::parse(host).ok_or_else(|| {
                StartError::Configuration(
                    crate::config::ConfigError::field("http.allowed_hosts").into(),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    allowed.extend(
        hosts
            .iter()
            .map(|host| Authority::parse(host).ok_or(StartError::Usage))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let hosts = allowed;
    if !address.ip().is_loopback() && hosts.is_empty() {
        return Err(StartError::AllowHost);
    }
    let (max_calls, max_connections) = crate::config::http_capacity(&globals, &settings)
        .map_err(|error| StartError::Configuration(error.into()))?;
    let fetch = crate::config::fetch(&globals, &settings).map(Arc::new);
    if let Err(error) = &fetch
        && error.is_pacing()
    {
        return Err(StartError::Configuration(error.clone()));
    }
    let listener = TcpListener::bind(address).map_err(|_| StartError::Local)?;
    let address = listener.local_addr().map_err(|_| StartError::Local)?;
    let recovery = fetch.is_err().then(|| Arc::new(super::Recovery::new()));
    let service = Arc::new(Service {
        calls: Calls::default(),
        connections: admission::Slots::new(max_connections),
        max_calls,
        max_connections,
        address,
        hosts,
        fetch,
        recovery,
        globals,
        root,
        settings,
    });
    let (stop, stopped) = std::sync::mpsc::sync_channel(1);
    // This standalone server owns shutdown, including inherited ignored signals.
    ctrlc::set_handler(move || {
        let _ = stop.try_send(());
    })
    .map_err(|_| {
        crate::output::diagnostic(b"biomcp: could not install HTTP shutdown handler\n");
        StartError::Local
    })?;
    let mut ready = serde_json::to_vec(&json!({"listening":address.to_string()}))
        .map_err(|_| StartError::Local)?;
    ready.push(b'\n');
    std::io::Write::write_all(&mut std::io::stderr().lock(), &ready)
        .map_err(|_| StartError::Local)?;
    connections(listener, service, stopped).map_err(|_| StartError::Local)
}
#[expect(
    clippy::disallowed_methods,
    reason = "Own bounded framing and active request workers through completion"
)]
fn connections(
    listener: TcpListener,
    service: Arc<Service>,
    stopped: std::sync::mpsc::Receiver<()>,
) -> io::Result<()> {
    listener.set_nonblocking(true)?;
    let mut workers: Vec<thread::JoinHandle<()>> = Vec::new();
    let max_workers = service.max_connections * 2 + 1;
    let readers = admission::Slots::new(service.max_connections + 1);
    loop {
        if stopped.try_recv().is_ok() {
            break;
        }
        let mut stream = match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false)?;
                stream
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                match stopped.recv_timeout(ACCEPT_POLL) {
                    Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        let mut retained = Vec::with_capacity(workers.len());
        for worker in workers {
            if worker.is_finished() {
                let _ = worker.join();
            } else {
                retained.push(worker);
            }
        }
        workers = retained;
        // Retain a whole-worker bound while phase permits change hands or diagnostics run.
        let reader = if workers.len() < max_workers {
            readers.take()
        } else {
            None
        };
        let Some(reader) = reader else {
            record(wire::unavailable(&mut stream));
            continue;
        };
        let connection = service.connections.take();
        let service = service.clone();
        workers.push(
            thread::Builder::new()
                .name("mcp-http".into())
                .spawn(move || {
                    record(service.handle(stream, reader, connection));
                })?,
        );
    }
    drop(listener);
    service.calls.cancel();
    for worker in workers {
        worker
            .join()
            .map_err(|_| io::Error::other("HTTP worker stopped"))?;
    }
    Ok(())
}
fn record(result: io::Result<()>) {
    if let Err(error) = result {
        crate::output::diagnostic(
            format!("biomcp: HTTP connection failed ({:?})\n", error.kind()).as_bytes(),
        );
    }
}
impl Service {
    fn handle(
        &self,
        mut stream: TcpStream,
        reader: admission::Permit,
        connection: Option<admission::Permit>,
    ) -> io::Result<()> {
        // A malformed or oversized framing closes without reflecting untrusted headers.
        let request = wire::read(&mut stream)?;
        // A reserved reader classifies health even when ordinary connections are full.
        let _connection = if request.path == "/health" {
            drop(connection);
            None
        } else {
            let Some(permit) = connection else {
                return wire::unavailable(&mut stream);
            };
            drop(reader);
            Some(permit)
        };
        let (status, value, revision) = self.route(&request);
        let body = value
            .map(|v| serde_json::to_vec(&v))
            .transpose()?
            .unwrap_or_default();
        let allow = (status == 405).then_some(if request.path == "/health" {
            "GET"
        } else {
            "POST"
        });
        wire::write(&mut stream, status, &body, revision, allow)
    }
    fn route(&self, request: &wire::Request) -> (u16, Option<Value>, Option<&'static str>) {
        if !self.allowed(request) {
            return (403, Some(json!({"error":"Forbidden"})), None);
        }
        if request.path == "/health" {
            return if request.method == "GET" {
                self.health()
            } else {
                (405, Some(json!({"error":"Method not allowed"})), None)
            };
        }
        if request.path != "/mcp" {
            return (404, Some(json!({"error":"Not found"})), None);
        }
        if request.method != "POST" {
            return (405, Some(json!({"error":"Method not allowed"})), None);
        }
        let value = match serde_json::from_slice::<Value>(&request.body) {
            Ok(value) => value,
            Err(_) => {
                return (
                    400,
                    Some(super::protocol::error(Value::Null, -32700, "Parse error")),
                    None,
                );
            }
        };
        let revision = match metadata::validate(request, &value) {
            Ok(revision) => revision,
            Err(error) => return (400, Some(error), None),
        };
        let declared_use = if self.globals.hosted {
            match request.declared_use() {
                Ok(usage) => Some(usage),
                Err(_) => {
                    return (
                        400,
                        Some(
                            json!({"error":"Invalid x-biomcp-use header; allowed values: restricted, commercial, academic"}),
                        ),
                        Some(revision),
                    );
                }
            }
        } else {
            None
        };
        let mut legacy = revision != super::protocol::REVISIONS[0];
        let result = match super::rpc::message(&request.body, &mut legacy, true, &self.root) {
            super::rpc::Message::Reply(reply) => reply,
            super::rpc::Message::Notification { .. } => return (202, None, Some(revision)),
            super::rpc::Message::Tool { id, params, modern } => {
                return self.tool(request, id, &params, modern, revision, declared_use);
            }
        };
        let status = if result.pointer("/error/code") == Some(&json!(-32601)) {
            404
        } else {
            200
        };
        (status, Some(result), Some(revision))
    }
    fn tool(
        &self,
        request: &wire::Request,
        id: Value,
        params: &Value,
        modern: bool,
        revision: &'static str,
        declared_use: Option<crate::command::DeclaredUse>,
    ) -> (u16, Option<Value>, Option<&'static str>) {
        let scope = self
            .globals
            .hosted
            .then(|| biomcp_fetch::Cancellation::with_budget(crate::config::BUDGET));
        let Some(call) = super::research::parse(params) else {
            return (
                200,
                Some(super::protocol::error(id, -32602, "Invalid params")),
                Some(revision),
            );
        };
        let keys = if self.globals.hosted {
            let keys = match request.source_keys() {
                Ok(keys) => keys,
                Err(_) => {
                    return (
                        400,
                        Some(json!({"error":"Invalid source key headers"})),
                        Some(revision),
                    );
                }
            };
            Some(keys)
        } else {
            None
        };
        let result = if let Some(scope) = scope {
            self.calls.run(
                self,
                HostedCall {
                    id,
                    modern,
                    call,
                    secrets: keys,
                    declared_use: declared_use.unwrap_or(crate::command::DeclaredUse::Restricted),
                    scope,
                },
            )
        } else {
            match super::research::execute(
                &call,
                crate::command::Context {
                    globals: &self.globals,
                    root: &self.root,
                    settings: &self.settings,
                    fetch: &self.fetch,
                    recovery: self.recovery.as_deref(),
                    secrets: keys.as_ref(),
                    declared_use,
                    scope: None,
                },
                true,
            ) {
                Ok(outcome) => super::protocol::tool_result(id, &outcome, modern),
                Err(_) => super::protocol::error(id, -32603, "Internal error"),
            }
        };
        (200, Some(result), Some(revision))
    }
    fn allowed(&self, request: &wire::Request) -> bool {
        if !matches!(request.one("origin"), Ok(None)) {
            return false;
        }
        let Ok(Some(host)) = request.one("host") else {
            return false;
        };
        let Some(host) = Authority::parse(host) else {
            return false;
        };
        // Absolute-form targets are never accepted; no Forwarded header affects admission.
        request.path.starts_with('/')
            && !request.path.starts_with("//")
            && (self.address.ip().is_loopback() && host.loopback(self.address)
                || self.hosts.iter().any(|allowed| allowed.permits(&host)))
    }
    fn health(&self) -> (u16, Option<Value>, Option<&'static str>) {
        let healthy = self
            .fetch
            .as_ref()
            .is_ok_and(|fetch| biomcp_query::store::health(fetch, HEALTH_BUDGET).is_ok());
        // Build the result through its public schema type, without source traffic.
        let value = biodata::Text::new(env!("CARGO_PKG_VERSION"))
            .ok()
            .and_then(|version| {
                serde_json::to_value(biomcp_types::HttpHealth {
                    version,
                    store: healthy,
                })
                .ok()
            });
        (if healthy { 200 } else { 503 }, value, None)
    }
}
