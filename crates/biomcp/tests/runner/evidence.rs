//! Stored cursor and hit positions through real command record/replay boundaries.
#![expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Checked test envelopes fail at the malformed boundary"
)]
use super::response_cases::checked_steps;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::json;
pub(super) fn trials() -> Vec<Trial> {
    // The test reconstructs runtime fragments and compares independent citation/reference identities across pages.
    vec![transcript_trial(
        crate::runner::pool_contract::EVIDENCE_FRAGMENTS.0,
        || overflow().map_err(|e| e.to_string().into()),
    )]
}

fn overflow() -> Result<(), Box<dyn std::error::Error>> {
    checked_steps(
        "article/evidence/source",
        |source, record| {
            if source == "pmc-oa"
                && record["url"]
                    .as_str()
                    .is_some_and(|url| url.contains(".xml?"))
            {
                let body = record["exchanges"][0]["body"]["value"].as_str().unwrap();
                let start = body.find("<body>").unwrap() + 6;
                let end = body.find("</body>").unwrap();
                let replacement = format!(
                    "<sec><title>Evidence</title><p><xref ref-type=\"bibr\" rid=\"B17\">exact</xref>a{}b</p></sec>",
                    " ".repeat(7000)
                );
                record["exchanges"][0]["body"]["value"] =
                    json!(format!("{}{}{}", &body[..start], replacement, &body[end..]));
            }
        },
        json!([{"run":["article","citation-evidence","33123537","17611497","--fulltext"],"expect":{
            "/result/fulltext/data/paragraphs/total":3,
            "/result/fulltext/data/paragraphs/items/1/character_offset":3500,
            "/result/fulltext/data/paragraphs/items/1/markdown":" ".repeat(3500)
        }}]),
    )?;
    checked_steps(
        "article/evidence/source",
        |source, record| {
            if source == "semantic-scholar" {
                let rows = record["exchanges"][0]["body"]["value"]["data"]
                    .as_array_mut()
                    .unwrap();
                for row in rows {
                    if row.pointer("/citedPaper/externalIds/PubMed") == Some(&json!("17611497")) {
                        row["contexts"] = json!(["a".repeat(10000), "last context"]);
                    }
                }
            }
        },
        json!(
            [ {"run":["article","citation-evidence","33123537","17611497"],"expect":{"/result/semantic_scholar/data/contexts/total":4,
            "/result/semantic_scholar/data/contexts/items/0/character_offset":0}},{"follow":"page",
            "expect":{"/result/semantic_scholar/data/contexts/items/0/character_offset":3500}},{"follow":"page",
            "expect":{"/result/semantic_scholar/data/contexts/items/0/character_offset":7000,
            "/result/semantic_scholar/data/contexts/items/1/text":"last context"}} ]
        ),
    )?;
    checked_steps(
        "article/evidence/source",
        |source, record| {
            if source == "pmc-oa"
                && record["url"]
                    .as_str()
                    .is_some_and(|url| url.contains(".xml?"))
            {
                let body = record["exchanges"][0]["body"]["value"].as_str().unwrap();
                let start = body.find("<body>").unwrap() + 6;
                let end = body.find("</body>").unwrap();
                let replacement = format!(
                    "<sec><title>Evidence</title><list><list-item><p><xref ref-type=\"bibr\" rid=\"B17 \
                        B99\">exact</xref>{}</p></list-item></list><p><xref ref-type=\"bibr\" \
                        rid=\"B170\">decoy</xref></p></sec>",
                    "z".repeat(10000)
                );
                record["exchanges"][0]["body"]["value"] =
                    json!(format!("{}{}{}", &body[..start], replacement, &body[end..]));
            }
        },
        json!(
            [ {"run":["article","citation-evidence","33123537","17611497","--fulltext"],"expect":{"/result/fulltext/data/paragraphs/total":3,
            "/result/fulltext/data/paragraphs/items/0/position":1,"/result/fulltext/data/paragraphs/items/0/part":1}},{"follow":"page",
            "expect":{"/result/fulltext/data/paragraphs/items/0/position":1,"/result/fulltext/data/paragraphs/items/0/character_offset":7000}} ]
        ),
    )?;
    Ok(())
}
