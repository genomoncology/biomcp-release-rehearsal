//! Scoped chunk callbacks preserve caller order and transport ownership.
#![expect(
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "Scoped public API tests use checked local fixtures; setup and worker failures must fail the test"
)]
use biomcp_fetch::{Input, Read};

mod support;
use biomcp_fetch::{Fetch, FetchConfig, KeyUse, Quota, Rate, RequestError, SourceKey};
#[expect(
    clippy::disallowed_types,
    reason = "Import only the counter or cell owned by this test fixture"
)]
use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use support::Answer;
#[path = "support/timing.rs"]
mod timing;
use timing::Server;

fn quota(in_flight: u8) -> &'static Quota {
    Box::leak(Box::new(Quota::new(
        SourceKey::declare("batch"),
        Rate::new(1000, 1000),
        None,
        in_flight,
    )))
}

#[test]
fn overlapping_requests_keep_input_order_and_independent_failures() {
    let server = Server::new();
    let spec = server.spec(quota(3), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(10)));
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            leg.batch(&["/0", "/1", "/2"], |leg, chunk| {
                leg.call(&spec.get(chunk), Input::Params(&[]), Read::<Answer>::json())
                    .map(|received| received.value)
            })
        });
        let requests: Vec<_> = (0..3).map(|_| server.next()).collect();
        for request in requests.into_iter().rev() {
            if request.text.contains("/1 ") {
                request.respond(400, "", "refused");
            } else {
                request.ok();
            }
        }
        let results = worker.join().unwrap();
        assert!(results[0].as_ref().unwrap().answer);
        assert!(
            matches!(results[1], Err(RequestError::Source(error)) if error.reason == biomcp_types::Reason::Rejected)
        );
        assert!(results[2].as_ref().unwrap().answer);
    });
}

#[test]
fn worker_bound_and_inline_nesting_hold_across_multiple_chunks() {
    let server = Server::new();
    let spec = server.spec(quota(1), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(30)));
    let chunks = vec![1; biomcp_fetch::MAX_BATCH_WORKERS * 2];
    let (sent, receive) = mpsc::channel();
    thread::scope(|scope| {
        let work = scope.spawn(|| {
            leg.batch(&chunks, |leg, value| {
                let worker = thread::current().id();
                let nested = leg.batch(&[3, 4], |_, inner| {
                    assert_eq!(worker, thread::current().id());
                    Ok(value + inner)
                });
                let (release, wait) = mpsc::channel();
                sent.send((worker, release)).unwrap();
                wait.recv_timeout(Duration::from_secs(30)).unwrap();
                nested.into_iter().collect::<Result<Vec<_>, _>>()
            })
        });
        // Every admitted worker overlaps, without assuming which one arrived first.
        let held: Vec<_> = (0..biomcp_fetch::MAX_BATCH_WORKERS)
            .map(|_| receive.recv_timeout(Duration::from_secs(30)).unwrap())
            .collect();
        let mut threads = HashSet::new();
        for (id, release) in held {
            threads.insert(id);
            release.send(()).unwrap();
        }
        for _ in biomcp_fetch::MAX_BATCH_WORKERS..chunks.len() {
            let (id, release) = receive.recv_timeout(Duration::from_secs(30)).unwrap();
            threads.insert(id);
            release.send(()).unwrap();
        }
        assert_eq!(threads.len(), biomcp_fetch::MAX_BATCH_WORKERS);
        assert_eq!(work.join().unwrap(), vec![Ok(vec![4, 5]); chunks.len()]);
    });
}

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "Observe owned test workers without leaking shared state across cases"
)]
fn empty_and_refused_batches_never_execute_callbacks() {
    let server = Server::new();
    let plain = server.spec(quota(1), KeyUse::None);
    let required = server.spec(
        quota(1),
        KeyUse::Required(biomcp_fetch::Credential {
            tier: biomcp_types::KeyTier::Fallback,
            secret: biomcp_fetch::SecretName::NcbiApiKey,
            place: biomcp_fetch::KeyPlace::Header("x-api-key"),
        }),
    );
    let live = Fetch::open(FetchConfig::default()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let replay = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Replay {
            dir: dir.path().to_owned(),
        },
        ..FetchConfig::default()
    })
    .unwrap();
    let foreign_expired = replay.deadline(Duration::ZERO);
    let calls = AtomicUsize::new(0);
    for (spec, deadline, reason) in [
        (required, foreign_expired, biomcp_types::Reason::NoKey),
        (plain, foreign_expired, biomcp_types::Reason::Rejected),
        (
            plain,
            live.deadline(Duration::ZERO),
            biomcp_types::Reason::DeadlineExceeded,
        ),
    ] {
        let leg = live.leg(spec, deadline);
        let callback = |_: &biomcp_fetch::Leg<'_>, _: &u8| {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        };
        assert!(leg.batch(&[], callback).is_empty());
        let results = leg.batch(&[1, 2], callback);
        assert!(results.into_iter().all(
            |result| matches!(result, Err(RequestError::Source(error)) if error.reason == reason)
        ));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(server.requests.try_recv().is_err());
}

#[test]
#[expect(
    clippy::disallowed_types,
    reason = "Observe owned test workers without leaking shared state across cases"
)]
fn panic_propagates_only_after_other_workers_finish() {
    let server = Server::new();
    let spec = server.spec(quota(1), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(30)));
    let (sent, receive) = mpsc::channel();
    let done = AtomicUsize::new(0);
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                leg.batch(&[true, false], |leg, panic| {
                    if *panic {
                        sent.send(()).unwrap();
                        panic!("callback failed");
                    }
                    let reply = leg
                        .call(
                            &spec.get("/held"),
                            Input::Params(&[]),
                            Read::<Answer>::json(),
                        )?
                        .value;
                    done.fetch_add(1, Ordering::SeqCst);
                    Ok(reply)
                })
            }))
        });
        receive.recv_timeout(Duration::from_secs(30)).unwrap();
        server.next().ok();
        assert!(worker.join().unwrap().is_err());
        assert_eq!(done.load(Ordering::SeqCst), 1);
    });
    // A later request still gets the provider slot after scoped cleanup.
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            leg.call(
                &spec.get("/after"),
                Input::Params(&[]),
                Read::<Answer>::json(),
            )
            .map(|received| received.value)
        });
        server.next().ok();
        assert!(worker.join().unwrap().unwrap().answer);
    });
}

#[test]
fn provider_slots_bound_a_batch_independently_of_workers() {
    let server = Server::new();
    let spec = server.spec(quota(2), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(30)));
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            leg.batch(&[(); 8], |leg, ()| {
                leg.call(
                    &spec.get("/quota"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
            })
        });
        let mut held = server.next();
        let mut next = server.next();
        for _ in 2..8 {
            let released = std::time::Instant::now();
            held.ok();
            held = next;
            next = server.next();
            assert!(
                next.started >= released,
                "provider slot reused before a response release"
            );
        }
        held.ok();
        next.ok();
        assert!(
            worker
                .join()
                .unwrap()
                .into_iter()
                .all(|reply| reply.unwrap().answer)
        );
    });
}

#[derive(Debug, serde::Deserialize)]
struct Number {
    value: usize,
}
impl biomcp_fetch::Reply for Number {
    fn check(&self) -> Result<(), biomcp_fetch::BadReply> {
        Ok(())
    }
}

fn recording(
    server: &Server,
    spec: &'static biomcp_fetch::SourceSpec,
    count: usize,
) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let capture = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Record {
            dir: dir.path().to_owned(),
        },
        ..FetchConfig::default()
    })
    .unwrap();
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            for value in 0..count {
                let leg = capture.leg(spec, capture.deadline(Duration::from_secs(30)));
                assert_eq!(
                    leg.call(
                        &spec.get("/number"),
                        Input::Params(&[]),
                        Read::<Number>::json()
                    )
                    .map(|received| received.value)
                    .unwrap()
                    .value,
                    value,
                    "Batch capture must retain every ordered source request"
                );
            }
        });
        for value in 0..count {
            server
                .next()
                .respond(200, "", &format!("{{\"value\":{value}}}"));
        }
        worker.join().unwrap();
    });
    drop(capture);
    for file in std::fs::read_dir(dir.path().join("pubmed")).unwrap() {
        let file = file.unwrap().path();
        let mut data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        for exchange in data["exchanges"].as_array_mut().unwrap() {
            exchange["delay_ms"] = 1000.into();
        }
        std::fs::write(file, serde_json::to_vec(&data).unwrap()).unwrap();
    }
    dir
}

fn replay(dir: &std::path::Path) -> Fetch {
    Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Replay {
            dir: dir.to_owned(),
        },
        source_deadlines: [(biomcp_types::Source::PubMed, Duration::from_secs(30))].into(),
        ..FetchConfig::default()
    })
    .unwrap()
}

#[test]
fn replay_retains_worker_progress_and_parent_joins_the_greatest_completion() {
    let server = Server::new();
    let spec = server.spec(quota(8), KeyUse::None);
    let dir = recording(&server, spec, 17);
    let fetch = replay(dir.path());
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(30)));
    let phase = std::sync::Barrier::new(biomcp_fetch::MAX_BATCH_WORKERS);
    let results = leg.batch(&[(); 16], |child, ()| {
        let number = child
            .call(
                &spec.get("/number"),
                Input::Params(&[]),
                Read::<Number>::json(),
            )?
            .value;
        let remaining = child.remaining();
        // Every worker is admitted; finish the round before any worker starts its next chunk.
        phase.wait();
        Ok((number.value, remaining))
    });
    let mut values = Vec::new();
    for (index, result) in results.into_iter().enumerate() {
        let (value, remaining) = result.unwrap();
        values.push(value);
        assert_eq!(
            remaining,
            Duration::from_secs(if index < 8 { 29 } else { 28 })
        );
    }
    values.sort();
    assert_eq!(values, (0..16).collect::<Vec<_>>());
    assert_eq!(leg.remaining(), Duration::from_secs(28));
    assert_eq!(
        leg.batch(&[()], |child, ()| child
            .call(
                &spec.get("/number"),
                Input::Params(&[]),
                Read::<Number>::json()
            )
            .map(|received| received.value))[0]
            .as_ref()
            .unwrap()
            .value,
        16
    );
    assert_eq!(leg.remaining(), Duration::from_secs(27));
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn replay_retains_sole_slot_completion_and_stops_expired_callbacks() {
    let server = Server::new();
    let spec = server.spec(quota(1), KeyUse::None);
    let dir = recording(&server, spec, 3);
    let fetch = replay(dir.path());
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(30)));
    let mut budgets: Vec<_> = leg
        .batch(&[(); 2], |child, ()| {
            child.call(
                &spec.get("/number"),
                Input::Params(&[]),
                Read::<Number>::json(),
            )?;
            Ok(child.remaining())
        })
        .into_iter()
        .map(Result::unwrap)
        .collect();
    budgets.sort();
    assert_eq!(
        budgets,
        vec![Duration::from_secs(28), Duration::from_secs(29)]
    );
    assert_eq!(leg.remaining(), Duration::from_secs(28));
    let exhausted = replay(dir.path());
    let leg = exhausted.leg(spec, exhausted.deadline(Duration::from_secs(2)));
    let results = leg.batch(&[(); 2], |child, ()| {
        child
            .call(
                &spec.get("/number"),
                Input::Params(&[]),
                Read::<Number>::json(),
            )
            .map(|received| received.value)
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert!(
        results
            .iter()
            .any(|result| matches!(result, Err(RequestError::Source(error)) if error.reason == biomcp_types::Reason::DeadlineExceeded))
    );
    assert_eq!(leg.remaining(), Duration::ZERO);
    assert!(
        leg.batch(&[()], |_, ()| -> Result<(), RequestError> {
            panic!("expired callback")
        })[0]
            .is_err()
    );
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn batch_stale_fallbacks_retain_original_failures_after_later_success() {
    let server = Server::new();
    let spec = server.spec(quota(2), KeyUse::None);
    let dir = tempfile::tempdir().unwrap();
    let cache = dir.path().join("cache.sqlite");
    let fetch = Fetch::open(FetchConfig {
        cache: Some(cache.clone()),
        ..FetchConfig::default()
    })
    .unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(30)));
    let endpoint = spec.get("/cached").fresh(biomcp_fetch::Freshness::Hours(1));
    thread::scope(|scope| {
        let seed = scope.spawn(|| {
            leg.call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                .map(|received| received.value)
        });
        server.next().ok();
        assert!(seed.join().unwrap().unwrap().answer);
    });
    rusqlite::Connection::open(&cache)
        .unwrap()
        .execute("UPDATE reply SET fresh_until=0", [])
        .unwrap();
    let stale_read = std::sync::Barrier::new(2);
    thread::scope(|scope| {
        let worker = scope.spawn(|| {
            leg.batch(&[false, true], |child, wait| {
                if *wait {
                    stale_read.wait();
                }
                child
                    .call(&endpoint, Input::Params(&[]), Read::<Answer>::json())
                    .map(|received| received.value)
            })
        });
        let first = server.next();
        // The first HTTP request proves its stale read released the cache.
        // Keep both live replies pending while the second callback reads stale.
        stale_read.wait();
        let second = server.next();
        first.respond(400, "", "failed");
        second.respond(401, "", "failed");
        assert!(
            worker
                .join()
                .unwrap()
                .into_iter()
                .all(|result| result.unwrap().answer)
        );
        let later = scope.spawn(|| {
            leg.call(
                &spec.get("/success"),
                Input::Params(&[]),
                Read::<Answer>::json(),
            )
            .map(|received| received.value)
        });
        server.next().ok();
        assert!(later.join().unwrap().unwrap().answer);
    });
    let report = leg.finish();
    assert!(report.stale);
    assert_eq!(report.failures.len(), 2);
    let mut statuses: Vec<_> = report
        .failures
        .iter()
        .map(|error| {
            assert_eq!(error.reason, biomcp_types::Reason::Rejected);
            error.status.unwrap()
        })
        .collect();
    statuses.sort();
    assert_eq!(statuses, vec![400, 401]);
}

#[test]
fn real_batch_requests_share_the_original_absolute_deadline() {
    let server = Server::new();
    let spec = server.spec(quota(8), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_millis(100)));
    // Never release a reply. The absolute budget must terminate every transport
    // and prevent subsequent chunks from gaining a new source budget.
    let results = leg.batch(&[(); 16], |child, ()| {
        child
            .call(
                &spec.get("/held"),
                Input::Params(&[]),
                Read::<Answer>::json(),
            )
            .map(|received| received.value)
    });
    assert_eq!(results.len(), 16);
    assert!(
        results
            .into_iter()
            .all(|result| matches!(result, Err(RequestError::Source(error)) if error.reason == biomcp_types::Reason::DeadlineExceeded))
    );
    assert_eq!(leg.remaining(), Duration::ZERO);
}
