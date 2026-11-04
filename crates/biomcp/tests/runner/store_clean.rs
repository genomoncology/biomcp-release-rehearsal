//! Cleanup is exercised through the isolated real command and shared SQLite file.
use super::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::path::PathBuf;
type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        transcript_trial("store-clean/expired-owned-state", || {
            clean().map_err(|e| e.to_string().into())
        }),
        transcript_trial("store-clean/publish-preserves-live-reader", || {
            publish().map_err(|e| e.to_string().into())
        }),
    ]
}
fn run(env: &Environment, args: &[&str]) -> Check<Value> {
    let args: Vec<_> = args.iter().map(|s| (*s).to_owned()).collect();
    let output = env.command(env!("CARGO_BIN_EXE_biomcp"), &args).output()?;
    let value: Value = serde_json::from_slice(&output.stdout)?;
    if !output.status.success() {
        return Err(format!("command failed: {value}").into());
    }
    if args.first().is_some_and(|s| s == "store")
        && output.stdout.len() > crate::runner::output_limit("store-clean")?
    {
        return Err(format!("cleanup exceeded 250 bytes: {}", output.stdout.len()).into());
    }
    Ok(value)
}
fn active(env: &Environment) -> PathBuf {
    env.root().join(format!(
        "cache/biomcp/cache-{}-{}.sqlite",
        biomcp_fetch::CACHE_NAMESPACE_VERSION,
        biomcp_query::store::CACHE_NAMESPACE_VERSION
    ))
}
fn seed(cx: &Connection) -> Check {
    cx.execute_batch(
        "INSERT INTO pool VALUES ('expired',x'01',0,0,1,'[]');
        INSERT INTO pool_item(pool,rank,identity,item) VALUES ('expired',0,'old','{}');
        INSERT INTO pool_current VALUES (x'01','expired');
        INSERT INTO trial_snapshot VALUES ('old-trial',x'02',0,'{}','[]');
        INSERT INTO citation_evidence VALUES ('old-citation',x'03',0,'{}','[]');
        INSERT INTO trial_snapshot VALUES ('live-trial',x'02',9223372036854775807,'{}','[]');
        INSERT INTO citation_evidence VALUES ('live-citation',x'03',9223372036854775807,'{}','[]');
        INSERT INTO reply(key,source,url,status,body,fetched_at,fresh_until) VALUES \
        (x'01','pubmed','https://example.test/',200,x'7b7d',0,0);
        INSERT INTO reply(key,source,url,status,body,fetched_at,fresh_until) VALUES \
        (x'02','pubmed','https://example.test/',200,x'7b7d',0,9223372036854775807);
        CREATE TABLE foreign_state(value TEXT); INSERT INTO foreign_state VALUES ('keep');
        INSERT INTO meta VALUES ('foreign-owner',77);",
    )?;
    Ok(())
}
fn count(cx: &Connection, table: &str) -> Check<i64> {
    Ok(cx.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))?)
}
fn clean() -> Check {
    let env = Environment::new()?;
    run(&env, &["store", "clean", "--offline"])?;
    let path = active(&env);
    let cx = Connection::open(&path)?;
    seed(&cx)?;
    let directory = path.parent().ok_or("missing cache directory")?;
    let old = directory.join("cache-1-2.sqlite");
    let foreign = directory.join("cache-1-3.sqlite");
    let future = directory.join("cache-1-999.sqlite");
    for file in [&old, &foreign, &future] {
        let old_cx = Connection::open(file)?;
        old_cx.execute_batch("CREATE TABLE meta(owner TEXT,version INTEGER); INSERT INTO meta VALUES ('biomcp-query',4); CREATE TABLE pool(id TEXT);")?;
    }
    Connection::open(&foreign)?.execute_batch("CREATE TABLE foreign_state(value TEXT)")?;
    let mut sidecars = Vec::new();
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", old.display()));
        std::fs::write(&sidecar, [])?;
        sidecars.push(sidecar);
    }
    let retained = coexisting_generation(directory)?;
    let unowned = unowned_legacy(directory)?;
    let result = run(&env, &["store", "clean", "--offline"])?;
    if retained.iter().any(|path| !path.exists()) {
        return Err("cleanup removed a coexisting generation or its sidecars".into());
    }
    if result.get("result")
        != Some(&json!({"replies":1,"pools":1,"items":1,"snapshots":2,"files":1}))
    {
        return Err(format!("wrong cleanup counts: {result}").into());
    }
    if old.exists() || !foreign.exists() || !future.exists() || !path.exists() {
        return Err("cleanup removed the wrong namespace".into());
    }
    for table in [
        "reply",
        "trial_snapshot",
        "citation_evidence",
        "foreign_state",
    ] {
        if count(&cx, table)? != 1 {
            return Err(format!("cleanup changed live {table}").into());
        }
    }
    for (file, table) in unowned {
        if !file.exists() {
            return Err("cleanup deleted an undeclared schema".into());
        }
        let sentinel: String =
            Connection::open(file)?
                .query_row(&format!("SELECT value FROM {table}"), [], |r| r.get(0))?;
        if sentinel != "keep" {
            return Err("cleanup changed undeclared schema data".into());
        }
    }
    if sidecars.iter().any(|path| path.exists()) {
        return Err("cleanup left SQLite sidecars".into());
    }
    fresh_content(&cx)?;
    if count(&cx, "pool_item")? != 0 || count(&cx, "pool_current")? != 0 {
        return Err("orphan pool rows survived".into());
    }
    Ok(())
}
fn coexisting_generation(directory: &std::path::Path) -> Check<Vec<PathBuf>> {
    let path = directory.join("cache-1-4.sqlite");
    let cx = Connection::open(&path)?;
    cx.execute_batch(
        "CREATE TABLE meta(owner TEXT,version INTEGER); INSERT INTO meta VALUES ('biomcp-query',4); CREATE \
        TABLE pool(id TEXT); INSERT INTO pool VALUES ('keep');",
    )?;
    drop(cx);
    let mut paths = vec![path.clone()];
    for suffix in ["-wal", "-shm"] {
        let sidecar = PathBuf::from(format!("{}{suffix}", path.display()));
        std::fs::write(&sidecar, [])?;
        paths.push(sidecar);
    }
    Ok(paths)
}
fn unowned_legacy(directory: &std::path::Path) -> Check<Vec<(PathBuf, &'static str)>> {
    let mut files = Vec::new();
    for (name, owner, table) in [
        ("cache-0-4.sqlite", "", "pool"),
        ("cache-0-3.sqlite", "biomcp-query", "reply"),
    ] {
        let path = directory.join(name);
        let cx = Connection::open(&path)?;
        cx.execute_batch(&format!(
            "CREATE TABLE meta(owner TEXT,version INTEGER); CREATE TABLE {table}(value TEXT); INSERT INTO {table} VALUES ('keep');"
        ))?;
        if !owner.is_empty() {
            cx.execute("INSERT INTO meta VALUES (?1,4)", [owner])?;
        }
        files.push((path, table));
    }
    Ok(files)
}

fn fresh_content(cx: &Connection) -> Check {
    let fresh_reply: bool = cx.query_row(
        "SELECT EXISTS(SELECT 1 FROM reply WHERE key=x'02' AND fresh_until=9223372036854775807)",
        [],
        |r| r.get(0),
    )?;
    let fresh_trial: String = cx.query_row("SELECT id FROM trial_snapshot", [], |r| r.get(0))?;
    let fresh_citation: String =
        cx.query_row("SELECT id FROM citation_evidence", [], |r| r.get(0))?;
    let foreign_value: String =
        cx.query_row("SELECT value FROM foreign_state", [], |r| r.get(0))?;
    let foreign_version: u32 = cx.query_row(
        "SELECT version FROM meta WHERE owner='foreign-owner'",
        [],
        |r| r.get(0),
    )?;
    if !fresh_reply
        || fresh_trial != "live-trial"
        || fresh_citation != "live-citation"
        || foreign_value != "keep"
        || foreign_version != 77
    {
        return Err("cleanup changed fresh or foreign content".into());
    }
    Ok(())
}

fn publish() -> Check {
    let env = Environment::new()?;
    let replay = super::launch::fixtures().join("replies/article/search/synthetic-small");
    let state = env.root().join("publish-cache");
    let replay = replay.to_str().ok_or("non-text fixture path")?;
    let state_text = state.to_str().ok_or("non-text cache path")?;
    let args = [
        "search",
        "article",
        "AKT1 E17K",
        "--limit",
        "2",
        "--replay",
        replay,
        "--case-cache",
        state_text,
    ];
    let first = run(&env, &args)?;
    let id = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("missing pool")?;
    let cx = Connection::open(state.join("replay.sqlite"))?;
    seed(&cx)?;
    // More than one publication batch: a single publish must not delete everything.
    cx.execute_batch(
        "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<150) INSERT INTO pool SELECT \
        'expired-'||x,x'04',0,0,0,'[]' FROM n;",
    )?;
    let reader = Connection::open(state.join("replay.sqlite"))?;
    reader.execute_batch("BEGIN")?;
    let live: i64 =
        reader.query_row("SELECT count(*) FROM pool_item WHERE pool=?1", [id], |r| {
            r.get(0)
        })?;
    let mut fresh = args.to_vec();
    fresh.push("--fresh");
    run(&env, &fresh)?;
    let remaining: i64 = cx.query_row("SELECT count(*) FROM pool WHERE expires=0", [], |r| {
        r.get(0)
    })?;
    if remaining != 51
        || count(&cx, "reply")? < 2
        || count(&cx, "trial_snapshot")? != 1
        || count(&cx, "citation_evidence")? != 1
    {
        return Err("publish did not bound cleanup or retained replies incorrectly".into());
    }
    let after: i64 =
        reader.query_row("SELECT count(*) FROM pool_item WHERE pool=?1", [id], |r| {
            r.get(0)
        })?;
    if live == 0 || live != after {
        return Err("publish changed a concurrent live reader".into());
    }
    reader.execute_batch("COMMIT")?;
    let mut page = args.to_vec();
    page.extend(["--pool", id]);
    let retained = run(&env, &page)?;
    if retained.pointer("/result/items") != first.pointer("/result/items") {
        return Err("publish changed live pool page".into());
    }
    let orphans: i64 = cx.query_row(
        "SELECT count(*) FROM pool_item i LEFT JOIN pool p ON p.id=i.pool WHERE p.id IS NULL",
        params![],
        |r| r.get(0),
    )?;
    if orphans != 0 {
        return Err("publish left orphan items".into());
    }
    Ok(())
}
