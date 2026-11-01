//! One local peer owns only the synthetic environments explicitly attached to it.
#![expect(
    clippy::disallowed_types,
    reason = "The local fixture shares I/O diagnostics and owned scratch with its worker thread"
)]
use serde::Serialize;
use std::{
    fs, io,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Debug, Serialize)]
struct Context {
    local: SocketAddr,
    peer: Option<SocketAddr>,
    hosts: Vec<String>,
    host: Option<String>,
    ordinal: usize,
    phase: &'static str,
    io_error: Option<String>,
}
#[derive(Debug)]
pub(crate) struct Evidence {
    context: Mutex<Context>,
    roots: Mutex<Vec<Arc<tempfile::TempDir>>>,
}
impl Evidence {
    pub(crate) fn new(local: SocketAddr, hosts: &[&str]) -> Arc<Self> {
        Arc::new(Self {
            context: Mutex::new(Context {
                local,
                peer: None,
                hosts: hosts.iter().map(|s| (*s).into()).collect(),
                host: None,
                ordinal: 0,
                phase: "accept",
                io_error: None,
            }),
            roots: Mutex::new(Vec::new()),
        })
    }
    pub(crate) fn attach(&self, root: Arc<tempfile::TempDir>) {
        let mut roots = self
            .roots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !roots.iter().any(|other| Arc::ptr_eq(other, &root)) {
            roots.push(root);
        }
    }
    pub(crate) fn accept(&self, stream: io::Result<TcpStream>) -> Result<TcpStream, String> {
        let socket = self.check("accept", stream)?;
        {
            let mut context = self
                .context
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            context.ordinal += 1;
            context.peer = socket.peer_addr().ok();
            context.host = None;
        }
        self.check(
            "read timeout setup",
            socket.set_read_timeout(Some(std::time::Duration::from_secs(10))),
        )?;
        self.check(
            "write timeout setup",
            socket.set_write_timeout(Some(std::time::Duration::from_secs(10))),
        )?;
        Ok(socket)
    }
    pub(crate) fn destination(&self, host: &str) {
        self.context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .host = Some(host.into());
    }
    pub(crate) fn check<T, E: std::fmt::Display>(
        &self,
        phase: &'static str,
        result: Result<T, E>,
    ) -> Result<T, String> {
        let mut context = self
            .context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        context.phase = phase;
        context.io_error = result.as_ref().err().map(ToString::to_string);
        result.map_err(|error| {
            format!(
                "source stub {phase}: {error}; local={} peer={:?} hosts={:?} host={:?} ordinal={}",
                context.local, context.peer, context.hosts, context.host, context.ordinal
            )
        })
    }
    pub(crate) fn preserve(&self, reason: &'static str) -> String {
        let context = self
            .context
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let roots = self
            .roots
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut locations = Vec::new();
        for root in roots.iter() {
            match retain(root.path(), &context, reason) {
                Ok(path) => locations.push(path.display().to_string()),
                Err(error) => locations.push(format!("retention failed: {error}")),
            }
        }
        let line = format!(
            "fixture {reason}: phase={} local={} peer={:?} hosts={:?} host={:?} ordinal={} io_error={:?}; evidence={locations:?}",
            context.phase,
            context.local,
            context.peer,
            context.hosts,
            context.host,
            context.ordinal,
            context.io_error
        );
        use std::io::Write;
        let _ = writeln!(io::stderr(), "{line}");
        line
    }
}
fn retain(root: &Path, context: &Context, reason: &str) -> io::Result<PathBuf> {
    let base = directory()?;
    let path = base.join(
        root.file_name()
            .ok_or_else(|| io::Error::other("missing fixture directory name"))?,
    );
    fs::create_dir(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    }
    copy(root, &path.join("scratch"))?;
    let mut manifest = serde_json::to_value(context)?;
    manifest
        .as_object_mut()
        .ok_or_else(|| io::Error::other("fixture context is not an object"))?
        .insert("reason".into(), reason.into());
    fs::write(
        path.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(path)
}
pub(crate) use crate::evidence::directory;

fn copy(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy(&entry.path(), &destination.join(entry.file_name()))?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination.join(entry.file_name()))?;
        }
        // Never follow a link out of an isolated fixture environment.
    }
    Ok(())
}

/// Keep actual replay inputs while their capture still owns the temporary directory.
pub(crate) fn retain_replay_owner(owner: &tempfile::TempDir) -> io::Result<()> {
    let root = owner.path().canonicalize()?;
    let retained = directory()?
        .join("replay-owners")
        .join(super::hash::hash(root.to_string_lossy().as_bytes()));
    fs::create_dir_all(&retained)?;
    let mut inputs = Vec::new();
    for name in ["recordings", "first", "second", "all", "replay"] {
        let source = root.join(name);
        if source.is_dir() {
            let destination = retained.join(name);
            if !destination.exists() {
                copy(&source, &destination)?;
            }
            inputs.push(name);
        }
    }
    fs::write(
        retained.join("manifest.json"),
        serde_json::to_vec(&serde_json::json!({"owner_root":root,"inputs":inputs}))?,
    )?;
    Ok(())
}
