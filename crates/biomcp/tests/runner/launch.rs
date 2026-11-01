use std::{fs, io, path::Path, sync::Arc};

use biomcp_test_support::environment as child_environment;
pub(super) use child_environment::ENV_PATHS;

#[derive(Debug)]
pub(super) struct Environment {
    root: Arc<tempfile::TempDir>,
    replay_owner: Option<Arc<tempfile::TempDir>>,
    transcript: Option<Arc<super::transcript::Context>>,
}

impl Environment {
    pub(super) fn new() -> io::Result<Self> {
        let root = child_environment::new()?;
        Ok(Self {
            root: Arc::new(root),
            replay_owner: None,
            transcript: super::transcript::current(),
        })
    }

    pub(super) fn with_replay_owner(mut self, owner: Arc<tempfile::TempDir>) -> Self {
        self.replay_owner = Some(owner);
        self
    }

    pub(super) fn config(&self, value: &serde_json::Value) -> io::Result<()> {
        let directory = self.root.path().join("config/biomcp");
        fs::create_dir_all(&directory)?;
        fs::write(directory.join("config.json"), serde_json::to_vec(value)?)
    }
    pub(super) fn globals(&self, replay: Option<&Path>, offline: bool) -> Vec<String> {
        if offline {
            vec![
                "--offline".into(),
                "--case-cache".into(),
                self.root.path().join("case-cache").to_string_lossy().into(),
            ]
        } else if let Some(replay) = replay {
            vec![
                "--replay".into(),
                replay.to_string_lossy().into(),
                "--case-cache".into(),
                self.root.path().join("case-cache").to_string_lossy().into(),
            ]
        } else {
            Vec::new()
        }
    }
    pub(super) fn shared_root(&self) -> Arc<tempfile::TempDir> {
        Arc::clone(&self.root)
    }

    pub(super) fn root(&self) -> &Path {
        self.root.path()
    }

    pub(super) fn command(
        &self,
        program: impl AsRef<Path>,
        args: &[String],
    ) -> super::capture::Capture {
        self.command_with_arguments(program, args, args)
    }

    // All case steps and launch-boundary proofs use this child-only boundary.
    #[expect(
        clippy::disallowed_methods,
        reason = "The launcher alone selects an explicitly installed artifact"
    )]
    pub(super) fn command_with_arguments(
        &self,
        program: impl AsRef<Path>,
        args: &[String],
        transcript_args: &[String],
    ) -> super::capture::Capture {
        let mut command =
            child_environment::command(&executable(program.as_ref()), self.root.path(), args);
        let nested = std::env::current_exe().is_ok_and(|runner| runner == program.as_ref());
        if let Some(profile) = std::env::var_os("LLVM_PROFILE_FILE")
            && let Some(directory) = Path::new(&profile).parent()
        {
            // A nested test runner remains harness execution. Its own product
            // children pass through this boundary and receive binary profiles.
            let pattern = if nested {
                "harness-%m-%p.profraw"
            } else {
                "bin-%m-%p.profraw"
            };
            command.env("LLVM_PROFILE_FILE", directory.join(pattern));
        }
        if nested {
            // Nested runner proofs must retain the selection, never falling
            // back to Cargo's binary merely because a helper cleared its env.
            for key in ["BIOMCP_TEST_EXECUTABLE", "BIOMCP_TEST_FIXTURES"] {
                if let Some(value) = std::env::var_os(key) {
                    command.env(key, value);
                }
            }
        }
        super::capture::Capture::new(
            command,
            if !nested {
                self.transcript.clone()
            } else {
                None
            },
            transcript_args,
            self.root.path(),
            self.replay_owner.clone(),
        )
    }
}

pub(crate) use super::fixture_paths::fixtures;

// Provenance and execution use the same installed-artifact selection.
#[expect(
    clippy::disallowed_methods,
    reason = "The launcher alone selects an explicitly installed artifact"
)]
pub(super) fn executable(program: &Path) -> std::path::PathBuf {
    if program == Path::new(env!("CARGO_BIN_EXE_biomcp"))
        && let Some(installed) = std::env::var_os("BIOMCP_TEST_EXECUTABLE")
    {
        return installed.into();
    }
    program.to_owned()
}

pub(super) fn arguments(args: &[String], globals: &[String]) -> Vec<String> {
    let mut result = args.to_vec();
    let position = result
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(result.len());
    result.splice(position..position, globals.iter().cloned());
    result
}

// Only this module knows the build checkout; fixtures may be copied elsewhere.
pub(super) fn checkout() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

tool_only! {
pub(super) fn fixture_script(name: &str) -> std::path::PathBuf {
    let root = if std::env::var_os("BIOMCP_TEST_FIXTURES").is_some() {
        fixtures().join("support")
    } else {
        checkout().join("../..")
    };
    root.join("scripts").join(name)
}
}

pub(super) fn fixture_text(relative: &str) -> Result<String, String> {
    let path = fixtures().join(relative);
    fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

// Feature ownership travels with the runner when installed fixtures are copied.
pub(super) fn feature_list() -> &'static str {
    include_str!("../../../../sdlc/features.md")
}

// Native command ceilings travel with the copied fixtures for both test targets.
tool_only! {
pub(super) fn native_budget(key: &str) -> Result<usize, String> {
    let limits: std::collections::BTreeMap<String, usize> =
        serde_saphyr::from_str(&fixture_text("native-budgets.yaml")?)
            .map_err(|error| error.to_string())?;
    limits
        .get(key)
        .copied()
        .ok_or_else(|| format!("unmeasured native proof: {key}"))
}
}
