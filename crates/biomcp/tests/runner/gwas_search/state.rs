//! Technical controls retain source semantics rather than inventing clinical negatives.
use super::{Environment, Value, body, call, fs, gwas_budgets, json, pairs, seed, url};

pub(super) fn expired() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let args = [
        "search",
        "gwas",
        "--pubmed",
        "17529967",
        "--p-value",
        "1e-200",
    ];
    let replay = seed(&env, &args, body(&url(&pairs()), 0, 6), 200, 0)?;
    let first = call("gwas_search/state/expired/1", &env, &replay, &args, 0)?;
    let page = super::super::action(&first, "page")?;
    fs::remove_dir_all(replay.join("gwas")).map_err(|e| e.to_string())?;
    for source in ["gwas", "restricted"] {
        let mut command = env.globals(Some(&replay), false);
        command.extend(page.clone());
        command.extend(["--disable-sources".into(), source.into()]);
        let value = gwas_budgets::invoke("gwas_search/state/expired/2", &env, command, 4)?;
        if value.pointer("/sources/0/reason") != Some(&json!("disabled"))
            || value.get("result").is_some()
        {
            return Err(format!("disabled source restored threshold pool: {value}"));
        }
        bounded(&value)?;
    }
    let db = rusqlite::Connection::open(env.root().join("case-cache/replay.sqlite"))
        .map_err(|e| e.to_string())?;
    db.execute("UPDATE pool SET expires=0", [])
        .map_err(|e| e.to_string())?;
    drop(db);
    let mut command = env.globals(Some(&replay), false);
    command.extend(page);
    let value = gwas_budgets::invoke("gwas_search/state/expired/3", &env, command, 3)?;
    let restart = super::super::action(&value, "restart")?;
    if value.pointer("/error/code") != Some(&json!("pool.expired"))
        || !args.iter().all(|arg| restart.contains(&(*arg).to_owned()))
        || restart.contains(&"--pool".to_owned())
    {
        return Err(format!("expired pool lost selectors or refetched: {value}"));
    }
    bounded(&value)
}

fn bounded(value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    if bytes > crate::runner::output_limit("gwas-state-bytes")?
        || bytes.div_ceil(4) > crate::runner::output_limit("gwas-state-tokens")?
    {
        return Err("synthetic state exceeds ceiling".into());
    }
    Ok(())
}
