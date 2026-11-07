//! Native release spelling stays in the annotation; only age assessment is normalized.
use crate::runner::{launch::Environment, network_stub::Stub, oncokb};
use serde_json::{Value, json};
use std::fs;

pub(super) fn check() -> Result<(), String> {
    for (native, assessed, max_age, old) in [
        (Some("10/24/2026"), Some("2026-10-24"), 1_000_000, false),
        (Some("01/01/2000"), Some("2000-01-01"), 365, true),
        (Some("unknown"), None, 365, false),
        (None, None, 365, false),
    ] {
        case(native, assessed, max_age, old)?;
    }
    Ok(())
}
fn case(
    native: Option<&str>,
    assessed: Option<&str>,
    max_age: u32,
    old: bool,
) -> Result<(), String> {
    let native = native.map(str::to_owned);
    let reply_date = native.clone();
    let stub = Stub::for_sources(
        &["www.oncokb.org"],
        |_| "application/json",
        move |request| {
            let mut value: Value =
                serde_json::from_str(&oncokb::reply(request)?).map_err(|e| e.to_string())?;
            let fields = value.as_object_mut().ok_or("annotation object")?;
            match &reply_date {
                Some(date) => {
                    fields.insert("lastUpdate".into(), json!(date));
                }
                None => {
                    fields.remove("lastUpdate");
                }
            }
            Ok(value.to_string())
        },
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    stub.retain(env.shared_root());
    let ca = env.root().join("ca.pem");
    fs::write(&ca, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(
        &json!({"proxy":format!("http://{}",stub.address),"ca_bundle":ca,
        "keys":{"oncokb_api_key":oncokb::KEY},"source_data_max_age_days":max_age}),
    )
    .map_err(|e| e.to_string())?;
    let result = oncokb::invoke(&env, &oncokb::args(), 0)?;
    stub.finish()?;
    let body_date = result
        .pointer("/result/candidates/0/sections/oncokb/data/last_update")
        .and_then(Value::as_str);
    if body_date != native.as_deref() {
        return Err(format!("native release spelling changed: {result}"));
    }
    let mut notices = result
        .get("sources")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|source| {
            source
                .get("notices")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        });
    let date = notices.find(|notice| notice.get("data_date").is_some());
    match (assessed, date) {
        (Some(expected), Some(notice))
            if notice.get("data_date").and_then(Value::as_str) == Some(expected)
                && notice.to_string().contains("Old data") == old =>
        {
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(format!("native release date assessment differs: {result}")),
    }
}
