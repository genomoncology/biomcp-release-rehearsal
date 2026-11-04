use std::{
    io::{self, Read},
    process::{Child, ExitStatus},
    thread,
    time::{Duration, Instant},
};

const WATCHDOG: Duration = Duration::from_secs(30);

// Reader threads drain pipes while the parent watches process termination. No
// timing assertion defines success; the watchdog only kills and reaps a hang.
pub(super) struct Process {
    pub(super) child: Child,
    stdout: thread::JoinHandle<io::Result<Vec<u8>>>,
    stderr: thread::JoinHandle<io::Result<Vec<u8>>>,
}

impl Process {
    pub(super) fn new(mut child: Child) -> Result<Self, String> {
        let stdout = child
            .stdout
            .take()
            .map_or_else(|| reader(io::empty()), reader);
        let stderr = child.stderr.take().ok_or("stderr missing")?;
        Ok(Self {
            child,
            stdout,
            stderr: reader(stderr),
        })
    }

    pub(super) fn finish(mut self) -> Result<(ExitStatus, Vec<u8>, Vec<u8>), String> {
        let started = Instant::now();
        let deadline = started + WATCHDOG;
        let status = loop {
            if let Some(status) = self.child.try_wait().map_err(|e| e.to_string())? {
                break status;
            }
            if Instant::now() >= deadline {
                let kill = self.child.kill();
                let status = self.child.wait();
                let stderr = joined(self.stderr)?;
                return Err(format!(
                    "MCP watchdog: server still running after {:?}; kill={kill:?}; \
                     reaped={status:?}; stderr (first {} of {} bytes)={:?}",
                    started.elapsed(),
                    stderr.len().min(4096),
                    stderr.len(),
                    String::from_utf8_lossy(stderr.get(..4096).unwrap_or(&stderr)),
                ));
            }
            thread::sleep(Duration::from_millis(10));
        };
        Ok((status, joined(self.stdout)?, joined(self.stderr)?))
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "The shared subprocess watchdog drains both pipes concurrently"
)]
fn reader(mut input: impl Read + Send + 'static) -> thread::JoinHandle<io::Result<Vec<u8>>> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        input.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

fn joined(handle: thread::JoinHandle<io::Result<Vec<u8>>>) -> Result<Vec<u8>, String> {
    handle
        .join()
        .map_err(|_| "pipe reader panicked")?
        .map_err(|e| e.to_string())
}

// Shared finite request batches use the same pipe draining and watchdog as all MCP trials.
pub(super) fn exchange(
    child: Child,
    frames: &[serde_json::Value],
) -> Result<(ExitStatus, Vec<u8>, Vec<u8>), String> {
    use std::io::Write;
    let mut process = Process::new(child)?;
    let mut stdin = process.child.stdin.take().ok_or("stdin missing")?;
    for frame in frames {
        serde_json::to_writer(&mut stdin, frame).map_err(|e| e.to_string())?;
        stdin.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    drop(stdin);
    process.finish()
}

pub(super) fn messages(
    status: ExitStatus,
    stdout: &[u8],
    stderr: &[u8],
) -> Result<Vec<serde_json::Value>, String> {
    if !status.success() || !stderr.is_empty() {
        return Err(format!(
            "MCP process: {:?}; {}",
            status.code(),
            String::from_utf8_lossy(stderr)
        ));
    }
    stdout
        .split(|b| *b == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).map_err(|e| e.to_string()))
        .collect()
}
