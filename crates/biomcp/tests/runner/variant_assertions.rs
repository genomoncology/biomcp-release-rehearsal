//! Source assertion identity, source-local absence and cache admission through the CLI.
use super::{launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
pub(super) fn trials() -> Vec<Trial> {
    // The test walks runtime assertion pages and verifies native row reconstruction and cached continuation ownership.
    vec![transcript_trial(
        crate::runner::required_contract::CLINVAR_PAGES.0,
        || native_walk().map_err(Into::into),
    )]
}
fn environment(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("synthetic-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}

fn native_walk() -> Result<(), String> {
    let stub = Stub::for_source(
        "eutils.ncbi.nlm.nih.gov",
        "application/xml",
        native_walk_body,
    )?;
    let env = environment(&stub)?;
    let mut args = [
        "get",
        "variant",
        "clinvar:128042",
        "clinvar",
        "--section-limit",
        "1",
    ]
    .map(String::from)
    .to_vec();
    let mut records = Vec::new();
    for offset in 0..4 {
        let output = env
            .command(env!("CARGO_BIN_EXE_biomcp"), &args)
            .output()
            .map_err(|e| e.to_string())?;
        let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
        if super::mcp::call_with_globals(&env, &args, false, &[])? != value {
            return Err("native assertion CLI/MCP mismatch".into());
        }
        records.push(native_row(&value, offset)?);
        let page = value
            .get("next")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|n| n.get("kind") == Some(&json!("page")));
        if offset < 3 {
            args = serde_json::from_value(
                page.and_then(|p| p.get("args"))
                    .cloned()
                    .ok_or("missing runnable assertion continuation")?,
            )
            .map_err(|e| e.to_string())?;
            if args.first().map(String::as_str) != Some("variant")
                || args.get(1).map(String::as_str) != Some("assertions")
            {
                return Err(format!("invalid continuation spelling: {args:?}"));
            }
        } else if page.is_some() {
            return Err("unexpected fifth assertion page".into());
        }
    }
    if records
        != [
            json!("VCV000128042"),
            json!("SCV000611258"),
            json!("SCV000601142"),
            json!("SCV000607207"),
        ]
        || stub.finish()?.len() != 1
    {
        return Err(format!(
            "native rows merged, reordered or requested again: {records:?}"
        ));
    }
    Ok(())
}

fn native_walk_body(request: &str) -> Result<String, String> {
    let xml = super::variant_assertion_wire::clinvar_body(request)?;
    // Explicit synthetic XML retains the public record's native not-provided classification.
    Ok(xml.replace(
        "</ClinicalAssertionList>",
        "<ClinicalAssertion><ClinVarAccession Accession=\"SCV000601142\" \
        Version=\"6\"/><Classification><GermlineClassification>pathogenic</GermlineClassification></Classification></ClinicalAssertion>\
                    <ClinicalAssertion><ClinVarAccession Accession=\"SCV000607207\" \
        Version=\"3\" SubmitterName=\"GenomeConnect, ClinGen\"/><Classification><ReviewStatus>no \
        classification provided</ReviewStatus><GermlineClassification>not \
        provided</GermlineClassification></Classification></ClinicalAssertion></ClinicalAssertionList>",
    ))
}

fn native_row(value: &Value, offset: u32) -> Result<Value, String> {
    let data = value
        .pointer("/result/candidates/0/sections/clinvar/data")
        .or_else(|| value.get("result"))
        .ok_or("missing assertion body")?;
    if data.pointer("/page/offset") != Some(&json!(offset))
        || data.pointer("/page/total") != Some(&json!(4))
    {
        return Err(format!("native assertion page lost bounds: {value}"));
    }
    let row = data
        .pointer("/assertions/0")
        .ok_or("missing native assertion")?;
    let record = row
        .pointer("/statement/attribution/record")
        .or_else(|| row.pointer("/unmapped/attribution/record"))
        .ok_or("missing source record")?;
    if offset == 3
        && (row.get("statement_unmapped") != Some(&json!(true))
            || row.pointer("/unmapped/classification") != Some(&json!("not provided"))
            || row.get("statement").is_some()
            || value.get("problems").is_some())
    {
        return Err(format!(
            "unsupported category invented a proposition or problem: {value}"
        ));
    }
    if offset == 2
        && (row.pointer("/statement/classification/label") != Some(&json!("pathogenic"))
            || row.pointer("/statement/proposition") != Some(&json!("pathogenicity"))
            || row.pointer("/statement/classification/codings").is_some())
    {
        return Err(format!(
            "native casing changed label or inferred framework: {value}"
        ));
    }
    Ok(record.clone())
}

// Response-only synthetic label edit. This does not claim a real drug-response assertion.
