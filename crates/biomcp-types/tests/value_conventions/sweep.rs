use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use std::{fmt::Debug, str::FromStr};

// This inventory covers every public biodata Display + FromStr value, including
// macro-defined vocabularies. Structured records, numeric coordinates/numbers,
// Markdown and native enums lack that pair; their JSON contracts have owners in
// the existing record tests. SourceText has an infallible From boundary below.
// No schema-pattern or unknown-enum policy checks belong to this sweep.
type Check = Result<(), Box<dyn std::error::Error>>;

fn plain_value_contract<T>(valid: &[&str], invalid: &[&str]) -> Check
where
    T: FromStr + Serialize + DeserializeOwned + Eq + Debug + ToString,
    T::Err: std::error::Error + 'static,
{
    for text in valid {
        let value: T = text.parse()?;
        assert_eq!(
            value.to_string().parse::<T>()?,
            value,
            "display must parse to the same value: {text}"
        );
        let wire = serde_json::to_value(&value)?;
        if wire.is_string() {
            assert_eq!(
                serde_json::from_value::<T>(json!(text))?,
                value,
                "JSON input must use the public parser: {text}"
            );
        }
        assert_eq!(
            serde_json::from_value::<T>(wire)?,
            value,
            "JSON must preserve the value: {text}"
        );
    }
    for text in invalid {
        assert!(
            text.parse::<T>().is_err(),
            "parser must reject near miss: {text:?}"
        );
        // Some values use structured JSON; their checks are exercised separately.
        assert!(
            serde_json::from_value::<T>(json!(text)).is_err(),
            "JSON must reject near miss: {text:?}"
        );
    }
    Ok(())
}

fn equal_hash<T>(left: &str, right: &str) -> Check
where
    T: FromStr + Eq + Debug + std::hash::Hash,
    T::Err: std::error::Error + 'static,
{
    use std::hash::{DefaultHasher, Hasher};
    let left: T = left.parse()?;
    let right: T = right.parse()?;
    assert_eq!(left, right, "fixtures must represent equal values");
    let mut a = DefaultHasher::new();
    let mut b = DefaultHasher::new();
    left.hash(&mut a);
    right.hash(&mut b);
    assert_eq!(a.finish(), b.finish(), "equal values must hash equally");
    Ok(())
}

#[test]
fn identifiers_and_variant_text_preserve_identity() -> Check {
    value_contract::<biodata::HgncId>(&["HGNC:1", "HGNC:4294967295"], &["HGNC:01"])?;
    value_contract::<biodata::PartialDate>(&["2024", "2024-02", "2024-02-29"], &["2023-02-29"])?;
    constructor_contract(
        biodata::Text::new,
        &[" checked α ", "a\\u{0001}"],
        &["\u{0001}"],
    )?;
    value_contract::<biodata::NcbiGeneId>(&["1", "4294967295"], &["01"])?;
    constructor_contract(biodata::GeneSymbol::new, &["BRAF", "bRaF"], &["1BRAF"])?;
    constructor_contract(
        biodata::EnsemblGeneId::new,
        &["ENSG00000157764", "ENSG00000157764.2"],
        &["ENSG00000157764.0"],
    )?;
    value_contract::<biodata::OmimId>(&["100000", "699999"], &["099999", "700000"])?;
    constructor_contract(
        biodata::UniProtAcc::new,
        &["P15056", "A0A024RBG1"],
        &["P15056-1"],
    )?;
    constructor_contract(biodata::Cytoband::new, &["7q34", "Xp22.33"], &["7q3"])?;
    value_contract::<biodata::Pmid>(&["1"], &["01"])?;
    value_contract::<biodata::Pmcid>(&["PMC1"], &["PMC01"])?;
    value_contract::<biodata::Doi>(&["10.1000/ABC", "10.1000/abc"], &["10.1000/a b"])?;
    value_contract::<biodata::EuropePmcId>(&["MED1", "PMC1"], &["MED0"])?;
    value_contract::<biodata::SemanticScholarId>(
        &["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
        &["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
    )?;
    Ok(())
}

#[test]
fn reference_and_variant_text_preserve_identity() -> Check {
    value_contract::<biodata::Accession>(
        &[
            "NM_004333.6",
            "NC_000007.14",
            "ENST00000288602.11",
            "LRG_1t1",
        ],
        &["NM_004333.0"],
    )?;
    value_contract::<biodata::AccessionBase>(&["NM_004333"], &["NM_004333.6"])?;
    value_contract::<biodata::Chromosome>(&["1", "X", "MT"], &["23"])?;
    constructor_contract(biodata::Contig::new, &["chr7", "NC_000007.14"], &["chr 7"])?;
    value_contract::<biodata::Base>(&["A", "N"], &["a"])?;
    value_contract::<biodata::Sequence>(&["", "ACGTN"], &["ACGT!"])?;
    value_contract::<biodata::HgvsText>(
        &["NM_004333.6:c.1799T>A", "NP_004324.2:p.Val600Glu"],
        &["NM_004333.6:c."],
    )?;
    value_contract::<biodata::RsId>(&["rs1"], &["rs01"])?;
    value_contract::<biodata::CaId>(&["CA000001"], &["CA1"])?;
    value_contract::<biodata::PaId>(&["PA1"], &["PA01"])?;
    value_contract::<biodata::ClinVarVariationId>(&["clinvar:1"], &["clinvar:01"])?;
    value_contract::<biodata::ClinVarAlleleId>(&["1"], &["01"])?;
    value_contract::<biodata::VcvId>(&["VCV000000001", "VCV000000001.2"], &["VCV000000000"])?;
    value_contract::<biodata::RcvId>(&["RCV000000001.1"], &["RCV000000001.0"])?;
    value_contract::<biodata::ScvId>(&["SCV000000001.1"], &["SCV000000001.0"])?;
    value_contract::<biodata::CosmicId>(&["COSV1", "COSM1", "COSN1"], &["COSV01"])?;
    value_contract::<biodata::VrsId>(
        &["ga4gh:VA.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
        &["ga4gh:VA.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
    )?;
    value_contract::<biodata::VariantId>(
        &[
            "rs1",
            "CA000001",
            "PA1",
            "VCV000000001",
            "clinvar:1",
            "COSV1",
            "ga4gh:VA.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ],
        &["rs01"],
    )?;
    value_contract::<biodata::Curie>(&["HGNC:1097"], &["HGNC:"])?;
    value_contract::<biodata::NctId>(&["NCT00000001"], &["NCT0000001"])?;
    value_contract::<biodata::TrialUploadTime>(&["2024-02-29T23:59"], &["2023-02-29T23:59"])?;
    Ok(())
}

#[test]
fn source_identifiers_preserve_identity() -> Check {
    value_contract::<biodata::DrugId>(&["CHEMBL25", "NCIT:C1"], &["NCIT:"])?;
    value_contract::<biodata::ReportId>(&["0001"], &["1x"])?;
    value_contract::<biodata::ClinPgxAccession>(&["PA1"], &["PA01"])?;
    value_contract::<biodata::PgxGeneKey>(&["CYP2D6"], &["1CYP"])?;
    value_contract::<biodata::PgxDiplotypeExpression>(&[" *1/*2 "], &["  "])?;
    constructor_contract(biodata::MondoId::new, &["MONDO:0000001"], &["MONDO:000001"])?;
    constructor_contract(biodata::NcitId::new, &["C123"], &["C"])?;
    value_contract::<biodata::GwasStudyId>(&["GCST000001"], &["GCST000000"])?;
    value_contract::<biodata::GwasAssociationId>(&["association:1"], &["association:01"])?;
    value_contract::<biodata::GwasTraitId>(&["EFO_0000001", "HP:0000001"], &["EFO_"])?;
    value_contract::<biodata::ScientificPValue>(&["1.50e-8", "1.5e-8", "1e0"], &["1e1", "0e-8"])?;
    value_contract::<biodata::HpId>(&["HP:0000001"], &["HP:000001"])?;
    constructor_contract(biodata::ReactomeId::new, &["R-HSA-1"], &["R-HSA-01"])?;
    constructor_contract(biodata::WikiPathwaysId::new, &["WP1"], &["WP01"])?;
    value_contract::<biodata::PathwayId>(&["R-HSA-1", "WP1"], &["WP01"])?;
    constructor_contract(
        |value| biodata::UniProtIsoform::new(value).ok_or(biodata::UniProtIsoformError),
        &["P15056", "P15056-2"],
        &["P15056-0"],
    )?;
    constructor_contract(biodata::AminoAcids::new, &["ACDEXUO"], &["ACDE!"])?;
    constructor_contract(biodata::InterProId::new, &["IPR000001"], &["IPR00001"])?;
    constructor_contract(biodata::ComplexPortalId::new, &["CPX-1"], &["CPX-01"])?;
    constructor_contract(biodata::StudyId::new, &["study_1"], &["study/1"])?;
    constructor_contract(
        biodata::StudyProfileId::new,
        &["study_mutations"],
        &["study/mutations"],
    )?;
    constructor_contract(
        biodata::StudySampleListId::new,
        &["study_all"],
        &["study/all"],
    )?;
    constructor_contract(biodata::CbioportalCancerTypeId::new, &["brca"], &["brca/"])?;
    constructor_contract(
        biodata::StudyResourceId::new,
        &["resource_1"],
        &["resource/1"],
    )?;
    constructor_contract(
        biodata::StudyBlobSha::new,
        &["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
        &["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"],
    )?;
    constructor_contract(
        biodata::StudyImportTime::new,
        &["2024-02-29 23:59:59"],
        &["2023-02-29 23:59:59"],
    )?;
    constructor_contract(
        biodata::GtrTestId::new,
        &["GTR000000001.1"],
        &["GTR000000001.0"],
    )?;
    constructor_contract(
        biodata::WhoProductId::new,
        &["1234-123-12"],
        &["1234-12-12"],
    )?;
    constructor_contract(biodata::PmaNumber::new, &["P000001"], &["P00001"])?;
    constructor_contract(biodata::PmaSupplement::new, &["S001"], &["S000"])?;
    constructor_contract(
        biodata::FdaDrugApplication::new,
        &["NDA000001", "BLA000001"],
        &["NDA00001"],
    )?;
    constructor_contract(
        biodata::DiagnosticDeviceId::new,
        &["P000001/S001", "K000001", "DEN000001", "H000001"],
        &["P000001/S000"],
    )?;
    constructor_contract(biodata::PmaSupplementCell::new, &["", "S001"], &["S000"])?;
    constructor_contract(
        biodata::DeviceRecallNumber::new,
        &["Z-0001-2024"],
        &["Z-001-2024"],
    )?;
    constructor_contract(
        biodata::DrugRecallNumber::new,
        &["D-0001-2024"],
        &["D-001-2024"],
    )?;
    Ok(())
}

#[test]
fn author_and_record_vocabularies_preserve_identity() -> Check {
    constructor_contract(
        biodata::CellosaurusAccession::new,
        &["CVCL_0030"],
        &["CVCL_030"],
    )?;
    constructor_contract(biodata::CellLineSourceId::new, &["ATCC:HTB-26"], &["ATCC:"])?;
    constructor_contract(biodata::ChemblCellId::new, &["CHEMBL1"], &["CHEMBL"])?;
    value_contract::<biodata::SemanticScholarAuthorId>(&["1"], &["01"])?;
    value_contract::<biodata::OrcidId>(&["0000-0002-1825-0097"], &["0000-0002-1825-0098"])?;
    value_contract::<biodata::AuthorId>(
        &["orcid:0000-0002-1825-0097", "semantic-scholar:1"],
        &["orcid:0000-0002-1825-0098"],
    )?;
    value_contract::<biodata::OrcidClientId>(&["APP-0123456789ABCDEF"], &["APP-0123456789ABCDE"])?;
    value_contract::<biodata::OrcidPublicVisibility>(&["public", "PUBLIC"], &["private"])?;
    value_contract::<biodata::License>(&["cc_by"], &["cc_by!"])?;
    value_contract::<biodata::TrialOutcomeKind>(&["primary"], &["primary!"])?;
    value_contract::<biodata::TrialDateKind>(&["ACTUAL"], &["ACTUAL!"])?;
    value_contract::<biodata::TrialArmKind>(&["EXPERIMENTAL"], &["EXPERIMENTAL!"])?;
    value_contract::<biodata::TrialInterventionKind>(&["BEHAVIORAL"], &["BEHAVIORAL!"])?;
    value_contract::<biodata::TrialReferenceKind>(&["BACKGROUND"], &["BACKGROUND!"])?;
    value_contract::<biodata::TrialPhase>(&["EARLY_PHASE1"], &["EARLY_PHASE1!"])?;
    value_contract::<biodata::TrialStatus>(
        &["ACTIVE_NOT_RECRUITING"],
        &["ACTIVE_NOT_RECRUITING!"],
    )?;
    value_contract::<biodata::StudyType>(&["INTERVENTIONAL"], &["INTERVENTIONAL!"])?;
    value_contract::<biodata::TrialSex>(&["ALL"], &["ALL!"])?;
    value_contract::<biodata::TrialSiteStatus>(&["NOT_YET_RECRUITING"], &["NOT_YET_RECRUITING!"])?;
    value_contract::<biodata::TrialContactRole>(&["CONTACT"], &["CONTACT!"])?;
    value_contract::<biodata::TrialOfficialRole>(&["SUB_INVESTIGATOR"], &["SUB_INVESTIGATOR!"])?;
    value_contract::<biodata::PhenotypeSynonymClass>(&["exact"], &["exact!"])?;
    value_contract::<biodata::PhenotypeHierarchyRelation>(&["super_classes"], &["super_classes!"])?;
    value_contract::<biodata::Build>(&["GRCh37"], &["GRCh37!"])?;
    value_contract::<biodata::Strand>(&["+"], &["+!"])?;
    value_contract::<biodata::PathwaySpecies>(&["Homo sapiens"], &["Homo sapiens!"])?;
    value_contract::<biodata::GtrTestType>(&["Clinical"], &["Clinical!"])?;
    value_contract::<biodata::GtrMethodList>(&["method_categories"], &["method_categories!"])?;
    value_contract::<biodata::DrugIdentityLevel>(&["ingredient"], &["ingredient!"])?;
    value_contract::<biodata::StudyFileKind>(&["file"], &["file!"])?;
    value_contract::<biodata::AuthorCountScope>(&["provider_record"], &["provider_record!"])?;
    value_contract::<biodata::ManeStatus>(&["select"], &["select!"])?;
    value_contract::<biodata::GeneStatus>(&["approved"], &["approved!"])?;
    value_contract::<biodata::LocusGroup>(&["protein_coding"], &["protein_coding!"])?;
    value_contract::<biodata::MatchKind>(&["symbol"], &["symbol!"])?;
    Ok(())
}

#[test]
fn scientific_vocabularies_preserve_identity() -> Check {
    value_contract::<biodata::SummarySource>(&["refseq"], &["refseq!"])?;
    value_contract::<biodata::HgncStatus>(&["entry_withdrawn"], &["entry_withdrawn!"])?;
    value_contract::<biodata::LocusType>(
        &["gene_with_protein_product"],
        &["gene_with_protein_product!"],
    )?;
    value_contract::<biodata::FaersAgeUnit>(&["800"], &["800!"])?;
    value_contract::<biodata::ExpressionSyntax>(&["hgvs.g"], &["hgvs.g!"])?;
    {
        let value = "NC_".parse::<biodata::RefSeqPrefix>()?;
        assert_eq!(value.to_string(), "NC_");
        assert_eq!(serde_json::to_value(value)?, json!("nc"));
        assert_eq!(
            serde_json::from_value::<biodata::RefSeqPrefix>(json!("nc"))?,
            value
        );
        assert!("NC_!".parse::<biodata::RefSeqPrefix>().is_err());
        assert!(serde_json::from_value::<biodata::RefSeqPrefix>(json!("NC_")).is_err());
        assert!(serde_json::from_value::<biodata::RefSeqPrefix>(json!("nc!")).is_err());
    }
    {
        let value = "ENSG".parse::<biodata::EnsemblFeature>()?;
        assert_eq!(value.to_string(), "ENSG");
        assert_eq!(serde_json::to_value(value)?, json!("gene"));
        assert_eq!(
            serde_json::from_value::<biodata::EnsemblFeature>(json!("gene"))?,
            value
        );
        assert!("ENSG!".parse::<biodata::EnsemblFeature>().is_err());
        assert!(serde_json::from_value::<biodata::EnsemblFeature>(json!("ENSG")).is_err());
        assert!(serde_json::from_value::<biodata::EnsemblFeature>(json!("gene!")).is_err());
    }
    Ok(())
}

#[test]
fn population_and_assertion_vocabularies_preserve_identity() -> Check {
    value_contract::<biodata::PopulationDataType>(&["exomes"], &["exomes!"])?;
    value_contract::<biodata::PopulationSex>(&["xx"], &["xx!"])?;
    value_contract::<biodata::GenotypeFilter>(&["raw"], &["raw!"])?;
    value_contract::<biodata::ProteinCoordinateConvention>(
        &["one_based_inclusive"],
        &["one_based_inclusive!"],
    )?;
    value_contract::<biodata::ProteinIsoformStatus>(&["Displayed"], &["Displayed!"])?;
    value_contract::<biodata::ProteinSequenceAvailability>(&["available"], &["available!"])?;
    {
        let value = "practice guideline".parse::<biodata::ReviewStatus>()?;
        assert_eq!(value.to_string(), "practice guideline");
        assert_eq!(serde_json::to_value(value)?, json!("practice_guideline"));
        assert_eq!(
            serde_json::from_value::<biodata::ReviewStatus>(json!("practice_guideline"))?,
            value
        );
        assert!(
            "practice guideline!"
                .parse::<biodata::ReviewStatus>()
                .is_err()
        );
        assert!(
            "practice_guideline"
                .parse::<biodata::ReviewStatus>()
                .is_err()
        );
        assert!(
            serde_json::from_value::<biodata::ReviewStatus>(json!("practice guideline")).is_err()
        );
        assert!(
            serde_json::from_value::<biodata::ReviewStatus>(json!("practice_guideline!")).is_err()
        );
    }
    value_contract::<biodata::AlleleOrigin>(&["germline"], &["germline!"])?;
    value_contract::<biodata::AcmgAmp2015Code>(&["pathogenic"], &["pathogenic!"])?;
    value_contract::<biodata::ClingenLowPenetrance2024Code>(
        &["pathogenic_low_penetrance"],
        &["pathogenic_low_penetrance!"],
    )?;
    value_contract::<biodata::AmpAscoCap2017TierCode>(&["tier_i"], &["tier_i!"])?;
    value_contract::<biodata::AmpAscoCap2017LevelCode>(&["level_a"], &["level_a!"])?;
    value_contract::<biodata::ClingenCgcVicc2022Code>(&["oncogenic"], &["oncogenic!"])?;
    value_contract::<biodata::CivicEvidenceLevelCode>(&["a"], &["a!"])?;
    value_contract::<biodata::OncokbOncogenicityCode>(&["oncogenic"], &["oncogenic!"])?;
    value_contract::<biodata::OncokbLevelCode>(&["level_1"], &["level_1!"])?;
    value_contract::<biodata::Proposition>(&["pathogenicity"], &["pathogenicity!"])?;
    value_contract::<biodata::Direction>(&["supports"], &["supports!"])?;
    value_contract::<biodata::Predicate>(&["is_causal_for"], &["is_causal_for!"])?;
    Ok(())
}

#[test]
fn equal_identity_uses_equal_hashes() -> Check {
    equal_hash::<biodata::OrcidPublicVisibility>("PUBLIC", "public")?;
    equal_hash::<biodata::Base>("A", "A")?;
    equal_hash::<biodata::Sequence>("ACGTN", "ACGTN")?;
    equal_hash::<biodata::GeneSymbol>("BRAF", "bRaF")?;
    equal_hash::<biodata::Doi>("10.1000/ABC", "10.1000/abc")?;
    equal_hash::<biodata::ScientificPValue>("1.50e-8", "1.5e-8")?;
    equal_hash::<biodata::ScientificPValue>("1.000e0", "1e0")?;
    equal_hash::<biodata::VcvId>("VCV000000001.1", "VCV000000001.1")?;
    equal_hash::<biodata::RcvId>("RCV000000001.1", "RCV000000001.1")?;
    equal_hash::<biodata::ScvId>("SCV000000001.1", "SCV000000001.1")?;
    Ok(())
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config {
        cases: 128, failure_persistence: None, ..Default::default()
    })]
    #[test]
    fn public_variant_parsers_never_panic_and_roundtrip(
        arbitrary in proptest::collection::vec(proptest::char::any(), 0..2200),
        accession in "(NM_|NC_)[0-9]{6}\\.[1-9][0-9]{0,3}",
        suffix in "[1-9][0-9]{0,6}[ACGT]>[ACGT]",
    ) {
        let arbitrary: String = arbitrary.into_iter().collect();
        for text in [arbitrary, accession.clone(), format!("{accession}:c.{suffix}")] {
            property_roundtrip::<biodata::Accession>(&text)?;
            property_roundtrip::<biodata::HgvsText>(&text)?;
            property_roundtrip::<biomcp_types::VariantInput>(&text)?;
        }
    }
}

fn property_roundtrip<T>(text: &str) -> Result<(), proptest::test_runner::TestCaseError>
where
    T: FromStr + Eq + Debug + ToString,
    T::Err: Debug,
{
    if let Ok(value) = text.parse::<T>() {
        let parsed = value.to_string().parse::<T>().ok();
        proptest::prop_assert_eq!(parsed.as_ref(), Some(&value), "display must roundtrip");
    }
    Ok(())
}

#[test]
fn source_text_and_markdown_use_their_public_constructors() -> Check {
    for text in ["", " ", " α\n", "\u{0001}"] {
        let source = biodata::SourceText::from(text);
        assert_eq!(
            source.to_string(),
            text,
            "source display must preserve every character"
        );
        assert_eq!(
            serde_json::from_value::<biodata::SourceText>(json!(text))?,
            source,
            "source JSON must preserve every character"
        );
        let constructed = biodata::Markdown::new(text);
        let decoded = serde_json::from_value::<biodata::Markdown>(json!(text));
        assert_eq!(
            constructed.is_ok(),
            decoded.is_ok(),
            "Markdown JSON must run constructor checks"
        );
        if let Ok(value) = constructed {
            assert_eq!(decoded?, value, "Markdown constructor and JSON must agree");
        }
    }
    Ok(())
}

#[test]
fn structured_text_values_reject_invalid_native_parts() -> Check {
    for (text, wire) in [
        ("1.50e-8", json!({"mantissa":"1.50", "exponent":-8})),
        ("1e0", json!({"mantissa":"1", "exponent":0})),
        ("1e1", json!({"mantissa":"1", "exponent":1})),
        ("0e-8", json!({"mantissa":"0", "exponent":-8})),
        ("1.e-8", json!({"mantissa":"1.", "exponent":-8})),
    ] {
        assert_eq!(
            text.parse::<biodata::ScientificPValue>().ok(),
            serde_json::from_value::<biodata::ScientificPValue>(wire).ok(),
            "probability text and JSON must check the same parts"
        );
    }
    for (text, wire) in [
        ("WP1", json!({"namespace":"wikipathways", "id":"WP1"})),
        ("WP01", json!({"namespace":"wikipathways", "id":"WP01"})),
        ("R-HSA-1", json!({"namespace":"reactome", "id":"R-HSA-1"})),
        ("R-HSA-01", json!({"namespace":"reactome", "id":"R-HSA-01"})),
    ] {
        assert_eq!(
            text.parse::<biodata::PathwayId>().ok(),
            serde_json::from_value::<biodata::PathwayId>(wire).ok(),
            "pathway JSON must check the native identifier"
        );
    }
    Ok(())
}

fn constructor_contract<T, E>(
    new: fn(&str) -> Result<T, E>,
    valid: &[&str],
    invalid: &[&str],
) -> Check
where
    T: FromStr + Serialize + DeserializeOwned + Eq + Debug + ToString + std::hash::Hash,
    T::Err: std::error::Error + 'static,
{
    value_contract::<T>(valid, invalid)?;
    for text in valid.iter().chain(invalid) {
        assert_eq!(
            new(text).ok(),
            text.parse::<T>().ok(),
            "construction and parsing must check the same input: {text:?}"
        );
    }
    Ok(())
}

fn value_contract<T>(valid: &[&str], invalid: &[&str]) -> Check
where
    T: FromStr + Serialize + DeserializeOwned + Eq + Debug + ToString + std::hash::Hash,
    T::Err: std::error::Error + 'static,
{
    plain_value_contract::<T>(valid, invalid)?;
    for text in valid {
        let canonical = text.parse::<T>()?.to_string();
        equal_hash::<T>(text, &canonical)?;
    }
    Ok(())
}

#[test]
fn coding_changes_preserve_checked_search_text_without_a_reference() -> Check {
    value_contract::<biodata::CodingChange>(
        &[
            "c.333G>T",
            "c.780-4C>G",
            "c.678-14_678-3del",
            "c.1dup",
            "c.1_2dup",
            "c.1_2insAC",
            "c.1delinsAC",
            "c.1_2delinsN",
            "c.-2_-1del",
            "c.*1_*2del",
            "c.-1_1insG",
            "c.1+1_1+2insA",
            "c.1-2_1-1insA",
            "c.1-1_1insA",
            "c.1_1+1insA",
            "c.1_*1insA",
            "c.1+2_2-2insA",
            "c.999999999999999999999999999999G>T",
        ],
        &[
            "c.333garbage",
            "c.0A>T",
            "c.01A>T",
            "c.1+0A>T",
            "c.1-0A>T",
            "c.1A>A",
            "c.1AA>T",
            "c.1A>TT",
            "c.1_2A>T",
            "c.2_1del",
            "c.-1_-2del",
            "c.1_1del",
            "c.1insA",
            "c.1_3insA",
            "c.1_*2insA",
            "c.1+1_3-1insA",
            "c.1-1_2insA",
            "c.1_2+1insA",
            "c.-3+1_-1-1insA",
            "c.*1+1_*3-1insA",
            "c.-2+1_1-1insA",
            "c.1+1_*2-1insA",
            "c.-1+1_*1-1insA",
            "c.1_2ins",
            "c.1delins",
            "c.1delA",
            "c.1inv",
            "c.[1A>T;2G>C]",
            "c.(1A>T)",
            "c.?A>T",
            "c.1a>t",
            "c.1A>U",
            "g.1A>T",
            "p.Val1Gly",
            "NM_000314.8:c.333G>T",
            "c.1A>T\n",
            "c.1A>T\"[VRNM] OR PTEN[GENE]",
        ],
    )
}

#[test]
fn source_timestamp_values_preserve_native_text() -> Check {
    value_contract::<biomcp_types::SourceTimestampValue>(
        &[
            "2024-05-03T10:00:00+02:00",
            "2024-02-29t00:00:00.123456789012z",
            "1990-12-31T23:59:60Z",
        ],
        &[
            "2024-05-03",
            "2023-02-29T10:00:00Z",
            "2024-05-03T10:00:00Z\n",
        ],
    )
}
