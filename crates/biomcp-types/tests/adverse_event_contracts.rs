//! adverse-event-types: caller-owned no-I/O report contracts.
use biodata::{FaersReport, MaudeReport};
use biomcp_types::Paged;
use biomcp_types::{AdverseEventGet, AdverseEventSearch};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
fn roundtrip<T: Serialize + DeserializeOwned>(value: Value) -> Result<Value, serde_json::Error> {
    serde_json::to_value(serde_json::from_value::<T>(value)?)
}
#[test]
fn source_requests_refuse_cross_family_filters_sections_and_invented_ids() {
    assert_eq!(
        roundtrip::<AdverseEventSearch>(json!({"text":"synthetic"})).unwrap()["source"],
        "faers"
    );
    for value in [
        json!({"text":"x","source":"maude","reaction":"x"}),
        json!({"text":"x","serious":"disability"}),
        json!({"text":"x","received_from":"2024-02-30"}),
        json!({"text":"x","received_from":"2024-02-02","received_to":"2024-01-01"}),
    ] {
        assert!(serde_json::from_value::<AdverseEventSearch>(value).is_err());
    }
    for value in [
        json!({"id":"FAERS:26940567"}),
        json!({"id":"26940567","section":"devices"}),
        json!({"id":"50952244","source":"maude","section":"drugs"}),
        json!({"id":"26940567","drug_index":0}),
        json!({"id":"26940567","section":"unii"}),
        json!({"id":"26940567","version":"1"}),
    ] {
        assert!(serde_json::from_value::<AdverseEventGet>(value).is_err());
    }
}
#[test]
fn report_records_preserve_array_order_empty_strings_and_missing_reporter_qualification() {
    let drugs: Vec<_> = (0..19)
        .map(|n| json!({"medicinalproduct":format!("synthetic-{n}"),"openfda":{"unii":["A","B"]}}))
        .collect();
    let reactions: Vec<_> = (0..22)
        .map(|n| json!({"term":{"label":format!("synthetic reaction {n}")},"outcome":"1"}))
        .collect();
    let report = json!(
        {"safetyreportid":"26940567","safetyreportversion":"1","receivedate":"20240101","receiptdate":"20240201","seriousness":{"death":"1",
        "hospitalization":"1"},"reporter":{"country":""},"patient":{"age":{"value":42.5,"unit":"801"}},"drugs":drugs,"reactions":reactions}
    );
    assert_eq!(roundtrip::<FaersReport>(report.clone()).unwrap(), report);
    for length in [0, 5] {
        let narrative: Vec<_> = (0..length)
            .map(|n| json!({"text_type_code":"N","text":format!("Synthetic narrative {n}")}))
            .collect();
        let mut device = json!({"mdr_report_key":"50952244","devices":[{"brand_name":""},{"brand_name":"synthetic"}],"narrative":narrative});
        if length == 0 {
            device.as_object_mut().unwrap().remove("narrative");
        }
        assert_eq!(roundtrip::<MaudeReport>(device.clone()).unwrap(), device);
    }
}

#[test]
fn headers_and_cards_ignore_unrecognized_fields_and_keep_their_disclosures() {
    use biomcp_types::{AdverseEventFaersCard, AdverseEventHeader, AdverseEventMaudeCard};
    let header = json!({"id":"26940567","source":"faers","version":"1","first_received":"20240101","serious":"1"});
    assert_eq!(
        roundtrip::<AdverseEventHeader>(header.clone()).unwrap(),
        header
    );
    let faers = json!(
        {"header":header,"snapshot":"synthetic-snapshot","drug_count":19,"reaction_count":22,"source_url":"https://example.invalid/source",
        "report_url":"https://example.invalid/exact-report","available_sections":["drugs","reactions"]}
    );
    let maude = json!(
        {"header":{"id":"50952244","source":"maude","event_type":"Death"},"snapshot":"synthetic-snapshot","device_count":1,
        "narrative_count":0,"source_url":"https://example.invalid/source","report_url":"https://example.invalid/exact-report",
        "available_sections":["devices","narrative"]}
    );
    roundtrip::<AdverseEventFaersCard>(faers.clone()).unwrap();
    roundtrip::<AdverseEventMaudeCard>(maude.clone()).unwrap();
    for field in ["drugs", "reactions", "counts", "title"] {
        let mut h = header.clone();
        h[field] = json!([]);
        assert_eq!(roundtrip::<AdverseEventHeader>(h).unwrap(), header);
        let mut f = faers.clone();
        f[field] = json!([]);
        assert_eq!(roundtrip::<AdverseEventFaersCard>(f).unwrap(), faers);
        let mut m = maude.clone();
        m[field] = json!([]);
        assert_eq!(roundtrip::<AdverseEventMaudeCard>(m).unwrap(), maude);
    }
}
#[test]
fn every_closed_selector_and_published_id_roundtrips() {
    use biomcp_types::{
        AdverseEventIdentifierSection, AdverseEventSection, AdverseEventSerious, AdverseEventSource,
    };
    for source in ["faers", "maude"] {
        assert_eq!(
            roundtrip::<AdverseEventSource>(json!(source)).unwrap(),
            source
        );
        let q: AdverseEventGet =
            serde_json::from_value(json!({"id":"001234","source":source})).unwrap();
        assert_eq!(q.id().as_str(), "001234");
        assert_eq!(q.source().as_str(), source);
        assert_eq!(q.section_limit(), 5);
    }
    for flag in [
        "any",
        "death",
        "hospitalization",
        "lifethreatening",
        "disabling",
        "congenital-anomaly",
        "other",
    ] {
        assert_eq!(roundtrip::<AdverseEventSerious>(json!(flag)).unwrap(), flag);
    }
    for name in [
        "drugs",
        "reactions",
        "unii",
        "rxcui",
        "product-ndc",
        "package-ndc",
        "spl-id",
        "spl-set-id",
        "application-number",
        "devices",
        "narrative",
    ] {
        let section: AdverseEventSection = serde_json::from_value(json!(name)).unwrap();
        assert_eq!(section.as_str(), name);
        assert_eq!(roundtrip::<AdverseEventSection>(json!(name)).unwrap(), name);
        let identifier = serde_json::from_value::<AdverseEventIdentifierSection>(json!(name));
        assert_eq!(identifier.is_ok(), section.is_identifier());
        if let Ok(value) = identifier {
            assert_eq!(value.as_str(), name);
        }
    }
    for name in ["all", "reporter", "vaccine", "unknown"] {
        assert!(serde_json::from_value::<AdverseEventSection>(json!(name)).is_err());
        assert!(serde_json::from_value::<AdverseEventSource>(json!(name)).is_err());
        assert!(serde_json::from_value::<AdverseEventSerious>(json!(name)).is_err());
    }
}
#[test]
fn paging_preserves_all_selectors_and_absolute_drug_scope() {
    use biomcp_types::{AdverseEventSection, AdverseEventSerious, AdverseEventSource};
    let query: AdverseEventSearch = serde_json::from_value(json!(
        {"text":"synthetic drug","reaction":"synthetic phrase","serious":"hospitalization","received_from":"2024-02-29",
        "received_to":"2024-12-31","limit":2}
    ))
    .unwrap();
    let next = query.at_offset(2);
    assert_eq!(
        next.text().map(biodata::Text::as_str),
        Some("synthetic drug")
    );
    assert_eq!(next.source(), AdverseEventSource::Faers);
    assert_eq!(next.reaction().unwrap().as_str(), "synthetic phrase");
    assert_eq!(next.serious(), Some(AdverseEventSerious::Hospitalization));
    assert_eq!(next.received_from().unwrap().to_string(), "2024-02-29");
    assert_eq!(next.received_to().unwrap().to_string(), "2024-12-31");
    assert_eq!((next.limit(), next.offset()), (2, 2));
    let card: AdverseEventGet =
        serde_json::from_value(json!({"id":"26940567","section_limit":2})).unwrap();
    let section = card
        .with_section(AdverseEventSection::PackageNdc, Some(18))
        .unwrap()
        .at_section_offset(2);
    assert_eq!(section.id(), card.id());
    assert_eq!(section.source(), card.source());
    assert_eq!(section.section(), Some(AdverseEventSection::PackageNdc));
    assert_eq!(
        (
            section.drug_index(),
            section.section_limit(),
            section.section_offset()
        ),
        (Some(18), 2, 2)
    );
    assert!(
        card.with_section(AdverseEventSection::Devices, None)
            .is_err()
    );
}

#[cfg(feature = "schema")]
fn envelope<T>(query: Value, body: Value) -> Result<(), Box<dyn std::error::Error>>
where
    T: biomcp_types::ResponseBody + schemars::JsonSchema + DeserializeOwned,
    T::Query: schemars::JsonSchema + DeserializeOwned,
{
    use biomcp_types::{Envelope, Status};
    let query = serde_json::from_value(query)?;
    let body = serde_json::from_value::<T>(body)?;
    let value = serde_json::to_value(Envelope::new(Status::Complete, query, Some(body)))?;
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<Envelope<T>>();
    let validator = jsonschema::draft202012::new(&serde_json::to_value(schema)?)?;
    assert!(validator.is_valid(&value), "{value}");
    assert_eq!(
        value.get("schema").and_then(Value::as_str),
        Some(T::SCHEMA),
        "Adverse event replies must retain their versioned envelope contract"
    );
    Ok(())
}
#[cfg(feature = "schema")]
#[test]
fn eight_consumer_envelopes_validate_and_identifier_pages_keep_closed_native_scope() {
    use biomcp_types::*;
    let query = json!({"id":"26940567"});
    let header = json!({"id":"26940567","source":"faers","version":"1"});
    let mut base = json!({"header":header,"snapshot":"synthetic-snapshot"});
    envelope::<AdverseEventPage>(
        json!({"text":"synthetic"}),
        json!({"source_query":"synthetic","items":[header],"unit":"reports"}),
    )
    .unwrap();
    let mut faers = base.clone();
    faers["drug_count"] = json!(19);
    faers["reaction_count"] = json!(22);
    faers["source_url"] = json!("https://example.invalid/source");
    faers["report_url"] = json!("https://example.invalid/report");
    faers["available_sections"] = json!(["drugs", "reactions"]);
    envelope::<AdverseEventFaersCard>(query.clone(), faers).unwrap();
    let mut maude = base.clone();
    maude["header"] = json!({"id":"50952244","source":"maude"});
    maude["device_count"] = json!(1);
    maude["narrative_count"] = json!(0);
    maude["source_url"] = json!("https://example.invalid/source");
    maude["report_url"] = json!("https://example.invalid/report");
    maude["available_sections"] = json!(["devices", "narrative"]);
    envelope::<AdverseEventMaudeCard>(json!({"id":"50952244","source":"maude"}), maude).unwrap();
    base["drugs"] = json!({"records":[{"index":18,"medicinalproduct":"","role":"1","identity_counts":{"unii":2}}],"offset":18,"total":19});
    envelope::<AdverseEventDrugs>(json!({"id":"26940567","section":"drugs"}), base.clone())
        .unwrap();
    base.as_object_mut().unwrap().remove("drugs");
    base["reactions"] = json!(
        {"records":[{"index":21,"term":{"label":"Synthetic reaction"},"vocabulary_version":"27.1","outcome":"5"}],"offset":21,"total":22}
    );
    let reaction = roundtrip::<AdverseEventReactions>(base.clone()).unwrap();
    assert!(reaction.pointer("/reactions/records/0/term/id").is_none());
    envelope::<AdverseEventReactions>(json!({"id":"26940567","section":"reactions"}), base.clone())
        .unwrap();
    base.as_object_mut().unwrap().remove("reactions");
    for (name, field) in [
        ("unii", "unii"),
        ("rxcui", "rxcui"),
        ("product-ndc", "product_ndc"),
        ("package-ndc", "package_ndc"),
        ("spl-id", "spl_id"),
        ("spl-set-id", "spl_set_id"),
        ("application-number", "application_number"),
    ] {
        let mut page = base.clone();
        page["section"] = json!(name);
        page[field] =
            json!({"drug_index":18,"records":["001-02","001-02",""],"offset":0,"total":3});
        assert_eq!(
            roundtrip::<AdverseEventIdentifiers>(page.clone()).unwrap(),
            page
        );
        envelope::<AdverseEventIdentifiers>(
            json!({"id":"26940567","section":name,"drug_index":18}),
            page.clone(),
        )
        .unwrap();
        let known = page.clone();
        page["arbitrary_field"] = json!([]);
        assert_eq!(roundtrip::<AdverseEventIdentifiers>(page).unwrap(), known);
    }
    base["header"] = json!({"id":"50952244","source":"maude"});
    base["devices"] = json!({"records":[{"index":0,"brand_name":""}],"offset":0,"total":1});
    envelope::<AdverseEventDevices>(
        json!({"id":"50952244","source":"maude","section":"devices"}),
        base.clone(),
    )
    .unwrap();
    base.as_object_mut().unwrap().remove("devices");
    base["narrative"] = json!({"records":[{"index":0,"type":"N","text":"  Complete synthetic text.\n"}],"offset":0,"total":1});
    assert_eq!(
        roundtrip::<AdverseEventNarrative>(base.clone()).unwrap(),
        base
    );
    envelope::<AdverseEventNarrative>(
        json!({"id":"50952244","source":"maude","section":"narrative"}),
        base,
    )
    .unwrap();
}
#[test]
fn request_boundaries_reject_missing_text_null_limits_invalid_dates_and_removed_options() {
    for value in [
        json!({}),
        json!({"text":""}),
        json!({"text":"x","limit":0}),
        json!({"text":"x","limit":51}),
        json!({"text":"x","limit":null}),
        json!({"text":"x","received_to":"2024"}),
        json!({"text":"x","source":"maude","received_from":"2020"}),
        json!({"text":"x","source":"maude","received_from":"2020-02","received_to":"2020-01"}),
        json!({"text":"MMR","source":"vaers","received_from":"2020-01"}),
        json!({"text":"x","count":"reaction"}),
        json!({"query":"x"}),
        json!({"text":"x","source":"vaccine"}),
        json!({"text":"x","source":"maude","serious":"death"}),
    ] {
        assert!(serde_json::from_value::<AdverseEventSearch>(value).is_err());
    }
    for value in [
        json!({"id":""}),
        json!({"id":"MAUDE:50952244","source":"maude"}),
        json!({"id":"1","section_limit":0}),
        json!({"id":"1","section_limit":51}),
        json!({"id":"1","section_limit":null}),
        json!({"id":"1","section":"all"}),
        json!({"id":"1","section":"reporter"}),
        json!({"id":"1","section":"drugs","drug_index":0}),
    ] {
        assert!(serde_json::from_value::<AdverseEventGet>(value).is_err());
    }
    for date in [
        json!({"text":"x","received_from":"2024-02-29"}),
        json!({"text":"x","received_to":"2024-02-29"}),
    ] {
        assert!(serde_json::from_value::<AdverseEventSearch>(date).is_ok());
    }
}
