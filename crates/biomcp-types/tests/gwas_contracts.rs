//! Public GWAS value and request contracts before command registration.
use biodata::{GwasAssociationId, GwasStudyId, GwasTraitId, ScientificPValue};
use biomcp_types::Paged;
use biomcp_types::{GwasGet, GwasSearch};
use serde_json::json;

#[test]
fn identities_keep_native_namespaces_and_refuse_malformed_values() {
    for value in ["GCST000035", "GCST90000001"] {
        assert!(serde_json::from_value::<GwasStudyId>(json!(value)).is_ok());
    }
    for value in ["GCST", "gcst000035", "GCST-35", "GCST000000"] {
        assert!(serde_json::from_value::<GwasStudyId>(json!(value)).is_err());
    }
    assert!(serde_json::from_value::<GwasAssociationId>(json!("association:6110")).is_ok());
    assert!(serde_json::from_value::<GwasAssociationId>(json!("association:0")).is_err());
    for value in ["MONDO_0004989", "EFO_0000305", "HP:0001250"] {
        assert!(serde_json::from_value::<GwasTraitId>(json!(value)).is_ok());
    }
    assert!(serde_json::from_value::<GwasTraitId>(json!("0004989")).is_err());
}

#[test]
fn exact_probability_keeps_underflow_and_threshold_equality()
-> Result<(), Box<dyn std::error::Error>> {
    for value in ["2e-76", "2e-28539", "1.250e-300"] {
        let number: ScientificPValue = value.parse()?;
        let wire = serde_json::to_value(&number)?;
        let decoded: ScientificPValue = serde_json::from_value(wire)?;
        assert_eq!(decoded, number);
    }
    let tiny: ScientificPValue = "2e-28539".parse()?;
    let ordinary: ScientificPValue = "2e-76".parse()?;
    assert!(tiny < ordinary);
    assert!(ordinary <= "2e-76".parse()?);
    for value in ["0e-76", "-2e-76", "2e1", "NaN", "2e", "2"] {
        assert!(value.parse::<ScientificPValue>().is_err());
    }
    Ok(())
}

#[test]
fn selectors_and_sections_reject_invalid_requests_before_io()
-> Result<(), Box<dyn std::error::Error>> {
    for request in [
        json!({}),
        json!({"gene":"FGFR2","include_child_traits":true}),
        json!({"trait":"breast carcinoma","extended_geneset":true}),
        json!({"gene":"FGFR2","limit":0}),
        json!({"gene":"FGFR2","sort":"other"}),
        json!({"gene":"FGFR2","variant":null}),
        json!({"gene":"FGFR2","trait":null}),
    ] {
        assert!(serde_json::from_value::<GwasSearch>(request).is_err());
    }
    let search: GwasSearch = serde_json::from_value(json!({"trait":"breast carcinoma"}))?;
    assert_eq!((search.limit(), search.offset()), (3, 0));
    assert_eq!(search.at_offset(3).offset(), 3);
    let threshold: GwasSearch =
        serde_json::from_value(json!({"gene":"FGFR2","p_value":"2e-28539"}))?;
    assert_eq!(
        threshold.p_value().map(ToString::to_string).as_deref(),
        Some("2e-28539")
    );
    assert_eq!(
        serde_json::to_value(threshold)?.get("p_value"),
        Some(&json!("2e-28539"))
    );
    for request in [
        json!({"kind":"association","id":"association:6110","section":"ancestries"}),
        json!({"kind":"study","id":"GCST000035","section":"loci"}),
    ] {
        assert!(serde_json::from_value::<GwasGet>(request).is_err());
    }
    for request in [
        json!({"kind":"association","id":"association:6110","section":"loci"}),
        json!({"kind":"study","id":"GCST000035","section":"associations"}),
        json!({"kind":"study","id":"GCST000035","section":"ancestries"}),
    ] {
        assert!(serde_json::from_value::<GwasGet>(request).is_ok());
    }
    Ok(())
}

#[test]
fn one_association_record_preserves_arrays_unknown_frequency_and_effects()
-> Result<(), Box<dyn std::error::Error>> {
    let value = json!(
        {"id":"association:6110","study":"GCST000035","traits":["Whole synthetic trait","Second trait"],
        "ontology_traits":[{"namespace":"unknown","id":"MONDO_0004989",
        "label":"Native label"}],"alleles":[{"rs_id":"rs2981582","allele":"A"},{"rs_id":"rs1","allele":"T"}],"p_value":0.0,
        "exact_p_value":{"mantissa":"2","exponent":-28539},"odds_ratio":1.26,"beta":"source beta text","risk_frequency":"NR",
        "publication":"17529967","author":"Native author","mapped_genes":["FGFR2","intergenic"],"locations":["10:123"],"ci_lower":1.1,
        "ci_upper":1.4}
    );
    let row: biomcp_types::GwasAssociation = serde_json::from_value(value.clone())?;
    assert_eq!(serde_json::to_value(&row)?, value);
    let card: biomcp_types::GwasAssociationCard =
        serde_json::from_value(json!({"association":value.clone()}))?;
    assert_eq!(card.association, row);
    let page: biomcp_types::GwasPage = serde_json::from_value(
        json!({"items":[value],"page":{"returned":1,"offset":0,"total":1}}),
    )?;
    assert_eq!(page.items.first(), Some(&row));
    Ok(())
}

#[cfg(feature = "schema")]
#[test]
fn consumer_schemas_are_standalone_and_keep_exact_probability_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    use biomcp_types::*;
    use schemars::schema_for;
    for schema in [
        schema_for!(Envelope<GwasPage>),
        schema_for!(Envelope<GwasAssociationCard>),
        schema_for!(Envelope<GwasLoci>),
        schema_for!(Envelope<GwasStudy>),
        schema_for!(Envelope<GwasAncestries>),
    ] {
        jsonschema::draft202012::new(&serde_json::to_value(schema)?)?;
    }
    let schema = serde_json::to_value(schema_for!(ScientificPValue))?;
    let validator = jsonschema::draft202012::new(&schema)?;
    assert!(validator.is_valid(&json!({"mantissa":"2","exponent":-28539})));
    assert!(!validator.is_valid(&json!({"mantissa":"0","exponent":-76})));
    Ok(())
}

#[cfg(feature = "schema")]
fn envelope<T>(
    query: serde_json::Value,
    body: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: biomcp_types::ResponseBody + serde::de::DeserializeOwned + schemars::JsonSchema,
    T::Query: serde::de::DeserializeOwned + schemars::JsonSchema,
{
    let value = serde_json::to_value(biomcp_types::Envelope::<T>::new(
        biomcp_types::Status::Complete,
        serde_json::from_value(query)?,
        Some(serde_json::from_value(body)?),
    ))?;
    let schema = serde_json::to_value(schemars::schema_for!(biomcp_types::Envelope<T>))?;
    let validator = jsonschema::draft202012::new(&schema)?;
    assert!(validator.is_valid(&value), "{value}");
    Ok(())
}

#[cfg(feature = "schema")]
#[test]
fn native_relation_scopes_have_valid_consumer_envelopes() -> Result<(), Box<dyn std::error::Error>>
{
    use biomcp_types::*;
    let study = json!({"kind":"study","id":"GCST000035"});
    let association = json!({"kind":"association","id":"association:6110"});
    envelope::<GwasPage>(
        json!({"operation":"search","query":{"gene":"FGFR2"}}),
        json!({"items":[],"page":{"offset":0,"returned":0,"total":0}}),
    )?;
    envelope::<GwasPage>(
        json!({"operation":"get","query":{"kind":"study","id":"GCST000035","section":"associations"}}),
        json!({"items":[],"page":{"offset":0,"returned":0,"total":0}}),
    )?;
    envelope::<GwasLoci>(
        association,
        json!(
            {"association":"association:6110","items":[{"description":"Native description","strongest_risk_alleles":["rs2981582-A"],
            "author_reported_genes":[{"name":"FGFR2","ensembl_ids":["ENSG00000066468"],"entrez_ids":["2263"]}]}]}
        ),
    )?;
    envelope::<GwasStudy>(
        study.clone(),
        json!(
            {"id":"GCST000035","reported_traits":["Synthetic source label"],"ontology_traits":[],"initial_sample_description":"Discovery prose",
            "replication_sample_description":"Replication prose","discovery_ancestries":["Source discovery count"],
            "replication_ancestries":["Independent replication count"],"genotyping_technologies":[],"summary_statistics_available":false}
        ),
    )?;
    envelope::<GwasAncestries>(
        study,
        json!(
            {"study":"GCST000035","items":[{"stage":"replication","number_of_individuals":13475,"ancestry_groups":["Native label"],
            "country_of_origin":[{"country":"NR"}],"country_of_recruitment":[{"country":"Source country","major_area":"Native area",
            "region":"Native region"}],"source_url":"https://www.ebi.ac.uk/gwas/rest/api/v2/studies/GCST000035/ancestries"}]}
        ),
    )?;
    Ok(())
}
