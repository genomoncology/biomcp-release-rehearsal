//! Public response values retain native source fields and distinguish data from failure.
use biodata::Text;
use biomcp_types::{
    ClinGenValidity, CosmicRecord, NciDiseaseRef, Reason, Section, Source, SourceRecord,
};
use serde_json::json;
#[test]
fn source_updates_keep_dates_and_timestamp_precision_distinct() {
    for wire in [
        json!("2024"),
        json!("2024-05"),
        json!("2024-05-03"),
        json!({"value":"2024-05-03T10:00:00.000100+02:00","timezone":"in-value"}),
    ] {
        let value: biomcp_types::SourceUpdated = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), wire);
    }
}
#[test]
fn local_source_timestamps_keep_precision_without_inventing_a_timezone() {
    for (native, timezone) in [
        ("2020-05-08T17:33:59.756164", "source-gave-none"),
        ("2020-05-08T17:33:59", "source-gave-none"),
        ("2024-05-03T10:00:00.000100+02:00", "in-value"),
    ] {
        let checked: biomcp_types::SourceDateTime = native.parse().unwrap();
        let wire = json!({"value":native,"timezone":timezone});
        assert_eq!(serde_json::to_value(&checked).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<biomcp_types::SourceDateTime>(wire).unwrap(),
            checked
        );
    }
    for malformed in [
        "2024-02-30T10:00:00",
        "2024-05-03T24:00:00",
        "2024-05-03T10:00:00.",
    ] {
        assert!(malformed.parse::<biomcp_types::SourceDateTime>().is_err());
    }
    assert!(
        serde_json::from_value::<biomcp_types::SourceTimestamp>(
            json!({"value":"2020-05-08T17:33:59.756164","timezone":"source-gave-none"})
        )
        .is_err()
    );
}
#[test]
fn source_timestamp_values_check_rfc3339_without_changing_offset_or_precision() {
    use biomcp_types::SourceTimestampValue;
    for native in [
        "2024-05-03T10:00:00+02:00",
        "2024-05-03T10:00:00.000100-04:30",
        "2024-02-29t00:00:00.123456789012z",
        "2024-05-03T10:00:00-00:00",
        "1990-12-31T23:59:60Z",
        "1990-12-31T15:59:60-08:00",
        "1991-01-01T00:59:60+01:00",
    ] {
        let checked: SourceTimestampValue = native.parse().unwrap();
        assert_eq!(checked.as_str(), native);
        let wire = json!(native);
        assert_eq!(serde_json::to_value(&checked).unwrap(), wire);
        assert_eq!(
            serde_json::from_value::<SourceTimestampValue>(wire).unwrap(),
            checked
        );
    }
    for native in [
        "2024",
        "2024-05-03",
        "2024-05-03T10:00:00",
        "2023-02-29T10:00:00Z",
        "2024-04-31T10:00:00Z",
        "2024-05-03T24:00:00Z",
        "2024-05-03T10:60:00Z",
        "2024-05-03T10:00:61Z",
        "2024-05-03T10:00:60Z",
        "2024-05-03T10:00:00.Z",
        "2024-05-03T10:00:00+24:00",
        "2024-05-03T10:00:00+02:60",
        "2024-05-03T10:00:00+0200",
        "2024-05-03T10:00:00Z\n",
        "2024-05-03T10:00:00.1.2Z",
        "2024-05-03T10:00:00＋02:00",
        "0000-01-01T00:00:00Z",
    ] {
        assert!(native.parse::<SourceTimestampValue>().is_err(), "{native}");
        assert!(
            serde_json::from_value::<SourceTimestampValue>(json!(native)).is_err(),
            "{native}"
        );
    }
}
#[test]
fn every_section_state_round_trips_without_conflating_data_and_failure() {
    #[cfg(feature = "schema")]
    {
        let schema = serde_json::to_value(schemars::schema_for!(SourceRecord)).unwrap();
        let validator = jsonschema::draft202012::new(&schema).unwrap();
        assert!(validator.is_valid(&json!({"namespace":"unknown","id":"native:0001"})));
        assert!(!validator.is_valid(&json!({"id":"native:0001"})));
        assert!(!validator.is_valid(&json!({"namespace":null,"id":"native:0001"})));
        assert!(serde_json::from_value::<SourceRecord>(json!({"id":"native:0001"})).is_err());
    }
    let states = [
        Section::Data {
            source: Source::Civic,
            data: Text::new("answer").unwrap(),
        },
        Section::Empty {
            source: Source::Civic,
            record: Some(SourceRecord {
                namespace: biodata::IdentifierNamespace::Unknown,
                id: Text::new("1").unwrap(),
                url: None,
            }),
        },
        Section::Empty {
            source: Source::Civic,
            record: None,
        },
        Section::Unavailable {
            source: Source::Civic,
            reason: Reason::Unavailable,
        },
        Section::Skipped {
            source: Source::Civic,
            reason: Reason::NoKey,
        },
        Section::NotApplicable,
    ];
    for state in states {
        let value = serde_json::to_value(&state).unwrap();
        assert_eq!(
            serde_json::from_value::<Section<Text>>(value).unwrap(),
            state
        );
    }
    let extended =
        json!({"state":"unavailable","source":"civic","reason":"unavailable","data":"bad"});
    assert_eq!(
        serde_json::from_value::<Section<Text>>(extended).unwrap(),
        Section::Unavailable {
            source: Source::Civic,
            reason: Reason::Unavailable,
        }
    );
}

#[test]
fn constraint_numbers_refuse_non_finite_values_and_empty_sections_omit_record() {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            biodata::FiniteNumber::try_from(value)
                .unwrap_err()
                .to_string(),
            "number.value.non_finite"
        );
    }
    for value in [0.0, -2.5, f64::MAX] {
        let checked = biodata::FiniteNumber::try_from(value).unwrap();
        assert_eq!(f64::from(checked), value);
        let wire = serde_json::to_string(&checked).unwrap();
        assert_eq!(
            serde_json::from_str::<biodata::FiniteNumber>(&wire).unwrap(),
            checked
        );
    }
    assert_eq!(
        serde_json::to_value(Section::<Text>::Empty {
            source: Source::Gnomad,
            record: None
        })
        .unwrap(),
        serde_json::json!({"state":"empty","source":"gnomad"})
    );
}

#[test]
fn markdown_preserves_layout_and_rejects_empty_values() -> Result<(), Box<dyn std::error::Error>> {
    let value = biodata::Markdown::new("  # Heading\n")?;
    assert_eq!(value.as_str(), "  # Heading\n");
    let json = serde_json::to_string(&value)?;
    assert_eq!(serde_json::from_str::<biodata::Markdown>(&json)?, value);
    assert!(biodata::Markdown::new(" \n").is_err());
    assert!(serde_json::from_str::<biodata::Markdown>("\" \"").is_err());
    let source = " \t# Heading\r\nbefore\u{202e}after ";
    let escaped = biodata::Markdown::new(source)?;
    assert_eq!(escaped.as_str(), " \t# Heading\r\nbefore\\u{202e}after ");
    let at_cap = format!("{}x", " ".repeat(Text::MAX_CHARS - 1));
    assert!(biodata::Markdown::new(&at_cap).is_ok());
    assert!(biodata::Markdown::new(&format!(" {at_cap}")).is_err());
    #[cfg(feature = "schema")]
    {
        let schema = serde_json::to_value(schemars::schema_for!(biodata::Markdown))?;
        let validator = jsonschema::validator_for(&schema)?;
        assert!(validator.is_valid(&json!(escaped)));
        assert!(validator.is_valid(&json!(at_cap)));
        assert!(!validator.is_valid(&json!(format!(" {at_cap}"))));
        assert!(!validator.is_valid(&json!(source)));
    }
    Ok(())
}

#[test]
fn response_identifiers_use_their_publisher_grammar() {
    let cosmic = json!({
        "myvariant_record":"chr7:g.140453136A>T", "identity":"unverified", "cosmic_id":"COSM476"
    });
    let mondo = json!({
        "disease":"source disease", "mondo_id":"MONDO:0000001", "mode_of_inheritance":"source mode",
        "classification":"source classification", "classification_date":"2025-01-01",
        "expert_panel":"source panel", "report_url":"https://example.org/report"
    });
    let ncit = json!({"name":"source name", "nci_thesaurus_concept_id":"C3224"});
    assert!(serde_json::from_value::<CosmicRecord>(cosmic.clone()).is_ok());
    assert!(serde_json::from_value::<ClinGenValidity>(mondo.clone()).is_ok());
    assert!(serde_json::from_value::<NciDiseaseRef>(ncit.clone()).is_ok());
    for bad in ["unrecognized", "COSM0", "COSM0476"] {
        let mut value = cosmic.clone();
        value["cosmic_id"] = json!(bad);
        assert!(
            serde_json::from_value::<CosmicRecord>(value).is_err(),
            "accepted {bad}"
        );
    }
    for bad in ["unrecognized", "DOID:0000001", "MONDO:1"] {
        let mut value = mondo.clone();
        value["mondo_id"] = json!(bad);
        assert!(
            serde_json::from_value::<ClinGenValidity>(value).is_err(),
            "accepted {bad}"
        );
    }
    for bad in ["unrecognized", "MONDO:0000001", "Cbad"] {
        let mut value = ncit.clone();
        value["nci_thesaurus_concept_id"] = json!(bad);
        assert!(
            serde_json::from_value::<NciDiseaseRef>(value).is_err(),
            "accepted {bad}"
        );
    }
}

#[test]
fn native_vocabulary_members_keep_exact_publisher_values() -> Result<(), serde_json::Error> {
    use biodata::CellLineSex;
    use biomcp_types::{CivicKind, CivicSignificance, CivicStatus, NciTrialPhase};
    assert!(matches!(
        serde_json::from_value::<CellLineSex>(json!("Female"))?,
        CellLineSex::Female
    ));
    assert!(matches!(
        serde_json::from_value::<CivicKind>(json!("PREDICTIVE"))?,
        CivicKind::Predictive
    ));
    assert!(matches!(
        serde_json::from_value::<CivicStatus>(json!("ACCEPTED"))?,
        CivicStatus::Accepted
    ));
    assert!(matches!(
        serde_json::from_value::<CivicSignificance>(json!("RESISTANCE"))?,
        CivicSignificance::Resistance
    ));
    assert!(matches!(
        serde_json::from_value::<NciTrialPhase>(json!("I_II"))?,
        NciTrialPhase::Phase1And2
    ));
    for original in [" new value ", "female", "PHASE1", ""] {
        let phase = serde_json::from_value::<NciTrialPhase>(json!(original))?;
        assert!(phase.is_unknown());
        assert_eq!(serde_json::to_value(phase)?, json!(original));
    }
    Ok(())
}

#[test]
fn normalization_source_text_retains_empty_values_and_whitespace() -> Result<(), serde_json::Error>
{
    let original = json!({"category":" infos ", "code":"", "text":"\n  native text  \n"});
    let message: biomcp_types::NormalizationMessage = serde_json::from_value(original.clone())?;
    assert_eq!(serde_json::to_value(message)?, original);
    Ok(())
}

#[test]
fn research_fetch_keeps_the_original_caller_identifier() -> Result<(), serde_json::Error> {
    let original = json!({"id":" gene BRAF ", "title":"BRAF", "text":"source Markdown\n",
        "url":"https://example.org/BRAF", "metadata":{"schema":"source.schema/1", "status":"complete"}});
    let fetch: biomcp_types::ResearchFetch<biomcp_types::ResearchEnvelope> =
        serde_json::from_value(original.clone())?;
    assert_eq!(serde_json::to_value(fetch)?, original);
    Ok(())
}

use biomcp_test_support::readers;
use biomcp_types::{Envelope, Help, ResponseBody, SchemaCatalog, Status};
use serde_json::Value;

#[test]
fn envelope_checks_body_identity_and_accepts_added_reply_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let mut wire = serde_json::to_value(Envelope::<Help>::new(
        Status::Complete,
        serde_json::from_value(json!({}))?,
        Some(Help { commands: vec![] }),
    ))?;
    wire["future"] = json!({"anything": true});
    wire["result"]["future"] = json!(17);
    let read: Envelope<Help> = serde_json::from_value(wire.clone())?;
    let rewritten = serde_json::to_value(read)?;
    assert_eq!(rewritten["schema"], Help::SCHEMA);
    assert!(rewritten.get("future").is_none());
    assert!(serde_json::from_value::<Envelope<SchemaCatalog>>(wire.clone()).is_err());
    wire.as_object_mut().ok_or("object")?.remove("schema");
    assert!(serde_json::from_value::<Envelope<Help>>(wire.clone()).is_err());
    wire["schema"] = json!(42);
    assert!(serde_json::from_value::<Envelope<Help>>(wire).is_err());
    Ok(())
}

#[test]
fn registered_reply_ids_are_unique_and_read_their_concrete_envelope()
-> Result<(), Box<dyn std::error::Error>> {
    let registry = readers::READERS;
    let mut identities = std::collections::BTreeSet::new();
    for reply in registry {
        assert!(identities.insert(reply.0), "{}", reply.0);
    }
    let reply = registry
        .iter()
        .find(|entry| entry.0 == SchemaCatalog::SCHEMA)
        .ok_or("catalog")?;
    let bytes = serde_json::to_vec(&Envelope::<SchemaCatalog>::new(
        Status::Complete,
        biomcp_types::EmptyQuery::default(),
        Some(SchemaCatalog { schemas: vec![] }),
    ))?;
    assert_eq!((reply.1)(&bytes)?, bytes);
    let mut wire: Value = serde_json::from_slice(&bytes)?;
    wire["schema"] = json!(Help::SCHEMA);
    assert!((reply.1)(&serde_json::to_vec(&wire)?).is_err());
    Ok(())
}
