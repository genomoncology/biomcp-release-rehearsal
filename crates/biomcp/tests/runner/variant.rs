//! Exact source request spelling crosses the real command and TLS transport.
use super::{launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
mod build_echo;
mod search_corroboration;
mod search_policy;
mod section_deadline;

pub(super) fn trials() -> Vec<Trial> {
    // The real peer checks resolver order, source requests and warm cache admission.
    vec![
        transcript_trial("variant-search/fallback-reference-confirmation", || {
            search_corroboration::check().map_err(Into::into)
        }),
        transcript_trial("variant-search/enabled-protein-selector", || {
            search_policy::enabled().map_err(Into::into)
        }),
        transcript_trial("variant-search/open-protein-selector", || {
            search_policy::check().map_err(Into::into)
        }),
        transcript_trial("variant-identity/sections-share-request-deadline", || {
            section_deadline::check().map_err(|error| error.to_string().into())
        }),
        transcript_trial("variant-identity/car-projection-wire", || {
            car().map_err(Into::into)
        }),
        transcript_trial("variant-identity/myvariant-exact-build-wire", || {
            build_echo::check().map_err(Into::into)
        }),
        transcript_trial(
            crate::runner::required_contract::VALIDATOR_RESOLVER.0,
            || validator_order().map_err(Into::into),
        ),
    ]
}

fn call(
    host: &'static str,
    body: fn(&str) -> Result<String, String>,
    args: &[&str],
) -> Result<(Value, Requests), String> {
    let stub = Stub::for_source(host, "application/json", body)?;
    let environment = source_environment(&stub)?;
    let args = args.iter().map(|s| (*s).into()).collect::<Vec<_>>();
    let output = environment
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    let value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    Ok((value, stub.finish()?))
}

pub(super) fn source_environment(stub: &Stub) -> Result<Environment, String> {
    let environment = Environment::new().map_err(|e| e.to_string())?;
    let ca = environment.root().join("synthetic-ca.pem");
    stub.retain(environment.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    environment
        .config(&json!({"proxy":format!("http://{}", stub.address), "ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(environment)
}

fn validator_order() -> Result<(), String> {
    let (value, requests) = call(
        "rest.variantvalidator.org",
        validator_body,
        &[
            "get",
            "variant",
            "NM_004333.4:c.1799T>A",
            "--resolver",
            "variantvalidator",
            "--build",
            "GRCh38",
        ],
    )?;
    if requests.len() != 1
        || value.pointer("/result/candidates/0/record/id") != Some(&json!("NM_004333.4:c.2A>T"))
        || value.pointer("/result/candidates/1/record/id") != Some(&json!("NM_004333.4:c.1A>T"))
        || value.pointer("/result/candidates/0/identity/representations/0/expression/value")
            != Some(&json!("NM_004333.4:c.2A>T"))
    {
        return Err(format!(
            "validator changed object order or lost transcript-only source forms: {value}"
        ));
    }
    Ok(())
}
fn validator_body(_: &str) -> Result<String, String> {
    Ok(
        "{\"flag\":\"gene_variant\",\"metadata\":{\"variantvalidator_version\":\"synthetic\"},\
                    \"z\":{\"submitted_variant\":\"NM_004333.4:c.1799T>A\",\"hgvs_transcript_variant\":\"NM_004333.4:c.2A>T\"},\"a\":{\
        \"submitted_variant\":\"NM_004333.4:c.1799T>A\",\"hgvs_transcript_variant\":\"NM_004333.4:c.1A>T\"}}"
            .into(),
    )
}

fn car() -> Result<(), String> {
    let (output, requests) = call(
        "reg.genome.network",
        car_body,
        &[
            "get",
            "variant",
            "NM_004333.4:c.1799T>A",
            "--disable-sources",
            "myvariant",
        ],
    )?;
    let [(_, request)] = requests.as_slice() else {
        return Err("expected one CAR request".into());
    };
    if !request.starts_with(
        "GET \
        /allele?hgvs=NM_004333.4%3Ac.1799T%3EA&fields=none%2B%40id%2BcommunityStandardTitle%2BgenomicAlleles\
        %2BtranscriptAlleles.MANE%2BexternalRecords.dbSNP%2BexternalRecords.ClinVarVariations HTTP/1.1",
    ) {
        return Err(format!("CAR projection must preserve literal plus modifiers: {request}"));
    }
    if output.pointer("/result/candidates/0/identity/ids/0") != Some(&json!("CA123643")) {
        return Err(format!("source identity lost: {output}"));
    }
    Ok(())
}
type Requests = Vec<(String, String)>;

fn car_body(_: &str) -> Result<String, String> {
    Ok(json!(
        {"@id":"http://reg.genome.network/allele/CA123643","genomicAlleles":[{"chromosome":"7","referenceGenome":"GRCh38",
        "referenceSequence":"http://reg.genome.network/refseq/RS000055","hgvs":["NC_000007.14:g.140753336A>T"],
        "coordinates":[{"start":140753335,"end":140753336,"referenceAllele":"A","allele":"T"}]}]}
    )
    .to_string())
}

fn myvariant_body(_: &str) -> Result<String, String> {
    Ok(json!({"_id":"chr7:g.140753336A>T",
        "chrom":"7","hg38":{"start":140753336,"end":140753336},
        "dbsnp":{"rsid":"rs113488022","gene":{"symbol":"BRAF"}},"clinvar":{"variant_id":13961}})
    .to_string())
}
