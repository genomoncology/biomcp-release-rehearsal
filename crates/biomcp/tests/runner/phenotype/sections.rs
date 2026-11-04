//! Feature phenotype-sections: source-owned text and direct link disclosure.
use super::{Stub, invoke, setup};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
pub(super) fn trials() -> Vec<Trial> {
    // The peer counts requests while the test changes runtime snapshots and follows source-owned class and hierarchy pages.
    vec![
        transcript_trial(crate::runner::required_contract::HPO_SNAPSHOT.0, || {
            retained().map_err(Into::into)
        }),
        transcript_trial(crate::runner::required_contract::MONARCH_XREFS.0, || {
            monarch().map_err(Into::into)
        }),
    ]
}

fn step(value: &Value, kind: &str, index: usize) -> Result<Vec<String>, String> {
    value
        .get("next")
        .and_then(Value::as_array)
        .ok_or("missing next")?
        .iter()
        .filter(|row| row.get("kind").and_then(Value::as_str) == Some(kind))
        .nth(index)
        .and_then(|row| row.get("args"))
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing {kind} step: {value}"))?
        .iter()
        .map(|word| word.as_str().map(String::from).ok_or("step word".into()))
        .collect()
}
fn monarch() -> Result<(), String> {
    let stub = Stub::for_source("api-v3.monarchinitiative.org", "application/json", |_| {
        Ok(json!(
            { "id":"HP:0000001","name":"Synthetic term","category":"biolink:PhenotypicFeature","synonym":["Unclassified"],
            "exact_synonym":["Exact"],"broad_synonym":["Broad"],"narrow_synonym":[],"related_synonym":["Related"],"xref":["UMLS:C0000001"],
            "node_hierarchy":{"super_classes":[{"id":"HP:0000002","name":"Parent"}],"sub_classes":[{"id":"HP:0000003","name":"Child"}]} }
        )
        .to_string())
    })?;
    let env = setup(&stub)?;
    for (section, expected) in [
        ("synonyms", json!([null, "exact", "broad", "related"])),
        ("hierarchy", json!(["super_classes", "sub_classes"])),
    ] {
        let result = invoke(
            &format!("sections/monarch/invoke1@section={section}"),
            &env,
            &[
                "get",
                "phenotype",
                "HP:0000001",
                section,
                "--source",
                "monarch",
                "--section-limit",
                "2",
            ],
            0,
        )?;
        if result.pointer("/query/section_limit") != Some(&json!(2)) {
            return Err("explicit limit changed".into());
        }
        classes(&result, section, &expected)?;
        if section == "synonyms" {
            let words = step(&result, "page", 0)?;
            let words = words.iter().map(String::as_str).collect::<Vec<_>>();
            let next = invoke(
                &format!("sections/monarch/invoke2@section={section}"),
                &env,
                &words,
                0,
            )?;
            if next.pointer("/result/items/0/relationship") != Some(&json!("broad"))
                || next.pointer("/result/items/1/relationship") != Some(&json!("related"))
            {
                return Err("native class walk changed".into());
            }
        }
    }
    let result = invoke(
        "sections/monarch/invoke3",
        &env,
        &[
            "get",
            "phenotype",
            "HP:0000001",
            "xrefs",
            "--source",
            "monarch",
        ],
        0,
    )?;
    if result.pointer("/result/items/0/id") != Some(&json!("UMLS:C0000001"))
        || result.pointer("/result/items/0/predicate").is_some()
    {
        return Err("xref invented predicate".into());
    }
    if stub.finish()?.len() != 1 {
        return Err("source sections bypassed checked entity cache".into());
    }
    Ok(())
}

fn retained() -> Result<(), String> {
    let mut count = 0;
    let stub = Stub::for_sources(
        &["ontology.jax.org"],
        |_| "application/json",
        move |_| {
            count += 1;
            Ok(json!({"id":"HP:0000001","name":"Term","synonyms":[if count==1 {"Original"} else {"Refreshed"}]}).to_string())
        },
    )?;
    let env = setup(&stub)?;
    let card = invoke(
        "sections/retained/invoke1",
        &env,
        &["get", "phenotype", "HP:0000001"],
        0,
    )?;
    let args = step(&card, "section", 0)?;
    let words = args.iter().map(String::as_str).collect::<Vec<_>>();
    invoke(
        "sections/retained/invoke2",
        &env,
        &["--fresh", "get", "phenotype", "HP:0000001"],
        0,
    )?;
    let original = invoke("sections/retained/invoke3", &env, &words, 0)?;
    if original.pointer("/result/items/0/value") != Some(&json!("Original")) {
        return Err("refresh replaced retained source reply".into());
    }
    let mut denied = words.clone();
    denied.extend(["--disable-sources", "hpo"]);
    let denied = invoke("sections/retained/invoke4", &env, &denied, 4)?;
    if denied.get("result").is_some()
        || denied.pointer("/sources/0/reason") != Some(&json!("disabled"))
    {
        return Err("disabled source restored retained content".into());
    }
    let mut foreign = args;
    *foreign.get_mut(2).ok_or("identity arg")? = "HP:0000002".into();
    let foreign = foreign.iter().map(String::as_str).collect::<Vec<_>>();
    invoke("sections/retained/invoke5", &env, &foreign, 3)?;
    if stub.finish()?.len() != 2 {
        return Err("retained page made a provider request".into());
    }
    Ok(())
}

fn classes(result: &Value, section: &str, expected: &Value) -> Result<(), String> {
    let field = if section == "synonyms" {
        "relationship"
    } else {
        "relation"
    };
    let items = result
        .pointer("/result/items")
        .and_then(Value::as_array)
        .ok_or("rows")?;
    let values = items
        .iter()
        .map(|v| v.get(field).cloned().unwrap_or(Value::Null))
        .collect::<Vec<_>>();
    if values
        != expected
            .as_array()
            .ok_or("expected")?
            .iter()
            .take(2)
            .cloned()
            .collect::<Vec<_>>()
    {
        return Err(format!("native classes or links changed: {result}"));
    }
    Ok(())
}
