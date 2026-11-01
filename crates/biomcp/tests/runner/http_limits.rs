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
    match socket.read(&mut byte) {
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
    match held.read(&mut byte) {
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
    let result = slow.read(&mut byte);
    // Measure the observed read event before stopping and joining the writer.
    let close_elapsed = started.elapsed();
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
    if !ended || close_elapsed > Duration::from_secs(8) || joined.is_err() {
        return Err(format!(
            "slow header deadline: read={result:?}, close_elapsed={close_elapsed:?}, \
             cleanup_elapsed={cleanup_elapsed:?}, writer_panicked={}",
            joined.is_err()
        ));
    }
    Ok(())
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
    // Framing reserves one reader beyond the eight active call slots for health.
    for _ in 0..9 {
        let mut socket = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
        socket
            .write_all(b"GET /health HTTP/1.1\r\n")
            .map_err(|e| e.to_string())?;
        held.push(socket);
    }
    // A silent refused peer must not consume the read budget or delay the next accept.
    for _ in 0..2 {
        let mut excess = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
        excess
            .set_read_timeout(Some(Duration::from_millis(800)))
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
    use std::os::unix::fs::OpenOptionsExt;
    // Every tool blocks in the existing per-call settings read until this writer closes.
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(serde_json::json!({})),
        None,
        None,
        None,
        &["--hosted".into()],
    )?;
    let config = server
        .environment()
        .root()
        .join("config/biomcp/config.json");
    std::fs::remove_file(&config).map_err(|e| e.to_string())?;
    nix::unistd::mkfifo(
        &config,
        nix::sys::stat::Mode::S_IRUSR | nix::sys::stat::Mode::S_IWUSR,
    )
    .map_err(|e| e.to_string())?;
    let writer = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(nix::libc::O_NONBLOCK)
        .open(&config)
        .map_err(|e| e.to_string())?;
    let mut held = Vec::new();
    for index in 1..=8 {
        let body = super::mcp::request(
            serde_json::json!(index),
            "tools/call",
            serde_json::json!({
                "name":"biomcp", "arguments":{"args":["get","protein",format!("P0000{index}")]}
            }),
        )
        .to_string();
        let mut socket = TcpStream::connect(server.address).map_err(|e| e.to_string())?;
        socket.write_all(format!(
            "POST /mcp HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\n\
             Accept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2026-07-28\r\nMcp-Method: tools/call\r\nMcp-Name: biomcp\r\n\
             Content-Length: {}\r\n\r\n{body}",
            server.address, body.len()
        ).as_bytes()).map_err(|e| e.to_string())?;
        held.push(socket);
    }
    // A refused ordinary request observes that all eight tool slots are occupied.
    let probe =
        super::mcp::request(serde_json::json!(9), "tools/list", serde_json::json!({})).to_string();
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let status = server.request("POST", "/mcp", &probe)?.0;
        if status == 503 {
            break;
        }
        if status != 200 || Instant::now() >= end {
            return Err(format!(
                "eight long calls did not occupy capacity: {status}"
            ));
        }
    }
    let (status, body) = server.request("GET", "/health", "")?;
    if status != 200 || body.get("store") != Some(&serde_json::json!(true)) {
        return Err(format!("busy server failed health: {status} {body}"));
    }
    drop(writer);
    drop(held);
    Ok(())
}
