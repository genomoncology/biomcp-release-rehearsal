use super::{Stub, Value, call, json, setup};
pub(super) fn run() -> Result<(), String> {
    let stub = Stub::for_source("www.ebi.ac.uk", "application/json", reply)?;
    let env = setup(&stub)?;
    let first = call(&env, &["protein", "domains", "P00533"], 0)?;
    super::super::protein_budgets::check(&first, "synthetic-domain-walk/first", "cli")?;
    let pool = first.pointer("/page/pool").cloned().ok_or("domain pool")?;
    if first.pointer("/page/total") != Some(&json!(26)) {
        return Err("source next pointer lost".into());
    }
    let mismatch = call(
        &env,
        &[
            "protein",
            "domains",
            "P00533",
            "--type",
            "domain",
            "--pool",
            pool.get("id").and_then(Value::as_str).ok_or("pool id")?,
        ],
        2,
    )?;
    super::super::protein_budgets::check(
        &mismatch,
        "synthetic-domain-walk/mismatched-type",
        "cli",
    )?;
    if mismatch.pointer("/error/code") != Some(&json!("usage.pool_mismatch")) {
        return Err("source pool ignored type selection".into());
    }
    let mut value = first;
    let mut ids = Vec::new();
    loop {
        ids.extend(
            value
                .pointer("/result/items")
                .and_then(Value::as_array)
                .ok_or("domain rows")?
                .iter()
                .map(|r| r.get("id").cloned().unwrap_or(Value::Null)),
        );
        if ids.len() == 26 {
            break;
        }
        let next = super::super::action(&value, "page")?;
        let words: Vec<_> = next.iter().map(String::as_str).collect();
        value = call(&env, &words, 0)?;
        super::super::protein_budgets::check(
            &value,
            &format!(
                "synthetic-domain-walk/offset-{}",
                value
                    .pointer("/page/offset")
                    .and_then(Value::as_u64)
                    .ok_or("offset")?
            ),
            "cli",
        )?;
        if value.pointer("/page/pool") != Some(&pool) {
            return Err("domain continuation renewed source snapshot".into());
        }
    }
    if ids.first() != Some(&json!("IPR000001")) || ids.last() != Some(&json!("IPR000026")) {
        return Err("retained domain order changed".into());
    }
    if ids
        .iter()
        .map(Value::to_string)
        .collect::<std::collections::BTreeSet<_>>()
        .len()
        != 26
    {
        return Err("domain walk lost or duplicated matches".into());
    }
    if stub.finish()?.len() != 2 {
        return Err("domain retained paging refetched a source page".into());
    }
    Ok(())
}
fn reply(request: &str) -> Result<String, String> {
    let (rows, next) = if request.contains("cursor=") {
        if !request.contains("cursor=native%7C2")
            || !request.contains("page=1")
            || !request.contains("page_size=25")
        {
            return Err("native cursor selectors changed".into());
        }
        (vec![row(26)], None)
    } else {
        (
            (1..26).map(row).collect(),
            Some(
                "https://www.ebi.ac.uk/interpro/api/entry/interpro/protein/uniprot/P00533/?cursor=native%7C2&page=1&page_size=25",
            ),
        )
    };
    Ok(json!({"count":26,"next":next,"results":rows}).to_string())
}
fn row(n: u32) -> Value {
    json!(
        {"metadata":{"accession":format!("IPR{n:06}"),"name":"Native match","source_database":"interpro","type":"domain",
        "member_databases":{}},"proteins":[{"accession":"p00533","protein_length":1210,"organism":"9606","entry_protein_locations":[]}]}
    )
}
