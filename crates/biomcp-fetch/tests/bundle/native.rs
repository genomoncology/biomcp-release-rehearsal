//! Complete native TSV replacement sets through the public transport boundary.
use super::*;
use biomcp_fetch::{BundlePart, BundleSet};
use std::io::Write;
const BODY: &str = "symbol\tband\tname\r\nBRAF\t\t\"A \"\"quoted\"\" name\"\r\n";
fn gzip(body: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(body).unwrap();
    encoder.finish().unwrap()
}
fn native(server: &support::Server, limit: u64) -> BundleSet<Gene> {
    BundleSet {
        spec: server.spec(&QUOTA, KeyUse::None),
        name: "native-tsv",
        parts: &[
            BundlePart {
                name: "compressed",
                path: "/one.gz",
                format: Some(RowFormat::NativeTsv {
                    gzip: true,
                    max_decoded_bytes: 1024,
                }),
            },
            BundlePart {
                name: "plain",
                path: "/two",
                format: None,
            },
        ],
        format: RowFormat::NativeTsv {
            gzip: false,
            max_decoded_bytes: limit,
        },
        fresh: Freshness::Days(1),
        row: PhantomData,
    }
}
fn reply(server: &support::Server, body: &[u8]) {
    let mut bytes = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .into_bytes();
    bytes.extend_from_slice(body);
    server.next().bytes(bytes);
}
#[test]
fn malformed_native_quotes_do_not_replace_the_prior_pair() {
    let server = support::Server::new();
    let set = native(&server, 1024);
    let dir = tempfile::tempdir().unwrap();
    let fetch = cached(&dir.path().join("cache.sqlite"));
    let bytes = gzip(BODY.as_bytes());
    std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync_set(&set));
        reply(&server, &bytes);
        reply(&server, BODY.as_bytes());
        call.join().unwrap().unwrap();
    });
    for malformed in [
        "symbol\tband\tname\nBRAF\t\t\"unterminated",
        "symbol\tband\tname\nBRAF\t\t\"closed\"suffix\n",
    ] {
        std::thread::scope(|scope| {
            let call = scope.spawn(|| fetch.sync_set(&set));
            reply(&server, &bytes);
            reply(&server, malformed.as_bytes());
            assert!(call.join().unwrap().is_err(), "accepted {malformed:?}");
        });
        let leg = fetch.leg(set.spec, fetch.deadline(Duration::from_secs(2)));
        let (rows, _) = leg
            .stored_set_rows(&set, Find::Exact(KeyKind::Symbol, "BRAF"))
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|r| r.value.name == "A \"quoted\" name"));
    }
    for name in ["bare \"quote\"", ""] {
        std::thread::scope(|scope| {
            let call = scope.spawn(|| fetch.sync_set(&set));
            reply(&server, &bytes);
            let body = format!("symbol\tband\tname\r\nBRAF\t\t{name}\r\n");
            reply(&server, body.as_bytes());
            call.join().unwrap().unwrap();
        });
        let leg = fetch.leg(set.spec, fetch.deadline(Duration::from_secs(2)));
        let (rows, _) = leg
            .stored_set_rows(&set, Find::Exact(KeyKind::Symbol, "BRAF"))
            .unwrap();
        assert_eq!(rows.last().unwrap().value.name, name);
    }
}
#[test]
fn heterogeneous_native_parts_preserve_empty_quoted_cells_and_reject_decoded_overflow() {
    let server = support::Server::new();
    let set = native(&server, 1024);
    let dir = tempfile::tempdir().unwrap();
    let fetch = cached(&dir.path().join("cache.sqlite"));
    let bytes = gzip(BODY.as_bytes());
    std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync_set(&set));
        reply(&server, &bytes);
        reply(&server, BODY.as_bytes());
        assert_eq!(call.join().unwrap().unwrap().rows, 2);
    });
    let read = || {
        let leg = fetch.leg(set.spec, fetch.deadline(Duration::from_secs(2)));
        leg.stored_set_rows(&set, Find::Exact(KeyKind::Symbol, "BRAF"))
            .unwrap()
    };
    let (rows, old) = read();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows.first().unwrap().value.name, "A \"quoted\" name");
    assert_eq!(rows.first().unwrap().value.band, "");
    for truncated in [false, true] {
        let limited = native(&server, 16);
        std::thread::scope(|scope| {
            let call = scope.spawn(|| fetch.sync_set(&limited));
            reply(
                &server,
                if truncated {
                    &bytes[..bytes.len() - 4]
                } else {
                    &bytes
                },
            );
            if !truncated {
                reply(&server, BODY.as_bytes());
            }
            assert!(call.join().unwrap().is_err());
        });
        let (rows, info) = read();
        assert_eq!(rows.len(), 2);
        assert_eq!(info.content_revision, old.content_revision);
    }
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn rejected_native_rows_keep_part_locations_and_valid_siblings() {
    let server = support::Server::new();
    let set = native(&server, 1024);
    let dir = tempfile::tempdir().unwrap();
    let fetch = cached(&dir.path().join("cache.sqlite"));
    let body = format!("{BODY}\t\t--- !!!\n{}", BODY.split_once('\n').unwrap().1);
    let info = std::thread::scope(|scope| {
        let call = scope.spawn(|| fetch.sync_set(&set));
        reply(&server, &gzip(body.as_bytes()));
        reply(&server, body.as_bytes());
        call.join().unwrap().unwrap()
    });
    assert_eq!(info.rows, 4);
    assert_eq!(
        info.rejections,
        ["compressed", "plain"].map(|part| {
            biomcp_fetch::BundleRejection {
                row: 3,
                reason: format!("{part}: no usable query key"),
            }
        })
    );
    let leg = fetch.leg(set.spec, fetch.deadline(Duration::from_secs(2)));
    let (rows, persisted) = leg
        .stored_set_rows(&set, Find::Exact(KeyKind::Symbol, "BRAF"))
        .unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| (row.part.as_ref(), row.row))
            .collect::<Vec<_>>(),
        [
            ("compressed", 2),
            ("compressed", 4),
            ("plain", 2),
            ("plain", 4)
        ]
    );
    assert_eq!(persisted.rejections, info.rejections);
}
