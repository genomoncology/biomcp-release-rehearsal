//! study-types: public no-I/O contracts from retained cBioPortal metadata shapes.
use biodata::{Study, StudyId, StudySampleList};
use biomcp_types::Paged;
use biomcp_types::{StudyGet, StudySearch};
use serde_json::{Value, json};

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(
    v: Value,
) -> Result<Value, serde_json::Error> {
    serde_json::to_value(serde_json::from_value::<T>(v)?)
}
fn header() -> Value {
    json!(
        {"studyId":"brca_tcga_pan_can_atlas_2018","name":"Synthetic study","cancerTypeId":"brca","allSampleCount":1084,"pmids":["29625048",
        "29596782","29625048"],"referenceGenome":"hg19"}
    )
}
fn study() -> Value {
    json!(
        {"header":header(),"description":"Synthetic source description","sequencedSampleCount":1066,"cnaSampleCount":1080,
        "structuralVariantCount":0,"treatmentCount":0,"importDate":"2026-08-20 21:46:46","groups":"",
        "readPermission":true,"publicStudy":true}
    )
}
#[test]
fn exact_ids_roundtrip_without_case_or_punctuation_repair() -> Result<(), Box<dyn std::error::Error>>
{
    for value in ["brca_tcga_pan_can_atlas_2018", "Mixed-Study.1"] {
        let id: StudyId = value.parse()?;
        assert_eq!(id.as_str(), value);
        assert_eq!(id.to_string(), value);
        assert_eq!(roundtrip::<StudyId>(json!(value))?, value);
    }
    assert_ne!("BRCA".parse::<StudyId>()?, "brca".parse::<StudyId>()?);
    for value in [
        "", " brca", "brca ", "../brca", "a/b", "a?x", "a%2Fb", "a\nb",
    ] {
        assert!(value.parse::<StudyId>().is_err());
        assert!(serde_json::from_value::<StudyGet>(json!({"id":value})).is_err());
    }
    Ok(())
}
#[test]
fn native_counts_and_publication_multiplicity_survive_without_member_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let v = study();
    assert_eq!(roundtrip::<Study>(v.clone())?, v);
    let mut zero = v;
    zero["patientCount"] = json!(0);
    assert_eq!(roundtrip::<Study>(zero.clone())?, zero);
    let list = json!(
        {"sampleListId":"brca_tcga_pan_can_atlas_2018_all","studyId":"brca_tcga_pan_can_atlas_2018","name":"Synthetic list",
        "description":"Synthetic description","category":"all_cases_in_study","sampleCount":1084}
    );
    assert_eq!(roundtrip::<StudySampleList>(list.clone())?, list);
    let mut absent = list.clone();
    absent
        .as_object_mut()
        .ok_or("object")?
        .remove("sampleCount");
    assert_eq!(roundtrip::<StudySampleList>(absent.clone())?, absent);
    let mut forbidden = list.clone();
    forbidden["sampleIds"] = json!(["synthetic-member"]);
    assert_eq!(roundtrip::<StudySampleList>(forbidden)?, list);
    Ok(())
}
#[test]
fn literal_search_and_bounded_section_requests_validate_before_io()
-> Result<(), Box<dyn std::error::Error>> {
    let q: StudySearch = serde_json::from_value(
        json!({"text":"breast AND lung","cancer_type":"mixed","limit":2,"offset":3,"pool":"snapshot"}),
    )?;
    assert_eq!(q.text().map(biodata::Text::as_str), Some("breast AND lung"));
    assert_eq!(q.cancer_type().map(|v| v.as_str()), Some("mixed"));
    assert_eq!((q.limit(), q.offset()), (2, 3));
    assert_eq!(q.at_offset(5).pool(), q.pool());
    assert!(serde_json::from_value::<StudySearch>(json!({"cancer_type":"brca"})).is_ok());
    for v in [
        json!({}),
        json!({"text":" "}),
        json!({"text":null}),
        json!({"text":null,"cancer_type":"brca"}),
        json!({"text":"breast","cancer_type":null}),
        json!({"text":"x","limit":0}),
        json!({"text":"x","limit":51}),
        json!({"text":"x","offset":-1}),
    ] {
        assert!(serde_json::from_value::<StudySearch>(v).is_err());
    }
    for section in ["profiles", "sample-lists", "files", "all"] {
        let q: StudyGet = serde_json::from_value(
            json!({"id":"brca_tcga_pan_can_atlas_2018","section":section,"section_limit":2}),
        )?;
        assert_eq!(q.id().as_str(), "brca_tcga_pan_can_atlas_2018");
        if section == "all" {
            assert!(q.at_section_offset(2).is_err());
        } else {
            assert_eq!(q.at_section_offset(2)?.section_offset(), 2);
        }
    }
    for v in [
        json!({"id":"brca","section_offset":1}),
        json!({"id":"brca","section":"counts"}),
        json!({"id":"brca","gene":"TP53"}),
        json!({"id":"brca","section_limit":0}),
        json!({"id":"brca","section_limit":null}),
        json!({"id":"brca","section":"all","section_offset":1}),
    ] {
        assert!(serde_json::from_value::<StudyGet>(v).is_err());
    }
    Ok(())
}
#[test]
fn exact_profile_resource_and_file_metadata_keep_their_native_scope()
-> Result<(), Box<dyn std::error::Error>> {
    use biodata::*;
    let profile = json!(
        {"molecularProfileId":"brca_tcga_pan_can_atlas_2018_armlevel_cna","studyId":"brca_tcga_pan_can_atlas_2018",
        "molecularAlterationType":"GENERIC_ASSAY","genericAssayType":"ARMLEVEL_CNA","datatype":"CATEGORICAL","name":"Synthetic profile",
        "description":"Synthetic description","patientLevel":false,"showProfileInAnalysisTab":true}
    );
    assert_eq!(roundtrip::<StudyProfile>(profile.clone())?, profile);
    let mut absent = profile;
    absent
        .as_object_mut()
        .ok_or("object")?
        .remove("genericAssayType");
    assert_eq!(roundtrip::<StudyProfile>(absent.clone())?, absent);
    let resource = json!(
        {"resourceId":"IDC_OHIF_CT","displayName":"Synthetic resource","description":"Synthetic resource description",
        "resourceType":"PATIENT","priority":"1","openByDefault":true,"cancerStudyIdentifier":"brca_tcga_pan_can_atlas_2018","sampleCount":7,
        "patientCount":6}
    );
    let mut card = study();
    card["resourceCounts"] = json!([resource]);
    assert_eq!(roundtrip::<Study>(card.clone())?, card);
    let file = json!(
        {"name":"LICENSE","path":"public/brca_tcga_pan_can_atlas_2018/LICENSE","htmlUrl":"https://example.invalid/blob",
        "downloadUrl":"https://example.invalid/raw","sha":"db7b143aab711f10d0349d3f7d56ded293a44c63","repositoryBlobSize":446,"kind":"file"}
    );
    assert_eq!(roundtrip::<StudyFile>(file.clone())?, file);
    let mut directory = file;
    directory["kind"] = json!("dir");
    directory
        .as_object_mut()
        .ok_or("object")?
        .remove("downloadUrl");
    assert_eq!(roundtrip::<StudyFile>(directory.clone())?, directory);
    for invalid in [
        "2026-02-29 01:02:03",
        "2026-10-26 24:00:00",
        "2026-10-26T01:00:00Z",
    ] {
        assert!(invalid.parse::<StudyImportTime>().is_err());
    }
    let imported: StudyImportTime = "2026-08-20 21:46:46".parse()?;
    assert_eq!(imported.to_string(), imported.as_str());
    let error = "bad/id".parse::<StudyId>().err().ok_or("rejected")?;
    assert_eq!(error.code(), "study.value.invalid");
    assert_eq!(error.to_string(), "study.value.invalid");
    Ok(())
}
#[cfg(feature = "schema")]
fn envelope<T>(query: Value, body: Value) -> Result<Value, Box<dyn std::error::Error>>
where
    T: biomcp_types::ResponseBody + schemars::JsonSchema + serde::de::DeserializeOwned,
    T::Query: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let q = serde_json::from_value(query)?;
    let body = serde_json::from_value::<T>(body)?;
    let v = serde_json::to_value(biomcp_types::Envelope::new(
        biomcp_types::Status::Complete,
        q,
        Some(body),
    ))?;
    let schema = schemars::generate::SchemaSettings::draft2020_12()
        .into_generator()
        .into_root_schema_for::<biomcp_types::Envelope<T>>();
    assert!(
        jsonschema::draft202012::new(&serde_json::to_value(schema)?)?.is_valid(&v),
        "Study replies must retain their versioned envelope contract"
    );
    assert_eq!(
        v.get("schema").and_then(Value::as_str),
        Some(T::SCHEMA),
        "Study replies must retain their versioned envelope contract"
    );
    Ok(v)
}
#[cfg(feature = "schema")]
#[test]
fn five_envelopes_keep_catalog_and_live_acquisition_times_independent()
-> Result<(), Box<dyn std::error::Error>> {
    use biomcp_types::*;
    let snapshot = json!({"revision":"retained-revision","acquired":1793059200});
    let page = json!({"offset":0,"returned":0,"total":0});
    envelope::<StudyPage>(
        json!({"text":"breast"}),
        json!({"items":[header()],"page":{"offset":0,"returned":1,"total":1},"snapshot":snapshot}),
    )?;
    let scope = json!({"study_id":"brca_tcga_pan_can_atlas_2018","items":[],"page":page,"acquired":1793145600});
    for section in ["profiles", "sample-lists", "files"] {
        let q = json!({"id":"brca_tcga_pan_can_atlas_2018","section":section});
        match section {
            "profiles" => {
                envelope::<StudyProfiles>(q, scope.clone())?;
            }
            "sample-lists" => {
                envelope::<StudySampleLists>(q, scope.clone())?;
            }
            _ => {
                envelope::<StudyFiles>(q, scope.clone())?;
            }
        }
    }
    let v = envelope::<StudyCard>(
        json!({"id":"brca_tcga_pan_can_atlas_2018","section":"all"}),
        json!(
            {"source_url":"https://www.cbioportal.org/study/summary?id=brca_tcga_pan_can_atlas_2018","study":study(),"snapshot":snapshot,
            "profiles":{"state":"data","source":"cbioportal","data":scope},"sample_lists":{"state":"skipped","source":"cbioportal",
            "reason":"disabled"}}
        ),
    )?;
    assert_ne!(
        v["result"]["snapshot"]["acquired"],
        v["result"]["profiles"]["data"]["acquired"]
    );
    assert!(v["result"]["study"].get("patientCount").is_none());
    Ok(())
}
