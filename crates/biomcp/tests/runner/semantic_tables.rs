//! Boundary tables run the same command runner with response-only fixture edits.
use super::response_cases::{case, checked_case};
use serde_json::{Value, json};
type Check = Result<(), Box<dyn std::error::Error>>;
const CAPTURE: &str = "article/edges/live";
const ARGS: &[&str] = &[
    "article",
    "references",
    "33123537",
    "--source",
    "semantic-scholar",
];
pub(super) fn check() -> Check {
    fields()?;
    anonymous()?;
    titleless_dois()?;
    failures()
}
fn fields() -> Check {
    for (contexts, flag, expected) in [
        (
            None,
            None,
            json!({"/result/items/0/key/namespace":"semantic-scholar"}),
        ),
        (
            Some(json!([])),
            Some(false),
            json!({"/result/items/0/contexts/total":0,"/result/items/0/influential":false}),
        ),
        (
            Some(json!([
                "First invented context.",
                "Second invented context."
            ])),
            Some(true),
            json!({"/result/items/0/contexts/total":2,"/result/items/0/contexts/items":["First invented context."],"/result/items/0/influential":true}),
        ),
    ] {
        let value = checked_case(
            CAPTURE,
            ARGS,
            |source, record| {
                if source != "semantic-scholar" {
                    return;
                }
                let mut edge = json!(
                    {"citedPaper":{"paperId":"0123456789abcdef0123456789abcdef01234567","externalIds":{},"title":"Invented article title.","year":2020}}
                );
                if let Some(value) = &contexts
                    && let Some(edge) = edge.as_object_mut()
                {
                    edge.insert("contexts".into(), value.clone());
                }
                if let Some(value) = flag
                    && let Some(edge) = edge.as_object_mut()
                {
                    edge.insert("isInfluential".into(), value.into());
                }
                reply(record, json!({"data":[edge]}));
            },
            expected,
            0,
        )?;
        if contexts.is_none() && value.pointer("/result/items/0/contexts").is_some() {
            return Err("missing contexts were invented".into());
        }
        if flag.is_none() && value.pointer("/result/items/0/influential").is_some() {
            return Err("missing influential flag was invented".into());
        }
    }
    Ok(())
}
fn failures() -> Check {
    case(
        CAPTURE,
        ARGS,
        |source, record| {
            if source == "semantic-scholar" {
                reply(
                    record,
                    json!({"data":[{"citedPaper":{"paperId":"bad","externalIds":{}}}]}),
                );
            }
        },
        json!({
            "/status": "partial",
            "/sources/0/state": "ok",
            "/result": {"equals": {}},
            "/page/returned": 0,
            "/page/total": 0,
            "/problems": [{
                "source": "semantic-scholar",
                "field": "paper_id",
                "code": "value.unknown"
            }]
        }),
        0,
    )?;
    for (value, reason) in [
        (json!({"data":[],"next":0}), "bad-reply"),
        (
            json!({"data":[{"citingPaper":{"paperId":"0123456789abcdef0123456789abcdef01234567","externalIds":{}}}]}),
            "bad-reply",
        ),
        (json!({"data":null}), "unavailable"),
    ] {
        case(
            CAPTURE,
            ARGS,
            |source, record| {
                if source == "semantic-scholar" {
                    reply(record, value.clone());
                }
            },
            json!({"/status":"failed","/sources/0/reason":reason}),
            4,
        )?;
    }
    case(
        CAPTURE,
        ARGS,
        |source, record| {
            if source == "semantic-scholar" {
                reply(record, json!({"data":[]}));
            }
        },
        json!({"/status":"complete","/page/total":0,"/page/truncated":false,}),
        0,
    )
}

fn anonymous() -> Check {
    let value = checked_case(
        CAPTURE,
        ARGS,
        |source, record| {
            if source != "semantic-scholar" {
                return;
            }
            let rows = [
                "First anonymous source row.",
                "Second anonymous source row.",
            ]
            .map(|title| {
                json!({
                    "citedPaper":{"paperId":null,"externalIds":null,"title":title,"year":2009},
                    "intents":["background"],"isInfluential":false,"contexts":["Invented context."]
                })
            });
            reply(record, json!({"data":rows}));
        },
        json!({"/status":"complete","/page/total":2,"/result/items/*/header/title":["First anonymous source row.","Second anonymous source row."]}),
        0,
    )?;
    if value
        .get("next")
        .and_then(|v| v.as_array())
        .is_some_and(|steps| !steps.is_empty())
    {
        return Err("anonymous rows gained an external step".into());
    }
    for i in 0..2 {
        if value.pointer(&format!("/result/items/{i}/key")).is_some() {
            return Err("anonymous row gained an identity".into());
        }
    }
    Ok(())
}

fn reply(record: &mut Value, value: Value) {
    if let Some(body) = record.pointer_mut("/exchanges/0/body/value") {
        *body = value;
    }
}

// An omitted title must not prevent identity-based PubMed resolution.
fn titleless_dois() -> Check {
    case(
        CAPTURE,
        &[
            "article",
            "references",
            "34096690",
            "--source",
            "semantic-scholar",
        ],
        |source, record| {
            if source != "semantic-scholar" {
                return;
            }
            if let Some(rows) = record
                .pointer_mut("/exchanges/0/body/value/data")
                .and_then(Value::as_array_mut)
            {
                for edge in rows {
                    if let Some(paper) = edge.get_mut("citedPaper")
                        && paper.pointer("/externalIds/DOI").is_some()
                        && paper.pointer("/externalIds/PubMed").is_none()
                        && let Some(fields) = paper.as_object_mut()
                    {
                        fields.insert("title".into(), Value::Null);
                    }
                }
            }
        },
        json!({"/result/items/2/pmid":"32955176", "/result/items/2/contexts/total":0}),
        0,
    )
}
