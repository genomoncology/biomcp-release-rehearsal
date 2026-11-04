//! Feature protein-types: neutral native records and checked standalone protein inputs.
use biodata::*;
use biomcp_types::*;
use serde_json::{Value, json};

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(
    value: Value,
) -> Result<Value, Box<dyn std::error::Error>> {
    Ok(serde_json::to_value(serde_json::from_value::<T>(value)?)?)
}

#[test]
fn stated_isoforms_and_residues_roundtrip_without_imputation()
-> Result<(), Box<dyn std::error::Error>> {
    for id in ["P15056", "P15056-2", "P00533-1", "A0A024RBG1-12"] {
        assert_eq!(roundtrip::<UniProtIsoform>(json!(id))?, id);
    }
    for id in ["P15056-0", "P15056-01", "P15056-", "P15056-2-1", "P15056.1"] {
        assert!(serde_json::from_value::<UniProtIsoform>(json!(id)).is_err());
    }
    assert_eq!(roundtrip::<UniProtIsoform>(json!("P15056"))?, "P15056");
    assert_eq!(
        roundtrip::<AminoAcids>(json!("ACDEFGHIKLMNPQRSTVWYUOX"))?,
        "ACDEFGHIKLMNPQRSTVWYUOX"
    );
    for residues in ["", "a", "B", "J", "Z", "*", "A C"] {
        assert!(serde_json::from_value::<AminoAcids>(json!(residues)).is_err());
    }
    assert_eq!(
        Error::ProteinResidueInvalid.code(),
        "protein.residue.invalid"
    );
    assert_eq!(Error::ProteinSiteOrder.code(), "protein.site.order");
    assert_eq!(
        Error::UniProtIsoformInvalid.code(),
        "id.uniprot_isoform.invalid"
    );
    for id in ["IPR000001", "IPR052999"] {
        assert_eq!(roundtrip::<InterProId>(json!(id))?, id);
    }
    for id in ["IPR1", "IPR00000x", "interpro:IPR000001"] {
        assert!(serde_json::from_value::<InterProId>(json!(id)).is_err());
    }
    assert_eq!(roundtrip::<ComplexPortalId>(json!("CPX-715"))?, "CPX-715");
    for id in ["CPX-0", "CPX-0715", "cpx-715", "CPX-715x"] {
        assert!(serde_json::from_value::<ComplexPortalId>(json!(id)).is_err());
    }
    Ok(())
}

#[test]
fn checked_requests_refuse_conflicts_and_bounds_before_io() -> Result<(), Box<dyn std::error::Error>>
{
    for value in [
        json!({"query":"EGFR"}),
        json!({"gene":"CDKN2A","reviewed":true,"limit":25,"offset":5,"pool":"retained"}),
    ] {
        let v = roundtrip::<ProteinSearch>(value)?;
        assert!(v.get("include_unreviewed").is_none());
    }
    for value in [
        json!({}),
        json!({"query":"EGFR","gene":"EGFR"}),
        json!({"query":"EGFR","limit":0}),
        json!({"gene":"BRAF","limit":26}),
        json!({"query":"EGFR","include_unreviewed":true}),
        json!({"query":null,"gene":"BRAF"}),
        json!({"query":"x","all_species":true}),
    ] {
        assert!(
            serde_json::from_value::<ProteinSearch>(value.clone()).is_err(),
            "{value}"
        );
    }
    for section in [
        "features",
        "domains",
        "isoforms",
        "complexes",
        "structures",
        "function",
        "sequence",
        "all",
    ] {
        roundtrip::<ProteinGet>(json!({"input":["P00533", "P38398-2"],"section":[section]}))?;
    }
    roundtrip::<ProteinGet>(json!({"input":["CDKN2A"]}))?;
    for value in [
        json!({"input":[]}),
        json!({"input":["P00533"],"section_limit":0}),
        json!({"input":["P00533"],"section":["associations"]}),
    ] {
        assert!(serde_json::from_value::<ProteinGet>(value).is_err());
    }
    roundtrip::<ProteinFeaturesQuery>(
        json!({"accession":"P00533","position":858,"type":"DNA binding"}),
    )?;
    assert!(
        serde_json::from_value::<ProteinFeaturesQuery>(json!({"accession":"P00533","position":0}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<ProteinDomainsQuery>(json!({"accession":"P00533","position":858}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<ProteinComplexMembersQuery>(json!({"accession":"P15056"}))
            .is_err()
    );
    roundtrip::<ProteinComplexMembersQuery>(json!({"accession":"P15056","complex":"CPX-715"}))?;
    roundtrip::<ProteinSequenceQuery>(json!({"isoform":"P38398-2"}))?;
    Ok(())
}

#[test]
fn uncertain_feature_bounds_keep_source_ranges() -> Result<(), Box<dyn std::error::Error>> {
    let feature = json!(
        {"protein":"P00533-1","span":{"start":{"min":711,"max":711},"end":{"min":979}},"kind":{"label":"Domain"},"original":"L",
        "alternative":["R"],"evidence":["ECO:0000269|PubMed:1"]}
    );
    assert_eq!(roundtrip::<UniProtFeature>(feature.clone())?, feature);
    for bound in [json!({}), json!({"min":2,"max":1})] {
        assert!(serde_json::from_value::<Bound>(bound).is_err());
    }
    let span = UncertainSpan0::from_inclusive(
        Bound::new(Some(712), Some(714))?,
        Bound::new(Some(979), Some(979))?,
    )?;
    assert_eq!(
        serde_json::to_value(span)?,
        json!({"start":{"min":711,"max":713},"end":{"min":979,"max":979}})
    );
    Ok(())
}

#[test]
fn source_rows_keep_discontinuous_fragments_and_independent_complex_totals()
-> Result<(), Box<dyn std::error::Error>> {
    let domain = json!(
        {"id":"IPR000494","name":"Synthetic source match","entry_type":"family","protein":"P00533","protein_length":1210,"taxon":9606,
        "locations":[{"fragments":[{"span":{"start":{"min":56,"max":56},"end":{"min":167,"max":167}},"dc_status":"N_TERMINAL_DISC"},
        {"span":{"start":{"min":360,"max":360},"end":{"min":480,"max":480}},"dc_status":"C_TERMINAL_DISC"}]},
        {"fragments":[{"span":{"start":{"min":700,"max":700},"end":{"min":710,"max":710}}}]}],"member_signatures":[{"namespace":"pfam",
        "id":"PF01030","name":"Synthetic member"}],"attribution":{"source":"interpro","record":"IPR000494"}}
    );
    assert_eq!(roundtrip::<ProteinDomainMatch>(domain.clone())?, domain);
    let complex = json!(
        {"id":"CPX-715","name":"Synthetic complex","taxon":9606,"predicted":true,"participant_total":93,"matched_participant":"P38398",
        "url":"https://example.org/CPX-715"}
    );
    assert_eq!(roundtrip::<ProteinComplexHeader>(complex.clone())?, complex);
    let member = json!({"id":"CHEBI:1","participant_type":"small molecule","name":"Synthetic non-protein member","stoichiometry":"minValue: 1, maxValue: 1"});
    let v = roundtrip::<ProteinComplexMember>(member.clone())?;
    assert_eq!(v, member);
    assert!(v.get("protein").is_none());
    Ok(())
}

#[test]
fn external_products_and_exact_sequences_remain_independent()
-> Result<(), Box<dyn std::error::Error>> {
    let external = json!({"ids":["Q8N726-1"],"name":"Synthetic External product","sequence_status":"External","direct_sequence":"not_checked"});
    assert_eq!(roundtrip::<ProteinIsoform>(external.clone())?, external);
    let absent = json!({"ids":["P00533-4"],"name":"Synthetic absent direct sequence","sequence_status":"Described","direct_sequence":"not_available"});
    assert_eq!(roundtrip::<ProteinIsoform>(absent.clone())?, absent);
    let sequence = json!(
        {"accession":"P38398-2","sequence":"M","length":1,"audit":{"entry_version":2,"sequence_version":1,
        "last_sequence_update":"2026-01-01"}}
    );
    assert_eq!(roundtrip::<ProteinSequence>(sequence.clone())?, sequence);
    for status in ["Displayed", "Described", "External"] {
        assert_eq!(roundtrip::<ProteinIsoformStatus>(json!(status))?, status);
    }
    assert!(serde_json::from_value::<ProteinIsoformStatus>(json!("canonical")).is_err());
    let old = json!({"accession":"P00533","name":"Synthetic protein","length":1210});
    assert_eq!(roundtrip::<biomcp_types::Protein>(old.clone())?, old);
    Ok(())
}

#[test]
fn exact_isoform_header_uses_shared_identity_fields_without_base_replacement()
-> Result<(), Box<dyn std::error::Error>> {
    let header = json!(
        {"protein":{"accession":"P38398-2","name":"Synthetic exact isoform","length":63},"entry_id":"Synthetic entry","taxon":9606,
        "reviewed":true,"url":"https://example.org/P38398-2"}
    );
    assert_eq!(roundtrip::<ProteinHeader>(header.clone())?, header);
    Ok(())
}

#[test]
fn inclusive_conversion_refuses_reversed_or_zero_residues() -> Result<(), Box<dyn std::error::Error>>
{
    let exact = |n| Bound::new(Some(n), Some(n));
    assert_eq!(
        UncertainSpan0::from_inclusive(exact(2)?, exact(1)?),
        Err(biodata::CoordinateError::ReversedBounds)
    );
    assert_eq!(
        UncertainSpan0::from_inclusive(exact(0)?, exact(1)?),
        Err(biodata::CoordinateError::ZeroPosition)
    );
    assert_eq!(
        UncertainSpan0::from_inclusive(exact(1)?, exact(0)?),
        Err(biodata::CoordinateError::ZeroPosition)
    );
    assert!(
        UncertainSpan0::from_inclusive(
            Bound::new(Some(2), Some(4))?,
            Bound::new(Some(1), Some(3))?
        )
        .is_ok()
    );
    Ok(())
}

#[test]
fn feature_provenance_and_sequence_availability_use_closed_native_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    let feature = json!(
        {"feature":{"protein":"P00533","span":{"start":{"min":711,"max":711},"end":{"min":979,"max":979}},"kind":{"label":"Domain"}},
        "attribution":{"source":"uniprot","record":"P00533"},"source_coordinates":"one_based_inclusive"}
    );
    assert_eq!(roundtrip::<ProteinFeature>(feature.clone())?, feature);
    for state in ["available", "not_available", "not_checked"] {
        assert_eq!(
            roundtrip::<ProteinSequenceAvailability>(json!(state))?,
            state
        );
    }
    assert!(serde_json::from_value::<ProteinSequenceAvailability>(json!("inferred")).is_err());
    assert!(serde_json::from_value::<ProteinCoordinateConvention>(json!("zero_based")).is_err());
    roundtrip::<ProteinRelationQuery>(
        json!({"accession":"P15056","limit":25,"offset":5,"pool":"retained"}),
    )?;
    assert!(
        serde_json::from_value::<ProteinRelationQuery>(json!({"accession":"P15056","limit":0}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<ProteinRelationQuery>(json!({"accession":"P15056","limit":26}))
            .is_err()
    );
    Ok(())
}
