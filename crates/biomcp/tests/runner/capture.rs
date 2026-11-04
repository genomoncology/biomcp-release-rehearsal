//! Record ordinary command outputs without replacing child processes or pipes.
use super::transcript::Context;
use serde_json::json;
use std::{
    ffi::OsStr,
    io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Output, Stdio},
    sync::Arc,
};

pub(super) struct Capture {
    command: Command,
    context: Option<Arc<Context>>,
    key: Option<String>,
    sequence_start: Option<usize>,
    args: Vec<String>,
    stable_args: Vec<String>,
    spawned: bool,
    executable: String,
    root: String,
    replay_owner: Option<Arc<tempfile::TempDir>>,
    finished: bool,
}
impl Capture {
    pub(super) fn new(
        command: Command,
        context: Option<Arc<Context>>,
        args: &[String],
        root: &Path,
        replay_owner: Option<Arc<tempfile::TempDir>>,
    ) -> Self {
        let stable = args
            .iter()
            .map(|arg| {
                arg.replace(&root.to_string_lossy().to_string(), "<root>")
                    .replace(
                        crate::runner::launch::checkout().to_string_lossy().as_ref(),
                        "<checkout>/crates/biomcp",
                    )
            })
            .collect::<Vec<_>>();
        Self {
            executable: command.get_program().to_string_lossy().into(),
            command,
            context,
            key: None,
            sequence_start: None,
            stable_args: stable,
            spawned: false,
            args: args.to_vec(),
            root: root.to_string_lossy().into(),
            replay_owner,
            finished: false,
        }
    }
    fn prepare(&mut self) {
        self.key = self.context.as_ref().map(|c| c.allocate(&self.stable_args));
        self.sequence_start = self.context.as_ref().map(|c| c.sequence());
        self.spawned = true;
        self.finished = false;
    }
    pub(super) fn spawn(&mut self) -> io::Result<Child> {
        self.prepare();
        self.command.spawn()
    }
    pub(super) fn output(&mut self) -> io::Result<Output> {
        self.prepare();
        let output = self.command.output()?;
        self.record(output.status, &output.stdout, "cli")?;
        Ok(output)
    }
    pub(super) fn wait_with_output(&mut self, child: Child) -> io::Result<Output> {
        let output = child.wait_with_output()?;
        self.record(output.status, &output.stdout, "cli")?;
        Ok(output)
    }
    pub(super) fn record(
        &mut self,
        status: ExitStatus,
        stdout: &[u8],
        surface: &str,
    ) -> io::Result<()> {
        if let (Some(context), Some(key)) = (&self.context, &self.key) {
            let mut execution = json!({"args":self.args,"surface":surface,
                "exit":status.code(),"stdout":stdout,"sequence_start":self.sequence_start,
                "executable":self.executable,"root":self.root,
                "checkout":crate::runner::launch::checkout().to_string_lossy().as_ref(),
                "version":env!("CARGO_PKG_VERSION")});
            if let Some((owner, root)) = self.owned_replay()? {
                super::network_stub::retain_replay_owner(owner)?;
                execution
                    .as_object_mut()
                    .ok_or_else(|| io::Error::other("execution object"))?
                    .insert("replay_owner_root".into(), json!(root));
            }
            context.record(key, execution).map_err(io::Error::other)?;
        }
        self.finished = true;
        Ok(())
    }
    fn owned_replay(&self) -> io::Result<Option<(&tempfile::TempDir, PathBuf)>> {
        let Some(owner) = self.replay_owner.as_deref() else {
            return Ok(None);
        };
        let mut flags = self
            .args
            .iter()
            .enumerate()
            .take_while(|(_, arg)| arg.as_str() != "--")
            .filter(|(_, arg)| arg.as_str() == "--replay");
        let Some((index, _)) = flags.next() else {
            return Ok(None);
        };
        if flags.next().is_some() {
            return Ok(None);
        }
        let root = owner.path().canonicalize()?;
        let allowed = [root.join("recordings"), root.join("all")];
        if self.args.get(index + 1).is_none_or(|arg| {
            !allowed
                .iter()
                .any(|path| path.to_str() == Some(arg.as_str()))
        }) {
            return Ok(None);
        }
        Ok(Some((owner, root)))
    }

    pub(super) fn stdin(&mut self, value: impl Into<Stdio>) -> &mut Self {
        self.command.stdin(value);
        self
    }
    pub(super) fn stdout(&mut self, value: impl Into<Stdio>) -> &mut Self {
        self.command.stdout(value);
        self
    }
    pub(super) fn stderr(&mut self, value: impl Into<Stdio>) -> &mut Self {
        self.command.stderr(value);
        self
    }
    pub(super) fn env(&mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> &mut Self {
        self.command.env(key, value);
        self
    }
}
impl Deref for Capture {
    type Target = Command;
    fn deref(&self) -> &Command {
        &self.command
    }
}
impl DerefMut for Capture {
    fn deref_mut(&mut self) -> &mut Command {
        &mut self.command
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        if self.spawned
            && !self.finished
            && let Some(context) = &self.context
        {
            context.uncaptured();
        }
    }
}
