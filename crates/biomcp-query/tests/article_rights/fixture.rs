use super::run;
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use biomcp_test_support::network_stub::Stub;
use biomcp_types::ReplyData;
use std::{path::Path, time::Duration};
fn stub(reply: impl FnMut(&str) -> Result<String, String> + Send + 'static) -> Stub {
    Stub::for_sources(
        &[
            "www.ncbi.nlm.nih.gov",
            "eutils.ncbi.nlm.nih.gov",
            "pmc-oa-opendata.s3.amazonaws.com",
            "www.ebi.ac.uk",
            "api.semanticscholar.org",
        ],
        |request| {
            if request.contains("list-type") || request.contains("efetch") {
                "application/xml"
            } else {
                "application/json"
            }
        },
        reply,
    )
    .unwrap()
}
/// A restricted client on a synthetic network, for replies a recording cannot vary.
pub(super) fn live(
    dir: &Path,
    reply: impl FnMut(&str) -> Result<String, String> + Send + 'static,
) -> (Stub, Fetch) {
    let stub = stub(reply);
    let fetch = Fetch::open(FetchConfig {
        proxy: Some(biomcp_fetch::Secret::new(format!(
            "http://{}",
            stub.address
        ))),
        network: biomcp_fetch::Network::new(
            None,
            None,
            "",
            biomcp_fetch::Certificates::from_pem(stub.certificate.as_bytes()).unwrap(),
        )
        .unwrap(),
        cache: Some(dir.join("live.sqlite")),
        cache_owners: vec![biomcp_query::store::OWNER],
        ..FetchConfig::default()
    })
    .unwrap()
    .restrict_uses([biomcp_types::Use::PerRecord].into())
    .with_hosted_secrets(biomcp_fetch::Secrets::default());
    (stub, fetch)
}
/// More papers than one lookup may check, each permitted when checked.
pub(super) fn many_papers(request: &str, papers: &[u32]) -> String {
    if request.contains("sentences") {
        let rows: Vec<_> = papers
            .iter()
            .map(
                |id| serde_json::json!({"pmid":id,"section":"RESULTS","text":format!("body {id}")}),
            )
            .collect();
        return serde_json::Value::from(rows).to_string();
    }
    let asked: Vec<_> = papers
        .iter()
        .filter(|id| request.contains(&id.to_string()))
        .collect();
    if request.contains("esummary") {
        let mut result = serde_json::json!({"uids": asked
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()});
        for id in &asked {
            result[id.to_string()] = serde_json::json!({
                "uid":id.to_string(),"title":format!("Paper {id}"),
                "articleids":[{"idtype":"pmc","value":format!("PMC{id}")}]
            });
        }
        return serde_json::json!({ "result": result }).to_string();
    }
    let id = asked.first().copied().copied().unwrap_or_default();
    if request.contains("list-type") {
        return format!(
            "<ListBucketResult><IsTruncated>false</IsTruncated><Contents><Key>PMC{id}.1/PMC{id}.1.json</Key></Contents></ListBucketResult>"
        );
    }
    serde_json::json!({
        "pmcid":format!("PMC{id}"), "pmid":id, "license_code":"CC BY",
        "xml_url":format!("https://pmc-oa-opendata.s3.amazonaws.com/PMC{id}.1/PMC{id}.1.xml")
    })
    .to_string()
}
pub(super) fn record(dir: &Path) {
    let stub = stub(response);
    let store = tempfile::tempdir().unwrap();
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Record { dir: dir.into() },
        proxy: Some(biomcp_fetch::Secret::new(format!(
            "http://{}",
            stub.address
        ))),
        network: biomcp_fetch::Network::new(
            None,
            None,
            "",
            biomcp_fetch::Certificates::from_pem(stub.certificate.as_bytes()).unwrap(),
        )
        .unwrap(),
        cache: Some(store.path().join("record.sqlite")),
        cache_owners: vec![biomcp_query::store::OWNER],
        ..FetchConfig::default()
    })
    .unwrap();
    let out = run(&fetch);
    let _ = biomcp_test_support::evidence::retain(dir).unwrap();
    let ReplyData::ArticlePage {
        body: Some(page), ..
    } = out.body
    else {
        panic!("{out:?}")
    };
    assert_eq!(
        page.items.len(),
        3,
        "record source outcomes: {:?}; local: {:?}",
        out.sources,
        out.local
    );
    // Record the license metadata without calling LitSense again.
    let deadline = fetch.command_deadline(Duration::from_secs(20));
    let leg = fetch.leg(&biomcp_adapters::pmc_oa::SPEC, deadline.deadline());
    for id in [1001, 1002, 1003] {
        let _ = biomcp_adapters::pmc_oa::read(&leg, format!("PMC{id}").parse().unwrap(), None);
    }
    let leg = fetch.leg(
        &biomcp_adapters::pubmed::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    biomcp_adapters::pubmed::summaries(
        &leg,
        &["1001".parse().unwrap(), "1002".parse().unwrap()],
        None,
    )
    .unwrap();
    record_other_routes(&fetch);
    let author_query = super::authors::query();
    let id = "123".parse().unwrap();
    let leg = fetch.leg(
        &biomcp_adapters::semantic_scholar::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    biomcp_adapters::semantic_scholar::author::papers(&leg, &id, &author_query).unwrap();
    let _ = biomcp_test_support::evidence::retain(dir).unwrap();
    stub.finish().unwrap();
}
fn record_other_routes(fetch: &Fetch) {
    let leg = fetch.leg(
        &biomcp_adapters::pubmed::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    let ids = ["1002".parse().unwrap()];
    let (articles, _) = biomcp_adapters::pubmed::fetch_articles(&leg, &ids, None).unwrap();
    assert_eq!(
        articles.first().unwrap().article.abstract_parts.len(),
        1,
        "the unrestricted fixture must retain its original abstract"
    );
    biomcp_adapters::pubmed::summaries(&leg, &ids, None).unwrap();
    let leg = fetch.leg(
        &biomcp_adapters::pubtator::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    biomcp_adapters::pubtator::annotations(
        &leg,
        &[
            "1001".parse().unwrap(),
            "1002".parse().unwrap(),
            "1003".parse().unwrap(),
        ],
    )
    .unwrap();
    let leg = fetch.leg(
        &biomcp_adapters::europepmc::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    for id in ["PPR1002", "PPR1003", "PPR1004"] {
        let (article, _) = biomcp_adapters::europepmc::core(&leg, &id.parse().unwrap()).unwrap();
        assert_eq!(
            article.value.abstract_parts.len(),
            1,
            "unrestricted Europe PMC fixtures retain their abstracts"
        );
    }
    let leg = fetch.leg(
        &biomcp_adapters::litsense::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    biomcp_adapters::litsense::search(&leg, "abstract rights witness").unwrap();
    let leg = fetch.leg(
        &biomcp_adapters::semantic_scholar::SPEC,
        fetch.deadline(Duration::from_secs(20)),
    );
    biomcp_adapters::semantic_scholar::edges(
        &leg,
        &biomcp_types::ArticleInput::Pmid("1001".parse().unwrap()),
        false,
        None,
    )
    .unwrap();
}

pub(super) fn response(request: &str) -> Result<String, String> {
    if request.contains("sentences") {
        if request.contains("abstract") {
            return Ok(include_str!("abstract-sentences.json").into());
        }
        return Ok(include_str!("sentences.json").into());
    }
    if request.contains("esummary") {
        return Ok(include_str!("summaries.json").into());
    }
    if request.contains("efetch") {
        return Ok(include_str!("abstract.xml").into());
    }
    if request.contains("biocjson") {
        return Ok(include_str!("pubtator.json").into());
    }
    if request.contains("PPR") {
        let (id, license) = if request.contains("PPR1002") {
            ("PPR1002", "CC BY NC")
        } else if request.contains("PPR1004") {
            ("PPR1004", "TDM")
        } else {
            ("PPR1003", "unrecognized")
        };
        return Ok(serde_json::json!({"result": {
            "id": id, "source": "PPR", "title": "Preprint identity",
            "license": license, "abstractText": format!("{id} abstract witness")
        }})
        .to_string());
    }
    if request.contains("/author/123/papers") {
        return Ok(author_papers());
    }
    if request.contains("/citations") {
        return Ok(contexts());
    }
    let id = if request.contains("PMC1001") {
        1001
    } else if request.contains("PMC1002") {
        1002
    } else {
        1003
    };
    if request.contains("list-type") {
        return Ok(format!(
            "<ListBucketResult><IsTruncated>false</IsTruncated><Contents><Key>PMC{id}.1/PMC{id}.1.json</Key></Contents></ListBucketResult>"
        ));
    }
    let license = match id {
        1001 => "CC BY",
        1002 => "CC BY NC",
        _ => "unrecognized",
    };
    Ok(serde_json::json!({
        "pmcid":format!("PMC{id}"), "pmid":id, "license_code":license,
        "xml_url":format!("https://pmc-oa-opendata.s3.amazonaws.com/PMC{id}.1/PMC{id}.1.xml")
    })
    .to_string())
}

fn contexts() -> String {
    let data: Vec<_> = [
        ("1001", "a", "permitted citation context"),
        ("1002", "b", "noncommercial citation context"),
        ("1003", "c", "unknown citation context"),
    ].into_iter().map(|(id, key, text)| serde_json::json!({
        "citingPaper": {"paperId":key.repeat(40),"externalIds":{"PubMed":id},"title":format!("Paper {id}")},
        "contexts":[text],"intents":["background"],"isInfluential":false
    })).collect();
    let mut data = data;
    data[2]["citingPaper"]["externalIds"] = serde_json::json!({"DOI":"10.1234/unknown"});
    // Native replies cannot borrow an identity from a cached author-detail shape.
    data[2]["citingPaper"]["header"] = serde_json::json!({"pmid":1001});
    data[2]["citingPaper"]["ignored"] = serde_json::json!({
        "contexts":{"items":["unknown malformed citation context"]}, "license":"cc_by"
    });
    serde_json::json!({"data":data}).to_string()
}

fn author_papers() -> String {
    let data: Vec<_> = [("1001","a"),("1002","b"),("1003","c")].into_iter().map(|(id,key)| serde_json::json!({
        "paperId":key.repeat(40),"externalIds":{"PubMed":id},"title":format!("Author paper {id}"),
        "abstract":format!("author abstract {id}")
    })).collect();
    serde_json::json!({"offset":0,"total":3,"data":data}).to_string()
}
