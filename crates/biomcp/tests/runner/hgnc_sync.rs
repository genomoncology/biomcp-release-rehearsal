//! Public CLI regressions use fresh homes and a held CONNECT to stop a real sync.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{
    fs,
    net::{TcpListener, TcpStream},
    process::{Child, Stdio},
    time::{Duration, Instant},
};

type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("cache-resilience/hgnc-trials-during-live-sync", || {
            live().map_err(Into::into)
        }),
        transcript_trial("cache-resilience/hgnc-trials-after-killed-sync", || {
            abandoned().map_err(Into::into)
        }),
        transcript_trial(
            "cache-resilience/hgnc-first-snapshot-wait-is-bounded",
            || bounded().map_err(Into::into),
        ),
        transcript_trial("cache-resilience/hgnc-first-snapshot-recovers", || {
            first().map_err(Into::into)
        }),
    ]
}
fn stub(env: &Environment) -> Result<Stub, String> {
    let capture = super::launch::fixtures().join("replies/trial/gene/hgnc");
    let mut bodies = Vec::new();
    for file in fs::read_dir(capture).map_err(|e| e.to_string())? {
        let record: Value = serde_json::from_slice(
            &fs::read(file.map_err(|e| e.to_string())?.path()).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        bodies.push((
            record
                .get("url")
                .and_then(Value::as_str)
                .ok_or("HGNC URL")?
                .ends_with("withdrawn.txt"),
            record
                .pointer("/exchanges/0/body/value")
                .and_then(Value::as_str)
                .ok_or("HGNC body")?
                .to_owned(),
        ));
    }
    let stub = Stub::for_sources(
        &["storage.googleapis.com", "clinicaltrials.gov"],
        |r| {
            if r.contains("/api/v2/") {
                "application/json"
            } else {
                "text/tab-separated-values"
            }
        },
        move |r| {
            if r.contains("/api/v2/version") {
                return Ok(r#"{"dataTimestamp":"2026-10-30T00:00:00Z"}"#.into());
            }
            if r.contains("/api/v2/studies") {
                return Ok(r#"{"studies":[],"totalCount":0}"#.into());
            }
            bodies
                .iter()
                .find(|(withdrawn, _)| *withdrawn == r.contains("withdrawn.txt"))
                .map(|(_, body)| body.clone())
                .ok_or("missing HGNC capture".into())
        },
    )?;
    configure(env, &stub)?;
    Ok(stub)
}
fn configure(env: &Environment, stub: &Stub) -> Check {
    stub.retain(env.shared_root());
    let cert = env.root().join("ca.pem");
    fs::write(&cert, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":cert}))
        .map_err(|e| e.to_string())
}
fn args(phase: &str) -> Vec<String> {
    [
        "search", "trial", "--gene", "BRAF", "--phase", phase, "--fresh",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
fn trial(env: &Environment, phase: &str) -> Check {
    let out = invoke(env, args(phase), 0)?;
    if out.get("status") != Some(&json!("complete")) {
        return Err(format!("gene trial search failed: {out}"));
    }
    Ok(())
}
struct Pending {
    child: Child,
    _socket: TcpStream,
}
impl Pending {
    fn start(env: &Environment) -> Result<Self, String> {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        env.config(
            &json!({"proxy":format!("http://{}",listener.local_addr().map_err(|e|e.to_string())?)}),
        )
        .map_err(|e| e.to_string())?;
        let mut child = env
            .command(
                env!("CARGO_BIN_EXE_biomcp"),
                &["sync".into(), "hgnc".into()],
            )
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| e.to_string())?;
        let end = Instant::now() + Duration::from_secs(15);
        loop {
            match listener.accept() {
                Ok((socket, _)) => {
                    return Ok(Self {
                        child,
                        _socket: socket,
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < end => {
                    if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
                        return Err(format!("sync exited before CONNECT: {status}"));
                    }
                    std::thread::park_timeout(Duration::from_millis(1));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("held sync watchdog: {e}"));
                }
            }
        }
    }
    fn kill(&mut self) -> Check {
        self.child.kill().map_err(|e| e.to_string())?;
        self.child.wait().map_err(|e| e.to_string())?;
        Ok(())
    }
}
impl Drop for Pending {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn database(env: &Environment) -> Result<rusqlite::Connection, String> {
    let dir = env.root().join("cache/biomcp");
    let path = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "sqlite"))
        .ok_or("cache missing")?;
    rusqlite::Connection::open(path).map_err(|e| e.to_string())
}
fn live() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let peer = stub(&env)?;
    trial(&env, "2")?;
    let _pending = Pending::start(&env)?;
    configure(&env, &peer)?;
    let db = database(&env)?;
    let holder: String = db
        .query_row(
            "SELECT holder FROM bundle_sync WHERE name='hgnc'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    trial(&env, "3")?;
    let current: String = db
        .query_row(
            "SELECT holder FROM bundle_sync WHERE name='hgnc'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if current != holder {
        return Err("reader stole active sync".into());
    }
    peer.finish()?;
    Ok(())
}
fn abandoned() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let peer = stub(&env)?;
    trial(&env, "2")?;
    let mut pending = Pending::start(&env)?;
    pending.kill()?;
    configure(&env, &peer)?;
    trial(&env, "3")?;
    peer.finish()?;
    Ok(())
}
fn first() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let mut pending = Pending::start(&env)?;
    let peer = stub(&env)?;
    let diagnostic = env.root().join("sqlite-errors.jsonl");
    fs::write(&diagnostic, []).map_err(|e| e.to_string())?;
    let failure = |stage: &str, detail: String| {
        format!("first snapshot {stage}: {detail}; SQLite diagnostics: {}",
            fs::read_to_string(&diagnostic).unwrap_or_default())
    };
    let mut command = env.command(env!("CARGO_BIN_EXE_biomcp"), &args("2"));
    command.env(biomcp_fetch::SQLITE_DIAGNOSTICS, &diagnostic);
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| failure("spawn failed", e.to_string()))?;
    pending.kill().map_err(|e| failure("publisher stop failed", e))?;
    let output = command.wait_with_output(child)
        .map_err(|e| failure("wait failed", e.to_string()))?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(failure("failed", format!("{}; stderr: {}",
            String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr))));
    }
    let result: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| failure("JSON failed", e.to_string()))?;
    if result.get("status") != Some(&json!("complete")) {
        return Err(failure("did not recover completely", result.to_string()));
    }
    peer.finish()?;
    Ok(())
}

// The public query API admits a short command budget that the CLI cannot set.
fn bounded() -> Check {
    use biomcp_fetch::{Fetch, FetchConfig, Secret};
    let env = Environment::new().map_err(|e| e.to_string())?;
    let _pending = Pending::start(&env)?;
    let db = database(&env)?;
    let path: String = db
        .query_row("PRAGMA database_list", [], |r| r.get(2))
        .map_err(|e| e.to_string())?;
    let fetch = Fetch::open(FetchConfig {
        cache: Some(path.into()),
        cache_owners: vec![biomcp_query::store::OWNER],
        proxy: Some(Secret::new("http://127.0.0.1:9")),
        ..Default::default()
    })
    .map_err(|e| e.to_string())?;
    let request = biomcp_types::Request::GetGene(
        biomcp_types::GeneGet::new(Box::new(["BRAF".parse().map_err(|e: biodata::Error| e.to_string())?])).map_err(|e| e.to_string())?
    );
    let result = biomcp_query::run(&biomcp_query::Ctx { fetch: &fetch, budget: Duration::from_millis(30), ncbi_email: None, fresh: false }, &request);
    if !result.sources.iter().any(|source| source.source == biomcp_types::Source::Hgnc && source.reason == Some(biomcp_types::Reason::Syncing)) {
        return Err(format!("first snapshot did not wait within its deadline: {result:?}"));
    }
    Ok(())
}
