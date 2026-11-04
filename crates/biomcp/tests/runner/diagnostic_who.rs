//! Feature diagnostic-who: exact cards consume a complete explicit stored export.

mod search;
use super::{configuration::invoke, launch::Environment};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    // The test follows runtime revision-bound pages and compares the complete candidate set without replacement requests.
    vec![
        transcript_trial(
            "diagnostic-who/two-readers-during-atomic-replacement",
            || concurrent().map_err(Into::into),
        ),
        transcript_trial(crate::runner::pool_contract::WHO_REVISION.0, || {
            search::walk().map_err(Into::into)
        }),
    ]
}
fn call(env: &Environment, replay: &Path, args: &[&str], exit: i32) -> Result<Value, String> {
    let mut command = env.globals(Some(replay), false);
    command.extend(args.iter().map(|s| (*s).into()));
    invoke(env, command, exit)
}

fn seed(env: &Environment) -> Result<PathBuf, String> {
    let replay = env.root().join("replay");
    fs::create_dir_all(replay.join("who-ivd")).map_err(|e| e.to_string())?;
    for entry in
        fs::read_dir(super::launch::fixtures().join("replies/diagnostic/who-synthetic/who-ivd"))
            .map_err(|e| e.to_string())?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        fs::copy(entry.path(), replay.join("who-ivd").join(entry.file_name()))
            .map_err(|e| e.to_string())?;
    }
    Ok(replay)
}

fn body(replay: &std::path::Path) -> Result<String, String> {
    let path = fs::read_dir(replay.join("who-ivd"))
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("missing recording")?
        .map_err(|e| e.to_string())?
        .path();
    let value: Value = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    value
        .pointer("/exchanges/0/body/value")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or("missing CSV".into())
}

pub(super) fn concurrent() -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env)?;
    let args = ["get", "diagnostic", "0466-046-00", "--source", "who-ivd"];
    let original = body(&replay)?;
    let replacement = original.replace("CODE-8", "REPLACED-8");
    let (admitted, receiving) = std::sync::mpsc::channel();
    let (release, released) = std::sync::mpsc::channel();
    let mut calls = 0;
    let stub = super::network_stub::Stub::for_sources(
        &["extranet.who.int"],
        |_| "text/csv",
        move |_| {
            calls += 1;
            if calls == 1 {
                return Ok(original.clone());
            }
            admitted.send(()).map_err(|e| e.to_string())?;
            released
                .recv_timeout(std::time::Duration::from_secs(20))
                .map_err(|e| e.to_string())?;
            Ok(replacement.clone())
        },
    )?;
    let cert = env.root().join("authority.pem");
    stub.retain(env.shared_root());
    fs::write(&cert, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":cert}))
        .map_err(|e| e.to_string())?;
    let plain = |args: &[&str]| invoke(&env, args.iter().map(|s| (*s).into()).collect(), 0);
    plain(&["sync", "who-ivd"])?;
    let before = plain(&args)?;
    std::thread::scope(|scope| -> Check {
        let sync = scope.spawn(|| plain(&["sync", "who-ivd"]));
        receiving
            .recv_timeout(std::time::Duration::from_secs(20))
            .map_err(|e| e.to_string())?;
        let readers = scope.spawn(|| plain(&args));
        let other = plain(&args);
        let first = readers.join().map_err(|_| "reader panicked")?;
        release.send(()).map_err(|e| e.to_string())?;
        sync.join().map_err(|_| "sync panicked")??;
        if first? != before || other? != before {
            return Err("reader observed partial replacement".into());
        }
        Ok(())
    })?;
    let after = plain(&args)?;
    if after.pointer("/result/native_rows/8/product_code") != Some(&json!("REPLACED-8"))
        || stub.finish()?.len() != 2
    {
        return Err("replacement failed or a stored read made a request".into());
    }
    Ok(())
}
