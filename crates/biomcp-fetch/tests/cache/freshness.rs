use super::*;
use biomcp_fetch::{Input, Read};
#[path = "absence.rs"]
mod absence;

#[test]
fn fresh_hits_skip_quota_and_stale_positives_survive_quota_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let quota = Box::leak(Box::new(Quota::new(
        SourceKey::declare("cache-blocked"),
        Rate::per_second(1000),
        None,
        1,
    )));
    let spec = server.spec(quota, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    thread::scope(|scope| {
        let held =
            scope.spawn(|| get(&fetch, spec, &spec.get("/answer"), &[("hold", Arg::Num(1))]));
        let request = server.next();
        assert!(
            fetch
                .leg(spec, fetch.deadline(Duration::from_millis(100)))
                .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value)
                .unwrap()
                .answer
        );
        expire(&path);
        let leg = fetch.leg(spec, fetch.deadline(Duration::from_millis(100)));
        assert!(
            leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value)
                .unwrap()
                .answer
        );
        let report = leg.finish();
        assert!(report.stale);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].reason, Reason::DeadlineExceeded);
        assert_eq!(report.failures[0].status, None);
        assert!(server.requests.try_recv().is_err());
        request.ok();
        held.join().unwrap().unwrap();
    });
}

#[test]
fn body_and_send_failures_can_use_only_a_checked_stale_positive() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    expire(&path);
    let report = thread::scope(|scope| {
        let worker = scope.spawn(|| {
            let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
            assert!(
                leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                    .map(|received| received.value)
                    .unwrap()
                    .answer
            );
            leg.finish()
        });
        server
            .next()
            .raw("HTTP/1.1 200 Stub\r\nContent-Type: application/json\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{cut");
        worker.join().unwrap()
    });
    assert_eq!(report.failures[0].reason, Reason::BadReply);
    assert_eq!(report.failures[0].status, Some(200));
    drop(server);
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    assert!(
        leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
            .unwrap()
            .answer
    );
    assert_eq!(leg.finish().failures[0].reason, Reason::Unavailable);
}

#[test]
fn local_write_busy_keeps_the_checked_live_reply_and_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    expire(&path);
    let connection = Connection::open(&path).unwrap();
    connection.execute_batch("BEGIN IMMEDIATE").unwrap();
    let deadline = fetch.deadline(Duration::from_millis(500));
    let (reply, report) = exchange(&server, 200, r#"{"answer":true}"#, || {
        let leg = fetch.leg(spec, deadline);
        let reply = leg
            .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
            .unwrap();
        (reply, leg.finish())
    });
    assert!(reply.answer);
    assert_eq!(report.cache_problems, [CacheError::Busy]);
    assert!(deadline.remaining() < Duration::from_millis(20));
    assert!(!report.stale);
    assert!(report.failures.is_empty());
    connection.execute_batch("ROLLBACK").unwrap();
}

#[test]
fn per_leg_freshness_reaches_batch_children_without_leaking_to_siblings() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    let fresh = fetch
        .leg(spec, fetch.deadline(Duration::from_secs(10)))
        .fresh(true);
    exchange(&server, 200, r#"{"answer":true,"fresh":true}"#, || {
        let results = fresh.batch(&[()], |leg, ()| {
            leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value)
        });
        assert!(results.into_iter().all(|result| result.unwrap().answer));
    });
    assert!(get(&fetch, spec, &endpoint, &[]).unwrap().answer);
    assert!(server.requests.try_recv().is_err());
    expire(&path);
    let failure = exchange(&server, 400, "rejected", || {
        fresh
            .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value)
    })
    .unwrap_err();
    assert_eq!(source(failure).reason, Reason::Rejected);
    assert!(!fresh.finish().stale);
}

#[test]
fn concurrent_replay_legs_share_the_cache_without_spurious_local_busy_errors() {
    use sha2::{Digest, Sha256};
    let directory = tempfile::tempdir().unwrap();
    let records = directory.path().join("pubmed");
    std::fs::create_dir(&records).unwrap();
    let spec: &'static SourceSpec = Box::leak(Box::new(SourceSpec {
        destinations: biomcp_fetch::Destinations::Primary,
        user_agent: None,
        terms: biomcp_fetch::Terms {
            name: "Test terms",
            url: "https://example.test/terms",
            usage: biomcp_types::Use::Open,
        },
        source: biomcp_types::Source::PubMed,
        base: "https://example.invalid",
        key: KeyUse::None,
        quota: Box::leak(Box::new(Quota::new(
            SourceKey::declare("parallel-cache"),
            Rate::new(1000, 1000),
            None,
            8,
        ))),
        deadline: Duration::from_secs(10),
    }));
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    for index in 0..8 {
        let url = format!("https://example.invalid/answer?index={index}");
        let key = Sha256::digest(format!("GET\0{url}").as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let recording = serde_json::json!(
            {"method":"GET","source":"pubmed","url":url,"exchanges":[{"status":200,"content_type":"application/json","delay_ms":0,
            "retry_after_ms":null,"body":{"kind":"json","value":{"answer":true}}}]}
        );
        std::fs::write(
            records.join(format!("{key}.json")),
            serde_json::to_vec(&recording).unwrap(),
        )
        .unwrap();
    }
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay {
            dir: directory.path().into(),
        },
        ..Default::default()
    })
    .unwrap();
    let start = std::sync::Barrier::new(8);
    thread::scope(|scope| {
        let workers: Vec<_> = (0..8)
            .map(|index| {
                let fetch = &fetch;
                let start = &start;
                let endpoint = &endpoint;
                scope.spawn(move || {
                    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
                    start.wait();
                    leg.call(
                        endpoint,
                        Input::Params(&[("index", Arg::Num(index))]),
                        Read::<Answer>::json(),
                    )
                    .map(|received| received.value)
                })
            })
            .collect();
        for worker in workers {
            assert!(worker.join().unwrap().unwrap().answer);
        }
    });
}

#[test]
fn mixed_cached_and_live_replies_keep_their_own_fetch_times() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[("id", Arg::Num(1))])
    })
    .unwrap();
    assert_eq!(
        Connection::open(&path)
            .unwrap()
            .execute("UPDATE reply SET fetched_at=1234567890", [])
            .unwrap(),
        1
    );
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    let cached_ready = std::sync::Barrier::new(2);
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            leg.batch(&[1, 2], |child, id| {
                // Live cache reads are opportunistic; keep the live miss from
                // contending with the cached read whose timestamp this checks.
                if *id == 2 {
                    cached_ready.wait();
                }
                let reply = child.call(
                    &endpoint,
                    Input::Params(&[("id", Arg::Num(*id))]),
                    Read::<Answer>::json(),
                );
                if *id == 1 {
                    cached_ready.wait();
                }
                reply
            })
        });
        let request = server.next();
        assert!(request.text.contains("id=2"));
        request.ok();
        assert!(
            worker
                .join()
                .unwrap()
                .into_iter()
                .all(|row| row.unwrap().value.answer)
        );
    });
    let report = leg.finish();
    assert_eq!(report.replies.len(), 2);
    assert_ne!(report.replies[0].identity, report.replies[1].identity);
    assert_eq!(
        report
            .replies
            .iter()
            .filter(|r| r.fetched == Some(1234567890))
            .count(),
        1
    );
    assert_eq!(
        report
            .replies
            .iter()
            .filter(|r| r.fetched > Some(1234567890))
            .count(),
        1
    );
    let offline = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        ..config(&path)
    })
    .unwrap();
    let leg = offline.leg(spec, offline.deadline(Duration::from_secs(10)));
    leg.call(
        &endpoint,
        Input::Params(&[("id", Arg::Num(1))]),
        Read::<Answer>::json(),
    )
    .unwrap();
    assert_eq!(leg.finish().replies[0].fetched, Some(1234567890));
}

#[test]
fn native_search_headers_survive_record_replay_and_offline_cache() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("native.sqlite");
    let recordings = directory.path().join("recordings");
    let server = Server::new();
    let original = spec(&server, KeyUse::None);
    let spec = Box::leak(Box::new(SourceSpec {
        destinations: biomcp_fetch::Destinations::Primary,
        user_agent: None,
        source: biomcp_types::Source::UniProt,
        base: original.base,
        key: KeyUse::None,
        quota: original.quota,
        deadline: original.deadline,
        terms: biomcp_fetch::Terms {
            name: "Synthetic",
            url: "https://example.test/terms",
            usage: biomcp_types::Use::Open,
        },
    }));
    let endpoint = spec.get("/uniprotkb/search").fresh(Freshness::Hours(1));
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: recordings.clone(),
        },
        ..config(&path)
    })
    .unwrap();
    let run = |fetch: &Fetch| {
        let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
        let (reply, meta) = leg
            .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| (received.value, received.meta))
            .unwrap();
        assert!(reply.answer);
        let page = meta.pagination.unwrap();
        assert_eq!(page.total, 17);
        assert_eq!(
            page.next.as_deref(),
            Some(
                "https://rest.uniprot.org/uniprotkb/search?fields=accession,id&cursor=native-cursor"
            )
        );
    };
    thread::scope(|scope| {
        let worker = scope.spawn(|| run(&fetch));
        server.next().respond(
            200,
            "X-Total-Results: 17\r\nLink: \
            <https://rest.uniprot.org/uniprotkb/search?fields=accession,id&cursor=native-cursor>; \
            rel=\"next\"\r\n",
            r#"{"answer":true}"#,
        );
        worker.join().unwrap();
    });
    drop(fetch);
    run(&Fetch::open(config(&path)).unwrap());
    run(&Fetch::open(FetchConfig {
        mode: Mode::Offline,
        ..config(&path)
    })
    .unwrap());
    run(&Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: recordings },
        ..FetchConfig::default()
    })
    .unwrap());
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn endpoint_json_budget_rejects_oversize_live_replay_and_cached_replies() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("budget.sqlite");
    let recordings = directory.path().join("recordings");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let wide = spec.get("/answer").bytes(256).fresh(Freshness::Hours(1));
    let narrow = wide.clone().bytes(32);
    let body = r#"{"answer":true,"padding":"synthetic source body beyond the endpoint budget"}"#;
    assert!(body.len() > 32);
    let recorder = Fetch::open(FetchConfig {
        mode: Mode::Record {
            dir: recordings.clone(),
        },
        ..config(&path)
    })
    .unwrap();
    assert!(
        exchange(&server, 200, body, || get(&recorder, spec, &wide, &[]))
            .unwrap()
            .answer
    );
    drop(recorder);
    let offline = Fetch::open(FetchConfig {
        mode: Mode::Offline,
        ..config(&path)
    })
    .unwrap();
    assert!(
        get(&offline, spec, &narrow, &[]).is_err(),
        "cached bytes bypassed the smaller endpoint budget"
    );
    let live = Fetch::open(config(&path)).unwrap();
    let failure = exchange(&server, 200, body, || get(&live, spec, &narrow, &[]));
    assert_eq!(source(failure.unwrap_err()).reason, Reason::BadReply);
    let replay = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: recordings },
        ..FetchConfig::default()
    })
    .unwrap();
    assert!(
        get(&replay, spec, &narrow, &[]).is_err(),
        "recorded bytes bypassed the endpoint budget"
    );
}

#[derive(serde::Deserialize)]
struct Absence {
    absent: bool,
}
impl Reply for Absence {
    fn check(&self) -> Result<(), BadReply> {
        if self.absent { Ok(()) } else { Err(BadReply) }
    }
    fn check_status(&self, status: u16) -> Result<(), BadReply> {
        if status == 404 {
            self.check()
        } else {
            Err(BadReply)
        }
    }
}

#[test]
fn busy_read_is_a_miss_before_the_live_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    let deadline = fetch.deadline(Duration::from_secs(5));
    let access = fetch.cache_access().unwrap();
    let report = thread::scope(|scope| {
        let worker = access
            .admit(deadline, || {
                let worker = scope.spawn(|| {
                    let leg = fetch.leg(spec, deadline);
                    assert!(
                        leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                            .map(|received| received.value)
                            .unwrap()
                            .answer
                    );
                    leg.finish()
                });
                // Observing HTTP while the cache is still held proves the read did
                // not spend the source budget waiting for the cache connection.
                server
                    .requests
                    .recv_timeout(Duration::from_secs(10))
                    .unwrap()
                    .ok();
                Ok(worker)
            })
            .unwrap();
        worker.join().unwrap()
    });
    assert_eq!(report.cache_problems, [CacheError::Busy]);
    assert!(report.failures.is_empty());
    assert!(!report.stale);
}

#[test]
fn negative_admission_and_json_absence_removal_keep_the_source_answer() {
    for json_absence in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("cache.sqlite");
        let server = Server::new();
        let spec = spec(&server, KeyUse::None);
        let fetch = Fetch::open(config(&path)).unwrap();
        let mut endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
        exchange(&server, 200, r#"{"answer":true}"#, || {
            get(&fetch, spec, &endpoint, &[])
        })
        .unwrap();
        if json_absence {
            endpoint = endpoint.json_absence();
        }
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch("BEGIN IMMEDIATE").unwrap();
        let (result, report) = exchange(&server, 404, r#"{"absent":true}"#, || {
            let leg = fetch
                .leg(spec, fetch.deadline(Duration::from_secs(10)))
                .fresh(true);
            let result = if json_absence {
                leg.call(&endpoint, Input::Params(&[]), Read::<Absence>::json())
                    .map(|received| received.value)
                    .map(|reply| reply.absent)
            } else {
                leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                    .map(|received| received.value)
                    .map(|reply| reply.answer)
            };
            (result, leg.finish())
        });
        if json_absence {
            assert!(result.unwrap());
        } else {
            assert_eq!(source(result.unwrap_err()).reason, Reason::NotFound);
        }
        assert_eq!(report.cache_problems, [CacheError::Busy]);
        assert!(report.failures.is_empty());
        assert!(!report.stale);
    }
    absence::legacy_json_absence_removes_a_positive();
}

#[test]
fn failed_invalidation_removal_cannot_reuse_an_unchecked_stale_entry() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let server = Server::new();
    let spec = spec(&server, KeyUse::None);
    let fetch = Fetch::open(config(&path)).unwrap();
    let endpoint = spec.get("/answer").fresh(Freshness::Hours(1));
    exchange(&server, 200, r#"{"answer":true}"#, || {
        get(&fetch, spec, &endpoint, &[])
    })
    .unwrap();
    let connection = Connection::open(&path).unwrap();
    connection
        .execute(
            "UPDATE reply SET body=?1, fresh_until=0",
            [b"{}".as_slice()],
        )
        .unwrap();
    connection.execute_batch("BEGIN IMMEDIATE").unwrap();
    let (result, report) = exchange(&server, 400, "rejected", || {
        let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(2)));
        let result = leg
            .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
            .map(|received| received.value);
        (result, leg.finish())
    });
    assert_eq!(source(result.unwrap_err()).reason, Reason::Rejected);
    assert_eq!(report.cache_problems, [CacheError::Busy]);
    assert!(!report.stale);
}
