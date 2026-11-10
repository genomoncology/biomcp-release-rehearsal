//! Stored payload failures remain distinct from SQLite access failures.
use super::{launch::Environment, response_cases::copy_capture, trial_get::command};
use serde_json::json;
type Check = Result<(), Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    vec![
        super::transcript::trial("drug-name-search/legacy-literal-pool", || {
            legacy_drug_pool(false).map_err(|e| e.to_string().into())
        }),
        super::transcript::trial("drug-name-search/legacy-expanded-pool", || {
            legacy_drug_pool(true).map_err(|e| e.to_string().into())
        }),
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

fn legacy_drug_pool(expanded: bool) -> Check {
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    copy_capture("search/drug-names", &replay, |_, _| {})?;
    let globals = env.globals(Some(&replay), false);
    let args = ["search", "article", "Canonical therapy", "--limit", "1"].map(str::to_owned);
    let mut literal = args.to_vec();
    literal.push("--exact-term".into());
    let first = command(&env, &literal, &globals, 0)?;
    let id = pool_id(&first)?;
    if first.pointer("/page/total") != Some(&json!(1)) {
        return Err(format!("literal control did not return one identity: {first}").into());
    }
    // Seed the pre-expansion representation in this isolated cache only: the literal pool takes
    // over the request and current slot of a real expanded search.
    let current = pool_id(&command(&env, &args, &globals, 0)?)?;
    let db = rusqlite::Connection::open(env.root().join("case-cache/replay.sqlite"))?;
    let current_key: Vec<u8> = db.query_row(
        "SELECT request_key FROM pool_current WHERE pool=?1",
        [&current],
        |r| r.get(0),
    )?;
    if db.execute(
        "UPDATE pool SET request_key=(SELECT request_key FROM pool WHERE id=?2) WHERE id=?1",
        rusqlite::params![id, current],
    )? != 1
        || db.execute(
            "UPDATE pool_current SET pool=?1 WHERE pool=?2",
            rusqlite::params![id, current],
        )? != 1
    {
        return Err("expected one isolated literal pool".into());
    }
    if expanded {
        seed_legacy_expansion(&db, &id, &current, &current_key)?;
    }
    drop(db);
    let expanded = command(&env, &args, &globals, 0)?;
    if expanded.pointer("/page/total") != Some(&json!(3)) {
        return Err(format!("legacy literal pool hid source names: {expanded}").into());
    }
    let rebuilt = pool_id(&expanded)?;
    if rebuilt == id {
        return Err("legacy literal pool was reused".into());
    }
    let again = command(&env, &args, &globals, 0)?;
    if pool_id(&again)? != rebuilt {
        return Err(format!("rebuilt drug pool did not become current: {again}").into());
    }
    let mut historical = args.to_vec();
    historical.extend(["--pool".into(), id]);
    let retained = command(&env, &historical, &globals, 0)?;
    if retained.pointer("/page/total") != Some(&json!(1)) {
        return Err(format!("explicit historical pool changed: {retained}").into());
    }
    Ok(())
}

fn pool_id(reply: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    reply
        .pointer("/page/pool/id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("no stored pool: {reply}").into())
}

/// The pre-completeness current-key format, including its old expansion notice.
fn seed_legacy_expansion(
    db: &rusqlite::Connection,
    id: &str,
    current: &str,
    current_key: &[u8],
) -> Check {
    use sha2::{Digest, Sha256};
    let query: biomcp_types::ArticleSearch =
        serde_json::from_value(json!({"text":"Canonical therapy", "limit":1}))?;
    let mut legacy = Sha256::new();
    legacy.update(b"biomcp.article_page/1\0");
    legacy.update(serde_json::to_vec(&query.identity())?);
    legacy.update([0]);
    legacy.update(b"[]");
    let key: [u8; 32] = legacy.finalize().into();
    db.execute(
        "UPDATE pool SET outcomes=(SELECT outcomes FROM pool WHERE id=?2) WHERE id=?1",
        rusqlite::params![id, current],
    )?;
    db.execute(
        "UPDATE pool_current SET request_key=?1 WHERE request_key=?2",
        rusqlite::params![key.as_slice(), current_key],
    )?;
    Ok(())
}
