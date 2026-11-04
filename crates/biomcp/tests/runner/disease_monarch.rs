//! Feature disease-monarch: native association facts stay source-owned.
use super::{launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, path::Path};

tool_only! { mod native; }
tool_only! { pub(super) use native::run as native; }
pub(super) fn trials() -> Vec<Trial> {
    // The peer checks literal direct-association requests and exact request counts independently of mapped output fields.
    vec![transcript_trial(
        crate::runner::required_contract::MONARCH_PHENOTYPES.0,
        || phenotype().map_err(Into::into),
    )]
}
fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("monarch-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn invoke(env: &Environment, words: &[&str], exit: i32) -> Result<Value, String> {
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &words.iter().map(|s| (*s).into()).collect::<Vec<String>>(),
        )
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) {
        return Err(format!(
            "unexpected exit {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout)
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}
fn association() -> Value {
    json!(
        {"id":"synthetic-row","subject":"HGNC:9967","subject_label":"Synthetic gene","object":"MONDO:0007958",
        "object_label":"Synthetic child","predicate":"biolink:associated_with","original_subject":"NCBIGene:5979",
        "original_object":"Orphanet:99361","primary_knowledge_source":"infores:orphanet",
        "aggregator_knowledge_source":["infores:monarchinitiative","infores:hpo-annotations"],"provided_by":"synthetic_collection",
        "has_evidence":["ECO:0000501"],"qualifiers":null,"negated":false,"subject_closure":null,"object_closure":["MONDO:0019003"]}
    )
}

fn record(env: &Environment, words: &[&str], directory: &Path, exit: i32) -> Result<Value, String> {
    let mut words = words.to_vec();
    let path = directory.to_str().ok_or("non-UTF8 path")?;
    words.extend(["--record", path]);
    let value = invoke(env, &words, exit)?;
    measure(&words, &value)?;
    Ok(value)
}
fn measure(words: &[&str], value: &Value) -> Result<(), String> {
    use std::io::Write;
    writeln!(
        std::io::stderr(),
        "Monarch {:?}: full={} comparable={}",
        words,
        serde_json::to_vec(value).map_err(|e| e.to_string())?.len(),
        super::source_terms::size(value, "")?
    )
    .map_err(|e| e.to_string())
}
fn replay(directory: &Path, mut steps: Value) -> Result<Value, String> {
    for step in steps.as_array_mut().ok_or("missing steps")? {
        let step = step.as_object_mut().ok_or("missing step")?;
        let ceiling = step
            .get("max_bytes")
            .and_then(Value::as_u64)
            .ok_or("bad ceiling")?;
        step.insert("max_tokens".into(), json!(ceiling.div_ceil(4)));
    }
    let file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    fs::write(
        file.path(),
        serde_json::to_vec(&json!(
            {"feature":"disease-monarch","story":"Selected association rows preserve native context","replay":directory,"steps":steps}
        ))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let value = super::run_case(file.path(), super::schema::Registry::embedded()?.as_ref())?;
    measure(&["replay"], &value)?;
    Ok(value)
}

fn phenotype() -> Result<(), String> {
    let stub = Stub::for_source_status(
        "api-v3.monarchinitiative.org",
        "application/json",
        |_| {
            let mut row = association();
            let map = row.as_object_mut().ok_or("missing synthetic row")?;
            for (key, value) in [
                ("subject", json!("MONDO:0019003")),
                ("object", json!("HP:0000001")),
                ("frequency_qualifier", json!("HP:0040283")),
                ("qualifiers", json!(["HP:0000002"])),
                ("onset_qualifier", json!("HP:0000003")),
                ("sex_qualifier", json!("PATO:0000384")),
                ("stage_qualifier", json!("HP:0000004")),
                ("species_context_qualifier", json!("NCBITaxon:9606")),
                ("disease_context_qualifier", json!("MONDO:0019003")),
                ("frequency_qualifier_label", json!("Synthetic frequency")),
                ("has_count", json!(24)),
                ("has_total", json!(35)),
                ("has_percentage", json!(68.57143)),
                ("has_quotient", json!(0.6857143)),
                (
                    "has_evidence_links",
                    json!([{"id":"ECO:0000501","label":"Synthetic evidence","url":"http://purl.obolibrary.org/obo/ECO_0000501"}]),
                ),
                ("publications", json!(["PMID:1"])),
                (
                    "publications_links",
                    json!([{"id":"PMID:1","label":"Synthetic publication"}]),
                ),
            ] {
                map.insert(key.into(), value);
            }
            Ok(json!({"limit":5,"offset":0,"total":1,"items":[row]}).to_string())
        },
        |_| 200,
    )?;
    let env = setup(&stub)?;
    let words = [
        "get",
        "disease",
        "MONDO:0019003",
        "phenotypes",
        "--source",
        "monarch",
        "--section-limit",
        "5",
    ];
    let directory = env.root().join("phenotypes");
    let value = record(&env, &words, &directory, 0)?;
    if value.get("next").is_some() {
        return Err("phenotype received gene or page action".into());
    }
    replay(
        &directory,
        json!(
            [{"run":words,"mcp":words,"expect":{"/result/items/0/association/evidence/0/label":"Synthetic evidence",
            "/result/items/0/association/publications/0":{"equals":{"id":"PMID:1","label":"Synthetic publication"}},
            "/result/items/0/association/has_count":24,"/result/items/0/association/has_total":35,
            "/result/items/0/association/has_percentage":68.57143,"/result/items/0/association/has_quotient":0.6857143},"max_bytes":2588}]
        ),
    )?;
    if stub.finish()?.len() != 1 {
        return Err("unexpected phenotype fanout".into());
    }
    Ok(())
}
