//! Feature article-search: partial snapshots page independently and never answer a new query.
use super::{
    configuration::{cached, invoke},
    launch::Environment,
};
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::path::Path;

pub(super) fn trials() -> Vec<Trial> {
    let mut trials = vec![
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
        super::transcript::trial(
            "article-search/whole-pool-publication/withheld-names",
            || withheld_names().map_err(Into::into),
        ),
    ];
    for entity in ["article", "trial"] {
        for mode in ["capped", "refused", "fallback"] {
            trials.push(super::transcript::trial(
                format!("drug-name-search/incomplete/{entity}/{mode}"),
                move || incomplete_names(entity, mode).map_err(Into::into),
            ));
        }
    }
    trials
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
    let first = invoke(&env, enabled.clone(), 0)?;
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
    // A failed read under another source list says nothing about this list's pool.
    let repeat = invoke(&env, enabled, 0)?;
    if repeat.pointer("/page/pool/id") != Some(&json!(id)) {
        return Err(format!("another list unpublished the pool: {repeat}"));
    }
    Ok(())
}

/// A search that may be missing names pages its own pool, and the next search reaches the sources.
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
    let command = ["search", "article", "EX-66", "--limit", "1"];
    let first = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    let id = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or(format!("failed lookup kept no pool: {first}"))?;
    let total = first
        .pointer("/page/total")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    if first.get("status").and_then(Value::as_str) != Some("partial")
        || first.pointer("/page/returned") != Some(&json!(1))
        || total < 2
        || first.pointer("/result/items/0/header/title").is_none()
    {
        return Err(format!(
            "failed lookup did not page a partial pool: {first}"
        ));
    }
    let step = first
        .get("next")
        .and_then(Value::as_array)
        .and_then(|steps| {
            steps
                .iter()
                .find(|step| step.get("kind") == Some(&json!("page")))
        })
        .ok_or(format!("no next page step: {first}"))?;
    if !step.to_string().contains(id) {
        return Err(format!("next page left the pool: {first}"));
    }
    let mut named = cached(&command, &cache, Some(&replay));
    named.extend(["--pool".into(), id.into(), "--offset".into(), "1".into()]);
    let second = invoke(&env, named, 0)?;
    if second.get("status").and_then(Value::as_str) != Some("partial")
        || second.pointer("/page/pool/id") != first.pointer("/page/pool/id")
        || second.pointer("/page/offset") != Some(&json!(1))
        || second.pointer("/page/returned") != Some(&json!(1))
        || second.pointer("/result/items/0/pmid") == first.pointer("/result/items/0/pmid")
        || second.pointer("/result/items/0/header/title").is_none()
    {
        return Err(format!("failed lookup's next page was lost: {second}"));
    }
    copy_capture("search/drug-names", &replay, |_, _| Ok(()))?;
    let third = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    if third.get("status").and_then(Value::as_str) != Some("complete")
        || third.pointer("/page/total") != Some(&json!(3))
        || third.pointer("/page/pool/id") == first.pointer("/page/pool/id")
    {
        return Err(format!(
            "new search did not reach recovered lookup: {third}"
        ));
    }
    let fourth = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    if fourth.pointer("/page/pool/id") != third.pointer("/page/pool/id") {
        return Err(format!("whole drug pool did not become current: {fourth}"));
    }
    Ok(())
}

/// A stored pool named under a list that disables its name source repeats none of those names.
fn withheld_names() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let cache = env.root().join("article-case-cache");
    let replay = env.root().join("replay");
    copy_capture("search/drug-names", &replay, |_, _| Ok(()))?;
    let command = ["search", "article", "EX-66", "--limit", "1"];
    let first = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    let id = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("expanded pool absent")?;
    let mut named = cached(&command, &cache, Some(&replay));
    named.extend([
        "--disable-sources".into(),
        "chembl".into(),
        "--pool".into(),
        id.into(),
    ]);
    let reply = invoke(&env, named, 4)?;
    if reply.get("status").and_then(Value::as_str) != Some("failed")
        || reply.pointer("/sources/0/term").is_some()
        || reply.pointer("/sources/0/notices").is_some()
        || reply.pointer("/query/text") != Some(&json!("EX-66"))
    {
        return Err(format!("disabled name source reached the reply: {reply}"));
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

/// Unseen or refused candidates cannot establish one structure, but a complete fallback can.
fn incomplete_names(entity: &str, mode: &str) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let cache = env.root().join("name-cache");
    let replay = env.root().join("replay");
    copy_capture("search/drug-names", &replay, |source, record| {
        if source == "mychem" {
            for exchange in record["exchanges"].as_array_mut().ok_or("exchanges")? {
                let body = &mut exchange["body"]["value"];
                if let Some(hits) = body.get_mut("hits").and_then(Value::as_array_mut) {
                    if mode == "refused" {
                        hits.push(Value::Null);
                        body["total"] = json!(hits.len());
                    } else {
                        body["total"] = json!(999);
                    }
                }
            }
        } else if source == "openfda-label" || source == "chembl" && mode != "fallback" {
            fail_exchanges(record)?;
        }
        Ok(())
    })?;
    let command = ["search", entity, "Canonical therapy", "--limit", "1"];
    let first = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
    if mode == "fallback" {
        if first.pointer("/page/total") != Some(&json!(3))
            || !first.to_string().contains("Example brand")
        {
            return Err(format!("complete fallback did not expand names: {first}"));
        }
    } else {
        let notice = first["sources"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|source| source["notices"].as_array().into_iter().flatten())
            .find(|notice| {
                notice["message"]
                    .as_str()
                    .is_some_and(|m| m.contains("did not finish"))
            });
        if notice.and_then(|n| n.get("names")) != Some(&json!(["Canonical therapy"])) {
            return Err(format!(
                "incomplete lookup expanded or hid uncertainty: {first}"
            ));
        }
        if entity == "article" {
            let next = invoke(&env, cached(&command, &cache, Some(&replay)), 0)?;
            if next.pointer("/page/pool/id") == first.pointer("/page/pool/id") {
                return Err("incomplete article pool became current".into());
            }
        }
    }
    Ok(())
}
