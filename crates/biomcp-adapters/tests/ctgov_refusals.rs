//! Public registry adapter boundaries distinguish refused records from retained fields.
use biomcp_adapters::{Refusal, ctgov};
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use biomcp_test_support::recordings;
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path, time::Duration};
type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn replay(capture: &str, body: Value) -> Check<(tempfile::TempDir, Fetch)> {
    let root = tempfile::tempdir()?;
    let target = root.path().join("recordings");
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/replies")
        .join(capture);
    let mut changed = false;
    recordings::copy_capture(&source, &target, |source, record| {
        if source == "ctgov"
            && record
                .get("url")
                .and_then(Value::as_str)
                .is_some_and(|url| url.contains("/studies"))
            && let Some(value) = record.pointer_mut("/exchanges/0/body/value")
        {
            *value = body.clone();
            changed = true;
            recordings::cache_acquisition(record, 42);
        }
    })?;
    assert!(changed, "native registry recording missing");
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: target },
        cache: Some(root.path().join("cache.sqlite")),
        ..Default::default()
    })?;
    Ok((root, fetch))
}

fn protocol() -> Value {
    json!({"identificationModule":{"nctId":"NCT04303780","briefTitle":"Retained trial"}})
}

fn detail(body: Value) -> Check<ctgov::detail::Detail> {
    let (_root, fetch) = replay("trial/get/NCT04303780", body)?;
    let leg = fetch.leg(&ctgov::SPEC, fetch.deadline(Duration::from_secs(20)));
    Ok(ctgov::detail::get(&leg, "NCT04303780".parse()?)?.0)
}

#[test]
fn detail_records_distinguish_optional_containers_and_properties() -> Check {
    for (fields, expected) in [
        (
            json!({"referencesModule":{"references":[false]}}),
            Refusal::Refused,
        ),
        (
            json!({"referencesModule":{"references":false}}),
            Refusal::None,
        ),
        (
            json!({"referencesModule":{"references":[{"citation":false}]}}),
            Refusal::None,
        ),
        (
            json!({"contactsLocationsModule":{"locations":[{"contacts":[false]}]}}),
            Refusal::Refused,
        ),
        (
            json!({"contactsLocationsModule":{"locations":[{"contacts":false}]}}),
            Refusal::None,
        ),
    ] {
        let mut protocol = protocol();
        protocol
            .as_object_mut()
            .ok_or("protocol object")?
            .extend(fields.as_object().ok_or("field object")?.clone());
        let mapped = detail(json!({"protocolSection":protocol}))?;
        assert_eq!(
            mapped.refusals, expected,
            "registry detail lost actual child refusal"
        );
        assert_eq!(mapped.document_refusals, Refusal::None);
        assert_eq!(mapped.card.nct_id.to_string(), "NCT04303780");
    }
    Ok(())
}

#[test]
fn document_records_keep_blank_labels_as_non_refusals() -> Check {
    for (docs, expected, count) in [
        (
            json!([{"filename":"Prot_001.pdf","label":" "}]),
            Refusal::None,
            1,
        ),
        (
            json!([{"filename":"Prot_001.pdf","label":false}]),
            Refusal::None,
            1,
        ),
        (json!(false), Refusal::None, 0),
        (json!([false]), Refusal::Refused, 0),
        (json!([{"label":"Retained label"}]), Refusal::Refused, 0),
        (json!([{"filename":"../Prot_001.pdf"}]), Refusal::Refused, 0),
    ] {
        let mapped = detail(json!({
            "protocolSection":protocol(),
            "documentSection":{"largeDocumentModule":{"largeDocs":docs}}
        }))?;
        assert_eq!(
            mapped.document_refusals, expected,
            "registry manifest lost actual refusal"
        );
        assert_eq!(mapped.refusals, expected);
        assert_eq!(mapped.documents.len(), count);
        if let Some(document) = mapped.documents.first() {
            assert!(document.label.is_none());
        }
    }
    Ok(())
}

#[test]
fn snapshot_facts_round_trip_and_legacy_fields_default_to_none() -> Check {
    let mapped = detail(json!({
        "protocolSection":protocol(),
        "documentSection":{"largeDocumentModule":{"largeDocs":[false]}}
    }))?;
    let mut saved = serde_json::to_value(&mapped)?;
    let restored: ctgov::detail::Detail = serde_json::from_value(saved.clone())?;
    assert_eq!(restored.refusals, Refusal::Refused);
    assert_eq!(restored.document_refusals, Refusal::Refused);
    let fields = saved.as_object_mut().ok_or("detail snapshot object")?;
    assert_eq!(fields.remove("refusals"), Some(json!(true)));
    assert_eq!(fields.remove("document_refusals"), Some(json!(true)));
    let legacy: ctgov::detail::Detail = serde_json::from_value(saved.clone())?;
    assert_eq!(legacy.refusals, Refusal::None);
    assert_eq!(legacy.document_refusals, Refusal::None);
    let mut encoded = serde_json::to_value(legacy)?;
    let fields = encoded.as_object_mut().ok_or("encoded snapshot object")?;
    fields.remove("refusals");
    fields.remove("document_refusals");
    assert_eq!(encoded, saved, "legacy snapshot payload changed");
    Ok(())
}

#[test]
fn search_refusals_preserve_native_continuation_and_retained_studies() -> Check {
    let params: BTreeMap<String, String> = [
        ("countTotal", "true"),
        ("fields", "NCTId,BriefTitle,OverallStatus,Phase,Condition,InterventionName,LastUpdatePostDate,HasResults"),
        ("pageSize", "1"),
        ("query.cond", "(melanoma)"),
    ].map(|(k,v)| (k.into(),v.into())).into();
    for (row, expected, count) in [
        (json!(false), Refusal::Refused, 0),
        (json!({"protocolSection":protocol()}), Refusal::None, 1),
        (
            json!({"protocolSection":{
                "identificationModule":{"nctId":"NCT04303780","briefTitle":false},
                "armsInterventionsModule":{"interventions":false}
            }}),
            Refusal::None,
            1,
        ),
        (
            json!({"protocolSection":{
                "identificationModule":{"nctId":"NCT04303780"},
                "armsInterventionsModule":{"interventions":[false]}
            }}),
            Refusal::Refused,
            1,
        ),
    ] {
        let (_root, fetch) = replay(
            "trial/wider-full",
            json!({
                "totalCount":1,"studies":[row],"nextPageToken":"native-cursor"
            }),
        )?;
        let leg = fetch.leg(&ctgov::SPEC, fetch.deadline(Duration::from_secs(20)));
        let page = ctgov::search(&leg, &params, &[], &[])?;
        assert_eq!(
            page.refusals, expected,
            "registry search lost actual refusal"
        );
        assert_eq!(page.items.len(), count);
        assert_eq!(
            page.next.as_ref().map(biodata::Text::as_str),
            Some("native-cursor")
        );
        assert_eq!(page.total, Some(1));
    }
    Ok(())
}
