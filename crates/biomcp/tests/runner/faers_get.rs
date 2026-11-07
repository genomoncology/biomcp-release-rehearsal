//! Feature adverse-event-get: synthetic exact identity and independent native sections.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), String>;
mod optional;
pub(super) fn trials() -> Vec<Trial> {
    // The test binds emitted snapshot IDs into later section requests and checks that those reads never reach the peer.
    vec![
        transcript_trial(crate::runner::required_contract::FAERS_SECTIONS.0, || {
            sections().map_err(Into::into)
        }),
        transcript_trial(
            crate::runner::required_contract::FAERS_IDENTIFIERS.0,
            || identifier_kinds().map_err(Into::into),
        ),
    ]
}
pub(super) fn native() -> Value {
    json!(
        {"meta":{"last_updated":"2024-09-01","results":{"skip":0,"limit":5,"total":1}},"results":[{"safetyreportid":"100",
        "safetyreportversion":"1","receivedate":"20240101","receiptdate":"20240202","serious":"1","seriousnessdeath":"1",
        "seriousnesshospitalization":"1","primarysource":{"qualification":"1","reportercountry":"US"},"patient":{"patientonsetage":"42.5",
        "patientonsetageunit":"801",
        "patientsex":"2","drug":[{"medicinalproduct":"Synthetic A","drugcharacterization":"1","openfda":{"unii":["TESTA"],"rxcui":["1"],
        "product_ndc":["SYNTHETIC"],"spl_id":["SYNTHETIC-ID"],"spl_set_id":["SYNTHETIC-SET"],"application_number":["SYNTHETIC-APPLICATION"],
        "package_ndc":["A","B","C","D","E","F"]}},{"medicinalproduct":"Synthetic B","drugcharacterization":"2","openfda":{"unii":[]}},
        {"medicinalproduct":""}],"reaction":[{"reactionmeddrapt":"Synthetic reaction","reactionmeddraversionpt":"27.0",
        "reactionoutcome":"1"}]}}]}
    )
}
fn setup(mode: &str) -> Result<(Environment, Stub), String> {
    let mut body = native();
    age_input(&mut body, mode)?;
    optional::input(&mut body, mode)?;
    if mode == "walks" {
        let drugs: Vec<_> = (0..19)
            .map(|n| json!({"medicinalproduct":format!("Synthetic drug {n}"), "drugcharacterization":if n%2==0 {"1"}else{"2"}}))
            .collect();
        let reactions: Vec<_> = (0..22)
            .map(|n| json!({"reactionmeddrapt":format!("Synthetic reaction {n}"),"reactionoutcome":"1"}))
            .collect();
        *body.pointer_mut("/results/0/patient/drug").ok_or("drugs")? = json!(drugs);
        *body
            .pointer_mut("/results/0/patient/reaction")
            .ok_or("reactions")? = json!(reactions);
    }
    if mode == "wrong-id" {
        *body
            .pointer_mut("/results/0/safetyreportid")
            .ok_or("report ID")? = json!("101");
    }
    if mode == "two-versions" {
        let mut second = body.pointer("/results/0").ok_or("first report")?.clone();
        *second.get_mut("safetyreportversion").ok_or("version")? = json!("2");
        body.get_mut("results")
            .and_then(Value::as_array_mut)
            .ok_or("results")?
            .push(second);
        *body.pointer_mut("/meta/results/total").ok_or("total")? = json!(2);
    }
    if mode == "absent" {
        body = json!({"error":{"code":"NOT_FOUND","message":"No matches found!"}});
    }
    let status = if mode == "absent" { 404 } else { 200 };
    let stub = Stub::for_source_status(
        "api.fda.gov",
        "application/json",
        move |request| {
            if !request.starts_with(
                "GET /drug/event.json?search=safetyreportid%3A%22100%22&limit=5&skip=0&sort=receivedate%3Adesc \
                HTTP/1.1",
            ) {
                return Err(format!("wrong get: {request}"));
            }
            Ok(body.to_string())
        },
        move |_| status,
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"ca_bundle":ca,"proxy":format!("http://{}",stub.address)}))
        .map_err(|e| e.to_string())?;
    Ok((env, stub))
}
fn args(section: Option<&str>) -> Vec<String> {
    let mut args = vec!["get".into(), "adverse-event".into(), "100".into()];
    if let Some(section) = section {
        args.push(section.into());
    }
    args
}
fn both(env: &Environment, args: Vec<String>, exit: i32) -> Result<Value, String> {
    let cli = invoke(env, args.clone(), exit)?;
    let mcp = super::mcp::call_with_globals(env, &args, exit >= 2, &[])?;
    if cli != mcp {
        return Err(format!("CLI/MCP differ: {cli} {mcp}"));
    }
    super::schema::Registry::embedded()?.validate(&cli, &args, true)?;
    let full = serde_json::to_vec(&cli).map_err(|e| e.to_string())?.len();
    let comparable = super::source_terms::size(&cli, "")?;
    budget(&cli, full, comparable)?;
    Ok(cli)
}
fn sections() -> Check {
    let (env, stub) = setup("valid")?;
    let card = both(&env, args(None), 0)?;
    card_facts(&card)?;
    let snapshot = card
        .pointer("/result/snapshot")
        .and_then(Value::as_str)
        .ok_or("snapshot")?;
    let mut drugs = args(Some("drugs"));
    drugs.extend(["--snapshot".into(), snapshot.into()]);
    let rows = both(&env, drugs, 0)?;
    if rows.pointer("/result/drugs/records/0/role") != Some(&json!("1"))
        || rows.pointer("/result/drugs/records/1/role") != Some(&json!("2"))
        || rows.pointer("/result/drugs/records/2/role").is_some()
        || rows.pointer("/result/drugs/records/1/identity_counts/unii") != Some(&json!(0))
        || rows
            .pointer("/result/drugs/records/2/identity_counts/unii")
            .is_some()
    {
        return Err(format!("independent drug fields: {rows}"));
    }
    let mut ids = args(Some("package-ndc"));
    ids.extend([
        "--drug-index".into(),
        "0".into(),
        "--section-limit".into(),
        "5".into(),
        "--section-offset".into(),
        "0".into(),
        "--snapshot".into(),
        snapshot.into(),
    ]);
    let first = both(&env, ids, 0)?;
    let next = first
        .get("next")
        .and_then(Value::as_array)
        .and_then(|v| v.iter().find(|s| s.get("kind") == Some(&json!("page"))))
        .and_then(|s| s.get("args"))
        .ok_or("identifier continuation")?;
    let next: Vec<String> = serde_json::from_value(next.clone()).map_err(|e| e.to_string())?;
    let last = both(&env, next, 0)?;
    if last.pointer("/result/package_ndc/records") != Some(&json!(["F"]))
        || last.pointer("/result/package_ndc/offset") != Some(&json!(5))
    {
        return Err(format!("identifier page: {last}"));
    }
    let mut reaction = args(Some("reactions"));
    reaction.extend(["--snapshot".into(), snapshot.into()]);
    let result = both(&env, reaction.clone(), 0)?;
    if result.pointer("/result/reactions/records/0/term/label")
        != Some(&json!("Synthetic reaction"))
        || result
            .pointer("/result/reactions/records/0/term/id")
            .is_some()
        || result.pointer("/result/reactions/records/0/outcome") != Some(&json!("1"))
    {
        return Err(format!("reaction facts: {result}"));
    }
    reaction.extend(["--disable-sources".into(), "meddra".into()]);
    let denied = invoke(&env, reaction, 4)?;
    if denied.get("result").is_some()
        || denied.pointer("/sources/0/reason") != Some(&json!("disabled"))
    {
        return Err(format!("component retained read leaked: {denied}"));
    }
    let mut expired = args(Some("drugs"));
    expired.extend(["--snapshot".into(), "0".repeat(64)]);
    let expired = invoke(&env, expired, 3)?;
    if expired.pointer("/next/0/kind") != Some(&json!("restart")) {
        return Err(format!("expired snapshot: {expired}"));
    }
    if stub.finish()?.len() != 1 {
        return Err("retained section paging reached HTTP".into());
    }
    Ok(())
}
fn card_facts(card: &Value) -> Check {
    if card.pointer("/result/drug_count") != Some(&json!(3))
        || card.pointer("/result/reaction_count") != Some(&json!(1))
        || card.pointer("/result/patient/age") != Some(&json!({"value":42.5,"unit":"801"}))
        || card.pointer("/result/patient/weight").is_some()
        || card.pointer("/result/latest_received") != Some(&json!("20240202"))
    {
        return Err(format!("card facts: {card}"));
    }
    Ok(())
}

fn identifier_kinds() -> Check {
    let (env, stub) = setup("valid")?;
    let card = both(&env, args(None), 0)?;
    let snapshot = card
        .pointer("/result/snapshot")
        .and_then(Value::as_str)
        .ok_or("snapshot")?;
    for (section, field, expected) in [
        ("unii", "unii", "TESTA"),
        ("rxcui", "rxcui", "1"),
        ("product-ndc", "product_ndc", "SYNTHETIC"),
        ("spl-id", "spl_id", "SYNTHETIC-ID"),
        ("spl-set-id", "spl_set_id", "SYNTHETIC-SET"),
        (
            "application-number",
            "application_number",
            "SYNTHETIC-APPLICATION",
        ),
    ] {
        let mut words = args(Some(section));
        words.extend([
            "--drug-index".into(),
            "0".into(),
            "--snapshot".into(),
            snapshot.into(),
        ]);
        let value = both(&env, words, 0)?;
        if value.pointer(&format!("/result/{field}/records")) != Some(&json!([expected]))
            || value.get("next").is_some()
        {
            return Err(format!("identifier rewritten or acquired pivot: {value}"));
        }
    }
    for (index, exit, expect_result) in [(1, 0, true), (2, 1, false)] {
        let mut words = args(Some("unii"));
        words.extend([
            "--drug-index".into(),
            index.to_string(),
            "--snapshot".into(),
            snapshot.into(),
        ]);
        let value = both(&env, words, exit)?;
        if value.get("result").is_some() != expect_result {
            return Err(format!("missing and empty list collapsed: {value}"));
        }
        if expect_result && value.pointer("/result/unii/records").is_some() {
            return Err(format!("empty source list: {value}"));
        }
    }
    if stub.finish()?.len() != 1 {
        return Err("identifier sections reached HTTP".into());
    }
    Ok(())
}

fn budget(value: &Value, full: usize, comparable: usize) -> Check {
    let section = value
        .pointer("/query/section")
        .and_then(Value::as_str)
        .unwrap_or("card");
    let status = value.get("status").and_then(Value::as_str).unwrap_or("");
    // j13008: compact CLI/MCP maxima; every ceiling is measured bytes plus15%.
    let names = match (section, status) {
        (_, "usage") => ("faers-envelope-143", "faers-envelope-143"),
        ("card", "complete") => ("faers-envelope-1442", "faers-envelope-1320"),
        ("card", "absent") => ("faers-envelope-298", "faers-envelope-183"),
        ("card", "failed") => ("faers-envelope-428", "faers-envelope-313"),
        ("drugs", "failed") => ("faers-envelope-419", "faers-envelope-304"),
        ("reactions", "failed") => ("faers-envelope-423", "faers-envelope-308"),
        ("unii", "failed") => ("faers-envelope-433", "faers-envelope-318"),
        ("unii", "absent") => ("faers-envelope-629", "faers-envelope-507"),
        ("drugs", "complete") => ("faers-envelope-1832", "faers-envelope-1710"),
        ("reactions", "complete") => ("faers-envelope-1403", "faers-envelope-1288"),
        ("unii", "complete") => ("faers-envelope-897", "faers-envelope-775"),
        ("rxcui", "complete") => ("faers-envelope-896", "faers-envelope-774"),
        ("product-ndc", "complete") => ("faers-envelope-922", "faers-envelope-800"),
        ("package-ndc", "complete") => ("faers-envelope-1128", "faers-envelope-1006"),
        ("spl-id", "complete") => ("faers-envelope-910", "faers-envelope-788"),
        ("spl-set-id", "complete") => ("faers-envelope-923", "faers-envelope-801"),
        ("application-number", "complete") => ("faers-envelope-955", "faers-envelope-833"),
        _ => return Err(format!("unmeasured FAERS envelope: {section}/{status}")),
    };
    let measured = (super::output_limit(names.0)?, super::output_limit(names.1)?);
    let caps = (
        (measured.0 * 115).div_ceil(100),
        (measured.1 * 115).div_ceil(100),
    );
    if full > caps.0 || comparable > caps.1 {
        return Err(format!(
            "FAERS output exceeds measured +15% ceiling: {full}/{comparable} > {caps:?}"
        ));
    }
    Ok(())
}

fn age_input(body: &mut Value, mode: &str) -> Check {
    if let Some(unit) = mode.strip_prefix("age-unit-") {
        let patient = body
            .pointer_mut("/results/0/patient")
            .and_then(Value::as_object_mut)
            .ok_or("patient")?;
        if unit == "absent" {
            patient.remove("patientonsetageunit");
        } else {
            patient.insert("patientonsetageunit".into(), json!(unit));
        }
    }
    if mode == "age-invalid" {
        *body
            .pointer_mut("/results/0/patient/patientonsetage")
            .ok_or("age")? = json!("NaN");
    }
    Ok(())
}
