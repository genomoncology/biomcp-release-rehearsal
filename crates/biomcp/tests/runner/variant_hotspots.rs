//! Native residue counts remain candidates when transcript equivalence is not established.
use super::{launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
pub(super) fn trials() -> Vec<Trial> {
    // The peer checks the exact bounded gene endpoint and that cached or disabled reads send no extra request.
    vec![transcript_trial(
        crate::runner::required_contract::HOTSPOTS.0,
        || candidate().map_err(Into::into),
    )]
}
fn candidate() -> Result<(), String> {
    let stub = Stub::for_sources(
        &["myvariant.info", "www.cancerhotspots.org"],
        |_| "application/json",
        body,
    )?;
    let environment = Environment::new().map_err(|e| e.to_string())?;
    let ca = environment.root().join("synthetic-ca.pem");
    stub.retain(environment.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    environment
        .config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    let args = [
        "get",
        "variant",
        "chr7:g.140753336A>T",
        "--build",
        "GRCh38",
        "hotspots",
    ]
    .map(Into::into);
    let out = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let value: Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    envelope(&value)?;
    let base = "/result/candidates/0/sections/hotspots/data";
    for (path, expected) in [
        ("/records/0/membership", json!("candidate")),
        ("/records/0/transcript", json!("ENST00000288602")),
        ("/records/0/residue", json!("V600")),
        ("/records/0/tumor_count", json!(12)),
        ("/records/0/alternates/0/count", json!(8)),
    ] {
        if value.pointer(&format!("{base}{path}")) != Some(&expected) {
            return Err(format!("native hotspot candidate lost at{path}: {value}"));
        }
    }
    let mcp = super::mcp::call_with_globals(&environment, &args, false, &[])?;
    if mcp.get("result") != value.get("result") {
        return Err("hotspots CLI/MCP source mapping differs".into());
    }
    let disabled_args = [
        "get",
        "variant",
        "chr7:g.140753336A>T",
        "--build",
        "GRCh38",
        "hotspots",
        "--disable-sources",
        "hotspots",
    ]
    .map(Into::into);
    let disabled = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &disabled_args)
        .output()
        .map_err(|e| e.to_string())?;
    let disabled: Value = serde_json::from_slice(&disabled.stdout).map_err(|e| e.to_string())?;
    if disabled.pointer("/result/candidates/0/sections/hotspots/reason") != Some(&json!("disabled"))
    {
        return Err("disabled hotspots did not skip before joins".into());
    }
    let requests = stub.finish()?;
    if requests.len() != 2
        || !requests.iter().any(|(_, request)| {
            request.starts_with("GET /api/hotspots/single/byGene/BRAF HTTP/1.1")
        })
    {
        return Err("hotspots did not use its declared bounded gene endpoint".into());
    }
    Ok(())
}
pub(super) fn body(request: &str) -> Result<String, String> {
    if request.starts_with("GET /v1/variant/") {
        Ok(json!({"_id":"chr7:g.140753336A>T",
        "chrom":"7","hg38":{"start":140753336,"end":140753336},
        "dbsnp":{"rsid":"rs113488022","gene":{"symbol":"BRAF"}},"clinvar":{"variant_id":13961}})
        .to_string())
    } else {
        // Counts are deliberately synthetic; provider data rights remain unverified.
        Ok(json!(
            [{"hugoSymbol":"BRAF","transcriptId":"ENST00000288602","residue":"V600","tumorCount":12,"variantAminoAcid":{"E":8,"K":4}},
            {"hugoSymbol":"BRAF","transcriptId":"ENST00000288602","residue":"D594","tumorCount":5,"variantAminoAcid":{"G":5}},
            {"hugoSymbol":"BRAF","transcriptId":"ENST00000288602","residue":"G466","tumorCount":3,"variantAminoAcid":{"A":3}}]
        )
        .to_string())
    }
}

fn envelope(value: &Value) -> Result<(), String> {
    let full = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    let mut comparable = super::source_terms::comparable(value, "")?;
    let source_terms = full
        - serde_json::to_vec(&comparable)
            .map_err(|e| e.to_string())?
            .len();
    let header = comparable
        .pointer_mut("/result/candidates/0")
        .and_then(Value::as_object_mut)
        .ok_or("missing candidate")?;
    let terms = header
        .remove("component_terms")
        .ok_or("missing component terms")?;
    let expected = biomcp_query::sources::component_label(
        biomcp_query::sources::components()
            .find(|component| {
                component.source == biomcp_types::Source::MyVariant && component.name == "clinvar"
            })
            .ok_or("missing component declaration")?
            .terms,
    )
    .map_err(|e| e.to_string())?;
    let component_bytes = serde_json::to_vec(&json!({"component_terms": &terms}))
        .map_err(|e| e.to_string())?
        .len()
        - 1;
    let payload = serde_json::to_vec(&comparable)
        .map_err(|e| e.to_string())?
        .len();
    if terms != json!([{"component":"clinvar","terms":expected}])
        || full - payload != component_bytes + source_terms
        || payload > crate::runner::output_limit("hotspots-payload")?
        || full - source_terms > crate::runner::output_limit("hotspots-envelope")?
    {
        return Err(format!(
            "hotspots full={full}, comparable={payload}; caps=1771/1536"
        ));
    }
    Ok(())
}
