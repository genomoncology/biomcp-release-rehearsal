//! Public stored article searches retain snapshots, headers and local failures.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "Public store and concurrency contracts own isolated fixture I/O and abort on invalid fixture setup"
)]

use biomcp_fetch::{CacheError, Fetch, FetchConfig, Mode, ReplayCache, RequestError};
use biomcp_query::{Ctx, Outcome};
use biomcp_test_support::recordings as recording_support;
use biomcp_types::{ArticlePage, ArticleSearch, ReplyData, Request, Status};
use std::{path::PathBuf, time::Duration};
fn recordings(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/replies/article/search")
        .join(name)
}
fn fetch(name: &str, cache: ReplayCache) -> Fetch {
    Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: recordings(name),
        },
        replay_cache: Some(cache),
        cache_owners: vec![biomcp_query::store::OWNER],
        ..FetchConfig::default()
    })
    .unwrap()
}
fn query(pool: Option<&str>, offset: u32) -> ArticleSearch {
    let query: ArticleSearch = serde_json::from_str(r#"{"text":"AKT1 E17K","limit":2}"#).unwrap();
    query.page(offset, pool.map(|value| value.parse().unwrap()))
}
fn run(fetch: &Fetch, query: ArticleSearch, fresh: bool) -> Outcome {
    biomcp_query::run(
        &Ctx {
            fetch,
            budget: Duration::from_secs(20),
            fresh,
            ncbi_email: None,
        },
        &Request::SearchArticle(query),
    )
}
fn page(out: Outcome) -> (biomcp_types::PageInfo, ArticlePage) {
    assert_eq!(
        out.local, None,
        "Article pages must preserve their snapshot and paging contract"
    );
    assert_eq!(
        out.status(),
        Status::Complete,
        "Article pages must preserve their snapshot and paging contract"
    );
    let ReplyData::ArticlePage {
        body: Some(page), ..
    } = out.body
    else {
        panic!("article page")
    };
    (out.page.unwrap(), page)
}
#[test]
fn expiring_one_snapshot_does_not_expire_its_replacement() {
    let cache = ReplayCache::temporary().unwrap();
    let fetch = fetch("synthetic-small", cache);
    let (first, _) = page(run(&fetch, query(None, 0), false));
    let old = first.pool.unwrap();
    let (second, _) = page(run(&fetch, query(None, 0), true));
    let new = second.pool.unwrap();
    let connection = fetch.cache_access().unwrap().connect().unwrap();
    connection
        .execute("UPDATE pool SET expires=0 WHERE id=?1", [old.id.as_str()])
        .unwrap();
    assert!(matches!(
        run(&fetch, query(Some(old.id.as_str()), 0), false).body,
        ReplyData::PoolError { mismatch: false }
    ));
    assert_eq!(
        page(run(&fetch, query(Some(new.id.as_str()), 0), false))
            .0
            .pool,
        Some(new)
    );
}
#[test]
fn store_corruption_and_exhausted_budget_remain_typed_local_failures() {
    let cache = ReplayCache::temporary().unwrap();
    let fetch = fetch("synthetic-small", cache);
    let (first, _) = page(run(&fetch, query(None, 0), false));
    let id = first.pool.unwrap().id;
    fetch
        .cache_access()
        .unwrap()
        .connect()
        .unwrap()
        .execute(
            "UPDATE pool SET outcomes='invalid' WHERE id=?1",
            [id.as_str()],
        )
        .unwrap();
    let out = run(&fetch, query(Some(id.as_str()), 0), false);
    assert_eq!(out.status(), Status::Failed);
    assert_eq!(out.local, Some(RequestError::Cache(CacheError::Encoding)));
    assert!(out.sources.is_empty());
    fetch
        .cache_access()
        .unwrap()
        .connect()
        .unwrap()
        .execute_batch("ALTER TABLE pool_current RENAME COLUMN request_key TO corrupt_key")
        .unwrap();
    let sql_failure = run(&fetch, query(None, 0), false);
    assert_eq!(
        sql_failure.local,
        Some(RequestError::Cache(CacheError::Database))
    );
    assert!(sql_failure.sources.is_empty());
    let out = biomcp_query::run(
        &Ctx {
            fetch: &fetch,
            budget: Duration::ZERO,
            fresh: false,
            ncbi_email: None,
        },
        &Request::SearchArticle(query(None, 0)),
    );
    assert_eq!(out.local, Some(RequestError::Cache(CacheError::Busy)));
}
#[test]
fn concurrent_commands_publish_distinct_complete_snapshots() {
    let cache = ReplayCache::temporary().unwrap();
    let fetch = fetch("synthetic-small", cache);
    page(run(&fetch, query(None, 0), false));
    let outcomes = std::thread::scope(|scope| {
        let one = scope.spawn(|| run(&fetch, query(None, 0), true));
        let two = scope.spawn(|| run(&fetch, query(None, 0), true));
        [one.join().unwrap(), two.join().unwrap()]
    });
    let mut ids = std::collections::BTreeSet::new();
    for outcome in outcomes {
        let (p, items) = page(outcome);
        let id = p.pool.unwrap().id;
        assert!(ids.insert(id.to_string()));
        assert_eq!(items.items.len(), 2);
        assert_ne!(items.items[0].pmid, items.items[1].pmid);
    }
    for id in ids {
        assert_eq!(
            page(run(&fetch, query(Some(&id), 0), false)).1.items.len(),
            2
        );
    }
}

#[test]
fn search_and_hydration_share_one_budget_while_pool_age_uses_wall_time() {
    let cache = ReplayCache::temporary().unwrap();
    let fetch = fetch("synthetic-deadline", cache);
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let first = biomcp_query::run(
        &Ctx {
            fetch: &fetch,
            budget: Duration::from_millis(300),
            fresh: false,
            ncbi_email: None,
        },
        &Request::SearchArticle(query(None, 0)),
    );
    assert_eq!(first.status(), Status::Failed);
    assert_eq!(first.local, None);
    assert_eq!(
        first.sources[0].reason,
        Some(biomcp_types::Reason::DeadlineExceeded)
    );
    let biomcp_types::StepKind::Fresh = first.next[0].0 else {
        panic!("fresh action")
    };
    let biomcp_query::Action::Run {
        request: Request::SearchArticle(retry),
        ..
    } = first.next[0].1.clone()
    else {
        panic!("retry retains pool")
    };
    assert!(retry.pool().is_some());
    let (p, _) = page(run(&fetch, retry, false));
    let pool = p.pool.unwrap();
    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!((before..=after).contains(&pool.created));
    assert_eq!(pool.expires - pool.created, 86400);
}

#[test]
fn concurrent_hydration_retains_accepted_headers_and_snapshot_metadata() {
    let cache = ReplayCache::temporary().unwrap();
    let fetch = fetch("synthetic-retry", cache);
    let first = run(&fetch, query(None, 0), false);
    let info = first.page.unwrap().pool.unwrap();
    let ReplyData::ArticlePage {
        body: Some(before), ..
    } = first.body
    else {
        panic!("article page")
    };
    assert!(before.items[0].header.is_none());
    let outcomes = std::thread::scope(|scope| {
        let one = scope.spawn(|| run(&fetch, query(Some(info.id.as_str()), 0), true));
        let two = scope.spawn(|| run(&fetch, query(Some(info.id.as_str()), 0), true));
        [one.join().unwrap(), two.join().unwrap()]
    });
    for out in outcomes {
        let (page, items) = page(out);
        assert_eq!(page.pool, Some(info.clone()));
        assert!(items.items[0].header.is_some());
        assert_eq!(items.items[1], before.items[1]);
    }
    let (_, items) = page(run(&fetch, query(Some(info.id.as_str()), 0), false));
    assert!(items.items.iter().all(|i| i.header.is_some()));
}

#[test]
fn sequential_replay_commands_receive_new_budgets_after_provider_quota_waits() {
    let cache = ReplayCache::temporary().unwrap();
    let fetch = fetch("synthetic-small", cache);
    for _ in 0..6 {
        let out = biomcp_query::run(
            &Ctx {
                fetch: &fetch,
                budget: Duration::from_secs(2),
                fresh: true,
                ncbi_email: None,
            },
            &Request::SearchArticle(query(None, 0)),
        );
        page(out);
    }
}
#[test]
fn replay_stages_count_elapsed_response_time_once_when_waiting_for_quota() {
    let directory = tempfile::tempdir().unwrap();
    let pubmed = directory.path().join("pubmed");
    std::fs::create_dir(&pubmed).unwrap();
    for entry in std::fs::read_dir(recordings("synthetic-small").join("pubmed")).unwrap() {
        let entry = entry.unwrap();
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(entry.path()).unwrap()).unwrap();
        let delay = if value["method"] == "GET" { 200 } else { 0 };
        for exchange in value["exchanges"].as_array_mut().unwrap() {
            exchange["delay_ms"] = delay.into();
        }
        std::fs::write(
            pubmed.join(entry.file_name()),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
    }
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: directory.path().to_owned(),
        },
        cache_owners: vec![biomcp_query::store::OWNER],
        ..Default::default()
    })
    .unwrap();
    let out = biomcp_query::run(
        &Ctx {
            fetch: &fetch,
            budget: Duration::from_millis(400),
            fresh: true,
            ncbi_email: None,
        },
        &Request::SearchArticle(query(None, 0)),
    );
    page(out);
}
#[test]
fn overlapping_replay_commands_complete_without_adding_each_others_spend() {
    let cache = ReplayCache::temporary().unwrap();
    let fetch = fetch("synthetic-small", cache);
    for _ in 0..3 {
        let outcomes = std::thread::scope(|scope| {
            let one = scope.spawn(|| {
                biomcp_query::run(
                    &Ctx {
                        fetch: &fetch,
                        budget: Duration::from_secs(2),
                        fresh: true,
                        ncbi_email: None,
                    },
                    &Request::SearchArticle(query(None, 0)),
                )
            });
            let two = scope.spawn(|| {
                biomcp_query::run(
                    &Ctx {
                        fetch: &fetch,
                        budget: Duration::from_secs(2),
                        fresh: true,
                        ncbi_email: None,
                    },
                    &Request::SearchArticle(query(None, 0)),
                )
            });
            [one.join().unwrap(), two.join().unwrap()]
        });
        let ids = outcomes
            .into_iter()
            .map(|out| page(out).0.pool.unwrap().id.to_string())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(ids.len(), 2);
    }
}

#[test]
fn stale_summary_fallback_stays_partial_when_its_page_is_read_offline() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("case");
    let cache = ReplayCache::create_case_with_owners(&path, &[]).unwrap();
    let recordings = directory.path().join("recordings");
    recording_support::copy_capture(
        &self::recordings("synthetic-small"),
        &recordings,
        |_, record| {
            recording_support::cache_acquisition(record, 42);
        },
    )
    .unwrap();
    let replay = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: recordings },
        replay_cache: Some(cache),
        cache_owners: vec![biomcp_query::store::OWNER],
        ..Default::default()
    })
    .unwrap();
    let (first, _) = page(run(&replay, query(None, 0), false));
    let id = first.pool.unwrap().id;
    let connection = replay.cache_access().unwrap().connect().unwrap();
    connection
        .execute_batch("UPDATE pool_item SET checked=0,header=NULL; UPDATE reply SET fresh_until=0")
        .unwrap();
    drop(connection);
    drop(replay);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    drop(listener);
    let live = Fetch::open(FetchConfig {
        cache: Some(path.join("replay.sqlite")),
        proxy: Some(biomcp_fetch::Secret::new(format!("http://{address}"))),
        cache_owners: vec![biomcp_query::store::OWNER],
        ..Default::default()
    })
    .unwrap();
    let first = run(&live, query(Some(id.as_str()), 0), false);
    assert_eq!(first.local, None);
    assert_eq!(first.status(), Status::Partial);
    assert_eq!(first.sources[0].state, biomcp_types::SourceState::Stale);
    let biomcp_query::Action::Run {
        request: Request::SearchArticle(retry),
        fresh: true,
    } = first
        .next
        .iter()
        .find(|step| step.0 == biomcp_types::StepKind::Fresh)
        .unwrap()
        .1
        .clone()
    else {
        panic!("fresh replacement")
    };
    assert!(retry.pool().is_none());
    drop(live);
    let offline = Fetch::open(FetchConfig {
        cache: Some(path.join("replay.sqlite")),
        mode: Mode::Offline,
        cache_owners: vec![biomcp_query::store::OWNER],
        ..Default::default()
    })
    .unwrap();
    let repeated = run(&offline, query(Some(id.as_str()), 0), false);
    assert_eq!(repeated.local, None);
    assert_eq!(repeated.status(), Status::Partial);
    assert_eq!(repeated.sources, first.sources);
    let ReplyData::ArticlePage {
        body: Some(body), ..
    } = repeated.body
    else {
        panic!("article page")
    };
    assert!(body.items.iter().all(|i| i.header.is_some()));
}
