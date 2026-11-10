//! An exact threshold cannot count associations it could not compare.
use super::{Environment, Value, body, json, pairs, seed, url};

pub(super) fn run() -> Result<(), String> {
    for mode in ["missing", "invalid", "refused", "all-missing"] {
        let env = Environment::new().map_err(|error| error.to_string())?;
        let args = [
            "search",
            "gwas",
            "--pubmed",
            "17529967",
            "--p-value",
            "1e-200",
        ];
        let mut page = body(&url(&pairs()), 0, 3);
        let rows = page
            .pointer_mut("/_embedded/associations")
            .and_then(Value::as_array_mut)
            .ok_or("association fixture")?;
        for (index, row) in rows.iter_mut().enumerate() {
            if mode == "all-missing" || index == 1 {
                let row = row.as_object_mut().ok_or("association object")?;
                match mode {
                    "refused" => {
                        row.insert("association_id".into(), json!("invalid"));
                    }
                    "invalid" => {
                        row.insert("pvalue_mantissa".into(), json!(0));
                    }
                    _ => {
                        row.remove("pvalue_mantissa");
                        row.remove("pvalue_exponent");
                    }
                }
            }
        }
        let replay = seed(&env, &args, page, 200, 0)?;
        let value = replay_call(&env, &replay, &args)?;
        check(&value, mode, if mode == "all-missing" { 0 } else { 2 })?;
        let pool = value
            .pointer("/page/pool/id")
            .and_then(Value::as_str)
            .ok_or("retained pool")?;
        if mode == "missing" {
            let db = rusqlite::Connection::open(env.root().join("case-cache/replay.sqlite"))
                .map_err(|error| error.to_string())?;
            db.execute("UPDATE trial_snapshot SET payload=json_set(payload,'$.dropped',json('false')) WHERE id=?1", [pool])
                .map_err(|error| error.to_string())?;
        }
        let mut explicit = args.to_vec();
        explicit.extend(["--pool", pool]);
        check(
            &replay_call(&env, &replay, &explicit)?,
            mode,
            if mode == "all-missing" { 0 } else { 2 },
        )?;
    }
    later_page()?;
    complete_controls()
}

fn complete_controls() -> Result<(), String> {
    for (threshold, exponent, expected) in [
        ("4e-254", -254, 3),
        ("1e-300", -254, 0),
        ("4e-320", -320, 3),
    ] {
        let env = Environment::new().map_err(|error| error.to_string())?;
        let args = [
            "search",
            "gwas",
            "--pubmed",
            "17529967",
            "--p-value",
            threshold,
        ];
        let mut page = body(&url(&pairs()), 0, 3);
        for row in page
            .pointer_mut("/_embedded/associations")
            .and_then(Value::as_array_mut)
            .ok_or("association fixture")?
        {
            let row = row.as_object_mut().ok_or("association object")?;
            row.insert("pvalue_exponent".into(), json!(exponent));
            if exponent == -320 {
                row.insert("p_value".into(), json!(0.0));
            }
        }
        let replay = seed(&env, &args, page, 200, 0)?;
        let value = replay_call(&env, &replay, &args)?;
        if value.pointer("/status") != Some(&json!("complete"))
            || value.pointer("/result/qualifying_total") != Some(&json!(expected))
            || value.pointer("/page/returned") != Some(&json!(expected))
        {
            return Err(format!(
                "{threshold}: comparable terminal count changed: {value}"
            ));
        }
    }
    Ok(())
}

fn later_page() -> Result<(), String> {
    let stub = crate::runner::network_stub::Stub::for_source(
        "www.ebi.ac.uk",
        "application/json",
        |request| {
            let mut value: Value = serde_json::from_str(&super::wire::live_body(request)?)
                .map_err(|error| error.to_string())?;
            if value.pointer("/page/number") == Some(&json!(0)) {
                let row = value
                    .pointer_mut("/_embedded/associations/1")
                    .and_then(Value::as_object_mut)
                    .ok_or("first page row")?;
                row.remove("pvalue_mantissa");
                row.remove("pvalue_exponent");
            }
            Ok(value.to_string())
        },
    )?;
    let env = crate::runner::variant::source_environment(&stub)?;
    let first = super::invoke(
        &env,
        [
            "search",
            "gwas",
            "--pubmed",
            "17529967",
            "--p-value",
            "1e-200",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        0,
    )?;
    check(&first, "first page", 2)?;
    let next = crate::runner::action(&first, "page")?;
    let last = super::invoke(&env, next, 0)?;
    check(&last, "clean terminal page", 3)?;
    if last.pointer("/page/truncated") != Some(&json!(false)) || stub.finish()?.len() != 2 {
        return Err(format!("uncertain walk did not terminate: {last}"));
    }
    Ok(())
}

fn check(value: &Value, mode: &str, expected: usize) -> Result<(), String> {
    if serde_json::to_vec(value)
        .map_err(|error| error.to_string())?
        .len()
        > crate::runner::output_limit("gwas-state-bytes")?
    {
        return Err("uncertain threshold exceeds output ceiling".into());
    }
    let count = value
        .pointer("/result/items")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    if value.pointer("/status") != Some(&json!("partial"))
        || value.pointer("/result/qualifying_total").is_some()
        || count != expected
        || value
            .pointer("/problems")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
    {
        return Err(format!("{mode}: unknown membership claimed exact: {value}"));
    }
    Ok(())
}

fn replay_call(
    env: &Environment,
    replay: &std::path::Path,
    args: &[&str],
) -> Result<Value, String> {
    let mut command = env.globals(Some(replay), false);
    command.extend(args.iter().map(|arg| (*arg).to_owned()));
    super::invoke(env, command, 0)
}

pub(super) fn pools() -> Result<(), String> {
    let env = Environment::new().map_err(|error| error.to_string())?;
    let args = [
        "search",
        "gwas",
        "--pubmed",
        "17529967",
        "--p-value",
        "1e-200",
    ];
    let mut page = body(&url(&pairs()), 0, 3);
    page.pointer_mut("/_embedded/associations/1")
        .and_then(Value::as_object_mut)
        .ok_or("association object")?
        .remove("pvalue_mantissa");
    let replay = seed(&env, &args, page, 200, 0)?;
    let first = replay_call(&env, &replay, &args)?;
    let repeated = replay_call(&env, &replay, &args)?;
    let id = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("first pool")?;
    if repeated.pointer("/page/pool/id") == first.pointer("/page/pool/id") {
        return Err("new search reused an incomplete observation as current".into());
    }
    let mut explicit = args.to_vec();
    explicit.extend(["--pool", id]);
    let old = replay_call(&env, &replay, &explicit)?;
    if old.pointer("/page/pool/id") != first.pointer("/page/pool/id") {
        return Err("old explicit pool lost its identity".into());
    }
    check(&old, "explicit incomplete pool", 2)
}

pub(super) fn stale_page() -> Result<(), String> {
    let stub = crate::runner::network_stub::Stub::for_source(
        "www.ebi.ac.uk",
        "application/json",
        super::wire::live_body,
    )?;
    let env = crate::runner::variant::source_environment(&stub)?;
    let cache = env.root().join(format!(
        "cache/biomcp/cache-{}-{}.sqlite",
        biomcp_fetch::CACHE_NAMESPACE_VERSION,
        biomcp_query::store::CACHE_NAMESPACE_VERSION
    ));
    let first = super::invoke(
        &env,
        [
            "search",
            "gwas",
            "--pubmed",
            "17529967",
            "--p-value",
            "1e-200",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
        0,
    )?;
    let db = rusqlite::Connection::open(cache).map_err(|error| error.to_string())?;
    db.execute(
        "UPDATE trial_snapshot SET payload=json_set(payload,'$.sources[0].state','stale')",
        [],
    )
    .map_err(|error| error.to_string())?;
    drop(db);
    let last = super::invoke(&env, crate::runner::action(&first, "page")?, 0)?;
    if last.pointer("/status") != Some(&json!("partial"))
        || last.pointer("/sources/0/state") != Some(&json!("stale"))
        || last.pointer("/result/qualifying_total").is_some()
        || stub.finish()?.len() != 2
    {
        return Err(format!("fresh extension erased stale observation: {last}"));
    }
    Ok(())
}
