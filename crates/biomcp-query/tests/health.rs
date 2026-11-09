//! Health honors the public caller and source budgets, without wall-clock sleeps.
#![expect(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    reason = "This public health fixture owns isolated synthetic recording I/O and must have valid setup"
)]
use biomcp_adapters::registry;
use biomcp_fetch::{Fetch, FetchConfig, Mode};
use biomcp_test_support::network_stub::Stub;
use biomcp_types::{Reason, Source, SourceState};
use std::{path::Path, time::Duration};

fn config(mode: Mode) -> FetchConfig {
    FetchConfig {
        mode,
        disabled_sources: registry::sources()
            .map(|spec| spec.source)
            .filter(|source| *source != Source::MyGene)
            .collect(),
        ..Default::default()
    }
}

fn capture(root: &Path) -> std::path::PathBuf {
    let stub = Stub::for_source("mygene.info", "application/json", |_| Ok("{}".into())).unwrap();
    let mut settings = config(Mode::Record { dir: root.into() });
    settings.proxy = Some(biomcp_fetch::Secret::new(format!(
        "http://{}",
        stub.address
    )));
    settings.network = biomcp_fetch::Network::new(
        None,
        None,
        "",
        biomcp_fetch::Certificates::from_pem(stub.certificate.as_bytes()).unwrap(),
    )
    .unwrap();
    let fetch = Fetch::open(settings).unwrap();
    let spec = &biomcp_adapters::mygene::SPEC;
    registry::probe(
        spec,
        &fetch.leg(spec, fetch.deadline(Duration::from_secs(20))),
    )
    .unwrap();
    let files: Vec<_> = std::fs::read_dir(root.join("mygene"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1, "exactly one synthetic health request");
    files.into_iter().next().unwrap()
}

#[test]
fn health_uses_command_and_source_budgets_and_preserves_real_failures() {
    let root = tempfile::tempdir().unwrap();
    let path = capture(root.path());
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for (delay, status, command, source, state, reason) in [
        (4_000, 200, 20, None, SourceState::Ok, None),
        (
            4_000,
            200,
            2,
            None,
            SourceState::Failed,
            Some(Reason::DeadlineExceeded),
        ),
        (
            4_000,
            200,
            20,
            Some(2),
            SourceState::Failed,
            Some(Reason::DeadlineExceeded),
        ),
        (
            9_000,
            200,
            20,
            None,
            SourceState::Failed,
            Some(Reason::DeadlineExceeded),
        ),
        (
            0,
            503,
            20,
            None,
            SourceState::Failed,
            Some(Reason::Unavailable),
        ),
    ] {
        let mut recording = original.clone();
        let exchange = recording.pointer_mut("/exchanges/0").unwrap();
        *exchange.get_mut("delay_ms").unwrap() = delay.into();
        *exchange.get_mut("status").unwrap() = status.into();
        std::fs::write(&path, serde_json::to_vec(&recording).unwrap()).unwrap();
        let mut settings = config(Mode::Replay {
            dir: root.path().into(),
        });
        if let Some(seconds) = source {
            settings
                .source_deadlines
                .insert(Source::MyGene, Duration::from_secs(seconds));
        }
        let fetch = Fetch::open(settings).unwrap();
        let health = biomcp_query::health::run(&fetch, Duration::from_secs(command)).unwrap();
        let observed = health
            .sources
            .iter()
            .find(|row| row.source == Source::MyGene)
            .unwrap();
        assert_eq!(
            (observed.state, observed.reason),
            (state, reason),
            "delay={delay}, status={status}, command={command}, source={source:?}"
        );
        assert!(
            health
                .sources
                .iter()
                .filter(|row| row.source != Source::MyGene)
                .all(|row| row.state == SourceState::Skipped)
        );
        assert_eq!(
            observed.latency_ms,
            Some(delay.min(command * 1_000).min(source.unwrap_or(8) * 1_000)),
            "report the budget actually spent"
        );
    }
}
