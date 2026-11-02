//! Consumer checks for the reviewed shared value changes.
use biodata::{AccessionBase, RcvId, ScvId, VcvId};
use std::collections::HashSet;

#[test]
fn assertion_selectors_preserve_native_wire_text() -> Result<(), Box<dyn std::error::Error>> {
    for (input, source, selector) in [
        ("clinvar:13961", "clinvar", "VCV000013961"),
        ("clinvar:13961", "clinvar", "RCV000014992"),
        ("clinvar:13961", "clinvar", "SCV000001234"),
        ("CA000434", "erepo", "PCER000001171"),
    ] {
        let wire = serde_json::json!({
            "input":input,"source":source,"assertion":selector,"offset":0
        });
        let query: biomcp_types::VariantAssertionsGet = serde_json::from_value(wire.clone())?;
        assert_eq!(
            query.assertion().ok_or("missing selector")?.to_string(),
            selector
        );
        assert_eq!(serde_json::to_value(query)?, wire);
        let mut invalid = wire;
        invalid["assertion"] = serde_json::json!(format!("{selector}.1"));
        assert!(serde_json::from_value::<biomcp_types::VariantAssertionsGet>(invalid).is_err());
    }
    Ok(())
}

#[test]
fn clinvar_versions_preserve_distinct_identity() -> Result<(), Box<dyn std::error::Error>> {
    macro_rules! versions {
        ($ty:ty, $prefix:literal) => {{
            let first: $ty = concat!($prefix, "000000001.1").parse()?;
            let second: $ty = concat!($prefix, "000000001.2").parse()?;
            assert_ne!(first, second, "ClinVar versions identify distinct records");
            assert!(
                first.same_accession(second),
                "the named accession comparison ignores version"
            );
            assert_eq!(
                HashSet::from([first, second]).len(),
                2,
                "hashing retains both versions"
            );
        }};
    }
    versions!(VcvId, "VCV");
    versions!(RcvId, "RCV");
    versions!(ScvId, "SCV");
    Ok(())
}

#[test]
fn accessions_write_canonical_variable_width_numbers() -> Result<(), Box<dyn std::error::Error>> {
    for (input, output) in [("LRG_0001t01", "LRG_1t1"), ("NM_000546", "NM_000546")] {
        let accession: AccessionBase = input.parse()?;
        assert_eq!(
            accession.to_string(),
            output,
            "the accession scheme controls digit width"
        );
        assert_eq!(
            serde_json::to_string(&accession)?,
            format!("\"{output}\""),
            "JSON uses canonical output"
        );
    }
    Ok(())
}

#[test]
fn consumer_parser_errors_keep_existing_codes() {
    for (input, code) in [("HGNC:0", "id.hgnc.invalid"), ("0", "id.ncbi_gene.invalid")] {
        let error = input
            .parse::<biomcp_types::GeneInput>()
            .expect_err("invalid gene selector");
        assert_eq!(
            error.to_string(),
            code,
            "selector retains its namespace error"
        );
    }
    let article = "0"
        .parse::<biomcp_types::ArticleInput>()
        .expect_err("invalid PMID");
    assert_eq!(
        article.to_string(),
        "id.article.invalid",
        "article error remains stable"
    );
    for value in [f64::NAN, f64::INFINITY, -1.0] {
        let error = biomcp_types::RnaAbundance::try_from(value).expect_err("invalid abundance");
        assert_eq!(
            error.to_string(),
            "number.value.non_finite",
            "abundance error remains stable"
        );
    }
    let invalid = "2024-02-30T12:00:00";
    let local = invalid
        .parse::<biomcp_types::SourceLocalTimestampValue>()
        .expect_err("invalid local date");
    let timestamp = format!("{invalid}Z")
        .parse::<biomcp_types::SourceTimestampValue>()
        .expect_err("invalid date");
    assert_eq!(
        local.to_string(),
        "date.invalid",
        "local date error remains stable"
    );
    assert_eq!(
        timestamp.to_string(),
        "date.invalid",
        "date error remains stable"
    );
}
