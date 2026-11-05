//! Real command replays exercise source-owned URL and title fallback.
use super::{Environment, call};
use serde_json::{Value, json};
pub(super) fn check() -> Result<(), String> {
    pathway()?;
    empty_title()?;
    nci()
}
fn pathway() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let globals = env.globals(
        Some(&super::super::launch::fixtures().join("replies/pathway/native")),
        false,
    );
    let searched = call(
        &env,
        "search",
        json!({"query":"pathway 'Signaling by BRAF'"}),
        &globals,
    )?;
    let hit = searched
        .pointer("/results/0")
        .ok_or("pathway research header missing")?;
    if hit.get("url") != Some(&json!("https://reactome.org/content/detail/R-HSA-6802952"))
        || hit.get("title") != searched.pointer("/metadata/envelope/result/items/0/name")
    {
        return Err(format!("pathway research identity differs: {hit}"));
    }
    let fetched = call(
        &env,
        "fetch",
        json!({"id":"pathway R-HSA-109581"}),
        &globals,
    )?;
    if fetched.get("title") != Some(&json!("Apoptosis"))
        || fetched.get("url") != Some(&json!("https://reactome.org/content/detail/R-HSA-109581"))
        || fetched
            .get("text")
            .and_then(Value::as_str)
            .is_none_or(|s| !s.contains("Apoptosis"))
    {
        return Err("pathway fetch lost its title, canonical source page or card".into());
    }
    Ok(())
}
fn nci() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"keys":{"nci_api_key":"synthetic-nci-replay"}}))
        .map_err(|e| e.to_string())?;
    let replay = env.root().join("nci-research");
    for capture in ["trial/nci/search", "trial/nci/get"] {
        super::super::response_cases::copy_capture(capture, &replay, |_, _| {})
            .map_err(|e| e.to_string())?;
    }
    let globals = env.globals(Some(&replay), false);
    let searched = call(
        &env,
        "search",
        json!({"query":"trial --source nci --condition C3224 --limit 1"}),
        &globals,
    )?;
    let hit = searched
        .pointer("/results/0")
        .ok_or("NCI research header missing")?;
    let id = hit
        .get("id")
        .and_then(Value::as_str)
        .ok_or("NCI fetch ID missing")?;
    let fetched = call(&env, "fetch", json!({"id":id}), &globals)?;
    let url =
        "https://www.cancer.gov/research/participate/clinical-trials-search/v?id=NCI-2021-12653";
    if fetched.get("url") != Some(&json!(url))
        || hit.get("url") != fetched.get("url")
        || fetched.get("title") != hit.get("title")
    {
        return Err("NCI research URL lost source identity or native title".into());
    }
    Ok(())
}

fn empty_title() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = env.root().join("empty-title");
    super::super::response_cases::copy_capture("phenotype/synthetic", &replay, |_, record| {
        if record.get("source").and_then(Value::as_str) == Some("hpo")
            && let Some(label) = record.pointer_mut("/exchanges/0/body/value/name")
        {
            *label = json!("");
        }
    })
    .map_err(|e| e.to_string())?;
    let globals = env.globals(Some(&replay), false);
    let fetched = call(
        &env,
        "fetch",
        json!({"id":"phenotype HP:0000001"}),
        &globals,
    )?;
    if fetched.get("title") != Some(&json!("Human Phenotype Ontology HP:0000001"))
        || fetched.get("url") != Some(&json!("https://hpo.jax.org/browse/term/HP:0000001"))
    {
        return Err("empty native label lost its citable card or changed the source label".into());
    }
    Ok(())
}
