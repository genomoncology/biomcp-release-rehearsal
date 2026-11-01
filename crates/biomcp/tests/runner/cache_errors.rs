//! Cache failures retain their category through the real command boundary.
use super::{launch::Environment, response_cases::copy_capture, trial_get::command};
use biomcp_fetch::{CacheError, Fetch, FetchConfig, Mode};
use serde_json::json;
type Check = Result<(), Box<dyn std::error::Error>>;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    [
        ("encoding", CacheError::Encoding),
        ("filesystem", CacheError::Filesystem),
        ("internal", CacheError::Internal),
        ("limit", CacheError::Limit),
    ]
    .into_iter()
    .map(|(kind, error)| {
        super::transcript::trial(
            format!("cache-resilience/{kind}-failure-keeps-kind"),
            move || failure(kind, error).map_err(|e| e.to_string().into()),
        )
    })
    .collect()
}
fn failure(kind: &str, error: CacheError) -> Check {
    if error.to_string() != format!("fetch.cache_{kind}") {
        return Err("cache failure lost its dotted code".into());
    }
    let env = Environment::new()?;
    let replay = env.root().join("recordings");
    let args = if matches!(kind, "encoding" | "limit") {
        copy_capture("article/search/synthetic-small", &replay, |_, _| {})?;
        ["search", "article", "AKT1 E17K", "--limit", "2"]
            .map(str::to_owned)
            .to_vec()
    } else {
        copy_capture("trial/get/NCT03734029", &replay, |_, _| {})?;
        ["get", "trial", "NCT03734029", "locations"]
            .map(str::to_owned)
            .to_vec()
    };
    let first = command(&env, &args, &env.globals(Some(&replay), false), 0)?;
    let path = env.root().join("case-cache/replay.sqlite");
    let fetch = Fetch::open(FetchConfig {
        cache: Some(path.clone()),
        cache_owners: vec![biomcp_query::store::OWNER],
        mode: Mode::Offline,
        ..Default::default()
    })?;
    let db = fetch.cache_access()?.connect()?;
    let mut globals = env.globals(None, true);
    let args = match kind {
        "encoding" => {
            if db.execute("UPDATE pool SET outcomes='invalid JSON'", [])? != 1 {
                return Err("expected one article pool".into());
            }
            args
        }
        "limit" => {
            if db.execute("UPDATE pool SET created=-1", [])? != 1 {
                return Err("expected one article pool".into());
            }
            args
        }
        "internal" => {
            db.execute(
                "UPDATE trial_snapshot SET payload=replace(payload,'NCT03734029','NCT00000001')",
                [],
            )?;
            super::action(&first, "page")?
        }
        "filesystem" => {
            globals = vec![
                "--replay".into(),
                replay.to_string_lossy().into(),
                "--case-cache".into(),
                env.root()
                    .join("missing-parent/case")
                    .to_string_lossy()
                    .into(),
            ];
            args
        }
        _ => return Err("unknown failure kind".into()),
    };
    let broken = command(&env, &args, &globals, 5)?;
    if broken.pointer("/local_error") != Some(&json!(kind)) {
        return Err(format!("cache failure lost {kind}: {broken}").into());
    }
    Ok(())
}
