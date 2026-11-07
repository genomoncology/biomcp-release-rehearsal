//! Both installed processes must accept searches while one ordinary cache is locked.
use super::{launch::Environment, mcp::request, process::Process};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::{ChildStdin, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

pub(super) struct Peer {
    process: Option<Process>,
    input: Option<ChildStdin>,
    replies: Receiver<Result<Value, String>>,
    reader: Option<thread::JoinHandle<Vec<u8>>>,
}
impl Peer {
    pub(super) fn with_globals(env: &Environment, globals: &[String]) -> Result<Self, String> {
        let mut args = vec!["serve".into()];
        args.extend_from_slice(globals);
        let mut child = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let input = child.stdin.take();
        let output = child.stdout.take().ok_or("stdout missing")?;
        let (tx, replies) = mpsc::channel();
        let reader = thread::spawn(move || {
            let mut stdout = Vec::new();
            let mut output = BufReader::new(output);
            loop {
                let mut line = Vec::new();
                let read = output.read_until(b'\n', &mut line);
                stdout.extend_from_slice(&line);
                if matches!(read, Ok(0)) {
                    break;
                }
                let failed = read.is_err();
                let value = read.map_err(|e| e.to_string()).and_then(|_| {
                    let value: Value = serde_json::from_slice(&line).map_err(|e| e.to_string())?;
                    if value.get("jsonrpc") != Some(&json!("2.0")) {
                        return Err("non-JSON-RPC stdout".into());
                    }
                    Ok(value)
                });
                if tx.send(value).is_err() || failed {
                    break;
                }
            }
            stdout
        });
        Ok(Self {
            process: Some(Process::new(child)?),
            input,
            replies,
            reader: Some(reader),
        })
    }
    pub(super) fn send(&mut self, frame: &Value) -> Result<(), String> {
        let writer = self.input.as_mut().ok_or("stdin missing")?;
        serde_json::to_writer(&mut *writer, frame).map_err(|e| e.to_string())?;
        writer
            .write_all(b"\n")
            .and_then(|()| writer.flush())
            .map_err(|e| e.to_string())
    }
    pub(super) fn next(&mut self) -> Result<Value, String> {
        let started = Instant::now();
        match self.replies.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(reply)) => Ok(reply),
            result => Err(self.failure(&format!(
                "publication reply after {:?}: {result:?}",
                started.elapsed()
            ))),
        }
    }
    fn failure(&mut self, context: &str) -> String {
        let mut capture = || -> Result<String, String> {
            let mut process = self.process.take().ok_or("process missing")?;
            let observed = process.child.try_wait().map_err(|e| e.to_string())?;
            if observed.is_none() {
                let _ = process.child.kill();
            }
            let (status, _, stderr) = process.finish()?;
            let stdout = self
                .reader
                .take()
                .ok_or("reader missing")?
                .join()
                .map_err(|_| "reader panicked")?;
            let directory = biomcp_test_support::evidence::directory()
                .and_then(|root| {
                    tempfile::Builder::new()
                        .prefix("publication-")
                        .tempdir_in(root)
                })
                .map_err(|e| e.to_string())?
                .keep();
            std::fs::write(directory.join("stdout"), &stdout).map_err(|e| e.to_string())?;
            std::fs::write(directory.join("stderr"), &stderr).map_err(|e| e.to_string())?;
            Ok(format!(
                "{context}; child before cleanup={observed:?}, reaped={status:?}; raw output={}; stdout first bytes={:?}; stderr (first {} of {} bytes)={:?}",
                directory.display(),
                stdout.get(..stdout.len().min(256)).unwrap_or(&stdout),
                stderr.len().min(4096),
                stderr.len(),
                String::from_utf8_lossy(stderr.get(..stderr.len().min(4096)).unwrap_or(&stderr))
            ))
        };
        capture().unwrap_or_else(|error| format!("{context}; diagnostic capture failed: {error}"))
    }
    pub(super) fn finish(mut self) -> Result<(), String> {
        drop(self.input.take());
        let (status, _, stderr) = self.process.take().ok_or("process missing")?.finish()?;
        self.reader
            .take()
            .ok_or("reader missing")?
            .join()
            .map_err(|_| "reader panicked")?;
        if !status.success() || !stderr.is_empty() || self.replies.try_recv().is_ok() {
            return Err("extra stdout or unsuccessful MCP shutdown".into());
        }
        Ok(())
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        if let Some(process) = &mut self.process {
            let _ = process.child.kill();
            let _ = process.child.wait();
        }
    }
}

pub(super) fn check(
    env: &Environment,
    release: impl FnOnce() -> Result<(), String>,
) -> Result<Vec<Value>, String> {
    let mut peers = [Peer::with_globals(env, &[])?, Peer::with_globals(env, &[])?];
    for peer in &mut peers {
        peer.send(&request(
            json!("search"),
            "tools/call",
            json!({"name":"biomcp","arguments":{
            "args":["search","article","AKT1 E17K","--limit","2","--exact-term","--offline","--fresh"]}}),
        ))?;
        peer.send(&request(json!("accepted"), "ping", json!({})))?;
    }
    // Ping is consumed after the search is admitted. Both searches remain
    // pending behind the write transaction, so process overlap is an event.
    for peer in &mut peers {
        let reply = peer.next()?;
        if reply.get("id") != Some(&json!("accepted")) || reply.get("error").is_some() {
            return Err(format!("search did not wait for ordinary cache: {reply}"));
        }
    }
    release()?;
    let mut results = Vec::new();
    for mut peer in peers {
        let reply = peer.next()?;
        if reply.get("id") != Some(&json!("search"))
            || reply.pointer("/result/isError") != Some(&json!(false))
        {
            return Err(format!("ordinary-cache search failed: {reply}"));
        }
        results.push(
            reply
                .pointer("/result/structuredContent")
                .ok_or("content missing")?
                .clone(),
        );
        peer.finish()?;
    }
    Ok(results)
}
