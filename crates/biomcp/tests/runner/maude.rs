//! Feature maude-reports: synthetic source-native device reports through CLI and MCP.
use super::{configuration::invoke as raw_invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
pub(super) type Check = Result<(), String>;

pub(super) fn trials() -> Vec<Trial> {
    // The test binds runtime report snapshots into complete device/narrative pages and checks that retained reads avoid HTTP.
    vec![
        transcript_trial(crate::runner::required_contract::MAUDE_SECTIONS.0, || {
            sections().map_err(Into::into)
        }),
        transcript_trial("maude-reports/rejects-nonnumeric-native-report-key", || {
            invalid_report_key().map_err(|error| error.to_string().into())
        }),
    ]
}
pub(super) fn row() -> Value {
    json!(
        {"mdr_report_key":"160","report_number":"Synthetic-160","event_type":"Other","date_of_event":"20240101","date_received":"20240202",
        "report_date":"20240303","date_report":"","number_devices_in_event":"9","number_patients_in_event":"2",
        "device":[{"device_sequence_number":"1","generic_name":"Synthetic device","brand_name":"","manufacturer_d_name":"Synthetic maker",
        "manufacturer_d_address_1":"Excluded"},{"device_sequence_number":"2","model_number":"MODEL2"}],"mdr_text":[{"mdr_text_key":"A",
        "patient_sequence_number":"1","text_type_code":"D","text":"Synthetic complete first narrative."},{"mdr_text_key":"B","text":""},
        {"mdr_text_key":"C"}]}
    )
}
pub(super) fn response(row: Value) -> String {
    json!({"meta":{"last_updated":"2024-09-01","results":{"skip":0,"limit":1,"total":1}},"results":[row]}).to_string()
}
pub(super) fn environment(stub: &Stub) -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let ca = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    std::fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"ca_bundle":ca,"proxy":format!("http://{}",stub.address)}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
pub(super) fn invoke(env: &Environment, args: Vec<String>, exit: i32) -> Result<Value, String> {
    let value = raw_invoke(env, args, exit)?;
    super::maude_budget::check(&value)?;
    Ok(value)
}
pub(super) fn both(env: &Environment, words: &[&str], exit: i32) -> Result<Value, String> {
    let args: Vec<String> = words.iter().map(|v| (*v).into()).collect();
    let cli = invoke(env, args.clone(), exit)?;
    super::schema::Registry::embedded()?.validate(&cli, &args, true)?;
    let mcp = super::mcp::call_with_globals(env, &args, exit >= 2, &[])?;
    if !super::reply_comparison::same_reply(&cli, &mcp)? {
        return Err(format!("MAUDE CLI/MCP differ: {cli} / {mcp}"));
    }
    Ok(cli)
}

fn sections() -> Check {
    let stub = Stub::for_source("api.fda.gov", "application/json", |_| Ok(response(row())))?;
    let env = environment(&stub)?;
    let devices = both(
        &env,
        &[
            "get",
            "adverse-event",
            "160",
            "devices",
            "--source",
            "maude",
            "--section-limit",
            "1",
        ],
        0,
    )?;
    if devices.pointer("/result/devices/total") != Some(&json!(2))
        || devices.pointer("/result/devices/records/0/brand_name") != Some(&json!(""))
        || devices.to_string().contains("Excluded")
    {
        return Err(format!("device projection changed: {devices}"));
    }
    let next = devices
        .get("next")
        .and_then(Value::as_array)
        .and_then(|v| v.iter().find(|v| v.get("kind") == Some(&json!("page"))))
        .and_then(|v| v.get("args"))
        .ok_or("device next page")?;
    let args: Vec<String> = serde_json::from_value(next.clone()).map_err(|e| e.to_string())?;
    let second = invoke(&env, args, 0)?;
    if second.pointer("/result/devices/records/0/index") != Some(&json!(1))
        || second.pointer("/result/devices/records/0/model") != Some(&json!("MODEL2"))
    {
        return Err("device page lost index/model".into());
    }
    let narrative = both(
        &env,
        &[
            "get",
            "adverse-event",
            "160",
            "narrative",
            "--source",
            "maude",
        ],
        0,
    )?;
    if narrative.pointer("/result/narrative/records/0/text")
        != Some(&json!("Synthetic complete first narrative."))
        || narrative.pointer("/result/narrative/records/1/text") != Some(&json!(""))
        || narrative
            .pointer("/result/narrative/records/2/text")
            .is_some()
    {
        return Err(format!("whole narrative/missingness lost: {narrative}"));
    }
    if devices.pointer("/result/snapshot") != narrative.pointer("/result/snapshot")
        || stub.finish()?.len() != 1
    {
        return Err("report sections changed snapshot or fetched twice".into());
    }
    Ok(())
}

fn invalid_report_key() -> Result<(), Box<dyn std::error::Error>> {
    super::response_cases::checked_steps(
        "adverse-event/maude-filters-synthetic",
        |source, record| {
            if source == "maude" {
                for exchange in record
                    .get_mut("exchanges")
                    .and_then(Value::as_array_mut)
                    .into_iter()
                    .flatten()
                {
                    if let Some(key) = exchange.pointer_mut("/body/value/results/0/mdr_report_key")
                    {
                        *key = json!("160x");
                    }
                }
            }
        },
        json!([{
            "run": ["search", "adverse-event", "--source", "maude", "--manufacturer", "Synthetic maker",
                "--product-code", "ABC", "--product-code", "XYZ", "--problem", "Synthetic problem"],
            "exit": 4, "is_error": true,
            "expect": {"/status": "failed", "/sources/0/reason": "bad-reply", "": {"omits": ["result"]}}
        }]),
    )?;
    Ok(())
}
