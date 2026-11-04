//! Feature stream-persistence: checked synthetic streams persist without network replay.
use biomcp_fetch::grpc::{Limits, StreamDecoder};
use biomcp_fetch::{
    BadReply, Fetch, FetchConfig, Freshness, KeyUse, Mode, Quota, Rate, RequestError, SourceKey,
    SourceSpec,
};
use biomcp_test_support::grpc as support;
use std::time::Duration;
use support::{Check, Fixture, Hold};
static QUOTA: Quota = Quota::new(
    SourceKey::declare("grpc-persistence"),
    Rate::per_second(1000),
    None,
    4,
);
const METHOD: &str = "/synthetic.Tensor/Exchange";
const SCHEMA: &str = "synthetic-tensor-v1";
struct Decoder {
    next: usize,
    limits: Limits,
}
impl Decoder {
    fn new(limits: Limits) -> Result<Self, BadReply> {
        limits.check().map_err(|_| BadReply)?;
        Ok(Self { next: 0, limits })
    }
}
impl StreamDecoder for Decoder {
    type Reply = Vec<u8>;
    fn message(&mut self, bytes: &[u8]) -> Result<(), BadReply> {
        if bytes.len() > self.limits.message_bytes
            || messages().get(self.next).map(Vec::as_slice) != Some(bytes)
        {
            return Err(BadReply);
        }
        self.next += 1;
        Ok(())
    }
    fn finish(self) -> Result<Vec<u8>, BadReply> {
        if self.next != messages().len() || self.limits.expanded_bytes < 4 {
            return Err(BadReply);
        }
        Ok(vec![1, 2, 3, 4])
    }
}
fn messages() -> Vec<Vec<u8>> {
    vec![vec![10, 1, 1, 16, 2, 32, 1], vec![10, 4, 1, 2, 3, 4, 16, 0]]
}
fn fetch(
    fixture: &Fixture,
    mode: Mode,
    cache: Option<std::path::PathBuf>,
    limits: Limits,
) -> Check<Fetch> {
    Ok(Fetch::open(FetchConfig {
        mode,
        cache,
        grpc_limits: limits,
        network: fixture.network(None, "")?,
        ..Default::default()
    })?)
}
fn get(
    fetch: &Fetch,
    source: &'static SourceSpec,
    method: &'static str,
    schema: &'static str,
    fresh: Freshness,
) -> Result<Vec<u8>, RequestError> {
    fetch
        .leg(source, fetch.deadline(Duration::from_secs(5)))
        .grpc_checked(
            &source.grpc(method).checked(schema, fresh),
            &[8, 7],
            Decoder::new,
        )
}
fn manifest(dir: &std::path::Path) -> Check<std::path::PathBuf> {
    Ok(std::fs::read_dir(dir.join("mygene"))?
        .filter_map(Result::ok)
        .map(|v| v.path())
        .find(|p| p.extension().is_some_and(|v| v == "json"))
        .ok_or("missing manifest")?)
}
#[test]
fn complete_recording_replays_checked_tensors_without_connections() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
    let dir = tempfile::tempdir()?;
    let record = fetch(
        &fixture,
        Mode::Record {
            dir: dir.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    assert_eq!(
        get(&record, source, METHOD, SCHEMA, Freshness::Never)?,
        [1, 2, 3, 4]
    );
    drop(record);
    let path = manifest(dir.path())?;
    let mut data: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    assert_eq!(data["exchanges"][0]["rpc"]["terminal"], 0);
    data["exchanges"][0]["rpc"]["messages"][0]["delay_ms"] = 1000.into();
    data["exchanges"][0]["rpc"]["messages"][1]["delay_ms"] = 1000.into();
    data["exchanges"][0]["rpc"]["delay_ms"] = 1000.into();
    std::fs::write(&path, serde_json::to_vec(&data)?)?;
    let replay = fetch(
        &fixture,
        Mode::Replay {
            dir: dir.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    let leg = replay.leg(source, replay.deadline(Duration::from_secs(5)));
    assert_eq!(
        leg.grpc_checked(
            &source.grpc(METHOD).checked(SCHEMA, Freshness::Never),
            &[8, 7],
            Decoder::new
        )?,
        [1, 2, 3, 4]
    );
    assert_eq!(leg.remaining(), Duration::from_secs(2));
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 1);
    fixture.finish()
}
#[test]
fn endpoint_ceilings_refuse_recorded_requests_messages_and_cumulative_wire() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
    let dir = tempfile::tempdir()?;
    let recording = fetch(
        &fixture,
        Mode::Record {
            dir: dir.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    assert_eq!(
        get(&recording, source, METHOD, SCHEMA, Freshness::Never)?,
        [1, 2, 3, 4]
    );
    drop(recording);
    for limits in [
        Limits {
            request_bytes: 1,
            ..Default::default()
        },
        Limits {
            message_bytes: 6,
            ..Default::default()
        },
        // Each message fits the derived per-message cap; their sum alone exceeds this ceiling.
        Limits {
            wire_bytes: 14,
            ..Default::default()
        },
    ] {
        let replay = fetch(
            &fixture,
            Mode::Replay {
                dir: dir.path().to_owned(),
            },
            None,
            Limits::default(),
        )?;
        let leg = replay.leg(source, replay.deadline(Duration::from_secs(5)));
        let result = leg.grpc_checked(
            &source
                .grpc(METHOD)
                .checked(SCHEMA, Freshness::Never)
                .bounded(limits),
            &[8, 7],
            Decoder::new,
        );
        assert!(result.is_err());
    }
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 1);
    fixture.finish()
}
#[test]
fn incomplete_and_failed_streams_never_enter_the_cache() -> Check {
    for (hold, status, messages) in [
        (Hold::Interrupt, 0, messages()),
        (Hold::None, 8, messages()),
        (Hold::None, 0, vec![messages()[0].clone()]),
        (Hold::None, 0, vec![messages()[1].clone()]),
    ] {
        let fixture = Fixture::with_messages(hold, status, false, false, messages)?;
        let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
        let dir = tempfile::tempdir()?;
        let cache = dir.path().join("cache.sqlite");
        let recording = dir.path().join("record");
        let record = fetch(
            &fixture,
            Mode::Record {
                dir: recording.clone(),
            },
            Some(cache.clone()),
            Limits::default(),
        )?;
        assert!(get(&record, source, METHOD, SCHEMA, Freshness::Hours(1)).is_err());
        drop(record);
        let data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(manifest(&recording)?)?)?;
        assert!(
            data["exchanges"][0]["rpc"]["interruption"].is_string()
                || data["exchanges"][0]["rpc"]["terminal"] == 8
        );
        let offline = fetch(&fixture, Mode::Offline, Some(cache), Limits::default())?;
        assert!(
            matches!(get(&offline,source,METHOD,SCHEMA,Freshness::Hours(1)), Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::Offline)
        );
        let replay = fetch(
            &fixture,
            Mode::Replay { dir: recording },
            None,
            Limits::default(),
        )?;
        assert!(get(&replay, source, METHOD, SCHEMA, Freshness::Never).is_err());
        assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 1);
        fixture.finish()?;
    }
    Ok(())
}
#[test]
fn cache_identity_and_current_decoder_limits_control_offline_reuse() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
    let dir = tempfile::tempdir()?;
    let cache = dir.path().join("cache.sqlite");
    let live = fetch(
        &fixture,
        Mode::default(),
        Some(cache.clone()),
        Limits::default(),
    )?;
    assert_eq!(
        get(&live, source, METHOD, SCHEMA, Freshness::Hours(1))?,
        [1, 2, 3, 4]
    );
    let offline = fetch(
        &fixture,
        Mode::Offline,
        Some(cache.clone()),
        Limits::default(),
    )?;
    assert_eq!(
        get(&offline, source, METHOD, SCHEMA, Freshness::Hours(1))?,
        [1, 2, 3, 4]
    );
    for (method, schema, fresh) in [
        ("/synthetic.Tensor/Other", SCHEMA, Freshness::Hours(1)),
        (METHOD, "schema-v2", Freshness::Hours(1)),
        (METHOD, SCHEMA, Freshness::Never),
    ] {
        assert!(get(&offline, source, method, schema, fresh).is_err());
    }
    let bounded = fetch(
        &fixture,
        Mode::Offline,
        Some(cache),
        Limits {
            expanded_bytes: 3,
            ..Default::default()
        },
    )?;
    assert!(get(&bounded, source, METHOD, SCHEMA, Freshness::Hours(1)).is_err());
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 1);
    fixture.finish()
}
#[test]
fn replay_refuses_missing_terminal_unsafe_or_missing_sidecars_and_current_limits() -> Check {
    let fixture = Fixture::with_messages(Hold::None, 0, false, false, messages())?;
    let source = fixture.with_quota(false, KeyUse::None, &QUOTA);
    let dir = tempfile::tempdir()?;
    let record = fetch(
        &fixture,
        Mode::Record {
            dir: dir.path().to_owned(),
        },
        None,
        Limits::default(),
    )?;
    get(&record, source, METHOD, SCHEMA, Freshness::Never)?;
    drop(record);
    let path = manifest(dir.path())?;
    let original: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    for (field, value) in [
        ("terminal", serde_json::Value::Null),
        (
            "messages",
            serde_json::json!([{"sidecar":"../escape.bin","delay_ms":0}]),
        ),
        (
            "messages",
            serde_json::json!([{"sidecar":"missing.bin","delay_ms":0}]),
        ),
    ] {
        let mut data = original.clone();
        data["exchanges"][0]["rpc"][field] = value;
        std::fs::write(&path, serde_json::to_vec(&data)?)?;
        let replay = fetch(
            &fixture,
            Mode::Replay {
                dir: dir.path().to_owned(),
            },
            None,
            Limits::default(),
        )?;
        assert!(get(&replay, source, METHOD, SCHEMA, Freshness::Never).is_err());
    }
    std::fs::write(&path, serde_json::to_vec(&original)?)?;
    for limits in [
        Limits {
            message_bytes: 6,
            ..Default::default()
        },
        Limits {
            messages: 1,
            ..Default::default()
        },
        Limits {
            expanded_bytes: 3,
            ..Default::default()
        },
        Limits {
            wire_bytes: 8,
            ..Default::default()
        },
    ] {
        let replay = fetch(
            &fixture,
            Mode::Replay {
                dir: dir.path().to_owned(),
            },
            None,
            limits,
        )?;
        assert!(get(&replay, source, METHOD, SCHEMA, Freshness::Never).is_err());
    }
    assert_eq!(fixture.count.load(std::sync::atomic::Ordering::SeqCst), 1);
    fixture.finish()
}
#[path = "grpc_persistence/policy.rs"]
mod policy;
#[path = "grpc_persistence/process.rs"]
mod process;
