use super::{generate, validator};
use biodata::{
    Accession, AccessionBase, AminoAcids, AuthorId, Base, CaId, CbioportalCancerTypeId,
    CellLineSourceId, CellosaurusAccession, ChemblCellId, Chromosome, ClinPgxAccession,
    ClinVarAlleleId, ClinVarVariationId, CodingChange, ComplexPortalId, Contig, CosmicId, Curie,
    Cytoband, DeviceRecallNumber, DiagnosticDeviceId, Doi, DrugId, EnsemblGeneId, EuropePmcId,
    FdaDrugApplication, GeneSymbol, GtrTestId, GwasAssociationId, GwasStudyId, GwasTraitId, HgncId,
    HgvsText, HpId, InterProId, MondoId, NcbiGeneId, NcitId, NctId, OmimId, OrcidClientId, OrcidId,
    PaId, PartialDate, PgxDiplotypeExpression, PgxGeneKey, PmaNumber, PmaSupplement,
    PmaSupplementCell, Pmcid, Pmid, RcvId, ReactomeId, ReportId, RsId, ScvId,
    SemanticScholarAuthorId, SemanticScholarId, Sequence, StudyBlobSha, StudyId, StudyImportTime,
    StudyProfileId, StudyResourceId, StudySampleListId, UniProtAcc, UniProtIsoform, VariantId,
    VcvId, VrsId, WhoProductId, WikiPathwaysId,
};
use biomcp_types::{
    ArticleInput, GeneInput, MyVariantQuery, SheetRows, VariantInput, VariantRegion, VariantSearch,
};
use schemars::JsonSchema;
use serde_json::json;

// Compare admission at the public parser and schema boundaries, including near misses.
fn text_admissions<T: JsonSchema + std::str::FromStr>(examples: &[&str]) {
    let schema = validator::<T>();
    for text in examples {
        assert_eq!(
            schema.is_valid(&json!(text)),
            text.parse::<T>().is_ok(),
            "{}: {text:?}",
            std::any::type_name::<T>()
        );
    }
}

macro_rules! cases {
    ($($ty:ty => [$($text:expr),* $(,)?]),* $(,)?) => {$(
        text_admissions::<$ty>(&[$($text),*]);
    )*};
}
pub(super) fn text_value_patterns_match_parser_admissions() {
    variant_identifier_admissions();
    cases! {
        biodata::Text => ["safe", "safe\n", "safe\r", "safe\u{0085}", "safe\u{202e}"],
        HgncId => ["HGNC:1", "HGNC:0", "HGNC:4294967295", "HGNC:4294967296"],
        NcbiGeneId => ["1", "01", "4294967295", "4294967296"],
        Pmid => ["1", "0", "4294967295", "4294967296"],
        Pmcid => ["PMC1", "PMC0", "PMC4294967295", "PMC4294967296"],
        Doi => ["10.1/a", "10.1/a\u{0080}", "10.1/a\u{009f}", "10.1/α", "10.1/a\n", "10.1/a\u{feff}", "10.1/a\u{0085}"],
        EuropePmcId => ["PPR0001", "PPR0000", "ABCDEFGH1", "ABCDEFGHI1", &format!("PPR{}", "1".repeat(61)), &format!("PPR{}", "1".repeat(62))],
        SemanticScholarId => ["0123456789abcdef0123456789ABCDEF0123456789", "0123456789abcdef0123456789abcdef012345678g"],
        GeneSymbol => ["BRAF", "123", "A/B@c", &"A".repeat(255), &"A".repeat(256)],
        OmimId => ["100000", "699999", "099999", "700000", "000001", "00000", "0000000"],
        UniProtAcc => ["P15056", "A0A023GPI8", "p15056", "P15056-1"],
        Cytoband => ["7q31.2", "7q01.2", "23q31.2", "7q3"],
        EnsemblGeneId => ["ENSG00000000001", "ENSG00000000001.1", "ENSG00000000001.0", "ENSG00000000001.4294967295", "ENSG00000000001.4294967296"],
        HgvsText => ["NM_004333.6:c.1A>G", "NM_004333.0:c.1A>G", "NM_004333.6(123):c.1A>G", "NM_004333.6(BRAF):c.1A>G", "NP_004324.1:c.1A>G",
            "NP_004324.1:p.?", "NC_000007.14(NP_004324.1):g.1A>G", "NC_000007.14(NP_004324.0):g.1A>G", "NM_004333.65536:c.1A>G",
            &format!("NM_004333.6({}):c.1A>G", "A".repeat(255)), &format!("NM_004333.6({}):c.1A>G", "A".repeat(256))],
        CodingChange => ["c.333G>T", "c.780-4C>G", "c.678-14_678-3del", "c.-1_*2dup",
            "c.1_2insAC", "c.1delinsN", "c.333garbage", "c.01G>T", "c.1+0G>T", "c.1insA",
            "c.1_2ins", "c.1A>U", "c.[1A>G;2G>A]", "c.(1A>G)", "c.1inv", "c.1A>G\n",
            "c.999999999999999999999999999999G>T"],
        Base => ["A", "N", "a", "U", "AA"],
        Sequence => ["", "ACGTRYSWKMBDHVN", "a", "U", "AC GT"],
        Chromosome => ["1", "22", "23", "X", "MT", "M", "chr1", "01"],
        Contig => ["chr1", "chr 1", &"x".repeat(255), &"x".repeat(256)],
        Curie => ["HGNC:1", "A:α", "A:", "A:a\u{0080}", "A:a\u{feff}"],
        HpId => ["HP:0000001", "HP:000001", "HP:00000001"],
        MondoId => ["MONDO:0000001", "MONDO:000001", "MONDO:00000001"],
        NcitId => ["C1", "C01", "C", "C1A"],
        NctId => ["NCT00000001", "NCT00000000", "NCT000000001"],
        ReportId => ["0", "0001", "", "FAERS1"],
        DrugId => ["CHEMBL1", "DB:1", "db:1", "DB:1:2", "DB:1/2", "DB:1//2"],
        GwasStudyId => ["GCST000001", "GCST000000", "GCST00001", "GCST0000001"],
        GwasAssociationId => ["association:1", "association:0", "association:18446744073709551615", "association:18446744073709551616"],
        GwasTraitId => ["EFO_0001", "HP:0000001", "HP:", "_0001"],
        ReactomeId => ["R-HSA-1", "R-HSA-0", "R-HSA-01", "R-MMU-1"],
        WikiPathwaysId => ["WP1", "WP0", "WP01", "WP1.1"],
        UniProtIsoform => ["P15056", "P15056-1", "P15056-0", "P15056-01"],
        AminoAcids => ["ACDEFGHIKLMNPQRSTVWYUOX", "", "B", "a"],
        InterProId => ["IPR000001", "IPR00001", "IPR0000001"],
        ComplexPortalId => ["CPX-1", "CPX-0", "CPX-01"],
        biodata::TrialUploadTime => ["2024-02-29T23:59", "2024-01-01T24:00", "2024-01-01T12:60",
            "2024-01-01T12:00Z", "2024-01-01T12:00:00", "2024-01-01T12:00\n", "2024-01-01T12:00\r"],
        StudyId => ["study_1", "study/1", "_study"],
        StudyProfileId => ["study.1", "study/1", "_study"],
        StudySampleListId => ["study-1", "study/1", "_study"],
        CbioportalCancerTypeId => ["type_1", "type/1", "_type"],
        StudyResourceId => ["resource-1", "resource/1", "_resource"],
        StudyBlobSha => ["0123456789abcdef0123456789abcdef0123456789", "0123456789ABCDEF0123456789ABCDEF0123456789"],
        StudyImportTime => ["2024-02-29 23:59:59", "0000-01-01 00:00:00", "2024-13-01 00:00:00", "2024-01-32 00:00:00", "2024-01-01 24:00:00"],
        PartialDate => ["0001", "0000", "2024-02", "2024-13", "2024-01-32"],
        CellosaurusAccession => ["CVCL_0001", "CVCL_AA01", "CVCL_00001", "CVCL_aa01"],
        CellLineSourceId => ["DepMap:ACH-1", "DepMap:", "A:a\u{009f}", "A:a\u{feff}"],
        ChemblCellId => ["CHEMBL1", "CHEMBL01", "CHEMBL", "CHEMBL1A"],
        GtrTestId => ["GTR000000001.1", "GTR000000001.0", "GTR000000001.01"],
        WhoProductId => ["1234-123-12", "12345-12345-12", "123-123-12"],
        PmaNumber => ["P000001", "P00001", "P0000001"],
        PmaSupplement => ["S001", "S000", "S01", "S1000"],
        PmaSupplementCell => ["", "S001", "S000", "S01"],
        FdaDrugApplication => ["NDA000001", "BLA000001", "NDA00001", "NDA0000001"],
        DeviceRecallNumber => ["Z-0001-2026", "Z-001-2026", "Z-00001-2026", "Z-0001-26"],
        DiagnosticDeviceId => ["P000001/S001", "P000001/S000", "K000001", "DEN000001", "H000001"],
        ClinPgxAccession => ["PA1", "PA0", "PA01"],
        PgxGeneKey => ["BRAF", "123", &"A".repeat(255), &"A".repeat(256)],
        PgxDiplotypeExpression => [" *1/*2 ", " ", "\u{0085}", "\u{feff}"],
        SemanticScholarAuthorId => ["1", "0", "01", "A"],
        OrcidId => ["0000-0002-1825-0097", "0000-0002-1825-009", "0000-0002-1825-00x7"],
        AuthorId => ["orcid:0000-0002-1825-0097", "semantic-scholar:1", "semanticscholar:1", "semantic-scholar:0"],
        OrcidClientId => ["0000-0002-1825-0097", "APP-0123456789ABCDEF", "APP-0123456789abcDEF"],
        ArticleInput => ["1", "PMC1", "PPR0001", "PPRAB0001", "PPRABCDE1", "PPRABCDEF1", "PPR0000", "4294967296", "PMC4294967296", "10.1/a\u{0080}",
            "https://doi.org/10.1/a"],
        GeneInput => ["BRAF", " HGNC:1 ", "HGNC:0", "hgnc:1", "ENSGbad", "0001", "4294967296", "Gene alias"],
        SheetRows => ["1-1", "01-1", "1-4294967296"],
        VariantRegion => ["chr1:1-2", "chr23:1-2", "chr1:01-2", "chr1:1-4294967296"],
        MyVariantQuery => [" query ", " ", "a\u{0080}", "\u{0085}", "\u{feff}", &"a".repeat(4102), &"a".repeat(4103)],
    }
}

fn variant_identifier_admissions() {
    cases! {
        CaId => ["CA000001", "CA1", "CA000000", "CA0000001", "CA1000000", "CA18446744073709551615", "CA18446744073709551616"],
        PaId => ["PA1", "PA0", "PA01", "PA18446744073709551615", "PA18446744073709551616"],
        RsId => ["rs1", "rs0", "rs01", "rs18446744073709551615", "rs18446744073709551616"],
        Accession => ["NM_004333.6", "NM_004333.0", "NM_004333.65535", "NM_004333.65536", "NM_000004333.1", "NM_001004333.1", "LRG_0001t01",
            "LRG_4294967295p65535", "LRG_4294967296", "LRG_1p65536"],
        AccessionBase => ["LRG_0001t01", "NM_004333", "NM_000004333", "NM_001004333", "LRG_4294967296", "ENST00000000001"],
        ClinVarVariationId => [
            "clinvar:1",
            "clinvar:0",
            "clinvar:4294967295",
            "clinvar:4294967296",
            "clinvar:18446744073709551615",
            "clinvar:18446744073709551616",
        ],
        ClinVarAlleleId => ["1", "4294967295", "4294967296", "0", "01", "18446744073709551615", "18446744073709551616"],
        VcvId => ["VCV000000001", "VCV000000001.4294967295", "VCV000000001.4294967296", "VCV000000000", "VCV000000001.0", "VCV000000001.1"],
        RcvId => ["RCV000000001", "RCV000000001.4294967295", "RCV000000001.4294967296", "RCV000000000", "RCV000000001.01"],
        ScvId => ["SCV000000001", "SCV000000001.4294967295", "SCV000000001.4294967296", "SCV000000000", "SCV000000001.1"],
        CosmicId => ["COSV18446744073709551615", "COSV18446744073709551616", "COSV1", "COSM1", "COSN1", "COSV0", "COSV01"],
        VrsId => ["ga4gh:VA.012345678901234567890123456789_-", "ga4gh:VA.0123456789012345678901234567890+"],
        VariantId => [
            "clinvar:4294967295",
            "clinvar:4294967296",
            "COSV18446744073709551615",
            "COSV18446744073709551616",
            "CA000001",
            "CA000000",
            "VCV000000001",
            "VCV000000000",
            "rs18446744073709551616",
        ],
        VariantInput => [
            "CA000001",
            "CA000000",
            "VCV000000000",
            "chr22:g.1A>G",
            "chr23:g.1A>G",
            "4294967295",
            "4294967296",
            "clinvar:4294967295",
            "clinvar:4294967296",
            "rs18446744073709551615",
            "18446744073709551615",
            "18446744073709551616",
        ],
    }
}

// These semantic constraints intentionally remain in the parser.
pub(super) fn text_value_schema_supersets_name_their_runtime_rules() {
    fn runtime_rule<T: JsonSchema + std::str::FromStr>(text: &str, rule: &str) {
        let schema = generate::<T>();
        assert!(
            schema
                .get("description")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|description| description.contains(rule)),
            "schema omits runtime rule {rule}"
        );
        assert!(validator::<T>().is_valid(&json!(text)), "schema: {text}");
        assert!(text.parse::<T>().is_err(), "parser: {text}");
    }
    runtime_rule::<CodingChange>("c.2_1del", "increasing range");
    runtime_rule::<CodingChange>("c.1A>A", "different substitution");
    runtime_rule::<CodingChange>("c.1_3insA", "insertion adjacency");
    runtime_rule::<OrcidId>("0000-0002-1825-0098", "checksum");
    runtime_rule::<AuthorId>("orcid:0000-0002-1825-0098", "checksum");
    runtime_rule::<OrcidClientId>("0000-0002-1825-0098", "checksum");
    runtime_rule::<PartialDate>("2023-02-29", "calendar");
    runtime_rule::<StudyImportTime>("2023-02-29 00:00:00", "calendar");
    runtime_rule::<biodata::TrialUploadTime>("2023-02-29T00:00", "calendar");
    runtime_rule::<SheetRows>("2-1", "end");
    runtime_rule::<SheetRows>("1-201", "200");
    runtime_rule::<VariantRegion>("chr1:2-1", "end");
    runtime_rule::<MyVariantQuery>(&format!("a{}", "α".repeat(2051)), "UTF-8 byte");
}

// ProteinQuery stays private; its public parent owns admission at this boundary.
pub(super) fn protein_query_schema_matches_public_search_admissions()
-> Result<(), Box<dyn std::error::Error>> {
    let schema = generate::<VariantSearch>();
    let protein = schema
        .pointer("/$defs/ProteinQuery")
        .ok_or("missing ProteinQuery schema")?;
    let validator = jsonschema::draft202012::new(protein)?;
    for text in [
        "V600E",
        "p.Val600Glu",
        "p.Ala613=",
        "p.A613=",
        "p.=",
        "p.Ala613==",
        "p.Ala613=/Val",
        "p.(Ala613=)",
        "Val600E",
        "*1Ter",
        "V0E",
        "V01E",
        "V4294967295E",
        "V4294967296E",
        "v600e",
        "V600",
        "V600E\n",
    ] {
        let parsed =
            serde_json::from_value::<VariantSearch>(json!({"gene":"BRAF", "protein":text}));
        assert_eq!(validator.is_valid(&json!(text)), parsed.is_ok(), "{text:?}");
    }
    Ok(())
}
