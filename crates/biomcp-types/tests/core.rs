//! Public library tables own the core value and JSON contracts.
use biodata::{
    Attribution, Build, Error, Fraction, PopulationDataType, PopulationFrequency, PopulationSex,
    Text, VariantRef,
};
use biomcp_types::{Problem, ProblemCode, Reason, Source};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use std::{fmt::Debug, str::FromStr};

type Check = Result<(), Box<dyn std::error::Error>>;

fn canonical_names<T>(rows: &[(T, &str)], error: Error) -> Check
where
    T: Copy + Debug + Eq + FromStr<Err = Error> + ToString + Serialize + DeserializeOwned,
{
    for &(value, name) in rows {
        assert_eq!(
            value.to_string(),
            name,
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
        assert_eq!(
            name.parse::<T>()?,
            value,
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
        let json = serde_json::to_string(&value)?;
        assert_eq!(
            json,
            format!("\"{name}\""),
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
        assert_eq!(
            serde_json::from_str::<T>(&json)?,
            value,
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
    }
    for input in ["", " ", "distinctive-rejected-input", "HGNC", " hgnc "] {
        let parsed = input.parse::<T>().err().ok_or("accepted an unknown name")?;
        assert_eq!(
            parsed, error,
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
        assert!(
            !parsed.to_string().contains("distinctive-rejected-input"),
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
        assert!(
            !format!("{parsed:?}").contains("distinctive-rejected-input"),
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
        let json = serde_json::to_string(input)?;
        let decoded = serde_json::from_str::<T>(&json)
            .err()
            .ok_or("decoded an unknown name")?;
        assert!(
            decoded.to_string().contains(error.code()),
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
        assert!(
            !decoded.to_string().contains("distinctive-rejected-input"),
            "Closed vocabularies must preserve canonical names and reject other forms"
        );
    }
    Ok(())
}

#[test]
fn source_names_round_trip_and_reject_unknown_names() -> Check {
    canonical_names(
        &[
            (Source::Hgnc, "hgnc"),
            (Source::MyGene, "mygene"),
            (Source::PubMed, "pubmed"),
            (Source::EuropePmc, "europepmc"),
            (Source::PubTator, "pubtator"),
            (Source::ClinicalTrialsGov, "ctgov"),
            (Source::Civic, "civic"),
        ],
        Error::SourceUnknown,
    )?;
    assert_eq!(Error::SourceUnknown.code(), "source.unknown");
    Ok(())
}

#[test]
fn reason_names_round_trip_and_reject_unknown_names() -> Check {
    canonical_names(
        &[
            (Reason::NotFound, "not-found"),
            (Reason::RateLimited, "rate-limited"),
            (Reason::Unavailable, "unavailable"),
            (Reason::Rejected, "rejected"),
            (Reason::BadReply, "bad-reply"),
            (Reason::DeadlineExceeded, "deadline-exceeded"),
            (Reason::Offline, "offline"),
            (Reason::NoKey, "no-key"),
            (Reason::Disabled, "disabled"),
            (Reason::ReplayMiss, "replay-miss"),
            (Reason::Syncing, "syncing"),
        ],
        Error::ReasonUnknown,
    )?;
    assert_eq!(Error::ReasonUnknown.code(), "reason.unknown");
    Ok(())
}

#[test]
fn problem_codes_round_trip_and_reject_unknown_codes() -> Check {
    canonical_names(
        &[(ProblemCode::ValueUnknown, "value.unknown")],
        Error::ProblemCodeUnknown,
    )?;
    assert_eq!(Error::ProblemCodeUnknown.code(), "problem_code.unknown");
    Ok(())
}

#[test]
fn text_construction_and_json_share_trimming_and_validation() {
    for (input, expected) in [
        ("summary", Some("summary")),
        ("  summary  ", Some("summary")),
        ("\u{2003}HER-2/neu α\u{2003}", Some("HER-2/neu α")),
        ("", None),
        (" \u{2003}", None),
    ] {
        let direct = Text::new(input);
        let parsed = input.parse::<Text>();
        let converted = Text::try_from(input.to_owned());
        assert_eq!(direct, parsed);
        assert_eq!(direct, converted);
        let json = serde_json::to_string(input).unwrap();
        let decoded = serde_json::from_str::<Text>(&json);
        if let Some(expected) = expected {
            let value = direct.unwrap();
            assert_eq!(value.as_str(), expected);
            assert_eq!(value.to_string(), expected);
            assert_eq!(value.to_string().parse::<Text>().unwrap(), value);
            assert_eq!(decoded.unwrap(), value);
            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(json, serde_json::to_string(expected).unwrap());
        } else {
            let error = direct.unwrap_err();
            assert_eq!(error, Error::TextEmpty);
            assert_eq!(error.code(), "text.empty");
            assert!(decoded.unwrap_err().to_string().contains(error.code()));
        }
    }
    fn standard_error(error: &dyn std::error::Error) {
        assert_eq!(error.to_string(), "text.empty");
        assert!(error.source().is_none());
    }
    standard_error(&Error::TextEmpty);
}

#[test]
fn text_refuses_controls_and_bounds_constructor_length() {
    for ch in (0..=0x1f)
        .chain(0x7f..=0x9f)
        .chain([
            0x061c, 0x200e, 0x200f, 0x202a, 0x202b, 0x202c, 0x202d, 0x202e, 0x2066, 0x2067, 0x2068,
            0x2069,
        ])
        .filter_map(char::from_u32)
    {
        for input in [
            format!("before{ch}after"),
            format!("{ch}after"),
            format!("before{ch}"),
        ] {
            assert!(Text::new(&input).is_err(), "{ch:?}");
            assert!(
                serde_json::from_value::<Text>(json!(input)).is_err(),
                "{ch:?}"
            );
            assert!(Text::from_source(&input).is_ok(), "{ch:?}");
        }
    }
    assert!(Text::new(&"α".repeat(1_048_576)).is_ok());
    assert!(Text::new(&"a".repeat(1_048_577)).is_err());
    #[cfg(feature = "schema")]
    {
        let schema = serde_json::to_value(schemars::schema_for!(Text)).unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        for at_cap in ["a".repeat(Text::MAX_CHARS), "α".repeat(Text::MAX_CHARS)] {
            assert!(
                validator.is_valid(&json!(at_cap)),
                "Valid text at the constructor cap must satisfy its schema"
            );
        }
        for control in ["safe\n", "safe\r", "safe\u{0085}", "safe\u{202e}"] {
            assert!(
                !validator.is_valid(&json!(control)),
                "Schema must reject controls at the end: {control:?}"
            );
        }
    }
}

#[test]
fn problems_round_trip_with_optional_record_and_ignore_unknown_fields() {
    for record in [None, Some(Text::new("record-123").unwrap())] {
        let problem = Problem {
            source: Source::PubMed,
            record,
            field: "publication_type".into(),
            code: ProblemCode::ValueUnknown,
        };
        let json = serde_json::to_value(&problem).unwrap();
        assert_eq!(json["source"], "pubmed");
        assert_eq!(json["field"], "publication_type");
        assert_eq!(json["code"], "value.unknown");
        if problem.record.is_some() {
            assert_eq!(json["record"], "record-123");
        } else {
            assert!(json.get("record").is_none());
        }
        assert_eq!(
            serde_json::from_value::<Problem>(json.clone()).unwrap(),
            problem
        );
        let mut extra = json;
        extra["unexpected"] = true.into();
        assert_eq!(serde_json::from_value::<Problem>(extra).unwrap(), problem);
    }
    let invalid = r#"{"source":"pubmed","record":"  ","field":"pmid","code":"value.unknown"}"#;
    let error = serde_json::from_str::<Problem>(invalid).unwrap_err();
    assert!(error.to_string().contains("text.empty"));
}

fn population_base() -> Result<PopulationFrequency, Box<dyn std::error::Error>> {
    let attribution = Attribution {
        approved: None,
        published: None,
        source: Source::Gnomad.text(),
        record: None,
        version: None,
        release: None,
        submitter: None,
        evaluated: None,
        url: None,
    };
    Ok(PopulationFrequency::new(
        VariantRef::allele("CA123643".parse()?),
        Build::GRCh38,
        Text::new("gnomad")?,
        Text::new("synthetic-release")?,
        attribution,
    ))
}

#[test]
fn population_strata_enforce_counts_and_frequencies() -> Check {
    let base = population_base()?;
    let value = base
        .clone()
        .with_stratum(
            Some(PopulationDataType::Exomes),
            Some(Text::new("synthetic-group")?),
            Some(PopulationSex::Xy),
            Some(Text::new("synthetic-subset")?),
        )?
        .with_counts(Some(2), Some(4), Some(0), Some(0))?;
    let mut reported = serde_json::to_value(&value)?;
    reported["allele_frequency"] = json!(0.5);
    reported["faf95"] = json!(0.25);
    let value: PopulationFrequency = serde_json::from_value(reported)?;
    let serialized = serde_json::to_value(&value)?;
    assert_eq!(serialized.get("hemizygote_count"), Some(&json!(0)));
    assert_eq!(serialized.get("allele_frequency"), Some(&json!(0.5)));
    assert_eq!(
        serde_json::from_value::<PopulationFrequency>(serialized.clone())?,
        value
    );
    assert_eq!(
        base.clone()
            .with_counts(Some(3), Some(2), None, None)
            .expect_err("count cannot exceed its denominator")
            .code(),
        "population.count.exceeds_number"
    );
    for invalid in [f64::NAN, f64::INFINITY, -0.01, 1.5] {
        assert_eq!(
            Fraction::try_from(invalid)
                .expect_err("fraction is bounded")
                .code(),
            "population.frequency.range"
        );
    }
    for (field, invalid) in [
        ("allele_count", json!(5)),
        ("allele_frequency", json!(-0.01)),
        ("faf95", json!(1.5)),
    ] {
        let mut changed = serialized.clone();
        changed
            .as_object_mut()
            .ok_or("population object required")?
            .insert(field.into(), invalid);
        assert!(serde_json::from_value::<PopulationFrequency>(changed).is_err());
    }
    Ok(())
}

#[test]
fn population_union_preserves_published_fields_and_refuses_misplacement() -> Check {
    let mut union = serde_json::to_value(population_base()?)?;
    for (field, source_value) in [
        ("heterozygote_count", json!(2)),
        ("grpmax_group", json!("synthetic-group")),
        ("genotype_filter", json!("raw")),
        ("homoplasmic_count", json!(1)),
        ("heteroplasmic_count", json!(3)),
        ("homoplasmic_frequency", json!(0.25)),
        ("heteroplasmic_frequency", json!(0.75)),
    ] {
        union[field] = source_value;
    }
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<PopulationFrequency>(
            union.clone()
        )?)?,
        union
    );
    for (field, bad) in [
        ("genotype_filter", json!("unknown")),
        ("ancestry_group", json!("synthetic")),
        ("homoplasmic_frequency", json!(1.5)),
        ("heteroplasmic_frequency", json!(-0.1)),
    ] {
        let mut invalid = union.clone();
        invalid[field] = bad;
        assert!(
            serde_json::from_value::<PopulationFrequency>(invalid).is_err(),
            "{field}"
        );
    }
    union["subject"] = json!({"level":"allele", "hgvs":"NC_012920.1:m.1A>T"});
    assert!(serde_json::from_value::<PopulationFrequency>(union.clone()).is_ok());
    union["subject"] = json!({"level":"allele", "hgvs":"NC_000007.14:g.1A>T"});
    assert!(serde_json::from_value::<PopulationFrequency>(union).is_err());
    Ok(())
}

#[test]
fn biomarker_replies_ignore_future_fields_while_queries_stay_closed() -> Check {
    let header = json!({
        "codes":["C96022"], "name":"CD274 Gene", "type":["reference_gene"],
        "count":326, "future_header":true
    });
    let page: biomcp_types::BiomarkerPage = serde_json::from_value(json!({
        "items":[header.clone()], "future_page":true
    }))?;
    assert_eq!(
        page.items.len(),
        1,
        "future fields must retain the native header"
    );
    let result: biomcp_types::BiomarkerResult = serde_json::from_value(json!({
        "card":{"header":header, "future_card":true}, "future_result":true
    }))?;
    assert!(
        result.card.is_some(),
        "future fields must retain the native card"
    );
    assert!(
        serde_json::from_value::<biomcp_types::BiomarkerSearch>(json!({
            "source":"nci", "name":"PD-L1", "future_query":true
        }))
        .is_err(),
        "unknown search inputs must remain rejected"
    );
    assert!(
        serde_json::from_value::<biomcp_types::BiomarkerGet>(json!({
            "id":"C96022", "future_query":true
        }))
        .is_err(),
        "unknown get inputs must remain rejected"
    );
    Ok(())
}

#[test]
fn unknown_wire_enum_value_is_private() {
    for input in [
        json!("synthetic-private-enum-value"),
        json!(987654321),
        json!(-987654321),
        json!(12345.6789),
        json!(true),
        json!(null),
        json!(["synthetic-private-enum-value"]),
        json!({"private": "synthetic-private-enum-value"}),
    ] {
        let error = serde_json::from_value::<biodata::TrialSiteStatus>(input)
            .expect_err("unknown enum value must be refused");
        assert_eq!(
            error.to_string(),
            "wire_enum.unknown",
            "decode errors must omit input"
        );
    }
}

#[test]
fn unrelated_value_failures_have_distinct_codes() {
    assert_ne!(
        "future-license"
            .parse::<biodata::License>()
            .unwrap_err()
            .to_string(),
        "source.unknown"
    );
    assert_ne!(
        "future-match"
            .parse::<biomcp_types::ArticleMatchKind>()
            .unwrap_err()
            .to_string(),
        "source.unknown"
    );
    assert_ne!(
        biodata::Bound::new(None, None).unwrap_err().to_string(),
        "coord.span.order"
    );
}

#[test]
fn trial_publication_values_follow_native_registry_grammar() {
    use biodata::{TrialReferenceKind, TrialUploadTime};
    for native in ["BACKGROUND", "RESULT", "DERIVED"] {
        let value: TrialReferenceKind = serde_json::from_value(json!(native)).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), json!(native));
    }
    assert!(serde_json::from_value::<TrialReferenceKind>(json!("OTHER")).is_err());
    for native in ["2024-02-29T23:59", "2023-04-17T19:05"] {
        let value: TrialUploadTime = native.parse().unwrap();
        assert_eq!(value.as_str(), native);
        assert_eq!(serde_json::to_value(value).unwrap(), json!(native));
    }
    for bad in [
        "2023-02-29T12:30",
        "2024-01-01T24:00",
        "2024-01-01T12:60",
        "2024-01-01",
        "2024-01-01T12:00Z",
        "2024-01-01T12:00:00",
        "0000-01-01T00:00",
    ] {
        assert!(bad.parse::<TrialUploadTime>().is_err(), "{bad}");
        assert!(serde_json::from_value::<TrialUploadTime>(json!(bad)).is_err());
    }
    let source = Text::from_source("before\u{202e}after").unwrap();
    assert_eq!(source.as_str(), r"before\u{202e}after");
    assert_eq!(Text::new(source.as_str()).unwrap(), source);
}
