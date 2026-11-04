use super::*;

#[test]
fn legacy_recording_has_no_acquisition_time() {
    let dir = tempfile::tempdir().unwrap();
    let server = Server::new();
    let spec = spec(&server);
    let capture = fetch(Mode::Record {
        dir: dir.path().to_owned(),
    });
    thread::scope(|scope| {
        let worker = scope.spawn(|| get(&capture, spec));
        server.next().ok();
        worker.join().unwrap().unwrap();
    });
    drop(capture);
    let path = file(dir.path());
    let mut recording: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    recording["exchanges"][0]
        .as_object_mut()
        .unwrap()
        .remove("fetched");
    std::fs::write(path, serde_json::to_vec(&recording).unwrap()).unwrap();
    let replay = fetch(Mode::Replay {
        dir: dir.path().to_owned(),
    });
    let leg = replay.leg(spec, replay.deadline(Duration::from_secs(10)));
    let (_, metadata) = leg
        .call(
            &spec.get("/answer"),
            Input::Params(&[]),
            Read::<Answer>::json(),
        )
        .map(|received| (received.value, received.meta))
        .unwrap();
    assert_eq!(
        metadata.fetched, None,
        "legacy recordings cannot establish acquisition time"
    );
}
