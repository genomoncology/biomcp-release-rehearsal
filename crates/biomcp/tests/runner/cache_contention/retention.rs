//! Feature cache-resilience: expired checked replies survive unrelated writes for a bounded grace.
use super::super::{
    configuration::{cached, invoke},
    launch::{Environment, fixtures},
};
use serde_json::{Value, json};

pub(super) fn stale() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let case = env.root().join("case");
    let protein = env.root().join("protein-recordings");
    let gene = env.root().join("gene-recordings");
    super::super::response_cases::cache_capture(
        "protein/synthetic",
        &protein,
        super::super::response_cases::CACHE_ACQUIRED,
    )?;
    super::super::response_cases::cache_capture(
        "gene/main",
        &gene,
        super::super::response_cases::CACHE_ACQUIRED,
    )?;
    let args = cached(&["get", "protein", "P00533"], &case, Some(&protein));
    let first = invoke(&env, args.clone(), 0)?;
    let db = rusqlite::Connection::open(case.join("replay.sqlite")).map_err(|e| e.to_string())?;
    let expired = db
        .execute(
            "UPDATE reply SET fresh_until=unixepoch()-1 WHERE source='uniprot'",
            [],
        )
        .map_err(|e| e.to_string())?;
    if expired != 1 {
        return Err(format!(
            "expected one checked protein cache row, expired {expired}"
        ));
    }
    let unrelated = cached(&["get", "gene", "BRAF"], &case, Some(&gene));
    invoke(&env, unrelated.clone(), 0)?;
    let offline = cached(&["get", "protein", "P00533"], &case, None);
    let stale = invoke(&env, offline.clone(), 0)?;
    if payload(&stale)? != payload(&first)?
        || stale.pointer("/result/items/0/status") != Some(&json!("partial"))
        || stale.pointer("/sources/0/state") != Some(&json!("stale"))
    {
        return Err(format!(
            "unrelated write deleted the stale fallback: {stale}"
        ));
    }
    db.execute(
        "UPDATE reply SET fresh_until=unixepoch()-?1-1 WHERE source='uniprot'",
        [biomcp_fetch::STALE_GRACE_SECONDS],
    )
    .map_err(|e| e.to_string())?;
    let mut fresh = unrelated;
    fresh.push("--fresh".into());
    invoke(&env, fresh, 0)?;
    let remaining: i64 = db
        .query_row(
            "SELECT count(*) FROM reply WHERE source='uniprot'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let expired = invoke(&env, offline, 4)?;
    if remaining != 0 || expired.pointer("/sources/0/reason") != Some(&json!("offline")) {
        return Err(format!(
            "grace-expired reply survived automatic pruning: {expired}"
        ));
    }
    Ok(())
}

pub(super) fn open_reader() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let case = env.root().join("case");
    let records = fixtures().join("replies/article/search/synthetic-small");
    let args = cached(
        &["search", "article", "AKT1 E17K", "--limit", "2"],
        &case,
        Some(&records),
    );
    let first = invoke(&env, args.clone(), 0)?;
    let db = rusqlite::Connection::open(case.join("replay.sqlite")).map_err(|e| e.to_string())?;
    db.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| e.to_string())?;
    let same = invoke(&env, args, 0)?;
    db.execute_batch("ROLLBACK").map_err(|e| e.to_string())?;
    if same.get("result") != first.get("result")
        || same.pointer("/page/pool/id") != first.pointer("/page/pool/id")
    {
        return Err(format!(
            "matching schema open did not stay read-only: {same}"
        ));
    }
    Ok(())
}

// Staleness changes item status while retaining every fetched record field.
fn payload(reply: &Value) -> Result<Value, String> {
    let mut body = reply
        .get("result")
        .cloned()
        .ok_or("fetched result missing")?;
    for row in body
        .get_mut("items")
        .and_then(Value::as_array_mut)
        .ok_or("fetched items missing")?
    {
        row.as_object_mut()
            .ok_or("record object missing")?
            .remove("status");
    }
    Ok(body)
}
