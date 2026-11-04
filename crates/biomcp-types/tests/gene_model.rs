//! Public gene entities keep source identity, ambiguity and complete prose.
use biodata::*;
use biomcp_types::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use std::fmt::Debug;
type Check = Result<(), Box<dyn std::error::Error>>;
fn round_trip<T: Serialize + DeserializeOwned + PartialEq + Debug>(value: &T) -> Check {
    assert_eq!(
        serde_json::from_value::<T>(serde_json::to_value(value)?)?,
        *value,
        "Gene values must preserve their checked serialization contract"
    );
    Ok(())
}
fn header() -> Result<GeneHeader, Error> {
    Ok(GeneHeader {
        hgnc_id: HgncId::new(1097).map_err(|_| Error::HgncIdInvalid)?,
        symbol: GeneSymbol::new("BRAF")?,
        name: Some(Text::new("BRAF name")?),
        locus_group: Some(LocusGroup::ProteinCoding),
        location: None,
        status: GeneStatus::Approved,
        r#match: None,
    })
}
fn gene() -> Result<Gene, Error> {
    Ok(Gene {
        hgnc_id: HgncId::new(1097).map_err(|_| Error::HgncIdInvalid)?,
        symbol: GeneSymbol::new("BRAF")?,
        entrez_id: None,
        ensembl_id: None,
        name: Some(Text::new("BRAF name")?),
        status: GeneStatus::Approved,
        locus_group: Some(LocusGroup::ProteinCoding),
        locus_type: None,
        location: None,
        previous_symbols: Box::new([]),
        aliases: Box::new([]),
        uniprot: Box::new([]),
        omim: Box::new([]),
        mane: Box::default(),
        spans: Box::new([]),
        summary: None,
    })
}
#[test]
fn gene_resolutions_preserve_every_candidate_and_withdrawn_identity() -> Check {
    let matching = Match {
        kind: MatchKind::Symbol,
        text: Text::new("BRAF")?,
    };
    let mut entity = gene()?;
    entity.entrez_id = Some(NcbiGeneId::new(673)?);
    entity.ensembl_id = Some("ENSG00000157764.12".parse()?);
    entity.locus_type = Some(LocusType::GeneWithProteinProduct);
    entity.previous_symbols = vec![GeneSymbol::new("BRAF1")?].into_boxed_slice();
    entity.aliases = vec![Text::new("alias with spaces / symbols")?].into_boxed_slice();
    entity.uniprot = vec!["P15056".parse()?].into_boxed_slice();
    entity.omim = vec!["164757".parse()?].into_boxed_slice();
    entity.mane = vec![Mane::new(
        ManeStatus::Select,
        "NM_004333.6".parse()?,
        "ENST00000646891.2".parse()?,
    )?]
    .into();
    let prose = "Complete source summary. ".repeat(1000);
    entity.summary = Some(Summary {
        text: Text::new(&prose)?,
        source: SummarySource::RefSeq,
        dated: Some("2026-10".parse()?),
    });
    entity.spans = vec![GenomicInterval {
        build: Some(Build::GRCh38),
        chromosome: "7".parse()?,
        span: Span0::new(140719327, 140924929)?,
        strand: Some(Strand::Minus),
    }]
    .into_boxed_slice();
    let value = serde_json::to_value(&entity)?;
    assert_eq!(value["summary"]["text"], prose.trim());
    round_trip(&entity)?;
    for resolution in [
        GeneResolution::Resolved {
            gene: Box::new(entity),
            r#match: matching,
        },
        GeneResolution::Ambiguous {
            candidates: vec![header()?, header()?].into_boxed_slice(),
        },
        GeneResolution::Withdrawn {
            hgnc_id: HgncId::new(18617)?,
            symbol: GeneSymbol::new("BRCA3")?,
            hgnc_status: HgncStatus::EntryWithdrawn,
            merged_into: vec![HgncId::new(1)?, HgncId::new(2)?].into_boxed_slice(),
        },
        GeneResolution::NotFound {},
    ] {
        round_trip(&resolution)?;
    }
    let absent = serde_json::to_value(gene()?)?;
    for field in [
        "locus_type",
        "location",
        "entrez_id",
        "ensembl_id",
        "mane",
        "summary",
    ] {
        assert!(absent.get(field).is_none());
    }
    assert_eq!(
        serde_json::to_value(GeneResolution::NotFound {})?,
        json!({"resolution":"not-found"})
    );
    let extended: GeneResolution =
        serde_json::from_value(json!({"resolution":"not-found","gene":{}}))?;
    assert_eq!(
        serde_json::to_value(extended)?,
        json!({"resolution":"not-found"})
    );
    assert!(serde_json::from_value::<Gene>(json!({})).is_err());
    round_trip(&header()?)?;
    Ok(())
}
#[test]
fn gene_metadata_can_be_absent_without_losing_identity() -> Check {
    let mut missing_metadata = gene()?;
    missing_metadata.name = None;
    missing_metadata.locus_group = None;
    round_trip(&missing_metadata)?;
    let missing_metadata = serde_json::to_value(missing_metadata)?;
    assert!(missing_metadata.get("name").is_none());
    assert!(missing_metadata.get("locus_group").is_none());
    Ok(())
}
#[test]
fn mane_requires_curated_versioned_transcripts_at_its_boundary() -> Check {
    let valid = Mane::new(
        ManeStatus::Select,
        "NM_004333.6".parse()?,
        "ENST00000646891.2".parse()?,
    )?;
    assert_eq!(valid.refseq().to_string(), "NM_004333.6");
    round_trip(&valid)?;
    for (refseq, ensembl) in [
        ("NM_004333", "ENST00000646891.2"),
        ("NM_004333.6", "ENST00000646891"),
        ("XM_004333.6", "ENST00000646891.2"),
    ] {
        assert!(
            serde_json::from_value::<Mane>(
                json!({"status":"select", "refseq":refseq,"ensembl":ensembl})
            )
            .is_err()
        );
    }
    assert!(
        serde_json::from_value::<Mane>(
            json!({"status":"plus_clinical", "refseq":"NR_123456.1","ensembl":"ENST00000646891.2"})
        )
        .is_ok()
    );
    assert!(serde_json::from_value::<Mane>(json!({"status":"select", "refseq":"NM_004333.6","ensembl":"ENST00000646891.2","extra":true})).is_ok());
    Ok(())
}
#[test]
fn genomic_intervals_preserve_assembly_strand_and_scaffold_names() -> Check {
    for name in ["1", "22", "X", "Y", "MT", "NW_015148969.2"] {
        let contig = Contig::new(name)?;
        assert_eq!(contig.as_str(), name);
        round_trip(&contig)?;
    }
    for name in ["", " ", "chr 7", "7\n", "é"] {
        assert!(Contig::new(name).is_err());
    }
    assert!(Contig::new(&"x".repeat(Contig::MAX_LEN)).is_ok());
    assert!(Contig::new(&"x".repeat(Contig::MAX_LEN + 1)).is_err());
    for build in [Some(Build::GRCh37), Some(Build::GRCh38), None] {
        for strand in [None, Some(Strand::Plus), Some(Strand::Minus)] {
            let interval = GenomicInterval {
                build,
                strand,
                chromosome: "7".parse()?,
                span: Span0::new(0, 0)?,
            };
            round_trip(&interval)?;
            let json = serde_json::to_value(interval)?;
            assert_eq!(json.get("strand").is_none(), strand.is_none());
        }
    }
    for value in [
        json!({"chromosome":"chr7","span":{"start":0,"end":1}}),
        json!({"build":"GRCh38","chromosome":"7","span":{"start":2,"end":1}}),
    ] {
        assert!(serde_json::from_value::<GenomicInterval>(value).is_err());
    }
    let summary = Summary {
        text: Text::new("full")?,
        source: SummarySource::RefSeq,
        dated: None,
    };
    round_trip(&summary)?;
    assert!(serde_json::to_value(summary)?.get("dated").is_none());
    Ok(())
}

#[test]
fn gene_locations_parse_publisher_forms_without_changing_text() -> Check {
    for (text, kind) in [
        ("Xp22.33 and Yp11.32", "bands"),
        ("Xp22.3 and Yp11.3", "bands"),
        ("7q34", "bands"),
        ("Xp21.2-p21.1", "range"),
        ("9p21.2-9p21.1", "range"),
        ("mitochondria", "mitochondrial"),
        ("publisher location text", "unparsed"),
        ("7q34 and invalid", "unparsed"),
        ("Xp21.2-invalid", "unparsed"),
        (" 7q34 ", "unparsed"),
    ] {
        let location = GeneLocation::from(text);
        assert_eq!(location.as_str(), text);
        assert_eq!(serde_json::to_value(&location)?, text);
        round_trip(&location)?;
        let parsed = match location.kind() {
            GeneLocationKind::Cytobands { first, additional } => {
                assert_eq!(
                    first.as_str(),
                    text.split(" and ").next().ok_or("no first band")?
                );
                assert_eq!(additional.len(), text.matches(" and ").count());
                "bands"
            }
            GeneLocationKind::Range { start, end } => {
                assert_eq!(start.as_str(), text.split('-').next().ok_or("no start")?);
                assert_eq!(
                    end.as_str(),
                    if text.starts_with('X') {
                        "Xp21.1"
                    } else {
                        "9p21.1"
                    }
                );
                "range"
            }
            GeneLocationKind::Mitochondrial => "mitochondrial",
            GeneLocationKind::Unparsed => "unparsed",
        };
        assert_eq!(parsed, kind);
    }
    Ok(())
}
