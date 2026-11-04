use super::*;
use biomcp_fetch::{Input, Read};

#[test]
fn auto_download_limit_keeps_owned_part_and_offline_miss_stays_offline() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let f = Fetch::open(FetchConfig {
        cache: Some(path.clone()),
        auto_sync_max_bytes: 1,
        ..Default::default()
    })
    .unwrap();
    let leg = f.leg(b.spec, f.deadline(Duration::from_secs(2)));
    std::thread::scope(|scope| {
        let call = scope.spawn(|| leg.rows(&b, Find::Words("kinase")));
        server.next().respond(200, "ETag: \"one\"\r\n", TSV);
        assert!(
            matches!(call.join().unwrap(),Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::Syncing)
        );
    });
    assert_eq!(
        std::fs::read_dir(path.with_extension("parts"))
            .unwrap()
            .count(),
        1
    );
    let offline = Fetch::open(FetchConfig {
        cache: Some(path),
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })
    .unwrap();
    assert!(matches!(offline.leg(b.spec,
offline.deadline(Duration::from_secs(1))).rows(&b,
Find::Words("kinase")),
Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::Offline));
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn interrupted_download_resumes_only_the_same_representation() {
    for scenario in [
        "safe", "ignored", "changed", "offset", "total", "missing", "412",
    ] {
        let server = support::Server::new();
        let b = declared(&server);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.sqlite");
        let f = cached(&path);
        let cut = TSV.len() / 2;
        std::thread::scope(|scope| {
            let call = scope.spawn(|| f.sync(&b));
            server.next().raw(&format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nETag: \"one\"\r\nConnection: close\r\n\r\n{}",
                TSV.len(),
                &TSV[..cut]
            ));
            assert!(call.join().unwrap().is_err());
        });
        std::thread::scope(|scope| {
            let call = scope.spawn(|| f.sync(&b));
            let request = server.next();
            let lower = request.text.to_ascii_lowercase();
            assert!(lower.contains(&format!("range: bytes={cut}-")));
            assert!(lower.contains("if-match: \"one\""));
            assert!(!lower.contains("if-range"));
            match scenario {
                "ignored" => request.respond(200, "ETag: \"two\"\r\n", TSV),
                "412" => {
                    request.respond(412, "", "");
                    let next = server.next();
                    assert!(!next.text.to_ascii_lowercase().contains("range:"));
                    next.respond(200, "ETag: \"two\"\r\n", TSV);
                }
                _ => {
                    let etag = if scenario == "changed" { "two" } else { "one" };
                    let offset = if scenario == "offset" { cut + 1 } else { cut };
                    let total = if scenario == "total" {
                        TSV.len() + 1
                    } else {
                        TSV.len()
                    };
                    let range = if scenario == "missing" {
                        String::new()
                    } else {
                        format!(
                            "Content-Range: bytes {offset}-{}/{total}\r\n",
                            TSV.len() - 1
                        )
                    };
                    request.respond(206, &format!("ETag: \"{etag}\"\r\n{range}"), &TSV[cut..]);
                }
            }
            let result = call.join().unwrap();
            if ["safe", "ignored", "412"].contains(&scenario) {
                assert_download(result.unwrap(), scenario, cut);
            } else {
                assert!(result.is_err());
            }
        });
        if ["changed", "offset", "total", "missing"].contains(&scenario) {
            std::thread::scope(|scope| {
                let call = scope.spawn(|| f.sync(&b));
                let request = server.next();
                assert!(
                    !request.text.to_ascii_lowercase().contains("range:"),
                    "unsafe resume metadata must force the next sync to restart"
                );
                request.respond(200, "ETag: \"new\"\r\n", TSV);
                assert_eq!(call.join().unwrap().unwrap().rows, 1);
            });
        }
        assert_eq!(
            std::fs::read_dir(path.with_extension("parts"))
                .unwrap()
                .count(),
            0
        );
    }
}

fn assert_download(info: biomcp_fetch::BundleInfo, scenario: &str, cut: usize) {
    assert_eq!(
        info.rows, 1,
        "Bundle downloads must publish only the complete checked file"
    );
    assert_eq!(
        info.bytes,
        TSV.len() as u64,
        "Bundle downloads must publish only the complete checked file"
    );
    let received = if scenario == "safe" {
        TSV.len() - cut
    } else {
        TSV.len()
    };
    assert_eq!(
        info.downloaded_bytes, received as u64,
        "Bundle downloads must publish only the complete checked file"
    );
}

#[test]
fn oversized_provider_length_does_not_poison_the_next_sync() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let f = cached(&dir.path().join("cache.sqlite"));
    for length in [u64::from(u32::MAX) + 1, u64::MAX] {
        std::thread::scope(|scope| {
            let call = scope.spawn(|| f.sync(&b));
            server.next().raw(&format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {length}\r\nETag: \"huge\"\r\nConnection: close\r\n\r\n"
            ));
            assert!(matches!(
                call.join().unwrap(),
                Err(biomcp_fetch::RequestError::Source(_))
            ));
        });
        assert_eq!(sync_reply(&f, &b, &server, 200, "", TSV).unwrap().rows, 1);
    }
}

#[test]
fn a_stalled_body_keeps_its_partial_bytes_and_resumes_within_a_new_budget() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let f = Fetch::open(FetchConfig {
        cache: Some(path.clone()),
        sync_budget: Duration::from_secs(1),
        ..Default::default()
    })
    .unwrap();
    let cut = TSV.len() / 2;
    std::thread::scope(|scope| {
        let call = scope.spawn(|| f.sync(&b));
        let request = server.next();
        let (release, wait) = std::sync::mpsc::channel();
        request
            .reply
            .send(support::Response {
                bytes: format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nETag: \"one\"\r\nConnection: close\r\n\r\n{}",
                    TSV.len(),
                    &TSV[..cut]
                )
                .into_bytes(),
                continuation: Some(wait),
            })
            .unwrap();
        assert!(
            matches!(call.join().unwrap(),Err(biomcp_fetch::RequestError::Source(e)) if e.reason==biomcp_types::Reason::Syncing)
        );
        drop(release);
    });
    let db = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        db.query_row("SELECT bytes FROM bundle_sync", [], |r| r.get::<_, u32>(0))
            .unwrap() as usize,
        cut
    );
    std::thread::scope(|scope| {
        let call = scope.spawn(|| f.sync(&b));
        let request = server.next();
        assert!(request.text.contains(&format!("bytes={cut}-")));
        request.respond(
            206,
            &format!(
                "ETag: \"one\"\r\nContent-Range: bytes {cut}-{}/{}\r\n",
                TSV.len() - 1,
                TSV.len()
            ),
            &TSV[cut..],
        );
        assert_eq!(call.join().unwrap().unwrap().rows, 1);
    });
    db.execute("UPDATE bundle SET fresh_until=0", []).unwrap();
    let offline = Fetch::open(FetchConfig {
        cache: Some(path),
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })
    .unwrap();
    let leg = offline.leg(b.spec, offline.deadline(Duration::from_secs(1)));
    assert_eq!(leg.rows(&b, Find::Words("kinase")).unwrap().len(), 1);
    assert!(leg.finish().stale);
}

#[test]
fn recovered_parts_reject_unowned_paths_without_touching_them() {
    for scenario in [
        "outside",
        "missing-outside",
        "traversal",
        "nested",
        "wrong-name",
        "missing-wrong-name",
        "missing",
        "directory",
        "symlink",
        "dangling-symlink",
    ] {
        for holder in [None, Some("expired")] {
            recovered_path_case(scenario, holder);
        }
    }
}

fn recovered_path_case(scenario: &str, holder: Option<&str>) {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let f = Fetch::open(FetchConfig {
        cache: Some(path.clone()),
        sync_budget: Duration::from_secs(1),
        ..Default::default()
    })
    .unwrap();
    let parts = path.with_extension("parts");
    std::fs::create_dir_all(parts.join("nested")).unwrap();
    let sentinel = dir.path().join(".tmpABC123");
    std::fs::write(&sentinel, "keep this file").unwrap();
    let Some(prior) = recovered_part_path(scenario, dir.path(), &parts, &sentinel) else {
        return;
    };
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute(
        "INSERT INTO bundle_sync VALUES('genes',?3,0,?1,?2,0,NULL,NULL)",
        rusqlite::params![
            format!("{}/genes", server.base),
            prior.to_str().unwrap(),
            holder
        ],
    )
    .unwrap();
    let snapshot = || {
        db.query_row(
            "SELECT json_array(name,holder,lease_until,url,part,bytes,etag,total) FROM bundle_sync",
            [],
            |row| row.get::<_, String>(0),
        )
        .unwrap()
    };
    let before = snapshot();
    if scenario == "missing" {
        assert_eq!(
            sync_reply(&f, &b, &server, 200, "", TSV).unwrap().rows,
            1,
            "Recovered bundle paths must preserve the owned complete file"
        );
        assert_eq!(
            std::fs::read_dir(&parts).unwrap().count(),
            1,
            "Recovered bundle paths must preserve the owned complete file"
        );
        return;
    }
    let result = f.sync(&b);
    assert!(
        matches!(result, Err(biomcp_fetch::RequestError::Cache(_))),
        "{scenario}: {result:?}"
    );
    assert_eq!(snapshot(), before, "{scenario}: recovery row changed");
    assert!(server.requests.try_recv().is_err(), "{scenario}");
    assert_eq!(
        std::fs::read_to_string(&sentinel).unwrap(),
        "keep this file",
        "Recovered bundle paths must preserve the owned complete file"
    );
    assert_eq!(
        prior.symlink_metadata().is_ok(),
        !scenario.starts_with("missing-"),
        "{scenario}"
    );
}

fn recovered_part_path(
    scenario: &str,
    dir: &std::path::Path,
    parts: &std::path::Path,
    sentinel: &std::path::Path,
) -> Option<std::path::PathBuf> {
    let prior = match scenario {
        "outside" => sentinel.to_path_buf(),
        "missing-outside" => dir.join(".tmpDEF456"),
        "traversal" => parts.join("../.tmpABC123"),
        "nested" => parts.join("nested/.tmpABC123"),
        "wrong-name" | "missing-wrong-name" => parts.join("sentinel"),
        _ => parts.join(".tmpABC123"),
    };
    if matches!(scenario, "symlink" | "dangling-symlink") {
        #[cfg(unix)]
        std::os::unix::fs::symlink(
            if scenario == "dangling-symlink" {
                dir.join("absent")
            } else {
                sentinel.to_path_buf()
            },
            &prior,
        )
        .unwrap();
        #[cfg(not(unix))]
        return None;
    } else if scenario == "directory" {
        std::fs::create_dir(&prior).unwrap();
    } else if ![
        "missing",
        "missing-outside",
        "missing-wrong-name",
        "outside",
        "traversal",
    ]
    .contains(&scenario)
    {
        std::fs::write(&prior, "keep this file").unwrap();
    }
    Some(prior)
}

#[test]
fn bundle_publication_finishes_while_another_owner_holds_the_reply_connection() {
    let directory = tempfile::tempdir().unwrap();
    let server = support::Server::<()>::new();
    let bundle = declared(&server);
    let fetch = Fetch::open(FetchConfig {
        cache: Some(directory.path().join("cache.sqlite")),
        sync_budget: Duration::from_secs(5),
        ..Default::default()
    })
    .unwrap();
    let access = fetch.cache_access().unwrap();
    std::thread::scope(|scope| {
        let sync = scope.spawn(|| fetch.sync(&bundle));
        let request = server.next();
        let answer = access
            .admit(fetch.deadline(Duration::from_secs(10)), || {
                // The request proves claim has finished. Keep the ordinary SQL
                // connection held while a multi-batch publication completes.
                let body = format!(
                    "symbol\tband\tname\n{}",
                    "BRAF\t9p21.1\tB raf kinase\n".repeat(600)
                );
                request.respond(200, "", &body);
                let answer = scope.spawn(|| {
                    let leg = fetch.leg(bundle.spec, fetch.deadline(Duration::from_secs(5)));
                    let endpoint = bundle.spec.get("/answer").fresh(Freshness::Hours(1));
                    assert!(
                        leg.call(
                            &endpoint,
                            Input::Params(&[]),
                            Read::<support::Answer>::json()
                        )
                        .map(|received| received.value)
                        .unwrap()
                        .answer
                    );
                    leg.finish()
                });
                server.next().ok();
                assert_eq!(sync.join().unwrap().unwrap().rows, 600);
                Ok(answer)
            })
            .unwrap();
        assert!(answer.join().unwrap().failures.is_empty());
    });
}

#[test]
fn replay_external_writer_stays_external_while_an_admission_owner_does_no_sql() {
    use biomcp_fetch::{CacheError, Mode, RequestError};
    let directory = tempfile::tempdir().unwrap();
    let records = directory.path().join("records");
    let server = support::Server::<()>::new();
    let bundle = declared(&server);
    let recorder = Fetch::open(FetchConfig {
        cache: Some(directory.path().join("record.sqlite")),
        mode: Mode::Record {
            dir: records.clone(),
        },
        ..Default::default()
    })
    .unwrap();
    sync_reply(&recorder, &bundle, &server, 200, "", TSV).unwrap();
    drop(recorder);
    let fetch = Fetch::open(FetchConfig {
        mode: Mode::Replay { dir: records },
        sync_budget: Duration::from_secs(60),
        ..Default::default()
    })
    .unwrap();
    let access = fetch.cache_access().unwrap();
    let external = access.connect().unwrap();
    external.execute_batch("BEGIN IMMEDIATE").unwrap();
    let (done, result) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        let worker = access
            .admit(fetch.deadline(Duration::from_secs(60)), || {
                let worker = scope.spawn(|| {
                    done.send(fetch.sync(&bundle)).unwrap();
                });
                // This watchdog is shorter than the frozen replay budget. A held
                // no-SQL owner must not turn an external refusal into that wait.
                assert!(matches!(
                    result.recv_timeout(Duration::from_secs(30)).unwrap(),
                    Err(RequestError::Cache(CacheError::Busy))
                ));
                Ok(worker)
            })
            .unwrap();
        external.execute_batch("ROLLBACK").unwrap();
        worker.join().unwrap();
    });
}

#[test]
fn revalidated_sync_reports_no_download_and_preserves_source_bytes() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let f = cached(&dir.path().join("cache.sqlite"));
    let imported = sync_reply(&f, &b, &server, 200, "ETag: \"one\"\r\n", TSV).unwrap();
    assert_eq!(imported.bytes, TSV.len() as u64);
    assert_eq!(imported.downloaded_bytes, TSV.len() as u64);
    let unchanged = sync_reply(&f, &b, &server, 304, "", "").unwrap();
    assert_eq!(unchanged.bytes, TSV.len() as u64);
    assert_eq!(unchanged.downloaded_bytes, 0);
}

#[test]
fn fresh_bundle_refresh_is_shared_by_one_leg_and_its_batch_children() {
    let server = support::Server::new();
    let bundle = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let fetch = cached(&dir.path().join("cache.sqlite"));
    sync_reply(&fetch, &bundle, &server, 200, "ETag: \"one\"\r\n", TSV).unwrap();
    let leg = fetch
        .leg(bundle.spec, fetch.deadline(Duration::from_secs(10)))
        .fresh(true);
    std::thread::scope(|scope| {
        let call = scope.spawn(|| leg.rows(&bundle, Find::Words("kinase")));
        server.next().respond(304, "", "");
        assert_eq!(call.join().unwrap().unwrap().len(), 1);
    });
    let first_report = leg.take_report();
    assert!(!first_report.stale);
    // Reading another gene after draining the report must reuse the refreshed file.
    // No server reply is supplied: another request is an observable failure.
    let values = leg.batch(&["BRAF", "MLH1"], |child, symbol| {
        child.rows(&bundle, Find::Exact(KeyKind::Symbol, symbol))
    });
    assert_eq!(
        values
            .into_iter()
            .map(|value| value.unwrap().len())
            .collect::<Vec<_>>(),
        [1, 0]
    );
    assert!(server.requests.try_recv().is_err());
    let sibling = fetch
        .leg(bundle.spec, fetch.deadline(Duration::from_secs(10)))
        .fresh(true);
    std::thread::scope(|scope| {
        let call = scope.spawn(|| sibling.rows(&bundle, Find::Words("kinase")));
        server.next().respond(304, "", "");
        assert_eq!(call.join().unwrap().unwrap().len(), 1);
    });
}

#[test]
fn killed_download_is_reclaimed_after_its_lease_expires() {
    let server = support::Server::<()>::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let _ = cached(&path);
    let mut old = Child::start(&path, server.base, "ok");
    let request = server.next();
    let db = rusqlite::Connection::open(&path).unwrap();
    let old_path: String = db
        .query_row("SELECT part FROM bundle_sync", [], |r| r.get(0))
        .unwrap();
    let (release, wait) = std::sync::mpsc::channel();
    request.reply.send(support::Response {
        bytes: format!(
            "HTTP/1.1 200 OK\r\nContent-Length: 1000000\r\nETag: \"one\"\r\nConnection: close\r\n\r\n{}",
            "x".repeat(32768)
        ).into_bytes(),
        continuation: Some(wait),
    }).unwrap();
    while std::fs::metadata(&old_path).unwrap().len() == 0 {
        assert!(
            old.process.try_wait().unwrap().is_none(),
            "child exited before writing its body"
        );
        std::thread::yield_now();
    }
    old.process.kill().unwrap();
    assert!(!old.process.wait().unwrap().success());
    drop(release);
    assert!(std::path::Path::new(&old_path).exists());
    db.execute("UPDATE bundle_sync SET lease_until=0", [])
        .unwrap();
    let mut successor = Child::start(&path, server.base, "ok");
    let request = server.next();
    assert!(!request.text.to_ascii_lowercase().contains("range:"));
    request.respond(200, "", TSV);
    successor.finish();
    assert!(!std::path::Path::new(&old_path).exists());
    assert_eq!(
        std::fs::read_dir(path.with_extension("parts"))
            .unwrap()
            .count(),
        0
    );
}
