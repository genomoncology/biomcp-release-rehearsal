//! The server owner interrupts a held publisher before joining its worker.
use super::*;

#[test]
fn caller_cancellation_leaves_the_server_owned_refresh_running() {
    let server = support::Server::with_reply_timeout(Duration::from_secs(60));
    let bundle = declared(&server);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let fetch = cached(&path).with_background_refresh();
    sync_reply(&fetch, &bundle, &server, 200, "", TSV).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute("UPDATE bundle SET fresh_until=0", []).unwrap();
    let (call, cancellation) = fetch.cancellable().unwrap();
    let leg = call.leg(bundle.spec, call.deadline(Duration::from_secs(30)));
    assert_eq!(
        leg.rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
            .unwrap()
            .len(),
        1
    );
    assert!(leg.finish().stale);
    let held = server.next();
    second_stale_read(&fetch, &bundle, &held);
    cancellation.cancel();
    drop(call);
    held.respond(200, "", &TSV.replace("B raf kinase", "Updated gene"));

    // Only the original worker can publish: this observer cannot start a
    // replacement download after an incorrectly inherited caller cancellation.
    let offline = cached(&path).offline();
    let leg = offline.leg(bundle.spec, offline.deadline(Duration::from_secs(30)));
    let watchdog = std::time::Instant::now() + Duration::from_secs(30);
    while leg.stored_bundle_state(&bundle).unwrap() != biomcp_types::LocalBundleState::Fresh {
        assert!(
            std::time::Instant::now() < watchdog,
            "caller cancellation stopped the server-owned publisher"
        );
        std::thread::park_timeout(Duration::from_millis(20));
    }
    let published = leg
        .rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
        .unwrap();
    assert_eq!(published[0].name, "Updated gene");
    drop(fetch);
    assert!(server.requests.try_recv().is_err());
}

#[test]
fn last_refresh_owner_stops_a_held_download_before_its_response_is_released() {
    // Keep the upstream open beyond the shutdown watchdog: its own timeout
    // must not accidentally satisfy the cancellation contract.
    let server = support::Server::with_reply_timeout(Duration::from_secs(60));
    let bundle = declared(&server);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cache.sqlite");
    let fetch = cached(&path).with_background_refresh();
    sync_reply(&fetch, &bundle, &server, 200, "", TSV).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute("UPDATE bundle SET fresh_until=0", []).unwrap();
    let (call, cancellation) = fetch.cancellable().unwrap();
    let leg = call.leg(bundle.spec, call.deadline(Duration::from_secs(30)));
    let rows = leg
        .rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(leg.finish().stale);
    let mut held = server.next();
    cancellation.cancel();
    drop(call);
    let part: String = db
        .query_row(
            "SELECT part FROM bundle_sync WHERE name='genes'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let part = std::fs::File::options()
        .read(true)
        .write(true)
        .open(part)
        .unwrap();
    assert!(matches!(
        part.try_lock(),
        Err(std::fs::TryLockError::WouldBlock)
    ));
    // The interrupted worker must finish its ordinary bounded cleanup while
    // this write remains held; simply detaching its thread is not completion.
    db.execute_batch("BEGIN IMMEDIATE").unwrap();

    let (stopped, completion) = std::sync::mpsc::channel();
    let owner = std::thread::spawn(move || {
        drop(fetch);
        stopped.send(()).unwrap();
    });
    let result = completion.recv_timeout(Duration::from_secs(30));
    let finished = part.try_lock();
    let disconnected = wait_for_disconnect(&mut held);
    db.execute_batch("ROLLBACK").unwrap();
    // Release the fixture even on the old implementation, so the failed
    // regression reaps its worker instead of leaving a detached download.
    held.respond(400, "", "cancelled fixture");
    owner.join().unwrap();
    assert!(
        result.is_ok(),
        "owner waited for the held upstream: {result:?}"
    );
    assert!(
        finished.is_ok(),
        "owner left its publisher file locked: {finished:?}"
    );
    assert!(
        disconnected.is_ok(),
        "owner left its socket open: {disconnected:?}"
    );
    assert!(
        server.requests.try_recv().is_err(),
        "shutdown retried the download"
    );
    let offline = cached(&path).offline();
    let leg = offline.leg(bundle.spec, offline.deadline(Duration::from_secs(30)));
    let retained = leg
        .rows(&bundle, Find::Exact(KeyKind::Symbol, "BRAF"))
        .unwrap();
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].name, rows[0].name);
    assert!(
        leg.finish().stale,
        "shutdown published an incomplete refresh"
    );
}

fn second_stale_read(fetch: &Fetch, bundle: &Bundle<Gene>, held: &support::Request<()>) {
    std::thread::scope(|scope| {
        let (replied, reply) = std::sync::mpsc::channel();
        let reader = scope.spawn(move || {
            let leg = fetch.leg(bundle.spec, fetch.deadline(Duration::from_secs(30)));
            let rows = leg.rows(bundle, Find::Exact(KeyKind::Symbol, "BRAF"));
            replied.send((rows, leg.finish())).unwrap();
        });
        let response = reply.recv_timeout(Duration::from_secs(30));
        if response.is_err() {
            // Release the original publisher before reaping a reader that
            // incorrectly joined it, preserving the failed assertion witness.
            let _ = held.reply.send(support::Response {
                bytes: b"HTTP/1.1 400 Stop\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_vec(),
                continuation: None,
            });
        }
        reader.join().unwrap();
        assert!(
            response.is_ok(),
            "second stale reader joined the held publisher"
        );
        let (rows, report) = response.unwrap();
        let rows = rows.unwrap();
        assert_eq!(rows.len(), 1, "second stale read lost the stored row");
        assert_eq!(
            rows.first().unwrap().name,
            "B raf kinase",
            "second stale read changed the held snapshot"
        );
        assert!(report.stale, "second stale read lost its freshness warning");
    });
}

fn wait_for_disconnect(held: &mut support::Request<()>) -> std::io::Result<()> {
    match std::io::Read::read(held, &mut [0]) {
        Ok(0) => Ok(()),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::NotConnected
            ) =>
        {
            Ok(())
        }
        Ok(_) => Err(std::io::Error::other(
            "held request received unexpected bytes",
        )),
        Err(error) => Err(error),
    }
}
