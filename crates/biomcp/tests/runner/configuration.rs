//! Process settings and persistent case-cache ownership through the shipped binary.
use super::launch::Environment;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn root() -> PathBuf {
    crate::runner::launch::fixtures().join("replies/gene/main")
}
pub(super) fn invoke(env: &Environment, args: Vec<String>, exit: i32) -> Result<Value, String> {
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.code() != Some(exit) || !output.stderr.is_empty() {
        return Err(format!(
            "expected exit {exit}: {:?}; stderr: {}; stdout: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(2000)
                .collect::<String>()
        ));
    }
    let value: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if let Some(kind) = value.get("local_error") {
        use std::io::Write;
        writeln!(
            std::io::stdout(),
            "LOCAL_ERROR_CLI_BYTES {kind} {}",
            output.stdout.len()
        )
        .map_err(|e| e.to_string())?;
    }
    if value.get("schema").and_then(Value::as_str)
        != Some(<biomcp_types::Help as biomcp_types::ResponseBody>::SCHEMA)
        && (args.iter().any(|word| word == "diagnostic")
            || args.windows(2).any(|words| {
                words.first().is_some_and(|word| word == "sync")
                    && words
                        .get(1)
                        .is_some_and(|word| matches!(word.as_str(), "gtr" | "who-ivd" | "fda-cdx"))
            }))
    {
        super::diagnostic_retirements::bounded(&value)?;
    }
    Ok(value)
}
pub(super) fn cached(args: &[&str], case: &Path, recording: Option<&Path>) -> Vec<String> {
    let mut args: Vec<String> = args.iter().map(|arg| (*arg).into()).collect();
    if let Some(recording) = recording {
        args.extend(["--replay".into(), recording.to_string_lossy().into()]);
    } else {
        args.push("--offline".into());
    }
    args.extend(["--case-cache".into(), case.to_string_lossy().into()]);
    args
}

pub(super) fn trials() -> Vec<Trial> {
    // These inspect files and SQLite ownership before mutation; JSON replies cannot show those side effects.
    vec![
        transcript_trial(
            "gene-identity/persistent-case-cache-owns-all-writes",
            || ownership().map_err(Into::into),
        ),
        transcript_trial("gene-identity/refuses-unowned-case-cache-paths", || {
            refusals().map_err(Into::into)
        }),
        transcript_trial(
            "article-search/owners-refused-before-journal-and-tables",
            || article_owners().map_err(Into::into),
        ),
    ]
}

fn ownership() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let user_cache = env.root().join("cache/biomcp");
    fs::create_dir(&user_cache).map_err(|e| e.to_string())?;
    let file = user_cache.join("replies.sqlite");
    fs::write(&file, b"prepopulated user cache sentinel").map_err(|e| e.to_string())?;
    let case = env.root().join("case");
    let records = env.root().join("known-recordings");
    super::response_cases::cache_capture(
        "gene/main",
        &records,
        super::response_cases::CACHE_ACQUIRED,
    )?;
    let replay = invoke(
        &env,
        cached(&["get", "gene", "BRAF"], &case, Some(&records)),
        0,
    )?;
    let offline = invoke(&env, cached(&["get", "gene", "BRAF"], &case, None), 0)?;
    if replay != offline {
        return Err("offline lost the checked replay reply".into());
    }
    let missing = env.root().join("missing");
    fs::create_dir(&missing).map_err(|e| e.to_string())?;
    let failed = invoke(
        &env,
        cached(&["get", "gene", "BRAF"], &case, Some(&missing)),
        5,
    )?;
    if failed.get("status") != Some(&json!("failed"))
        || failed.pointer("/result/items/0/answer").is_some()
    {
        return Err("warm rows masked a missing recording".into());
    }
    if fs::read(&file).map_err(|e| e.to_string())? != b"prepopulated user cache sentinel" {
        return Err("replay wrote the user cache".into());
    }
    let sync_case = env.root().join("sync-case");
    let value = invoke(
        &env,
        cached(&["sync", "hgnc"], &sync_case, Some(&root())),
        0,
    )?;
    if value.get("schema") != Some(&json!("biomcp.sync/7"))
        || value.pointer("/result/rows") != Some(&json!(176))
        || value.pointer("/query/source") != Some(&json!("hgnc"))
    {
        return Err("source sync lost metadata".into());
    }
    let withdrawn = invoke(&env, cached(&["get", "gene", "BRCA3"], &sync_case, None), 0)?;
    if withdrawn.pointer("/result/items/0/answer/resolution") != Some(&json!("withdrawn")) {
        return Err("HGNC sync omitted withdrawn identity".into());
    }
    Ok(())
}

fn refusals() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let case = env.root().join("owned");
    invoke(
        &env,
        cached(&["get", "gene", "MLL2"], &case, Some(&root())),
        0,
    )?;
    for (name, marker, bytes) in [
        ("unmarked", None, b"foreign database".as_slice()),
        (
            "foreign",
            Some(b"another owner".as_slice()),
            b"foreign database".as_slice(),
        ),
        (
            "bad-database",
            Some(b"biomcp.replay-case/1\n".as_slice()),
            b"foreign database".as_slice(),
        ),
    ] {
        let path = env.root().join(name);
        fs::create_dir(&path).map_err(|e| e.to_string())?;
        if let Some(marker) = marker {
            fs::write(path.join(".biomcp-case"), marker).map_err(|e| e.to_string())?;
        }
        fs::write(path.join("replay.sqlite"), bytes).map_err(|e| e.to_string())?;
        invoke(&env, cached(&["get", "gene", "BRAF"], &path, None), 5)?;
        if fs::read(path.join("replay.sqlite")).map_err(|e| e.to_string())? != bytes {
            return Err("refusal changed foreign files".into());
        }
    }
    #[cfg(unix)]
    {
        let link = env.root().join("linked-root");
        std::os::unix::fs::symlink(&case, &link).map_err(|e| e.to_string())?;
        invoke(&env, cached(&["get", "gene", "BRAF"], &link, None), 5)?;
        let path = env.root().join("linked-file");
        fs::create_dir(&path).map_err(|e| e.to_string())?;
        fs::copy(case.join(".biomcp-case"), path.join(".biomcp-case"))
            .map_err(|e| e.to_string())?;
        std::os::unix::fs::symlink(case.join("replay.sqlite"), path.join("replay.sqlite"))
            .map_err(|e| e.to_string())?;
        invoke(&env, cached(&["get", "gene", "BRAF"], &path, None), 5)?;
    }
    Ok(())
}

fn article_owners() -> Result<(), String> {
    for owned in [false, true] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        let parent = env.root().join("cache/biomcp");
        fs::create_dir_all(&parent).map_err(|e| e.to_string())?;
        let file = parent.join(format!(
            "cache-{}-{}.sqlite",
            biomcp_fetch::CACHE_NAMESPACE_VERSION,
            biomcp_query::store::CACHE_NAMESPACE_VERSION
        ));
        let case = env.root().join("case");
        let replay = if owned {
            Some(
                biomcp_fetch::ReplayCache::create_case_with_owners(&case, &[])
                    .map_err(|e| e.to_string())?,
            )
        } else {
            None
        };
        let fetch = biomcp_fetch::Fetch::open(biomcp_fetch::FetchConfig {
            cache: Some(file),
            mode: biomcp_fetch::Mode::Offline,
            replay_cache: replay,
            ..Default::default()
        })
        .map_err(|e| e.to_string())?;
        let connection = fetch
            .cache_access()
            .map_err(|e| e.to_string())?
            .connect()
            .map_err(|e| e.to_string())?;
        drop(fetch);
        connection
            .execute(
                "INSERT INTO meta(owner,version) VALUES ('biomcp-query',?1)",
                [biomcp_query::store::OWNER.version + 1],
            )
            .map_err(|e| e.to_string())?;
        connection
            .pragma_update(None, "journal_mode", "DELETE")
            .map_err(|e| e.to_string())?;
        let mut args = vec![
            "search".into(),
            "article".into(),
            "AKT1 E17K".into(),
            "--offline".into(),
        ];
        if owned {
            args.extend(["--case-cache".into(), case.to_string_lossy().into()]);
        }
        let result = invoke(&env, args, 5)?;
        if result.pointer("/error/code") != Some(&json!("local.failure"))
            || result.get("local_error") != Some(&json!("version"))
        {
            return Err("wrong owner was not a local configuration error".into());
        }
        let journal = connection
            .query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let pool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='pool')",
                [],
                |r| r.get::<_, bool>(0),
            )
            .map_err(|e| e.to_string())?;
        if journal != "delete" || pool {
            return Err("wrong query owner allowed journal or table mutation".into());
        }
    }
    Ok(())
}
