//! Feature clinpgx-annotations: native tiers through actual CLI and MCP.
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
        (required_contract::CLINPGX_CLINICAL.0, "clinical"),
        (required_contract::CLINPGX_KIND.0, "guideline"),
        (required_contract::CLINPGX_ANNOTATION.0, "annotation"),
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
fn args(name: &str) -> Vec<String> {
    if name == "annotation" {
        return [
            "get",
            "pgx",
            "--annotation-id",
            "PA100001",
            "--source",
            "clinpgx",
        ]
        .map(str::to_owned)
        .to_vec();
    }
    let mut args = [
        "get",
        "pgx",
        "--gene",
        "DPYD",
        "annotations",
        "--source",
        "clinpgx",
        "--section-limit",
        "1",
    ]
    .map(str::to_owned)
    .to_vec();
    if name == "default" {
        args.truncate(args.len().saturating_sub(2));
    }
    if name == "wide" {
        args = args
            .into_iter()
            .map(|word| if word == "1" { "5".into() } else { word })
            .collect();
    }
    if matches!(name, "guideline" | "label") {
        args.extend(["--kind".into(), name.into()]);
    }
    args
}
fn native(name: &str) -> Result<Value, String> {
    let data = match name {
        "annotation" => rows::detail(1)?,
        "clinical" => json!([rows::clinical(1, "DPYD"), rows::clinical(2, "DPYD")]),
        "wide" | "default" => Value::Array((1..=6).map(|id| rows::clinical(id, "DPYD")).collect()),
        "empty" => json!([]),
        "label" => rows::references("LabelAnnotation"),
        _ => rows::references("GuidelineAnnotation"),
    };
    Ok(json!({"data":data,"status":"success"}))
}
fn both(env: &Environment, args: Vec<String>, exit: i32) -> Result<Value, String> {
    let cli = invoke(env, args.clone(), exit)?;
    super::schema::Registry::embedded()?.validate(&cli, &args, exit < 2)?;
    let start = args
        .iter()
        .position(|s| matches!(s.as_str(), "get" | "search"))
        .ok_or("command")?;
    let (globals, command) = args.split_at(start);
    let mcp = super::mcp::call_with_globals(env, command, exit >= 2, globals)?;
    if cli != mcp {
        return Err("CLI/MCP differs".into());
    }
    Ok(cli)
}
fn scenario(name: &'static str) -> Check {
    let native = native(name)?;
    let original = native.clone();
    let stub = Stub::for_sources(
        &["api.clinpgx.org"],
        |_| "application/json",
        move |request| {
            request_matches(name, request)?;
            Ok(native.to_string())
        },
    )?;
    let env = environment(&stub)?;
    let args = args(name);
    let record = env.root().join("record");
    let mut recording = vec!["--record".into(), record.to_string_lossy().into()];
    recording.extend(args.clone());
    let cold = invoke(&env, recording, 0)?;
    if both(&env, args.clone(), 0)? != cold {
        return Err("warm cache changed native response".into());
    }
    super::pgx_retirements::bounded(&format!("ClinPGx {name}"), &cold)?;
    verify(name, &cold, &original)?;
    let fresh = Environment::new().map_err(|e| e.to_string())?;
    let mut replay = vec!["--replay".into(), record.to_string_lossy().into()];
    replay.extend(args.clone());
    if both(&fresh, replay, 0)? != cold {
        return Err("recorded native response changed".into());
    }
    if !matches!(name, "annotation" | "empty") {
        walk(&env, &cold)?;
    }
    blocked(&env, &args, &record)?;
    let mut md = vec!["--format".into(), "md".into()];
    md.extend(args);
    let text = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &md)
        .output()
        .map_err(|e| e.to_string())?;
    if !text.status.success() || !String::from_utf8_lossy(&text.stdout).contains("clinpgx") {
        return Err("Markdown lost source".into());
    }
    if stub.finish()?.len() != 1 {
        return Err("local page, cache, replay or disabled request opened HTTP".into());
    }
    Ok(())
}
fn request_matches(name: &str, r: &str) -> Check {
    let path = match name {
        "annotation" => "clinicalAnnotation/PA100001",
        "guideline" => "guidelineAnnotation",
        "label" => "labelAnnotation",
        _ => "clinicalAnnotation",
    };
    if !r.starts_with(&format!("GET /v1/data/{path}?"))
        || !r.contains(if name == "annotation" {
            "view=base"
        } else {
            "view=min"
        })
    {
        return Err(format!("wrong native route: {r}"));
    }
    if name != "annotation"
        && !r.contains(if matches!(name, "guideline" | "label") {
            "relatedGenes.symbol=DPYD"
        } else {
            "location.genes.symbol=DPYD"
        })
    {
        return Err("lost gene selector".into());
    }
    if r.contains("offset=") || r.contains("limit=") {
        return Err("invented provider pagination".into());
    }
    Ok(())
}
fn verify(name: &str, v: &Value, native: &Value) -> Check {
    if name == "empty"
        && (v.get("status") != Some(&json!("complete"))
            || v.pointer("/sources/0/state") != Some(&json!("ok")))
    {
        return Err("Q33 successful source-local zero became failure or inferred normal".into());
    }
    if name == "empty" {
        super::pgx_retirements::bounded("ClinPGx empty", v)?;
    }
    if name == "default"
        && super::source_terms::size(v, "")? > super::output_limit("clinpgx-clinical-page")?
    {
        return Err("clinical default exceeds its reviewed output ceiling".into());
    }
    if name == "annotation" {
        let mut expected = native.get("data").cloned().ok_or("native data")?;
        super::assertions::qualify_numeric_ids(&mut expected, "ClinPGx");
        super::assertions::omit_empty_lists(&mut expected);
        if v.pointer("/result/annotation") != Some(&expected) {
            return Err("base fields changed".into());
        }
        return Ok(());
    }
    let rows = v
        .pointer("/result/items")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let expected = native
        .get("data")
        .and_then(Value::as_array)
        .ok_or("native array")?;
    let mut expected_rows = json!(
        expected
            .iter()
            .take(match name {
                "wide" => 5,
                "default" => 3,
                _ => 1,
            })
            .cloned()
            .collect::<Vec<_>>()
    );
    super::assertions::qualify_numeric_ids(&mut expected_rows, "ClinPGx");
    super::assertions::omit_empty_lists(&mut expected_rows);
    if rows
        != expected_rows
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
        || v.pointer("/result/page/total") != Some(&json!(expected.len()))
        || v.pointer("/result/page/total_kind") != Some(&json!("observed"))
    {
        return Err("native rows or observed scope changed".into());
    }
    let empty = vec![];
    let next = v.get("next").and_then(Value::as_array).unwrap_or(&empty);
    if matches!(name, "clinical" | "default")
        && next.first().and_then(|v| v.get("kind")) != Some(&json!("disambiguate"))
    {
        return Err("clinical candidate not first".into());
    }
    if name == "wide"
        && (next.len() != 6
            || next
                .iter()
                .take(5)
                .any(|step| step.get("kind") != Some(&json!("disambiguate")))
            || next.last().and_then(|step| step.get("kind")) != Some(&json!("page")))
    {
        return Err("selectable page lost candidates before continuation".into());
    }
    if matches!(name, "guideline" | "label")
        && next
            .iter()
            .any(|v| v.get("kind") == Some(&json!("disambiguate")))
    {
        return Err("reference invented full guidance get".into());
    }
    Ok(())
}
fn walk(env: &Environment, v: &Value) -> Check {
    let next = v
        .get("next")
        .and_then(Value::as_array)
        .and_then(|v| v.iter().find(|v| v.get("kind") == Some(&json!("page"))))
        .and_then(|v| v.get("args"))
        .and_then(Value::as_array)
        .ok_or("page step")?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or("page argument".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !next.iter().any(|arg| arg == "--snapshot") {
        return Err("continuation lost retained collection selector".into());
    }
    let mut disabled = vec!["--disable-sources".into(), "noncommercial".into()];
    disabled.extend(next.clone());
    if both(env, disabled, 4)?.pointer("/sources/0/reason") != Some(&json!("disabled")) {
        return Err("retained collection bypassed source admission".into());
    }
    let second = both(env, next, 0)?;
    let returned = v
        .pointer("/result/page/returned")
        .and_then(Value::as_u64)
        .ok_or("returned")?;
    let total = v
        .pointer("/result/page/total")
        .and_then(Value::as_u64)
        .ok_or("total")?;
    if second.pointer("/result/page/offset") != v.pointer("/result/page/returned")
        || second.pointer("/result/page/returned") != Some(&json!(returned.min(total - returned)))
        || second.pointer("/result/items/0") == v.pointer("/result/items/0")
    {
        return Err("local page skipped or repeated row".into());
    }
    for key in ["source", "gene", "kind", "section_limit"] {
        let p = format!("/query/{key}");
        if second.pointer(&p) != v.pointer(&p) {
            return Err(format!("page lost {key}"));
        }
    }
    Ok(())
}
fn blocked(env: &Environment, args: &[String], record: &std::path::Path) -> Check {
    for source in ["clinpgx", "noncommercial"] {
        for replay in [false, true] {
            let mut command = vec!["--disable-sources".into(), source.into()];
            if replay {
                command.extend(["--replay".into(), record.to_string_lossy().into()]);
            }
            command.extend(args.iter().cloned());
            let v = both(env, command, 4)?;
            if v.pointer("/sources/0/reason") != Some(&json!("disabled")) {
                return Err("disabled source restored data".into());
            }
        }
    }
    Ok(())
}
