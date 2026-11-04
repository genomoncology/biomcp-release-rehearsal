//! Synthetic native replies exercise provider syntax through the installed executable.
use super::{
    configuration::invoke,
    launch::Environment,
    network_stub::{self, Stub},
};
use crate::runner::transcript::trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    let mut trials = ["pubtator", "semantic-scholar", "litsense"]
        .into_iter()
        .map(|source| {
            trial(
                format!("article-search/native-{source}-gene-query"),
                move || article(source).map_err(Into::into),
            )
        })
        .collect::<Vec<_>>();
    trials.push(trial(
        "drug-native-sources/opentargets-native-mapped-boundaries",
        || opentargets().map_err(Into::into),
    ));
    trials
}
fn environment(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    stub.retain(env.shared_root());
    let cert = env.root().join("ca.pem");
    std::fs::write(&cert, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"ca_bundle":cert,"proxy":format!("http://{}",stub.address)}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn article_request(source: &str) -> &'static str {
    match source {
        "pubtator" => {
            "GET /research/pubtator3-api/search/?text=%40GENE_673%20AND%20synthetic%20words&page=1 HTTP/1.1"
        }
        "litsense" => {
            "GET /research/litsense2-api/api/sentences/?query=BRAF%20synthetic%20words&rerank=true HTTP/1.1"
        }
        _ => {
            concat!(
                "GET /graph/v1/paper/search?query=BRAF%20synthetic%20words&limit=100&fields=",
                "externalIds%2Ctitle%2Cvenue%2Cyear%2CpublicationDate%2Cauthors&year=2000-2020 HTTP/1.1"
            )
        }
    }
}
fn article(source: &'static str) -> Result<(), String> {
    let expected = article_request(source);
    let stub = Stub::for_sources(
        &[
            "storage.googleapis.com",
            "www.ncbi.nlm.nih.gov",
            "api.semanticscholar.org",
        ],
        |request| {
            if request.contains("hgnc/") {
                "text/tab-separated-values"
            } else {
                "application/json"
            }
        },
        move |request| {
            if request.contains("hgnc/") {
                return network_stub::body(request.contains("withdrawn.txt"));
            }
            if request.lines().next() != Some(expected) {
                return Err(format!("unexpected {source} query: {request}"));
            }
            Ok(match source {
                "pubtator" => {
                    r#"{"results":[],"count":0,"current":1,"total_pages":1,"page_size":100}"#
                }
                "litsense" => "[]",
                _ => r#"{"total":0,"data":[]}"#,
            }
            .into())
        },
    )?;
    let env = environment(&stub)?;
    let mut args = vec![
        "search",
        "article",
        "synthetic words",
        "--gene",
        "BRAF",
        "--source",
        source,
    ];
    if source == "semantic-scholar" {
        args.extend(["--since", "2000", "--until", "2020"]);
    }
    let result = invoke(&env, args.into_iter().map(str::to_owned).collect(), 0);
    let requests = stub.finish()?;
    record_requests(&requests)?;
    let value = result?;
    let term = if source == "pubtator" {
        "@GENE_673 AND synthetic words"
    } else {
        "BRAF synthetic words"
    };
    let outcome = value
        .get("sources")
        .and_then(Value::as_array)
        .and_then(|rows| {
            rows.iter()
                .find(|v| v.get("source") == Some(&json!(source)))
        })
        .ok_or("source outcome missing")?;
    if outcome.get("term") != Some(&json!(term))
        || value.pointer("/page/total") != Some(&json!(0))
        || value.get("status") != Some(&json!("complete"))
    {
        return Err(format!(
            "native query changed its public empty reply: {value}"
        ));
    }
    if requests
        .iter()
        .filter(|(_, request)| request.lines().next() == Some(expected))
        .count()
        != 1
    {
        return Err("provider query was not sent exactly once".into());
    }
    Ok(())
}
fn opentargets() -> Result<(), String> {
    let stub = Stub::for_source(
        "api.platform.opentargets.org",
        "application/json",
        |request| {
            if request.lines().next() != Some("POST /api/v4/graphql HTTP/1.1") {
                return Err(format!("unexpected Open Targets request: {request}"));
            }
            let body: Value =
                serde_json::from_str(request.split_once("\r\n\r\n").ok_or("missing body")?.1)
                    .map_err(|e| e.to_string())?;
            if body.get("variables") == Some(&json!({"q":"synthetic words"})) {
                if body.get("query")
                    != Some(&json!(
                        "query DrugSearch($q:String!){search(queryString:$q,entityNames:[\"drug\"],page:{index:0,size:10}){total hits{id name entity}}}"
                    ))
                {
                    return Err(format!("Open Targets search syntax changed: {body}"));
                }
                return Ok(json!({"data":{"search":{"total":1,"hits":[{"id":"CHEMBL1","name":"Synthetic drug","entity":"drug"}]}}}).to_string());
            }
            if body.get("variables") != Some(&json!({"id":"CHEMBL1"}))
                || body.get("query")
                    != Some(&json!(concat!(
                        "query Drug($id:String!){drug(chemblId:$id){id name indications{rows{maxClinicalStage disease{id ",
                        "name}}} mechanismsOfAction{rows{mechanismOfAction actionType targets{id approvedSymbol ",
                        "approvedName}}}}}"
                    )))
            {
                return Err(format!("Open Targets detail syntax changed: {body}"));
            }
            Ok(json!({"data":{"drug":{
                "id":"CHEMBL1","name":"Synthetic drug",
                "indications":{"rows":[{"disease":{"id":"EFO_0001","name":"Synthetic condition"}}]},
                "mechanismsOfAction":{"rows":[{"targets":[{"id":"ENSG00000141510","approvedSymbol":"TP53"}]}]}
            }}}).to_string())
        },
    )?;
    let env = environment(&stub)?;
    let search = invoke(
        &env,
        [
            "search",
            "drug",
            "synthetic words",
            "--source",
            "opentargets",
        ]
        .map(str::to_owned)
        .to_vec(),
        0,
    )?;
    if search.pointer("/result/items/0/id") != Some(&json!("CHEMBL1")) {
        return Err(format!(
            "Open Targets search lost native identity: {search}"
        ));
    }
    for section in [None, Some("targets"), Some("indications")] {
        let mut args = vec!["get", "drug", "CHEMBL1", "--source", "opentargets"];
        if let Some(section) = section {
            args.push(section);
        }
        let value = invoke(&env, args.into_iter().map(str::to_owned).collect(), 0)?;
        if value.get("status") != Some(&json!("complete"))
            || value
                .get("problems")
                .and_then(Value::as_array)
                .is_some_and(|p| !p.is_empty())
        {
            return Err(format!("Open Targets mapped boundary failed: {value}"));
        }
    }
    let requests = stub.finish()?;
    record_requests(&requests)?;
    if requests.len() != 2 {
        return Err("Open Targets search/detail cache reads changed".into());
    }
    Ok(())
}

fn record_requests(requests: &[(String, String)]) -> Result<(), String> {
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "native requests: {}",
        serde_json::to_string(requests).map_err(|e| e.to_string())?
    )
    .map_err(|e| e.to_string())
}
