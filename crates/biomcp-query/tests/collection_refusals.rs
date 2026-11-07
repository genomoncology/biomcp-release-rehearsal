//! Query paging preserves source refusals and the accepted row order.
use biomcp_fetch::{Certificates, Fetch, FetchConfig, Network, Secret};
use biomcp_test_support::network_stub::Stub;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::time::Duration;
type Check = Result<(), Box<dyn std::error::Error>>;

fn trials() -> Vec<Trial> {
    vec![Trial::test(
        "drug-native-sources/collection-refusals-stay-with-accepted-collection",
        || collection_pages().map_err(|error| error.to_string().into()),
    )]
}

fn collection_pages() -> Check {
    for section in ["targets", "indications"] {
        for offset in 0..3 {
            let mut expected = None;
            for refused in [false, true] {
                let result = query_page(section, offset, refused)?;
                if refused {
                    assert_eq!(
                        Some(result),
                        expected,
                        "accepted page changed after refusal"
                    );
                } else {
                    expected = Some(result);
                }
            }
        }
    }
    Ok(())
}

#[expect(
    clippy::disallowed_macros,
    reason = "The synthetic publisher supplies malformed native rows to the public query"
)]
fn synthetic_body(refused: bool) -> String {
    let mut targets = vec![
        json!({"id":"ENSG00000141510"}),
        json!({"id":"ENSG00000146648"}),
    ];
    let mut indications = vec![
        json!({"disease":{"id":"EFO_0001"}}),
        json!({"disease":{"id":"EFO_0002"}}),
    ];
    if refused {
        targets.insert(1, json!({"id":"invalid"}));
        indications.insert(1, json!({"disease":{"id":" "}}));
    }
    json!({"data":{"drug":{"id":"CHEMBL1","name":"synthetic",
        "mechanismsOfAction":{"rows":[{"targets":targets}]},
        "indications":{"rows":indications}}}})
    .to_string()
}

fn query_page(
    section: &str,
    offset: u32,
    refused: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let body = synthetic_body(refused);
    let stub = Stub::for_source(
        "api.platform.opentargets.org",
        "application/json",
        move |_| Ok(body.clone()),
    )?;
    let fetch = Fetch::open(FetchConfig {
        network: Network::new(
            None,
            Some(Secret::new(format!("http://{}", stub.address))),
            "",
            Certificates::from_pem(stub.certificate.as_bytes())?,
        )?,
        ..Default::default()
    })?;
    let query = serde_json::from_str(&format!(
        r#"{{"input":"CHEMBL1","source":"opentargets","section":["{section}"],"section_offset":{offset},"section_limit":1}}"#
    ))?;
    let result = biomcp_query::run(
        &biomcp_query::Ctx {
            fetch: &fetch,
            budget: Duration::from_secs(20),
            fresh: false,
            drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
            ncbi_email: None,
        },
        &biomcp_types::Request::GetDrug(query),
    );
    assert_eq!(
        result.status(),
        if refused {
            biomcp_types::Status::Partial
        } else {
            biomcp_types::Status::Complete
        },
        "query lost collection refusal"
    );
    let value = match result.body {
        biomcp_types::ReplyData::DrugTargets {
            body: Some(body), ..
        } => serde_json::to_value(body)?,
        biomcp_types::ReplyData::DrugIndications {
            body: Some(body), ..
        } => serde_json::to_value(body)?,
        _ => return Err("query lost section body".into()),
    };
    let data = value.pointer(&format!("/{section}/data"));
    let data = data.ok_or("accepted page missing")?;
    assert_eq!(
        data.get("total"),
        Some(&Value::from(2)),
        "accepted total changed"
    );
    assert_eq!(
        data.get("offset"),
        Some(&Value::from(offset)),
        "accepted offset changed"
    );
    assert_eq!(
        data.get("records")
            .and_then(Value::as_array)
            .map_or(0, Vec::len),
        usize::from(offset < 2),
        "accepted rows changed"
    );
    let page = result.page.ok_or("query lost page metadata")?;
    assert_eq!(page.total, 2, "query total changed");
    assert_eq!(
        page.returned,
        u32::from(offset < 2),
        "query received count changed"
    );
    assert_eq!(
        result.next.is_empty(),
        offset > 0,
        "query continuation changed"
    );
    let requests = stub.finish()?;
    assert_eq!(requests.len(), 1, "query did not reach synthetic publisher");
    Ok(data.clone())
}

fn main() {
    libtest_mimic::run(&libtest_mimic::Arguments::from_args(), trials()).exit();
}
