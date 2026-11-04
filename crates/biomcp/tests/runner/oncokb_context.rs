//! Resolve keyed source selectors while keyless calls stop before identity I/O.
use super::{launch::Environment, network_stub::Stub, oncokb};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, io::Write};
const SECTION: &str = "/result/candidates/0/sections/oncokb";
pub(super) fn trials() -> Vec<Trial> {
    // The peer binds request context and verifies that independent tool calls do not inherit earlier annotation context.
    vec![transcript_trial(
        crate::runner::required_contract::ONCOKB_ALL.0,
        || run().map_err(Into::into),
    )]
}
fn run() -> Result<(), String> {
    let (stub, observed) = peer()?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    let disabled = unrelated_sources();
    let config = json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,"disable_sources":disabled});
    env.config(&config).map_err(|e| e.to_string())?;
    for id in ["rs113488022", "CA123643"] {
        let args = args(id)?;
        let skipped = oncokb::invoke(&env, &args, 0)?;
        if skipped.pointer(&format!("{SECTION}/reason")) != Some(&json!("no-key")) {
            return Err("ID did not stop before resolution without a key".into());
        }
        let mcp = super::mcp::call_with_globals(&env, &args, false, &[])?;
        if skipped.get("result") != mcp.get("result") {
            return Err("keyless ID CLI/MCP differs".into());
        }
    }
    let mut keyed = config;
    keyed.as_object_mut().ok_or("config")?.insert(
        "keys".into(),
        json!({"oncokb_api_key":"synthetic-oncokb-variant-key"}),
    );
    env.config(&keyed).map_err(|e| e.to_string())?;
    for id in ["rs113488022", "CA123643"] {
        resolved(&env, id)?;
    }
    let all = ["get", "variant", "rs113488022", "all"].map(str::to_owned);
    let missing = oncokb::invoke(&env, &all, 0)?;
    if missing.pointer(&format!("{SECTION}/state")) != Some(&json!("skipped"))
        || !missing
            .get("next")
            .and_then(Value::as_array)
            .is_some_and(|steps| {
                steps.iter().any(|s| {
                    s.get("kind") == Some(&json!("configure"))
                        && s.get("args") == Some(&json!(["help", "get", "variant"]))
                })
            })
    {
        return Err(format!(
            "all omitted the explicit context help step: {missing}"
        ));
    }
    empty_components(&missing)?;
    let cold: Vec<_> = observed.try_iter().collect();
    let mcp = super::mcp::call_with_globals(&env, &all, false, &[])?;
    let warm: Vec<_> = observed.try_iter().collect();
    if missing.get("result") != mcp.get("result") || missing.get("next") != mcp.get("next") {
        return Err("all context CLI/MCP differs".into());
    }
    oncokb::invoke(&env, &["help", "get", "variant"].map(str::to_owned), 0)?;
    let requests = stub.finish()?;
    retain_requests(&requests, cold, warm)?;
    let added = component_requests(&requests)?;
    if requests
        .iter()
        .filter(|(_, r)| r.starts_with("GET /api/v1/annotate/"))
        .count()
        != 1
        // Shared identity projection reuses the rsID lookup for the later all request.
        || requests.iter().filter(|(_, r)| r.starts_with("GET /v1/query?")).count() != 1
        || requests.iter().filter(|(_, r)| r.starts_with("GET /allele/CA123643?")).count() != 1
        || requests.len() - added != 3
    {
        return Err(format!(
            "keyless or contextless request escaped, or warm cache missed: {}",
            requests.len()
        ));
    }
    Ok(())
}
fn peer() -> Result<(Stub, std::sync::mpsc::Receiver<String>), String> {
    let (observed, observation) = std::sync::mpsc::channel();
    let stub = Stub::for_sources(
        &["myvariant.info", "www.oncokb.org", "reg.genome.network"],
        |_| "application/json",
        move |request| {
            observed
                .send(request.to_owned())
                .map_err(|e| e.to_string())?;
            reply(request)
        },
    )?;
    Ok((stub, observation))
}
fn retain_requests(
    requests: &[(String, String)],
    cold: Vec<String>,
    warm: Vec<String>,
) -> Result<(), String> {
    writeln!(
        std::io::stdout(),
        "WARM_CACHE_REQUEST_PHASES {}",
        json!({"cold_setup":cold,"warm":warm})
    )
    .map_err(|e| e.to_string())?;
    if let Some(context) = super::transcript::current() {
        context.record("cache/variant-oncokb/all/requests", json!(requests))?;
    }
    if !warm.is_empty() {
        return Err(format!("warm call made {} extra live requests", warm.len()));
    }
    Ok(())
}
fn args(id: &str) -> Result<Vec<String>, String> {
    let mut args = oncokb::args();
    *args.get_mut(2).ok_or("input")? = id.into();
    Ok(args)
}
fn resolved(env: &Environment, id: &str) -> Result<(), String> {
    let args = args(id)?;
    let resolved = oncokb::invoke(env, &args, 0)?;
    if resolved.pointer(&format!("{SECTION}/state")) != Some(&json!("data")) {
        return Err(format!(
            "keyed ID was not resolved for annotation: {resolved}"
        ));
    }
    let mcp = super::mcp::call_with_globals(env, &args, false, &[])?;
    if resolved.get("result") != mcp.get("result") {
        return Err("resolved annotation CLI/MCP differs".into());
    }
    Ok(())
}
fn reply(request: &str) -> Result<String, String> {
    if request.starts_with("GET /allele/CA123643") {
        return Ok(json!(
            {"@id":"http://reg.genome.network/allele/CA123643","genomicAlleles":[{"referenceGenome":"GRCh38",
            "hgvs":["NC_000007.14:g.140753336A>T"],"coordinates":[{"start":140753335,"end":140753336,"referenceAllele":"A","allele":"T"}]}]}
        )
        .to_string());
    }
    if request.starts_with("GET /v1/query?") {
        return Ok(json!({"total":1,"hits":[{"_id":"chr7:g.140753336A>T",
        "chrom":"7","hg38":{"start":140753336,"end":140753336},
        "dbsnp":{"rsid":"rs113488022","gene":{"symbol":"BRAF"}}}]})
        .to_string());
    }
    oncokb::reply(request)
}

fn unrelated_sources() -> Vec<String> {
    use biomcp_types::Source;
    let allowed = [Source::MyVariant, Source::AlleleRegistry, Source::Oncokb];
    biomcp_query::sources::sources()
        .filter(|spec| !allowed.contains(&spec.source))
        .map(|spec| spec.source.to_string())
        .collect()
}

// The synthetic parent deliberately lacks all three added component payloads.
fn empty_components(value: &Value) -> Result<(), String> {
    for name in ["predict", "conservation", "cosmic"] {
        if value.pointer(&format!("/result/candidates/0/sections/{name}/state"))
            != Some(&json!("empty"))
        {
            return Err(format!("all did not preserve empty {name}: {value}"));
        }
    }
    Ok(())
}
fn component_requests(requests: &[(String, String)]) -> Result<usize, String> {
    for fields in [
        "fields=_id%2Ccadd.phred%2Ccadd.rawscore%2Ccadd.sift%2Cdbnsfp.hgvsp%2Cdbnsfp.ensembl.transcriptid%2Cdbnsfp.polyphen2\
                    %2Cdbnsfp.revel%2Cdbnsfp.mutationtaster",
        "fields=_id%2Cdbnsfp.hgvsp%2Cdbnsfp.ensembl.transcriptid%2Cdbnsfp.gerp%2B%2B%2Cdbnsfp.phastcons%2Cdbnsfp.phylop%2Cdbnsfp.siphy_29way",
        "fields=_id%2Ccosmic",
    ] {
        let count = requests
            .iter()
            .filter(|(_, request)| {
                request.starts_with("GET /v1/variant/chr7%3Ag.140753336A%3ET?")
                    && request.contains(&format!("?{fields}&assembly=hg38 "))
            })
            .count();
        if count != 1 {
            return Err(format!(
                "expected one added component request for {fields}: {requests:?}"
            ));
        }
    }
    Ok(3)
}
