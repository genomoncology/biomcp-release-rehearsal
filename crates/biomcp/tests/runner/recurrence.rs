//! Explicit cohort bounds cross the executable and TLS; no clinical interpretation.
mod admission;
use super::{launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
pub(super) fn trials() -> Vec<Trial> {
    let mut trials = vec![transcript_trial(
        "variant-recurrence/exact-page-wire-and-warm-cache",
        || wire().map_err(Into::into),
    )];
    trials.push(transcript_trial(
        "variant-recurrence/mutations-cache-admission",
        || admission::check().map_err(Into::into),
    ));
    trials
}
pub(super) fn args() -> Vec<String> {
    [
        "variant",
        "recurrence",
        "CA123643",
        "--source",
        "cbioportal",
        "--gene",
        "BRAF",
        "--study",
        "msk_impact_2017",
        "--profile",
        "msk_impact_2017_mutations",
        "--sample-set",
        "msk_impact_2017_all",
        "--limit",
        "3",
    ]
    .into_iter()
    .map(Into::into)
    .collect()
}
pub(super) fn config(env: &Environment, stub: &Stub) -> Result<(), String> {
    let ca = env.root().join("synthetic.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())
}
pub(super) fn call(env: &Environment, args: &[String], exit: i32) -> Result<Value, String> {
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), args)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) {
        return Err(format!(
            "expected exit {exit}, got {:?}: {} {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    super::schema::Registry::embedded()?.validate(&value, args, exit == 0)?;
    Ok(value)
}
fn invoke(env: &Environment, args: &[String]) -> Result<Value, String> {
    call(env, args, 0)
}

fn wire() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let stub = Stub::for_sources(
        &[
            "www.cbioportal.org",
            "reg.genome.network",
            "storage.googleapis.com",
        ],
        |r| {
            if r.contains("hgnc_complete_set") {
                "text/tab-separated-values"
            } else {
                "application/json"
            }
        },
        cbio,
    )?;
    config(&env, &stub)?;
    let first = invoke(&env, &args())?;
    if first.pointer("/result/observed_sample_count") != Some(&json!(1))
        || first.pointer("/result/selected_sample_count") != Some(&json!(100))
        || first.pointer("/result/more") != Some(&json!(true))
        || first.pointer("/result/all_pages_read") != Some(&json!(false))
        || first.pointer("/result/denominator").is_some()
        || first.pointer("/result/terms/unverified") != Some(&json!(true))
    {
        return Err(format!("wrong page counts or rights: {first}"));
    }
    let next: Vec<String> = serde_json::from_value(
        first
            .pointer("/next/0/args")
            .cloned()
            .ok_or("continuation missing")?,
    )
    .map_err(|e| e.to_string())?;
    let second = invoke(&env, &next)?;
    if second.pointer("/result/observed_sample_count") != Some(&json!(1))
        || second.pointer("/result/more") != Some(&json!(false))
        || second.pointer("/result/all_pages_read") != Some(&json!(false))
    {
        return Err(format!(
            "continuation incorrectly became cohort total: {second}"
        ));
    }
    let mut offline = args();
    offline.push("--offline".into());
    invoke(&env, &offline)?;
    let requests = stub.finish()?;
    if requests
        .iter()
        .filter(|(_, r)| r.starts_with("GET /api/"))
        .count()
        != 5
    {
        return Err(format!(
            "warm pages repeated metadata or source reads: {}",
            requests.len()
        ));
    }
    for (_, request) in requests.iter().filter(|(_, r)| r.contains("/mutations?")) {
        if !request.contains("sampleListId=msk_impact_2017_all&entrezGeneId=673&projection=SUMMARY&pageSize=3&pageNumber=") {
            return Err(format!("lost explicit cohort or HGNC bound: {request}"));
        }
    }
    Ok(())
}
fn car(_: &str) -> Result<String, String> {
    Ok(json!(
        {"@id":"http://reg.genome.network/allele/CA123643","genomicAlleles":[{"chromosome":"7","referenceGenome":"GRCh37",
        "hgvs":["NC_000007.13:g.140453136A>T"],"coordinates":[{"start":140453135,"end":140453136,"referenceAllele":"A","allele":"T"}]}]}
    )
    .to_string())
}
fn hgnc(_: &str) -> Result<String, String> {
    Ok("hgnc_id\tsymbol\tname\tlocus_group\tlocus_type\tstatus\tentrez_id\nHGNC:1097\tBRAF\tB-Raf \
        proto-oncogene, serine/threonine kinase\tprotein-coding gene\tgene with protein \
        product\tApproved\t673\n"
        .into())
}
pub(super) fn row(sample: &str, alternate: &str) -> Value {
    json!(
        {"molecularProfileId":"msk_impact_2017_mutations","studyId":"msk_impact_2017","sampleId":sample,"entrezGeneId":673,"chr":"7",
        "startPosition":140453136,"endPosition":140453136,"referenceAllele":"A","variantAllele":alternate,"ncbiBuild":"GRCh37"}
    )
}
pub(super) fn cbio(request: &str) -> Result<String, String> {
    if request.starts_with("GET /allele/") {
        return car(request);
    }
    if request.contains("hgnc_complete_set") {
        return hgnc(request);
    }
    Ok(if request.contains("/sample-lists?") {
        json!([{"studyId":"msk_impact_2017","sampleListId":"msk_impact_2017_all","sampleCount":100}])
    } else if request.starts_with("GET /api/studies/") {
        json!({"studyId":"msk_impact_2017",
"name":"Synthetic cohort",
"description":"Synthetic notice: commercial reuse requires permission",
"citation":"Synthetic citation"})
    } else if request.contains("/mutations?") {
        if request.contains("pageNumber=1") {
            json!([row("s2", "T")])
        } else {
            json!([row("s1", "T"), row("s1", "T"), row("wrong-allele", "G")])
        }
    } else {
        json!({"studyId":"msk_impact_2017","molecularProfileId":"msk_impact_2017_mutations","molecularAlterationType":"MUTATION_EXTENDED"})
    }
    .to_string())
}
