//! Feature store-coexistence: allowed reads retain both owned store generations.
use super::{configuration::invoke, launch::Environment};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
type Check<T = ()> = Result<T, Box<dyn std::error::Error>>;
const NAME: &str = "who-ivd";
const ID: &str = "0466-046-00";

pub(super) fn trials() -> Vec<Trial> {
    [
        ("imports-after-safe-cleanup", imports as fn() -> Check),
        ("obsolete-published-bundle-survives-cleanup", obsolete),
        ("current-published-bundle-wins", current),
        ("recorded-stored-read-transfers-without-provider", recorded),
        ("interrupted-transfer-retains-both-files", interrupted),
        ("owner-refusal-preserves-both-files", ownership),
        ("health-and-disabled-read-never-transfer", admission),
    ]
    .into_iter()
    .map(|(name, run)| {
        transcript_trial(format!("store-coexistence/{name}"), move || {
            run().map_err(|e| e.to_string().into())
        })
    })
    .collect()
}
fn files(env: &Environment) -> Check<(PathBuf, PathBuf)> {
    let folder = env.root().join("cache/biomcp");
    fs::create_dir_all(&folder)?;
    let old = folder.join("cache-1-4.sqlite");
    let current = folder.join("cache-1-5.sqlite");
    for path in [&old, &current] {
        drop(biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
            cache: Some(path.clone()),
            cache_owners: vec![biomcp_query::store::OWNER],
            mode: biomcp_fetch::Mode::Offline,
            ..Default::default()
        })?);
    }
    Connection::open(&old)?.execute("INSERT INTO meta VALUES ('biomcp-query',4)", [])?;
    Ok((old, current))
}
fn seed(path: &Path, label: &str, retrieved: i64) -> Check {
    let cx = Connection::open(path)?;
    let row = json!({"part":"products","row":2,"value":{
        "WHO Product ID":ID,"Product name":label,"Product Code":"synthetic",
        "Assay Format":"synthetic","Manufacturer name":"Synthetic",
        "Pathogen/Disease/Marker":"synthetic","Regulatory Version":"synthetic",
        "Year prequalification":"2026"
    }});
    cx.execute(
        "INSERT INTO bundle VALUES (?1,100,1,9223372036854775807,NULL,NULL)",
        [NAME],
    )?;
    cx.execute(
        "INSERT INTO bundle_row VALUES (?1,0,?2)",
        params![NAME, row.to_string()],
    )?;
    cx.execute(
        "INSERT INTO bundle_key VALUES (?1,0,?2,0)",
        params![NAME, ID],
    )?;
    cx.execute("INSERT INTO bundle_words VALUES (?1,0,'synthetic')", [NAME])?;
    cx.execute(
        "INSERT INTO bundle_content VALUES (?1,?2,?3)",
        params![
            NAME,
            format!(
                "sha256:{}",
                label.repeat(64).chars().take(64).collect::<String>()
            ),
            retrieved
        ],
    )?;
    Ok(())
}
fn read(env: &Environment, extra: &[&str], exit: i32) -> Check<Value> {
    let mut args = vec!["get", "diagnostic", ID, "--source", "who-ivd", "--offline"];
    args.extend_from_slice(extra);
    Ok(invoke(
        env,
        args.into_iter().map(str::to_owned).collect(),
        exit,
    )?)
}
fn assert_label(value: &Value, expected: &str) -> Check {
    if value.pointer("/result/native_rows/0/product_name") != Some(&json!(expected)) {
        return Err(format!("missing {expected}: {value}").into());
    }
    Ok(())
}
fn count(path: &Path, table: &str) -> Check<u32> {
    Ok(Connection::open(path)?.query_row(
        &format!(
            "SELECT count(*) FROM {table} WHERE {}=?1",
            if table == "bundle" || table == "bundle_content" {
                "name"
            } else {
                "bundle"
            }
        ),
        [NAME],
        |r| r.get(0),
    )?)
}
fn imported(old: &Path, current: &Path) -> Check {
    for table in [
        "bundle",
        "bundle_row",
        "bundle_key",
        "bundle_words",
        "bundle_content",
    ] {
        if count(old, table)? != 1 || count(current, table)? != 1 {
            return Err(format!("transfer lost {table}").into());
        }
    }
    let metadata = |path: &Path| -> Check<(String, i64)> {
        Ok(Connection::open(path)?.query_row(
            "SELECT revision,retrieved_at FROM bundle_content WHERE name=?1",
            [NAME],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?)
    };
    if metadata(old)? != metadata(current)? {
        return Err("transfer changed content revision".into());
    }
    Ok(())
}
fn imports() -> Check {
    let env = Environment::new()?;
    let (old, current) = files(&env)?;
    seed(&old, "a", 123)?;
    let before = fs::read(&old)?;
    invoke(
        &env,
        vec!["store".into(), "clean".into(), "--offline".into()],
        0,
    )?;
    if !old.exists() || count(&current, "bundle")? != 0 {
        return Err("cleanup changed the retained generations".into());
    }
    let value = read(&env, &[], 0)?;
    assert_label(&value, "a")?;
    imported(&old, &current)?;
    if fs::read(&old)? != before {
        return Err("transfer changed its source file".into());
    }
    Ok(())
}
fn current() -> Check {
    let env = Environment::new()?;
    let (old, current) = files(&env)?;
    seed(&old, "a", i64::MAX)?;
    seed(&current, "b", 1)?;
    // An unused predecessor must not control a current published bundle.
    Connection::open(&old)?
        .execute("UPDATE meta SET version=999 WHERE owner='biomcp-query'", [])?;
    let before = fs::read(&old)?;
    let value = read(&env, &[], 0)?;
    assert_label(&value, "b")?;
    let row: String =
        Connection::open(&current)?.query_row("SELECT row FROM bundle_row", [], |r| r.get(0))?;
    if !row.contains("\"Product name\":\"b\"") || fs::read(&old)? != before {
        return Err("older bundle overrode the current publication".into());
    }
    Ok(())
}
fn interrupted() -> Check {
    let env = Environment::new()?;
    let (old, current) = files(&env)?;
    seed(&old, "a", 123)?;
    let before = fs::read(&old)?;
    // A real SQLite write failure interrupts the final transaction deterministically.
    Connection::open(&current)?.execute_batch("CREATE TRIGGER interrupt_transfer BEFORE INSERT ON bundle_row BEGIN SELECT RAISE(ABORT,'interrupted'); END;")?;
    let value = read(&env, &[], 5)?;
    if value.get("local_error") != Some(&json!("database"))
        || count(&current, "bundle")? != 0
        || fs::read(&old)? != before
    {
        return Err(format!("interrupted transfer changed a published copy: {value}").into());
    }
    Connection::open(&current)?.execute_batch("DROP TRIGGER interrupt_transfer")?;
    read(&env, &[], 0)?;
    imported(&old, &current)
}
fn ownership() -> Check {
    let env = Environment::new()?;
    let (old, current) = files(&env)?;
    seed(&old, "a", 123)?;
    Connection::open(&old)?
        .execute("UPDATE meta SET version=999 WHERE owner='biomcp-query'", [])?;
    let before = fs::read(&old)?;
    let value = read(&env, &[], 5)?;
    if value.get("local_error") != Some(&json!("version"))
        || count(&current, "bundle")? != 0
        || fs::read(&old)? != before
    {
        return Err(format!("foreign owner was changed or imported: {value}").into());
    }
    Ok(())
}
fn admission() -> Check {
    let env = Environment::new()?;
    let (old, current) = files(&env)?;
    seed(&old, "a", 123)?;
    let before = fs::read(&old)?;
    invoke(&env, vec!["health".into(), "--offline".into()], 0)?;
    read(&env, &["--disable-sources", "who-ivd"], 4)?;
    if count(&current, "bundle")? != 0 || fs::read(&old)? != before {
        return Err("health or disabled source transferred data".into());
    }
    Ok(())
}

fn obsolete() -> Check {
    let env = Environment::new()?;
    let (_, current) = files(&env)?;
    let obsolete = current.with_file_name("cache-1-3.sqlite");
    drop(biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
        cache: Some(obsolete.clone()),
        mode: biomcp_fetch::Mode::Offline,
        ..Default::default()
    })?);
    let empty = fs::read(&obsolete)?;
    invoke(
        &env,
        vec!["store".into(), "clean".into(), "--offline".into()],
        0,
    )?;
    if !obsolete.exists() || fs::read(&obsolete)? != empty {
        return Err(
            "cleanup removed an empty bundle table that can still receive publication".into(),
        );
    }
    seed(&obsolete, "a", 123)?;
    let before = fs::read(&obsolete)?;
    invoke(
        &env,
        vec!["store".into(), "clean".into(), "--offline".into()],
        0,
    )?;
    if !obsolete.exists() || fs::read(&obsolete)? != before {
        return Err("cleanup removed an unsupported file's only published bundle".into());
    }
    Ok(())
}

fn recorded() -> Check {
    let env = Environment::new()?;
    let (old, current) = files(&env)?;
    seed(&old, "a", 123)?;
    let before = fs::read(&old)?;
    let stub = super::network_stub::Stub::new()?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    fs::write(&ca, &stub.certificate)?;
    env.config(&json!({"ca_bundle":ca,"proxy":format!("http://{}",stub.address)}))?;
    let records = env.root().join("records");
    let value = invoke(
        &env,
        vec![
            "get".into(),
            "diagnostic".into(),
            ID.into(),
            "--source".into(),
            "who-ivd".into(),
            "--record".into(),
            records.to_string_lossy().into(),
        ],
        0,
    )?;
    assert_label(&value, "a")?;
    imported(&old, &current)?;
    if fs::read(&old)? != before || !stub.finish()?.is_empty() {
        return Err("recorded stored read changed its source or contacted a provider".into());
    }
    Ok(())
}
