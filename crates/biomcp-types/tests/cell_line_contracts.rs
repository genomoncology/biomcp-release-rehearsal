//! Feature cell-line-types: source-owned cell identities and standalone contracts before I/O.
use biodata::*;
use biomcp_types::*;
use serde_json::{Value, json};
fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(
    v: Value,
) -> serde_json::Result<Value> {
    serde_json::to_value(serde_json::from_value::<T>(v)?)
}
#[test]
fn checked_cell_identifiers_keep_their_exact_namespace() {
    for id in ["CVCL_2119", "CVCL_A794", "CVCL_E3VV", "CVCL_ZZZZ"] {
        assert_eq!(roundtrip::<CellosaurusAccession>(json!(id)).unwrap(), id);
        let parsed: CellosaurusAccession = id.parse().unwrap();
        assert_eq!(
            parsed.to_string().parse::<CellosaurusAccession>().unwrap(),
            parsed
        );
    }
    for id in [
        "CVCL_1",
        "CVCL_12345",
        "cvcl_2119",
        "CVCL_21-9",
        "CVCL_2119 ",
    ] {
        assert!(serde_json::from_value::<CellosaurusAccession>(json!(id)).is_err());
    }
    for id in [
        "DepMap:ACH-000362",
        "ChEMBL-Cells:CHEMBL3706573",
        "HGNC:HGNC:7132",
        "NCIt:C8263",
        "ORDO:Orphanet_514",
    ] {
        assert_eq!(roundtrip::<CellLineSourceId>(json!(id)).unwrap(), id);
        let parsed: CellLineSourceId = id.parse().unwrap();
        assert_eq!(
            parsed.to_string().parse::<CellLineSourceId>().unwrap(),
            parsed
        );
    }
    for id in [
        "ACH-000362",
        "CHEMBL3706573",
        ":foo",
        "DepMap:",
        "DepMap:ACH 1",
    ] {
        assert!(serde_json::from_value::<CellLineSourceId>(json!(id)).is_err());
    }
    for id in ["CHEMBL3706573", "CHEMBL3308063"] {
        assert_eq!(roundtrip::<ChemblCellId>(json!(id)).unwrap(), id);
        let parsed: ChemblCellId = id.parse().unwrap();
        assert_eq!(parsed.to_string().parse::<ChemblCellId>().unwrap(), parsed);
    }
    for id in [
        "CHEMBL",
        "chembl3706573",
        "CHEMBL37x",
        "chembl:CHEMBL3706573",
    ] {
        assert!(serde_json::from_value::<ChemblCellId>(json!(id)).is_err());
    }
}
#[test]
fn requests_reject_bad_bounds_and_identities_before_io() {
    let q: CellLineSearch = serde_json::from_value(json!({"query":"MV4;11"})).unwrap();
    assert_eq!(q.query().as_str(), "MV4;11");
    assert_eq!(q.limit(), 5);
    assert_eq!(q.offset(), 0);
    for v in [
        json!({}),
        json!({"query":""}),
        json!({"query":"x","limit":0}),
        json!({"query":"x","limit":26}),
        json!({"query":"x","offset":-1}),
        json!({"query":"x","sort":"name"}),
    ] {
        assert!(serde_json::from_value::<CellLineSearch>(v).is_err());
    }
    for section in ["names", "identity", "xrefs", "variants", "chembl", "all"] {
        roundtrip::<CellLineGet>(
            json!({"id":"CVCL_2119","section":section,"section_limit":25,"section_offset":2}),
        )
        .unwrap();
    }
    roundtrip::<CellLineGet>(json!({"id":"DepMap:ACH-000362"})).unwrap();
    for v in [
        json!({"id":"MOLM13"}),
        json!({"id":"CVCL_12"}),
        json!({"id":["CVCL_2119","CVCL_0004"]}),
        json!({"id":"CVCL_2119","section_limit":0}),
        json!({"id":"CVCL_2119","section_limit":26}),
        json!({"id":"CVCL_2119","section_offset":-1}),
        json!({"id":"CVCL_2119","section":"drug_response"}),
    ] {
        assert!(serde_json::from_value::<CellLineGet>(v).is_err());
    }
    roundtrip::<CellLineAssayCountQuery>(json!({"id":"CHEMBL3706573"})).unwrap();
    for id in ["CVCL_2119", "ChEMBL-Cells:CHEMBL3706573", "MOLM13"] {
        assert!(serde_json::from_value::<CellLineAssayCountQuery>(json!({"id":id})).is_err());
    }
}

// Synthetic contract rows preserve only the multiplicity and absence observed in the
// retained capture archives. They are not native replies or curation answers.
fn xref(database: &str, accession: &str) -> Value {
    json!({"namespace":database,"id":accession})
}
#[test]
fn whole_variations_keep_fusion_members_and_non_pubmed_provenance() {
    let fusion = json!({"variation_type":"Gene fusion","description":"source fusion spelling",
        "genes":[{"reference":xref("HGNC","HGNC:7132"),"hgnc":"HGNC:7132"},{"reference":xref("HGNC","HGNC:7136"),"hgnc":"HGNC:7136"}],
        "provenance":[{"reference":{"resource_internal_ref":"PubMed=9305600"}},{"institution":"Example institution","value":"Native value"}]});
    assert_eq!(
        roundtrip::<CellLineVariation>(fusion.clone()).unwrap(),
        fusion
    );
    let variation = json!({"variation_type":"Native future label","mutation_type":"Native future mutation",
        "description":"native coordinates without assembly","note":"Complete source splice note","zygosity":"Native future zygosity",
        "genes":[{"reference":xref("HGNC","HGNC:7989"),"hgnc":"HGNC:7989"}],
        "references":[xref("ClinVar","VCV000375874")],
        "provenance":[{"xref":xref("DepMap","ACH-000336")},{"xref":xref("Cosmic-CLP","1290455")}]});
    let row = roundtrip::<CellLineVariation>(variation.clone()).unwrap();
    assert_eq!(row, variation);
    assert!(row.get("allele").is_none());
    assert!(row["provenance"][0].get("reference").is_none());
    let page = json!({"accession":"CVCL_1844","items":{"items":[variation],"total":3},"offset":2});
    assert_eq!(roundtrip::<CellLineVariants>(page.clone()).unwrap(), page);
}
#[test]
fn collections_keep_independent_totals_and_complete_qualified_text() {
    let warning = json!({"scope":"Native warning category","text":"Some initial stocks had the source problem. Subsequent stocks were corrected."});
    assert_eq!(
        roundtrip::<CellLineWarning>(warning.clone()).unwrap(),
        warning
    );
    let page = json!({"accession":"CVCL_0007","species":{"items":[xref("NCBI_TaxID","9606")],"total":1},
        "diseases":{"items":[xref("NCIt","C8263"),xref("ORDO","Orphanet_514")],"total":2},
        "warnings":{"items":[warning],"total":1},"offset":0});
    assert_eq!(
        roundtrip::<CellLineIdentityPage>(page.clone()).unwrap(),
        page
    );
    let page = json!({"accession":"CVCL_0004","items":{"items":[xref("Example","entry-1")],"total":1398},"offset":1397});
    assert_eq!(roundtrip::<CellLineXrefs>(page.clone()).unwrap(), page);
    let name = json!({"kind":"identifier","value":"MV4;11"});
    assert_eq!(roundtrip::<CellLineName>(name.clone()).unwrap(), name);
    let dates = json!({"created":"2012-04-04","updated":"2026-06-25","version":48});
    assert_eq!(
        roundtrip::<CellLineEntryDates>(dates.clone()).unwrap(),
        dates
    );
}
#[test]
fn chembl_absence_and_synthetic_zero_remain_source_local() {
    let record = json!({"id":"CHEMBL3706573","cell_id":{"namespace":"ChEMBL","id":5547},"cellosaurus_id":"CVCL_2119","name":"MOLM-13"});
    let got = roundtrip::<CellLineChemblRecord>(record.clone()).unwrap();
    assert_eq!(got, record);
    assert!(got.get("clo").is_none());
    assert!(got.get("efo").is_none());
    for count in [0, 1408] {
        let value = json!({"id":"CHEMBL3706573","assay_count":count});
        assert_eq!(
            roundtrip::<CellLineAssayCount>(value.clone()).unwrap(),
            value
        );
    }
    let skipped: Section<CellLineChembl> = Section::Skipped {
        source: Source::Chembl,
        reason: Reason::Disabled,
    };
    assert_eq!(serde_json::to_value(skipped).unwrap()["state"], "skipped");
}

#[test]
fn section_requests_keep_independent_defaults_and_offsets() {
    let q: CellLineGet = serde_json::from_value(json!({"id":"CVCL_2119"})).unwrap();
    assert!(matches!(q.id(), CellLineInput::Accession(_)));
    assert_eq!(q.section(), None);
    assert_eq!(q.section_limit(), None);
    assert_eq!(q.section_offset(), 0);
    let q: CellLineGet = serde_json::from_value(json!({"id":"ChEMBL-Cells:CHEMBL3706573","section":"all","section_limit":1,"section_offset":25})).unwrap();
    assert!(matches!(q.id(), CellLineInput::Source(_)));
    assert_eq!(q.section(), Some(CellLineSection::All));
    assert_eq!(q.section_limit(), Some(1));
    assert_eq!(q.section_offset(), 25);
    for value in [
        json!({"id":"CVCL_2119","section":null}),
        json!({"id":"CVCL_2119","section_limit":null}),
        json!({"id":"CVCL_2119","drug":"x"}),
    ] {
        assert!(serde_json::from_value::<CellLineGet>(value).is_err());
    }
    let identity = json!(
        {"accession":"CVCL_2119","name":"Source display","category":"Native category","species":[xref("NCBI_TaxID","9606")],
        "diseases":[xref("NCIt","C8263"),xref("ORDO","Orphanet_514")]}
    );
    assert_eq!(
        roundtrip::<CellLineIdentity>(identity.clone()).unwrap(),
        identity
    );
}

#[cfg(feature = "schema")]
fn check_envelope<T>(query: Value, body: Value) -> Result<(), Box<dyn std::error::Error>>
where
    T: ResponseBody + schemars::JsonSchema + serde::de::DeserializeOwned,
    T::Query: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let envelope = Envelope::<T>::new(
        Status::Complete,
        serde_json::from_value(query)?,
        Some(serde_json::from_value(body)?),
    );
    let value = serde_json::to_value(envelope)?;
    let schema = serde_json::to_value(schemars::schema_for!(Envelope<T>))?;
    let validator = jsonschema::validator_for(&schema).map_err(|error| error.to_string())?;
    assert!(
        validator.is_valid(&value),
        "Cell line replies must preserve their versioned envelope"
    );
    assert_eq!(
        value.get("schema"),
        Some(&json!(T::SCHEMA)),
        "Cell line replies must preserve their versioned envelope"
    );
    Ok(())
}
#[cfg(feature = "schema")]
#[test]
fn standalone_envelopes_validate_all_eight_consumer_schemas() {
    let get = json!({"id":"CVCL_2119"});
    let empty = json!({"items":[],"total":0});
    let header = json!({"accession":"CVCL_2119","name":"Native display","category":"Native category","species":empty,"diseases":empty});
    check_envelope::<CellLinePage>(
        json!({"query":"native spelling"}),
        json!({"items":[header.clone()]}),
    )
    .unwrap();
    check_envelope::<CellLineCard>(
        get.clone(),
        json!(
            {"input":"CVCL_2119","status":"complete","header":header,"dates":{},"warnings":empty,"candidates":[],"sections":{},"more":["names",
            "identity","xrefs","variants","chembl"]}
        ),
    )
    .unwrap();
    let page = json!({"accession":"CVCL_2119","items":empty,"offset":0});
    check_envelope::<CellLineNames>(get.clone(), page.clone()).unwrap();
    check_envelope::<CellLineXrefs>(get.clone(), page.clone()).unwrap();
    check_envelope::<CellLineVariants>(get.clone(), page.clone()).unwrap();
    check_envelope::<CellLineChembl>(get.clone(), page).unwrap();
    check_envelope::<CellLineIdentityPage>(
        get,
        json!({"accession":"CVCL_2119","species":empty,"diseases":empty,"warnings":empty,"offset":0}),
    )
    .unwrap();
    check_envelope::<CellLineAssayCount>(
        json!({"id":"CHEMBL3706573"}),
        json!({"id":"CHEMBL3706573","assay_count":0}),
    )
    .unwrap();
}

#[cfg(feature = "schema")]
#[test]
fn qualified_id_schema_matches_control_character_refusals() {
    let schema = serde_json::to_value(schemars::schema_for!(CellLineSourceId)).unwrap();
    let input_schema = serde_json::to_value(schemars::schema_for!(CellLineInput)).unwrap();
    for schema in [schema, input_schema] {
        let validator = jsonschema::validator_for(&schema).unwrap();
        for id in [
            "DepMap:ACH-000362",
            "HGNC:HGNC:7132",
            "DepMap:ACH\u{007f}1",
            "DepMap:ACH\u{0085}1",
            "DepMap:ACH 1",
        ] {
            let value = json!(id);
            assert_eq!(
                validator.is_valid(&value),
                serde_json::from_value::<CellLineSourceId>(value).is_ok(),
                "{id:?}"
            );
        }
    }
}
