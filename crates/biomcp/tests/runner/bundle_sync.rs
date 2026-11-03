//! Bundle HTTP recovery through the installed CLI and an isolated local TLS publisher.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use libtest_mimic::Trial;
use serde_json::{Value, json};
#[expect(
    clippy::disallowed_types,
    reason = "Count fixture responses and select response phases across the TLS worker"
)]
use std::sync::atomic::AtomicUsize;
use std::{
    fs,
    io::Write,
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    vec![
        super::transcript::trial("gene-sections/bundle-rate-limit-deadline", || {
            limited(false).map_err(Into::into)
        }),
        super::transcript::trial("gene-sections/bundle-rate-limit-short-retry", || {
            limited(true).map_err(Into::into)
        }),
        super::transcript::trial("gene-sections/bundle-etag-refresh", || {
            refresh(false).map_err(Into::into)
        }),
        super::transcript::trial("gene-sections/bundle-modified-refresh", || {
            refresh(true).map_err(Into::into)
        }),
    ]
}
fn body() -> Result<String, String> {
    let root = super::launch::fixtures().join("replies/gene/gencc/gencc");
    let file = fs::read_dir(root)
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("missing GenCC fixture")?
        .map_err(|e| e.to_string())?;
    let record: Value = serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    record
        .get("exchanges")
        .and_then(Value::as_array)
        .ok_or("missing exchanges")?
        .iter()
        .find(|value| value.get("status") == Some(&json!(200)))
        .and_then(|value| value.pointer("/body/value"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or("missing CSV".into())
}
fn configure(env: &Environment, peer: &Stub) -> Check {
    peer.retain(env.shared_root());
    let ca = env.root().join("ca.pem");
    fs::write(&ca, &peer.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"proxy":format!("http://{}",peer.address),"ca_bundle":ca}))
        .map_err(|e| e.to_string())
}
fn sync(env: &Environment) -> Result<std::process::Output, String> {
    env.command(
        env!("CARGO_BIN_EXE_biomcp"),
        &["sync".into(), "gencc".into()],
    )
    .output()
    .map_err(|e| e.to_string())
}
#[expect(
    clippy::disallowed_types,
    reason = "Observe the number of admitted fixture requests across the TLS worker"
)]
fn limited(short: bool) -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let count = Arc::new(AtomicUsize::new(0));
    let body_count = count.clone();
    let status_count = count.clone();
    let csv = body()?;
    let peer = Stub::with_headers(
        &["thegencc.org"],
        |_| "text/csv",
        move |_| {
            let attempt = body_count.fetch_add(1, Ordering::SeqCst);
            Ok(if short && attempt > 0 {
                csv.clone()
            } else {
                String::new()
            })
        },
        |_, n| n,
        move |_| {
            if short && status_count.load(Ordering::SeqCst) > 1 {
                200
            } else {
                429
            }
        },
        move |_| format!("Retry-After: {}\r\n", if short { 1 } else { 16433 }),
    )?;
    configure(&env, &peer)?;
    let started = Instant::now();
    let output = sync(&env)?;
    writeln!(
        std::io::stdout(),
        "bundle retry short={short} elapsed_ms={} sync_budget_seconds={}",
        started.elapsed().as_millis(),
        biomcp_fetch::FetchConfig::default().sync_budget.as_secs()
    )
    .map_err(|e| e.to_string())?;
    if short {
        if !output.status.success()
            || count.load(Ordering::SeqCst) != 2
            || started.elapsed() < Duration::from_secs(1)
        {
            return Err(format!("short retry did not recover: {output:?}"));
        }
    } else {
        if count.load(Ordering::SeqCst) != 1 {
            return Err("long retry made another source request".into());
        }
        rate_limited(&output)?;
    }
    peer.finish()?;
    Ok(())
}
fn database(env: &Environment) -> Result<rusqlite::Connection, String> {
    let path = fs::read_dir(env.root().join("cache/biomcp"))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "sqlite")
        })
        .ok_or("missing database")?;
    rusqlite::Connection::open(path).map_err(|e| e.to_string())
}
fn snapshot(db: &rusqlite::Connection) -> Result<(String, String, i64), String> {
    db.query_row(concat!("SELECT (SELECT group_concat(row) FROM bundle_row WHERE bundle=b.name),c.revision,c.retrieved_at ",
        "FROM bundle b JOIN bundle_content c ON c.name=b.name WHERE b.name='gencc-submissions'"),
        [], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?))).map_err(|e| e.to_string())
}
#[expect(
    clippy::disallowed_types,
    reason = "Select response phases after observing installed CLI completions"
)]
fn refresh(modified: bool) -> Check {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let stage = Arc::new(AtomicUsize::new(0));
    let peer = publisher(stage.clone(), modified)?;
    configure(&env, &peer)?;
    invoke(&env, vec!["sync".into(), "gencc".into()], 0)?;
    let db = database(&env)?;
    let before = snapshot(&db)?;
    db.execute_batch("UPDATE bundle SET fresh_until=0 WHERE name='gencc-submissions';
        CREATE TRIGGER forbid_bundle_row_delete BEFORE DELETE ON bundle_row BEGIN SELECT RAISE(ABORT,'unchanged rows rewritten'); END;
        CREATE TRIGGER forbid_bundle_row_insert BEFORE INSERT ON bundle_row BEGIN SELECT RAISE(ABORT,'unchanged rows rewritten'); END;")
        .map_err(|e| e.to_string())?;
    // The normal stale marker invalidates provenance; restore its original facts before testing revalidation.
    db.execute(
        "INSERT INTO bundle_content VALUES('gencc-submissions',?1,?2)",
        rusqlite::params![before.1, before.2],
    )
    .map_err(|e| e.to_string())?;
    stage.store(1, Ordering::SeqCst);
    let value = invoke(&env, vec!["sync".into(), "gencc".into()], 0)?;
    let fresh: bool = db
        .query_row(
            "SELECT fresh_until>unixepoch() FROM bundle WHERE name='gencc-submissions'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    if value.pointer("/result/downloaded_bytes") != Some(&json!(0))
        || !fresh
        || snapshot(&db)? != before
    {
        return Err(format!(
            "304 changed stored content or failed to extend freshness: {value}"
        ));
    }
    db.execute_batch(
        "DROP TRIGGER forbid_bundle_row_delete; DROP TRIGGER forbid_bundle_row_insert;",
    )
    .map_err(|e| e.to_string())?;
    stage.store(2, Ordering::SeqCst);
    let value = invoke(&env, vec!["sync".into(), "gencc".into()], 0)?;
    if value
        .pointer("/result/downloaded_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
        || (!modified && value.pointer("/result/etag") != Some(&json!("\"changed\"")))
    {
        return Err(format!("changed answer was not downloaded: {value}"));
    }
    let retained = snapshot(&db)?;
    if retained == before {
        return Err("changed answer retained the old row order and revision".into());
    }
    stage.store(3, Ordering::SeqCst);
    let failure = sync(&env)?;
    if failure.status.code() != Some(4) || snapshot(&db)? != retained {
        return Err("failed refresh lost the stored copy".into());
    }
    if peer.finish()?.len() != 4 {
        return Err("unexpected bundle body request count".into());
    }
    Ok(())
}

#[expect(
    clippy::disallowed_types,
    reason = "Select the source response after each completed installed CLI sync"
)]
fn publisher(stage: Arc<AtomicUsize>, modified: bool) -> Result<Stub, String> {
    let reply_stage = stage.clone();
    let status_stage = stage.clone();
    let header_stage = stage.clone();
    let csv = body()?;
    let mut lines = csv.lines();
    let header = lines.next().ok_or("missing CSV header")?;
    let mut rows: Vec<_> = lines.collect();
    rows.reverse();
    let changed = format!("{header}\n{}\n", rows.join("\n"));
    let original = csv.clone();
    Stub::with_headers(
        &["thegencc.org"],
        |_| "text/csv",
        move |request| {
            let phase = reply_stage.load(Ordering::SeqCst);
            if phase > 0 {
                let expected = if modified {
                    "if-modified-since: wed, 21 oct 2015 07:28:00 gmt"
                } else if phase == 3 {
                    "if-none-match: \"changed\""
                } else {
                    "if-none-match: \"first\""
                };
                if !request.to_ascii_lowercase().contains(expected) {
                    return Err(format!("refresh omitted stored validator: {request}"));
                }
            }
            Ok(match phase {
                1 | 3 => String::new(),
                2 => changed.clone(),
                _ => original.clone(),
            })
        },
        |_, n| n,
        move |_| match status_stage.load(Ordering::SeqCst) {
            1 => 304,
            3 => 429,
            _ => 200,
        },
        move |_| {
            if header_stage.load(Ordering::SeqCst) == 3 {
                "Retry-After: 16433\r\n".into()
            } else if modified {
                "Last-Modified: Wed, 21 Oct 2015 07:28:00 GMT\r\n".into()
            } else if header_stage.load(Ordering::SeqCst) == 2 {
                "ETag: \"changed\"\r\n".into()
            } else {
                "ETag: \"first\"\r\n".into()
            }
        },
    )
}

fn rate_limited(output: &std::process::Output) -> Check {
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.code() != Some(4)
        || value.pointer("/sources/0/reason") != Some(&json!("rate-limited"))
        || value.pointer("/sources/0/notices/0/retry_after_seconds") != Some(&json!(16433))
        || !stderr.contains("rate-limited")
        || !stderr.contains("16433")
    {
        return Err(format!(
            "long retry did not stop with source guidance: {value}; {stderr}"
        ));
    }
    Ok(())
}
