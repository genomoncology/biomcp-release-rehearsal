use super::*;

#[derive(Debug, Deserialize)]
struct Keyed {
    symbol: Option<String>,
    #[serde(rename = "name")]
    _name: String,
}
impl Row for Keyed {
    fn keys(&self) -> Keys {
        Keys {
            exact: self
                .symbol
                .iter()
                .map(|s| (KeyKind::Symbol, s.clone().into()))
                .collect(),
            words: Vec::new(),
        }
    }
}

#[test]
fn rejected_rows_preserve_siblings_and_survive_revalidation_and_reuse() {
    for format in [RowFormat::Tsv, RowFormat::Csv { banner_lines: 0 }] {
        let server = support::Server::<()>::new();
        let bundle = Bundle::<Keyed> {
            spec: server.spec(&QUOTA, KeyUse::None),
            name: "keyed",
            path: "/rows",
            format,
            fresh: Freshness::Days(1),
            row: PhantomData,
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache.sqlite");
        let fetch = cached(&path);
        let body =
            "symbol\tname\nGOOD\tvalid\n\tprivate missing identity\nBAD\t\nLAST\tvalid sibling\n";
        let body = if matches!(format, RowFormat::Tsv) {
            body.into()
        } else {
            body.replace('\t', ",")
        };
        let sync = || {
            std::thread::scope(|scope| {
                let call = scope.spawn(|| fetch.sync(&bundle));
                server.next().respond(200, "ETag: \"rows\"\r\n", &body);
                call.join().unwrap().unwrap()
            })
        };
        let info = sync();
        assert_eq!(info.rows, 2);
        assert_eq!(
            info.rejections,
            vec![
                biomcp_fetch::BundleRejection {
                    row: 3,
                    reason: "no usable query key".into()
                },
                biomcp_fetch::BundleRejection {
                    row: 4,
                    reason: "cannot decode row".into()
                },
            ]
        );
        let leg = fetch.leg(bundle.spec, fetch.deadline(Duration::from_secs(2)));
        assert_eq!(
            leg.rows(&bundle, Find::Exact(KeyKind::Symbol, "LAST"))
                .unwrap()
                .len(),
            1
        );
        let (_, reused) = leg
            .stored_rows(&bundle, Find::Exact(KeyKind::Symbol, "GOOD"))
            .unwrap();
        assert_eq!(reused.rejections, info.rejections);
        if matches!(format, RowFormat::Tsv) {
            drop(fetch);
            reopen_and_import(&bundle, &path, &server, &info.rejections);
        } else {
            assert_eq!(sync().rejections, info.rejections);
        }
    }
}

#[test]
fn opening_older_bundle_metadata_keeps_rows_without_rejections() {
    let server = support::Server::new();
    let bundle = declared(&server);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let fetch = cached(&path);
    sync_reply(&fetch, &bundle, &server, 200, "", TSV).unwrap();
    drop(fetch);
    let database = rusqlite::Connection::open(&path).unwrap();
    database
        .execute_batch("DROP TABLE bundle_rejection;
            UPDATE meta SET version=4 WHERE owner='biomcp-fetch';
            INSERT INTO reply(key,source,url,status,content_type,body,fetched_at,fresh_until)
                VALUES(x'01','test','https://example.test/retained',200,'application/octet-stream',x'00ff',12,34);
            CREATE TABLE foreign_data(value TEXT);
            INSERT INTO foreign_data VALUES('retained foreign bytes');
            INSERT INTO meta VALUES('foreign-owner',99);")
        .unwrap();
    drop(database);
    let imported = Fetch::open(FetchConfig {
        cache: Some(directory.path().join("next.sqlite")),
        cache_predecessors: vec![path.clone()],
        ..Default::default()
    })
    .unwrap();
    let (rows, metadata) = imported
        .leg(bundle.spec, imported.deadline(Duration::from_secs(2)))
        .stored_rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(metadata.rejections.is_empty());
    drop(imported);
    let reopened = cached(&path);
    let leg = reopened.leg(bundle.spec, reopened.deadline(Duration::from_secs(2)));
    let (rows, metadata) = leg
        .stored_rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(metadata.rejections.is_empty());
    retained_legacy_state(&path);
}

#[test]
fn punctuation_only_word_keys_are_rejected_without_losing_indexed_siblings() {
    #[derive(Debug, Deserialize)]
    struct Word {
        name: String,
    }
    impl Row for Word {
        fn keys(&self) -> Keys {
            Keys {
                exact: Vec::new(),
                words: vec![self.name.clone().into()],
            }
        }
    }
    for format in [
        RowFormat::Tsv,
        RowFormat::Csv { banner_lines: 0 },
        RowFormat::NativeTsv {
            gzip: false,
            max_decoded_bytes: 1024,
        },
        RowFormat::NativeCsv { max_bytes: 1024 },
    ] {
        let server = support::Server::<()>::new();
        let bundle = Bundle::<Word> {
            spec: server.spec(&QUOTA, KeyUse::None),
            name: "words",
            path: "/words",
            format,
            fresh: Freshness::Days(1),
            row: PhantomData,
        };
        let dir = tempfile::tempdir().unwrap();
        let fetch = cached(&dir.path().join("cache.sqlite"));
        let info = std::thread::scope(|scope| {
            let call = scope.spawn(|| fetch.sync(&bundle));
            server
                .next()
                .respond(200, "", "name\nvalid sibling\n--- !!!\n");
            call.join().unwrap().unwrap()
        });
        assert_eq!(info.rows, 1);
        assert_eq!(
            info.rejections,
            vec![biomcp_fetch::BundleRejection {
                row: 3,
                reason: "no usable query key".into()
            }]
        );
        let leg = fetch.leg(bundle.spec, fetch.deadline(Duration::from_secs(2)));
        let (rows, _) = leg.stored_rows(&bundle, Find::Words("sibling")).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "valid sibling");
    }
}

fn reopen_and_import(
    bundle: &Bundle<Keyed>,
    path: &std::path::Path,
    server: &support::Server,
    expected: &[biomcp_fetch::BundleRejection],
) {
    let reopened = cached(path);
    assert_eq!(
        reopened
            .leg(bundle.spec, reopened.deadline(Duration::from_secs(2)))
            .stored_rows(bundle, Find::Exact(KeyKind::Symbol, "GOOD"))
            .unwrap()
            .1
            .rejections,
        expected,
        "Reopening retains every rejection"
    );
    drop(reopened);
    let reopened = Fetch::open(FetchConfig {
        cache: Some(path.with_file_name("next.sqlite")),
        cache_predecessors: vec![path.to_owned()],
        ..Default::default()
    })
    .unwrap();
    let (_, imported) = reopened
        .leg(bundle.spec, reopened.deadline(Duration::from_secs(2)))
        .stored_rows(bundle, Find::Exact(KeyKind::Symbol, "GOOD"))
        .unwrap();
    assert_eq!(
        imported.rejections, expected,
        "Import retains every rejection"
    );
    let revalidated = std::thread::scope(|scope| {
        let call = scope.spawn(|| reopened.sync(bundle));
        server.next().respond(304, "", "");
        call.join().unwrap().unwrap()
    });
    assert_eq!(
        revalidated.rejections, expected,
        "A 304 retains every rejection"
    );
}

fn retained_legacy_state(path: &std::path::Path) {
    let database = rusqlite::Connection::open(path).unwrap();
    assert_eq!(
        database
            .query_row(
                "SELECT body,fetched_at,fresh_until FROM reply WHERE key=x'01'",
                [],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                }
            )
            .unwrap(),
        (vec![0, 255], 12, 34),
        "Migration retains cached reply bytes and timestamps"
    );
    assert_eq!(
        database
            .query_row("SELECT value FROM foreign_data", [], |row| row
                .get::<_, String>(0))
            .unwrap(),
        "retained foreign bytes",
        "Migration retains foreign tables"
    );
    assert_eq!(
        database
            .query_row(
                "SELECT version FROM meta WHERE owner='foreign-owner'",
                [],
                |row| row.get::<_, u32>(0)
            )
            .unwrap(),
        99,
        "Migration retains foreign owner metadata"
    );
    assert_eq!(
        database
            .query_row(
                "SELECT version FROM meta WHERE owner='biomcp-fetch'",
                [],
                |row| row.get::<_, u32>(0)
            )
            .unwrap(),
        biomcp_fetch::VERSION,
        "Migration updates the transport owner"
    );
}

#[test]
fn complete_part_recovers_from_an_unsatisfiable_resume() {
    let server = support::Server::new();
    let bundle = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let fetch = cached(&path);
    let invalid = "symbol\tsymbol\nBRAF\tkinase\n";
    assert!(matches!(
        sync_reply(&fetch, &bundle, &server, 200, "ETag: \"one\"\r\n", invalid),
        Err(biomcp_fetch::RequestError::Source(error))
            if error.reason == biomcp_types::Reason::BadReply
    ));
    // Recreate a complete part retained by an older version after parsing failed.
    let parts = path.with_extension("parts");
    let legacy = parts.join(".tmpABC416");
    std::fs::write(&legacy, invalid).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    let prior: Option<String> = db
        .query_row("SELECT part FROM bundle_sync", [], |row| row.get(0))
        .ok();
    if let Some(prior) = prior {
        std::fs::remove_file(prior).unwrap();
    }
    db.execute(
        "INSERT OR REPLACE INTO bundle_sync VALUES('genes',NULL,0,?1,?2,?3,?4,?3)",
        rusqlite::params![
            format!("{}/genes", server.base),
            legacy.to_str().unwrap(),
            invalid.len() as i64,
            "\"one\""
        ],
    )
    .unwrap();
    std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync(&bundle));
        let request = server.next();
        let headers = request.text.to_ascii_lowercase();
        assert!(headers.contains(&format!("range: bytes={}-", invalid.len())));
        assert!(headers.contains("if-match: \"one\""));
        request.respond(416, "", "");
        let request = server.next();
        let headers = request.text.to_ascii_lowercase();
        assert!(!headers.contains("range:"));
        assert!(!headers.contains("if-match:"));
        request.respond(200, "ETag: \"one\"\r\n", TSV);
        let info = call.join().unwrap().unwrap();
        assert_eq!(info.rows, 1);
        assert_eq!(info.downloaded_bytes, TSV.len() as u64);
    });
    assert_eq!(std::fs::read_dir(parts).unwrap().count(), 0);
}

#[test]
fn a_completed_body_with_a_bad_header_is_discarded_before_the_next_sync() {
    let server = support::Server::new();
    let bundle = declared(&server);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cache.sqlite");
    let fetch = cached(&path);
    assert!(
        sync_reply(
            &fetch,
            &bundle,
            &server,
            200,
            "ETag: \"one\"\r\n",
            "symbol\tsymbol\nBRAF\tkinase\n",
        )
        .is_err()
    );
    assert_eq!(
        std::fs::read_dir(path.with_extension("parts"))
            .unwrap()
            .count(),
        0
    );
    let db = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        db.query_row("SELECT count(*) FROM bundle_sync", [], |row| {
            row.get::<_, u32>(0)
        })
        .unwrap(),
        0
    );
    std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync(&bundle));
        let request = server.next();
        let headers = request.text.to_ascii_lowercase();
        assert!(!headers.contains("range:"));
        assert!(!headers.contains("if-match:"));
        request.respond(200, "ETag: \"one\"\r\n", TSV);
        assert_eq!(call.join().unwrap().unwrap().rows, 1);
    });
}
