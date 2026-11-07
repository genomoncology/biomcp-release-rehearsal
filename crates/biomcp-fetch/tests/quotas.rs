//! Public quota declaration boundaries and shared provider behavior.
#![expect(
    clippy::disallowed_methods,
    reason = "Scoped callers exercise shared public transport state against controlled local HTTP peers"
)]
use biomcp_fetch::{Input, Read};

use biomcp_fetch::{Quota, Rate, SourceKey};

#[test]
fn quota_declarations_reject_empty_invalid_and_zero_limits() {
    for key in ["", "Upper", "space here", "slash/name", "é"] {
        assert!(std::panic::catch_unwind(|| SourceKey::declare(key)).is_err());
    }
    for (rate, burst) in [(0, 1), (1, 0), (0, 0)] {
        assert!(std::panic::catch_unwind(|| Rate::new(rate, burst)).is_err());
    }
    assert!(std::panic::catch_unwind(|| Rate::every_seconds(0)).is_err());
    assert!(
        std::panic::catch_unwind(|| Quota::new(
            SourceKey::declare("valid-1"),
            Rate::per_second(1),
            None,
            0
        ))
        .is_err()
    );
    const VALID: Quota = Quota::new(
        SourceKey::declare("valid-1"),
        Rate::new(u16::MAX, u16::MAX),
        Some(Rate::per_second(1)),
        u8::MAX,
    );
    assert!(format!("{VALID:?}").contains("valid-1"));
}

mod support;
#[path = "support/timing.rs"]
mod timing;
use biomcp_fetch::{
    Credential, Fetch, FetchConfig, KeyPlace, KeyUse, Mode, RateShare, RequestError, Secret,
    SecretName, Secrets, SourceSpec,
};
use biomcp_types::Reason;
use sha2::{Digest, Sha256};
use std::{sync::mpsc, thread, time::Duration};
use support::Answer;
use timing::Server;

fn quota(key: &'static str, rate: Rate, keyed: Option<Rate>, in_flight: u8) -> &'static Quota {
    Box::leak(Box::new(Quota::new(
        SourceKey::declare(key),
        rate,
        keyed,
        in_flight,
    )))
}

fn serial_override(shared: &Quota) -> Result<Fetch, Box<dyn std::error::Error>> {
    let mut source_pacing = biomcp_fetch::SourcePacing::default();
    source_pacing.insert(
        shared,
        biomcp_fetch::PacingOverride {
            in_flight: Some(1),
            ..Default::default()
        },
    )?;
    Ok(Fetch::open(FetchConfig {
        source_pacing,
        ..Default::default()
    })?)
}

#[test]
fn canceled_waiter_leaves_shared_in_flight_queue_and_releases_successor() {
    let server = Server::new();
    let shared = quota("shared", Rate::per_second(1000), None, 2);
    let first = server.spec(shared, KeyUse::None);
    let second = server.spec(shared, KeyUse::None);
    let independent = server.spec(
        quota("independent", Rate::per_second(1000), None, 1),
        KeyUse::None,
    );
    let fetch = serial_override(shared).unwrap();
    let restricted = fetch.restrict_sources([biomcp_types::Source::Oncokb].into());
    assert!(!fetch.disabled(biomcp_types::Source::Oncokb));
    assert!(
        restricted
            .restrict_sources(Default::default())
            .disabled(biomcp_types::Source::Oncokb)
    );
    thread::scope(|scope| {
        let holder = scope.spawn(|| {
            fetch
                .leg(first, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &first.get("/holder"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let held = server.next();
        let canceled = scope.spawn(|| {
            restricted
                .leg(second, restricted.deadline(Duration::from_millis(100)))
                .call(
                    &second.get("/cancel"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        // Completion with the holder still blocked proves the waiter exhausted its queue budget.
        assert_eq!(
            source(canceled.join().unwrap().unwrap_err()).reason,
            Reason::DeadlineExceeded
        );
        assert!(server.requests.try_recv().is_err());
        let successor = scope.spawn(|| {
            restricted
                .leg(second, restricted.deadline(Duration::from_secs(5)))
                .call(
                    &second.get("/successor"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let other = scope.spawn(|| {
            fetch
                .leg(independent, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &independent.get("/other"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let request = server.next();
        assert!(request.text.contains("/other"));
        request.ok();
        assert!(other.join().unwrap().unwrap().answer);
        held.ok();
        server.next().ok();
        assert!(holder.join().unwrap().unwrap().answer);
        assert!(successor.join().unwrap().unwrap().answer);
    });
}

#[test]
fn configured_optional_key_selects_keyed_rate_without_bypassing_shared_queue() {
    for configured in [false, true] {
        let server = Server::new();
        let shared = quota(
            "keyed",
            Rate::every_seconds(2),
            Some(Rate::per_second(1)),
            1,
        );
        let spec = server.spec(
            shared,
            KeyUse::Optional(Credential {
                tier: biomcp_types::KeyTier::Fallback,
                secret: SecretName::SemanticScholarApiKey,
                place: KeyPlace::Header("x-api-key"),
            }),
        );
        assert_eq!(
            shared.interval(configured),
            Duration::from_secs(if configured { 1 } else { 2 })
        );
        let mut config = FetchConfig::default();
        if configured {
            config.secrets.insert(
                SecretName::SemanticScholarApiKey,
                Secret::new("placeholder"),
            );
        }
        let fetch = Fetch::open(config).unwrap();
        let (sent, receive) = mpsc::channel();
        thread::scope(|scope| {
            scope.spawn(|| {
                for _ in 0..2 {
                    let result = fetch
                        .leg(spec, fetch.deadline(Duration::from_secs(5)))
                        .call(
                            &spec.get("/answer"),
                            Input::Params(&[]),
                            Read::<Answer>::json(),
                        )
                        .map(|received| received.value);
                    sent.send(result).unwrap();
                }
            });
            let first = server.next();
            let started = first.started;
            assert_eq!(
                first
                    .text
                    .to_ascii_lowercase()
                    .contains("x-api-key: placeholder"),
                configured
            );
            first.ok();
            assert!(receive.recv().unwrap().unwrap().answer);
            let second = server.next();
            let floor = if configured { 750 } else { 1500 };
            assert!(second.started.duration_since(started) >= Duration::from_millis(floor));
            second.ok();
            assert!(receive.recv().unwrap().unwrap().answer);
        });
        assert!(server.requests.try_recv().is_err());
    }
}

#[test]
fn declared_burst_admits_only_its_capacity_and_expired_rate_waiters_leave() {
    let server = Server::new();
    let spec = server.spec(quota("burst", Rate::new(1, 2), None, 3), KeyUse::None);
    let fetch = Fetch::open(FetchConfig::default()).unwrap();
    thread::scope(|scope| {
        let first = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &spec.get("/first"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let held = server.next();
        let second = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &spec.get("/second"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let also_held = server.next();
        let canceled = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_millis(100)))
                .call(
                    &spec.get("/expired"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        assert_eq!(
            source(canceled.join().unwrap().unwrap_err()).reason,
            Reason::DeadlineExceeded
        );
        assert!(server.requests.try_recv().is_err());
        held.ok();
        also_held.ok();
        assert!(first.join().unwrap().unwrap().answer);
        assert!(second.join().unwrap().unwrap().answer);
        let successor = scope.spawn(|| {
            fetch
                .leg(spec, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &spec.get("/successor"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        server.next().ok();
        assert!(successor.join().unwrap().unwrap().answer);
    });
}

#[test]
fn shared_queue_applies_each_arriving_sources_credential_rate() {
    let server = Server::new();
    let shared = quota("mixed", Rate::per_second(5), Some(Rate::per_second(100)), 1);
    let keyed = server.spec(
        shared,
        KeyUse::Optional(Credential {
            tier: biomcp_types::KeyTier::Fallback,
            secret: SecretName::NcbiApiKey,
            place: KeyPlace::Header("x-api-key"),
        }),
    );
    let unkeyed = server.spec(shared, KeyUse::None);
    let mut config = FetchConfig::default();
    config
        .secrets
        .insert(SecretName::NcbiApiKey, Secret::new("placeholder"));
    let fetch = Fetch::open(config).unwrap();
    thread::scope(|scope| {
        let first = scope.spawn(|| {
            fetch
                .leg(keyed, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &keyed.get("/keyed"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let request = server.next();
        let started = request.started;
        request.ok();
        assert!(first.join().unwrap().unwrap().answer);
        let second = scope.spawn(|| {
            fetch
                .leg(unkeyed, fetch.deadline(Duration::from_secs(5)))
                .call(
                    &unkeyed.get("/unkeyed"),
                    Input::Params(&[]),
                    Read::<Answer>::json(),
                )
                .map(|received| received.value)
        });
        let request = server.next();
        let spacing = request.started.duration_since(started);
        request.ok();
        assert!(second.join().unwrap().unwrap().answer);
        // Arrival jitter can shorten the limiter's 200 ms spacing; no spacing is near zero.
        assert!(spacing >= Duration::from_millis(150), "{spacing:?}");
    });
}

#[expect(
    clippy::panic,
    reason = "A cache error must never satisfy a provider-failure assertion"
)]
fn source(error: biomcp_fetch::RequestError) -> biomcp_fetch::SourceError {
    let biomcp_fetch::RequestError::Source(error) = error else {
        panic!("expected provider failure: {error}");
    };
    error
}

type Check = Result<(), Box<dyn std::error::Error>>;

fn caller(fetch: &Fetch, key: Option<&str>) -> Fetch {
    let mut secrets = Secrets::default();
    if let Some(key) = key {
        secrets.insert(SecretName::NcbiApiKey, Secret::new(key));
    }
    fetch.with_hosted_secrets(secrets)
}

fn fixture(
    share: RateShare,
) -> Result<(tempfile::TempDir, Fetch, &'static SourceSpec), Box<dyn std::error::Error>> {
    fixture_pacing(share, None)
}

fn fixture_pacing(
    share: RateShare,
    pacing: Option<biomcp_fetch::PacingOverride>,
) -> Result<(tempfile::TempDir, Fetch, &'static SourceSpec), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let url = "https://example.invalid/answer";
    let hash = Sha256::digest([b"GET\0".as_slice(), url.as_bytes()].concat())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    std::fs::create_dir(dir.path().join("pubmed"))?;
    std::fs::write(
        dir.path().join("pubmed").join(format!("{hash}.json")),
        serde_json::to_vec(&serde_json::json!({
            "method":"GET","source":"pubmed","url":url,
            "exchanges":vec![serde_json::json!({"status":200,"content_type":"application/json","delay_ms":0,
                "body":{"kind":"json","value":{"answer":true}}}); 3]
        }))?,
    )?;
    let spec = Box::leak(Box::new(SourceSpec {
        source: biomcp_types::Source::PubMed,
        base: "https://example.invalid",
        destinations: biomcp_fetch::Destinations::Primary,
        user_agent: None,
        key: KeyUse::Optional(Credential {
            secret: SecretName::NcbiApiKey,
            place: KeyPlace::Header("x-api-key"),
            tier: biomcp_types::KeyTier::Fallback,
        }),
        quota: Box::leak(Box::new(Quota::new(
            SourceKey::declare("hosted-test"),
            Rate::per_second(2),
            Some(Rate::per_second(4)),
            1,
        ))),
        deadline: Duration::from_secs(10),
        terms: biomcp_fetch::Terms {
            name: "Synthetic test data",
            url: "https://example.test/terms",
            usage: biomcp_types::Use::Open,
        },
    }));
    let mut source_pacing = biomcp_fetch::SourcePacing::default();
    if let Some(pacing) = pacing {
        source_pacing.insert(spec.quota, pacing)?;
    }
    let fetch = Fetch::open(FetchConfig {
        source_pacing,
        mode: Mode::Replay {
            dir: dir.path().into(),
        },
        rate_share: share,
        ..FetchConfig::default()
    })?;
    Ok((dir, fetch, spec))
}

fn get(
    fetch: &Fetch,
    spec: &'static SourceSpec,
    budget: Duration,
) -> Result<Duration, RequestError> {
    let leg = fetch.leg(spec, fetch.deadline(budget));
    let reply: Answer = leg
        .call(&spec.get("/answer"), Input::Params(&[]), Read::json())?
        .value;
    assert!(reply.answer, "the recorded request must complete");
    Ok(leg.remaining())
}

#[test]
fn one_callers_exhausted_bucket_does_not_delay_another_key() -> Check {
    let (_dir, shared, spec) = fixture(RateShare::default())?;
    let a = caller(&shared, Some("placeholder-caller-a"));
    let b = caller(&shared, Some("placeholder-caller-b"));
    get(&a, spec, Duration::from_secs(1))?;
    let error = get(&a, spec, Duration::from_millis(50));
    assert!(
        matches!(error, Err(RequestError::Source(e)) if e.reason == Reason::DeadlineExceeded),
        "the exhausted caller must reach its typed pacing deadline"
    );
    assert_eq!(
        get(&b, spec, Duration::from_millis(50))?,
        Duration::from_millis(50),
        "another credential must have its own initial token"
    );
    Ok(())
}

#[test]
fn separate_keyless_callers_share_one_source_bucket() -> Check {
    let (_dir, shared, spec) = fixture(RateShare::default())?;
    get(&caller(&shared, None), spec, Duration::from_secs(1))?;
    let error = get(&caller(&shared, None), spec, Duration::from_millis(50));
    assert!(
        matches!(error, Err(RequestError::Source(e)) if e.reason == Reason::DeadlineExceeded),
        "a second keyless caller must not receive a new bucket"
    );
    Ok(())
}

#[test]
fn rate_share_scales_only_keyed_hosted_source_rates() -> Check {
    for (key, expected) in [(None, 500), (Some("placeholder-rate-key"), 500)] {
        let (_dir, shared, spec) = fixture("0.5".parse()?)?;
        let fetch = caller(&shared, key);
        get(&fetch, spec, Duration::from_secs(2))?;
        assert_eq!(
            get(&fetch, spec, Duration::from_secs(2))?,
            Duration::from_millis(2000 - expected),
            "only a keyed hosted bucket must scale its source rate"
        );
    }
    Ok(())
}

#[test]
fn rate_share_rejects_zero_nonfinite_and_excess_shares() {
    for value in ["0", "-0.5", "1.01", "NaN", "inf", "-inf", "", "bad"] {
        assert!(
            value.parse::<RateShare>().is_err(),
            "invalid rate share {value}"
        );
    }
    for value in ["1", "0.5", "1e-20"] {
        assert!(
            value.parse::<RateShare>().is_ok(),
            "positive rate share {value}"
        );
    }
}

#[test]
fn operator_rates_space_requests_and_leave_keyed_rates_independent() -> Check {
    use biomcp_fetch::{PacingOverride, PacingRate};
    for (input, keyed, expected) in [
        (
            PacingOverride {
                rate: Some(PacingRate {
                    requests: 1,
                    seconds: 3,
                }),
                ..PacingOverride::default()
            },
            false,
            Duration::from_secs(3),
        ),
        (
            PacingOverride {
                rate: Some(PacingRate {
                    requests: 1,
                    seconds: 3,
                }),
                ..PacingOverride::default()
            },
            true,
            Duration::from_millis(250),
        ),
        (
            PacingOverride {
                keyed_rate: Some(PacingRate {
                    requests: 1,
                    seconds: 2,
                }),
                ..PacingOverride::default()
            },
            true,
            Duration::from_secs(2),
        ),
    ] {
        let (_dir, fetch, spec) = fixture_pacing(RateShare::default(), Some(input))?;
        let fetch = caller(&fetch, keyed.then_some("synthetic-pacing-key"));
        get(&fetch, spec, Duration::from_secs(10))?;
        let remaining = get(&fetch, spec, Duration::from_secs(10))?;
        assert_eq!(Duration::from_secs(10) - remaining, expected);
    }
    Ok(())
}
