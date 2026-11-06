//! Stdio scheduling and the single response writer.
mod current;
mod framing;
mod hosted;
pub(crate) mod http;
mod legacy;
mod output_schema;
mod protocol;
mod research;
mod rpc;
mod typed;
mod worker;

use framing::Frame;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{self, Write},
    sync::mpsc,
    thread,
};

const MAX_ACTIVE_CALLS: usize = 8;
const MAX_RETAINED_EVENTS: usize = 32;

enum Event {
    Input(Frame),
    Completed(String, io::Result<crate::command::Outcome>),
}

struct Call {
    args: Vec<std::ffi::OsString>,
    id: Value,
    modern: bool,
    cancelled: bool,
    cancellation: Option<biomcp_fetch::Cancellation>,
    worker: thread::JoinHandle<()>,
}

struct Server {
    sender: mpsc::SyncSender<Event>,
    active: BTreeMap<String, Call>,
    legacy: bool,
    globals: crate::grammar::Globals,
    root: std::sync::Arc<crate::grammar::Root>,
    settings: std::sync::Arc<crate::config::Settings>,
    fetch: Result<std::sync::Arc<biomcp_fetch::Fetch>, crate::config::StartupError>,
    recovery: Option<std::sync::Arc<Recovery>>,
}

pub(crate) fn serve(
    globals: crate::grammar::Globals,
    fetch: Result<biomcp_fetch::Fetch, crate::config::StartupError>,
    root: std::sync::Arc<crate::grammar::Root>,
    settings: std::sync::Arc<crate::config::Settings>,
) -> io::Result<()> {
    let (sender, receiver) = mpsc::sync_channel(MAX_RETAINED_EVENTS);
    start_input(sender.clone())?;
    let recovery = fetch.is_err().then(|| std::sync::Arc::new(Recovery::new()));
    let mut server = Server {
        sender,
        active: BTreeMap::new(),
        legacy: false,
        globals,
        root,
        settings,
        fetch: fetch.map(|fetch| std::sync::Arc::new(fetch.with_background_refresh())),
        recovery,
    };
    let mut output = io::stdout().lock();
    let mut eof = false;
    while !eof || !server.active.is_empty() {
        let event = receiver
            .recv()
            .map_err(|_| io::Error::other("MCP input stopped"))?;
        let reply = match event {
            Event::Input(Frame::Eof) => {
                eof = true;
                None
            }
            Event::Input(Frame::Malformed) => {
                eof = true;
                Some(protocol::error(Value::Null, -32700, "Parse error"))
            }
            Event::Input(Frame::Oversized) => {
                Some(protocol::error(Value::Null, -32600, "Frame too large"))
            }
            Event::Input(Frame::Message(frame)) => server.message(&frame)?,
            Event::Completed(key, outcome) => server.complete(&key, outcome)?,
        };
        if let Some(reply) = reply {
            let mut bytes = serde_json::to_vec(&reply)?;
            bytes.push(b'\n');
            output.write_all(&bytes)?;
            output.flush()?;
        }
    }
    // The input thread may still be in a blocking stdin read after output breaks.
    // Dropping its handle lets main return without joining that read.
    Ok(())
}

#[expect(
    clippy::disallowed_methods,
    reason = "Stdio input owns the bounded frame channel"
)]
fn start_input(sender: mpsc::SyncSender<Event>) -> io::Result<()> {
    thread::Builder::new()
        .name("mcp-input".into())
        .spawn(move || {
            let mut input = io::stdin().lock();
            loop {
                let frame = framing::read(&mut input).unwrap_or(Frame::Malformed);
                let terminal = matches!(frame, Frame::Malformed | Frame::Eof);
                if sender.send(Event::Input(frame)).is_err() || terminal {
                    break;
                }
            }
        })?;
    Ok(())
}

impl Server {
    fn message(&mut self, frame: &[u8]) -> io::Result<Option<Value>> {
        match rpc::message(
            frame,
            &mut self.legacy,
            self.globals.hosted,
            crate::command::Context::new(&self.globals, &self.fetch, &self.root, &self.settings),
            crate::config::mcp_tool_mode(&self.globals, &self.settings),
        ) {
            rpc::Message::Reply(reply) => Ok(Some(reply)),
            rpc::Message::Tool { id, params, modern } => self.admit(id, &params, modern),
            rpc::Message::Notification { method, params } => {
                if method == "notifications/cancelled"
                    && let Some(id) = params.get("requestId").filter(|id| protocol::valid_id(id))
                    && let Some(call) = self.active.get_mut(&id.to_string())
                {
                    call.cancelled = true;
                    if let Some(cancellation) = &call.cancellation {
                        cancellation.cancel();
                    }
                }
                Ok(None)
            }
        }
    }

    fn admit(&mut self, id: Value, params: &Value, modern: bool) -> io::Result<Option<Value>> {
        let Some(call) = typed::parse(
            &self.root,
            params,
            crate::config::mcp_tool_mode(&self.globals, &self.settings),
            self.globals.fresh,
        ) else {
            return Ok(Some(protocol::error(id, -32602, "Invalid params")));
        };
        let call = match call {
            Ok(call) => call,
            Err(rejected) => {
                return typed::rejection(
                    id,
                    rejected,
                    modern,
                    crate::command::Context::new(
                        &self.globals,
                        &self.fetch,
                        &self.root,
                        &self.settings,
                    ),
                )
                .map(Some);
            }
        };
        let key = id.to_string();
        if self.active.contains_key(&key) {
            return Ok(Some(protocol::error(id, -32600, "Request already active")));
        }
        if self.active.len() >= MAX_ACTIVE_CALLS {
            return Ok(Some(protocol::error(id, -31000, "Tool capacity exceeded")));
        }
        let globals = self.globals.clone();
        let root = self.root.clone();
        let settings = self.settings.clone();
        let recovery = self.recovery.clone();
        let (fetch, cancellation) = match &self.fetch {
            Ok(fetch) => {
                let Ok((fetch, cancellation)) = fetch.cancellable() else {
                    return Ok(Some(protocol::error(id, -32603, "Internal error")));
                };
                (Ok(std::sync::Arc::new(fetch)), Some(cancellation))
            }
            Err(error) => (
                Err(error.clone()),
                Some(biomcp_fetch::Cancellation::with_budget(
                    crate::config::budget(&settings),
                )),
            ),
        };
        let scope = recovery.as_ref().and(cancellation.clone());
        let original_args = call.args.clone();
        self.start_call(key.clone(), id, modern, cancellation, move || {
            let mut context = crate::command::Context::new(&globals, &fetch, &root, &settings);
            context.recovery = recovery.as_deref();
            context.scope = scope.as_ref();
            research::execute(&call, context, false)
        })?;
        if let Some(call) = self.active.get_mut(&key) {
            call.args = original_args;
        }
        Ok(None)
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "Bounded tool workers own command execution"
    )]
    fn start_call(
        &mut self,
        key: String,
        id: Value,
        modern: bool,
        cancellation: Option<biomcp_fetch::Cancellation>,
        work: impl FnOnce() -> io::Result<crate::command::Outcome> + Send + 'static,
    ) -> io::Result<()> {
        let sender = self.sender.clone();
        let worker_key = key.clone();
        let worker = thread::Builder::new()
            .name("mcp-tool".into())
            .spawn(move || {
                let outcome = worker::execute(work);
                let _ = sender.send(Event::Completed(worker_key, outcome));
            })?;
        self.active.insert(
            key,
            Call {
                args: Vec::new(),
                id,
                modern,
                cancelled: false,
                cancellation,
                worker,
            },
        );
        Ok(())
    }

    fn complete(
        &mut self,
        key: &str,
        outcome: io::Result<crate::command::Outcome>,
    ) -> io::Result<Option<Value>> {
        let Some(call) = self.active.remove(key) else {
            return Ok(None);
        };
        call.worker
            .join()
            .map_err(|_| io::Error::other("MCP worker stopped"))?;
        if call.cancelled {
            return Ok(None);
        }
        Ok(Some(match outcome {
            Ok(outcome) => typed::tool_result(
                call.id,
                &outcome,
                call.modern,
                &self.globals,
                &self.settings,
                &self.root,
                &call.args,
            ),
            Err(_) => protocol::error(call.id, -32603, "Internal error"),
        }))
    }
}

/// A failed startup may publish exactly one successful static transport snapshot.
#[derive(Clone)]
pub(crate) struct Snapshot {
    pub(crate) fetch: std::sync::Arc<biomcp_fetch::Fetch>,
    pub(crate) settings: std::sync::Arc<crate::config::Settings>,
}
#[expect(
    clippy::disallowed_types,
    reason = "Serialize failed startup retries until one shared transport snapshot succeeds"
)]
pub(crate) struct Recovery(std::sync::Mutex<Option<Snapshot>>);
impl Recovery {
    #[expect(
        clippy::disallowed_types,
        reason = "The MCP setup owner alone publishes the first successful static snapshot"
    )]
    pub(crate) fn new() -> Self {
        Self(std::sync::Mutex::new(None))
    }
    pub(crate) fn snapshot(
        &self,
        globals: &crate::grammar::Globals,
    ) -> Result<(Snapshot, bool), crate::config::StartupError> {
        let mut stored = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(snapshot) = stored.as_ref() {
            return Ok((snapshot.clone(), false));
        }
        let settings = std::sync::Arc::new(crate::config::file::read_file(globals.config.as_ref()));
        let fetch = std::sync::Arc::new(
            crate::config::fetch(globals, &settings)?.with_background_refresh(),
        );
        let snapshot = Snapshot { fetch, settings };
        *stored = Some(snapshot.clone());
        Ok((snapshot, true))
    }
}
