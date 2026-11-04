//! Caller-visible checked gene identifier contracts.
use biodata::{
    Accession, AccessionBase, Cytoband, EnsemblGeneId, GeneSymbol, NcbiGeneId, OmimId, UniProtAcc,
};
use biomcp_types::GeneInput;
use serde::{Serialize, de::DeserializeOwned};
use std::{collections::HashSet, fmt::Debug, str::FromStr};
type Check = Result<(), Box<dyn std::error::Error>>;
fn values<T>(valid: &[&str], invalid: &[&str]) -> Check
where
    T: FromStr + Serialize + DeserializeOwned + Debug + PartialEq,
    T::Err: std::error::Error + 'static,
{
    for text in valid {
        let value: T = text.parse()?;
        let json = serde_json::to_string(&value)?;
        assert_eq!(
            json,
            serde_json::to_string(text)?,
            "Gene identifiers must reject unsupported forms and roundtrip exactly"
        );
        assert_eq!(
            serde_json::from_str::<T>(&json)?,
            value,
            "Gene identifiers must reject unsupported forms and roundtrip exactly"
        );
    }
    for text in invalid {
        let error = text
            .parse::<T>()
            .err()
            .ok_or("accepted invalid identifier")?;
        assert!(
            !error.to_string().contains("distinctive-rejected-input"),
            "Gene identifiers must reject unsupported forms and roundtrip exactly"
        );
        assert!(
            serde_json::from_str::<T>(&serde_json::to_string(text)?).is_err(),
            "Gene identifiers must reject unsupported forms and roundtrip exactly"
        );
    }
    for json in ["null", "1", "{}", "[]"] {
        assert!(
            serde_json::from_str::<T>(json).is_err(),
            "Gene identifiers must reject unsupported forms and roundtrip exactly"
        );
    }
    Ok(())
}
#[test]
fn identifiers_keep_canonical_source_spelling_and_refuse_bad_boundaries() -> Check {
    values::<NcbiGeneId>(
        &["1", "673", "4294967295"],
        &["", "0", "01", "-1", "4294967296", " 673"],
    )?;
    values::<OmimId>(
        &["100000", "164757", "699999"],
        &[
            "000001", "099999", "700000", "12345", "1234567", "12345x", " 164757",
        ],
    )?;
    values::<EnsemblGeneId>(
        &["ENSG00000157764", "ENSG00000157764.12"],
        &[
            "ENSG0000015776",
            "ENSG000001577640",
            "ENSG00000157764.0",
            "ENSG00000157764.01",
            "ENSG00000157764.4294967296",
            "ENST00000157764",
            "ENSG00000157764.x",
        ],
    )?;
    values::<AccessionBase>(
        &["NM_004333", "ENST00000646891", "XM_123456789"],
        &["NM_004333.6", "NM_1234567", "ENST0000064689"],
    )?;
    values::<Accession>(
        &["NM_004333.6", "ENST00000646891.2"],
        &["NM_004333", "NM_004333.04", "NM_1234567.1"],
    )?;
    values::<UniProtAcc>(
        &["P15056", "Q9Y6K9", "A0A024RBG1", "A1B2C3"],
        &[
            "P15056-1",
            "p15056",
            "P1505",
            "A0A024RBG12",
            "O1A2B3C4D5",
            "A10000",
            "P1505X",
        ],
    )?;
    values::<Cytoband>(
        &["7q34", "9p21.3", "Xp21.2", "Yq11.221"],
        &[
            "0p11",
            "23q11",
            "chr7q34",
            "7r34",
            "Xp21.2-p21.1",
            "7q",
            "7q34.",
            "7q04",
            "MTp11",
        ],
    )?;
    let gene: EnsemblGeneId = "ENSG00000157764.12".parse()?;
    assert_eq!(gene.base(), "ENSG00000157764");
    assert_eq!(gene.version(), Some(12));
    assert_eq!(
        "NM_004333".parse::<AccessionBase>()?.to_string(),
        "NM_004333"
    );
    assert_eq!(NcbiGeneId::new(673)?.get(), 673);
    assert!(NcbiGeneId::new(0).is_err());
    Ok(())
}
#[test]
fn symbols_preserve_spelling_with_one_folded_equality_and_hash() -> Check {
    values::<GeneSymbol>(
        &["C17orf50", "C4B_2", "IGH@", "HLA-A", "BRAF"],
        &[
            "", " BRAF", "BR AF", "BRAF\n", "βRAF", "\u{7f}", "1BRAF", "BRAF!", "BRAF:foo",
        ],
    )?;
    let a = GeneSymbol::new("C17orf50")?;
    let b = GeneSymbol::new("C17ORF50")?;
    assert_eq!(a, b);
    assert_eq!(a.as_str(), "C17orf50");
    assert_eq!(HashSet::from([a, b]).len(), 1);
    assert!(GeneSymbol::new(&"x".repeat(GeneSymbol::MAX_LEN)).is_ok());
    assert!(GeneSymbol::new(&"x".repeat(GeneSymbol::MAX_LEN + 1)).is_err());
    Ok(())
}
#[test]
fn gene_inputs_classify_explicit_identifiers_and_keep_alias_text() -> Check {
    assert!(matches!(
        " HGNC:1097 ".parse::<GeneInput>()?,
        GeneInput::HgncId(_)
    ));
    assert!(matches!(
        "ENSG00000157764.2".parse::<GeneInput>()?,
        GeneInput::EnsemblId(_)
    ));
    assert!(matches!(
        "673".parse::<GeneInput>()?,
        GeneInput::EntrezId(_)
    ));
    values::<GeneInput>(
        &[
            "HGNC:1097",
            "ENSG00000157764.2",
            "673",
            "HER-2/neu",
            "MLN 19",
            "BRAF",
        ],
        &["", " ", "HGNC:x", "ENSGwrong", "0"],
    )?;
    assert_eq!(" HGNC:1097 ".parse::<GeneInput>()?.to_string(), "HGNC:1097");
    for text in ["HER-2/neu", "MLN 19", "BRAF"] {
        assert!(
            matches!(text.parse::<GeneInput>()?, GeneInput::Text(value) if value.as_str() == text)
        );
    }
    for text in [
        "",
        " \t",
        "HGNC:x",
        "hgnc:1097",
        "ensg00000157764",
        "HGNC:0",
        "ENSGwrong",
        "0",
        "01",
        "4294967296",
    ] {
        assert!(text.parse::<GeneInput>().is_err());
    }
    Ok(())
}
