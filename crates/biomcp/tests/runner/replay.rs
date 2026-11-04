//! Feature gene-identity: replay misses stay source failures and repeated server calls use checked cache.
use super::configuration::{cached, invoke};
use super::{launch::Environment, mcp};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
use std::{
    io::{BufRead, BufReader, Write},
    process::Stdio,
    sync::mpsc,
    thread,
    time::Duration,
};

pub(super) fn trials() -> Vec<Trial> {
    // These calls share one live server; independent CLI invocations cannot expose its replay cursor.
    vec![
        transcript_trial("article-search/concurrent-process-pool-publication", || {
            article_processes().map_err(Into::into)
        }),
        transcript_trial("gene-identity/replay-server-reuses-checked-reply", || {
            repeated().map_err(Into::into)
        }),
        transcript_trial("article-search/persistent-replay-command-epochs", || {
            article_replay_epoch().map_err(Into::into)
        }),
    ]
}
fn repeated() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let records = env.root().join("known-recordings");
    super::response_cases::cache_capture(
        "gene/main",
        &records,
        super::response_cases::CACHE_ACQUIRED,
    )?;
    let mut args = vec!["serve".into()];
    args.extend(env.globals(Some(&records), false));
    let mut child = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut input = child.stdin.take().ok_or("stdin missing")?;
    let output = child.stdout.take().ok_or("stdout missing")?;
    let mut process = super::process::Process::new(child)?;
    let (tx, rx) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(output).lines() {
            if tx.send(line.map_err(|e| e.to_string())).is_err() {
                break;
            }
        }
    });
    let result = repeat_calls(&mut input, &rx);
    if result.is_err() {
        let _ = process.child.kill();
    }
    drop(input);
    let finish = process.finish();
    reader.join().map_err(|_| "reader panicked")?;
    result?;
    let (status, _, stderr) = finish?;
    if !status.success() || !stderr.is_empty() {
        return Err("server failed".into());
    }
    Ok(())
}
fn repeat_calls(
    input: &mut impl Write,
    rx: &mpsc::Receiver<Result<String, String>>,
) -> Result<(), String> {
    let mut first = None;
    // Measured MCP text: 2365, 2365, 2672 and 1428 bytes, plus fifteen percent.
    for (id, args, limit) in [
        (
            1,
            vec!["get", "gene", "BRAF"],
            super::output_limit("replay-gene-card")?,
        ),
        (
            2,
            vec!["get", "gene", "BRAF"],
            super::output_limit("replay-gene-card")?,
        ),
        (
            3,
            vec!["get", "gene", "BRAF", "protein"],
            super::output_limit("replay-gene-protein")?,
        ),
        (
            4,
            vec!["get", "gene", "BRAF", "--fresh"],
            super::output_limit("replay-gene-fresh")?,
        ),
    ] {
        let frame = mcp::request(
            json!(id),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args}}),
        );
        serde_json::to_writer(&mut *input, &frame).map_err(|e| e.to_string())?;
        input.write_all(b"\n").map_err(|e| e.to_string())?;
        input.flush().map_err(|e| e.to_string())?;
        let line = rx
            .recv_timeout(Duration::from_secs(30))
            .map_err(|_| "reply watchdog")??;
        let reply: Value = serde_json::from_str(&line).map_err(|e| e.to_string())?;
        let result = reply.get("result").ok_or("missing result")?;
        if reply.get("id") != Some(&json!(id)) || result.get("isError") != Some(&json!(id >= 3)) {
            return Err(format!("replay call failed: {reply}"));
        }
        let content = result.get("structuredContent").ok_or("missing content")?;
        let text = result
            .pointer("/content/0/text")
            .and_then(Value::as_str)
            .ok_or("missing text")?;
        if serde_json::from_str::<Value>(text).map_err(|e| e.to_string())? != *content {
            return Err("text differs from structured content".into());
        }
        if text.len() > limit {
            return Err("replay MCP text exceeded its budget".into());
        }
        if id >= 3 && !replay_miss(content) {
            return Err("new or exhausted request lost replay_miss".into());
        }
        if id == 1 {
            first = Some(content.clone());
        }
        if id == 2 && first.as_ref() != Some(content) {
            return Err("repeat changed reply".into());
        }
    }
    Ok(())
}

fn replay_miss(reply: &Value) -> bool {
    reply
        .get("sources")
        .and_then(Value::as_array)
        .is_some_and(|sources| {
            sources
                .iter()
                .any(|source| source.get("reason") == Some(&json!("replay-miss")))
        })
}

fn article_replay_epoch() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let root = env.root().join("recordings");
    let pubmed = root.join("pubmed");
    std::fs::create_dir_all(&pubmed).map_err(|e| e.to_string())?;
    let original =
        crate::runner::launch::fixtures().join("replies/article/search/synthetic-small/pubmed");
    for entry in std::fs::read_dir(original).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let mut value: Value =
            serde_json::from_slice(&std::fs::read(entry.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let exchanges = value
            .get_mut("exchanges")
            .and_then(Value::as_array_mut)
            .ok_or("exchanges missing")?;
        let first = exchanges.first().ok_or("exchange missing")?.clone();
        *exchanges = vec![first; 32];
        std::fs::write(
            pubmed.join(entry.file_name()),
            serde_json::to_vec(&value).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    article_sequence(&env, &env.globals(Some(&root), false))
}

fn article_sequence(environment: &Environment, globals: &[String]) -> Result<(), String> {
    use std::{
        io::{BufRead, BufReader},
        sync::mpsc,
        thread,
        time::Duration,
    };
    let mut args = vec!["serve".into()];
    args.extend_from_slice(globals);
    let mut child = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdin = child.stdin.take().ok_or("stdin missing")?;
    let stdout = child.stdout.take().ok_or("stdout missing")?;
    let mut process = super::process::Process::new(child)?;
    let (sender, receiver) = mpsc::channel();
    let reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if sender.send(line.map_err(|e| e.to_string())).is_err() {
                break;
            }
        }
    });
    let result: Result<(), String> = (|| {
        let mut pools = std::collections::BTreeSet::new();
        for batch in (1..=24)
            .map(|id| vec![id])
            .chain(std::iter::once(vec![25, 26]))
        {
            for id in &batch {
                let frame = mcp::request(
                    json!(id),
                    "tools/call",
                    json!({"name":"biomcp","arguments":{"args":["search","article","AKT1 E17K","--limit","2","--fresh"]}}),
                );
                serde_json::to_writer(&mut stdin, &frame).map_err(|e| e.to_string())?;
                stdin.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            stdin.flush().map_err(|e| e.to_string())?;
            let mut ids = std::collections::BTreeSet::new();
            for _ in &batch {
                let line = receiver
                    .recv_timeout(Duration::from_secs(30))
                    .map_err(|_| "MCP reply watchdog")??;
                check_article_reply(&line, &batch, &mut ids, &mut pools)?;
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = process.child.kill();
    }
    drop(stdin);
    let finish = process.finish();
    reader.join().map_err(|_| "reply reader panicked")?;
    result?;
    let (status, _, stderr) = finish?;
    if !status.success() || !stderr.is_empty() {
        return Err("persistent article server failed".into());
    }
    Ok(())
}

fn check_article_reply(
    line: &str,
    batch: &[u64],
    ids: &mut std::collections::BTreeSet<u64>,
    pools: &mut std::collections::BTreeSet<String>,
) -> Result<(), String> {
    let reply: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    let id = reply
        .get("id")
        .and_then(Value::as_u64)
        .ok_or("correlation missing")?;
    if !batch.contains(&id)
        || !ids.insert(id)
        || reply.pointer("/result/isError") != Some(&json!(false))
        || reply.pointer("/result/structuredContent/status") != Some(&json!("complete"))
    {
        return Err(format!("persistent article replay failed: {reply}"));
    }
    let pool = reply
        .pointer("/result/structuredContent/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("pool missing")?;
    if !pools.insert(pool.to_owned()) {
        return Err("fresh command reused pool".into());
    }
    Ok(())
}

fn article_processes() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let case = env.root().join("case");
    let fixture = env.root().join("known-recordings");
    super::response_cases::cache_capture(
        "article/search/synthetic-small",
        &fixture,
        super::response_cases::CACHE_ACQUIRED,
    )?;
    let base = ["search", "article", "AKT1 E17K", "--limit", "2"];
    let initial = invoke(&env, cached(&base, &case, Some(&fixture)), 0)?;
    let expected = initial.pointer("/result/items").ok_or("items missing")?;
    let parent = env.root().join("cache/biomcp");
    fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
    let file = parent.join(format!(
        "cache-{}-{}.sqlite",
        biomcp_fetch::CACHE_NAMESPACE_VERSION,
        biomcp_query::store::CACHE_NAMESPACE_VERSION
    ));
    // Replay and ordinary caches have the same owned schema. Copy only after
    // the seed process closes; it has checkpointed the WAL and released locks.
    fs::copy(case.join("replay.sqlite"), &file).map_err(|e| e.to_string())?;
    let fetch = biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
        cache: Some(file),
        mode: biomcp_fetch::Mode::Offline,
        cache_owners: vec![biomcp_query::store::OWNER],
        ..Default::default()
    })
    .map_err(|e| e.to_string())?;
    let connection = fetch
        .cache_access()
        .map_err(|e| e.to_string())?
        .connect()
        .map_err(|e| e.to_string())?;
    connection
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| e.to_string())?;
    let replies = super::publication::check(&env, || {
        connection
            .execute_batch("COMMIT")
            .map_err(|e| e.to_string())
    })?;
    let mut pools = std::collections::BTreeSet::new();
    for reply in replies {
        if reply.pointer("/result/items") != Some(expected) {
            return Err("concurrent ordinary-cache search lost items".into());
        }
        let pool = reply
            .pointer("/page/pool/id")
            .and_then(Value::as_str)
            .ok_or("published pool missing")?;
        if !pools.insert(pool.to_owned()) || initial.pointer("/page/pool/id") == Some(&json!(pool))
        {
            return Err("concurrent fresh pools collided".into());
        }
        let mut args: Vec<String> = base.iter().map(|v| (*v).into()).collect();
        args.extend(["--offline".into(), "--pool".into(), pool.into()]);
        if invoke(&env, args, 0)?.pointer("/result/items") != Some(expected) {
            return Err("published snapshot changed".into());
        }
    }
    Ok(())
}
