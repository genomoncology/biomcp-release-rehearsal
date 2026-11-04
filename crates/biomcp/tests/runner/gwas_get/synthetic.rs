use super::{
    BASE, Environment, Stub, Value, ancestry_body, call, fs, gwas_budgets, json, seed, source,
};

pub(super) fn ancestry() -> Result<(), String> {
    let args = [
        "get",
        "gwas",
        "GCST000035",
        "ancestries",
        "--section-limit",
        "1",
    ];
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = seed(&env, &args, ancestry_body().to_string(), 200, 0)?;
    let mut value = call("gwas_get/synthetic/ancestry/1", &env, &replay, &args, 0)?;
    fs::remove_dir_all(replay.join("gwas")).map_err(|e| e.to_string())?;
    let mut counts = vec![
        value
            .pointer("/result/items/0/number_of_individuals")
            .cloned()
            .ok_or("first row")?,
    ];
    for _ in 0..2 {
        let args = super::super::action(&value, "page")?;
        let mut command = env.globals(Some(&replay), false);
        command.extend(args);
        value = gwas_budgets::invoke("gwas_get/synthetic/ancestry/2", &env, command, 0)?;
        counts.push(
            value
                .pointer("/result/items/0/number_of_individuals")
                .cloned()
                .ok_or("next row")?,
        );
    }
    if counts != vec![json!(10), json!(11), json!(12)] {
        return Err(format!("ancestry walk: {counts:?}"));
    }
    let mut disabled = env.globals(Some(&replay), false);
    disabled.extend(
        [
            "get",
            "gwas",
            "GCST000035",
            "ancestries",
            "--disable-sources",
            "restricted",
        ]
        .map(str::to_owned),
    );
    let value = gwas_budgets::invoke("gwas_get/synthetic/ancestry/3", &env, disabled, 4)?;
    if value.pointer("/sources/0/reason") != Some(&json!("disabled")) {
        return Err("cached disabled ancestry".into());
    }
    let env = Environment::new().map_err(|e| e.to_string())?;
    let mut rows = ancestry_body();
    *rows
        .pointer_mut("/_embedded/ancestries/1/_links/self/href")
        .ok_or("row link")? = json!(format!("{BASE}/studies/GCST000037/ancestries/1"));
    let replay = seed(&env, &args, rows.to_string(), 200, 0)?;
    let value = call(
        "gwas_get/synthetic/ancestry/4",
        &env,
        &replay,
        &["get", "gwas", "GCST000035", "ancestries"],
        0,
    )?;
    if value.pointer("/status") != Some(&json!("partial"))
        || value.pointer("/page/total") != Some(&json!(2))
    {
        return Err(format!("foreign ancestry admitted: {value}"));
    }
    Ok(())
}
pub(super) fn contexts() -> Result<(), String> {
    let stub = Stub::for_source("www.ebi.ac.uk", "application/json", source)?;
    let live = super::super::variant::source_environment(&stub)?;
    let value = gwas_budgets::invoke(
        "gwas_get/synthetic/contexts/1",
        &live,
        vec![
            "get".into(),
            "gwas".into(),
            "association:6114".into(),
            "loci".into(),
        ],
        0,
    )?;
    for id in ["ENSG00000066468", "2263"] {
        if !value
            .get("next")
            .and_then(Value::as_array)
            .is_some_and(|s| s.iter().any(|s| s.pointer("/args/2") == Some(&json!(id))))
        {
            return Err(format!("missing exact gene {id}: {value}"));
        }
    }
    if stub.finish()?.len() != 1 {
        return Err("loci performed a gene request".into());
    }
    let stub = Stub::for_source("www.ebi.ac.uk", "application/json", source)?;
    let live = super::super::variant::source_environment(&stub)?;
    let first = gwas_budgets::invoke(
        "gwas_get/synthetic/contexts/2",
        &live,
        ["get", "gwas", "GCST000035", "associations"]
            .map(str::to_owned)
            .into(),
        0,
    )?;
    let next = super::super::action(&first, "page")?;
    let last = gwas_budgets::invoke("gwas_get/synthetic/contexts/3", &live, next, 0)?;
    if first.pointer("/result/items/0/id") != Some(&json!("association:6110"))
        || last.pointer("/result/items/0/id") != Some(&json!("association:6113"))
        || last.pointer("/query/query/id") != Some(&json!("GCST000035"))
        || last.pointer("/page/total") != Some(&json!(6))
    {
        return Err(format!("study source walk: {first} {last}"));
    }
    if stub.finish()?.len() != 2 {
        return Err("study walk request count".into());
    }
    Ok(())
}
