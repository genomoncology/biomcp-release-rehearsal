//! CIViC exact membership crosses the real CLI, source policy, POST body and TLS cache.
use super::{launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;

pub(super) fn trials() -> Vec<Trial> {
    // The peer supplies cursor-dependent pages and counts requests to prove that repeated or empty cursors cannot progress.
    vec![transcript_trial(
        crate::runner::pool_contract::VARIANT_CIVIC_CURSOR.0,
        || stalled_cursor().map_err(Into::into),
    )]
}
fn configure(environment: &Environment, stub: &Stub) -> Result<(), String> {
    let ca = environment.root().join("synthetic-ca.pem");
    stub.retain(environment.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    environment
        .config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())
}
fn run(environment: &Environment, args: &[&str]) -> Result<Value, String> {
    let args = args.iter().map(|a| (*a).into()).collect::<Vec<_>>();
    let out = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.code() != Some(4) || !out.stderr.is_empty() {
        return Err(format!("wrong cursor refusal exit {:?}", out.status.code()));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|e| format!("{e}: {}", String::from_utf8_lossy(&out.stdout)))
}

pub(super) fn civic_body(request: &str) -> Result<String, String> {
    let evidence = json!(
        {"totalCount":2,"nodes":[{"id":1409,"status":"ACCEPTED","evidenceType":"PREDICTIVE","evidenceLevel":"A",
        "evidenceDirection":"DOES_NOT_SUPPORT","significance":"RESISTANCE","disease":{"name":"melanoma","doid":"1909"},
        "therapies":[{"name":"dabrafenib"}],"therapyInteractionType":null,
        "source":{"citationId":"25741868","sourceType":"PUBMED"}}],"pageInfo":{"hasNextPage":true,
        "endCursor":"synthetic-e1"}}
    );
    let assertions = json!(
        {"totalCount":2,"nodes":[{"id":7,"status":"ACCEPTED","assertionType":"PREDICTIVE","assertionDirection":"SUPPORTS",
        "significance":"SENSITIVITYRESPONSE","ampLevel":"TIER_I_LEVEL_A","disease":{"name":"melanoma","doid":"1909"},
        "therapies":[{"name":"dabrafenib"},{"name":"trametinib"}],"therapyInteractionType":"COMBINATION",
        "variantOrigin":"SOMATIC"}],"pageInfo":{"hasNextPage":true,
        "endCursor":"synthetic-a1"}}
    );
    let mut reply = json!(
        {"data":{"molecularProfiles":{"totalCount":1,"nodes":[{"id":120,"name":"BRAF V600E","link":"/molecular-profiles/120",
        "isComplex":false,"variants":[{"id":12,"name":"V600E","link":"/variants/12","alleleRegistryId":"CA123643",
        "hgvsDescriptions":["NP_004324.2:p.Val600Glu"],"feature":{"name":"BRAF"}}],"evidenceItems":evidence,"assertions":assertions}],
        "pageInfo":{"hasNextPage":false,"endCursor":null}}} }
    );
    let evidence_base = "/data/molecularProfiles/nodes/0/evidenceItems";
    let mut second = reply
        .pointer(&format!("{evidence_base}/nodes/0"))
        .ok_or("first evidence row")?
        .clone();
    *second.get_mut("id").ok_or("second evidence ID")? = json!(9851);
    reply
        .pointer_mut(&format!("{evidence_base}/nodes"))
        .and_then(Value::as_array_mut)
        .ok_or("evidence rows")?
        .push(second);
    *reply
        .pointer_mut(&format!("{evidence_base}/totalCount"))
        .ok_or("evidence total")? = json!(3);
    let (_, body) = request
        .split_once("\r\n\r\n")
        .ok_or("request body missing")?;
    let request: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let base = "/data/molecularProfiles/nodes/0";
    let empty = json!({"totalCount":2,"nodes":[],"pageInfo":{"hasNextPage":true,"endCursor":null}});
    if request
        .pointer("/variables/evidence_after")
        .and_then(Value::as_str)
        .is_some()
    {
        *reply
            .pointer_mut(&format!("{base}/evidenceItems/nodes/0/id"))
            .ok_or("evidence id")? = json!(1411);
        reply
            .pointer_mut(&format!("{base}/evidenceItems/nodes"))
            .and_then(Value::as_array_mut)
            .ok_or("evidence rows")?
            .truncate(1);
        *reply
            .pointer_mut(&format!("{base}/evidenceItems/pageInfo/hasNextPage"))
            .ok_or("evidence page")? = json!(false);
        *reply
            .pointer_mut(&format!("{base}/assertions"))
            .ok_or("assertions")? = empty;
    } else if request
        .pointer("/variables/assertion_after")
        .and_then(Value::as_str)
        .is_some()
    {
        *reply
            .pointer_mut(&format!("{base}/assertions/nodes/0/id"))
            .ok_or("assertion id")? = json!(8);
        *reply
            .pointer_mut(&format!("{base}/assertions/pageInfo/hasNextPage"))
            .ok_or("assertion page")? = json!(false);
        *reply
            .pointer_mut(&format!("{base}/evidenceItems"))
            .ok_or("evidence")? = empty;
    }
    Ok(reply.to_string())
}

fn source_environment(
    body: fn(&str) -> Result<String, String>,
) -> Result<(Environment, Stub), String> {
    let environment = Environment::new().map_err(|e| e.to_string())?;
    let source = Stub::for_source("civicdb.org", "application/json", body)?;
    configure(&environment, &source)?;
    Ok((environment, source))
}

fn stalled_cursor() -> Result<(), String> {
    let (environment, source) = source_environment(stalled_body)?;
    for cursor in ["synthetic-e1", "empty"] {
        let value = run(
            &environment,
            &[
                "get",
                "variant",
                "CA123643",
                "civic",
                "--civic-profile",
                "120",
                "--civic-kind",
                "evidence",
                "--civic-cursor",
                cursor,
            ],
        )?;
        if value.get("result").is_some()
            || value.pointer("/status") != Some(&json!("failed"))
            || value.pointer("/sources/0/source") != Some(&json!("civic"))
            || value.pointer("/sources/0/state") != Some(&json!("failed"))
            || value.pointer("/sources/0/reason") != Some(&json!("bad-reply"))
        {
            return Err(format!(
                "nonprogressing native stream emitted evidence or a repeating action: {value}"
            ));
        }
    }
    source.finish()?;
    Ok(())
}
fn stalled_body(request: &str) -> Result<String, String> {
    let mut raw: Value = serde_json::from_str(&civic_body(request)?).map_err(|e| e.to_string())?;
    let base = "/data/molecularProfiles/nodes/0/evidenceItems";
    *raw.pointer_mut(&format!("{base}/pageInfo/hasNextPage"))
        .ok_or("page flag")? = json!(true);
    if request.contains("\"evidence_after\":\"empty\"") {
        *raw.pointer_mut(&format!("{base}/nodes")).ok_or("nodes")? = json!([]);
        *raw.pointer_mut(&format!("{base}/pageInfo/endCursor"))
            .ok_or("cursor")? = Value::Null;
    }
    Ok(raw.to_string())
}
