use super::{Check, Refusal, replay::replay};
use biomcp_adapters::{cellosaurus, complex_portal, europepmc, litvar};
use serde_json::{Value, json};
use std::time::Duration;

#[test]
fn complex_xref_refusal_preserves_participant_and_snapshot() -> Check {
    for (xrefs, expected) in [
        (json!(false), Refusal::None),
        (
            json!([{"database":"GO","identifier":"0000001","name":false}]),
            Refusal::None,
        ),
        (json!([false]), Refusal::Refused),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("protein/examples", |source, record| {
            if source == "complex-portal"
                && let Some(value) =
                    record.pointer_mut("/exchanges/0/body/value/elements/0/interactors/0/xrefs")
            {
                *value = xrefs.clone();
                changed = true;
            }
        })?;
        assert!(changed, "Complex Portal native participant fixture missing");
        let leg = fetch.leg(
            &complex_portal::SPEC,
            fetch.deadline(Duration::from_secs(20)),
        );
        let page = complex_portal::search(&leg, &"P00533".parse()?)?;
        assert_eq!(
            page.refusals, expected,
            "complex collection lost xref refusal"
        );
        let complex = page.items.first().ok_or("missing anchored complex")?;
        assert_eq!(complex.refusals, expected);
        assert_eq!(complex.members.len(), 1);
        let restored: complex_portal::Collection =
            serde_json::from_value(serde_json::to_value(&page)?)?;
        assert_eq!(restored.refusals, expected);
        assert_eq!(
            restored
                .items
                .first()
                .ok_or("missing snapshot complex")?
                .refusals,
            expected
        );
    }
    Ok(())
}

#[test]
fn cell_line_variation_refusal_preserves_retained_parent() -> Check {
    for (xrefs, expected) in [
        (json!(false), Refusal::None),
        (
            json!([{"database":"HGNC","accession":"HGNC:76","label":false}]),
            Refusal::None,
        ),
        (json!([false]), Refusal::Refused),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("cell-line/native-sections", |source, record| {
            if source == "cellosaurus"
                && let Some(value) = record.pointer_mut("/exchanges/0/body/value/Cellosaurus/cell-line-list/0/sequence-variation-list/0")
                && let Some(value) = value.as_object_mut()
            {
                value.insert("xref-list".into(), xrefs.clone());
                changed = true;
            }
        })?;
        assert!(changed, "Cellosaurus native variation fixture missing");
        let leg = fetch.leg(&cellosaurus::SPEC, fetch.deadline(Duration::from_secs(20)));
        let card = cellosaurus::complete(&leg, &"CVCL_0004".parse()?, None)?;
        assert_eq!(card.refusals, expected, "complete card lost child refusal");
        assert!(card.card.is_some());
    }
    Ok(())
}

#[test]
fn european_search_refuses_record_identity_without_refusing_title() -> Check {
    for (row, expected) in [
        (
            json!({"id":"12345","source":"MED","title":false}),
            Refusal::None,
        ),
        (json!({"id":"bad","source":"invalid"}), Refusal::Refused),
        (json!(false), Refusal::Refused),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("article/citations/synthetic-cap", |source, record| {
            if source == "europepmc"
                && let Some(value) = record.pointer_mut("/exchanges/0/body/value")
            {
                *value = json!({"hitCount":1,"resultList":{"result":[row]}});
                changed = true;
            }
        })?;
        assert!(changed, "Europe PMC native search fixture missing");
        let leg = fetch.leg(&europepmc::SPEC, fetch.deadline(Duration::from_secs(20)));
        let page = europepmc::search(&leg, "EGFR lung cancer", biomcp_types::ArticleSort::Rank)?;
        assert_eq!(page.refusals, expected, "search lost actual record refusal");
        assert_eq!(page.count, 1);
    }
    Ok(())
}

#[test]
fn publication_identifier_refusals_preserve_native_count() -> Check {
    for (ids, expected) in [
        (json!([1, 2]), Refusal::None),
        (json!([0, 2]), Refusal::Refused),
        (json!([false, 2]), Refusal::Refused),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("article/litvar/combined-rsid", |source, record| {
            if source == "litvar"
                && record
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|u| u.ends_with("/publications"))
                && let Some(value) = record.pointer_mut("/exchanges/0/body/value")
            {
                *value = json!({"pmids":ids,"pmids_count":2});
                changed = true;
            }
        })?;
        assert!(changed, "LitVar native publications fixture missing");
        let leg = fetch.leg(&litvar::SPEC, fetch.deadline(Duration::from_secs(20)));
        let litvar::Selection::Exact(variant) = litvar::select(&leg, None, "rs121434592")?.value
        else {
            return Err("exact LitVar fixture did not select identity".into());
        };
        let page = litvar::publications(&leg, variant)?;
        assert_eq!(
            page.refusals, expected,
            "LitVar lost publication identity refusal"
        );
        assert_eq!(page.count, Some(2));
        assert!(page.ids.iter().any(|id| id.get() == 2));
    }
    Ok(())
}
