//! Feature mcp-hosted: the call timer covers a blocked settings read before source work.
use libtest_mimic::Trial;

// Bounds diagnose a stuck fixture; no assertion measures admission or cleanup speed.
#[cfg(unix)]
const DEADLOCK_GUARD: std::time::Duration = std::time::Duration::from_secs(60);

pub(super) fn trials() -> Vec<Trial> {
    #[cfg(unix)]
    {
        vec![super::transcript::trial(
            "mcp-hosted/pacing/whole-call-deadline",
            || blocked_settings().map_err(Into::into),
        )]
    }
    #[cfg(not(unix))]
    {
        Vec::new()
    }
}

#[cfg(unix)]
fn blocked_settings() -> Result<(), String> {
    use super::http::Server;
    use nix::{sys::stat::Mode, unistd::mkfifo};
    use std::{fs::OpenOptions, io::Write, os::unix::fs::OpenOptionsExt, time::Instant};
    let mut server = Server::configured_globals(
        "127.0.0.1:0",
        Some(serde_json::json!({})),
        None,
        None,
        None,
        &["--hosted".into()],
    )?;
    let path = server
        .environment()
        .root()
        .join("config/biomcp/config.json");
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    mkfifo(&path, Mode::S_IRUSR | Mode::S_IWUSR).map_err(|e| e.to_string())?;
    let address = server.address;
    let worker = std::thread::spawn(move || request(address));
    // A successful nonblocking writer open witnesses the blocked reader's admission.
    let end = Instant::now() + DEADLOCK_GUARD;
    let mut writer = loop {
        match OpenOptions::new()
            .write(true)
            .custom_flags(nix::libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(writer) => break writer,
            Err(error)
                if error.raw_os_error() == Some(nix::libc::ENXIO) && Instant::now() < end =>
            {
                std::thread::yield_now();
            }
            Err(error) => {
                std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                server
                    .environment()
                    .config(&serde_json::json!({}))
                    .map_err(|e| e.to_string())?;
                let _ = worker.join();
                return Err(format!("settings reader did not enter the FIFO: {error}"));
            }
        }
    };
    // The response must arrive while the settings read still has no EOF.
    let reply = worker.join().map_err(|_| "deadline client stopped")?;
    writer.write_all(b"{}").map_err(|e| e.to_string())?;
    drop(writer);
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    server
        .environment()
        .config(&serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    let reply = reply?;
    if reply.pointer("/error/data/reason") != Some(&serde_json::json!("deadline-exceeded")) {
        return Err(format!(
            "whole call did not return the typed timeout: {reply}"
        ));
    }
    // Finished timed-out work must release ownership so the same service can answer again.
    let (status, reply) = server.request(
        "POST",
        "/mcp",
        &super::mcp::request(serde_json::json!(2), "tools/list", serde_json::json!({})).to_string(),
    )?;
    if status != 200 || reply.pointer("/result/tools/0/name") != Some(&serde_json::json!("biomcp"))
    {
        return Err(format!(
            "service did not recover after timed-out settings read: {reply}"
        ));
    }
    server.stop()
}

#[cfg(unix)]
fn request(address: std::net::SocketAddr) -> Result<serde_json::Value, String> {
    use std::io::{Read, Write};
    let body = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
        "params":{"name":"biomcp","arguments":{"args":["get","gene","KRAS","--offline"]}}})
    .to_string();
    let mut stream = std::net::TcpStream::connect(address).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(DEADLOCK_GUARD))
        .map_err(|e| e.to_string())?;
    let frame = format!(
        "POST /mcp HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\n\
        Accept: application/json, text/event-stream\r\nMCP-Protocol-Version: 2025-11-25\r\n\
        Connection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(frame.as_bytes())
        .map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("whole-call response failed while settings remained blocked: {e}"))?;
    let (_, body) = response
        .split_once("\r\n\r\n")
        .ok_or("deadline response has no body")?;
    serde_json::from_str(body).map_err(|e| e.to_string())
}
