//! Feature protein-sections: native coordinates, alternative identities and bounded disclosure.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::{fs, path::Path};
mod boundaries;

pub(super) fn trials() -> Vec<Trial> {
    // The peer checks exact source coordinates, bound page identities and request counts after section continuations.
    vec![transcript_trial(
        crate::runner::pool_contract::FEATURES.0,
        || boundaries::features().map_err(Into::into),
    )]
}
fn setup(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("source-ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn entry() -> Value {
    json!(
        {"primaryAccession":"P00533","uniProtkbId":"SYNTHETIC_HUMAN","entryType":"UniProtKB reviewed (Swiss-Prot)",
        "organism":{"taxonId":9606},"proteinDescription":{"recommendedName":{"fullName":{"value":"Synthetic protein"}}},
        "sequence":{"length":30,"value":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},"features":[ {"type":"Domain",
        "description":"Native domain","location":{"start":{"value":10,"modifier":"EXACT"},"end":{"value":20,"modifier":"EXACT"}},
        "evidences":[{"evidenceCode":"ECO:0000269","source":"PubMed","id":"1"}]},{"type":"Region","description":"Uncertain edges",
        "location":{"start":{"value":5,"modifier":"OUTSIDE"},"end":{"value":25,"modifier":"OUTSIDE"}}},{"type":"Domain",
        "description":"Alternative sequence only","location":{"sequence":"P00533-2","start":{"value":10,"modifier":"EXACT"},
        "end":{"value":20,"modifier":"EXACT"}}},{"type":"Region","location":{"start":{"modifier":"UNKNOWN"},"end":{"value":30,
        "modifier":"EXACT"}}}],"comments":[{"commentType":"FUNCTION","texts":[{"value":"Whole synthetic paragraph."}]},
        {"commentType":"FUNCTION","molecule":"Isoform 2","texts":[{"value":"Separate qualified paragraph."}]},
        {"commentType":"ALTERNATIVE PRODUCTS","isoforms":[{"name":{"value":"External"},"isoformIds":["Q8N726"],
        "isoformSequenceStatus":"External"}]}],"uniProtKBCrossReferences":[{"database":"PDB","id":"1ABC"}]}
    )
}
fn call(
    env: &Environment,
    replay: Option<&Path>,
    args: &[&str],
    exit: i32,
) -> Result<Value, String> {
    let mut words = env.globals(replay, false);
    words.extend(args.iter().map(|s| (*s).to_owned()));
    invoke(env, words, exit).map_err(|e| format!("{args:?}: {e}"))
}

tool_only! {}
