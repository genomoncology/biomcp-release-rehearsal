//! Event-driven proof that pending source work leaves the server responsive.
//! Cancellation interrupts sockets and cooperative checks; DNS resolution and TCP connect retain their existing deadline bounds.
use super::{launch::Environment, mcp::request, process::Process};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{Shutdown, TcpListener},
    process::Stdio,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("mcp-parity/oversized-frame-drains-active-call", || {
            oversized().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/cancel-stops-work-and-frees-capacity", || {
            cancelled().map_err(Into::into)
        }),
        transcript_trial("mcp-parity/cancel-preserves-another-active-call", || {
            isolated_cancel().map_err(Into::into)
        }),
    ]
}

const WATCHDOG: Duration = Duration::from_secs(30);
enum Probe {
    Stop,
    Cancelled,
    Isolation,
}
struct ProxyControl {
    commands: mpsc::Sender<Probe>,
    checked: mpsc::Receiver<Result<(), String>>,
}
impl ProxyControl {
    fn check(&self, probe: Probe) -> Result<(), String> {
        self.commands.send(probe).map_err(|e| e.to_string())?;
        self.checked
            .recv_timeout(WATCHDOG)
            .map_err(|_| "proxy check watchdog")?
    }
}

pub(super) fn check() -> Result<(), String> {
    run(Mode::Reply)
}
pub(super) fn oversized() -> Result<(), String> {
    run(Mode::Oversized)
}
pub(super) fn isolated_cancel() -> Result<(), String> {
    run(Mode::Isolation)
}
pub(super) fn cancelled() -> Result<(), String> {
    run(Mode::Cancel)
}
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Reply,
    Oversized,
    Cancel,
    Isolation,
}
fn run(mode: Mode) -> Result<(), String> {
    let environment = Environment::new().map_err(|e| e.to_string())?;
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let address = listener.local_addr().map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let (connected_tx, connected_rx) = mpsc::channel();
    let (commands, release_rx) = mpsc::channel();
    let (checked_tx, checked) = mpsc::channel();
    let control = ProxyControl { commands, checked };
    let (certificate, tls) = super::network_stub::identity(&["rest.uniprot.org"])?;
    configure(&environment, address, &certificate)?;
    let proxy = thread::spawn(move || hold(listener, connected_tx, release_rx, checked_tx, tls));
    let mut child = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &["serve".into()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdin = child.stdin.take().ok_or("missing stdin")?;
    let stdout = child.stdout.take().ok_or("missing stdout")?;
    let mut process = Process::new(child)?;
    let (reply_tx, reply_rx) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let reply = line
                .map_err(|e| e.to_string())
                .and_then(|line| serde_json::from_str::<Value>(&line).map_err(|e| e.to_string()));
            if reply_tx.send(reply).is_err() {
                break;
            }
        }
    });
    let result = prove(mode, &mut stdin, &connected_rx, &reply_rx)
        .and_then(|()| mode_input(mode, &mut stdin, &reply_rx, &connected_rx, &control));
    let _ = control.commands.send(Probe::Stop);
    let proxy_result = proxy.join().map_err(|_| "proxy panicked")?;
    if result.is_err() || proxy_result.is_err() {
        let _ = process.child.kill();
    }
    drop(stdin);
    let finish = process.finish();
    reader.join().map_err(|_| "reply reader panicked")?;
    result?;
    proxy_result?;
    let (status, _, stderr) = finish?;
    if !status.success() || !stderr.is_empty() {
        return Err("held proxy server failed".into());
    }
    mode_reply(mode, &reply_rx)
}
fn configure(
    environment: &Environment,
    address: std::net::SocketAddr,
    certificate: &str,
) -> Result<(), String> {
    let path = environment.root().join("source.pem");
    fs::write(&path, certificate).map_err(|e| e.to_string())?;
    environment
        .config(&json!({"proxy":format!("http://{address}"), "ca_bundle":path}))
        .map_err(|e| e.to_string())
}
fn mode_input(
    mode: Mode,
    stdin: &mut impl Write,
    replies: &mpsc::Receiver<Result<Value, String>>,
    connected: &mpsc::Receiver<Result<(), String>>,
    control: &ProxyControl,
) -> Result<(), String> {
    match mode {
        Mode::Reply => Ok(()),
        Mode::Cancel => cancel_capacity(stdin, replies, control),
        Mode::Isolation => {
            send(
                stdin,
                request(
                    json!("other"),
                    "tools/call",
                    json!({"name":"biomcp", "arguments":{"args":["get","protein","P04637"]}}),
                ),
            )?;
            connected
                .recv_timeout(WATCHDOG)
                .map_err(|_| "second active CONNECT watchdog")??;
            send(
                stdin,
                json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":"call"}}),
            )?;
            control.check(Probe::Isolation)?;
            let reply = replies
                .recv_timeout(WATCHDOG)
                .map_err(|_| "other call completion watchdog")??;
            if reply.get("id") != Some(&json!("other"))
                || reply.pointer("/result/structuredContent/status") != Some(&json!("complete"))
            {
                return Err(format!(
                    "cancellation stopped the other active call: {reply}"
                ));
            }
            Ok(())
        }
        Mode::Oversized => {
            stdin
                .write_all(&vec![b'x'; 65537])
                .map_err(|e| e.to_string())?;
            stdin.write_all(b"\n").map_err(|e| e.to_string())?;
            stdin.flush().map_err(|e| e.to_string())?;
            let reply = replies
                .recv_timeout(WATCHDOG)
                .map_err(|_| "oversized reply watchdog")??;
            if reply.pointer("/error/code") != Some(&json!(-32600)) {
                return Err(format!("missing oversized frame error: {reply}"));
            }
            Ok(())
        }
    }
}

fn mode_reply(mode: Mode, reply_rx: &mpsc::Receiver<Result<Value, String>>) -> Result<(), String> {
    if matches!(mode, Mode::Cancel | Mode::Isolation) {
        if reply_rx.try_recv().is_ok() {
            return Err("cancelled call replied".into());
        }
        return Ok(());
    }
    let reply = reply_rx
        .recv_timeout(WATCHDOG)
        .map_err(|_| "missing tool reply")??;
    if reply.get("id") != Some(&json!("call"))
        || reply.pointer("/result/structuredContent/status") != Some(&json!("failed"))
        || reply.pointer("/result/isError") != Some(&json!(true))
    {
        return Err(format!("wrong tool outcome: {reply}"));
    }
    Ok(())
}
fn prove(
    mode: Mode,
    stdin: &mut impl Write,
    connected: &mpsc::Receiver<Result<(), String>>,
    replies: &mpsc::Receiver<Result<Value, String>>,
) -> Result<(), String> {
    let args = if matches!(mode, Mode::Cancel | Mode::Isolation) {
        vec!["get", "protein", "P15056"]
    } else {
        vec!["get", "gene", "BRAF"]
    };
    send(
        stdin,
        request(
            json!("call"),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args}}),
        ),
    )?;
    connected
        .recv_timeout(WATCHDOG)
        .map_err(|_| "CONNECT watchdog")??;
    if replies.try_recv().is_ok() {
        return Err("tool completed before held CONNECT was released".into());
    }
    send(
        stdin,
        request(
            json!("disabled"),
            "tools/call",
            json!({ "name":"biomcp", "arguments":{"args":["get","gene","BRAF","--disable-sources","hgnc"]} }),
        ),
    )?;
    let disabled = replies
        .recv_timeout(WATCHDOG)
        .map_err(|_| "disabled call watchdog")??;
    if disabled.get("id") != Some(&json!("disabled"))
        || disabled.pointer("/result/structuredContent/sources/0/reason")
            != Some(&json!("disabled"))
    {
        return Err(format!(
            "call source policy escaped its request: {disabled}"
        ));
    }
    send(stdin, request(json!("ping"), "ping", json!({})))?;
    let reply = replies
        .recv_timeout(WATCHDOG)
        .map_err(|_| "ping watchdog")??;
    if reply.get("id") != Some(&json!("ping")) || reply.get("error").is_some() {
        return Err(format!("ping did not pass pending call: {reply}"));
    }
    Ok(())
}
fn send(output: &mut impl Write, value: Value) -> Result<(), String> {
    serde_json::to_writer(&mut *output, &value).map_err(|e| e.to_string())?;
    output
        .write_all(b"\n")
        .and_then(|_| output.flush())
        .map_err(|e| e.to_string())
}
fn hold(
    listener: TcpListener,
    connected: mpsc::Sender<Result<(), String>>,
    release: mpsc::Receiver<Probe>,
    checked: mpsc::Sender<Result<(), String>>,
    tls: std::sync::Arc<rustls::ServerConfig>,
) -> Result<(), String> {
    let deadline = Instant::now() + WATCHDOG;
    let mut held = Vec::new();
    loop {
        match release.try_recv() {
            Ok(Probe::Stop) => break,
            Ok(Probe::Cancelled) => {
                let _ = checked.send(held.iter_mut().try_for_each(peer_eof));
            }
            Ok(Probe::Isolation) => {
                let _ = checked.send(resume_other(&mut held, tls.clone()));
            }
            Err(_) => (),
        }
        if Instant::now() >= deadline {
            return Err("release watchdog".into());
        }
        match listener.accept() {
            Ok((stream, _)) => held.push(proxy_request(stream, &connected)?),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5))
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    for stream in held {
        if let Err(error) = stream.shutdown(Shutdown::Both)
            && error.kind() != std::io::ErrorKind::NotConnected
        {
            return Err(error.to_string());
        }
    }
    Ok(())
}
fn peer_eof(stream: &mut std::net::TcpStream) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| e.to_string())?;
    let mut byte = [0];
    match super::socket_read::read(stream, &mut byte) {
        Ok(0) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::ConnectionReset => Ok(()),
        other => Err(format!("cancelled socket did not close: {other:?}")),
    }
}
fn resume_other(
    held: &mut Vec<std::net::TcpStream>,
    config: std::sync::Arc<rustls::ServerConfig>,
) -> Result<(), String> {
    if held.len() != 2 {
        return Err("isolation needs two active sockets".into());
    }
    let mut other = held.pop().ok_or("missing other socket")?;
    peer_eof(held.first_mut().ok_or("missing cancelled socket")?)?;
    other.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut byte = [0];
    match super::socket_read::read(&mut other, &mut byte) {
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => (),
        result => return Err(format!("cancellation closed the other socket: {result:?}")),
    }
    other.set_nonblocking(false).map_err(|e| e.to_string())?;
    other
        .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
        .map_err(|e| e.to_string())?;
    let connection = rustls::ServerConnection::new(config).map_err(|e| e.to_string())?;
    let mut tls = rustls::StreamOwned::new(connection, other);
    let request = super::network_stub::source_request(&mut tls)?;
    if !request.starts_with("GET /uniprotkb/P04637.json ") {
        return Err(format!("wrong independent request: {request}"));
    }
    let body = json!(
        {"primaryAccession":"P04637","uniProtkbId":"SYNTH_HUMAN","entryType":"UniProtKB reviewed (Swiss-Prot)","organism":{"taxonId":9606},
        "proteinDescription":{"recommendedName":{"fullName":{"value":"synthetic protein"}}},"sequence":{"length":1}}
    )
    .to_string();
    write!(
        tls,
        "HTTP/1.1 200 Synthetic\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: \
        close\r\n\r\n{body}",
        body.len()
    )
    .map_err(|e| e.to_string())?;
    tls.flush().map_err(|e| e.to_string())
}
fn proxy_request(
    mut stream: std::net::TcpStream,
    connected: &mpsc::Sender<Result<(), String>>,
) -> Result<std::net::TcpStream, String> {
    stream.set_nonblocking(false).map_err(|e| e.to_string())?;
    stream
        .set_read_timeout(Some(WATCHDOG))
        .map_err(|e| e.to_string())?;
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") && header.len() < 8192 {
        let mut byte = [0];
        stream.read_exact(&mut byte).map_err(|e| e.to_string())?;
        header.extend(byte);
    }
    let result = if header.starts_with(b"CONNECT ") && header.ends_with(b"\r\n\r\n") {
        Ok(())
    } else {
        Err("proxy expected CONNECT".into())
    };
    let _ = connected.send(result);
    Ok(stream)
}

fn cancel_capacity(
    stdin: &mut impl Write,
    replies: &mpsc::Receiver<Result<Value, String>>,
    control: &ProxyControl,
) -> Result<(), String> {
    for id in 1..8 {
        send(
            stdin,
            request(
                json!(id),
                "tools/call",
                json!({"name":"biomcp", "arguments":{ "args":["get","protein","P15056"]}}),
            ),
        )?;
    }
    send(
        stdin,
        request(
            json!("ninth"),
            "tools/call",
            json!({"name":"biomcp", "arguments":{"args":["help"]}}),
        ),
    )?;
    let ninth = replies
        .recv_timeout(WATCHDOG)
        .map_err(|_| "capacity probe watchdog")??;
    if ninth.get("id") != Some(&json!("ninth"))
        || ninth.pointer("/error/code") != Some(&json!(-31000))
    {
        return Err(format!("eight workers were not occupied: {ninth}"));
    }
    for id in std::iter::once(json!("call")).chain((1..8).map(|id| json!(id))) {
        send(
            stdin,
            json!({"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":id}}),
        )?;
    }
    control.check(Probe::Cancelled)?;
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        send(
            stdin,
            request(
                json!("free"),
                "tools/call",
                json!({"name":"biomcp", "arguments":{"args":["help"]}}),
            ),
        )?;
        let reply = replies
            .recv_timeout(end.saturating_duration_since(Instant::now()))
            .map_err(|_| "cancel did not stop work")??;
        if reply.get("id") != Some(&json!("free")) {
            return Err(format!("cancelled call replied: {reply}"));
        }
        if reply.get("error").is_none() {
            return Ok(());
        }
        if reply.pointer("/error/code") != Some(&json!(-31000)) || Instant::now() >= end {
            return Err(format!("cancel did not free capacity: {reply}"));
        }
    }
}
