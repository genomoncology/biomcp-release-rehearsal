//! Observe the first caller consuming stdin before starting the competing writer.
use super::{capture::Capture, launch::Environment, process::Process};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};
use serde_json::Value;
use std::{
    io::Write,
    os::fd::{AsFd, OwnedFd},
    os::unix::net::UnixStream,
    process::Stdio,
    time::{Duration, Instant},
};
const FIRST: &str = "synthetic-waiting-credential";
const SECOND: &str = "synthetic-concurrent-credential";

pub(super) fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let args = ["key", "set", "semantic-scholar"].map(str::to_owned);
    let (mut input, reader) = UnixStream::pair().map_err(|e| e.to_string())?;
    let observer = reader.try_clone().map_err(|e| e.to_string())?;
    let (byte, remainder) = FIRST
        .as_bytes()
        .split_first()
        .ok_or("missing first input byte")?;
    input
        .write_all(std::slice::from_ref(byte))
        .map_err(|e| e.to_string())?;
    let mut command = env.command(env!("CARGO_BIN_EXE_biomcp"), &args);
    let first = command
        .stdin(Stdio::from(OwnedFd::from(reader)))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let first = Process::new(first)?;
    if let Err(error) = consumed(&observer) {
        drop(input);
        let _ = first.finish();
        return Err(error);
    }
    let (mut second, mut child) = spawn(&env, &args)?;
    child
        .child
        .stdin
        .take()
        .ok_or("missing second stdin")?
        .write_all(format!("{SECOND}\n").as_bytes())
        .map_err(|e| e.to_string())?;
    let result = child.finish();
    // Release and reap the first caller even when the second writer hangs.
    input
        .write_all(remainder)
        .and_then(|()| input.write_all(b"\n"))
        .map_err(|e| e.to_string())?;
    drop(input);
    let (first_status, first_out, _) = first.finish()?;
    let (status, stdout, _) = result?;
    second
        .record(status, &stdout, "cli")
        .map_err(|e| e.to_string())?;
    command
        .record(first_status, &first_out, "cli")
        .map_err(|e| e.to_string())?;
    if !status.success() || first_status.code() != Some(2) {
        return Err(format!(
            "second writer={status}; waiting caller={first_status}"
        ));
    }
    let value: Value = serde_json::from_slice(&first_out).map_err(|e| e.to_string())?;
    if value.pointer("/error/code").and_then(Value::as_str) != Some("usage.arguments") {
        return Err(format!(
            "waiting caller did not require replacement: {value}"
        ));
    }
    let config =
        std::fs::read(env.root().join("config/biomcp/config.json")).map_err(|e| e.to_string())?;
    let config: Value = serde_json::from_slice(&config).map_err(|e| e.to_string())?;
    if config
        .pointer("/keys/semantic_scholar_api_key")
        .and_then(Value::as_str)
        != Some(SECOND)
    {
        return Err("waiting caller overwrote the concurrent credential".into());
    }
    Ok(())
}
fn spawn(env: &Environment, args: &[String]) -> Result<(Capture, Process), String> {
    let mut command = env.command(env!("CARGO_BIN_EXE_biomcp"), args);
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok((command, Process::new(child)?))
}
fn consumed(observer: &UnixStream) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut descriptors = [PollFd::new(observer.as_fd(), PollFlags::POLLIN)];
    loop {
        match poll(&mut descriptors, PollTimeout::ZERO) {
            Ok(0) => return Ok(()),
            Ok(_) if Instant::now() < deadline => std::thread::yield_now(),
            other => {
                return Err(format!(
                    "first caller did not consume its input byte: {other:?}"
                ));
            }
        }
    }
}
