//! Public native mapping boundaries preserve actual refusal facts.
#[path = "custom_refusals/collections.rs"]
mod collections;
use biomcp_test_support::recordings;
#[path = "custom_refusals/replay.rs"]
mod replay;
#[path = "custom_refusals/stored.rs"]
mod stored;
#[path = "custom_refusals/transport.rs"]
mod transport;
use biomcp_adapters::{Refusal, chembl, drugsfda, gwas, jats, nci, semantic_scholar};
use biomcp_fetch::xml::XmlReply;
use serde_json::json;
type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn publication_members_distinguish_records_from_optional_properties() -> Check {
    for (authors, expected) in [
        (json!(false), Refusal::None),
        (json!([]), Refusal::None),
        (json!([{"name":false}]), Refusal::None),
        (json!([false]), Refusal::Refused),
    ] {
        let row = json!({"paperId":"0123456789abcdef0123456789abcdef01234567","authors":authors});
        let search: semantic_scholar::Search =
            serde_json::from_value(json!({"total":1,"data":[row.clone()]}))?;
        assert_eq!(
            search.refusals, expected,
            "search lost native child refusal"
        );
        let recommendations: semantic_scholar::Recommendations =
            serde_json::from_value(json!({"recommendedPapers":[row]}))?;
        assert_eq!(recommendations.refusals, expected);
        assert_eq!(search.data.len(), 1);
        assert_eq!(recommendations.recommended_papers.len(), 1);
    }
    let search: semantic_scholar::Search =
        serde_json::from_value(json!({"total":1,"data":[false]}))?;
    assert_eq!(search.refusals, Refusal::Refused);
    assert!(search.data.is_empty());
    Ok(())
}

#[test]
fn author_paper_snapshot_preserves_child_refusal_and_legacy_absence() -> Check {
    for (authors, expected) in [
        (json!(false), Refusal::None),
        (json!([false]), Refusal::Refused),
    ] {
        let page: semantic_scholar::author::PapersPage = serde_json::from_value(json!({
            "offset":0,"next":25,"total":1,"data":[{
                "paperId":"0123456789abcdef0123456789abcdef01234567","authors":authors
            }]
        }))?;
        assert_eq!(
            page.refusals, expected,
            "author page lost native child refusal"
        );
        let saved = serde_json::to_value(&page)?;
        let restored: semantic_scholar::author::PapersPage = serde_json::from_value(saved.clone())?;
        assert_eq!(restored.refusals, expected);
        assert_eq!(
            restored.data.first().ok_or("missing paper")?.refusals,
            expected
        );
        assert_eq!(
            saved.get("refusals"),
            Some(&json!(expected == Refusal::Refused))
        );
        assert_eq!(restored.next, Some(25));
    }
    let legacy: semantic_scholar::author::PapersPage =
        serde_json::from_value(json!({"offset":0,"data":[]}))?;
    assert_eq!(legacy.refusals, Refusal::None);
    Ok(())
}

#[test]
fn nci_trial_children_refuse_records_without_refusing_optional_fields() -> Check {
    for (diseases, expected) in [
        (json!(false), Refusal::None),
        (
            json!([{"nci_thesaurus_concept_id":"C3224","name":false}]),
            Refusal::None,
        ),
        (
            json!([{"nci_thesaurus_concept_id":false}]),
            Refusal::Refused,
        ),
    ] {
        let page: nci::Page = serde_json::from_value(json!({"total":1,"data":[{
            "nci_id":"NCI-2026-00001","nct_id":"NCT04303780","diseases":diseases
        }]}))?;
        assert_eq!(
            page.refusals, expected,
            "trial collection lost child refusal"
        );
        let trial = page.data.first().ok_or("missing retained trial")?;
        assert_eq!(trial.refusals, expected);
        assert_eq!(trial.card(false).nct_id.to_string(), "NCT04303780");
    }
    Ok(())
}

#[test]
fn application_children_distinguish_native_members_and_optional_metadata() -> Check {
    for (products, expected) in [
        (json!(false), Refusal::None),
        (
            json!([{"product_number":"001","brand_name":false}]),
            Refusal::None,
        ),
        (json!([{"product_number":false}]), Refusal::Refused),
        (
            json!([{"product_number":"001","active_ingredients":[false]}]),
            Refusal::Refused,
        ),
    ] {
        let row: drugsfda::Application =
            serde_json::from_value(json!({"application_number":"NDA020895","products":products}))?;
        assert_eq!(row.refusals(), expected, "application lost child refusal");
        assert_eq!(row.header().id.to_string(), "NDA020895");
    }
    Ok(())
}

#[test]
fn association_children_distinguish_required_identity_and_optional_properties() -> Check {
    for (traits, expected) in [
        (json!(false), Refusal::None),
        (
            json!([{"efo_id":"EFO_0000305","efo_trait":false}]),
            Refusal::None,
        ),
        (json!([false]), Refusal::Refused),
        (json!([{"efo_id":"invalid"}]), Refusal::Refused),
    ] {
        let row: gwas::Association =
            serde_json::from_value(json!({"association_id":1,"efo_traits":traits}))?;
        assert_eq!(
            row.refusals, expected,
            "association lost actual child refusal"
        );
        assert_eq!(row.value.id.get(), 1);
    }
    Ok(())
}

#[test]
fn molecule_window_owns_selected_member_refusals() -> Check {
    for (members, expected) in [
        (json!(false), Refusal::None),
        (json!([false]), Refusal::Refused),
    ] {
        let page: chembl::Molecules = serde_json::from_value(json!({
            "molecules":[{"molecule_chembl_id":"CHEMBL25","molecule_synonyms":members}],
            "page_meta":{"total_count":2,"offset":0,"limit":1}
        }))?;
        assert_eq!(
            page.window.refusals(),
            expected,
            "native window lost child refusal"
        );
        let row = page
            .window
            .rows
            .first()
            .ok_or("missing retained molecule")?;
        assert_eq!(
            row.refusals(),
            Refusal::None,
            "window must own transferred fact"
        );
        assert_eq!(page.window.received(), 1);
    }
    for (fields, expected) in [
        (json!({"molecule_hierarchy":false}), Refusal::None),
        (
            json!({"molecule_hierarchy":{"parent_chembl_id":""}}),
            Refusal::Refused,
        ),
        (
            json!({"molecule_synonyms":[{"molecule_synonym":"Retained","syn_type":false}]}),
            Refusal::None,
        ),
        (
            json!({"molecule_synonyms":[{"syn_type":"OTHER"}]}),
            Refusal::Refused,
        ),
        (
            json!({"cross_references":[{"xref_id":"","xref_src":"ChEBI"}]}),
            Refusal::Refused,
        ),
    ] {
        let mut native = json!({"molecule_chembl_id":"CHEMBL25"});
        native
            .as_object_mut()
            .ok_or("molecule object")?
            .extend(fields.as_object().ok_or("field object")?.clone());
        let card = chembl::molecule_card(serde_json::from_value(native)?, None)?;
        assert_eq!(
            card.refusals, expected,
            "molecule projection lost actual child refusal"
        );
    }
    Ok(())
}

#[test]
fn bibliography_identity_refusal_preserves_body_and_optional_metadata() -> Check {
    for (reference, expected) in [
        (
            "<ref><mixed-citation>Unidentified</mixed-citation></ref>",
            Refusal::Refused,
        ),
        (
            "<ref id=\"B1\"><pub-id pub-id-type=\"doi\">invalid</pub-id></ref>",
            Refusal::None,
        ),
    ] {
        let xml = format!(
            "<article><body><p>Retained body.</p></body><back><ref-list>{reference}</ref-list></back></article>"
        );
        let body = jats::Body::decode_xml(&xml).map_err(|_| "JATS protocol")?;
        assert_eq!(
            body.refusals, expected,
            "bibliography lost identity refusal"
        );
        assert!(!body.rows.is_empty());
    }
    Ok(())
}
