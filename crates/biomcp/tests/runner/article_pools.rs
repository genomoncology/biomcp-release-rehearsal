//! Feature article-search: partial snapshots page independently and never answer a new query.
use super::{
    configuration::{cached, invoke},
    launch::Environment,
};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::path::Path;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        super::transcript::trial("article-search/whole-pool-publication/search", || {
            partial(
                "article/fold/failed-pubmed",
                &[
                    "search",
                    "article",
                    "AKT1",
                    "--author",
                    "Carpten",
                    "--source",
                    "pubmed,europepmc",
                ],
                "",
            )
            .map_err(Into::into)
        }),
        super::transcript::trial(
            "article-search/whole-pool-publication/recommendations",
            || {
                partial(
                    "article/recommendations/single",
                    &["article", "recommendations", "34096690", "--limit", "5"],
                    "doi",
                )
                .map_err(Into::into)
            },
        ),
        super::transcript::trial("article-search/whole-pool-publication/edges", || {
            partial(
                "article/edges/live",
                &[
                    "article",
                    "references",
                    "34096690",
                    "--source",
                    "semantic-scholar",
                    "--offset",
                    "16",
                    "--limit",
                    "1",
                ],
                "doi",
            )
            .map_err(Into::into)
        }),
        super::transcript::trial("article-search/whole-pool-publication/disabled", || {
            disabled().map_err(Into::into)
        }),
        super::transcript::trial("article-search/whole-pool-publication/drug-names", || {
            failed_names().map_err(Into::into)
        }),
    ]
}

fn partial(recording: &str, command: &[&str], damage: &str) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = env.root().join("replay");
    copy_capture(recording, &replay, |source, record| {
        if damage.is_empty() {
            Ok(())
        } else {
            fail_doi_lookup(source, record)
        }
    })?;
    let cache = env.root().join("article-case-cache");
    let first = invoke(&env, cached(command, &cache, Some(&replay)), 0)?;
    if first.get("status").and_then(Value::as_str) != Some("partial") {
        return Err(format!("partial fixture did not remain useful: {first}"));
    }
    let id = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("partial pool absent")?;
    let mut explicit = cached(command, &cache, Some(&replay));
    explicit.extend(["--pool".into(), id.into()]);
    let page = invoke(&env, explicit, 0)?;
    let original = first.get("result").ok_or("initial result absent")?;
    let retained = page.get("result").ok_or("explicit page result absent")?;
    if page.pointer("/page/pool/id") != Some(&json!(id)) || retained != original {
        return Err("explicit partial pool lost stable rows".into());
    }
    // Recovery must be observed by a new search, while the explicit partial pool stays fixed.
    copy_capture(recording, &replay, |_, record| {
        if damage.is_empty() {
            accept_exchanges(record)
        } else {
            Ok(())
        }
    })?;
    let second = invoke(&env, cached(command, &cache, Some(&replay)), 0)?;
    if second.get("status").and_then(Value::as_str) != Some("complete") {
        return Err(format!(
            "new query did not observe source recovery: {second}"
        ));
    }
    if second.pointer("/page/pool/id") == Some(&json!(id)) {
        return Err(format!("new query reused partial pool for {recording}"));
    }
    Ok(())
}

fn disabled() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let cache = env.root().join("article-case-cache");
    let replay = env.root().join("replay");
    copy_capture("article/fold/failed-pubmed", &replay, |_, record| {
        accept_exchanges(record)
    })?;
    let command = [
        "search",
        "article",
        "AKT1",
        "--author",
        "Carpten",
        "--source",
        "pubmed,europepmc",
    ];
    let mut enabled = cached(&command, &cache, Some(&replay));
    enabled.extend(["--disable-sources".into(), "pubmed".into()]);
    let first = invoke(&env, enabled, 0)?;
    let id = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("complete pool absent")?;
    let mut changed = cached(&command, &cache, Some(&replay));
    changed.extend(["--disable-sources".into(), "europepmc".into()]);
    let second = invoke(&env, changed.clone(), 0)?;
    if second.get("status").and_then(Value::as_str) != Some("complete")
        || second.pointer("/sources/0/source").and_then(Value::as_str) != Some("pubmed")
        || second.pointer("/sources/0/state").and_then(Value::as_str) != Some("ok")
    {
        return Err(format!(
            "new source policy did not reach enabled source: {second}"
        ));
    }
    changed.extend(["--pool".into(), id.into()]);
    let explicit = invoke(&env, changed, 4)?;
    if explicit.get("status").and_then(Value::as_str) != Some("failed")
        || !explicit
            .get("sources")
            .and_then(Value::as_array)
            .is_some_and(|sources| {
                sources.iter().any(|source| {
                    source.get("source").and_then(Value::as_str) == Some("europepmc")
                        && source.get("reason").and_then(Value::as_str) == Some("disabled")
                })
            })
    {
        return Err("explicit pool did not retain failure branch".into());
    }
    Ok(())
}

/// A search that may be missing names keeps no pool, so the next search reaches the sources.
fn failed_names() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let cache = env.root().join("article-case-cache");
    let replay = env.root().join("replay");
    copy_capture("search/drug-names", &replay, |source, record| {
        if matches!(source, "mychem" | "chembl") {
            fail_exchanges(record)
        } else {
            Ok(())
        }
    })?;
    let command = ["search", "article", "Canonical therapy"];
    let first = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    if first.get("status").and_then(Value::as_str) != Some("partial")
        || first
            .pointer("/page/pool")
            .is_some_and(|pool| !pool.is_null())
    {
        return Err(format!("failed lookup stored a pool: {first}"));
    }
    copy_capture("search/drug-names", &replay, |_, _| Ok(()))?;
    let second = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    if second.get("status").and_then(Value::as_str) != Some("complete")
        || second.pointer("/page/total") != Some(&json!(3))
    {
        return Err(format!(
            "new search did not reach recovered lookup: {second}"
        ));
    }
    let third = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    if third.pointer("/page/pool/id") != second.pointer("/page/pool/id") {
        return Err(format!("whole drug pool did not become current: {third}"));
    }
    Ok(())
}

// Fixture mutations are fallible even though the shared copy callback returns no value.
fn copy_capture(
    recording: &str,
    replay: &Path,
    mut change: impl FnMut(&str, &mut Value) -> Result<(), String>,
) -> Result<(), String> {
    let mut setup = Ok(());
    super::response_cases::copy_capture(recording, replay, |source, record| {
        if setup.is_ok() {
            setup = change(source, record).map_err(|error| format!("{source} fixture: {error}"));
        }
    })
    .map_err(|error| error.to_string())?;
    setup
}

fn fail_doi_lookup(source: &str, record: &mut Value) -> Result<(), String> {
    if source != "pubmed" {
        return Ok(());
    }
    let url = record
        .get("url")
        .and_then(Value::as_str)
        .ok_or("missing recording URL")?;
    if !url.contains("%5Bdoi%5D") {
        return Ok(());
    }
    let mut failure = record
        .get("exchanges")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .cloned()
        .ok_or("missing first DOI lookup exchange")?;
    *failure
        .get_mut("status")
        .ok_or("missing DOI lookup status")? = json!(500);
    *record
        .get_mut("exchanges")
        .ok_or("missing DOI lookup exchanges")? =
        json!([failure.clone(), failure.clone(), failure]);
    Ok(())
}

fn fail_exchanges(record: &mut Value) -> Result<(), String> {
    let exchanges = record
        .get_mut("exchanges")
        .and_then(Value::as_array_mut)
        .ok_or("missing recording exchanges")?;
    let mut failure = exchanges
        .first()
        .cloned()
        .ok_or("recording has no exchanges")?;
    *failure.get_mut("status").ok_or("missing exchange status")? = json!(500);
    *exchanges = vec![failure.clone(), failure.clone(), failure];
    Ok(())
}

fn accept_exchanges(record: &mut Value) -> Result<(), String> {
    let exchanges = record
        .get_mut("exchanges")
        .and_then(Value::as_array_mut)
        .ok_or("missing recording exchanges")?;
    if exchanges.is_empty() {
        return Err("recording has no exchanges".into());
    }
    for exchange in exchanges {
        *exchange
            .get_mut("status")
            .ok_or("missing exchange status")? = json!(200);
    }
    Ok(())
}
