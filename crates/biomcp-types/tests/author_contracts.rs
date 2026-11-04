//! Public author contracts before adapters exist; rows below are synthetic.
use biodata::*;
use biomcp_types::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
fn roundtrip<T: DeserializeOwned + Serialize>(v: Value) -> Result<Value, serde_json::Error> {
    serde_json::to_value(serde_json::from_value::<T>(v)?)
}
#[test]
fn source_qualified_identity_checks_checksum_and_spelling() {
    for (input, canonical) in [
        ("semantic-scholar:2060978", "semantic-scholar:2060978"),
        ("orcid:0000-0002-4299-3018", "orcid:0000-0002-4299-3018"),
        ("orcid:0000-0002-1694-233X", "orcid:0000-0002-1694-233X"),
    ] {
        let id: AuthorId = input.parse().unwrap();
        assert_eq!(id.to_string(), canonical);
        assert_eq!(
            roundtrip::<AuthorId>(json!(input)).unwrap(),
            json!(canonical)
        );
        assert_eq!(canonical.parse::<AuthorId>().unwrap(), id);
    }
    for bad in [
        "semanticscholar:2060978",
        "2060978",
        "0000-0002-4299-3018",
        "orcid:0000-0002-4299-3019",
        "semantic-scholar:0",
        "semantic-scholar:12 3",
    ] {
        assert!(bad.parse::<AuthorId>().is_err(), "{bad}");
    }
}
#[test]
fn requests_validate_before_io_and_preserve_selection() {
    let q: AuthorSearch =
        serde_json::from_value(json!({"query":"Example name","source":"semanticscholar"})).unwrap();
    assert_eq!(q.limit(), 5);
    assert_eq!(q.offset(), 0);
    assert_eq!(q.source(), AuthorSearchSource::SemanticScholar);
    assert_eq!(
        roundtrip::<AuthorSearch>(json!({"query":"Example name","source":"semanticscholar"}))
            .unwrap()
            .get("source")
            .unwrap(),
        "semantic-scholar"
    );
    for value in [
        json!({"query":"x","limit":0}),
        json!({"query":"x","limit":26}),
        json!({"query":"x","offset":-1}),
        json!({"query":" "}),
        json!({"query":"x","source":"orcid"}),
    ] {
        assert!(serde_json::from_value::<AuthorSearch>(value).is_err());
    }
    assert!(serde_json::from_value::<AuthorGet>(json!({"ids":[]})).is_err());
    assert!(
        serde_json::from_value::<AuthorGet>(json!({"ids":vec!["semantic-scholar:123";20]})).is_ok()
    );
    assert!(
        serde_json::from_value::<AuthorGet>(json!({"ids":vec!["semantic-scholar:123";21]}))
            .is_err()
    );
    assert!(serde_json::from_value::<AuthorGet>(json!({"ids":["2060978"]})).is_err());
    let ids = json!({"ids":["semantic-scholar:2060978","orcid:0000-0002-4299-3018"]});
    assert_eq!(roundtrip::<AuthorGet>(ids.clone()).unwrap(), ids);
    let p: AuthorPapersQuery = serde_json::from_value(json!({"id":"semantic-scholar:2060978","full":true,"limit":25,"offset":5,"pool":"native-pool"})).unwrap();
    assert_eq!(p.section(), Some(AuthorPaperSection::Details));
    assert_eq!(p.limit(), 25);
    assert_eq!(p.offset(), 5);
    for value in [
        json!({"id":"semantic-scholar:1","limit":0}),
        json!({"id":"semantic-scholar:1","limit":26}),
        json!({"id":"semantic-scholar:1","offset":-1}),
        json!({"id":"semantic-scholar:1","section":"unknown"}),
        json!({"id":"semantic-scholar:1","section":null}),
    ] {
        assert!(serde_json::from_value::<AuthorPapersQuery>(value).is_err());
    }
}
#[test]
fn metadata_absence_and_repeated_byline_names_remain_source_local() {
    // Synthetic repetition protects the required contract; D's detailed pages
    // contain no repeated display names. No native titles or abstracts enter Git.
    let header = json!({"paper_id":"0123456789abcdef0123456789abcdef01234567",
"title":"Synthetic paper",
"external_ids":[{"namespace":"CorpusId","id":"CorpusId:123"}],
"doi":"10.1234/example"});
    let detail = json!(
        {"header":header,"authors":[{"id":"123","name":"Repeated name"},{"id":"456","name":"Repeated name"}],
        "publication_types":["JournalArticle","Review"],"open_access_pdf":{}}
    );
    assert_eq!(
        roundtrip::<AuthorPaperDetail>(detail.clone()).unwrap(),
        detail
    );
    assert!(detail.get("header").unwrap().get("pmid").is_none());
    assert!(detail.get("fields_of_study").is_none());
    let absent = json!({"id":"semantic-scholar:123","name":"Synthetic name"});
    assert_eq!(roundtrip::<AuthorHeader>(absent.clone()).unwrap(), absent);
    let empty = json!({"id":"semantic-scholar:123","name":"Synthetic name","counts":{"scope":"provider_record","paper_count":0}});
    assert_eq!(roundtrip::<AuthorHeader>(empty.clone()).unwrap(), empty);
}
fn assertion(code: u64, title: Option<&str>) -> Value {
    let mut v = json!(
        {"put_code":code,"work_type":"journal-article","source":{"source_client_id":"APP-1234567890ABCDEF","source_name":"Synthetic source",
        "assertion_origin_orcid":"0000-0002-4299-3018"},"visibility":"public","external_ids":[{"namespace":"doi","id":"10.1234/example",
        "relationship":"self"},{"namespace":"wosuid","id":"native-value","relationship":"part-of"}]}
    );
    if let (Some(title), Some(object)) = (title, v.as_object_mut()) {
        object.insert("title".into(), json!(title));
    }
    v
}
#[test]
fn orcid_groups_keep_all_ordered_assertions_and_native_relationships() {
    // A 0003 has 73 groups with two summaries; A 0005 has absent title/date.
    let group = json!(
        {"external_ids":[{"namespace":"pmid","id":"123","relationship":"self"}],"assertions":[assertion(2,Some("Synthetic title")),
        assertion(1,None)]}
    );
    assert_eq!(roundtrip::<OrcidWorkGroup>(group.clone()).unwrap(), group);
    let works = json!({"author":"0000-0002-4299-3018","groups":[group],"observed_groups":380,"collection_scope":"whole-public-works-reply"});
    assert_eq!(roundtrip::<OrcidWorks>(works.clone()).unwrap(), works);
    assert_eq!(
        roundtrip::<OrcidPublicVisibility>(json!("PUBLIC")).unwrap(),
        json!("public")
    );
    for client in [
        "APP-KO1W9Y3EYYW6ESL8",
        "0000-0001-9884-1913",
        "0000-0002-5982-8983",
    ] {
        assert_eq!(client.parse::<OrcidClientId>().unwrap().to_string(), client);
    }
    assert_eq!(
        "PUBLIC"
            .parse::<OrcidPublicVisibility>()
            .unwrap()
            .to_string(),
        "public"
    );
    for private in ["limited", "private", "LIMITED", "PRIVATE"] {
        assert!(serde_json::from_value::<OrcidPublicVisibility>(json!(private)).is_err());
    }
    let mut invalid = assertion(1, None);
    invalid
        .as_object_mut()
        .unwrap()
        .insert("visibility".into(), json!("private"));
    assert!(serde_json::from_value::<OrcidWorkAssertion>(invalid).is_err());
}
#[cfg(feature = "schema")]
fn check<T: ResponseBody + schemars::JsonSchema + serde::de::DeserializeOwned>(
    query: Value,
    body: Value,
) -> Result<(), Box<dyn std::error::Error>>
where
    T::Query: schemars::JsonSchema + serde::de::DeserializeOwned,
{
    let value = serde_json::to_value(Envelope::<T>::new(
        Status::Complete,
        serde_json::from_value(query)?,
        Some(serde_json::from_value(body)?),
    ))?;
    let schema = serde_json::to_value(schemars::schema_for!(Envelope<T>))?;
    assert!(
        jsonschema::validator_for(&schema)
            .map_err(|error| error.to_string())?
            .is_valid(&value),
        "Author replies must retain native identity and their schema contract"
    );
    assert_eq!(
        value.get("schema"),
        Some(&json!(T::SCHEMA)),
        "Author replies must retain native identity and their schema contract"
    );
    Ok(())
}
#[cfg(feature = "schema")]
#[test]
fn five_standalone_envelopes_validate_without_shared_request_changes() {
    check::<AuthorPage>(
        json!({"query":"Synthetic name"}),
        json!({"items":[],"matching_records":0}),
    )
    .unwrap();
    check::<AuthorCards>(json!({"ids":["semantic-scholar:123"]}), json!({"items":[]})).unwrap();
    let query = json!({"id":"semantic-scholar:123"});
    let body = json!({"author":"semantic-scholar:123","items":[]});
    check::<AuthorPapers>(query.clone(), body.clone()).unwrap();
    check::<AuthorPaperDetails>(query, body).unwrap();
    check::<OrcidWorks>(
        json!({"id":"orcid:0000-0002-4299-3018"}),
        json!({"author":"0000-0002-4299-3018","groups":[],"observed_groups":0,"collection_scope":"whole-public-works-reply"}),
    )
    .unwrap();
}
