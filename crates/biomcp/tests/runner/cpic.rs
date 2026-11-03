//! Feature cpic-pgx: synthetic native retrieval through the actual CLI and MCP.
use super::required_contract;

mod rows;
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    // Keep the real HTTP headers, cache request counts and response-bound pages.
    [
        (required_contract::CPIC_PAIRS.0, "pairs"),
        (required_contract::CPIC_VIEW.0, "view"),
        (required_contract::CPIC_TABLE.0, "table"),
        (required_contract::CPIC_GUIDELINES.0, "guidelines"),
        (required_contract::CPIC_DIPLOTYPE.0, "diplotype"),
    ]
    .into_iter()
    .map(|(name, mode)| transcript_trial(name, move || scenario(mode).map_err(Into::into)))
    .collect()
}
fn environment(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"ca_bundle":ca,"proxy":format!("http://{}",stub.address)}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn both(env: &Environment, args: Vec<String>, exit: i32) -> Result<Value, String> {
    let cli = invoke(env, args.clone(), exit)?;
    super::schema::Registry::embedded()?.validate(&cli, &args, exit < 2)?;
    let start = args
        .iter()
        .position(|s| matches!(s.as_str(), "search" | "get"))
        .ok_or("missing command")?;
    let (globals, command) = args.split_at(start);
    let mcp = super::mcp::call_with_globals(env, command, exit >= 2, globals)?;
    if cli != mcp {
        return Err(format!("CLI/MCP differs: {cli} {mcp}"));
    }
    Ok(cli)
}
fn scenario(name: &'static str) -> Check {
    let (args, body) = rows::scenario(name)?;
    let returned = body.as_array().ok_or("native array")?.len();
    let total = if matches!(
        name,
        "pairs" | "view" | "table" | "guidelines" | "guidelines-many"
    ) {
        2
    } else {
        returned
    };
    let range = if returned == 0 {
        "*/0".into()
    } else {
        format!("0-{}/{total}", returned - 1)
    };
    let original = body.clone();
    let stub = Stub::counted(
        move |request| {
            check_request(name, request)?;
            let mut body = body.clone();
            if request.contains("offset=1") {
                if name == "pairs" {
                    *body.pointer_mut("/0/pairid").ok_or("synthetic field")? = json!(2);
                } else if name == "view" {
                    *body
                        .pointer_mut("/0/recommendationid")
                        .ok_or("synthetic field")? = json!(2);
                } else if name == "table" {
                    *body.pointer_mut("/0/id").ok_or("synthetic field")? = json!(2);
                }
            }
            Ok(body.to_string())
        },
        move |request| {
            if request.contains("offset=1") {
                "Content-Range: 1-1/2\r\n".into()
            } else {
                format!("Content-Range: {range}\r\n")
            }
        },
        move |_| if total > returned { 206 } else { 200 },
    )?;
    let env = environment(&stub)?;
    let record = env.root().join("record");
    let mut recording = vec!["--record".into(), record.to_string_lossy().into()];
    recording.extend(args.clone());
    let cold = invoke(&env, recording, 0)?;
    let warm = both(&env, args.clone(), 0)?;
    if cold != warm {
        return Err("warm cache lost native response".into());
    }
    let replay_env = Environment::new().map_err(|e| e.to_string())?;
    let mut replay = vec!["--replay".into(), record.to_string_lossy().into()];
    replay.extend(args.clone());
    if both(&replay_env, replay, 0)? != cold {
        return Err("recording lost native response".into());
    }
    verify_body(name, &cold, &original, total)?;
    markdown(&env, &args, cold.get("next").is_some())?;
    blocked(&env, &args, &record)?;
    let expected = follow_page(&env, &cold, total)?;
    if stub.finish()?.len() != expected {
        return Err("warm cache or disabled call opened HTTP".into());
    }
    Ok(())
}
fn check_request(name: &str, r: &str) -> Check {
    if !r.to_ascii_lowercase().contains("prefer: count=exact") {
        return Err("exact count header missing".into());
    }
    let route = match name {
        "pairs" => "pair_view",
        "card" => "gene",
        "view" | "missing-second-gene" => "recommendation_view",
        "table" => "recommendation",
        "guidelines" | "guidelines-many" => "guideline_summary_view",
        _ => "diplotype",
    };
    if !r.starts_with(&format!("GET /v1/{route}?")) {
        return Err(format!("wrong source route: {r}"));
    }
    if name == "table" && (!r.contains("drugid=eq.RxNorm%3A10324") || !r.contains("order=id.asc")) {
        return Err("native drug filter/order missing".into());
    }
    if matches!(name, "view" | "missing-second-gene")
        && (!r.contains("lookupkey-%3E%3EDPYD=not.is.null")
            || !r.contains("order=recommendationid.asc"))
    {
        return Err(format!("native view selector/order missing: {r}"));
    }
    Ok(())
}
fn verify_next(name: &str, value: &Value) -> Check {
    let empty = vec![];
    let steps = value
        .get("next")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    if name == "guidelines-many" {
        let genes = steps
            .iter()
            .filter(|s| s.get("kind") == Some(&json!("pivot")))
            .filter_map(|s| {
                s.get("args")
                    .and_then(Value::as_array)
                    .and_then(|args| args.get(2))
            })
            .collect::<Vec<_>>();
        let expected = ["TPMT", "NUDT15", "CYP2D6", "UGT1A1", "TP53", "BRCA1"].map(Value::from);
        if genes != expected.iter().collect::<Vec<_>>()
            || steps.last().and_then(|s| s.get("kind")) != Some(&json!("page"))
        {
            return Err(format!(
                "linked genes or page lost from guideline actions: {steps:?}"
            ));
        }
    }
    if steps
        .iter()
        .any(|s| s.get("kind") == Some(&json!("disambiguate")))
    {
        return Err("invented pair selection".into());
    }
    if name == "pairs" && steps.first().and_then(|s| s.get("kind")) != Some(&json!("page")) {
        return Err("page must precede card/recommendations".into());
    }
    if name == "card" && steps.len() != 3 {
        return Err("gene card must offer exactly implemented sections".into());
    }
    if steps.iter().filter_map(|s| s.get("args")).any(|a| {
        a.as_array()
            .is_some_and(|v| v.contains(&json!("help")) || v.contains(&json!("diplotype")))
    }) {
        return Err("help or diplotype invented next".into());
    }
    if matches!(name, "view" | "table")
        && steps.first().and_then(|s| s.get("kind")) != Some(&json!("page"))
    {
        return Err("recommendation page missing".into());
    }
    Ok(())
}

fn verify_body(name: &str, cold: &Value, original: &Value, total: usize) -> Check {
    let output_rows = if name == "card" {
        json!([cold.pointer("/result/identity").ok_or("identity")?])
    } else if matches!(name, "diplotype" | "empty") {
        cold.pointer("/result/diplotypes")
            .unwrap_or(&json!([]))
            .clone()
    } else {
        cold.pointer("/result/items").ok_or("items")?.clone()
    };
    let mut expected = original.clone();
    super::assertions::qualify_numeric_ids(&mut expected, "CPIC");
    super::assertions::omit_empty_lists(&mut expected);
    super::assertions::omit_null_fields(
        &mut expected,
        &[
            "guidelinename",
            "guidelineurl",
            "clinpgxlevel",
            "pgxtesting",
            "pmids",
            "comments",
        ],
    );
    if output_rows != expected {
        return Err(format!("native context changed: {output_rows}"));
    }
    if let Some(page) = cold.pointer("/result/page")
        && (page.get("total") != Some(&json!(total))
            || page.get("total_kind") != Some(&json!("source")))
    {
        return Err("native count lost".into());
    }
    if name == "table" && cold.pointer("/result/representation") != Some(&json!("table")) {
        return Err("table confused with view".into());
    }
    if name == "empty"
        && (cold.get("status") != Some(&json!("complete"))
            || cold.pointer("/sources/0/state") != Some(&json!("ok")))
    {
        return Err("Q33 source-local zero became failure or inferred normal".into());
    }
    if name == "missing-second-gene" && cold.pointer("/result/items/0/lookupkey/TPMT").is_some() {
        return Err("Q16 missing second gene acquired a result".into());
    }
    super::pgx_retirements::bounded(&format!("CPIC {name}"), cold)?;
    verify_next(name, cold)?;
    Ok(())
}
fn markdown(env: &Environment, args: &[String], has_next: bool) -> Check {
    let mut md = args.to_vec();
    md.extend(["--format".into(), "md".into()]);
    let text = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &md)
        .output()
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8(text.stdout).map_err(|e| e.to_string())?;
    if !text.contains("cpic")
        || (has_next && !text.contains("biomcp"))
        || text.trim_start().starts_with('{')
    {
        return Err("Markdown omitted source or follow-up commands".into());
    }
    Ok(())
}

fn follow_page(env: &Environment, value: &Value, total: usize) -> Result<usize, String> {
    if total != 2 {
        return Ok(1);
    }
    let args = value
        .get("next")
        .and_then(Value::as_array)
        .and_then(|v| v.iter().find(|s| s.get("kind") == Some(&json!("page"))))
        .and_then(|s| s.get("args"))
        .and_then(Value::as_array)
        .ok_or("missing next page")?
        .iter()
        .map(|s| {
            s.as_str()
                .map(str::to_owned)
                .ok_or("non-string next".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let page = both(env, args, 0)?;
    if page.pointer("/result/page/offset") != Some(&json!(1))
        || page.pointer("/result/page/returned") != Some(&json!(1))
        || page.pointer("/result/page/total") != Some(&json!(2))
    {
        return Err("next page lost its native count/offset".into());
    }
    for field in ["gene", "source", "drug_id", "cpic_level"] {
        let p = format!("/query/{field}");
        if page.pointer(&p) != value.pointer(&p) {
            return Err(format!("page lost {field}"));
        }
    }
    Ok(2)
}

fn blocked(env: &Environment, args: &[String], record: &std::path::Path) -> Check {
    for switch in ["cpic"] {
        for replay in [false, true] {
            let mut blocked = vec!["--disable-sources".into(), switch.into()];
            if replay {
                blocked.extend(["--replay".into(), record.to_string_lossy().into()]);
            }
            blocked.extend(args.iter().cloned());
            let value = both(env, blocked, 4)?;
            if value.pointer("/sources/0/reason") != Some(&json!("disabled")) {
                return Err("disabled source restored cache or replay".into());
            }
        }
    }
    Ok(())
}
