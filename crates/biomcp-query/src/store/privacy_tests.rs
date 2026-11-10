//! Security contract: licensed routes cannot yet reach query continuation stores from the CLI.
use super::{EvidenceState, Store};
use biomcp_fetch::{Fetch, FetchConfig, Mode, Secret, SecretName, Secrets};
use biomcp_types::{ArticleItem, SourceOutcome};
use std::{path::Path, time::Duration};

type Check = Result<(), Box<dyn std::error::Error>>;
const KEY: &[u8] = b"synthetic-query";
const PAYLOAD: &str = "synthetic licensed payload";

fn caller(path: &Path, key: Option<&str>, hosted: bool) -> Result<Fetch, biomcp_fetch::FetchError> {
    let fetch = Fetch::open(FetchConfig {
        cache: Some(path.into()),
        cache_owners: vec![super::OWNER],
        ..FetchConfig::default()
    })?;
    let mut secrets = Secrets::default();
    if let Some(key) = key {
        secrets.insert(SecretName::OncokbApiKey, Secret::new(key));
    }
    Ok(if hosted {
        fetch.with_hosted_secrets(secrets)
    } else {
        fetch.with_secrets(secrets)
    })
}
fn store(fetch: &Fetch) -> Result<Store<'_>, biomcp_fetch::CacheError> {
    Store::open(
        fetch.cache_access()?,
        fetch.deadline(Duration::from_secs(20)),
    )
}
fn source(private: bool) -> SourceOutcome {
    let source = if private {
        biomcp_types::Source::Oncokb
    } else {
        biomcp_types::Source::PubMed
    };
    biomcp_adapters::registry::outcome(source, biomcp_types::SourceStatus::Ok)
}
fn item() -> ArticleItem {
    ArticleItem {
        text_withheld: None,
        context_license: None,
        key: None,
        pmid: None,
        found_by: Default::default(),
        cited_by: None,
        sentence: None,
        intents: None,
        influential: None,
        contexts: None,
        header: None,
    }
}

#[test]
fn stored_licensed_owners_admit_same_key_and_refuse_other_or_missing_keys() -> Check {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("cache.sqlite");
    let first = caller(&path, Some("synthetic-owner-a"), true)?;
    let sources = [source(true)];
    let mut a = store(&first)?;
    let pool = a.publish(KEY, &[item()], &sources, &[], false)?;
    a.publish_current(&pool, KEY)?;
    let window = a.publish_source_pool(KEY, PAYLOAD, 1, &sources, false)?;
    let snapshot = a.save_source_snapshot(KEY, PAYLOAD, &sources)?;
    let evidence = a.save_evidence(None, KEY, PAYLOAD, &sources)?;
    for key in [Some("synthetic-owner-a"), Some("synthetic-owner-b"), None] {
        let same = key == Some("synthetic-owner-a");
        let fetch = caller(&path, key, true)?;
        let mut current = store(&fetch)?;
        assert_eq!(
            current.find(KEY, Some(pool.info.id.as_str()))?.is_some(),
            same
        );
        assert_eq!(current.find(KEY, None)?.is_some(), same);
        assert_eq!(current.page(pool.info.id.as_str(), 0, 10).is_ok(), same);
        assert_eq!(
            current
                .source_snapshot(snapshot.as_str(), KEY, |_| false)?
                .is_some(),
            same
        );
        assert_eq!(
            current
                .source_snapshot(window.info.id.as_str(), KEY, |_| false)?
                .is_some(),
            same
        );
        assert_eq!(
            matches!(current.evidence(evidence.as_str(), KEY, |_| false)?, EvidenceState::Ready(text) if text == PAYLOAD),
            same
        );
        assert_eq!(
            current
                .hydrate(pool.info.id.as_str(), &[], &[], &[], None, false)
                .is_ok(),
            same
        );
        assert_eq!(
            current
                .save_evidence(Some(&evidence), KEY, PAYLOAD, &sources)
                .is_ok(),
            same
        );
        let supplied = super::pool::Pool {
            info: window.info.clone(),
            key: KEY.into(),
            sources: sources.to_vec(),
            problems: Vec::new(),
            dropped: false,
        };
        assert_eq!(
            current
                .extend_source_pool(supplied, PAYLOAD, 2, &sources, false)
                .is_ok(),
            same
        );
        if same {
            assert_eq!(
                current.save_source_snapshot(KEY, PAYLOAD, &sources)?,
                snapshot
            );
        } else if key.is_some() {
            assert_ne!(
                current.save_source_snapshot(KEY, PAYLOAD, &sources)?,
                snapshot
            );
        }
    }
    assert_eq!(
        a.source_snapshot(snapshot.as_str(), KEY, |_| false)?,
        Some(PAYLOAD.into())
    );
    storage_privacy(&first, pool.info.id.as_str())?;
    Ok(())
}

fn storage_privacy(fetch: &Fetch, id: &str) -> Check {
    let connection = fetch.cache_access()?.connect()?;
    let outcomes: String =
        connection.query_row("SELECT outcomes FROM pool WHERE id=?1", [id], |r| r.get(0))?;
    assert!(outcomes.contains("ownership"));
    assert!(!outcomes.contains("synthetic-owner"));
    assert!(
        !serde_json::to_string(
            &store(fetch)?
                .find(KEY, Some(id))?
                .ok_or("pool missing")?
                .sources
        )?
        .contains("ownership")
    );
    Ok(())
}

#[test]
fn public_stores_remain_shared_and_legacy_licensed_rows_require_ownership() -> Check {
    for private in [false, true] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("cache.sqlite");
        let original = caller(&path, Some("synthetic-owner-a"), false)?;
        let mut ordinary = store(&original)?;
        let sources = [source(private)];
        let pool = ordinary.publish(KEY, &[item()], &sources, &[], false)?;
        ordinary.publish_current(&pool, KEY)?;
        let window = ordinary.publish_source_pool(KEY, PAYLOAD, 1, &sources, false)?;
        let snapshot = ordinary.save_source_snapshot(KEY, PAYLOAD, &sources)?;
        let evidence = ordinary.save_evidence(None, KEY, PAYLOAD, &sources)?;
        for key in [Some("synthetic-owner-a"), Some("synthetic-owner-b"), None] {
            let fetch = caller(&path, key, true)?;
            let mut hosted = store(&fetch)?;
            assert_eq!(
                hosted.find(KEY, Some(pool.info.id.as_str()))?.is_some(),
                !private
            );
            assert_eq!(hosted.find(KEY, None)?.is_some(), !private);
            assert_eq!(hosted.page(pool.info.id.as_str(), 0, 1).is_ok(), !private);
            assert_eq!(
                hosted
                    .source_snapshot(snapshot.as_str(), KEY, |_| false)?
                    .is_some(),
                !private
            );
            assert_eq!(
                hosted
                    .source_snapshot(window.info.id.as_str(), KEY, |_| false)?
                    .is_some(),
                !private
            );
            assert_eq!(
                matches!(
                    hosted.evidence(evidence.as_str(), KEY, |_| false)?,
                    EvidenceState::Ready(_)
                ),
                !private
            );
            assert_eq!(
                hosted
                    .hydrate(pool.info.id.as_str(), &[], &[], &[], None, false)
                    .is_ok(),
                !private
            );
            assert_eq!(
                hosted
                    .save_evidence(Some(&evidence), KEY, PAYLOAD, &sources)
                    .is_ok(),
                !private
            );
            if !private {
                assert_eq!(
                    hosted.save_source_snapshot(KEY, PAYLOAD, &sources)?,
                    snapshot
                );
            }
        }
        let other = caller(&path, None, false)?;
        assert!(
            store(&other)?
                .find(KEY, Some(pool.info.id.as_str()))?
                .is_some()
        );
        assert_eq!(
            ordinary.source_snapshot(snapshot.as_str(), KEY, |_| false)?,
            Some(PAYLOAD.into())
        );
    }
    Ok(())
}

#[test]
fn dropped_pool_facts_keep_caller_ownership_during_hydration() -> Check {
    for private in [false, true] {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("cache.sqlite");
        let first = caller(&path, Some("synthetic-owner-a"), true)?;
        let sources = [source(private)];
        let mut a = store(&first)?;
        let pool = a.publish(KEY, &[item()], &sources, &[], true)?;
        assert!(
            a.find(KEY, Some(pool.info.id.as_str()))?
                .ok_or("pool missing")?
                .dropped
        );
        a.hydrate(pool.info.id.as_str(), &[], &[], &[], None, true)?;
        assert!(
            a.find(KEY, Some(pool.info.id.as_str()))?
                .ok_or("pool missing")?
                .dropped
        );
        let other = caller(&path, Some("synthetic-owner-b"), true)?;
        let mut b = store(&other)?;
        assert_eq!(
            b.find(KEY, Some(pool.info.id.as_str()))?.is_some(),
            !private
        );
        assert_eq!(
            b.hydrate(pool.info.id.as_str(), &[], &[], &[], None, true)
                .is_ok(),
            !private
        );
        assert!(
            a.find(KEY, Some(pool.info.id.as_str()))?
                .ok_or("pool missing")?
                .dropped
        );
    }
    Ok(())
}

// SQLite connection state cannot be observed through the CLI; the installed
// held-store-lock case owns the reader and sync behavior.
#[test]
fn query_connections_keep_virtual_sqlite_waits_zero_and_live_waits_bounded() -> Check {
    for replay in [false, true] {
        let directory = tempfile::tempdir()?;
        let fetch = Fetch::open(FetchConfig {
            mode: if replay {
                Mode::Replay {
                    dir: directory.path().into(),
                }
            } else {
                Mode::default()
            },
            cache: Some(directory.path().join("cache.sqlite")),
            cache_owners: vec![super::OWNER],
            ..FetchConfig::default()
        })?;
        let store = Store::open(
            fetch.cache_access()?,
            fetch.deadline(Duration::from_secs(120)),
        )?;
        let milliseconds: i64 = store.read(|connection| {
            connection
                .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
                .map_err(super::db)
        })?;
        if replay {
            assert_eq!(
                milliseconds, 0,
                "a virtual deadline cannot drive a real SQLite wait"
            );
        } else {
            assert!(
                milliseconds > 0 && milliseconds <= 120_000,
                "live SQLite waits must retain the remaining real budget: {milliseconds}"
            );
        }
    }
    Ok(())
}

#[test]
fn source_windows_admit_only_the_recorded_owner_and_enabled_source() -> Check {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("cache.sqlite");
    let fetch = caller(&path, Some("synthetic-owner-a"), true)?;
    let window = store(&fetch)?.publish_source_pool(KEY, PAYLOAD, 1, &[source(true)], false)?;
    for key in [Some("synthetic-owner-a"), Some("synthetic-owner-b"), None] {
        let fetch = caller(&path, key, true)?;
        let current = store(&fetch)?;
        for selected in [None, Some(window.info.id.as_str())] {
            assert_eq!(
                current
                    .find_source_pool(KEY, selected, |_| false)?
                    .is_some(),
                key == Some("synthetic-owner-a")
            );
            assert!(current.find_source_pool(KEY, selected, |_| true)?.is_none());
        }
    }
    Ok(())
}

// Stale handles stand in for delayed concurrent writers at the transaction boundary.
#[test]
fn source_defaults_follow_only_the_retained_complete_observation() -> Check {
    let directory = tempfile::tempdir()?;
    let fetch = caller(&directory.path().join("cache.sqlite"), None, false)?;
    let mut a = store(&fetch)?;
    let mut b = store(&fetch)?;
    let sources = [source(false)];
    let healthy = a.publish_source_pool(KEY, "healthy", 2, &sources, false)?;
    let old = b
        .find_source_pool(KEY, Some(healthy.info.id.as_str()), |_| false)?
        .ok_or("healthy")?
        .0;
    let partial = b.publish_source_pool(KEY, "partial", 1, &sources, true)?;
    assert_eq!(
        a.find_source_pool(KEY, None, |_| false)?
            .ok_or("current")?
            .1,
        "healthy"
    );
    b.extend_source_pool(old, "shorter", 1, &sources, true)?;
    assert_eq!(
        a.find_source_pool(KEY, None, |_| false)?
            .ok_or("current")?
            .1,
        "healthy"
    );
    let newer = a.publish_source_pool(KEY, "newer", 3, &sources, false)?;
    b.extend_source_pool(healthy, "incomplete old extension", 3, &sources, true)?;
    assert_eq!(
        a.find_source_pool(KEY, None, |_| false)?
            .ok_or("current")?
            .0
            .info
            .id,
        newer.info.id
    );
    b.extend_source_pool(partial, "recovered explicit", 2, &sources, false)?;
    assert_eq!(
        a.find_source_pool(KEY, None, |_| false)?
            .ok_or("current")?
            .1,
        "newer"
    );
    let mut stale = source(false);
    stale.set_status(biomcp_types::SourceStatus::Stale(None));
    b.extend_source_pool(newer, "newer", 3, &[stale], false)?;
    assert!(a.find_source_pool(KEY, None, |_| false)?.is_none());
    Ok(())
}

#[test]
fn legacy_source_defaults_do_not_change_explicit_id_admission() -> Check {
    let directory = tempfile::tempdir()?;
    let fetch = caller(&directory.path().join("cache.sqlite"), None, false)?;
    let mut a = store(&fetch)?;
    let sources = [source(false)];
    let partial = a.publish_source_pool(KEY, "partial", 1, &sources, true)?;
    a.write(|tx| {
        tx.execute(
            "INSERT INTO pool_current(request_key,pool) VALUES(?1,?2)",
            rusqlite::params![KEY, partial.info.id.as_str()],
        )
        .map_err(super::db)?;
        Ok(())
    })?;
    assert!(a.find_source_pool(KEY, None, |_| false)?.is_none());
    assert_eq!(
        a.find_source_pool(KEY, Some(partial.info.id.as_str()), |_| false)?
            .ok_or("explicit")?
            .1,
        "partial"
    );
    let private = a.publish_source_pool(KEY, "private", 1, &[source(true)], false)?;
    let other = caller(
        &directory.path().join("cache.sqlite"),
        Some("synthetic-other"),
        true,
    )?;
    assert!(
        store(&other)?
            .find_source_pool(KEY, Some(private.info.id.as_str()), |_| false)?
            .is_none()
    );
    Ok(())
}
