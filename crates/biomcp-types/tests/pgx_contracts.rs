//! pgx-types: checked public requests and neutral records, with no I/O.
use biodata::ClinPgxAccession;
use biomcp_types::Paged;
use biomcp_types::{PgxGet, PgxSearch};
use serde_json::json;
#[test]
fn explicit_sources_and_section_ownership_are_checked_before_io()
-> Result<(), Box<dyn std::error::Error>> {
    let search: PgxSearch = serde_json::from_value(
        json!({"gene":"DPYD","source":"cpic","cpic_level":"B/C","limit":2}),
    )?;
    assert_eq!(search.at_offset(2).gene(), search.gene());
    assert_eq!(search.at_offset(2).cpic_level(), search.cpic_level());
    for value in [
        json!({"gene":"DPYD"}),
        json!({"gene":"DPYD","source":"clinpgx"}),
        json!({"gene":"DPYD","source":"cpic","limit":0}),
        json!({"gene":"DPYD","source":"cpic","drug":"x"}),
    ] {
        assert!(serde_json::from_value::<PgxSearch>(value).is_err());
    }
    for source in ["cpic", "clinpgx"] {
        for section in [
            None,
            Some("recommendations"),
            Some("guidelines"),
            Some("diplotype"),
            Some("annotations"),
        ] {
            let mut q = json!({"gene":"TPMT","source":source});
            if let Some(s) = section {
                q["section"] = json!(s);
            }
            if section == Some("diplotype") {
                q["diplotype"] = json!("*1/*3A");
            }
            let valid = if source == "cpic" {
                section != Some("annotations")
            } else {
                section == Some("annotations")
            };
            assert_eq!(
                serde_json::from_value::<PgxGet>(q.clone()).is_ok(),
                valid,
                "{q}"
            );
        }
    }
    for value in [
        json!({"gene":"TPMT","source":"cpic","section":"diplotype"}),
        json!({"gene":"TPMT","source":"cpic","diplotype":"*1/*3A"}),
        json!({"gene":"TPMT","source":"cpic","section":"guidelines","drug_id":"RxNorm:10324"}),
        json!({"gene":"TPMT","source":"cpic","kind":"clinical"}),
        json!({"gene":"TPMT","source":"cpic","section_limit":null}),
    ] {
        assert!(serde_json::from_value::<PgxGet>(value).is_err());
    }
    let a: ClinPgxAccession = "PA166134421".parse()?;
    assert_eq!(a.to_string(), "PA166134421");
    assert_ne!(
        std::any::TypeId::of::<ClinPgxAccession>(),
        std::any::TypeId::of::<biodata::PaId>()
    );
    for invalid in [
        "PA",
        "PA0",
        "PA-1",
        "pa166134421",
        " PA166134421",
        "clinpgx:PA166134421",
    ] {
        assert!(invalid.parse::<ClinPgxAccession>().is_err());
    }
    Ok(())
}

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(
    value: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(serde_json::from_value::<T>(value)?)
}
fn recommendation() -> serde_json::Value {
    json!(
        {"recommendationid":{"namespace":"CPIC","id":42},"lookupkey":{"TPMT":"synthetic first","NUDT15":"synthetic second"},"drugname":"synthetic drug",
        "guidelinename":"synthetic guide","guidelineurl":"https://example.invalid/guide","implications":{"TPMT":"synthetic implication"},
        "drugrecommendation":"  Synthetic source text.\n","classification":"synthetic classification",
        "phenotypes":{"TPMT":"synthetic phenotype"},"activityscore":{"TPMT":"n/a","NUDT15":"0.5"},"population":"synthetic population",
        "comments":null}
    )
}
fn diplotype() -> serde_json::Value {
    json!(
        {"genesymbol":"TPMT","diplotype":"*3A/*1","function1":"synthetic","function2":"synthetic","activityvalue1":"n/a",
        "activityvalue2":"n/a","totalactivityscore":"n/a","description":"synthetic","diplotypekey":{"TPMT":{"*3A":1,"*1":1}},
        "generesult":"synthetic result","ehrpriority":"synthetic priority","consultationtext":"  Synthetic consultation.\n",
        "lookupkey":{"TPMT":"synthetic"}}
    )
}
fn annotation() -> serde_json::Value {
    json!(
        {"id":{"namespace":"ClinPGx","id":42},"accessionId":"PA1234","levelOfEvidence":{"id":{"namespace":"ClinPGx",
        "id":3},"resource":"synthetic","term":"synthetic evidence",
        "termId":"synthetic-term"},"location":{"id":{"namespace":"ClinPGx","id":7},"copyNumber":"synthetic",
        "diplotypes":[{"id":{"namespace":"ClinPGx","id":99}}],"displayName":"synthetic location",
        "genes":[{"objCls":"Gene","id":"PA1","symbol":"TPMT","name":"synthetic"}],"gpPosition":-1,"haplotypes":[{"objCls":"Haplotype",
        "id":"PA2","symbol":"*3A","name":"synthetic"}],"linkedObjects":[{"objCls":"Haplotype","id":"PA2","symbol":"*3A",
        "name":"synthetic"}],"refSeqPosition":-1,"tagGene":false,"type":"synthetic"},"name":"synthetic annotation",
        "relatedChemicals":[{"objCls":"Chemical","id":"PA3","name":"synthetic first"},{"objCls":"Chemical","id":"PA4",
        "name":"synthetic second"}],"relatedChemicalsLogic":"synthetic relation",
        "types":["synthetic first","synthetic second"]}
    )
}
#[test]
fn native_records_keep_joint_context_source_text_and_literal_not_applicable()
-> Result<(), Box<dyn std::error::Error>> {
    use biodata::*;
    for value in [recommendation(), {
        let mut v = recommendation();
        v["comments"] = json!("  Synthetic comments.\n");
        v
    }] {
        let mut expected = value.clone();
        if expected
            .get("comments")
            .is_some_and(serde_json::Value::is_null)
        {
            expected.as_object_mut().ok_or("object")?.remove("comments");
        }
        assert_eq!(roundtrip::<PgxRecommendation>(value.clone())?, expected);
        assert!(serde_json::from_value::<Statement>(value).is_err());
    }
    let d = diplotype();
    assert_eq!(roundtrip::<PgxDiplotypeResult>(d.clone())?, d);
    assert!(serde_json::from_value::<Statement>(d).is_err());
    let mut table = recommendation();
    let object = table.as_object_mut().ok_or("object")?;
    for key in [
        "recommendationid",
        "drugname",
        "guidelinename",
        "guidelineurl",
    ] {
        object.remove(key);
    }
    object.extend(serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(json!(
        {"comments":"synthetic comments","id":{"namespace":"CPIC","id":91},"guidelineid":{"namespace":"CPIC",
        "id":3},"drugid":"RxNorm:10324","allelestatus":{},"version":4,
        "dosinginformation":true,"alternatedrugavailable":false,"otherprescribingguidance":false}
    ))?);
    assert_eq!(roundtrip::<PgxDrugRecommendation>(table.clone())?, table);
    assert!(serde_json::from_value::<PgxRecommendation>(table).is_err());
    let pair = json!(
        {"pairid":{"namespace":"CPIC","id":3},"drugid":"RxNorm:10324","drugname":"synthetic","genesymbol":"TPMT","guidelinename":null,"guidelineurl":null,
        "cpiclevel":"B/C","clinpgxlevel":"synthetic distinct level","pgxtesting":null,"pmids":null,"usedforrecommendation":"No",
        "provisional":false}
    );
    let mut expected = pair.clone();
    for name in ["guidelinename", "guidelineurl", "pgxtesting", "pmids"] {
        expected.as_object_mut().ok_or("object")?.remove(name);
    }
    assert_eq!(roundtrip::<PgxPair>(pair)?, expected);
    let guide = json!(
        {"guideline_name":"synthetic","guideline_url":"https://example.invalid/guide","drugs":["second","first","second"],
        "genes":[{"url":"https://example.invalid/gene","symbol":"TPMT"}]}
    );
    assert_eq!(roundtrip::<PgxGuidelineReference>(guide.clone())?, guide);
    Ok(())
}
#[test]
fn annotation_tiers_preserve_native_arrays_missing_locations_and_base_history()
-> Result<(), Box<dyn std::error::Error>> {
    use biodata::*;
    let header = annotation();
    assert_eq!(roundtrip::<PgxAnnotationHeader>(header.clone())?, header);
    let mut detail = header.clone();
    let object = detail.as_object_mut().ok_or("object")?;
    object.extend(serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(json!(
        {"allelePhenotypes":[{"id":{"namespace":"ClinPGx","id":1},"allele":"synthetic allele","limitedEvidence":false,"phenotype":"  Synthetic text.\n"}],
        "conflictingVariantAnnotationIds":[{"namespace":"ClinPGx","id":12},{"namespace":"ClinPGx",
        "id":13}],"history":[{"id":{"namespace":"ClinPGx","id":1},"date":"2026-01-01","type":"synthetic",
        "version":3},{"id":{"namespace":"ClinPGx","id":2},
        "date":"2026-02-01","type":"synthetic","version":4,"description":"synthetic change"}],"objCls":"ClinicalAnnotation",
        "overrideLevel":false,"pediatric":false,"relatedAlleles":[{"objCls":"Haplotype","id":"PA2","name":"synthetic","symbol":"*1"}],
        "score":1.5,"scoreDetails":"synthetic details"}
    ))?);
    assert_eq!(roundtrip::<PgxAnnotationDetail>(detail.clone())?, detail);
    assert_eq!(roundtrip::<PgxAnnotationHeader>(detail)?, header);
    let reference = json!({"objCls":"GuidelineAnnotation","id":"PA5","name":"synthetic",});
    assert_eq!(
        roundtrip::<PgxAnnotationReference>(reference.clone())?,
        reference
    );
    Ok(())
}
#[test]
fn native_selectors_and_closed_sets_roundtrip_and_pages_keep_all_context()
-> Result<(), Box<dyn std::error::Error>> {
    use biomcp_types::*;
    for value in ["cpic", "clinpgx"] {
        assert_eq!(roundtrip::<PgxSource>(json!(value))?, value);
    }
    for value in ["recommendations", "guidelines", "diplotype", "annotations"] {
        assert_eq!(roundtrip::<PgxSection>(json!(value))?, value);
    }
    for value in ["clinical", "guideline", "label"] {
        assert_eq!(roundtrip::<PgxAnnotationKind>(json!(value))?, value);
        let q: PgxGet = serde_json::from_value(
            json!({"gene":"TPMT","source":"clinpgx","section":"annotations","kind":value,"section_limit":2}),
        )?;
        let next = q.at_section_offset(2)?;
        assert_eq!(next.kind(), q.kind());
        assert_eq!(next.gene(), q.gene());
        assert_eq!(next.source(), q.source());
        assert_eq!((next.section_limit(), next.section_offset()), (2, 2));
    }
    for value in ["source", "observed"] {
        assert_eq!(roundtrip::<PgxTotalKind>(json!(value))?, value);
    }
    for invalid in ["all", "pharmgkb", "unknown"] {
        assert!(serde_json::from_value::<PgxSource>(json!(invalid)).is_err());
        assert!(serde_json::from_value::<PgxSection>(json!(invalid)).is_err());
        assert!(serde_json::from_value::<PgxAnnotationKind>(json!(invalid)).is_err());
        assert!(serde_json::from_value::<PgxTotalKind>(json!(invalid)).is_err());
    }
    let q: PgxGet = serde_json::from_value(
        json!({"gene":"CYP2D6","source":"cpic","section":"recommendations","drug_id":"RxNorm:10324","section_limit":2}),
    )?;
    assert_eq!(q.at_section_offset(2)?.drug_id(), q.drug_id());
    let q: PgxGet = serde_json::from_value(
        json!({"gene":"TPMT","source":"cpic","section":"diplotype","diplotype":"*3A/*1"}),
    )?;
    assert_eq!(
        q.diplotype().map(biodata::PgxDiplotypeExpression::as_str),
        Some("*3A/*1")
    );
    let a: PgxAnnotationGet = serde_json::from_value(json!({"id":"PA1234","source":"clinpgx"}))?;
    assert_eq!(a.id().as_str(), "PA1234");
    assert_eq!(a.source(), PgxSource::Clinpgx);
    assert!(
        serde_json::from_value::<PgxAnnotationGet>(json!({"id":"PA1234","source":"cpic"})).is_err()
    );
    Ok(())
}
#[cfg(feature = "schema")]
fn envelope<T>(
    query: serde_json::Value,
    body: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: biomcp_types::ResponseBody + schemars::JsonSchema + serde::de::DeserializeOwned,
    T::Query: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let query = serde_json::from_value(query)?;
    let body = serde_json::from_value::<T>(body)?;
    let value = serde_json::to_value(biomcp_types::Envelope::new(
        biomcp_types::Status::Complete,
        query,
        Some(body),
    ))?;
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<biomcp_types::Envelope<T>>();
    let validator = jsonschema::draft202012::new(&serde_json::to_value(schema)?)?;
    assert!(validator.is_valid(&value), "{value}");
    assert_eq!(
        value.get("schema").and_then(serde_json::Value::as_str),
        Some(T::SCHEMA),
        "PGx replies must retain their versioned envelope contract"
    );
    Ok(())
}
#[cfg(feature = "schema")]
#[test]
fn all_consumer_envelopes_validate_with_independent_source_or_observed_totals()
-> Result<(), Box<dyn std::error::Error>> {
    use biomcp_types::*;
    let q = json!({"gene":"TPMT","source":"cpic"});
    let header = json!({"gene":"TPMT","source":"cpic"});
    let scope = json!({"offset":0,"returned":0,"total":0,"total_kind":"source"});
    envelope::<PgxPage>(q.clone(), json!({"header":header,"items":[],"page":scope}))?;
    envelope::<PgxCard>(
        q.clone(),
        json!(
            {"header":header,"identity":{"symbol":"TPMT","hgncid":"HGNC:12014","ncbiid":"7172","ensemblid":"ENSG00000137364",
            "clinpgxid":"PA356","lookupmethod":"synthetic method","version":42}}
        ),
    )?;
    envelope::<PgxGuidelines>(
        json!({"gene":"TPMT","source":"cpic","section":"guidelines"}),
        json!({"header":header,"items":[],"page":scope}),
    )?;
    envelope::<PgxRecommendations>(
        json!({"gene":"TPMT","source":"cpic","section":"recommendations"}),
        json!({"representation":"view","header":header,"items":[recommendation()],"page":{"offset":0,"returned":1,"total":2,"total_kind":"source"}}),
    )?;
    envelope::<PgxRecommendations>(
        json!({"gene":"TPMT","source":"cpic","section":"recommendations","drug_id":"RxNorm:10324"}),
        json!({"representation":"table","header":header,"items":[],"page":scope}),
    )?;
    envelope::<PgxDiplotype>(
        json!({"gene":"TPMT","source":"cpic","section":"diplotype","diplotype":"*3A/*1"}),
        json!({"header":header,"diplotypes":[diplotype(),diplotype()]}),
    )?;
    let clinical = annotation();
    let mut detail = clinical.clone();
    detail
        .as_object_mut()
        .ok_or("object")?
        .extend(serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(json!(
            {"allelePhenotypes":[],"conflictingVariantAnnotationIds":[],"history":[],"objCls":"ClinicalAnnotation","overrideLevel":false,
            "pediatric":false,"relatedAlleles":[],"score":0.0,"scoreDetails":"synthetic"}
        ))?);
    envelope::<PgxAnnotation>(
        json!({"id":"PA1234","source":"clinpgx"}),
        json!({"header":{"id":"PA1234","source":"clinpgx"},"annotation":detail}),
    )?;
    for kind in ["clinical", "guideline", "label"] {
        let items = if kind == "clinical" {
            json!([clinical])
        } else {
            json!([{"objCls":"synthetic","id":"PA5","name":"synthetic",}])
        };
        envelope::<PgxAnnotations>(
            json!({"gene":"TPMT","source":"clinpgx","section":"annotations","kind":kind}),
            json!(
                {"annotation_kind":kind,"header":{"gene":"TPMT","source":"clinpgx"},"items":items,"page":{"offset":0,"returned":1,"total":1,
                "total_kind":"observed"}}
            ),
        )?;
    }
    Ok(())
}

#[test]
fn declared_expressions_and_source_map_keys_keep_their_exact_spelling()
-> Result<(), Box<dyn std::error::Error>> {
    use biodata::{PgxDiplotypeExpression, PgxGeneKey};
    for spelling in ["*3A/*1", " *2x2/*4 ", "*1/*3+*6"] {
        let v: PgxDiplotypeExpression = spelling.parse()?;
        assert_eq!(v.as_str(), spelling);
        assert_eq!(v.to_string(), spelling);
        assert_eq!(
            roundtrip::<PgxDiplotypeExpression>(json!(spelling))?,
            spelling
        );
    }
    assert!(" ".parse::<PgxDiplotypeExpression>().is_err());
    let upper: PgxGeneKey = "TPMT".parse()?;
    let lower: PgxGeneKey = "tpmt".parse()?;
    assert_ne!(upper, lower);
    assert_eq!(upper.as_str(), "TPMT");
    assert_eq!(lower.to_string(), "tpmt");
    assert!("not a gene key".parse::<PgxGeneKey>().is_err());
    assert_eq!(roundtrip::<PgxGeneKey>(json!("TPMT"))?, "TPMT");
    Ok(())
}

#[test]
fn card_section_actions_recheck_source_context_and_keep_the_declared_gene()
-> Result<(), Box<dyn std::error::Error>> {
    use biomcp_types::*;
    let q: PgxSearch =
        serde_json::from_value(json!({"gene":"DPYD","source":"cpic","limit":2,"offset":3}))?;
    assert_eq!(q.source(), PgxSource::Cpic);
    assert_eq!((q.limit(), q.offset()), (2, 3));
    let card: PgxGet = serde_json::from_value(json!({"gene":"DPYD","source":"cpic"}))?;
    let annotations = card.with_section(PgxSection::Annotations, PgxSource::Clinpgx)?;
    assert_eq!(annotations.gene(), card.gene());
    assert_eq!(annotations.section(), Some(PgxSection::Annotations));
    assert_eq!(annotations.kind(), Some(PgxAnnotationKind::Clinical));
    assert_eq!(annotations.section_limit(), 3);
    assert!(card.with_snapshot(biodata::Text::new("retained")?).is_err());
    let retained = annotations.with_snapshot(biodata::Text::new("retained")?)?;
    assert_eq!(
        retained.snapshot().map(biodata::Text::as_str),
        Some("retained")
    );
    assert!(retained.restart().snapshot().is_none());
    let default_clinical: PgxGet =
        serde_json::from_value(json!({"gene":"DPYD","source":"clinpgx","section":"annotations"}))?;
    assert_eq!(default_clinical.section_limit(), 3);
    assert_eq!(card.section_limit(), 5);
    for limit in [json!(0), json!(null), json!(51)] {
        assert!(serde_json::from_value::<PgxGet>(json!({"gene":"DPYD","source":"clinpgx","section":"annotations","section_limit":limit})).is_err());
    }
    assert!(
        card.with_section(PgxSection::Annotations, PgxSource::Cpic)
            .is_err()
    );
    assert!(
        card.with_section(PgxSection::Diplotype, PgxSource::Cpic)
            .is_err()
    );
    assert!(card.at_section_offset(1).is_err());
    for v in [
        json!({"gene":"DPYD","source":"cpic","limit":51}),
        json!({"gene":"DPYD","source":"cpic","limit":null}),
        json!({"gene":"DPYD","source":"cpic","offset":-1}),
    ] {
        assert!(serde_json::from_value::<PgxSearch>(v).is_err());
    }
    assert!(
        serde_json::from_value::<PgxGet>(
            json!({"gene":"DPYD","source":"cpic","section":"recommendations","drug_id":"tamoxifen"})
        )
        .is_err()
    );
    Ok(())
}
