//! Feature gwas-get: exact records and retained independent context pages.
use super::required_contract;
use super::{configuration::invoke, gwas_budgets, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

mod synthetic;
pub(super) fn trials() -> Vec<Trial> {
    // The test removes replay input before ancestry continuations and counts real loci/study peer requests.
    vec![
        transcript_trial(required_contract::GWAS_ANCESTRY.0, || {
            synthetic::ancestry().map_err(Into::into)
        }),
        transcript_trial(required_contract::GWAS_ASSOCIATIONS.0, || {
            synthetic::contexts().map_err(Into::into)
        }),
    ]
}
const BASE: &str = "https://www.ebi.ac.uk/gwas/rest/api/v2";
fn card() -> Value {
    json!({"association_id":6114,"accession_id":"GCST000035","reported_trait":["Synthetic trait"],
        "efo_traits":[{"efo_id":"MONDO_0004989","efo_trait":"Synthetic mapped trait"}],
        "snp_allele":[{"rs_id":"rs3817198","effect_allele":"C"}],"p_value":3e-9,
        "pvalue_mantissa":3,"pvalue_exponent":-9,"or_per_copy_num":1.07,"pubmed_id":"17529967"})
}
fn study() -> Value {
    json!({"accession_id":"GCST000035","pubmed_id":17529967,"disease_trait":"Synthetic trait",
        "efo_traits":[],"discovery_ancestry":[],"replication_ancestry":[],"genotyping_technologies":[],
        "initial_sample_size":"Native prose repeated repeated","replication_sample_size":"Native scope differs"})
}
fn ancestry_body() -> Value {
    json!({"_embedded":{"ancestries":(0..3).map(|n|json!({"type":"replication","number_of_individuals":10+n,
        "ancestral_groups":[{"ancestral_group":"Synthetic group"}],"country_of_origin":[{"country_name":"NR"}],
        "country_of_recruitment":[],"_links":{"self":{"href":format!("{BASE}/studies/GCST000035/ancestries/{n}")}}})).collect::<Vec<_>>()}})
}
fn source(request: &str) -> Result<String, String> {
    let path = request
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .ok_or("target")?;
    Ok(if request.contains("/ancestries") {
        ancestry_body()
    } else if request.contains("/loci") {
        loci()
    } else if request.contains("/associations?") {
        associations(path)
    } else if request.contains("/studies/") {
        study()
    } else {
        card()
    }
    .to_string())
}
fn loci() -> Value {
    json!({"_embedded":{"loci":[{"description":"Synthetic locus","strongest_risk_alleles":[{"risk_allele_name":"rs3817198-C"}],
        "author_reported_genes":[{"gene_name":"Synthetic gene","ensembl_gene_ids":["ENSG00000066468"],"entrez_gene_ids":["2263"]}],
        "_links":{"self":{"href":format!("{BASE}/associations/6114/loci/1")}}}]}})
}
fn associations(path: &str) -> Value {
    let page = if path.contains("page=1") { 1 } else { 0 };
    let rows = (page * 3..page * 3 + 3)
        .map(|n| {
            let mut row = card();
            if let Some(id) = row.get_mut("association_id") {
                *id = json!(6110 + n);
            }
            row
        })
        .collect::<Vec<_>>();
    json!({"_embedded":{"associations":rows},"page":{"size":3,"number":page,"totalElements":6,"totalPages":2},
        "_links":{"self":{"href":format!("https://www.ebi.ac.uk{path}")}}})
}

fn call(
    phase: &str,
    env: &Environment,
    replay: &Path,
    args: &[&str],
    exit: i32,
) -> Result<Value, String> {
    let mut command = env.globals(Some(replay), false);
    command.extend(args.iter().map(|s| (*s).to_owned()));
    gwas_budgets::invoke(phase, env, command, exit)
}
fn seed(
    env: &Environment,
    args: &[&str],
    body: String,
    status: u16,
    delay: u64,
) -> Result<PathBuf, String> {
    let stub = Stub::for_source("www.ebi.ac.uk", "application/json", source)?;
    let record_env = super::variant::source_environment(&stub)?;
    let original = record_env.root().join("recorded");
    let mut command = args.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    command.extend(["--record".into(), original.to_string_lossy().into()]);
    invoke(&record_env, command, 0)?;
    if stub.finish()?.len() != 1 {
        return Err("synthetic seed request count".into());
    }
    let directory = env.root().join("replay/gwas");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(original.join("gwas")).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let mut recording: Value =
            serde_json::from_slice(&fs::read(entry.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let acquired = recording
            .pointer("/exchanges/0/fetched")
            .and_then(Value::as_i64)
            .ok_or("recording acquisition")?;
        let exchange = json!({"status":status,"content_type":"application/json","delay_ms":delay,"retry_after_ms":null,"body":{"kind":"text","value":body}});
        *recording.get_mut("exchanges").ok_or("exchanges")? =
            json!([exchange.clone(), exchange.clone(), exchange]);
        super::response_cases::cache_acquisition(&mut recording, acquired);
        fs::write(directory.join(entry.file_name()), recording.to_string())
            .map_err(|e| e.to_string())?;
    }
    Ok(env.root().join("replay"))
}
