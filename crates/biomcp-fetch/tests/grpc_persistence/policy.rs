//! Cover source policy, credential containment and capture publication through the checked surface.
use super::*;
use biomcp_fetch::{Credential, KeyPlace, Secret, SecretName};
#[test]
fn credentials_stay_out_of_requests_recordings_and_offline_reads_without_a_key() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let credential = Credential {
        tier: biomcp_types::KeyTier::Fallback,
        secret: SecretName::OncokbApiKey,
        place: KeyPlace::Header("x-fixture-key"),
    };
    let source = fixture.with_quota(false, KeyUse::Required(credential), &QUOTA);
    let directory = tempfile::tempdir()?;
    let mut config = FetchConfig {
        mode: Mode::Record {
            dir: directory.path().to_owned(),
        },
        network: fixture.network(None, "")?,
        ..Default::default()
    };
    config
        .secrets
        .insert(credential.secret, Secret::new("private-synthetic-key"));
    let record = Fetch::open(config)?;
    assert!(
        record
            .leg(source, record.deadline(Duration::from_secs(5)))
            .grpc_checked(
                &source.grpc(METHOD).checked(SCHEMA, Freshness::Never),
                b"private-synthetic-key",
                Decoder::new
            )
            .is_err()
    );
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 0);
    get(&record, source, METHOD, SCHEMA, Freshness::Never)?;
    drop(record);
    for entry in std::fs::read_dir(directory.path().join("mygene"))? {
        let bytes = std::fs::read(entry?.path())?;
        assert!(
            !bytes
                .windows(b"private-synthetic-key".len())
                .any(|v| v == b"private-synthetic-key")
        );
    }
    let replay = fetch(
        &fixture,
        Mode::Replay {
            dir: directory.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    assert!(
        matches!(get(&replay,source,METHOD,SCHEMA,Freshness::Never),Err(RequestError::Source(e)) if e.reason==biomcp_types::Reason::NoKey)
    );
    let offline = fetch(&fixture, Mode::Offline, None, Limits::default())?;
    assert!(
        matches!(get(&offline,source,METHOD,SCHEMA,Freshness::Hours(1)),Err(RequestError::Source(e)) if e.reason==biomcp_types::Reason::NoKey)
    );
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 1);
    fixture.finish()
}
#[test]
fn concurrent_captures_keep_each_stream_order_and_terminal_marker() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
    let directory = tempfile::tempdir()?;
    let record = fetch(
        &fixture,
        Mode::Record {
            dir: directory.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    let mut workers = Vec::new();
    for _ in 0..4 {
        let record = record.clone();
        workers.push(std::thread::spawn(move || {
            get(&record, source, METHOD, SCHEMA, Freshness::Never)
        }));
    }
    for worker in workers {
        assert_eq!(worker.join().map_err(|_| "caller panicked")??, [1, 2, 3, 4]);
    }
    drop(record);
    let replay = fetch(
        &fixture,
        Mode::Replay {
            dir: directory.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    for _ in 0..4 {
        assert_eq!(
            get(&replay, source, METHOD, SCHEMA, Freshness::Never)?,
            [1, 2, 3, 4]
        );
    }
    assert!(matches!(
        get(&replay, source, METHOD, SCHEMA, Freshness::Never),
        Err(RequestError::Recording(
            biomcp_fetch::RecordError::Exhausted
        ))
    ));
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 4);
    fixture.finish()
}
#[test]
fn failed_sidecar_publication_keeps_the_old_manifest_and_allows_a_later_capture() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
    let directory = tempfile::tempdir()?;
    let record = fetch(
        &fixture,
        Mode::Record {
            dir: directory.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    get(&record, source, METHOD, SCHEMA, Freshness::Never)?;
    let path = manifest(directory.path())?;
    let before = std::fs::read(&path)?;
    let stem = path
        .file_stem()
        .ok_or("no stem")?
        .to_str()
        .ok_or("bad stem")?;
    let conflict = path.with_file_name(format!("{stem}-grpc-1-1.bin"));
    let partial = path.with_file_name(format!("{stem}-grpc-1-0.bin"));
    std::fs::write(&conflict, b"owned fixture conflict")?;
    assert!(matches!(
        get(&record, source, METHOD, SCHEMA, Freshness::Never),
        Err(RequestError::Recording(_))
    ));
    assert_eq!(std::fs::read(&path)?, before);
    assert!(!partial.exists());
    assert_eq!(std::fs::read(&conflict)?, b"owned fixture conflict");
    std::fs::remove_file(conflict)?;
    assert_eq!(
        get(&record, source, METHOD, SCHEMA, Freshness::Never)?,
        [1, 2, 3, 4]
    );
    drop(record);
    let replay = fetch(
        &fixture,
        Mode::Replay {
            dir: directory.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    for _ in 0..2 {
        assert_eq!(
            get(&replay, source, METHOD, SCHEMA, Freshness::Never)?,
            [1, 2, 3, 4]
        );
    }
    fixture.finish()
}
#[test]
fn live_wire_limit_rejects_the_stream_before_spool_admission() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
    // The two valid protobufs consume 25 framed wire bytes. The worker must
    // reject the second DATA frame rather than delivering it to the spool.
    let live = Fetch::open(FetchConfig {
        max_body: 24,
        network: fixture.network(None, "")?,
        ..Default::default()
    })?;
    let error = get(&live, source, METHOD, SCHEMA, Freshness::Never).unwrap_err();
    assert!(
        matches!(error, RequestError::Source(e) if e.reason == biomcp_types::Reason::BadReply),
        "expected worker wire rejection, got {error:?}"
    );
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 1);
    fixture.finish()
}

fn stream_secrets(key: Option<&str>) -> biomcp_fetch::Secrets {
    let mut secrets = biomcp_fetch::Secrets::default();
    if let Some(key) = key {
        secrets.insert(SecretName::OncokbApiKey, Secret::new(key));
    }
    secrets
}
fn stream_source(fixture: &Fixture, private: bool) -> &'static SourceSpec {
    let credential = Credential {
        tier: if private {
            biomcp_types::KeyTier::NoFallback
        } else {
            biomcp_types::KeyTier::Fallback
        },
        secret: SecretName::OncokbApiKey,
        place: KeyPlace::Header("x-fixture-key"),
    };
    fixture.with_quota(
        false,
        if private {
            KeyUse::Required(credential)
        } else {
            KeyUse::Optional(credential)
        },
        &QUOTA,
    )
}
#[test]
fn hosted_grpc_cache_and_replay_cursors_follow_declared_credential_tiers() -> Check {
    for private in [true, false] {
        let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
        let source = stream_source(&fixture, private);
        let directory = tempfile::tempdir()?;
        let record = fetch(
            &fixture,
            Mode::Record {
                dir: directory.path().into(),
            },
            None,
            Limits::default(),
        )?;
        // An ordinary legacy capture coexists; hosted private callers cannot admit it.
        get(
            &record.with_secrets(stream_secrets(Some("synthetic-stream-a"))),
            source,
            METHOD,
            SCHEMA,
            Freshness::Never,
        )?;
        let keyed = record.with_hosted_secrets(stream_secrets(Some("synthetic-stream-a")));
        for _ in 0..2 {
            get(&keyed, source, METHOD, SCHEMA, Freshness::Never)?;
        }
        let replay = fetch(
            &fixture,
            Mode::Replay {
                dir: directory.path().into(),
            },
            None,
            Limits::default(),
        )?;
        let a = replay.with_hosted_secrets(stream_secrets(Some("synthetic-stream-a")));
        let b = replay.with_hosted_secrets(stream_secrets(Some("synthetic-stream-b")));
        let missing = replay.with_hosted_secrets(stream_secrets(None));
        assert_eq!(
            get(&a, source, METHOD, SCHEMA, Freshness::Hours(1))?,
            [1, 2, 3, 4]
        );
        let cached = get(&b.offline(), source, METHOD, SCHEMA, Freshness::Hours(1));
        let keyless = get(
            &missing.offline(),
            source,
            METHOD,
            SCHEMA,
            Freshness::Hours(1),
        );
        if private {
            assert!(
                matches!(cached, Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::Offline)
            );
            assert!(matches!(
                get(&b, source, METHOD, SCHEMA, Freshness::Never),
                Err(RequestError::Recording(biomcp_fetch::RecordError::Missing))
            ));
            assert!(
                matches!(keyless, Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::NoKey)
            );
            assert_eq!(
                get(&a, source, METHOD, SCHEMA, Freshness::Never)?,
                [1, 2, 3, 4]
            );
        } else {
            assert_eq!(cached?, [1, 2, 3, 4]);
            assert_eq!(keyless?, [1, 2, 3, 4]);
            assert_eq!(
                get(&b, source, METHOD, SCHEMA, Freshness::Never)?,
                [1, 2, 3, 4]
            );
        }
        assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 3);
        fixture.finish()?;
    }
    Ok(())
}
