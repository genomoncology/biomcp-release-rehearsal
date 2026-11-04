//! Persistent servers reload credential snapshots, verified at the local TLS wire.
use super::{key_registration::invoke, launch::Environment, mcp, network_stub::Stub};
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader, Write},
    process::Stdio,
    sync::mpsc,
    thread,
    time::Duration,
};
const ENV: &str = "synthetic-registration-environment";
const FIRST: &str = "synthetic-registration-first";
const SECOND: &str = "synthetic-registration-second";
type Check = Result<(), String>;
pub(super) enum Client {
    Stdio {
        environment: Environment,
        process: super::process::Process,
        input: std::process::ChildStdin,
        replies: mpsc::Receiver<Result<String, String>>,
        reader: thread::JoinHandle<()>,
    },
    Http(super::http::Server),
}
impl Client {
    pub(super) fn environment(&self) -> &Environment {
        match self {
            Self::Stdio { environment, .. } => environment,
            Self::Http(server) => server.environment(),
        }
    }
    pub(super) fn call(&mut self, id: usize, args: &[&str]) -> Result<Value, String> {
        let frame = mcp::request(
            json!(id),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args}}),
        );
        let value = match self {
            Self::Http(server) => server.request("POST", "/mcp", &frame.to_string())?.1,
            Self::Stdio { input, replies, .. } => {
                serde_json::to_writer(&mut *input, &frame).map_err(|e| e.to_string())?;
                input.write_all(b"\n").map_err(|e| e.to_string())?;
                input.flush().map_err(|e| e.to_string())?;
                let line = replies
                    .recv_timeout(Duration::from_secs(20))
                    .map_err(|_| "key reload reply watchdog")??;
                serde_json::from_str(&line).map_err(|e| e.to_string())?
            }
        };
        for marker in [ENV, FIRST, SECOND] {
            if value.to_string().contains(marker) {
                return Err("key leaked into MCP result".into());
            }
        }
        if value.get("id") != Some(&json!(id)) {
            return Err("key reload correlation".into());
        }
        value
            .pointer("/result/structuredContent")
            .cloned()
            .ok_or("missing key reload tool result".into())
    }
    pub(super) fn finish(self) -> Check {
        match self {
            Self::Http(mut server) => server.stop(),
            Self::Stdio {
                process,
                input,
                reader,
                ..
            } => {
                drop(input);
                let (status, _, stderr) = process.finish()?;
                reader.join().map_err(|_| "key reload reader")?;
                if !status.success() || !stderr.is_empty() {
                    return Err("key reload server failed".into());
                }
                Ok(())
            }
        }
    }
}
pub(super) fn start(
    http: bool,
    fallback: bool,
    config: Value,
    variable: &str,
) -> Result<Client, String> {
    if http {
        return Ok(Client::Http(super::http::Server::keys(
            config,
            fallback.then_some((variable, ENV)),
        )?));
    }
    let environment = Environment::new().map_err(|e| e.to_string())?;
    environment.config(&config).map_err(|e| e.to_string())?;
    let mut command = environment.command(env!("CARGO_BIN_EXE_biomcp"), &["serve".into()]);
    if fallback {
        command.env(variable, ENV);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let input = child.stdin.take().ok_or("stdin")?;
    let output = child.stdout.take().ok_or("stdout")?;
    let process = super::process::Process::new(child)?;
    let (sender, replies) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(output).lines() {
            if sender.send(line.map_err(|e| e.to_string())).is_err() {
                break;
            }
        }
    });
    Ok(Client::Stdio {
        environment,
        process,
        input,
        replies,
        reader,
    })
}
pub(super) fn check(http: bool, fallback: bool) -> Check {
    check_source(http, fallback, false)
}

fn check_source(http: bool, fallback: bool, nci: bool) -> Check {
    let assets = Environment::new().map_err(|e| e.to_string())?;
    let source = if nci { "nci" } else { "semantic-scholar" };
    let variable = if nci { "NCI_API_KEY" } else { "S2_API_KEY" };
    let stub = Stub::for_source(
        if nci {
            "clinicaltrialsapi.cancer.gov"
        } else {
            "api.semanticscholar.org"
        },
        "application/json",
        |_| Ok(r#"{"total":0,"data":[]}"#.into()),
    )?;
    let certificate = assets.root().join("ca.pem");
    stub.retain(assets.shared_root());
    std::fs::write(&certificate, &stub.certificate).map_err(|e| e.to_string())?;
    let mut client = start(
        http,
        fallback,
        json!({"proxy":format!("http://{}",stub.address),"ca_bundle":certificate,"disable_sources":["reactome"]}),
        variable,
    )?;
    let mut expected = Vec::new();
    for (index, key) in [None, Some(FIRST), Some(SECOND), None]
        .into_iter()
        .enumerate()
    {
        match index {
            1 => {
                invoke(
                    client.environment(),
                    &["key", "set", source],
                    &format!("{FIRST}\n"),
                    0,
                )?;
            }
            2 => {
                invoke(
                    client.environment(),
                    &["key", "set", source, "--replace"],
                    &format!("{SECOND}\n"),
                    0,
                )?;
            }
            3 => {
                invoke(client.environment(), &["key", "remove", source], "", 0)?;
            }
            _ => {}
        }
        let selected = key.or(fallback.then_some(ENV));
        let args = query(index, nci, source);
        let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
        let result = client.call(index, &refs)?;
        if result.pointer("/sources/0/reason") == Some(&json!("no-key")) {
            if selected.is_some() {
                return Err("server failed to reload key".into());
            }
        } else {
            if result.pointer("/result/items").is_some() {
                return Err(format!("local synthetic query failed: {result}"));
            }
            expected.push(selected);
        }
    }
    let refusal = client.call(99, &["key", "set", source])?;
    if refusal.get("status") != Some(&json!("usage")) {
        return Err("MCP accepted key management".into());
    }
    client.finish()?;
    check_wire(stub, expected)
}
fn check_wire(stub: Stub, expected: Vec<Option<&str>>) -> Check {
    let requests = stub.finish()?;
    if requests.len() != expected.len() {
        return Err("reload changed request count".into());
    }
    for ((_, request), key) in requests.iter().zip(expected) {
        let actual = request.lines().find_map(|line| {
            line.split_once(':')
                .filter(|(name, _)| name.eq_ignore_ascii_case("x-api-key"))
                .map(|(_, value)| value.trim())
        });
        if actual != key {
            return Err("server sent a stale or wrong-precedence key".into());
        }
    }
    Ok(())
}

fn query(index: usize, nci: bool, source: &str) -> Vec<String> {
    if nci {
        vec![
            "search".into(),
            "trial".into(),
            "--source".into(),
            source.into(),
            "--keyword".into(),
            format!("synthetic {index}"),
        ]
    } else {
        vec![
            "search".into(),
            "article".into(),
            format!("synthetic EGFR {index}"),
            "--source".into(),
            source.into(),
        ]
    }
}
