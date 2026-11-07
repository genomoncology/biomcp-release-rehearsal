//! Every published query choice keeps its meaning across letter case and adapters.
use crate::runner::{
    http::Server,
    launch::Environment,
    mcp::{exchange_with_globals, request},
};
use libtest_mimic::Trial;
use serde_json::{Map, Value, json};
use std::io::Write;

pub(super) fn trials() -> Vec<Trial> {
    vec![
        crate::runner::transcript::trial("query-validation/enumerated-letter-case", || {
            check().map_err(Into::into)
        }),
        crate::runner::transcript::trial("query-validation/declared-field-sources", || {
            declared_sources().map_err(Into::into)
        }),
    ]
}

// A source fact is a promise: the command takes that field with that source.
fn declared_sources() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let mut failures = Vec::new();
    let mut count = 0;
    for (verb, entity, id) in biomcp_types::command_paths() {
        let schema = cli(&env, &["schema".into(), id.into()])?;
        let properties = field(&schema, "properties")
            .as_object()
            .ok_or("properties")?;
        let choices = choice_node(field(field(&schema, "properties"), "source"), &schema)
            .get("enum")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for (key, property) in properties {
            let fact = property.get("x-required-component");
            let sources = fact
                .and_then(Value::as_array)
                .cloned()
                .or_else(|| fact.map(|word| vec![word.clone()]))
                .unwrap_or_default();
            for source in sources.iter().filter(|source| choices.contains(source)) {
                let word = source.as_str().ok_or("source word")?;
                let mut values = seed(verb, entity, &schema)?;
                values.insert("source".into(), source.clone());
                values.extend(
                    fact_context(entity, key)
                        .as_object()
                        .ok_or("context")?
                        .clone(),
                );
                if (entity, key.as_str()) == ("pgx", "annotation_id") {
                    values.remove("gene"); // The annotation form takes its ID in place of a gene.
                }
                let sample = sample(key, property, &schema);
                values.insert(key.into(), sample);
                count += 1;
                let reply = cli(&env, &argv(verb, entity, &schema, &values))?;
                // Only an argument refusal breaks the promise; a later answer such as an
                // expired retained pool means the command took the field.
                if reply.get("status") == Some(&json!("usage")) {
                    failures.push(format!("{verb} {entity}.{key} with {word}: {reply}"));
                }
            }
        }
    }
    writeln!(
        std::io::stdout(),
        "FIELD_SOURCES pairs={count} failures={}",
        failures.len()
    )
    .map_err(|e| e.to_string())?;
    if failures.is_empty() && count > 0 {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn fact_context(entity: &str, key: &str) -> Value {
    match (entity, key) {
        ("pgx", "kind" | "snapshot") => json!({"section":"annotations"}),
        ("pgx", "diplotype") => json!({"section":"diplotype"}),
        ("pgx", "drug_id") => json!({"section":"recommendations"}),
        ("variant", "tumor_type") => json!({"build":"GRCh38"}),
        _ => json!({}),
    }
}

fn sample(key: &str, property: &Value, schema: &Value) -> Value {
    let node = choice_node(property, schema);
    if let Some(choice) = node
        .get("enum")
        .and_then(Value::as_array)
        .and_then(|v| v.first())
    {
        return choice.clone();
    }
    match key {
        "condition_id" => json!("MedGen:C0025202"),
        "device_id" => json!("P000001"),
        "drug_application" => json!("NDA012345"),
        "gene" => json!("BRAF"),
        "gene_id" => json!(673),
        "diplotype" => json!("*1/*2"),
        "drug_id" => json!("RxNorm:1234"),
        "annotation_id" => json!("PA166104949"),
        key if key.ends_with("_from") || key.ends_with("_to") => json!("2020-01-01"),
        "since" | "until" => json!("2020"),
        _ if node.get("type") == Some(&json!("boolean")) => json!(true),
        _ => json!("melanoma"),
    }
}

fn check() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed", "mcp_typed_output_schema":true}))
        .map_err(|e| e.to_string())?;
    local_choices(&env)?;
    let mut failures = Vec::new();
    let mut count = 0;
    let mut added = 0;
    for (verb, entity, id) in biomcp_types::command_paths() {
        let schema = cli(&env, &["schema".into(), id.into()])?;
        let properties = field(&schema, "properties")
            .as_object()
            .ok_or("properties")?;
        for (key, property) in properties {
            if property.get("x-cli-flags").is_some() {
                continue; // These are separate boolean flags on both public adapters.
            }
            let choices = choice_node(property, &schema).get("enum");
            let Some(choice) = choices.and_then(Value::as_array).and_then(|v| {
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
            }) else {
                continue;
            };
            count += 1;
            if let Err(error) = parameter(&env, verb, entity, &schema, key, choice) {
                failures.push(format!("{verb} {entity}.{key}: {error}"));
            }
            for word in listed(key, choices).filter(|word| *word != choice) {
                added += 1;
                if let Err(error) = listed_value(&env, (verb, entity), &schema, key, word) {
                    failures.push(format!("{verb} {entity}.{key}={word}: {error}"));
                }
            }
        }
    }
    hosted_sources(&env, &mut failures)?;
    writeln!(
        std::io::stdout(),
        "ENUM_CASE parameters={count} values={added} failures={}",
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
            crate::runner::protocol::typed_catalog::entity(
                &format!("{verb}_{entity}").replace('-', "_"),
                json!(values),
            ),
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

// Country holds about 230 names from one generated list, so a sample covers it.
fn listed<'a>(key: &str, choices: Option<&'a Value>) -> impl Iterator<Item = &'a str> {
    let step = if key == "country" { 25 } else { 1 };
    choices
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .step_by(step)
        .filter_map(Value::as_str)
}

// A rule about other fields may still refuse; only a refusal of this field is a false listing.
fn listed_value(
    env: &Environment,
    (verb, entity): (&str, &str),
    schema: &Value,
    key: &str,
    word: &str,
) -> Result<(), String> {
    let mut values = seed(verb, entity, schema)?;
    context(verb, entity, key, &mut values)?;
    let property = field(field(schema, "properties"), key);
    if let Some(scope) = choice_node(property, schema).get("x-enum-by-field") {
        scope_for(scope, word, &mut values);
    }
    let array = resolved(property, schema).get("type") == Some(&json!("array"));
    values.insert(key.into(), if array { json!([word]) } else { json!(word) });
    let reply = cli(env, &argv(verb, entity, schema, &values))?;
    if refuses(&reply, key, word) {
        return Err(format!("CLI refused a listed value: {reply}"));
    }
    let frame = request(
        json!(1),
        "tools/call",
        crate::runner::protocol::typed_catalog::entity(
            &format!("{verb}_{entity}").replace('-', "_"),
            json!(values),
        ),
    );
    let replies = exchange_with_globals(env, &[frame], &["--offline".into()])?;
    match replies
        .first()
        .and_then(|reply| reply.pointer("/result/structuredContent"))
    {
        Some(typed) if !refuses(typed, key, word) => Ok(()),
        typed => Err(format!("typed tool refused a listed value: {typed:?}")),
    }
}

fn scope_for(scope: &Value, word: &str, values: &mut Map<String, Value>) {
    let owner = field(scope, "choices").as_object().and_then(|choices| {
        choices
            .iter()
            .find(|(_, words)| words.as_array().is_some_and(|v| v.contains(&json!(word))))
    });
    if let (Some((owner, _)), Some(name)) = (owner, field(scope, "field").as_str()) {
        values.insert(name.into(), json!(owner));
    }
}

// A refusal that lists the word, names a source or offers other sources is a rule about
// other fields; only a refusal of this field that omits the word is a false listing.
fn refuses(reply: &Value, key: &str, word: &str) -> bool {
    let error = field(reply, "error");
    let option = error.get("option").and_then(Value::as_str);
    let listed = field(error, "valid_values")
        .as_array()
        .is_some_and(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .any(|value| value.eq_ignore_ascii_case(word))
        });
    error.get("code") == Some(&json!("usage.arguments"))
        && option.is_some_and(|option| option.trim_start_matches('-').replace('-', "_") == key)
        && !listed
        && error.get("source").is_none()
        && error.get("message")
            != Some(&json!(
                "a selected source does not support the requested filters"
            ))
}

// The hosted list must never offer a word that only reaches a source hosting has switched off.
fn hosted_sources(env: &Environment, failures: &mut Vec<String>) -> Result<(), String> {
    let server = Server::configured_globals(
        "127.0.0.1:0",
        Some(json!({"mcp_tool_mode":"typed"})),
        None,
        None,
        None,
        &["--offline".into(), "--hosted".into()],
    )?;
    let frame = request(json!(1), "tools/list", json!({}));
    let (_, catalog) = server.request("POST", "/mcp", &frame.to_string())?;
    let tools = catalog
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .ok_or("hosted tools")?;
    let tools = crate::runner::protocol::typed_catalog::commands(tools);
    for (verb, entity, id) in biomcp_types::command_paths() {
        let name = format!("{verb}_{entity}").replace('-', "_");
        let Some(tool) = tools.iter().find(|tool| tool["name"] == name.as_str()) else {
            continue;
        };
        let schema = cli(env, &["schema".into(), id.into()])?;
        for word in listed(
            "source",
            tool.pointer("/inputSchema/properties/source/enum"),
        ) {
            let mut values = seed(verb, entity, &schema)?;
            context(verb, entity, "source", &mut values)?;
            values.insert("source".into(), json!(word));
            let call = request(
                json!(2),
                "tools/call",
                crate::runner::protocol::typed_catalog::entity(&name, json!(values)),
            );
            let (_, reply) = server.request("POST", "/mcp", &call.to_string())?;
            if only_disabled(reply.pointer("/result/structuredContent/sources")) {
                failures.push(format!("hosted {name} offers refused source {word}"));
            }
        }
    }
    Ok(())
}

fn only_disabled(sources: Option<&Value>) -> bool {
    sources
        .and_then(Value::as_array)
        .filter(|sources| !sources.is_empty())
        .is_some_and(|sources| sources.iter().all(|s| s["reason"] == "disabled"))
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

pub(in crate::runner) fn seed(
    verb: &str,
    entity: &str,
    schema: &Value,
) -> Result<Map<String, Value>, String> {
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

pub(in crate::runner) fn argv(
    verb: &str,
    entity: &str,
    schema: &Value,
    values: &Map<String, Value>,
) -> Vec<String> {
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
