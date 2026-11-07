//! Cache mutation and a short per-call deadline unavailable through the binary.
#![expect(
    clippy::unwrap_used,
    reason = "Fixture setup must fail on invalid checked inputs"
)]
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use biomcp_test_support::recordings as recording_support;
use std::{path::PathBuf, time::Duration};
fn replay(dir: PathBuf) -> Fetch {
    Fetch::open(FetchConfig {
        mode: Mode::Replay { dir },
        ..Default::default()
    })
    .unwrap()
}
#[test]
// Expire only the case-owned documented cache metadata.
fn stale_identity_and_annotation_sources_make_only_their_answers_partial() {
    use biomcp_fetch::ReplayCache;
    use biomcp_query::{Ctx, run};
    use biomcp_types::{GeneGet, GeneSearch, ReplyData, Request, Source, SourceState, Status};
    let directory = tempfile::tempdir().unwrap();
    let case_path = directory.path().join("case");
    let cache = ReplayCache::create_case_with_owners(&case_path, &[]).unwrap();
    let recordings = directory.path().join("recordings");
    let original = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replies/gene/main");
    recording_support::copy_capture(&original, &recordings, |_, record| {
        recording_support::cache_acquisition(record, 42);
    })
    .unwrap();
    let query = Request::GetGene(GeneGet::new(vec!["BRAF".parse().unwrap()].into()).unwrap());
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: recordings },
        replay_cache: Some(cache.clone()),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        run(
            &Ctx {
                ncbi_email: None,
                fetch: &fetch,
                budget: Duration::from_secs(20),
                fresh: false,
                drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
            },
            &query
        )
        .status(),
        Status::Complete
    );
    drop(fetch);
    let database = rusqlite::Connection::open(case_path.join("replay.sqlite")).unwrap();
    database
        .execute("UPDATE bundle SET fresh_until=0", [])
        .unwrap();
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        replay_cache: Some(cache),
        ..Default::default()
    })
    .unwrap();
    let cx = Ctx {
        ncbi_email: None,
        fetch: &fetch,
        budget: Duration::from_secs(20),
        fresh: false,
        drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
    };
    let outcome = run(&cx, &query);
    assert_eq!(outcome.status(), Status::Partial);
    assert_eq!(outcome.sources[0].source, Source::Hgnc);
    assert_eq!(outcome.sources[0].state, SourceState::Stale);
    assert_eq!(outcome.sources[1].state, SourceState::Ok);
    let ReplyData::GeneCards {
        body: Some(cards), ..
    } = outcome.body
    else {
        panic!("gene cards");
    };
    assert_eq!(cards.items[0].status, Status::Partial);
    let search =
        Request::SearchGene(GeneSearch::new(None, Some("9p21.3".parse().unwrap()), 10, 0).unwrap());
    assert_eq!(run(&cx, &search).status(), Status::Partial);
    database
        .execute("UPDATE bundle SET fresh_until=9223372036854775807", [])
        .unwrap();
    database
        .execute("UPDATE reply SET fresh_until=0", [])
        .unwrap();
    let outcome = run(&cx, &query);
    assert_eq!(outcome.status(), Status::Partial);
    assert_eq!(outcome.sources[0].state, SourceState::Ok);
    assert_eq!(outcome.sources[1].source, Source::MyGene);
    assert_eq!(outcome.sources[1].state, SourceState::Stale);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    reason = "Mutate isolated public recordings"
)]
fn clingen_local_failure_survives_later_source_deadline() {
    use biomcp_fetch::RequestError;
    use biomcp_query::{Ctx, run};
    use biomcp_types::{GeneGet, GeneSectionWord, Request};
    let directory = tempfile::tempdir().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replies/gene/clingen");
    for source in ["hgnc", "clingen"] {
        std::fs::create_dir(directory.path().join(source)).unwrap();
        for entry in std::fs::read_dir(root.join(source)).unwrap() {
            let path = entry.unwrap().path();
            let mut record: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            if record["url"].as_str().unwrap().contains("dosage") {
                continue;
            }
            record["exchanges"][0]["delay_ms"] = 0.into();
            // Remove annotation IDs so this isolates the ClinGen local/source boundary.
            if source == "hgnc" {
                let body = record["exchanges"][0]["body"]["value"].as_str().unwrap();
                let mut lines = body.lines();
                let header = lines.next().unwrap();
                if let Some(index) = header.split('\t').position(|v| v == "entrez_id") {
                    let mut output = format!("{header}\n");
                    for line in lines {
                        let mut cells: Vec<_> = line.split('\t').collect();
                        cells[index] = "";
                        output.push_str(&cells.join("\t"));
                        output.push('\n');
                    }
                    record["exchanges"][0]["body"]["value"] = output.into();
                }
            }
            std::fs::write(
                directory
                    .path()
                    .join(source)
                    .join(path.file_name().unwrap()),
                serde_json::to_vec(&record).unwrap(),
            )
            .unwrap();
        }
    }
    let fetch = replay(directory.path().to_owned());
    let query = Request::GetGene(
        GeneGet::with_sections(
            vec!["TP53".parse().unwrap(), "MLH1".parse().unwrap()].into(),
            vec![GeneSectionWord::ClinGen].into(),
        )
        .unwrap(),
    );
    let outcome = run(
        &Ctx {
            ncbi_email: None,
            fetch: &fetch,
            budget: Duration::from_millis(1500),
            fresh: false,
            drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
        },
        &query,
    );
    assert_eq!(
        outcome
            .sources
            .iter()
            .find(|source| source.source == biomcp_types::Source::ClinGen)
            .unwrap()
            .reason,
        Some(biomcp_types::Reason::Syncing)
    );
    assert!(
        matches!(outcome.local, Some(RequestError::Recording(_))),
        "a later source failure must retain the earlier local error: {:?}",
        outcome.local
    );
}

#[test]
fn concurrent_replay_annotations_and_bundle_publication_keep_checked_sections() {
    use biomcp_query::{Ctx, run};
    use biomcp_types::{GeneGet, GeneSectionWord, Request, Status};
    let query = Request::GetGene(
        GeneGet::with_sections(
            ["MLH1", "MSH2", "MSH6", "PMS2", "EPCAM"]
                .into_iter()
                .map(|id| id.parse().unwrap())
                .collect(),
            vec![GeneSectionWord::ClinGen].into(),
        )
        .unwrap(),
    );
    // Feature: cache-resilience. The public query fans out annotation admission
    // and bundle publication on the same Fetch; each iteration owns its cache.
    for attempt in 0..16 {
        let fetch = replay(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/replies/gene/clingen"),
        );
        let outcome = run(
            &Ctx {
                ncbi_email: None,
                fetch: &fetch,
                budget: Duration::from_secs(20),
                fresh: false,
                drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
            },
            &query,
        );
        assert_eq!(
            outcome.status(),
            Status::Complete,
            "concurrent replay attempt {attempt}: local={:?}; sources={:?}",
            outcome.local,
            outcome.sources
        );
        assert!(outcome.local.is_none());
    }
}
