//! Feature pathway-types: public no-I/O identity and consumer contracts.
use biodata::{Pathway, PathwayEventRef, PathwayId, PathwayMember, WikiPathwaysId};
use biomcp_types::Paged;
use biomcp_types::{PathwayEventsQuery, PathwayGenesQuery, PathwayGet, PathwaySearch};
use serde_json::{Value, json};

fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(
    v: Value,
) -> Result<Value, serde_json::Error> {
    serde_json::to_value(serde_json::from_value::<T>(v)?)
}

#[test]
fn identities_preserve_native_namespaces_and_refuse_foreign_species() {
    for id in ["WP382", "WP1"] {
        assert_eq!(
            roundtrip::<WikiPathwaysId>(json!(id)).expect("public round-trip"),
            id
        );
    }
    for id in ["WP", "WP0", "WP0382", "wp382", "WP382_r140476", "hsa:04010"] {
        assert!(serde_json::from_value::<WikiPathwaysId>(json!(id)).is_err());
    }
    for id in [
        json!({"namespace":"reactome","id":"R-HSA-5673001"}),
        json!({"namespace":"wikipathways","id":"WP382"}),
    ] {
        assert_eq!(
            roundtrip::<PathwayId>(id.clone()).expect("public round-trip"),
            id
        );
    }
    for id in [
        json!({"source":"reactome","id":"R-HSA-5673001"}),
        json!({"namespace":"reactome","id":"R-MMU-5673001"}),
        json!({"namespace":"reactome","id":"R-HSA-5673001.12"}),
        json!({"namespace":"kegg","id":"hsa04010"}),
    ] {
        assert!(serde_json::from_value::<PathwayId>(id).is_err());
    }
}

#[test]
fn search_selectors_and_page_bounds_are_checked_before_io() {
    for query in [
        json!({"query":"MAPK signaling"}),
        json!({"query":"MAPK signaling","source":["reactome","wikipathways"]}),
        json!({"gene":"HGNC:1097"}),
        json!({"top_level":true}),
        json!({"query":"MAPK","source":"wikipathways","limit":50,"offset":5,"pool":"snapshot","fresh":true}),
    ] {
        let checked: PathwaySearch = serde_json::from_value(query).expect("valid selector");
        assert_eq!(checked.at_offset(10).offset(), 10);
        assert_eq!(
            checked.reactome_leg().source(),
            biomcp_types::PathwaySource::Reactome
        );
        assert!((1..=50).contains(&checked.limit()));
    }
    for query in [
        json!({}),
        json!({"query":""}),
        json!({"query":"   "}),
        json!({"query":null}),
        json!({"gene":null}),
        json!({"query":"MAPK","gene":"HGNC:1097"}),
        json!({"query":"MAPK","top_level":true}),
        json!({"gene":"HGNC:1097","top_level":true}),
        json!({"source":[],"query":"MAPK"}),
        json!({"source":["reactome","reactome"],"query":"MAPK"}),
        json!({"source":"wikipathways","gene":"HGNC:1097"}),
        json!({"source":"wikipathways","top_level":true}),
        json!({"query":"MAPK","limit":0}),
        json!({"query":"MAPK","limit":51}),
        json!({"query":"MAPK","type":"Pathway"}),
        json!({"query":"MAPK","source":"all"}),
    ] {
        assert!(
            serde_json::from_value::<PathwaySearch>(query.clone()).is_err(),
            "{query}"
        );
    }
    let value = roundtrip::<PathwaySearch>(json!({"query":"MAPK"})).expect("public round-trip");
    assert_eq!(value.get("source"), Some(&json!(["reactome"])));
    assert_eq!(value.get("limit"), Some(&json!(5)));
}

#[test]
fn source_owned_sections_and_continuations_refuse_unavailable_relations() {
    for id in ["R-HSA-5673001", "WP382"] {
        for section in [None, Some("summary"), Some("all")] {
            let mut v = json!({"id":id});
            if let Some(section) = section {
                v.as_object_mut()
                    .expect("object")
                    .insert("section".into(), json!(section));
            }
            assert!(serde_json::from_value::<PathwayGet>(v).is_ok());
        }
    }
    for section in ["genes", "events"] {
        assert!(
            serde_json::from_value::<PathwayGet>(json!({"id":"R-HSA-5673001","section":section}))
                .is_ok()
        );
        assert!(
            serde_json::from_value::<PathwayGet>(json!({"id":"WP382","section":section})).is_ok()
                == (section == "genes")
        );
    }
    for v in [
        json!({"id":"R-MMU-5673001"}),
        json!({"id":"hsa04010"}),
        json!({"id":"WP0"}),
        json!({"id":"WP382","section":"enrichment"}),
        json!({"id":"WP382","section":null}),
    ] {
        assert!(serde_json::from_value::<PathwayGet>(v).is_err());
    }
    let q = json!({"id":"R-HSA-5673001","limit":1,"offset":5,"pool":"snapshot"});
    assert_eq!(
        roundtrip::<PathwayGenesQuery>(q.clone()).expect("public round-trip"),
        q
    );
    assert_eq!(
        roundtrip::<PathwayEventsQuery>(q.clone()).expect("public round-trip"),
        q
    );
    let details: PathwayGenesQuery =
        serde_json::from_value(json!({"id":"R-HSA-5673001","details":"details"}))
            .expect("explicit full-context disclosure");
    assert!(details.at_offset(5).details());
    assert!(details.restart().details());
    assert!(
        serde_json::from_value::<PathwayEventsQuery>(
            json!({"id":"R-HSA-5673001","details":"details"})
        )
        .is_err()
    );
    assert!(
        serde_json::from_value::<PathwayGenesQuery>(json!({"id":"R-HSA-5673001","details":null}))
            .is_err()
    );
    let wiki = json!({"id":"WP382","limit":5,"offset":0});
    assert_eq!(
        roundtrip::<PathwayGenesQuery>(wiki.clone()).expect("WikiPathways genes"),
        wiki
    );
    assert!(serde_json::from_value::<PathwayEventsQuery>(wiki).is_err());
    assert!(
        serde_json::from_value::<PathwayGenesQuery>(json!({"id":"WP382","details":"details"}))
            .is_err()
    );
    for q in [
        json!({"id":"R-HSA-5673001","limit":51}),
        json!({"id":"R-HSA-5673001","limit":0}),
    ] {
        assert!(serde_json::from_value::<PathwayGenesQuery>(q.clone()).is_err());
        assert!(serde_json::from_value::<PathwayEventsQuery>(q).is_err());
    }
}

#[test]
fn neutral_records_keep_absence_membership_and_unresolved_event_identity() {
    let base = json!(
        {"id":{"namespace":"reactome","id":"R-HSA-5673001"},"name":"RAF/MAP kinase cascade",
        "species":"Homo sapiens","version":"R-HSA-5673001.12"}
    );
    assert_eq!(
        roundtrip::<Pathway>(base.clone()).expect("public round-trip"),
        base
    );
    let mut empty = base.clone();
    empty
        .as_object_mut()
        .expect("object")
        .insert("description".into(), json!(""));
    assert_eq!(
        roundtrip::<Pathway>(empty.clone()).expect("public round-trip"),
        empty
    );
    let mut foreign = base;
    foreign
        .as_object_mut()
        .expect("object")
        .insert("species".into(), json!("Mus musculus"));
    assert!(serde_json::from_value::<Pathway>(foreign).is_err());
    for accession in ["P01116-1", "P01116-2"] {
        let member = json!(
            {"reference":{"namespace":"uniprot","id":accession},"name":format!("UniProt:{accession} KRAS"),"gene":"HGNC:6407",
            "schema_class":"ReferenceIsoform","physical_context_count":1,"participants":[{"db_id":{"namespace":"Reactome","id":5672718},
            "name":"p21 RAS:GTP:activated RAF homo/heterodimer complexes [plasma membrane]","schema_class":"Complex"}]}
        );
        assert_eq!(
            roundtrip::<PathwayMember>(member.clone()).expect("public round-trip"),
            member
        );
    }
    let unknown =
        json!({"reference":{"namespace":"chebi","id":"15377"},"physical_context_count":0});
    assert_eq!(
        roundtrip::<PathwayMember>(unknown.clone()).expect("public round-trip"),
        unknown
    );
    for event in [
        json!({"kind":"database","db_id":{"namespace":"Reactome","id":5673001}}),
        json!({"kind":"object","db_id":{"namespace":"Reactome","id":5668599},"name":"Native reaction","schema_class":"Reaction"}),
        json!({"kind":"object","db_id":{"namespace":"Reactome","id":5673001},"id":"R-HSA-5673001",
        "version":"R-HSA-5673001.12","name":"Native pathway","schema_class":"Pathway"}),
    ] {
        assert_eq!(
            roundtrip::<PathwayEventRef>(event.clone()).expect("public round-trip"),
            event
        );
    }
    assert!(
        serde_json::from_value::<PathwayEventRef>(
            json!({"kind":"database","db_id":{"namespace":"Reactome","id":0}})
        )
        .is_err()
    );
}

#[test]
fn native_short_page_keeps_total_scope_and_next_position() {
    let page = json!(
        {"items":[{"id":{"namespace":"reactome","id":"R-HSA-5673001"},"name":"RAF/MAP kinase cascade","species":"Homo sapiens"}],
        "page":{"offset":0,"returned":1,"total":917,"total_kind":"source","next_offset":5}}
    );
    assert_eq!(
        roundtrip::<biomcp_types::PathwayPage>(page.clone()).expect("public round-trip"),
        page
    );
    let page = json!(
        {"header":{"id":{"namespace":"reactome","id":"R-HSA-5673001"},"name":"RAF/MAP kinase cascade","species":"Homo sapiens"},
        "items":[{"kind":"database","db_id":{"namespace":"Reactome","id":5668599}}],"page":{"offset":5,
        "returned":1,"total":81,"total_kind":"observed","next_offset":10}}
    );
    assert_eq!(
        roundtrip::<biomcp_types::PathwayEvents>(page.clone()).expect("public round-trip"),
        page
    );
}

#[test]
fn wiki_metadata_sync_receipts_keep_each_catalog_scope() {
    let receipt = json!(
        {"downloaded_bytes":0,"search":{"sha256":"search-digest","bytes":2346682,"revision":"search-revision","human_rows":1131,"total_rows":2218},
        "get":{"sha256":"get-digest","bytes":1224217,"revision":"get-revision","human_rows":1131,"total_rows":2218}}
    );
    assert_eq!(
        roundtrip::<biomcp_types::WikiPathwaysSync>(receipt.clone()).expect("public round-trip"),
        receipt
    );
}

#[test]
fn legacy_gene_reference_keeps_its_wire_shape_and_checked_identity() {
    let reference = json!({"id":"R-HSA-5673001","name":"RAF/MAP kinase cascade"});
    assert_eq!(
        roundtrip::<biomcp_types::PathwayRef>(reference.clone()).expect("public round-trip"),
        reference
    );
    let id: biomcp_types::ReactomeId = "R-HSA-5673001".parse().expect("human identity");
    let same: biodata::ReactomeId = id;
    assert_eq!(same.to_string(), "R-HSA-5673001");
    for raw in ["R-HSA-5673001", "WP382"] {
        let id: PathwayId = raw.parse().expect("native identity");
        assert_eq!(id.to_string(), raw);
    }
}

#[test]
fn source_summation_multiplicity_and_pathway_citations_stay_independent() {
    // Minimal source-field projection; text/title and the second summation are synthetic controls.
    let pathway = json!(
        {"id":{"namespace":"reactome","id":"R-HSA-5673001"},"name":"RAF/MAP kinase cascade","species":"Homo sapiens",
        "summations":[{"db_id":{"namespace":"Reactome","id":5683030},"text":"Complete retained source text."},
        {"db_id":{"namespace":"Reactome","id":5683031},"text":"Second source summation."}],
        "references":[{"db_id":{"namespace":"Reactome","id":5672144},"pubmed":"24937142","title":"Native reference"},
        {"db_id":{"namespace":"Reactome","id":208231},"pubmed":"17496910"}]}
    );
    assert_eq!(
        roundtrip::<Pathway>(pathway.clone()).expect("public round-trip"),
        pathway
    );
    let wiki = json!(
        {"id":{"namespace":"wikipathways","id":"WP382"},"name":"MAPK signaling pathway","species":"Homo sapiens","revision":"2025-08-29",
        "description":"Native catalog excerpt ...","cited_in":""}
    );
    assert_eq!(
        roundtrip::<Pathway>(wiki.clone()).expect("public round-trip"),
        wiki
    );
}

#[test]
fn all_card_retains_each_requested_relation_state_without_inventing_rows() {
    let card = json!(
        {"pathway":{"id":{"namespace":"reactome","id":"R-HSA-5673001"},"name":"RAF/MAP kinase cascade","species":"Homo sapiens"},
        "sections":["summary","genes","events","all"],"genes":{"state":"empty","source":"reactome"},"events":{"state":"not-applicable"}}
    );
    assert_eq!(
        roundtrip::<biomcp_types::PathwayCard>(card.clone()).expect("public round-trip"),
        card
    );
}
