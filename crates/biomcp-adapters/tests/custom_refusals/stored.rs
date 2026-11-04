use super::{Check, Refusal, replay::replay};
use biomcp_adapters::{gtr, maude, pubmed};
use serde_json::{Value, json};
use std::time::Duration;

#[test]
fn stored_gtr_relations_distinguish_declined_identity_and_optional_values() -> Check {
    for (gene_id, symbol, expected) in [("672", "BRCA1", Refusal::None), ("", "", Refusal::Refused)]
    {
        let mut changed = false;
        let (_root, fetch) = replay("diagnostic/gtr-synthetic", |source, record| {
            if source == "gtr"
                && record
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|u| u.ends_with("test_condition_gene.txt"))
                && let Some(value) = record.pointer_mut("/exchanges/0/body/value")
            {
                let header = concat!(
                    "#accession_version\ttest_type\tobject\tGTR_identifier\tMIM_number\t",
                    "object_name\tgene_or_SNOMED_CT_ID\tgene_symbol\n"
                );
                *value = json!(format!(
                    "{header}GTR000006692.3\tUnknown\tgene\t\t\tSynthetic relation\t{gene_id}\t{symbol}\n"
                ));
                changed = true;
            }
        })?;
        assert!(changed, "GTR native relation report fixture missing");
        gtr::sync(&fetch)?;
        let leg = fetch.leg(&gtr::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query: biomcp_types::DiagnosticGet =
            serde_json::from_value(json!({"id":"GTR000006692.3","source":"gtr"}))?;
        let biomcp_types::DiagnosticId::Gtr(id) = query.id() else {
            return Err("expected GTR identity".into());
        };
        let row = gtr::get(&leg, &query, id)?;
        assert_eq!(row.refusals, expected, "GTR lost actual relation refusal");
        assert!(row.card.is_some());
        if expected == Refusal::None {
            assert!(!row.relations.is_empty());
            assert!(
                !row.problems.is_empty(),
                "optional native value control must disclose its problem"
            );
        }
    }
    Ok(())
}

#[test]
fn citation_indexing_refusals_preserve_the_article() -> Check {
    for (heading, expected) in [
        (
            "<MeshHeading><DescriptorName MajorTopicYN=\"N\">Retained</DescriptorName></MeshHeading>",
            Refusal::None,
        ),
        ("<MeshHeading></MeshHeading>", Refusal::Refused),
    ] {
        let prefix = concat!(
            "<PubmedArticleSet><PubmedArticle><MedlineCitation><PMID>28944238</PMID>",
            "<Article><ArticleTitle>Retained title</ArticleTitle></Article><MeshHeadingList>"
        );
        let suffix = "</MeshHeadingList></MedlineCitation></PubmedArticle></PubmedArticleSet>";
        let xml = format!("{prefix}{heading}{suffix}");
        let mut changed = false;
        let (_root, fetch) = replay("article/get/synthetic-derived", |source, record| {
            if source == "pubmed"
                && record
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|u| u.ends_with("efetch.fcgi"))
                && let Some(value) = record.pointer_mut("/exchanges/0/body/value")
            {
                *value = json!(xml);
                changed = true;
            }
        })?;
        assert!(changed, "PubMed native citation fixture missing");
        let leg = fetch.leg(&pubmed::SPEC, fetch.deadline(Duration::from_secs(20)));
        let (rows, _) = pubmed::fetch_articles(&leg, &["28944238".parse()?], None)?;
        let row = rows.first().ok_or("citation absent")?;
        assert_eq!(
            row.refusals, expected,
            "citation lost actual indexing refusal"
        );
        assert_eq!(row.article.pmid.map(|id| id.get()), Some(28944238));
    }
    Ok(())
}

#[test]
fn device_report_refusals_preserve_native_positions_and_optional_values() -> Check {
    for (devices, expected) in [
        (json!(false), Refusal::None),
        (json!([{"brand_name":false}]), Refusal::None),
        (json!([false, {}]), Refusal::Refused),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("adverse-event/adapter-values/maude", |source, record| {
            if source == "maude"
                && record
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|u| u.contains("mdr_report_key"))
                && let Some(value) = record.pointer_mut("/exchanges/0/body/value/results/0/device")
            {
                *value = devices.clone();
                changed = true;
            }
        })?;
        assert!(changed, "MAUDE native report fixture missing");
        let leg = fetch.leg(&maude::SPEC, fetch.deadline(Duration::from_secs(20)));
        let query = serde_json::from_value(json!({"id":"160","source":"maude"}))?;
        let row = maude::get(&leg, &query)?.ok_or("device report absent")?;
        assert_eq!(row.refusals, expected, "MAUDE lost examined device refusal");
        assert_eq!(row.value.data.mdr_report_key.to_string(), "160");
        if expected == Refusal::Refused {
            assert_eq!(
                row.value
                    .positions
                    .devices
                    .as_ref()
                    .ok_or("missing native positions")?
                    .indexes
                    .as_ref(),
                &[1]
            );
        }
    }
    Ok(())
}
