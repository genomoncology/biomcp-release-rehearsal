//! First readers wait for an admitted publisher; no fixture sleeps release it.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, io::Write, process::Stdio, sync::mpsc, time::Duration};

pub(super) fn trials() -> Vec<Trial> {
    vec![
        super::transcript::trial("gene-sections/gencc-first-publisher", || {
            first_reader(false).map_err(Into::into)
        }),
        super::transcript::trial("gene-sections/gencc-first-publisher-deadline", || {
            first_reader(true).map_err(Into::into)
        }),
    ]
}
fn first_reader(expires: bool) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = super::launch::fixtures().join("replies/gene/gencc");
    let body = native_body(&replay, "gencc")?;
    let hgnc = native_body(&replay, "hgnc")?;
    let (admitted, admission) = mpsc::channel();
    let (publish, publication) = mpsc::channel();
    let peer = Stub::for_sources(
        &["thegencc.org", "storage.googleapis.com"],
        |request| {
            if request.contains("hgnc_complete_set") {
                "text/tab-separated-values"
            } else {
                "text/csv"
            }
        },
        move |request| {
            if request.contains("hgnc_complete_set") {
                return Ok(hgnc.clone());
            }
            admitted.send(()).map_err(|e| e.to_string())?;
            publication
                .recv_timeout(Duration::from_secs(30))
                .map_err(|e| e.to_string())?;
            Ok(body.clone())
        },
    )?;
    peer.retain(env.shared_root());
    let ca = env.root().join("ca.pem");
    fs::write(&ca, &peer.certificate).map_err(|e| e.to_string())?;
    let mut config = json!({"proxy":format!("http://{}",peer.address),"ca_bundle":ca,
        "disable_sources":["oncokb"]});
    env.config(&config).map_err(|e| e.to_string())?;
    invoke(&env, vec!["get".into(), "gene".into(), "ODC1".into()], 0)?;
    let mut command = env.command(
        env!("CARGO_BIN_EXE_biomcp"),
        &["sync".into(), "gencc".into()],
    );
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    if let Err(error) = admission.recv_timeout(Duration::from_secs(15)) {
        let _ = child.kill();
        let output = command.wait_with_output(child).map_err(|e| e.to_string())?;
        return Err(format!(
            "publisher admission failed: {error}; stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    if expires {
        config
            .as_object_mut()
            .ok_or("config map")?
            .insert("command_timeout_seconds".into(), json!(1));
        env.config(&config).map_err(|e| e.to_string())?;
    }
    let result = reader(&env, publish, expires);
    let output = command.wait_with_output(child).map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "publisher failed: {}",
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    if result.is_err() {
        let retained =
            biomcp_test_support::evidence::retain(env.root()).map_err(|e| e.to_string())?;
        writeln!(
            std::io::stderr(),
            "GenCC reader evidence: {}",
            retained.display()
        )
        .map_err(|e| e.to_string())?;
    }
    peer.finish()?;
    result
}

pub(super) fn native_body(replay: &std::path::Path, source: &str) -> Result<String, String> {
    let recording = fs::read_dir(replay.join(source))
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("GenCC recording missing")?
        .map_err(|e| e.to_string())?
        .path();
    let record: Value = serde_json::from_slice(&fs::read(recording).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    record
        .get("exchanges")
        .and_then(Value::as_array)
        .ok_or("GenCC exchanges")?
        .iter()
        .find(|exchange| exchange.get("status") == Some(&json!(200)))
        .and_then(|exchange| exchange.pointer("/body/value"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "GenCC CSV".into())
}

fn reader(env: &Environment, publish: mpsc::Sender<()>, expires: bool) -> Result<(), String> {
    let observations = env.root().join("reader-observations.jsonl");
    std::thread::scope(|scope| {
        let (done, completion) = mpsc::channel();
        let environment = env;
        let destination = &observations;
        scope.spawn(move || {
            let result = read_result(environment, destination, expires);
            let _ = done.send(result);
        });
        let admission = if expires {
            // Wait admission is an observation event, never file creation.
            // The deadline can also end during cache access before that event.
            let end = std::time::Instant::now() + Duration::from_secs(15);
            loop {
                if reader_waiting(&observations) {
                    break completion
                        .recv_timeout(Duration::from_secs(15))
                        .map(Some)
                        .map_err(|e| e.to_string());
                }
                match completion.try_recv() {
                    Ok(value) => break Ok(Some(value)),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        break Err("reader disconnected before wait admission".into());
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }
                if std::time::Instant::now() >= end {
                    break Err("reader neither entered bundle wait nor completed".into());
                }
                std::thread::yield_now();
            }
        } else {
            wait_for_reader(&observations, &completion).map(|()| None)
        };
        // Only actual wait admission, premature completion, or the configured
        // deadline releases publication. A startup delay cannot produce a pass.
        publish.send(()).map_err(|e| e.to_string())?;
        let value = match admission? {
            Some(result) => result?,
            None => completion
                .recv_timeout(Duration::from_secs(15))
                .map_err(|e| e.to_string())??,
        };
        let valid = if expires {
            value.get("local_error") == Some(&json!("busy"))
                || value
                    .get("sources")
                    .and_then(Value::as_array)
                    .is_some_and(|sources| {
                        sources.iter().any(|source| {
                            source.get("source") == Some(&json!("gencc"))
                                && source.get("reason") == Some(&json!("syncing"))
                        })
                    })
        } else {
            value.pointer("/result/items/0/sections/gencc/state") == Some(&json!("data"))
        };
        if !valid {
            return Err(format!(
                "first reader did not respect publication and deadline: {value}"
            ));
        }
        Ok(())
    })
}
pub(super) fn wait_for_reader(
    path: &std::path::Path,
    completion: &mpsc::Receiver<Result<Value, String>>,
) -> Result<(), String> {
    let end = std::time::Instant::now() + Duration::from_secs(15);
    loop {
        if reader_waiting(path) {
            return Ok(());
        }
        match completion.try_recv() {
            Ok(value) => return Err(format!("reader completed before wait admission: {value:?}")),
            Err(mpsc::TryRecvError::Disconnected) => {
                return Err("reader disconnected before wait admission".into());
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if std::time::Instant::now() >= end {
            return Err("reader never entered bundle wait".into());
        }
        std::thread::yield_now();
    }
}

fn reader_waiting(path: &std::path::Path) -> bool {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .any(|event| event.get("role") == Some(&json!("bundle-wait")))
}

fn read_result(
    env: &Environment,
    observations: &std::path::Path,
    expires: bool,
) -> Result<Value, String> {
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &[
                "get".into(),
                "gene".into(),
                "ODC1".into(),
                "gencc".into(),
                "--observe".into(),
                observations.to_string_lossy().into(),
            ],
        )
        .output()
        .map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let busy = expires
        && output.status.code() == Some(5)
        && value.get("local_error") == Some(&json!("busy"));
    let phase = json!({"bundle_wait": reader_waiting(observations),
        "exit": output.status.code(), "local_error": value.get("local_error")});
    fs::write(env.root().join("reader-phase.json"), phase.to_string())
        .map_err(|e| e.to_string())?;
    writeln!(std::io::stdout(), "GENCC_READER_PHASE {phase}").map_err(|e| e.to_string())?;
    if ((!output.status.success() || value.get("local_error").is_some()) && !busy)
        || !output.stderr.is_empty()
    {
        return Err(format!(
            "reader failed: {phase}; stderr: {}; stdout: {value}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(value)
}
