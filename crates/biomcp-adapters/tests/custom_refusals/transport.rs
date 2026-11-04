use super::{Check, Refusal, replay::replay};
use biomcp_adapters::{cpic, faers, interpro, monarch};
use serde_json::{Value, json};
use std::time::Duration;

#[test]
fn cpic_record_identity_refusal_preserves_native_consumption() -> Check {
    for (row, expected, accepted) in [
        (
            json!({"pairid":1,"genesymbol":"DPYD","drugname":false}),
            Refusal::None,
            1,
        ),
        (
            json!({"pairid":false,"genesymbol":"DPYD"}),
            Refusal::Refused,
            0,
        ),
        (json!({"pairid":1,"genesymbol":"TPMT"}), Refusal::Refused, 0),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("pgx/cpic-synthetic", |source, record| {
            if source == "cpic"
                && record
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.contains("pair_view") && s.contains("limit=5"))
                && let Some(body) = record.pointer_mut("/exchanges/0/body/value")
            {
                *body = json!([row]);
                changed = true;
            }
        })?;
        assert!(changed, "CPIC native page fixture missing");
        let query = serde_json::from_value(json!({"gene":"DPYD","source":"cpic"}))?;
        let leg = fetch.leg(&cpic::SPEC, fetch.deadline(Duration::from_secs(20)));
        let page = cpic::search(&leg, &query)?;
        assert_eq!(page.refusals, expected, "CPIC lost actual identity refusal");
        assert_eq!(page.items.len(), accepted);
        assert_eq!(page.consumed, 1);
        assert_eq!(page.total, 2);
    }
    Ok(())
}

#[test]
fn interpro_fragments_distinguish_record_bounds_and_optional_containers() -> Check {
    for (fragments, expected) in [
        (json!(false), Refusal::None),
        (
            json!([{"start":1,"end":2,"dc-status":false}]),
            Refusal::None,
        ),
        (json!([false]), Refusal::Refused),
        (json!([{"start":3,"end":2}]), Refusal::Refused),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("protein/examples", |source, record| {
            if source == "interpro"
                && let Some(body) = record.pointer_mut(
                    "/exchanges/0/body/value/results/0/proteins/0/entry_protein_locations",
                )
            {
                *body = json!([{"fragments":fragments}]);
                changed = true;
            }
        })?;
        assert!(changed, "InterPro native location fixture missing");
        let leg = fetch.leg(&interpro::SPEC, fetch.deadline(Duration::from_secs(20)));
        let page = interpro::page(&leg, &"P00533".parse()?, None, 25)?;
        assert_eq!(
            page.refusals, expected,
            "InterPro lost examined fragment refusal"
        );
        assert_eq!(page.items.len(), 1);
        let encoded = serde_json::to_value(&page)?;
        let restored: interpro::Page = serde_json::from_value(encoded.clone())?;
        assert_eq!(restored.refusals, expected);
        let mut legacy = encoded;
        legacy
            .as_object_mut()
            .ok_or("stored page object")?
            .remove("refusals");
        let restored: interpro::Page = serde_json::from_value(legacy)?;
        assert_eq!(restored.refusals, Refusal::None);
    }
    Ok(())
}

#[test]
fn hierarchy_members_distinguish_identity_from_optional_labels() -> Check {
    for (rows, expected) in [
        (json!(false), Refusal::None),
        (json!([{"id":"HP:0000002","name":false}]), Refusal::None),
        (json!([{"id":false}]), Refusal::Refused),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("phenotype/synthetic", |source, record| {
            if source == "monarch"
                && record
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.contains("/entity/"))
                && let Some(body) = record.pointer_mut("/exchanges/0/body/value")
                && let Some(body) = body.as_object_mut()
            {
                body.insert("node_hierarchy".into(), json!({"super_classes":rows}));
                changed = true;
            }
        })?;
        assert!(changed, "Monarch exact term fixture missing");
        let query = serde_json::from_value(json!({"id":"HP:0000001","source":"monarch"}))?;
        let leg = fetch.leg(&monarch::SPEC, fetch.deadline(Duration::from_secs(20)));
        let row = monarch::phenotype::get(&leg, &query)?;
        assert_eq!(
            row.refusals, expected,
            "hierarchy lost actual member refusal"
        );
        assert!(row.hierarchy.is_some());
    }
    Ok(())
}

#[test]
fn report_member_refusal_preserves_native_indexes_and_optional_values() -> Check {
    for (drugs, expected) in [
        (json!(false), Refusal::None),
        (json!([{"medicinalproduct":false}]), Refusal::None),
        (
            json!([false,{"medicinalproduct":"Retained"}]),
            Refusal::Refused,
        ),
    ] {
        let mut changed = false;
        let (_root, fetch) = replay("adverse-event/faers-synthetic", |source, record| {
            if source == "faers"
                && record
                    .get("url")
                    .and_then(Value::as_str)
                    .is_some_and(|s| s.contains("safetyreportid"))
                && let Some(body) =
                    record.pointer_mut("/exchanges/0/body/value/results/0/patient/drug")
            {
                *body = drugs.clone();
                changed = true;
            }
        })?;
        assert!(changed, "FAERS exact report fixture missing");
        let leg = fetch.leg(&faers::SPEC, fetch.deadline(Duration::from_secs(20)));
        let row = faers::detail::get(&leg, &"100".parse()?)?;
        assert_eq!(
            row.refusals, expected,
            "FAERS lost actual native drug refusal"
        );
        assert!(row.report.is_some());
        if expected == Refusal::Refused {
            assert_eq!(
                row.positions
                    .drugs
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
