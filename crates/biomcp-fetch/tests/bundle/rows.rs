use super::*;
use biomcp_fetch::{Input, Read};

#[test]
fn indexed_tsv_rows_are_literal_and_offline() {
    let server = support::Server::<()>::new();
    let spec = server.spec(&QUOTA, KeyUse::None);
    let bundle = Bundle::<Gene> {
        spec,
        name: "genes",
        path: "/genes",
        format: RowFormat::Tsv,
        fresh: Freshness::Days(1),
        row: PhantomData,
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let fetch = Fetch::open(FetchConfig {
        cache: Some(path.clone()),
        ..Default::default()
    })
    .unwrap();
    std::thread::scope(|scope| {
        let run = scope.spawn(|| fetch.sync(&bundle).unwrap());
        server.next().respond(
            200,
            "ETag: \"one\"\r\n",
            "symbol\tband\tname\nBRAF\t9p21.1\t\"B raf kinase\"\nOTHER\t19p21\tOther\n",
        );
        let info = run.join().unwrap();
        assert_eq!(info.rows, 2);
        assert!(info.rejections.is_empty());
    });
    // Reply and bundle entries use independent tables even for the same URL.
    std::thread::scope(|scope| {
        let call = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(2)))
                .call(
                    &spec.get("/genes").fresh(Freshness::Days(1)),
                    Input::Params(&[]),
                    Read::<support::Answer>::json(),
                )
                .map(|received| received.value)
        });
        server.next().ok();
        assert!(call.join().unwrap().unwrap().answer);
    });
    let leg = fetch.leg(spec, fetch.deadline(Duration::from_secs(2)));
    assert_eq!(
        leg.rows(&bundle, Find::Exact(KeyKind::Symbol, "braf"))
            .unwrap()[0]
            .symbol,
        "BRAF"
    );
    assert_eq!(
        leg.rows(&bundle, Find::Prefix(KeyKind::Band, "9p21."))
            .unwrap()
            .len(),
        1
    );
    assert!(
        leg.rows(&bundle, Find::Prefix(KeyKind::Band, "%"))
            .unwrap()
            .is_empty()
    );
    for term in ["\"", "*", "AND", "NEAR("] {
        let _ = leg.rows(&bundle, Find::Words(term)).unwrap();
    }
    assert_eq!(leg.rows(&bundle, Find::Words("kinase")).unwrap().len(), 1);
    let offline = Fetch::open(FetchConfig {
        cache: Some(path),
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        offline
            .leg(spec, offline.deadline(Duration::from_secs(2)))
            .rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn revalidation_swaps_all_indexes_and_bad_rows_keep_old_rows() {
    let server = support::Server::new();
    let b = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let f = cached(&path);
    sync_reply(&f, &b, &server, 200, "ETag: \"one\"\r\n", TSV).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute("UPDATE bundle SET fresh_until=0", []).unwrap();
    std::thread::scope(|scope| {
        let call = scope.spawn(|| f.sync(&b));
        let request = server.next();
        assert!(
            request
                .text
                .to_ascii_lowercase()
                .contains("if-none-match: \"one\"")
        );
        request.respond(304, "", "");
        assert!(call.join().unwrap().unwrap().fresh);
    });
    assert!(
        db.query_row(
            "SELECT sql FROM sqlite_master WHERE name='bundle_words'",
            [],
            |r| r.get::<_, String>(0)
        )
        .unwrap()
        .contains("fts5")
    );
    for bad in [
        "symbol\tsymbol\nA\tB\n",
        "\tband\nA\tB\n",
        "symbol\tband\tname\nBAD\tx\t\"broken\n",
        "symbol\tband\tname\nBAD\tx\tin\"ner\n",
        "symbol\tband\tname\nBAD\t\"broken\nLAST\t9p\tvalid\n",
        "symbol\tband\tname\nBAD\tx\t\"closed\"suffix\textra\nLAST\t9p\tvalid\n",
    ] {
        assert!(sync_reply(&f, &b, &server, 200, "ETag: \"bad\"\r\n", bad).is_err());
        assert_eq!(
            db.query_row(
                "SELECT count(*) FROM bundle_key WHERE key='braf'",
                [],
                |r| r.get::<_, u32>(0)
            )
            .unwrap(),
            1
        );
    }
    sync_reply(
        &f,
        &b,
        &server,
        200,
        "ETag: \"two\"\r\n",
        "symbol\tband\tname\nNEW\t1q\tNew words\n",
    )
    .unwrap();
    let leg = f.leg(b.spec, f.deadline(Duration::from_secs(2)));
    assert!(
        leg.rows(&b, Find::Exact(KeyKind::Symbol, "BRAF"))
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        leg.rows(&b, Find::Words("new words")).unwrap()[0].symbol,
        "NEW"
    );
}
#[test]
fn banner_csv_records_are_checked_before_atomic_publication() {
    let server = support::Server::new();
    let mut bundle = declared(&server);
    bundle.format = RowFormat::Csv { banner_lines: 2 };
    let dir = tempfile::tempdir().unwrap();
    let fetch = cached(&dir.path().join("cache.sqlite"));
    let good = "banner\r\ncreated today\r\n\"symbol\",\"band\",\"name\"\r\n+++,+++,+++\r\nBRAF,9p,\"B, raf \"\"kinase\"\"\nwhole\"\r\n";
    assert_eq!(
        sync_reply(&fetch, &bundle, &server, 200, "", good)
            .unwrap()
            .rows,
        1
    );
    std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync(&bundle));
        let request = server.next();
        assert!(request.text.contains("text/csv"));
        assert!(!request.text.to_ascii_lowercase().contains("if-"));
        request.respond(
            200,
            "ETag: \"generated\"\r\nLast-Modified: Wed, 21 Oct 2015 07:28:00 GMT\r\n",
            good,
        );
        call.join().unwrap().unwrap();
    });
    std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync(&bundle));
        let request = server.next();
        let headers = request.text.to_ascii_lowercase();
        assert!(headers.contains("if-none-match: \"generated\""));
        request.respond(200, "", good);
        call.join().unwrap().unwrap();
    });
    for bad in [
        "banner\ncreated\nsymbol,band,name\nBRAF,9p,\"open",
        "banner\ncreated\nsymbol,band,name\nBRAF,9p,bad\"quote\n",
        "banner\ncreated\nsymbol,band,name\nBRAF,9p,\"closed\"x\n",
        "banner\ncreated\nsymbol,,name\nBRAF,9p,a\n",
        "banner\ncreated\nsymbol,symbol,name\nBRAF,9p,a\n",
    ] {
        assert!(sync_reply(&fetch, &bundle, &server, 200, "", bad).is_err());
        let leg = fetch.leg(bundle.spec, fetch.deadline(Duration::from_secs(2)));
        let rows = leg
            .rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "B, raf \"kinase\"\nwhole");
    }
    let capture = dir.path().join("capture");
    let record = Fetch::open(FetchConfig {
        cache: Some(dir.path().join("record.sqlite")),
        mode: biomcp_fetch::Mode::Record {
            dir: capture.clone(),
        },
        ..Default::default()
    })
    .unwrap();
    sync_reply(&record, &bundle, &server, 200, "", good).unwrap();
    let replay = Fetch::open(FetchConfig {
        mode: biomcp_fetch::Mode::Replay { dir: capture },
        ..Default::default()
    })
    .unwrap();
    assert_eq!(replay.sync(&bundle).unwrap().rows, 1);
}
#[derive(Debug, Deserialize)]
struct Medicine {
    id: String,
    name: String,
}
impl Row for Medicine {
    fn keys(&self) -> Keys {
        Keys {
            exact: vec![(KeyKind::Id, self.id.clone().into())],
            words: vec![self.name.clone().into()],
        }
    }
}
fn bundle(server: &support::Server) -> Bundle<Medicine> {
    Bundle {
        spec: server.spec(&QUOTA, KeyUse::None),
        name: "medicines",
        path: "/medicines",
        format: RowFormat::Json {
            rows: "data",
            metadata: "meta",
        },
        fresh: Freshness::Days(1),
        row: PhantomData,
    }
}
#[test]
fn json_bundle_reads_never_sync_and_failed_replacements_keep_old_rows() {
    let server = support::Server::new();
    let b = bundle(&server);
    let dir = tempfile::tempdir().unwrap();
    let f = cached(&dir.path().join("cache.sqlite"));
    let leg = f
        .leg(b.spec, f.deadline(Duration::from_secs(5)))
        .fresh(true);
    assert!(leg.stored_rows(&b, Find::Words("Tibsovo")).is_err());
    let body =
        r#"{"meta":{"timestamp":"2026-10-24T18:22:32Z"},"data":[{"id":"old","name":"Tibsovo"}]}"#;
    std::thread::scope(|s| {
        let call = s.spawn(|| f.sync(&b));
        server.next().respond(
            200,
            "ETag: \"catalog-one\"\r\nLast-Modified: Mon, 26 Oct 2026 00:00:00 GMT\r\n",
            body,
        );
        let info = call.join().unwrap().unwrap();
        assert_eq!(info.bytes, body.len() as u64);
        assert_eq!(info.etag.as_deref(), Some("\"catalog-one\""));
        assert_eq!(
            info.last_modified.as_deref(),
            Some("Mon, 26 Oct 2026 00:00:00 GMT")
        );
        assert!(info.retrieved_at.is_some());
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(body.as_bytes())
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(
            info.content_revision.as_deref(),
            Some(format!("sha256:{digest}").as_str())
        );
    });
    for invalid in [
        "{",
        r#"{"meta":{},"data":[{"id":"bad"}]}"#,
        r#"{"meta":{},"data":[]}"#,
    ] {
        std::thread::scope(|s| {
            let call = s.spawn(|| f.sync(&b));
            server.next().respond(200, "", invalid);
            assert!(call.join().unwrap().is_err());
        });
        let (rows, info) = leg.stored_rows(&b, Find::Words("Tibsovo")).unwrap();
        assert_eq!(rows[0].id, "old");
        assert_eq!(info.rows, 1);
        assert_eq!(info.etag.as_deref(), Some("\"catalog-one\""));
        assert!(info.retrieved_at.is_some());
    }
    assert!(server.requests.try_recv().is_err());
}
#[test]
fn two_independent_readers_observe_one_complete_json_replacement() {
    let server = support::Server::new();
    let b = bundle(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let writer = cached(&path);
    let old =
        r#"{"meta":{},"data":[{"id":"old1","name":"Tibsovo"},{"id":"old2","name":"Tibsovo"}]}"#;
    let new = r#"{"meta":{},"data":[{"id":"new1","name":"Tibsovo"},{"id":"new2","name":"Tibsovo"},{"id":"new3","name":"Tibsovo"}]}"#;
    std::thread::scope(|s| {
        let call = s.spawn(|| writer.sync(&b));
        server.next().respond(200, "", old);
        call.join().unwrap().unwrap();
    });
    let readers = [cached(&path), cached(&path)];
    let barrier = std::sync::Barrier::new(3);
    std::thread::scope(|s| {
        let threads: Vec<_> = readers
            .iter()
            .map(|reader| {
                s.spawn(|| {
                    let leg = reader.leg(b.spec, reader.deadline(Duration::from_secs(10)));
                    assert_eq!(
                        leg.stored_rows(&b, Find::Words("Tibsovo")).unwrap().0.len(),
                        2
                    );
                    barrier.wait();
                    loop {
                        let (rows, info) = leg.stored_rows(&b, Find::Words("Tibsovo")).unwrap();
                        let ids: Vec<_> = rows.iter().map(|row| row.id.as_str()).collect();
                        assert_eq!(info.rows, rows.len() as u64);
                        if ids == ["new1", "new2", "new3"] {
                            break;
                        }
                        assert_eq!(ids, ["old1", "old2"]);
                    }
                })
            })
            .collect();
        let update = s.spawn(|| writer.sync(&b));
        let request = server.next();
        barrier.wait();
        request.respond(200, "", new);
        update.join().unwrap().unwrap();
        for reader in threads {
            reader.join().unwrap();
        }
    });
    assert!(server.requests.try_recv().is_err());
}
#[test]
fn truncated_json_download_retains_the_complete_prior_bundle() {
    let server = support::Server::new();
    let b = bundle(&server);
    let dir = tempfile::tempdir().unwrap();
    let writer = cached(&dir.path().join("cache.sqlite"));
    std::thread::scope(|s| {
        let call = s.spawn(|| writer.sync(&b));
        server.next().respond(
            200,
            "",
            r#"{"meta":{},"data":[{"id":"old","name":"Tibsovo"}]}"#,
        );
        call.join().unwrap().unwrap();
    });
    std::thread::scope(|s| {
        let call = s.spawn(|| writer.sync(&b));
        server.next().raw(
            "HTTP/1.1 200 Stub\r\nContent-Length: 200\r\nConnection: close\r\n\r\n{\"meta\":{}",
        );
        assert!(call.join().unwrap().is_err());
    });
    let leg = writer.leg(b.spec, writer.deadline(Duration::from_secs(5)));
    assert_eq!(
        leg.stored_rows(&b, Find::Words("Tibsovo")).unwrap().0[0].id,
        "old"
    );
}

// Complete-set atomicity remains at the existing public bundle transport boundary.
mod sets {
    use super::super::support;
    use biomcp_fetch::{
        BundlePart, BundleSet, Fetch, FetchConfig, Find, Freshness, KeyKind, KeyUse, Keys, Quota,
        Rate, Row, RowFormat, SourceKey,
    };
    use serde::Deserialize;
    use std::{marker::PhantomData, time::Duration};
    #[derive(Debug, Deserialize)]
    struct Native {
        name: String,
    }
    impl Row for Native {
        fn keys(&self) -> Keys {
            Keys {
                exact: vec![(KeyKind::Id, self.name.clone().into())],
                words: vec![self.name.clone().into()],
            }
        }
    }
    static QUOTA: Quota = Quota::new(
        SourceKey::declare("bundle-set-contract"),
        Rate::per_second(1000),
        None,
        4,
    );
    static PARTS: &[BundlePart] = &[
        BundlePart {
            format: None,
            name: "A",
            path: "/A",
        },
        BundlePart {
            format: None,
            name: "B",
            path: "/B",
        },
        BundlePart {
            format: None,
            name: "D",
            path: "/D",
        },
        BundlePart {
            format: None,
            name: "H",
            path: "/H",
        },
        BundlePart {
            format: None,
            name: "L",
            path: "/L",
        },
        BundlePart {
            format: None,
            name: "P",
            path: "/P",
        },
        BundlePart {
            format: None,
            name: "R",
            path: "/R",
        },
        BundlePart {
            format: None,
            name: "V",
            path: "/V",
        },
    ];
    fn declared(server: &support::Server) -> BundleSet<Native> {
        BundleSet {
            spec: server.spec(&QUOTA, KeyUse::None),
            name: "synthetic-complete-set",
            parts: PARTS,
            format: RowFormat::Csv { banner_lines: 0 },
            fresh: Freshness::Days(7),
            row: PhantomData,
        }
    }
    fn cached(path: &std::path::Path) -> Fetch {
        Fetch::open(FetchConfig {
            cache: Some(path.to_owned()),
            ..Default::default()
        })
        .unwrap()
    }
    fn names(fetch: &Fetch, set: &BundleSet<Native>, word: &str) -> Vec<String> {
        let (rows, info) = fetch
            .leg(set.spec, fetch.deadline(Duration::from_secs(3)))
            .stored_set_rows(set, Find::Words(word))
            .unwrap();
        assert_eq!(
            info.rows, 8,
            "Bundle rows must retain source names in their published order"
        );
        assert!(
            info.content_revision
                .as_deref()
                .is_some_and(|s| s.starts_with("sha256:")),
            "Bundle rows must retain source names in their published order"
        );
        rows.into_iter().map(|r| r.value.name).collect()
    }
    #[test]
    fn complete_set_readers_see_one_revision_and_failed_parts_preserve_it() {
        let server = support::Server::new();
        let set = declared(&server);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.sqlite");
        let writer = cached(&path);
        let reader = cached(&path);
        let second_reader = cached(&path);
        std::thread::scope(|scope| {
            let call = scope.spawn(|| writer.sync_set(&set));
            for part in PARTS {
                server
                    .next()
                    .respond(200, "", &format!("name\nOLD {}\n", part.name));
            }
            assert_eq!(call.join().unwrap().unwrap().rows, 8);
        });
        assert_eq!(names(&reader, &set, "OLD").len(), 8);
        std::thread::scope(|scope| {
            let call = scope.spawn(|| writer.sync_set(&set));
            for (i, part) in PARTS.iter().enumerate() {
                let request = server.next();
                assert!(request.text.starts_with(&format!("GET {} ", part.path)));
                if i == 7 {
                    assert_eq!(names(&reader, &set, "OLD").len(), 8);
                    assert!(names(&reader, &set, "NEW").is_empty());
                    assert_eq!(names(&second_reader, &set, "OLD").len(), 8);
                    assert!(names(&second_reader, &set, "NEW").is_empty());
                }
                request.respond(200, "", &format!("name\nNEW {}\n", part.name));
            }
            call.join().unwrap().unwrap();
        });
        assert!(names(&reader, &set, "OLD").is_empty());
        assert_eq!(names(&reader, &set, "NEW").len(), 8);
        assert_eq!(names(&second_reader, &set, "NEW").len(), 8);
        for malformed in [false, true] {
            std::thread::scope(|scope| {
                let call = scope.spawn(|| writer.sync_set(&set));
                for (i, part) in PARTS.iter().enumerate() {
                    let request = server.next();
                    if i == 3 {
                        if malformed {
                            request.respond(200, "", "wrong\nmissing-name\n");
                        } else {
                            request.respond(404, "", "missing partition");
                        }
                        break;
                    }
                    request.respond(200, "", &format!("name\nBROKEN {}\n", part.name));
                }
                assert!(call.join().unwrap().is_err());
            });
            assert_eq!(names(&reader, &set, "NEW").len(), 8);
            assert_eq!(names(&second_reader, &set, "NEW").len(), 8);
            assert!(names(&reader, &set, "BROKEN").is_empty());
        }
        // Every read above is stored-only. No unresponded read request exists.
        assert!(server.requests.try_recv().is_err());
    }

    #[test]
    fn older_positional_writers_invalidate_computed_revision_on_replace_update_and_delete() {
        let server = support::Server::new();
        let set = declared(&server);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.sqlite");
        let fetch = cached(&path);
        for sql in [
            "INSERT OR REPLACE INTO bundle VALUES('synthetic-complete-set',1,8,0,NULL,NULL)",
            "UPDATE bundle SET bytes=2 WHERE name='synthetic-complete-set'",
            "DELETE FROM bundle WHERE name='synthetic-complete-set'",
        ] {
            std::thread::scope(|scope| {
                let call = scope.spawn(|| fetch.sync_set(&set));
                for part in PARTS {
                    server
                        .next()
                        .respond(200, "", &format!("name\nOLD {}\n", part.name));
                }
                call.join().unwrap().unwrap();
            });
            let db = fetch.cache_access().unwrap().connect().unwrap();
            let recursive: bool = db
                .pragma_query_value(None, "recursive_triggers", |r| r.get(0))
                .unwrap();
            assert!(
                !recursive,
                "proof must use ordinary SQLite trigger settings"
            );
            assert_eq!(
                db.query_row("SELECT count(*) FROM bundle_content", [], |r| r
                    .get::<_, u32>(0))
                    .unwrap(),
                1
            );
            db.execute(sql, []).unwrap();
            assert_eq!(
                db.query_row("SELECT count(*) FROM bundle_content", [], |r| r
                    .get::<_, u32>(0))
                    .unwrap(),
                0
            );
            if !sql.starts_with("DELETE") {
                let (_, info) = fetch
                    .leg(set.spec, fetch.deadline(Duration::from_secs(3)))
                    .stored_set_rows(&set, Find::Words("OLD"))
                    .unwrap();
                assert!(info.content_revision.is_none());
            }
        }
    }
}
