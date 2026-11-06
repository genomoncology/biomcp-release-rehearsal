//! MCP HTTP through the actual process and isolated local sockets.
use super::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    process::Child,
    time::Duration,
};
#[path = "http_start.rs"]
mod start;
type Check = Result<(), String>;
tool_only! {
pub(super) fn readiness_proof() -> Check {
    start::readiness_proof()
}
}
pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial(crate::runner::pool_contract::HTTP_CAPACITY.0, || {
            super::http_capacity::check().map_err(Into::into)
        }),
        #[cfg(unix)]
        transcript_trial("mcp-http/ignored-interrupt-keeps-serving", || {
            start::ignored_interrupt().map_err(Into::into)
        }),
        transcript_trial("mcp-http/local-health-and-discovery", || {
            lifecycle().map_err(Into::into)
        }),
        transcript_trial(
            "mcp-http/revisions-parity-and-concurrent-identities",
            || protocols().map_err(Into::into),
        ),
        transcript_trial(crate::runner::pool_contract::HTTP_HOST.0, || {
            admission().map_err(Into::into)
        }),
        transcript_trial("mcp-http/framing-resource-deadlines-and-shutdown", || {
            super::http_limits::check().map_err(Into::into)
        }),
        transcript_trial("mcp-http/storage-refusal-and-nonlocal-bind", || {
            storage().map_err(Into::into)
        }),
    ]
}

#[expect(
    clippy::disallowed_types,
    reason = "Serialize diagnostic observations while HTTP callers share the server"
)]
pub(super) struct Server {
    pub(super) child: Child,
    pub(super) address: SocketAddr,
    _environment: Environment,
    pub(super) diagnostics: std::sync::Mutex<std::sync::mpsc::Receiver<String>>,
    diagnostic_reader: Option<std::thread::JoinHandle<()>>,
}
impl Drop for Server {
    fn drop(&mut self) {
        if self.stop().is_err() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(reader) = self.diagnostic_reader.take() {
            let _ = reader.join();
        }
    }
}
impl Server {
    pub(super) fn stop(&mut self) -> Check {
        self.shutdown(false)
    }
    fn shutdown(&mut self, _terminate: bool) -> Check {
        if self.child.try_wait().map_err(|e| e.to_string())?.is_some() {
            return Ok(());
        }
        #[cfg(unix)]
        {
            let pid = nix::unistd::Pid::from_raw(
                self.child.id().try_into().map_err(|_| "HTTP process ID")?,
            );
            nix::sys::signal::kill(
                pid,
                if _terminate {
                    nix::sys::signal::Signal::SIGTERM
                } else {
                    nix::sys::signal::Signal::SIGINT
                },
            )
            .map_err(|e| e.to_string())?;
        }
        // Native Windows console Ctrl-C is exercised by the platform rehearsal.
        #[cfg(windows)]
        self.child.kill().map_err(|e| e.to_string())?;
        let end = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = self.child.try_wait().map_err(|e| e.to_string())? {
                if cfg!(unix) && !status.success() {
                    return Err(format!("HTTP shutdown: {status}"));
                }
                return Ok(());
            }
            if std::time::Instant::now() >= end {
                return Err("HTTP shutdown deadline".into());
            }
            std::thread::yield_now();
        }
    }
    pub(super) fn start() -> Result<Self, String> {
        Self::configured("127.0.0.1:0", None, true)
    }
    pub(super) fn environment(&self) -> &Environment {
        &self._environment
    }
    pub(super) fn keys(config: Value, key: Option<(&str, &str)>) -> Result<Self, String> {
        Self::configured_key("127.0.0.1:0", Some(config), None, key, None)
    }
    fn configured(bind: &str, config: Option<Value>, replay: bool) -> Result<Self, String> {
        Self::configured_key(
            bind,
            config,
            replay.then(|| super::launch::fixtures().join("replies/gene/main")),
            None,
            None,
        )
    }
    pub(super) fn start_replay(path: std::path::PathBuf) -> Result<Self, String> {
        Self::configured_key("127.0.0.1:0", None, Some(path), None, None)
    }

    pub(super) fn request(
        &self,
        method: &str,
        path: &str,
        body: &str,
    ) -> Result<(u16, Value), String> {
        self.request_with_headers(method, path, body, "")
    }
    pub(super) fn request_with_headers(
        &self,
        method: &str,
        path: &str,
        body: &str,
        extra: &str,
    ) -> Result<(u16, Value), String> {
        let value = serde_json::from_str::<Value>(body).ok();
        let method_header = value
            .as_ref()
            .and_then(|v| v.get("method"))
            .and_then(Value::as_str)
            .map(|method| format!("Mcp-Method: {method}\r\n"))
            .unwrap_or_default();
        let name_header = value
            .as_ref()
            .and_then(|v| v.pointer("/params/name"))
            .and_then(Value::as_str)
            .map(|name| format!("Mcp-Name: {name}\r\n"))
            .unwrap_or_default();
        let headers = format!(
            "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: \
        2026-07-28\r\n{method_header}{name_header}{extra}"
        );
        let (status, body, _) =
            self.raw(method, path, &self.address.to_string(), &headers, body)?;
        Ok((status, body.unwrap_or(Value::Null)))
    }
    pub(super) fn raw(
        &self,
        method: &str,
        path: &str,
        host: &str,
        headers: &str,
        body: &str,
    ) -> Result<(u16, Option<Value>, String), String> {
        let mut socket = TcpStream::connect(self.address).map_err(|e| e.to_string())?;
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| e.to_string())?;
        let wire = format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}\r\n{headers}Connection: close\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        socket
            .write_all(wire.as_bytes())
            .map_err(|e| e.to_string())?;
        let mut bytes = Vec::new();
        // A refusal closes without reading caller bytes, which can end with TCP reset.
        if let Err(error) = socket.take(1024 * 1024).read_to_end(&mut bytes)
            && error.kind() != std::io::ErrorKind::ConnectionReset
        {
            return Err(error.to_string());
        }
        let response = String::from_utf8(bytes).map_err(|e| e.to_string())?;
        let (headers, body) = response
            .split_once("\r\n\r\n")
            .ok_or("missing HTTP headers")?;
        let length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .ok_or("missing response length")?;
        if body.len() != length {
            return Err("incomplete HTTP response body".into());
        }
        let status = headers
            .split_whitespace()
            .nth(1)
            .ok_or("missing status")?
            .parse()
            .map_err(|_| "bad status")?;
        Ok((
            status,
            if body.is_empty() {
                None
            } else {
                Some(serde_json::from_str(body).map_err(|e| format!("HTTP JSON body: {e}"))?)
            },
            headers.to_owned(),
        ))
    }
}
pub(super) fn lifecycle() -> Check {
    let mut server = Server::start()?;
    let (status, health) = super::http_limits::read_only_health(&server)?;
    if status != 200 || health.get("version") != Some(&json!(env!("CARGO_PKG_VERSION"))) {
        return Err(format!("local health: {status} {health}"));
    }
    let schema = super::schema::ARTIFACTS
        .iter()
        .find(|(id, _)| *id == "biomcp.http-health/2")
        .ok_or("HTTP health schema missing")?
        .1;
    let schema: Value = serde_json::from_str(schema).map_err(|e| e.to_string())?;
    jsonschema::validator_for(&schema)
        .map_err(|e| e.to_string())?
        .validate(&health)
        .map_err(|e| e.to_string())?;
    let request = json!({"jsonrpc":"2.0","id":1,"method":"server/discover","params":{
        "_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28",
        "io.modelcontextprotocol/clientCapabilities":{}}}});
    let (status, reply) = server.request("POST", "/mcp", &request.to_string())?;
    if status != 200
        || reply.pointer("/result/supportedVersions")
            != Some(&json!([
                "2026-07-28",
                "2025-11-25",
                "2025-06-18",
                "2025-03-26"
            ]))
    {
        return Err(format!("HTTP discover: {status} {reply}"));
    }
    server.stop()
}

fn legacy(id: u32, method: &str, params: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
}
fn legacy_headers() -> &'static str {
    "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-11-25\r\n"
}
fn protocols() -> Check {
    let server = Server::start()?;
    legacy_versions(&server)?;
    let discover = super::mcp::request(json!(7), "server/discover", json!({}));
    server.request("POST", "/mcp", &discover.to_string())?;
    let list = legacy(7, "tools/list", json!({}));
    let host = server.address.to_string();
    let body = list.to_string();
    let (status, reply, _) = server.raw("POST", "/mcp", &host, legacy_headers(), &body)?;
    if status != 200
        || reply
            .as_ref()
            .and_then(|v| v.pointer("/result/resultType"))
            .is_some()
    {
        return Err("modern client changed legacy request state".into());
    }
    let args: Vec<String> = ["get", "gene", "BRAF"].map(str::to_owned).into();
    let globals = server._environment.globals(
        Some(&super::launch::fixtures().join("replies/gene/main")),
        false,
    );
    let expected = super::mcp::call_with_globals(&server._environment, &args, false, &globals)?;
    let call = super::mcp::request(
        json!(7),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":args}}),
    );
    let (status, reply) = server.request("POST", "/mcp", &call.to_string())?;
    if status != 200 || reply.pointer("/result/structuredContent") != Some(&expected) {
        return Err("replayed HTTP call lost stdio parity".into());
    }
    concurrent(&server, &globals)
}
fn legacy_versions(server: &Server) -> Check {
    let base = "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\n";
    let host = server.address.to_string();
    for revision in ["2025-11-25", "2025-06-18", "2025-03-26"] {
        let explicit = format!("MCP-Protocol-Version: {revision}\r\n");
        for version_header in [explicit, String::new()] {
            let headers = format!("{base}{version_header}");
            let init = json!({"jsonrpc":"2.0","id":7,"method":"initialize","params":{
                "protocolVersion":revision,"capabilities":{},"clientInfo":{"name":"local-proof","version":"1"}}});
            let call = json!({"jsonrpc":"2.0","id":8,"method":"tools/call",
                "params":{"name":"biomcp","arguments":{"args":["help"]}}});
            for (request, pointer, expected) in [
                (init, "/result/protocolVersion", json!(revision)),
                (call, "/result/isError", json!(false)),
            ] {
                let fallback = version_header.is_empty() && pointer == "/result/isError";
                let transport_revision = if fallback { "2025-03-26" } else { revision };
                let body = request.to_string();
                let (status, reply, response) =
                    server.raw("POST", "/mcp", &host, &headers, &body)?;
                if status != 200
                    || reply.as_ref().and_then(|v| v.pointer(pointer)) != Some(&expected)
                    || !response
                        .contains(&format!("MCP-Protocol-Version: {transport_revision}\r\n"))
                    || response.to_ascii_lowercase().contains("session-id")
                {
                    return Err(format!("legacy {revision}: {status} {reply:?}"));
                }
            }
        }
    }
    let headers = format!("{base}MCP-Protocol-Version: 2099-01-01\r\n");
    let body = legacy(9, "tools/list", json!({})).to_string();
    let (status, reply, _) = server.raw("POST", "/mcp", &host, &headers, &body)?;
    if status != 400
        || reply.as_ref().and_then(|v| v.pointer("/error/code")) != Some(&json!(-32022))
    {
        return Err(format!("unsupported revision: {status} {reply:?}"));
    }
    Ok(())
}
fn concurrent(server: &Server, globals: &[String]) -> Check {
    // Distinct replies expose cross-client ID and protocol state mistakes.
    let modern_args = vec!["help".to_string()];
    let old_args = vec!["help".to_string(), "get".into(), "gene".into()];
    let modern_expected =
        super::mcp::call_with_globals(&server._environment, &modern_args, false, globals)?;
    let old_expected =
        super::mcp::call_with_globals(&server._environment, &old_args, false, globals)?;
    let modern = super::mcp::request(
        json!(7),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":modern_args}}),
    );
    let old = legacy(
        7,
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":old_args}}),
    );
    let barrier = std::sync::Barrier::new(3);
    let (modern, old, health) = std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            barrier.wait();
            server.request("POST", "/mcp", &modern.to_string())
        });
        let b = scope.spawn(|| {
            barrier.wait();
            server.raw(
                "POST",
                "/mcp",
                &server.address.to_string(),
                legacy_headers(),
                &old.to_string(),
            )
        });
        let health = scope.spawn(|| {
            barrier.wait();
            server.request("GET", "/health", "")
        });
        Ok::<_, String>((
            a.join().map_err(|_| "modern HTTP caller stopped")?,
            b.join().map_err(|_| "legacy HTTP caller stopped")?,
            health.join().map_err(|_| "HTTP health caller stopped")?,
        ))
    })?;
    let (health_status, health) = health?;
    if health_status != 200 || health.get("store") != Some(&json!(true)) {
        return Err("concurrent local store health failed".into());
    }
    let (modern_status, modern) = modern?;
    let (old_status, old, _) = old?;
    let old = old.ok_or("legacy response missing")?;
    if modern_status != 200
        || old_status != 200
        || modern.pointer("/result/structuredContent") != Some(&modern_expected)
        || old.pointer("/result/structuredContent") != Some(&old_expected)
        || modern.pointer("/result/resultType") != Some(&json!("complete"))
        || old.pointer("/result/resultType").is_some()
    {
        return Err("concurrent equal IDs crossed client results or protocol state".into());
    }
    Ok(())
}

fn admission() -> Check {
    let server = Server::configured_key("127.0.0.1:0", None, None, None, Some("clinic.example"))?;
    if server.raw("GET", "/health", "clinic.example", "", "")?.0 != 200 {
        return Err("explicit allowed Host was rejected".into());
    }
    metadata_cases(&server)?;
    let host = server.address.to_string();
    for (authority, headers) in [
        ("foreign.example", ""),
        ("localhost.evil", ""),
        ("localhost:0", ""),
        ("user@localhost", ""),
        ("[::1]extra", ""),
        ("localhost:65536", ""),
        ("localhost:+8080", ""),
        (&host, "Host: foreign.example\r\n"),
        (&host, "Origin: http://localhost\r\n"),
        (&host, "Origin: null\r\n"),
        (&host, "Origin: invalid\r\nOrigin: invalid\r\n"),
    ] {
        for path in ["/health", "/missing", "/mcp"] {
            let (status, _, response) = server.raw("GET", path, authority, headers, "")?;
            if status != 403
                || response
                    .to_ascii_lowercase()
                    .contains("access-control-allow")
            {
                return Err(format!(
                    "authority/origin admitted: {authority} {headers:?} {status}"
                ));
            }
        }
    }
    let forwarded = "Forwarded: host=foreign.example\r\nX-Forwarded-Host: foreign.example\r\n";
    if server.raw("GET", "/health", &host, forwarded, "")?.0 != 200 {
        return Err("forwarded headers affected admission".into());
    }
    for method in ["GET", "DELETE", "OPTIONS", "PUT"] {
        let (status, _, headers) = server.raw(method, "/mcp", &host, "", "")?;
        if status != 405 || !headers.contains("Allow: POST\r\n") {
            return Err(format!("method {method} admitted or missing Allow"));
        }
    }
    let request = super::mcp::request(json!(1), "tools/list", json!({}));
    for headers in [
        "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2026-07-28\r\n",
        "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2026-07-28\r\nMcp-Method: ping\r\n",
        "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-11-25\r\n",
    ] {
        let (status, reply, _) =
            server.raw("POST", "/mcp", &host, headers, &request.to_string())?;
        if status != 400
            || reply.as_ref().and_then(|v| v.pointer("/error/code")) != Some(&json!(-32020))
        {
            return Err("HTTP metadata mismatch was not rejected".into());
        }
    }
    let notification = json!({"jsonrpc":"2.0","method":"notifications/initialized"});
    let (status, reply, _) = server.raw(
        "POST",
        "/mcp",
        &host,
        legacy_headers(),
        &notification.to_string(),
    )?;
    if status != 202 || reply.is_some() {
        return Err("notification must have an empty 202 response".into());
    }
    Ok(())
}
fn storage() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    for args in [
        vec!["serve", "--http", "0.0.0.0:0"],
        vec!["serve", "--http", "127.0.0.1:0", "--allow-host", "bad/path"],
    ] {
        let args: Vec<String> = args.into_iter().map(str::to_owned).collect();
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &args)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.code() != Some(2) {
            return Err("unsafe bind must fail with usage status".into());
        }
    }
    let failed = Server::configured(
        "127.0.0.1:0",
        Some(json!({"ca_bundle":"missing-private-ca.pem"})),
        false,
    )?;
    let (status, body, _) = failed.raw("GET", "/health", &failed.address.to_string(), "", "")?;
    if status != 503 || body != Some(json!({"version":env!("CARGO_PKG_VERSION"),"store":false})) {
        return Err("local initialization failure leaked details or passed health".into());
    }
    let ipv6 = Server::configured("[::1]:0", None, true)?;
    let authority = format!("[0:0:0:0:0:0:0:1]:{}", ipv6.address.port());
    if ipv6.raw("GET", "/health", &authority, "", "")?.0 != 200 {
        return Err("equivalent IPv6 authority was rejected".into());
    }
    let mut exposed = Server::configured(
        "0.0.0.0:0",
        Some(json!({"http":{"allowed_hosts":["local.example"]}})),
        true,
    )?;
    exposed.address = SocketAddr::new(
        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        exposed.address.port(),
    );
    if exposed.raw("GET", "/health", "LOCAL.EXAMPLE", "", "")?.0 != 200
        || exposed.raw("GET", "/health", "other.example", "", "")?.0 != 403
    {
        return Err("configured host allowlist was not enforced".into());
    }
    Ok(())
}

fn metadata_cases(server: &Server) -> Check {
    let request = super::mcp::request(json!(9), "tools/list", json!({}));
    for (accept, expected) in [
        ("Application/JSON, Text/Event-Stream", 200),
        ("application/*, text/*", 200),
        ("*/*;q=1, application/json;q=0, text/event-stream", 400),
        ("*/*;q=0, application/json, text/event-stream", 200),
        ("application/*;q=0, */*;q=1, application/json;q=1", 200),
        ("application/json; q=1.000, text/event-stream; q=0.5", 200),
        ("application/json; q=0.001, text/event-stream", 200),
        ("application/json; q=0, text/event-stream", 400),
        ("application/json; q=1.001, text/event-stream", 400),
        ("application/json; q=0.0001, text/event-stream", 400),
    ] {
        let headers = format!(
            "Content-Type: application/json\r\nAccept: {accept}\r\nMCP-Protocol-Version: 2026-07-28\r\nMcp-Method: tools/list\r\n"
        );
        let (status, _, _) = server.raw(
            "POST",
            "/mcp",
            &server.address.to_string(),
            &headers,
            &request.to_string(),
        )?;
        if status != expected {
            return Err(format!(
                "Accept media range {accept:?}: expected {expected}, received {status}"
            ));
        }
    }
    let request = super::mcp::request(
        json!(9),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":["help"]}}),
    );
    let prefix = "Content-Type: application/json\r\nAccept: application/json, \
        text/event-stream\r\nMCP-Protocol-Version: 2026-07-28\r\nMcp-Method: tools/call\r\n";
    for (name, expected) in [
        ("=?base64?YmlvbWNw?=", 200),
        ("=?base64?***?=", 400),
        ("other", 400),
    ] {
        let headers = format!("{prefix}Mcp-Name: {name}\r\n");
        let (status, reply, _) = server.raw(
            "POST",
            "/mcp",
            &server.address.to_string(),
            &headers,
            &request.to_string(),
        )?;
        if status != expected
            || expected == 400
                && reply.as_ref().and_then(|v| v.pointer("/error/code")) != Some(&json!(-32020))
        {
            return Err("encoded tool name did not match its body".into());
        }
    }
    let request = super::mcp::request(json!(9), "unavailable-method", json!({}));
    let (status, reply) = server.request("POST", "/mcp", &request.to_string())?;
    if status != 404 || reply.pointer("/error/code") != Some(&json!(-32601)) {
        return Err("unsupported modern method needs HTTP 404 and RPC -32601".into());
    }
    let headers = "Content-Type: application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2024-11-05\r\n";
    let (status, reply, _) = server.raw(
        "POST",
        "/mcp",
        &server.address.to_string(),
        headers,
        &request.to_string(),
    )?;
    if status != 400
        || reply.as_ref().and_then(|v| v.pointer("/error/code")) != Some(&json!(-32022))
    {
        return Err("unsupported protocol version was not reported".into());
    }
    Ok(())
}
