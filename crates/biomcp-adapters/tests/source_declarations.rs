//! Source declaration contracts shared by every command interface.
use std::collections::BTreeSet;
#[test]
fn bundle_names_are_unique_and_keep_the_public_command_contract() {
    let names: Vec<_> = biomcp_adapters::decl::bundles()
        .map(|bundle| bundle.name())
        .collect();
    assert_eq!(
        names,
        [
            "hgnc",
            "clingen",
            "gencc",
            "wikipathways",
            "cbioportal-studies",
            "ema",
            "ddinter",
            "who-pq",
            "gtr",
            "who-ivd",
            "fda-cdx"
        ],
        "source declarations keep every public sync word"
    );
    assert_eq!(
        names.len(),
        names.iter().collect::<BTreeSet<_>>().len(),
        "each sync word selects exactly one bundle"
    );
}

use biomcp_adapters::quotas::{configure, policies};
use biomcp_fetch::PacingOverride;
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn every_registry_entry_records_primary_policy_or_bounded_absence() {
    let mut names = BTreeSet::new();
    for policy in policies() {
        assert!(
            names.insert(policy.quota.key().name()),
            "duplicate provider bucket"
        );
        let citation = policy
            .citation
            .trim_start()
            .strip_prefix("Checked ")
            .and_then(|text| text.split_once(": "));
        assert!(
            citation.is_some_and(|(date, text)| {
                date.parse::<biodata::PartialDate>()
                    .is_ok_and(|date| date.day().is_some())
                    && text.split_whitespace().next().is_some_and(|url| {
                        url.starts_with("https://")
                            && biomcp_fetch::check_annotation_url(url).is_ok()
                    })
            }),
            "missing dated primary citation: {policy:?}"
        );
        assert!(
            policy.citation.contains("Published") || policy.citation.contains("No "),
            "missing policy or recorded absence: {policy:?}"
        );
        let (requests, seconds) = policy.quota.rate().ratio();
        let input = serde_json::from_value(json!({"rate":{"requests":requests,"seconds":seconds}, "in_flight":policy.quota.in_flight()})).unwrap();
        assert!(configure(&BTreeMap::from([(policy.quota.key().name().into(), input)])).is_ok());
    }
    assert!(!names.is_empty());
}
#[test]
fn exact_rate_cross_products_refuse_above_ceiling_and_preserve_small_positive_rates() {
    for (source, value, accepted) in [
        ("gwas", json!({"rate":{"requests":100,"seconds":60}}), true),
        ("gwas", json!({"rate":{"requests":101,"seconds":60}}), false),
        ("hgnc", json!({"rate":{"requests":1,"seconds":65535}}), true),
        (
            "hgnc",
            json!({"rate":{"requests":65535,"seconds":65535}}),
            true,
        ),
        (
            "hgnc",
            json!({"rate":{"requests":65535,"seconds":65534}}),
            false,
        ),
        (
            "ncbi-eutils",
            json!({"rate":{"requests":3,"seconds":1},"keyed_rate":{"requests":10,"seconds":1}}),
            true,
        ),
        (
            "ncbi-eutils",
            json!({"keyed_rate":{"requests":11,"seconds":1}}),
            false,
        ),
        (
            "ncbi-eutils",
            json!({"keyed_rate":{"requests":0,"seconds":1}}),
            false,
        ),
        (
            "hgnc",
            json!({"keyed_rate":{"requests":1,"seconds":1}}),
            false,
        ),
        ("typo", json!({}), false),
    ] {
        let input: PacingOverride = serde_json::from_value(value).unwrap();
        assert_eq!(
            configure(&BTreeMap::from([(source.into(), input)])).is_ok(),
            accepted,
            "{source}: {input:?}"
        );
    }
}
