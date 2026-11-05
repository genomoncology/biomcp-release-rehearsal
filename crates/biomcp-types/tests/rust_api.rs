//! Public Rust consumers read typed values without serializing them to JSON.
use biodata::*;
use biomcp_types::{Reason, Source, SourceHealth, SourceOutcome, SourceStatus, VariantHotspots};
use serde_json::json;
use std::{collections::HashSet, fmt::Debug, hash::Hash, str::FromStr};
type Check = Result<(), Box<dyn std::error::Error>>;

#[test]
fn variant_consumer_reads_coding_representations_statements_and_frequencies() -> Check {
    // A notation example only; this test asserts no molecular interpretation.
    let notation = "NM_005228.5:c.2573T>G";
    let variant = Variant {
        ids: vec!["VCV000013961.3".parse()?].into(),
        representations: vec![Representation {
            source: Text::new("synthetic")?,
            expression: Expression::new(
                ExpressionSyntax::HgvsC,
                Text::new(notation)?,
                Some(Text::new("21.1")?),
            )?,
            build: None,
            allele: None,
        }]
        .into(),
        genes: vec![SourceGeneRef::try_from(GeneSymbol::new("EGFR")?)?].into(),
    };
    let coding = variant
        .representations
        .iter()
        .find(|v| v.expression.syntax() == ExpressionSyntax::HgvsC)
        .expect("coding representation");
    assert_eq!(coding.expression.value().as_str(), notation);
    assert_eq!(
        coding.expression.syntax_version().map(Text::as_str),
        Some("21.1")
    );
    let subject = VariantRef::allele_hgvs(notation.parse()?);
    let attribution = Attribution {
        approved: None,
        published: None,
        source: Text::new("synthetic")?,
        record: None,
        version: None,
        release: None,
        submitter: None,
        url: None,
        evaluated: None,
    };
    let statement = Statement::new(
        subject.clone().into(),
        "pathogenicity".parse()?,
        Term {
            label: Text::new("source label")?,
            codings: Box::default(),
        },
        attribution.clone(),
    );
    assert_eq!(
        serde_json::to_value(&statement.subject)?,
        serde_json::to_value(&subject)?,
        "Subject conversion preserves the variant wire"
    );
    let frequency: PopulationFrequency = serde_json::from_value(json!({
        "subject": subject,
        "build": Build::GRCh38,
        "dataset": "synthetic",
        "dataset_version": "v1",
        "attribution": attribution,
        "allele_count": 1,
        "allele_number": 100,
        "allele_frequency": 0.01,
    }))?;
    assert_eq!(
        frequency.subject().hgvs().map(HgvsText::as_str),
        Some(notation)
    );
    assert_eq!(frequency.dataset().as_str(), "synthetic");
    assert_eq!(frequency.dataset_version().as_str(), "v1");
    assert_eq!(frequency.build(), Build::GRCh38);
    assert_eq!(frequency.allele_count(), Some(1));
    assert_eq!(frequency.allele_number(), Some(100));
    assert_eq!(frequency.allele_frequency().map(f64::from), Some(0.01));
    assert!(frequency.filters().is_empty());
    let coordinates = TrialCoordinates::new(40.0.try_into()?, (-80.0).try_into()?)?;
    assert_eq!(f64::from(coordinates.latitude()), 40.0);
    assert_eq!(f64::from(coordinates.longitude()), -80.0);
    Ok(())
}

#[test]
fn consumers_read_each_subject_level_without_json() -> Check {
    let gene = SourceGeneRef::try_from(GeneSymbol::new("EGFR")?)?;
    let allele = VariantRef::allele("VCV000013961.3".parse()?);
    let protein = VariantRef::protein_hgvs(
        gene.clone(),
        "NP_005219.2:p.?".parse()?,
        Some(Text::new("source label")?),
    );
    let profile = VariantRef::profile(
        Text::new("profile:1")?,
        Text::new("source profile")?,
        vec![allele, protein].into(),
    )?;
    assert_eq!(profile.record().map(Text::as_str), Some("profile:1"));
    assert_eq!(profile.label().map(Text::as_str), Some("source profile"));
    let members: Vec<_> = profile.members().collect();
    assert_eq!(
        members
            .first()
            .and_then(VariantRef::id)
            .map(ToString::to_string),
        Some("VCV000013961.3".into())
    );
    let protein = members.get(1).expect("protein member");
    assert_eq!(protein.gene(), Some(&gene));
    assert_eq!(
        protein.hgvs().map(HgvsText::as_str),
        Some("NP_005219.2:p.?")
    );
    assert_eq!(protein.label().map(Text::as_str), Some("source label"));
    assert!(protein.members().next().is_none());
    Ok(())
}

fn hashable<T: Eq + Hash + Debug + FromStr>(text: &str) -> Check
where
    T::Err: std::error::Error + 'static,
{
    let mut ids = HashSet::new();
    assert!(
        ids.insert(text.parse::<T>()?),
        "first {text} enters the collection"
    );
    assert!(
        ids.contains(&text.parse::<T>()?),
        "parsed {text} has consistent equality and hashing"
    );
    Ok(())
}

#[test]
fn identifiers_and_notation_support_hash_collections_and_borrowed_text() -> Check {
    hashable::<VariantId>("VCV000013961")?;
    hashable::<Accession>("NM_005228.5")?;
    hashable::<AccessionBase>("NM_005228")?;
    hashable::<HgvsText>("NM_005228.5:c.2573T>G")?;
    hashable::<Chromosome>("7")?;
    hashable::<RsId>("rs1")?;
    hashable::<CaId>("CA000001")?;
    hashable::<PaId>("PA1")?;
    hashable::<ClinVarVariationId>("clinvar:1")?;
    hashable::<ClinVarAlleleId>("1")?;
    hashable::<CosmicId>("COSV1")?;
    hashable::<VrsId>("ga4gh:VA.abcdefghijklmnopqrstuvwxyz012345")?;
    hashable::<Doi>("10.1/Example")?;
    hashable::<NctId>("NCT00000001")?;
    hashable::<Curie>("MONDO:0000001")?;
    hashable::<Pmid>("1")?;
    hashable::<Pmcid>("PMC1")?;
    hashable::<HgncId>("HGNC:3236")?;
    hashable::<NcbiGeneId>("1956")?;
    hashable::<EnsemblGeneId>("ENSG00000146648")?;
    hashable::<OmimId>("131550")?;
    hashable::<UniProtAcc>("P00533")?;
    hashable::<EuropePmcId>("MED1")?;
    hashable::<SemanticScholarId>("0123456789abcdef0123456789abcdef01234567")?;
    hashable::<TrialStatus>("RECRUITING")?;
    hashable::<TrialPhase>("PHASE1")?;
    hashable::<StudyType>("INTERVENTIONAL")?;
    assert_eq!("10.1/Example".parse::<Doi>()?.as_str(), "10.1/Example");
    for text in ["NCT00000001", "NCT99999999"] {
        let id: NctId = text.parse()?;
        assert_eq!(id.text(), text);
        assert_eq!(id.to_string(), text, "display retains the source spelling");
        assert_eq!(serde_json::to_value(id)?, json!(text));
    }
    assert_eq!("MONDO:0000001".parse::<Curie>()?.as_str(), "MONDO:0000001");
    assert!("unknown".parse::<TrialStatus>().is_err());
    assert_eq!(
        "RECRUITING".parse::<TrialStatus>()?.to_string(),
        "RECRUITING"
    );
    let mut expressions = HashSet::new();
    expressions.insert(Expression::hgvs(Text::new("NM_005228.5:c.2573T>G")?)?);
    assert_eq!(expressions.len(), 1);
    Ok(())
}

fn same_identity<T: Eq + Hash + Debug + FromStr + ToString + serde::Serialize>(
    prefix: &str,
) -> Check
where
    T::Err: std::error::Error + 'static,
{
    let base = format!("{prefix}000013961");
    let mut ids = HashSet::new();
    for suffix in ["", ".3", ".4"] {
        let text = format!("{base}{suffix}");
        let id: T = text.parse()?;
        assert_eq!(id.to_string(), text, "display retains the source spelling");
        assert_eq!(
            serde_json::to_value(&id)?,
            json!(text),
            "serialization retains the source version"
        );
        ids.insert(id);
    }
    assert_eq!(
        ids.len(),
        3,
        "source versions retain distinct identities for {prefix}"
    );
    assert!(
        !ids.contains(&format!("{prefix}000013962").parse::<T>()?),
        "different {prefix} records retain distinct identities"
    );
    Ok(())
}

#[test]
fn clinvar_accessions_compare_by_record_identity_and_preserve_source_versions() -> Check {
    same_identity::<VcvId>("VCV")?;
    same_identity::<RcvId>("RCV")?;
    same_identity::<ScvId>("SCV")?;
    assert_eq!(
        "VCV000013961.3".parse::<VcvId>()?.to_string(),
        "VCV000013961.3"
    );
    Ok(())
}

#[test]
fn source_identifiers_support_hash_collections() -> Check {
    hashable::<MondoId>("MONDO:0000001")?;
    hashable::<NcitId>("C1")?;
    hashable::<DrugId>("CHEMBL1")?;
    hashable::<ReportId>("001")?;
    hashable::<HpId>("HP:0000001")?;
    hashable::<GwasStudyId>("GCST000001")?;
    hashable::<GwasAssociationId>("association:1")?;
    hashable::<GwasTraitId>("EFO_0000001")?;
    hashable::<ClinPgxAccession>("PA1")?;
    hashable::<PgxGeneKey>("EGFR")?;
    hashable::<GtrTestId>("GTR000551465.2")?;
    hashable::<WhoProductId>("0123-456-00")?;
    hashable::<PmaNumber>("P000001")?;
    hashable::<PmaSupplement>("S001")?;
    hashable::<FdaDrugApplication>("NDA000001")?;
    hashable::<DiagnosticDeviceId>("P000001/S001")?;
    hashable::<PmaSupplementCell>("")?;
    hashable::<DeviceRecallNumber>("Z-0001-2026")?;
    hashable::<ReactomeId>("R-HSA-1")?;
    hashable::<WikiPathwaysId>("WP1")?;
    hashable::<PathwayId>("R-HSA-1")?;
    hashable::<UniProtIsoform>("P00533-1")?;
    hashable::<InterProId>("IPR000001")?;
    hashable::<ComplexPortalId>("CPX-1")?;
    hashable::<StudyId>("synthetic")?;
    hashable::<StudyProfileId>("synthetic_profile")?;
    hashable::<StudySampleListId>("synthetic_samples")?;
    hashable::<CbioportalCancerTypeId>("synthetic")?;
    hashable::<StudyResourceId>("synthetic_resource")?;
    hashable::<StudyBlobSha>("0123456789abcdef0123456789abcdef01234567")?;
    hashable::<CellosaurusAccession>("CVCL_0001")?;
    hashable::<CellLineSourceId>("synthetic:1")?;
    hashable::<ChemblCellId>("CHEMBL1")?;
    hashable::<SemanticScholarAuthorId>("1")?;
    hashable::<OrcidId>("0000-0002-1825-0097")?;
    hashable::<OrcidClientId>("APP-0123456789ABCDEF")?;
    hashable::<AuthorId>("orcid:0000-0002-1825-0097")?;
    Ok(())
}

// Feature: variant-profile-reference. Source profile values retain identities and relations.
fn member(id: u32, feature: &str) -> Result<VariantRef, Error> {
    VariantRef::source(
        format!("civic:{id}").parse()?,
        Text::new(feature)?,
        Box::default(),
    )
}
fn profile(source: &str, members: Vec<VariantRef>) -> Result<VariantRef, Error> {
    VariantRef::profile_expression(
        Text::new("1")?,
        Text::new(source)?,
        members.into(),
        ProfileExpression::civic(Text::new(source)?)?,
    )
}
#[test]
fn native_fusion_profile_does_not_require_an_allele() -> Check {
    let value = profile("#VID1", vec![member(1, "BCR::ABL1")?])?;
    let json = serde_json::to_value(&value)?;
    assert_eq!(
        json["members"][0],
        json!({"level":"source","record":"civic:1","feature":"BCR::ABL1"})
    );
    assert_eq!(json["expression"]["source"], "#VID1");
    assert!(value.members().all(|member| member.id().is_none()));
    assert_eq!(serde_json::from_value::<VariantRef>(json)?, value);
    Ok(())
}
#[test]
fn boolean_tree_retains_grouping_negation_and_exact_source_text() -> Check {
    let source = "#VID1 AND ( #VID2 OR NOT #VID3 )";
    let value = profile(
        source,
        vec![member(1, "A")?, member(2, "B")?, member(3, "C")?],
    )?;
    let expression = value.expression().unwrap();
    assert_eq!(expression.source().as_str(), source);
    let ProfileNode::And { operands } = expression.tree() else {
        panic!("AND root")
    };
    assert!(matches!(operands[1], ProfileNode::Or { .. }));
    assert_eq!(
        serde_json::to_value(expression)?["tree"]["operands"][1]["operands"][1]["operator"],
        "not"
    );
    Ok(())
}
#[test]
fn profile_refuses_missing_members_and_invalid_relations() -> Check {
    assert_eq!(
        profile("#VID1 AND #VID2", vec![member(1, "A")?]),
        Err(Error::ProfileMemberMissing)
    );
    assert_eq!(
        profile("#VID1", vec![member(1, "A")?, member(1, "A")?]),
        Err(Error::ProfileMemberDuplicate)
    );
    for source in [
        "#VID1 AND #VID2 OR #VID3",
        "NOT (#VID1)",
        "#VID1 AND",
        "(#VID1",
        "#VID0",
        "#VID01",
        "#VID1 tail",
    ] {
        assert!(
            ProfileExpression::civic(Text::new(source)?).is_err(),
            "{source}"
        );
    }
    Ok(())
}
#[test]
fn single_variant_json_keeps_its_existing_form() -> Check {
    let value = VariantRef::allele("VCV000013961.3".parse()?);
    assert_eq!(
        serde_json::to_value(value)?,
        json!({"level":"allele","id":"VCV000013961.3"})
    );
    Ok(())
}

#[test]
fn source_member_preserves_only_source_stated_leaf_forms() -> Check {
    let allele = VariantRef::allele("VCV000013961.3".parse()?);
    let source = VariantRef::source(
        "civic:1".parse()?,
        Text::new("feature")?,
        vec![allele.clone()].into(),
    )?;
    assert_eq!(source.forms().collect::<Vec<_>>(), vec![&allele]);
    assert!(source.id().is_none());
    assert!(
        VariantRef::source(
            "civic:2".parse()?,
            Text::new("feature")?,
            vec![source].into()
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn wire_expression_cannot_contradict_its_machine_source() {
    let value = json!({"source":"#VID1", "tree":{"operator":"member","record":"civic:2"}});
    assert!(serde_json::from_value::<ProfileExpression>(value).is_err());
}

#[test]
fn profile_keeps_additional_source_members_and_native_feature_identity() -> Check {
    let source = VariantRef::source_with_feature(
        "civic:1".parse()?,
        Text::new("exact feature")?,
        Some("civic:42".parse()?),
        Box::default(),
    )?;
    let value = profile("#VID1", vec![source, member(2, "other feature")?])?;
    let members = value.members().collect::<Vec<_>>();
    assert_eq!(members.len(), 2);
    assert_eq!(
        members[0].feature_record().map(|record| record.as_str()),
        Some("civic:42")
    );
    assert_eq!(
        serde_json::to_value(value)?["members"][0]["feature_record"],
        "civic:42"
    );
    Ok(())
}

#[test]
fn wire_expression_requires_authoritative_machine_source() {
    for source in ["unrelated label", "A AND B", "VID1", "civic:1"] {
        let value = json!({"source":source,"tree":{"operator":"member","record":"civic:1"}});
        assert!(
            serde_json::from_value::<ProfileExpression>(value).is_err(),
            "{source}"
        );
    }
}

#[test]
fn expression_depth_and_size_limits_match_on_construction_and_roundtrip() -> Check {
    for (depth, tail, valid) in [
        (64, "#VID1", true),
        (63, "NOT #VID1", true),
        (64, "NOT #VID1", false),
        (65, "#VID1", false),
    ] {
        let source = format!(
            "{}{}{}",
            "#VID1 AND (".repeat(depth),
            tail,
            ")".repeat(depth)
        );
        let expression = ProfileExpression::civic(Text::new(&source)?);
        assert_eq!(expression.is_ok(), valid, "depth={depth}, tail={tail}");
        if let Ok(expression) = expression {
            assert_eq!(
                serde_json::from_value::<ProfileExpression>(serde_json::to_value(&expression)?)?,
                expression
            );
        }
    }
    for (members, valid) in [(4096, true), (4097, false)] {
        let source = vec!["#VID1"; members].join(" AND ");
        let expression = ProfileExpression::civic(Text::new(&source)?);
        assert_eq!(expression.is_ok(), valid, "members={members}");
        if let Ok(expression) = expression {
            assert_eq!(
                serde_json::from_value::<ProfileExpression>(serde_json::to_value(&expression)?)?,
                expression
            );
        }
    }
    Ok(())
}

#[test]
fn profile_refuses_nested_native_haplotypes() -> Check {
    let compound = VariantRef::clinvar_haplotype(biodata::ClinvarHaplotype::new(
        "clinvar:441268".parse()?,
        "VCV000441268".parse()?,
        Text::new("5")?,
        Text::new("1")?,
        vec![biodata::ClinvarHaplotypeMember {
            id: "clinvar:17864".parse()?,
            allele_id: "32903".parse()?,
        }]
        .into(),
    )?)?;
    assert_eq!(
        VariantRef::profile(
            Text::new("profile")?,
            Text::new("native profile")?,
            vec![compound.clone()].into()
        ),
        Err(Error::VariantRefInvalid)
    );
    assert!(
        serde_json::from_value::<VariantRef>(json!({
            "level":"profile", "record":"profile", "label":"native profile", "members":[compound]
        }))
        .is_err()
    );
    Ok(())
}

#[test]
fn source_status_is_atomic_across_updates_clones_and_wire_input() -> Result<(), serde_json::Error> {
    let mut source = SourceOutcome::new(
        Source::FdaCdx,
        biomcp_types::Use::Open,
        "terms".into(),
        "https://example.org".into(),
        SourceStatus::Ok,
    );
    for status in [
        SourceStatus::Ok,
        SourceStatus::Stale(Some(Reason::Unavailable)),
        SourceStatus::Failed(Some(Reason::NoJoinId)),
        SourceStatus::Skipped(Some(Reason::Unsupported)),
        SourceStatus::Syncing,
    ] {
        source.set_status(status);
        let health = SourceHealth::new(
            Source::FdaCdx,
            "terms".parse().unwrap(),
            "https://example.org".parse().unwrap(),
            biomcp_types::Use::Open,
            false,
            status,
        );
        let health: SourceHealth = serde_json::from_value(serde_json::to_value(health)?)?;
        assert_eq!(
            (health.state, health.reason),
            (source.state, source.reason),
            "health probes retain the same canonical status"
        );
        let wire = serde_json::to_value(source.clone())?;
        let restored: SourceOutcome = serde_json::from_value(wire)?;
        assert_eq!(
            restored.status(),
            status,
            "valid source status survives serialization"
        );
    }
    source.set_status(SourceStatus::Ok);
    for reason in ["unsupported", "no-join-id"] {
        let mut wire = serde_json::to_value(&source)?;
        wire["reason"] = json!(reason);
        let mut health = wire.clone();
        health["disabled"] = json!(false);
        assert!(
            serde_json::from_value::<SourceHealth>(health).is_err(),
            "health answer cannot import reason {reason}"
        );
        assert!(
            serde_json::from_value::<SourceOutcome>(wire).is_err(),
            "current answer cannot import reason {reason}"
        );
    }
    Ok(())
}

#[test]
fn hotspot_data_cannot_import_a_whole_section_failure_reason() -> Result<(), serde_json::Error> {
    let wire = json!({"records":[],"page":{"offset":0,"returned":0,"total":0}});
    let data: VariantHotspots = serde_json::from_value(wire.clone())?;
    assert!(
        serde_json::to_value(data)?.get("reason").is_none(),
        "data has no failure reason"
    );
    let mut contradiction = wire;
    contradiction["reason"] = json!("no-join-id");
    assert!(
        serde_json::from_value::<VariantHotspots>(contradiction).is_err(),
        "hotspot data rejects a failure reason"
    );
    Ok(())
}
