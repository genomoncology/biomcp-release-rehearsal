//! Existing native reference control and synthetic retained citation boundary.
use super::native_fields::check as native;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), Box<dyn std::error::Error>>;
pub(super) fn trial() -> Trial {
    transcript_trial(crate::runner::pool_contract::ARTICLE_CITATIONS.0, || {
        run().map_err(|e| e.to_string().into())
    })
}
fn run() -> Check {
    native(
        "article/edges/live",
        &[
            "article",
            "references",
            "33123537",
            "--source",
            "semantic-scholar",
        ],
        "semantic-scholar",
        "/exchanges/0/body/value/data/0/isInfluential",
        "influential",
        json!({"/result/items/0/pmid":"99100142"}),
        "/result/items/0/influential",
    )?;
    doi_fallback_context()?;
    retained_citations()
}

/// A PubMed search built from graph DOIs reports those DOIs as its inputs.
fn doi_fallback_context() -> Check {
    let env = crate::runner::launch::Environment::new()?;
    let replay = crate::runner::launch::fixtures().join("replies/article/edges/live");
    let report = env.root().join("doi-fallback.jsonl");
    let mut args = [
        "article",
        "references",
        "34096690",
        "--source",
        "semantic-scholar",
    ]
    .map(str::to_owned)
    .to_vec();
    args.extend(env.globals(Some(&replay), false));
    args.extend(["--observe".into(), report.to_string_lossy().into()]);
    crate::runner::configuration::invoke(&env, args, 0)?;
    let rows = std::fs::read_to_string(report)?
        .lines()
        .map(serde_json::from_str::<biomcp_fetch::Observation>)
        .collect::<Result<Vec<_>, _>>()?;
    let discovered = rows
        .iter()
        .filter_map(|row| row.provenance.discoveries.as_ref())
        .flatten()
        .find(|ids| ids.namespace == "doi")
        .ok_or("graph reply lost its DOI discovery")?;
    let searches = rows
        .iter()
        .filter(|row| row.url.contains("esearch.fcgi"))
        .collect::<Vec<_>>();
    let complete = |row: &&biomcp_fetch::Observation| {
        row.provenance.input_context.is_some()
            && row.provenance.inputs.as_ref().is_some_and(|ids| {
                ids.namespace == "doi"
                    && !ids.ids.is_empty()
                    && ids.ids.iter().all(|id| discovered.ids.contains(id))
            })
    };
    if searches.is_empty() || !searches.iter().all(complete) {
        return Err("DOI fallback search lost its discovery inputs or context".into());
    }
    Ok(())
}

fn retained_citations() -> Check {
    let env = crate::runner::launch::Environment::new()?;
    let replay = crate::runner::launch::fixtures().join("replies/article/edges/synthetic-window");
    let globals = env.globals(Some(&replay), false);
    let mut args = [
        "article",
        "citations",
        "23000897",
        "--source",
        "semantic-scholar",
        "--limit",
        "1",
    ]
    .map(str::to_owned)
    .to_vec();
    let mut first_args = args.clone();
    first_args.extend(globals.clone());
    let first = crate::runner::configuration::invoke(&env, first_args, 0)?;
    crate::runner::schema::Registry::embedded()?.validate(&first, &args, true)?;
    let pool = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("citation pool")?;
    args.extend(["--pool".into(), pool.into(), "--offset".into(), "1".into()]);
    let mut retained_args = args.clone();
    retained_args.extend(globals);
    let kept = crate::runner::configuration::invoke(&env, retained_args, 0)?;
    crate::runner::schema::Registry::embedded()?.validate(&kept, &args, true)?;
    if kept.pointer("/result/items/0/header/title") != Some(&json!("Synthetic source page2."))
        || kept.pointer("/page/offset") != Some(&json!(1))
    {
        return Err("retained citation page changed its source order".into());
    }
    Ok(())
}
