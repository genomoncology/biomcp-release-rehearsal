use super::{Check, both, environment, words};
use crate::runner::{configuration::invoke, network_stub::Stub};
use serde_json::{Value, json};
pub(super) fn run() -> Check {
    let stub = Stub::for_source("wonder.cdc.gov", "application/xml", |request| {
        if !request
            .to_ascii_lowercase()
            .contains("user-agent: wget/1.21.4")
            || !request.starts_with("POST /controller/datarequest/D8 ")
            || !request.contains("accept_datause_restrictions=true")
        {
            return Err("WONDER compatibility headers or restrictions missing".into());
        }
        let vaccine = if request.contains("COVID19") {
            "COVID19"
        } else {
            "MMR"
        };
        let group = if request.contains("%3Cvalue%3ED8.V10%3C") {
            "D8.V10"
        } else if request.contains("%3Cvalue%3ED8.V1%3C") {
            "D8.V1"
        } else {
            "D8.V13-level2"
        };
        Ok(super::fixture::table(vaccine, group))
    })?;
    let env = environment(&stub)?;
    let mut args = words("MMR");
    args.push("symptoms".into());
    args.extend(["--limit".into(), "2".into()]);
    let first = both(&env, args.clone(), 0)?;
    if first.pointer("/result/scope/variable/code") != Some(&json!("D8.V12"))
        || first.pointer("/result/scope/value/code") != Some(&json!("00"))
        || first.pointer("/result/scope/value/label")
            != Some(&json!("The United States/Territories/Unknown"))
    {
        return Err(format!("native geographic scope omitted: {first}"));
    }
    if first.pointer("/result/report_total") != Some(&json!("5"))
        || first.pointer("/result/footer/reports/dt") != Some(&json!("9"))
        || first.pointer("/result/footer/percent/dt") != Some(&json!("180.00%"))
        || first.pointer("/page/total") != Some(&json!(3))
        || first.pointer("/result/rows/1/label/code") != Some(&json!("synthetic-code"))
    {
        return Err(format!("native totals, labels or codes changed: {first}"));
    }
    let page = next(&first, "page")?;
    let second = both(&env, page.clone(), 0)?;
    if second.pointer("/query/offset") != Some(&json!(2))
        || second.pointer("/result/rows/0/reports/v") != Some(&json!("Suppressed"))
        || second.pointer("/result/rows/0/reports/suppression") != Some(&json!("synthetic"))
        || second
            .get("next")
            .and_then(Value::as_array)
            .is_some_and(|v| v.iter().any(|s| s.get("kind") == Some(&json!("page"))))
    {
        return Err("retained page lost suppression or terminal state".into());
    }
    for (section, total) in [("seriousness", "7"), ("age", "11")] {
        let mut args = words("MMR");
        args.push(section.into());
        let value = both(&env, args, 0)?;
        if value.pointer("/result/report_total") != Some(&json!(total)) {
            return Err("section borrowed another total".into());
        }
    }
    let mut alias = words("mMr");
    alias.extend(["--limit".into(), "2".into()]);
    invoke(&env, alias, 0)?;
    let mut absent = page;
    let position = absent
        .iter()
        .position(|s| s == "--snapshot")
        .ok_or("snapshot selector")?;
    *absent.get_mut(position + 1).ok_or("snapshot value")? = "missing-snapshot".into();
    let expired = both(&env, absent, 3)?;
    if expired.pointer("/error/code") != Some(&json!("pool.expired")) {
        return Err("missing snapshot did not expire".into());
    }
    if stub.finish()?.len() != 3 {
        return Err("paging or alias fetched instead of retaining native table".into());
    }
    Ok(())
}
fn next(value: &Value, kind: &str) -> Result<Vec<String>, String> {
    let args = value
        .get("next")
        .and_then(Value::as_array)
        .and_then(|v| v.iter().find(|s| s.get("kind") == Some(&json!(kind))))
        .and_then(|s| s.get("args"))
        .ok_or("native table continuation")?;
    serde_json::from_value(args.clone()).map_err(|e| e.to_string())
}
