//! Stored payload failures remain distinct from SQLite access failures.
use super::{launch::Environment, response_cases::copy_capture, trial_get::command};
use serde_json::json;
type Check = Result<(), Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    vec![
        super::transcript::trial("cache-resilience/older-source-snapshot-rebuilds", || {
            older_version().map_err(|e| e.to_string().into())
        }),
        super::transcript::trial(
            "cache-resilience/corrupted-snapshot-reports-payload-error",
            || corrupted().map_err(|e| e.to_string().into()),
        ),
        super::transcript::trial("author-papers/name-search-selects-paper-author", || {
            name_lookup().map_err(|e| e.to_string().into())
        }),
    ]
}
fn corrupted() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("trial/get/NCT03734029", &replay, |_, _| {})?;
    let args = ["get", "trial", "NCT03734029", "locations"].map(str::to_owned);
    let first = command(&env, &args, &env.globals(Some(&replay), false), 0)?;
    let next = super::action(&first, "page")?;
    let fetch = biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
        cache: Some(env.root().join("case-cache/replay.sqlite")),
        cache_owners: vec![biomcp_query::store::OWNER],
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })?;
    let db = fetch.cache_access()?.connect()?;
    if db.execute("UPDATE trial_snapshot SET payload='invalid JSON'", [])? != 1 {
        return Err("expected one retained trial snapshot".into());
    }
    let broken = command(&env, &next, &env.globals(None, true), 5)?;
    if broken.pointer("/local_error") != Some(&json!("snapshot")) {
        return Err(format!("broken snapshot lost its own problem: {broken}").into());
    }
    Ok(())
}
fn name_lookup() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("author/synthetic", &replay, |_, record| {
        if let Some(id) = record.pointer_mut("/exchanges/0/body/value/data/0/authorId") {
            *id = json!("123");
        }
    })?;
    copy_capture("author/papers", &replay, |_, _| {})?;
    let search = ["search", "author", "Louis Williams"].map(str::to_owned);
    let found = command(&env, &search, &env.globals(Some(&replay), false), 0)?;
    let id = found
        .pointer("/result/items/0/id")
        .and_then(serde_json::Value::as_str)
        .ok_or("author ID")?;
    let args = ["author", "papers", id].map(str::to_owned);
    let papers = command(&env, &args, &env.globals(Some(&replay), false), 0)?;
    if papers.pointer("/result/author") != Some(&json!(id))
        || papers.pointer("/result/items/0/title") != Some(&json!("Synthetic whole paper title"))
        || papers.get("local_error").is_some()
    {
        return Err(format!("selected author lost its papers: {papers}").into());
    }
    Ok(())
}

fn older_version() -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("author/synthetic", &replay, |_, _| {})?;
    let args = ["search", "author", "Louis Williams"].map(str::to_owned);
    let first = command(&env, &args, &env.globals(Some(&replay), false), 0)?;
    let fetch = biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
        cache: Some(env.root().join("case-cache/replay.sqlite")),
        cache_owners: vec![biomcp_query::store::OWNER],
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })?;
    let db = fetch.cache_access()?.connect()?;
    // The preceding author store used this selector key and a different payload shape.
    let old = serde_json::to_vec(&json!([
        "biomcp.author_snapshot/3",
        "Louis Williams",
        "semantic-scholar",
        5
    ]))?;
    for table in ["trial_snapshot", "pool", "pool_current"] {
        db.execute(&format!("UPDATE {table} SET request_key=?1"), [&old])?;
    }
    db.execute(
        "UPDATE trial_snapshot SET payload='older incompatible bytes'",
        [],
    )?;
    let rebuilt = command(&env, &args, &env.globals(Some(&replay), false), 0)?;
    if first.pointer("/result/items") != rebuilt.pointer("/result/items")
        || rebuilt.get("local_error").is_some()
        || first.pointer("/page/pool/id") == rebuilt.pointer("/page/pool/id")
    {
        return Err(format!("older snapshot was not rebuilt: {rebuilt}").into());
    }
    let count: u32 = db.query_row("SELECT count(*) FROM trial_snapshot", [], |r| r.get(0))?;
    if count != 2 {
        return Err("expected a rebuilt snapshot alongside the older version".into());
    }
    Ok(())
}
