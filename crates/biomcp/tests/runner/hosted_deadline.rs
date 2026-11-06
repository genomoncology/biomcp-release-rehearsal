//! Feature mcp-hosted: the call timer covers a blocked settings read before source work.
use libtest_mimic::Trial;

// Bounds diagnose a stuck fixture; no assertion measures admission or cleanup speed.
#[cfg(unix)]
const DEADLOCK_GUARD: std::time::Duration = std::time::Duration::from_secs(60);

pub(super) fn trials() -> Vec<Trial> {
    #[cfg(unix)]
    {
        vec![
            super::transcript::trial("mcp-hosted/replay-partial-at-deadline", || {
                replay_partial().map_err(Into::into)
            }),
            super::transcript::trial("mcp-hosted/pacing/whole-call-deadline", || {
                blocked_settings().map_err(Into::into)
            }),
        ]
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

#[cfg(unix)]
fn replay_partial() -> Result<(), String> {
    use serde_json::{Value, json};
    let environment = super::launch::Environment::new().map_err(|e| e.to_string())?;
    let replay = environment.root().join("replay");
    let source = super::launch::fixtures().join("replies/gene/main");
    for name in ["hgnc", "mygene"] {
        let destination = replay.join(name);
        std::fs::create_dir_all(&destination).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(source.join(name)).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let bytes = std::fs::read(entry.path()).map_err(|e| e.to_string())?;
            let mut value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            for exchange in value
                .get_mut("exchanges")
                .and_then(Value::as_array_mut)
                .ok_or("missing exchanges")?
            {
                exchange.as_object_mut().ok_or("invalid exchange")?.insert(
                    "delay_ms".into(),
                    json!(if name == "mygene" { 60_000 } else { 0 }),
                );
            }
            std::fs::write(destination.join(entry.file_name()), value.to_string())
                .map_err(|e| e.to_string())?;
        }
    }
    let server = super::http::Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"command_timeout_seconds":15})),
        Some(replay),
        Some(("ONCOKB_TOKEN", "diagnostic-private-key")),
        None,
        &["--hosted".into()],
    )?;
    let body = super::mcp::request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp", "arguments":{"args":["get", "gene", "KRAS"]}}),
    );
    let (status, reply) = server.request("POST", "/mcp", &body.to_string())?;
    let content = reply
        .pointer("/result/structuredContent")
        .cloned()
        .or_else(|| serde_json::from_str(reply.pointer("/result/content/0/text")?.as_str()?).ok())
        .ok_or_else(|| format!("partial answer lost: {status} {reply}"))?;
    if status != 200
        || content.get("status") != Some(&json!("partial"))
        || !content.to_string().contains("KRAS")
        || !content
            .get("sources")
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                sources.iter().any(|source| {
                    source.get("source") == Some(&json!("mygene"))
                        && source.get("state") == Some(&json!("failed"))
                        && source.get("reason") == Some(&json!("deadline-exceeded"))
                })
            })
    {
        return Err(format!(
            "fast replay answer lost to slow source: {status} {content}"
        ));
    }
    let event = server
        .diagnostics
        .lock()
        .map_err(|_| "diagnostic receiver poisoned")?
        .recv_timeout(DEADLOCK_GUARD)
        .map_err(|e| e.to_string())?;
    let event: Value = serde_json::from_str(&event).map_err(|e| e.to_string())?;
    if event.get("tool") != Some(&json!("biomcp"))
        || event.get("status") != Some(&json!("partial"))
        || event.get("failed_sources") != Some(&json!(["mygene"]))
        || event.to_string().contains("KRAS")
        || event.to_string().contains("diagnostic-private-key")
    {
        return Err(format!("unsafe or missing partial diagnostic: {event}"));
    }
    Ok(())
}
