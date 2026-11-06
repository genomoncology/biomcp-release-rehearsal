//! Every published query choice keeps its meaning across letter case and adapters.
use crate::runner::{
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use libtest_mimic::Trial;
use serde_json::{Map, Value, json};
use std::io::Write;

pub(super) fn trials() -> Vec<Trial> {
    vec![crate::runner::transcript::trial(
        "query-validation/enumerated-letter-case",
        || check().map_err(Into::into),
    )]
}

fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed", "mcp_typed_output_schema":true}))
        .map_err(|e| e.to_string())?;
    local_choices(&env)?;
    let mut failures = Vec::new();
    let mut count = 0;
    for (verb, entity, id) in biomcp_types::command_paths() {
        let schema = cli(&env, &["schema".into(), id.into()])?;
        let properties = field(&schema, "properties")
            .as_object()
            .ok_or("properties")?;
        for (key, property) in properties {
            if property.get("x-cli-flags").is_some() {
                continue; // These are separate boolean flags on both public adapters.
            }
            let Some(choice) = choice_node(property, &schema)
                .get("enum")
                .and_then(Value::as_array)
                .and_then(|v| {
                    property
                        .get("default")
                        .filter(|value| v.contains(value))
                        .and_then(Value::as_str)
                        .or_else(|| v.iter().filter_map(Value::as_str).find(|s| !s.is_ascii()))
                        .or_else(|| {
                            v.iter()
                                .filter_map(Value::as_str)
                                .find(|s| s.to_lowercase() != s.to_uppercase())
                        })
                        .or_else(|| v.iter().find_map(Value::as_str))
                })
            else {
                continue;
            };
            count += 1;
            if let Err(error) = parameter(&env, verb, entity, &schema, key, choice) {
                failures.push(format!("{verb} {entity}.{key}: {error}"));
            }
        }
    }
    writeln!(
        std::io::stdout(),
        "ENUM_CASE parameters={count} failures={}",
        failures.len()
    )
    .map_err(|e| e.to_string())?;
    if failures.is_empty() && count > 0 {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn parameter(
    env: &Environment,
    verb: &str,
    entity: &str,
    schema: &Value,
    key: &str,
    canonical: &str,
) -> Result<(), String> {
    let mut values = seed(verb, entity, schema)?;
    let property = field(field(schema, "properties"), key);
    let array = resolved(property, schema).get("type") == Some(&json!("array"));
    let value = |text: &str| if array { json!([text]) } else { json!(text) };
    context(verb, entity, key, &mut values)?;
    values.insert(key.into(), value(canonical));
    let expected = cli(env, &argv(verb, entity, schema, &values))?;
    if expected.get("query").is_none() {
        return Err(format!(
            "canonical baseline did not reach an accepted query: {expected}"
        ));
    }
    let typed = |values: &Map<String, Value>| {
        let frame = request(
            json!(1),
            "tools/call",
            json!({
                "name":format!("{verb}_{entity}").replace('-', "_"), "arguments":values
            }),
        );
        exchange_with_globals(env, &[frame], &["--offline".into()])
    };
    let typed_expected = typed(&values)?;
    if typed_expected
        .first()
        .and_then(|reply| reply.pointer("/result/structuredContent/query"))
        != expected.get("query")
    {
        return Err(format!(
            "canonical typed query differs from CLI query: CLI {expected}; typed {typed_expected:?}"
        ));
    }
    for spelling in [canonical.to_lowercase(), canonical.to_uppercase()] {
        values.insert(key.into(), value(&spelling));
        let actual = cli(env, &argv(verb, entity, schema, &values))?;
        if actual != expected {
            return Err(format!("CLI {spelling}: expected {expected}; got {actual}"));
        }
        let replies = typed(&values)?;
        if replies != typed_expected {
            return Err(format!(
                "typed {spelling}: expected {typed_expected:?}; got {replies:?}"
            ));
        }
    }
    Ok(())
}

fn choice_node<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
    let node = resolved(node, root);
    node.get("items")
        .map_or(node, |items| choice_node(items, root))
}

fn resolved<'a>(node: &'a Value, root: &'a Value) -> &'a Value {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        return resolved(
            root.pointer(reference.trim_start_matches('#'))
                .unwrap_or(node),
            root,
        );
    }
    if let Some(branch) = node
        .get("anyOf")
        .and_then(Value::as_array)
        .and_then(|v| v.iter().find(|n| n.get("type") != Some(&json!("null"))))
    {
        return resolved(branch, root);
    }
    node
}

fn seed(verb: &str, entity: &str, schema: &Value) -> Result<Map<String, Value>, String> {
    let mut values = Map::new();
    for key in schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let key = key.as_str().ok_or("required name")?;
        let node = resolved(field(field(schema, "properties"), key), schema);
        let text = node
            .get("enum")
            .and_then(Value::as_array)
            .and_then(|v| v.first())
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                operand(
                    if matches!(verb, "get" | "search") {
                        entity
                    } else {
                        verb
                    },
                    key,
                )
            });
        values.insert(
            key.into(),
            if node.get("type") == Some(&json!("array")) {
                json!([text])
            } else {
                json!(text)
            },
        );
    }
    if entity == "pgx" {
        values.insert("gene".into(), json!("CYP2D6"));
    }
    if verb == "search" {
        for key in ["query", "text"] {
            if field(schema, "properties").get(key).is_some() {
                values.insert(key.into(), json!("melanoma"));
            }
        }
    }
    Ok(values)
}

fn operand<'a>(entity: &str, key: &'a str) -> &'a str {
    match key {
        "gene" => "KRAS",
        "study" => "study",
        "profile" => "profile",
        "sample_set" => "all",
        "text" | "query" => "melanoma",
        "pmid" => "12345678",
        "hgvs" => "NM_000059.4:c.68_69del",
        _ => match entity {
            "variant" => "rs7412",
            "trial" => "NCT00000001",
            "article" => "12345678",
            "protein" => "P01116",
            "disease" => "MONDO:0007254",
            "phenotype" => "HP:0001250",
            "pathway" => "R-HSA-109581",
            "drug" => "CHEMBL941",
            "author" => "orcid:0000-0002-1825-0097",
            "adverse-event" => "10000001",
            "diagnostic" => "GTR000000001.1",
            "biomarker" => "C96022",
            "cell-line" => "CVCL_0030",
            "gwas" => "association:6114",
            _ => "KRAS",
        },
    }
}

fn argv(verb: &str, entity: &str, schema: &Value, values: &Map<String, Value>) -> Vec<String> {
    let mut args = vec!["--offline".into(), verb.into(), entity.into()];
    let mut positional = Vec::new();
    for (key, value) in values {
        let property = field(field(schema, "properties"), key);
        let words = if let Some(values) = value.as_array() {
            values.clone()
        } else {
            vec![value.clone()]
        };
        let words = words
            .iter()
            .map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned))
            .collect::<Vec<_>>();
        if let Some(index) = property.get("x-positional").and_then(Value::as_u64) {
            positional.push((index, words));
        } else {
            for word in words {
                args.push(format!("--{}={word}", key.replace('_', "-")));
            }
        }
    }
    positional.sort_by_key(|(index, _)| *index);
    args.extend(positional.into_iter().flat_map(|(_, words)| words));
    args
}

fn cli(env: &Environment, args: &[String]) -> Result<Value, String> {
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), args)
        .output()
        .map_err(|e| e.to_string())?;
    serde_json::from_slice(&output.stdout).map_err(|e| format!("CLI envelope: {e}"))
}

fn local_choices(env: &Environment) -> Result<(), String> {
    for (before, choice, after) in [
        (vec!["--format"], "json", vec!["version"]),
        (vec!["--offline", "sync"], "hgnc", vec![]),
        (vec!["key", "set"], "oncokb", vec![]),
        (vec!["key", "remove"], "oncokb", vec![]),
        (vec!["help", "key"], "oncokb", vec![]),
        (vec!["help", "network"], "proxy", vec![]),
        (vec!["serve", "--tool-mode"], "typed", vec![]),
        (vec!["mcp-config", "--client"], "mcp-servers", vec![]),
    ] {
        let run = |spelling: &str| {
            let args = before
                .iter()
                .map(|v| (*v).to_owned())
                .chain([spelling.into()])
                .chain(after.iter().map(|v| (*v).to_owned()))
                .collect::<Vec<_>>();
            env.command(env!("CARGO_BIN_EXE_biomcp"), &args)
                .output()
                .map_err(|e| e.to_string())
        };
        let expected = run(choice)?;
        for spelling in [choice.to_ascii_lowercase(), choice.to_ascii_uppercase()] {
            let actual = run(&spelling)?;
            if actual.status != expected.status || actual.stdout != expected.stdout {
                return Err(format!(
                    "local enum {before:?} {spelling}: {} / {}",
                    String::from_utf8_lossy(&expected.stdout),
                    String::from_utf8_lossy(&actual.stdout)
                ));
            }
        }
    }
    Ok(())
}

fn field<'a>(node: &'a Value, key: &str) -> &'a Value {
    node.get(key).unwrap_or(&Value::Null)
}

// Valid source and section contexts keep semantic refusals from masking parser behavior.
fn context(
    verb: &str,
    entity: &str,
    key: &str,
    values: &mut Map<String, Value>,
) -> Result<(), String> {
    let extra = match (verb, entity, key) {
        ("get", "drug", "civic_kind" | "evidence_level" | "evidence_status") => {
            json!({"source":"civic", "section":["civic"], "civic_kind":"evidence"})
        }
        ("get", "variant", "civic_kind" | "evidence_level" | "evidence_status") => {
            json!({"section":["civic"], "civic_kind":"evidence", "civic_profile":1})
        }
        ("get", "pgx", "kind") => json!({"source":"clinpgx", "section":"annotations"}),
        ("get", "disease", "ontology") => json!({"source":"ols"}),
        ("search", "drug", "classification") => json!({"source":"fda-drug-recall"}),
        ("search", "drug", "name_selector") => json!({"source":"chembl"}),
        ("search", "adverse-event", "section") => json!({"source":"vaers", "text":"COVID19"}),
        ("search", "trial", "biomarker_use") => json!({"source":"nci", "biomarker_code":"C96022"}),
        ("search", "variant", _) => json!({"gene":"KRAS", "protein":"p.G12D"}),
        _ => json!({}),
    };
    values.extend(extra.as_object().ok_or("context")?.clone());
    if (verb, entity) == ("search", "variant")
        || (verb, entity, key) == ("search", "trial", "biomarker_use")
    {
        values.remove("query");
    }
    Ok(())
}
