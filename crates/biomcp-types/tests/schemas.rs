//! Public schema contract tests; file I/O stays outside the pure library.
#![cfg(feature = "schema")]
#[path = "schemas/annotations.rs"]
mod annotations;
#[path = "schemas/compatibility.rs"]
mod compatibility;
#[path = "schemas/compatibility_cases.rs"]
mod compatibility_cases;
#[path = "schemas/compatibility_recursive_cases.rs"]
mod compatibility_recursive_cases;
#[path = "schemas/drug.rs"]
mod drug;
#[path = "schemas/generation.rs"]
mod generation;
#[path = "schemas/ownership.rs"]
mod ownership;
#[path = "schemas/policy.rs"]
mod policy;
#[path = "schemas/release.rs"]
mod release;
#[path = "schemas/temporal.rs"]
mod temporal;
#[test]
fn temporal_properties_have_one_type_across_registered_schemas() -> Result<(), String> {
    temporal::temporal_properties_have_one_type_across_registered_schemas()
}
#[test]
fn temporal_property_check_rejects_conflicting_definition_types() -> Result<(), String> {
    temporal::temporal_property_check_rejects_conflicting_definition_types()
}
#[path = "schemas/values.rs"]
mod values;

use biodata::*;
use biomcp_types::*;
use schemars::{JsonSchema, generate::SchemaSettings};
use serde_json::Value;
use std::{fs, path::Path};

struct Contract {
    kind: crate::policy::ContractKind,
    name: &'static str,
    id: &'static str,
    rust_type: &'static str,
    generate: fn() -> Value,
}

#[expect(
    clippy::expect_used,
    reason = "Schema test setup must serialize a generated schema"
)]
fn generate<T: JsonSchema>() -> Value {
    serde_json::to_value(
        SchemaSettings::draft2020_12()
            .into_generator()
            .into_root_schema_for::<T>(),
    )
    .expect("schema serializes")
}

macro_rules! contracts {
        ($($ty:ty => ($name:literal, $id:expr $(, $kind:ident)?)),* $(,)?) => {
            vec![$(Contract {
                kind: contracts!(@kind $($kind)?),
                name: $name,
                id: $id,
                rust_type: std::any::type_name::<$ty>(),
                generate: contracts!(@generate $ty $(, $kind)?),
            }),*]
        };
        (@generate $ty:ty, query) => { crate::generation::query::<$ty> };
        (@generate $ty:ty) => { crate::generation::reply::<$ty> };
        (@kind query) => { crate::policy::ContractKind::Query };
        (@kind) => { crate::policy::ContractKind::Reply };
    }

macro_rules! reply_contracts {
    ($($method:ident: $body:ty => $name:literal),+ $(,)?) => {
        vec![$(Contract {
            kind: policy::ContractKind::Reply,
            name: $name,
            id: <$body as ResponseBody>::SCHEMA,
            rust_type: std::any::type_name::<Envelope<$body>>(),
            generate: generation::reply::<Envelope<$body>>,
        }),+]
    };
}

#[path = "schemas/commands.rs"]
mod commands;

#[path = "schemas/disease.rs"]
mod disease;

#[path = "schemas/registry.rs"]
mod registry;
use registry::{common_registry, research_registry};

fn article_registry() -> Vec<Contract> {
    contracts! {
        Article => ("article", "biomcp.article/4"),
        ArticleHeader => ("article-header", "biomcp.article-header/5"),
        EuropePmcId => ("europepmc-id", "biodata.europepmc-id/2"),
        Pmid => ("pmid", "biodata.pmid/2"),
        Pmcid => ("pmcid", "biodata.pmcid/2"),
        Doi => ("doi", "biodata.doi/2"),
    }
}

fn registry() -> Vec<Contract> {
    let mut contracts = contracts! {
        HttpHealth => ("http-health", "biomcp.http-health/2"),
        Markdown => ("markdown", "biodata.markdown/2"),
        ErrorEnvelope => ("error", "biomcp.error/4"),
        CellLineSearch => ("cell-line-search-query", "biomcp.cell-line-search-query/2", query),
        Request => ("request", "biomcp.request/32", query),
        HgncId => ("hgnc-id", "biodata.hgnc-id/2"),
        PartialDate => ("partial-date", "biodata.partial-date/2"),
        Position1 => ("position1", "biodata.position1/1"),
        Span0 => ("span0", "biodata.span0/1"),
        Text => ("text", "biodata.text/2"),
        Source => ("source", "biomcp.source/1"),
        Reason => ("reason", "biomcp.reason/2"),
        ProblemCode => ("problem-code", "biomcp.problem-code/1"),
        Problem => ("problem", "biomcp.problem/2"),
        GeneInput => ("gene-input", "biomcp.gene-input/2", query),
        Gene => ("gene", "biodata.gene/7"),
        GeneHeader => ("gene-header", "biomcp.gene-header/6"),
        GeneResolution => ("gene-resolution", "biomcp.gene-resolution/7"),
        NcbiGeneId => ("ncbi-gene-id", "biodata.ncbi-gene-id/2"),
        EnsemblGeneId => ("ensembl-gene-id", "biodata.ensembl-gene-id/2"),
        GeneSymbol => ("gene-symbol", "biodata.gene-symbol/2"),
        UniProtAcc => ("uniprot-acc", "biodata.uniprot-acc/1"),
        OmimId => ("omim-id", "biodata.omim-id/1"),
        AccessionBase => ("accession-base", "biodata.accession-base/2"),
        SourceGeneRef => ("gene-ref", "biodata.gene-ref/3"),
        Cytoband => ("cytoband", "biodata.cytoband/1"),
        GeneLocation => ("gene-location", "biodata.gene-location/1"),
        GenomicInterval => ("genomic-interval", "biodata.genomic-interval/2"),
        GeneStatus => ("gene-status", "biodata.gene-status/1"),
        LocusGroup => ("locus-group", "biodata.locus-group/1"),
        LocusType => ("locus-type", "biodata.locus-type/1"),
        MatchKind => ("match-kind", "biodata.match-kind/1"),
        Match => ("gene-match", "biomcp.gene-match/2"),
        Mane => ("mane", "biodata.mane/1"),
        Summary => ("gene-summary", "biodata.gene-summary/3"),
        SummarySource => ("summary-source", "biodata.summary-source/1"),
        HgncStatus => ("hgnc-status", "biodata.hgnc-status/2"),
        Build => ("build", "biodata.build/2"),
        Strand => ("strand", "biodata.strand/1"),
    };
    contracts.extend(common_registry());
    contracts.extend(article_registry());
    contracts.extend(research_registry());
    contracts.extend(disease::registry());
    contracts.extend(variant_contracts());
    contracts.extend(trial_contracts());
    contracts.extend(drug_contracts());
    contracts.extend(commands::registry());
    contracts.extend(biomcp_types::reply_types!(reply_contracts));
    contracts
}

fn variant_contracts() -> Vec<Contract> {
    contracts! {
        PopulationFrequency => ("population-frequency", "biodata.population-frequency/8"),
        Statement => ("statement", "biodata.statement/7"),
    }
}

fn drug_contracts() -> Vec<Contract> {
    contracts! {
        DrugId => ("drug-id", "biodata.drug-id/2"),
        DrugIdentityLevel => ("drug-identity-level", "biodata.drug-identity-level/1"),
        DrugIdentity => ("drug-identity", "biodata.drug-identity/4"),
        DrugIdentityRelation => ("drug-identity-relation", "biodata.drug-identity-relation/5"),
        DrugProduct => ("drug-product", "biodata.drug-product/5"),
        DrugLabelReference => ("drug-label-reference", "biodata.drug-label-reference/4"),
        DrugLabelSection => ("drug-label-section", "biodata.drug-label-section/4"),
        DrugNativeTarget => ("drug-native-target", "biodata.drug-native-target/3"),
        DrugDiseaseReference => ("drug-disease-reference", "biodata.drug-disease-reference/3"),
        DrugStage => ("drug-stage", "biodata.drug-stage/1"),
        DrugNativeReference => ("drug-native-reference", "biodata.drug-native-reference/2"),
        DrugTargetRelation => ("drug-target-relation", "biodata.drug-target-relation/7"),
        DrugIndicationClaim => ("drug-indication-claim", "biodata.drug-indication-claim/7"),
        DrugInteractionClaim => ("drug-interaction-claim", "biodata.drug-interaction-claim/7"),
        AuthorizationDate => ("authorization-date", "biodata.authorization-date/3"),
        AuthorizationEvent => ("authorization-event", "biodata.authorization-event/4"),
        DrugNameSelector => ("drug-name-selector", "biomcp.drug-name-selector/1", query),
        DrugWhoRow => ("drug-who-row", "biomcp.drug-who-row/2"),
        DrugSource => ("drug-source", "biomcp.drug-source/1"),
        DrugSection => ("drug-section", "biomcp.drug-section/1"),
        DrugHeader => ("drug-header", "biomcp.drug-header/3"),
        DrugRecallHeader => ("drug-recall-header", "biomcp.drug-recall-header/2"),
    }
}

fn trial_contracts() -> Vec<Contract> {
    contracts! {
        TrialConditionResolution => ("trial-condition-resolution", "biomcp.trial-condition-resolution/4"),
        NctId => ("nct-id", "biodata.nct-id/1"),
        TrialHeader => ("trial-header", "biomcp.trial-header/4"),
    }
}

#[test]
fn checked_in_schema_definition_names_have_one_body() -> Result<(), Box<dyn std::error::Error>> {
    release::checked_in_schema_definition_names_have_one_body()
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "Schema contract tests own file and environment I/O outside the pure library"
)]
fn checked_in_schemas_match_types() {
    let update = std::env::var_os("UPDATE_SCHEMAS").is_some_and(|value| value == "1");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let reviews = policy::read_reviews().expect("read reviewed schema transitions");
    // Collect and check the whole registry before writing any contract.
    let pending: Vec<_> = registry()
        .into_iter()
        .map(|contract| {
            let mut new = (contract.generate)();
            new["$id"] = contract.id.into();
            generation::command_descriptions(&mut new, contract.id);
            compatibility::check_reviewed(&new, &new, contract.kind, &reviews)
                .unwrap_or_else(|error| panic!("{}: {error}", contract.name));
            let path = root.join(format!("{}.json", contract.name));
            if path.exists() {
                let old: Value =
                    serde_json::from_str(&fs::read_to_string(&path).expect("read contract"))
                        .expect("parse contract");
                compatibility::check_reviewed(&old, &new, contract.kind, &reviews)
                    .unwrap_or_else(|error| panic!("{}: {error}", contract.name));
            } else {
                assert!(update, "missing {}", path.display());
            }
            let bytes = format!(
                "{}\n",
                serde_json::to_string_pretty(&new).expect("serialize schema")
            );
            (path, bytes)
        })
        .collect();
    if update {
        fs::create_dir_all(&root).expect("create schema directory");
    }
    for (path, bytes) in pending {
        if update {
            fs::write(&path, bytes).expect("write contract");
        } else {
            assert_eq!(
                fs::read_to_string(&path).expect("read contract"),
                bytes,
                "schema drift at {}; review compatibility before UPDATE_SCHEMAS=1",
                path.display()
            );
        }
    }
}

#[test]
fn consumed_queries_stay_inside_the_flat_command_grammar() {
    for schema in [
        generate::<BiomarkerSearch>(),
        generate::<BiomarkerGet>(),
        generate::<StudySearch>(),
        generate::<StudyGet>(),
        generate::<GeneGet>(),
        generate::<GeneSearch>(),
        generate::<ArticleSearch>(),
    ] {
        assert_eq!(schema["type"], "object");
        assert!(schema.get("oneOf").is_none());
        if schema.get("anyOf").is_some() {
            assert!(
                compatibility::selectors(&schema)
                    .expect("selector union")
                    .is_some()
            );
        }
        for property in schema["properties"]
            .as_object()
            .expect("query properties")
            .values()
        {
            flat_value(&schema, property);
        }
    }
}
#[expect(
    clippy::expect_used,
    reason = "The grammar contract must fail on unresolved generated schema references"
)]
fn flat_value(root: &Value, node: &Value) {
    if let Some(reference) = node.get("$ref").and_then(Value::as_str) {
        flat_value(
            root,
            root.pointer(reference.strip_prefix('#').expect("local reference"))
                .expect("resolved reference"),
        );
    } else if let Some(branches) = node.get("anyOf").and_then(Value::as_array) {
        assert_eq!(
            branches.len(),
            2,
            "Query schemas must stay inside the supported flat command grammar"
        );
        assert_eq!(
            branches
                .iter()
                .filter(|branch| branch["type"] == "null")
                .count(),
            1,
            "Query schemas must stay inside the supported flat command grammar"
        );
        for branch in branches.iter().filter(|branch| branch["type"] != "null") {
            flat_value(root, branch);
        }
    } else if node["type"] == "array" {
        let items = node.get("items").expect("array items");
        assert_ne!(
            items["type"], "array",
            "Query schemas must stay inside the supported flat command grammar"
        );
        flat_value(root, items);
    } else {
        assert!(
            ["string", "integer", "boolean"].contains(&node["type"].as_str().expect("scalar type")),
            "Query schemas must stay inside the supported flat command grammar"
        );
    }
}

#[test]
fn generated_schemas_preserve_release_contracts() -> Result<(), Box<dyn std::error::Error>> {
    release::generated_schemas_preserve_release_contracts()
}

#[test]
fn published_reply_contracts_are_open_and_query_choices_stay_closed() {
    for contract in registry() {
        let mut schema = (contract.generate)();
        schema["$id"] = contract.id.into();
        compatibility::check_reviewed(&schema, &schema, contract.kind, &policy::Reviews::default())
            .unwrap_or_else(|error| panic!("{}: {error}", contract.name));
        match contract.kind {
            policy::ContractKind::Reply => assert_open(&schema, contract.name),
            policy::ContractKind::Query => {
                assert!(
                    schema.get("x-extensible-enum").is_none(),
                    "{}",
                    contract.name
                );
            }
        }
    }
    for (schema, expected) in [
        (generation::reply::<biomcp_types::Source>(), "hgnc"),
        (generation::reply::<biomcp_types::Reason>(), "not-found"),
        (generation::reply::<biomcp_types::DrugSource>(), "chembl"),
        (generation::reply::<biodata::Proposition>(), "pathogenicity"),
        (generation::reply::<biodata::Predicate>(), "is_causal_for"),
    ] {
        assert!(schema.get("enum").is_none(), "reply enum remains closed");
        assert!(
            schema["x-extensible-enum"]
                .as_array()
                .unwrap()
                .contains(&expected.into())
        );
        let validator = jsonschema::draft202012::new(&schema).unwrap();
        assert!(validator.is_valid(&serde_json::json!("future_value")));
        assert!(!validator.is_valid(&serde_json::json!(42)));
    }
    let drug = generation::query::<biomcp_types::DrugGet>();
    let validator = jsonschema::draft202012::new(&drug).unwrap();
    assert!(validator.is_valid(&serde_json::json!({"input":"CHEMBL941","source":"chembl"})));
    assert!(
        !validator.is_valid(&serde_json::json!({"input":"CHEMBL941","source":"future_source"}))
    );
    let schema = generation::query::<biomcp_types::GeneGet>();
    assert_eq!(schema["additionalProperties"], false);
    let validator = jsonschema::draft202012::new(&schema).unwrap();
    assert!(validator.is_valid(&serde_json::json!({"input":["KRAS"]})));
    assert!(!validator.is_valid(&serde_json::json!({"input":["KRAS"],"future_field":true})));
}

fn assert_open(value: &Value, name: &str) {
    match value {
        Value::Object(object) => {
            assert_ne!(
                object.get("additionalProperties"),
                Some(&Value::Bool(false)),
                "{name}"
            );
            assert!(
                !object.contains_key("x-enum-open"),
                "unpublished marker in {name}"
            );
            for child in object.values() {
                assert_open(child, name);
            }
        }
        Value::Array(items) => {
            for child in items {
                assert_open(child, name);
            }
        }
        _ => {}
    }
}

#[test]
fn population_frequencies_share_one_bounded_fraction() {
    let schema = generate::<biodata::PopulationFrequency>();
    for field in ["allele_frequency", "faf95"] {
        assert_eq!(
            schema["properties"][field]["anyOf"][0]["$ref"], "#/$defs/Fraction",
            "{field} uses Fraction"
        );
    }
    assert_eq!(schema["$defs"]["Fraction"]["minimum"], 0.0);
    assert_eq!(schema["$defs"]["Fraction"]["maximum"], 1.0);
}

#[test]
fn source_forms_and_profile_members_match_admitted_subject_levels()
-> Result<(), Box<dyn std::error::Error>> {
    let schema = generate::<VariantRef>();
    let branches = schema["oneOf"].as_array().ok_or("reference branches")?;
    for (level, field, count) in [("source", "forms", 2), ("profile", "members", 3)] {
        let branch = branches
            .iter()
            .find(|branch| branch["properties"]["level"]["const"] == level)
            .ok_or("reference level")?;
        let items = branch["properties"][field]["items"]["oneOf"]
            .as_array()
            .ok_or("flat tagged item branches")?;
        assert_eq!(items.len(), count);
        for item in items {
            assert_eq!(item["type"], "object");
            assert_eq!(item["properties"]["level"]["type"], "string");
            assert!(item["properties"]["level"]["const"].is_string());
            assert!(
                item["required"]
                    .as_array()
                    .ok_or("required fields")?
                    .iter()
                    .any(|field| field == "level")
            );
            assert!(item.get("allOf").is_none());
            assert!(item.get("$ref").is_none());
        }
    }
    let validator = jsonschema::draft202012::new(&schema)?;
    let allele = serde_json::json!({"level":"allele", "id":"VCV000013961.3"});
    let protein =
        serde_json::json!({"level":"protein", "gene":{"label":"EGFR"}, "label":"source label"});
    let source = |forms| {
        serde_json::json!({
            "level":"source", "record":"civic:1", "feature":"native feature", "forms":forms
        })
    };
    let profile = |members| {
        serde_json::json!({
            "level":"profile", "record":"1", "label":"native profile", "members":members
        })
    };
    for (value, valid) in [
        (source(vec![allele.clone()]), true),
        (source(vec![protein.clone()]), true),
        (profile(vec![allele.clone(), protein, source(vec![])]), true),
        (source(vec![source(vec![])]), false),
        (source(vec![profile(vec![])]), false),
        (profile(vec![profile(vec![])]), false),
        (profile(vec![source(vec![source(vec![allele])])]), false),
    ] {
        assert_eq!(validator.is_valid(&value), valid, "schema: {value}");
        assert_eq!(
            serde_json::from_value::<VariantRef>(value.clone()).is_ok(),
            valid,
            "admission: {value}"
        );
    }
    Ok(())
}

#[test]
fn identifier_objects_use_namespace_and_id() -> Result<(), Box<dyn std::error::Error>> {
    release::identifier_objects_use_namespace_and_id()
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "Schema contracts read published files at the existing test I/O boundary"
)]
fn every_command_path_has_one_grammar_or_one_shared_rewrite()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas");
    let schemas = fs::read_dir(root)?
        .map(|entry| {
            let schema: Value = serde_json::from_str(&fs::read_to_string(entry?.path())?)?;
            Ok(schema
                .get("$id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned())
        })
        .collect::<Result<std::collections::BTreeSet<_>, Box<dyn std::error::Error>>>()?;
    commands::every_path_has_one_grammar_or_one_shared_rewrite(&schemas)
}

#[test]
fn section_rules_keep_the_complete_source_owned_prose() -> Result<(), String> {
    generation::section_rules_keep_the_complete_source_owned_prose()
}

#[test]
fn required_component_annotations_are_metadata_with_checked_sources() -> Result<(), String> {
    annotations::required_component_annotations_are_metadata_with_checked_sources()
}
