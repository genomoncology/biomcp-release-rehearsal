//! Retained GWAS pages with derived field/identity faults and a synthetic TLS transport.
use super::super::{configuration::invoke, launch, network_stub::Stub, schema::Registry, variant};
use super::{Check, save};
use biomcp_types::Paged;
use biomcp_types::{GwasPage, GwasPageQuery};
use serde_json::{Value, json};
use std::path::Path;

const ARGS: [&str; 6] = ["search", "gwas", "--pubmed", "17529967", "--limit", "3"];
const TARGET: &str = "/gwas/rest/api/v2/associations?direction=asc&page=";
const TAIL: &str = "&pubmed_id=17529967&size=3&sort=p_value HTTP/1.1";

pub(super) fn run(directory: &Path, registry: &Registry) -> Check {
    let mut failures = Vec::new();
    for fault in [
        "positive",
        "metadata",
        "identities",
        "zero-identities",
        "publication",
        "selector-shape",
        "self",
    ] {
        let result = scenario(directory, registry, fault);
        if let Err(error) = result {
            failures.push(format!("{fault}: {error}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn scenario(directory: &Path, registry: &Registry, fault: &str) -> Check {
    let mut first = native("page0.body")?;
    let second = native("page1.body")?;
    alter(&mut first, fault)?;
    let refused = matches!(fault, "publication" | "selector-shape" | "self");
    let all_bad = matches!(fault, "identities" | "zero-identities");
    let stub = Stub::for_source("www.ebi.ac.uk", "application/json", move |request| {
        let line = request.lines().next().ok_or("GWAS request line missing")?;
        if line == format!("GET {TARGET}0{TAIL}") {
            Ok(first.to_string())
        } else if line == format!("GET {TARGET}1{TAIL}") {
            Ok(second.to_string())
        } else {
            Err(format!("undeclared GWAS request: {line}"))
        }
    })?;
    let env = variant::source_environment(&stub)?;
    let args = ARGS.map(str::to_owned).to_vec();
    let first = execute(&env, registry, &args, false, if refused { 4 } else { 0 })?;
    save(directory, &format!("gwas-{fault}-first"), &first)?;
    if refused {
        reason(&first, "bad-reply")?;
    } else {
        window(&first, 0, if all_bad { &[] } else { &[6110, 6111, 6112] })?;
        fields(&first, fault)?;
        let (next, expected_query) = continuation(&first)?;
        let last = execute(&env, registry, &next, false, 0)?;
        save(directory, &format!("gwas-{fault}-next"), &last)?;
        window(&last, 3, &[6113, 6114, 11346])?;
        let cached = execute(&env, registry, &next, true, 0)?;
        window(&cached, 3, &[6113, 6114, 11346])?;
        if query(&last)? != expected_query || query(&cached)? != expected_query {
            return Err("GWAS continuation changed query selectors or page size".into());
        }
    }
    let uncached = refused || all_bad;
    let offline = execute(&env, registry, &args, true, if uncached { 4 } else { 0 })?;
    save(directory, &format!("gwas-{fault}-offline"), &offline)?;
    if uncached {
        reason(&offline, "offline")?;
    } else {
        window(&offline, 0, &[6110, 6111, 6112])?;
        fields(&offline, fault)?;
    }
    let requests = stub.finish()?;
    if requests.len() != if refused { 1 } else { 2 } {
        return Err("GWAS offline read contacted the source or native page count changed".into());
    }
    Ok(())
}

fn native(file: &str) -> Check<Value> {
    let text = launch::fixture_text(&format!("fixtures/gwas/native-window/{file}"))?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn alter(body: &mut Value, fault: &str) -> Check {
    if fault == "positive" {
        return Ok(());
    }
    if fault == "metadata" {
        *body
            .pointer_mut("/_embedded/associations/0/first_author")
            .ok_or("native GWAS author missing")? = json!({"derived_bad_shape":true});
        return Ok(());
    }
    for row in body
        .pointer_mut("/_embedded/associations")
        .and_then(Value::as_array_mut)
        .ok_or("native GWAS associations missing")?
    {
        *row.get_mut("association_id")
            .ok_or("native GWAS identity missing")? = if fault == "zero-identities" {
            json!(0)
        } else {
            json!({"derived_bad_shape":true})
        };
    }
    match fault {
        "identities" | "zero-identities" => (),
        "publication" | "selector-shape" => {
            *body
                .pointer_mut("/_embedded/associations/0/pubmed_id")
                .ok_or("native GWAS selector missing")? = if fault == "publication" {
                json!("17529974")
            } else {
                json!({"derived_bad_shape":true})
            };
        }
        "self" => {
            let link = body
                .pointer_mut("/_links/self/href")
                .ok_or("native GWAS self link missing")?;
            *link = json!(
                link.as_str()
                    .ok_or("native self link is not text")?
                    .replace("17529967", "17529974")
            );
        }
        _ => return Err("unknown GWAS derived fault".into()),
    }
    Ok(())
}

fn execute(
    env: &launch::Environment,
    registry: &Registry,
    args: &[String],
    offline: bool,
    exit: i32,
) -> Check<Value> {
    let mut command: Vec<_> = args
        .iter()
        .filter(|word| word.as_str() != "--fresh")
        .cloned()
        .collect();
    // The shared launcher isolates the normal cache in this environment's root.
    // The grammar permits --case-cache only with replay or offline mode.
    command.push(if offline { "--offline" } else { "--fresh" }.into());
    let value = invoke(env, command, exit)?;
    registry.validate(&value, args, exit == 0)?;
    Ok(value)
}

fn window(value: &Value, offset: u32, ids: &[u64]) -> Check {
    // GwasPage's public serde contract omits an empty items vector.
    let actual: Vec<_> = page(value)?.items.iter().map(|row| row.id.get()).collect();
    if actual.as_slice() != ids
        || value.pointer("/page/offset") != Some(&json!(offset))
        || value.pointer("/page/returned") != Some(&json!(ids.len()))
        || value.pointer("/page/total") != Some(&json!(6))
    {
        return Err("GWAS refused identities shifted native positions, IDs or denominator".into());
    }
    Ok(())
}

fn fields(value: &Value, fault: &str) -> Check {
    if fault == "positive" {
        return Ok(());
    }
    let field = if fault == "metadata" {
        "first_author"
    } else {
        "association_id"
    };
    let problems = value
        .get("problems")
        .and_then(Value::as_array)
        .ok_or("GWAS problems missing")?;
    if problems.len() != 1
        || problems.first().and_then(|problem| problem.get("field")) != Some(&json!(field))
    {
        return Err(format!("GWAS lost the precise {field} problem"));
    }
    if fault == "metadata" && value.pointer("/result/items/0/author").is_some() {
        return Err("GWAS kept malformed author metadata".into());
    }
    Ok(())
}

fn continuation(value: &Value) -> Check<(Vec<String>, GwasPageQuery)> {
    let GwasPageQuery::Search(original) = query(value)? else {
        return Err("GWAS native search echoed a different operation".into());
    };
    let expected = GwasPageQuery::Search(original.at_offset(3));
    if page(value)?.continuation.as_ref() != Some(&expected) {
        return Err("GWAS native continuation changed selectors or consumed offset".into());
    }
    let args = value
        .get("next")
        .and_then(Value::as_array)
        .and_then(|steps| {
            steps
                .iter()
                .find(|step| step.get("kind") == Some(&json!("page")))
        })
        .and_then(|step| step.get("args"))
        .and_then(Value::as_array)
        .ok_or("GWAS all-bad native page lost its continuation")?
        .iter()
        .map(|word| {
            word.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "GWAS continuation requires text arguments".into())
        })
        .collect::<Check<Vec<_>>>()?;
    Ok((args, expected))
}

fn page(value: &Value) -> Check<GwasPage> {
    serde_json::from_value(value.get("result").cloned().ok_or("GWAS result missing")?)
        .map_err(|error| error.to_string())
}

fn query(value: &Value) -> Check<GwasPageQuery> {
    serde_json::from_value(value.get("query").cloned().ok_or("GWAS query missing")?)
        .map_err(|error| error.to_string())
}

fn reason(value: &Value, expected: &str) -> Check {
    if value.pointer("/sources/0/reason") != Some(&json!(expected)) || value.get("result").is_some()
    {
        return Err(format!(
            "GWAS expected {expected} refusal without results: {value}"
        ));
    }
    Ok(())
}
