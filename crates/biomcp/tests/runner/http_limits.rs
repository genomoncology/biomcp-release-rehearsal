//! Hostile HTTP framing, request budgets, connection capacity and process ownership.
use super::http::Server;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::{Duration, Instant},
};
type Check = Result<(), String>;
fn closed(address: SocketAddr, data: &[u8]) -> Check {
    let mut socket = TcpStream::connect(address).map_err(|e| e.to_string())?;
    socket
        .set_read_timeout(Some(Duration::from_secs(4)))
        .map_err(|e| e.to_string())?;
    socket.write_all(data).map_err(|e| e.to_string())?;
    let mut byte = [0];
    match super::socket_read::read(&mut socket, &mut byte) {
        Ok(0) => Ok(()),
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe
            ) =>
        {
            Ok(())
        }
        other => Err(format!("rejected framing did not close: {other:?}")),
    }
}
pub(super) fn check() -> Check {
    chunked().map_err(|error| format!("chunked: {error}"))?;
    overload().map_err(|error| format!("overload: {error}"))?;
    dropped_peer().map_err(|error| format!("dropped peer: {error}"))?;
    #[cfg(unix)]
    busy_health().map_err(|error| format!("busy health: {error}"))?;
    #[cfg(unix)]
    configured_capacity().map_err(|error| format!("configured capacity: {error}"))?;
    let mut server = Server::start()?;
    for bytes in [
        format!(
            "POST /mcp HTTP/1.1\r\nHost: {}\r\nTransfer-Encoding: chunked\r\nContent-Length: 0\r\n\r\n",
            server.address
        ),
        format!(
            "POST /mcp HTTP/1.1\r\nHost: {}\r\nContent-Length: 65537\r\n\r\n",
            server.address
        ),
        format!(
            "POST /mcp HTTP/1.1\r\nHost: {}\r\nContent-Length: 1\r\nContent-Length: 1\r\n\r\nx",
            server.address
        ),
        format!(
            "POST /mcp HTTP/1.1\r\nHost: {}\r\nTransfer-Encoding: gzip\r\n\r\n0\r\n\r\n",
            server.address
        ),
        format!(
            "GET /health HTTP/1.1\r\nHost: {}\r\nX-Fill: {}",
            server.address,
            "x".repeat(16384)
        ),
    ] {
        closed(server.address, bytes.as_bytes())?;
    }
    slow_header(server.address)?;
    let mut held = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
    held.set_read_timeout(Some(Duration::from_secs(4)))
        .map_err(|e| e.to_string())?;
    server.stop()?;
    let mut byte = [0];
    match super::socket_read::read(&mut held, &mut byte) {
        Ok(0) => (),
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => (),
        other => return Err(format!("HTTP shutdown retained a client socket: {other:?}")),
    }
    if TcpStream::connect(server.address).is_ok() {
        return Err("shutdown left a listener".into());
    }
    Ok(())
}

fn slow_header(address: SocketAddr) -> Check {
    // wire::READ_BUDGET is two seconds from server admission. Start before
    // connecting so client setup consumes that same budget, not a new one.
    let connected = Instant::now();
    let mut slow = TcpStream::connect(address).map_err(|e| e.to_string())?;
    // Keep the socket timeout beyond the asserted bound so EOF proves closure.
    // The server budget is two seconds; allow scheduler delay under coverage.
    slow.set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    slow.write_all(b"GET /health HTTP/1.1\r\nX-Slow: ")
        .map_err(|e| e.to_string())?;
    let started = Instant::now();
    // Continuously send data while waiting for the server's close event. An idle
    // timeout would stay open forever; the absolute deadline must end the read.
    let copy = slow.try_clone().map_err(|e| e.to_string())?;
    let (stop, receiver) = std::sync::mpsc::channel::<()>();
    let writer = std::thread::spawn(move || {
        let mut copy = copy;
        while receiver.recv_timeout(Duration::from_millis(100)).is_err() {
            if copy.write_all(b"x").is_err() {
                break;
            }
        }
    });
    let mut byte = [0];
    let result = super::socket_read::read(&mut slow, &mut byte);
    // Measure the observed read event before stopping and joining the writer.
    let close_elapsed = started.elapsed();
    let connection_elapsed = connected.elapsed();
    let cleanup_started = Instant::now();
    let _ = stop.send(());
    let _ = slow.shutdown(std::net::Shutdown::Both);
    let joined = writer.join();
    let cleanup_elapsed = cleanup_started.elapsed();
    let ended = match &result {
        Ok(0) => true,
        Err(error) => error.kind() == std::io::ErrorKind::ConnectionReset,
        _ => false,
    };
    if !ended
        || connection_elapsed < Duration::from_secs(2)
        || close_elapsed > Duration::from_secs(8)
        || joined.is_err()
    {
        return Err(format!(
            "slow header deadline: read={result:?}, close_elapsed={close_elapsed:?}, \
             connection_elapsed={connection_elapsed:?}, cleanup_elapsed={cleanup_elapsed:?}, writer_panicked={}",
            joined.is_err()
        ));
    }
    Ok(())
}

tool_only! {
pub(super) fn early_close_proof() -> Check {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let address = listener.local_addr().map_err(|e| e.to_string())?;
    let peer = std::thread::spawn(move || -> std::io::Result<()> {
        let (mut socket, _) = listener.accept()?;
        socket.set_read_timeout(Some(Duration::from_secs(4)))?;
        super::socket_read::read(&mut socket, &mut [0; 1024])?;
        socket.shutdown(std::net::Shutdown::Both)
    });
    let result = slow_header(address);
    peer.join().map_err(|_| "early-close peer panicked")?
        .map_err(|e| e.to_string())?;
    match result {
        Err(error) if error.starts_with("slow header deadline:") => Ok(()),
        other => Err(format!("early closing peer was not rejected: {other:?}")),
    }
}
}

fn chunked() -> Check {
    let server = Server::start()?;
    let request = b"{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/list\",\
                    \"params\":{\"_meta\":{\"io.modelcontextprotocol/protocolVersion\":\"2026-07-28\",\"io.modelcontextprotocol/clientCapabilities\":{}}}}";
    let mut socket = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
    socket
        .set_read_timeout(Some(Duration::from_secs(4)))
        .map_err(|e| e.to_string())?;
    let head = format!(
        "POST /mcp HTTP/1.1\r\nHost: {}\r\nTransfer-Encoding: chunked\r\nContent-Type: \
            application/json\r\nAccept: application/json, text/event-stream\r\nMCP-Protocol-Version: \
            2026-07-28\r\nMcp-Method: tools/list\r\n\r\n",
        server.address
    );
    socket
        .write_all(head.as_bytes())
        .map_err(|e| e.to_string())?;
    for part in request.chunks(17) {
        let frame = format!(
            "{:x};source=local\r\n{}\r\n",
            part.len(),
            std::str::from_utf8(part).map_err(|e| e.to_string())?
        );
        socket
            .write_all(frame.as_bytes())
            .map_err(|e| e.to_string())?;
    }
    socket
        .write_all(b"0\r\nX-Ignored-Trailer: value\r\n\r\n")
        .map_err(|e| e.to_string())?;
    let mut reply = String::new();
    socket
        .read_to_string(&mut reply)
        .map_err(|e| e.to_string())?;
    let (head, body) = reply
        .split_once("\r\n\r\n")
        .ok_or("chunk response missing")?;
    let body: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if !head.starts_with("HTTP/1.1 200 ")
        || body.pointer("/result/tools/0/name") != Some(&serde_json::json!("biomcp"))
    {
        return Err("valid chunked MCP request was refused".into());
    }
    for frame in [
        "10001\r\n",
        "1ffffffffffffffff\r\n",
        "1\r\nxZ\r\n",
        "0\r\nBad-Trailer\r\n\r\n",
    ] {
        let wire = format!(
            "POST /mcp HTTP/1.1\r\nHost: {}\r\nTransfer-Encoding: chunked\r\n\r\n{frame}",
            server.address
        );
        closed(server.address, wire.as_bytes())?;
    }
    Ok(())
}
fn overload() -> Check {
    let server = Server::start()?;
    let mut held = Vec::new();
    // Held peers keep every reader busy only for the server's two-second read budget.
    let readers_free = std::time::Instant::now() + Duration::from_secs(2);
    // Framing can use the full existing worker bound (twice the connection cap plus one).
    for _ in 0..17 {
        let mut socket = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
        socket
            .write_all(b"GET /health HTTP/1.1\r\n")
            .map_err(|e| e.to_string())?;
        held.push(socket);
    }
    // A silent refused peer must not consume the read budget or delay the next accept.
    for _ in 0..2 {
        let mut excess = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
        // A test that reached this point too late has no overload left to observe.
        let Some(left) = readers_free
            .checked_duration_since(std::time::Instant::now())
            .filter(|left| !left.is_zero())
        else {
            return Ok(());
        };
        excess
            .set_read_timeout(Some(left))
            .map_err(|e| e.to_string())?;
        let mut response = String::new();
        excess
            .read_to_string(&mut response)
            .map_err(|e| e.to_string())?;
        if !response.starts_with("HTTP/1.1 503 ") {
            return Err(format!(
                "excess connection did not receive 503: {response:?}"
            ));
        }
    }
    drop(held);
    Ok(())
}

pub(super) fn read_only_health(server: &Server) -> Result<(u16, serde_json::Value), String> {
    let cache = server.environment().root().join("case-cache");
    let database = std::fs::read_dir(&cache)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "sqlite")
        })
        .ok_or("missing cache database")?;
    let connection = rusqlite::Connection::open(&database).map_err(|e| e.to_string())?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    let permissions = std::fs::metadata(&cache)
        .map_err(|e| e.to_string())?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cache, std::fs::Permissions::from_mode(0o555))
            .map_err(|e| e.to_string())?;
    }
    let response = server.request("GET", "/health", "");
    #[cfg(unix)]
    std::fs::set_permissions(&cache, permissions).map_err(|e| e.to_string())?;
    connection
        .execute_batch("ROLLBACK; INSERT INTO meta(owner,version) VALUES ('biomcp-query',4)")
        .map_err(|e| e.to_string())?;
    let (status, old) = server.request("GET", "/health", "")?;
    let version: u32 = connection
        .query_row(
            "SELECT version FROM meta WHERE owner='biomcp-query'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if status != 503 || old.get("store") != Some(&serde_json::json!(false)) || version != 4 {
        return Err(format!(
            "health admitted or migrated an old query schema: {status} {old} version={version}"
        ));
    }
    connection
        .execute_batch("DELETE FROM meta WHERE owner='biomcp-query'; DROP TABLE reply")
        .map_err(|e| e.to_string())?;
    let (status, damaged) = server.request("GET", "/health", "")?;
    let tables: u32 = connection
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='reply'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    // Health promises compatibility/openability, not payload-table integrity or repair.
    if status != 200 || damaged.get("store") != Some(&serde_json::json!(true)) || tables != 0 {
        return Err(format!(
            "health promise changed on a damaged payload table: {status} {damaged}; tables={tables}"
        ));
    }
    response
}

fn dropped_peer() -> Check {
    let server = Server::start()?;
    let mut socket = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
    socket.write_all(format!(
        "POST /mcp HTTP/1.1\r\nHost: {}\r\nX-Biomcp-Key-Oncokb-Token: synthetic-secret\r\nContent-Length: 999\r\n\r\nprivate-body",
        server.address
    ).as_bytes()).map_err(|e| e.to_string())?;
    socket
        .shutdown(std::net::Shutdown::Both)
        .map_err(|e| e.to_string())?;
    drop(socket);
    let event = server
        .diagnostics
        .lock()
        .map_err(|_| "HTTP diagnostics receiver poisoned")?
        .recv_timeout(Duration::from_secs(5))
        .map_err(|_| "dropped HTTP peer was not recorded")?;
    if event != "biomcp: HTTP connection failed (InvalidData)" {
        return Err(format!("unsafe or missing connection diagnostic: {event}"));
    }
    if server.request("GET", "/health", "")?.0 != 200 {
        return Err("server stopped after the client dropped".into());
    }
    Ok(())
}

#[cfg(unix)]
fn busy_health() -> Check {
    busy_capacity(false, false)
}

#[cfg(unix)]
fn configured_capacity() -> Check {
    busy_capacity(true, false)?;
    busy_capacity(true, true)
}

#[cfg(unix)]
fn busy_capacity(configured: bool, flags: bool) -> Check {
    let config = if flags {
        serde_json::json!({"http_max_calls":6,"http_max_connections":1})
    } else if configured {
        serde_json::json!({"http_max_calls":2,"http_max_connections":4})
    } else {
        serde_json::json!({})
    };
    let mut globals = vec!["--hosted".into()];
    if flags {
        globals.extend([
            "--http_max_calls".into(),
            "2".into(),
            "--http_max_connections".into(),
            "4".into(),
        ]);
    }
    let server =
        Server::configured_globals("127.0.0.1:0", Some(config), None, None, None, &globals)?;
    let mut held = Vec::new();
    let mut writers = Vec::new();
    for index in 1..=if configured { 2 } else { 8 } {
        let body = super::mcp::request(
            serde_json::json!(index),
            "tools/call",
            serde_json::json!({
                "name":"biomcp", "arguments":{"args":["get","protein",format!("P0000{index}")]}
            }),
        )
        .to_string();
        let (socket, writer) = hold_settings_reader(&server, &body)?;
        held.push(socket);
        writers.push(writer);
    }
    if configured {
        held.push(hold_header(&server)?);
    }
    // Every FIFO open witnesses a real admitted tool call before this probe competes.
    let probe =
        super::mcp::request(serde_json::json!(9), "tools/list", serde_json::json!({})).to_string();
    let status = server.request("POST", "/mcp", &probe)?.0;
    if status != if configured { 200 } else { 503 } {
        return Err(format!(
            "admitted long calls did not retain capacity: {status}; held={}",
            held_outcomes(&mut held)
        ));
    }
    if configured {
        refuse_over_capacity(&server, &probe, &mut held)?;
    } else {
        refused_call_diagnostic(&server)?;
    }
    let (status, body) = server.request("GET", "/health", "")?;
    if status != 200 || body.get("store") != Some(&serde_json::json!(true)) {
        return Err(format!("busy server failed health: {status} {body}"));
    }
    drop(writers);
    drop(held);
    Ok(())
}

#[cfg(unix)]
fn refused_call_diagnostic(server: &Server) -> Check {
    let call = super::mcp::request(
        serde_json::json!(10),
        "tools/call",
        serde_json::json!({"name":"fetch", "arguments":{"id":"protein private-query"}}),
    );
    let (status, _) = server.request("POST", "/mcp", &call.to_string())?;
    if status != 503 {
        return Err(format!("tool call escaped connection cap: {status}"));
    }
    let diagnostics = server
        .diagnostics
        .lock()
        .map_err(|_| "diagnostic receiver poisoned")?;
    let event = diagnostics
        .recv_timeout(Duration::from_secs(60))
        .map_err(|e| e.to_string())?;
    let event: serde_json::Value = serde_json::from_str(&event).map_err(|e| e.to_string())?;
    if event.get("event") != Some(&serde_json::json!("mcp_call"))
        || event.get("tool") != Some(&serde_json::json!("fetch"))
        || event.get("status") != Some(&serde_json::json!("error"))
        || event.get("failed_sources") != Some(&serde_json::json!([]))
        || event.to_string().contains("private-query")
        || diagnostics.try_recv().is_ok()
    {
        return Err(format!(
            "unsafe, missing or repeated capacity diagnostic: {event}"
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn hold_header(server: &Server) -> Result<TcpStream, String> {
    let mut socket = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
    socket
        .write_all(b"POST /mcp HTTP/1.1\r\n")
        .map_err(|e| e.to_string())?;
    Ok(socket)
}

#[cfg(unix)]
fn refuse_over_capacity(server: &Server, probe: &str, held: &mut Vec<TcpStream>) -> Check {
    let call = super::mcp::request(
        serde_json::json!(10),
        "tools/call",
        serde_json::json!({"name":"biomcp","arguments":{"args":["get","protein","P00003"]}}),
    )
    .to_string();
    let (status, body) = server.request("POST", "/mcp", &call)?;
    if status != 200
        || body.pointer("/error/code") != Some(&serde_json::json!(-31000))
        || !super::capacity_refusal(&body, 2)
    {
        return Err(format!("third tool escaped call cap: {status} {body}"));
    }
    // Two tool connections plus two unfinished headers fill the normal budget.
    held.push(hold_header(server)?);
    let (status, body) = server.request("POST", "/mcp", probe)?;
    if status != 503 || body != serde_json::json!({"error":"Service unavailable"}) {
        return Err(format!(
            "fifth connection escaped connection cap: {status} {body}"
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn hold_settings_reader(server: &Server, body: &str) -> Result<(TcpStream, std::fs::File), String> {
    use std::os::unix::fs::OpenOptionsExt;
    let config = server
        .environment()
        .root()
        .join("config/biomcp/config.json");
    // Unlinking preserves each previous reader's inode and its owned writer.
    std::fs::remove_file(&config).map_err(|e| e.to_string())?;
    nix::unistd::mkfifo(
        &config,
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .map_err(|e| e.to_string())?;
    let mut socket = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
    socket
        .write_all(
            format!(
                "POST /mcp HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\n\
         Accept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2026-07-28\r\n\
         Mcp-Method: tools/call\r\nMcp-Name: biomcp\r\nContent-Length: {}\r\n\r\n{body}",
                server.address,
                body.len()
            )
            .as_bytes(),
        )
        .map_err(|e| e.to_string())?;
    // A writer cannot open until this call has opened the fresh FIFO for reading.
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(nix::libc::O_NONBLOCK)
            .open(&config)
        {
            Ok(writer) => return Ok((socket, writer)),
            Err(error)
                if error.raw_os_error() == Some(nix::libc::ENXIO) && Instant::now() < end =>
            {
                std::thread::yield_now();
            }
            Err(error) => return Err(format!("settings reader did not enter its FIFO: {error}")),
        }
    }
}

#[cfg(unix)]
fn held_outcomes(held: &mut [TcpStream]) -> String {
    held.iter_mut()
        .map(|socket| {
            if let Err(error) = socket.set_nonblocking(true) {
                return error.to_string();
            }
            let mut response = String::new();
            let result = socket.take(8192).read_to_string(&mut response);
            if response.is_empty() {
                return format!("pending {result:?}");
            }
            let Some((head, body)) = response.split_once("\r\n\r\n") else {
                return format!("incomplete header: {} bytes", response.len());
            };
            let value = serde_json::from_str::<serde_json::Value>(body).ok();
            format!(
                "{} rpc_error={:?} tool_error={:?}",
                head.lines().next().unwrap_or_default(),
                value.as_ref().and_then(|value| value.get("error")),
                value
                    .as_ref()
                    .and_then(|value| value.pointer("/result/structuredContent/error"))
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}
