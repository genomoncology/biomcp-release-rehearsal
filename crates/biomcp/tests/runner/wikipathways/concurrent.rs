//! Concurrent readers retain one complete catalog while replacement is admitted.
use super::{Check, Environment, GET, Value, budgets, call, catalog, fs, json, recording, seed};
pub(super) fn run() -> Check {
    use std::time::{Duration, Instant};
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env, catalog(true, false), catalog(false, false), 200)?;
    call(
        "wikipathways/concurrent/1",
        &env,
        &replay,
        &["sync", "wikipathways"],
        0,
    )?;
    seed(&env, catalog(true, true), catalog(false, true), 200)?;
    let (mut writer, observed, mut command) = writer(&env, &replay)?;
    let until = Instant::now() + Duration::from_secs(30);
    while fs::metadata(&observed).map_or(true, |m| m.len() == 0) {
        if Instant::now() > until || writer.try_wait().map_err(|e| e.to_string())?.is_some() {
            let _ = writer.kill();
            let _ = writer.wait();
            return Err("replacement never admitted its first catalog".into());
        }
        std::thread::yield_now();
    }
    let results = std::thread::scope(|scope| {
        let one = scope.spawn(|| {
            call(
                "wikipathways/concurrent/2",
                &env,
                &replay,
                &["search", "pathway", "Atlas", "--source", "wikipathways"],
                0,
            )
        });
        let two = scope.spawn(|| {
            call(
                "wikipathways/concurrent/3",
                &env,
                &replay,
                &["search", "pathway", "Atlas", "--source", "wikipathways"],
                0,
            )
        });
        [one.join(), two.join()]
    });
    let finished = writer.wait_with_output().map_err(|e| e.to_string())?;
    command
        .record(finished.status, &finished.stdout, "cli")
        .map_err(|e| e.to_string())?;
    if !finished.status.success() {
        return Err(format!(
            "replacement failed {:?}: stdout={} stderr={}",
            finished.status.code(),
            String::from_utf8_lossy(&finished.stdout),
            String::from_utf8_lossy(&finished.stderr)
        ));
    }
    let receipt = serde_json::from_slice(&finished.stdout).map_err(|e| e.to_string())?;
    budgets::check(
        &receipt,
        "wikipathways/concurrent/1: sync wikipathways",
        "cli",
    )?;
    for result in results {
        let result = result.map_err(|_| "reader panicked")??;
        check_reader(&result)?;
    }
    Ok(())
}
fn check_reader(result: &Value) -> Check {
    let names = result
        .pointer("/result/items")
        .and_then(Value::as_array)
        .ok_or("items")?
        .iter()
        .filter_map(|r| r.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    if names.len() != 5
        || !(names.iter().all(|n| n.starts_with("Literal Atlas"))
            || names.iter().all(|n| n.starts_with("New Atlas")))
    {
        return Err(format!("reader observed partial replacement: {result}"));
    }
    Ok(())
}

fn writer(
    env: &Environment,
    replay: &std::path::Path,
) -> Result<
    (
        std::process::Child,
        std::path::PathBuf,
        super::super::capture::Capture,
    ),
    String,
> {
    use std::process::Stdio;
    let path = recording(replay, GET);
    let mut reply: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    *reply.pointer_mut("/exchanges/0/delay_ms").ok_or("delay")? = json!(1500);
    fs::write(path, reply.to_string()).map_err(|e| e.to_string())?;
    let observed = env.root().join("replacement-observations.jsonl");
    let mut args = env.globals(Some(replay), false);
    args.extend([
        "sync".into(),
        "wikipathways".into(),
        "--observe".into(),
        observed.to_string_lossy().into_owned(),
    ]);
    let mut command = env.command(env!("CARGO_BIN_EXE_biomcp"), &args);
    let writer = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok((writer, observed, command))
}
