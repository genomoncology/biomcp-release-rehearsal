//! The real cache expires between source replies; rejected identity must retain stale checked data.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;
pub(super) fn trials() -> Vec<Trial> {
    vec![transcript_trial(
        "drug-identity/stale-fallback-keeps-rejected-record-problem",
        || stale().map_err(Into::into),
    )]
}
fn stale() -> Result<(), String> {
    let mut count = 0;
    let stub = Stub::for_source_status(
        "api.fda.gov",
        "application/json",
        move |_| {
            count += 1;
            let mut value = label();
            if count == 2 {
                corrupt(&mut value, "conflicting-spl")?;
            }
            Ok(value.to_string())
        },
        |_| 200,
    )?;
    let env = Environment::new().map_err(|e| e.to_string())?;
    let cert = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    fs::write(&cert, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"ca_bundle":cert,"proxy":format!("http://{}",stub.address)}))
        .map_err(|e| e.to_string())?;
    let args = [
        "get",
        "drug",
        "492dbdb2-077e-4064-bff3-372d6af0a7a2",
        "--source",
        "openfda-label",
    ]
    .map(str::to_owned)
    .to_vec();
    let original = invoke(&env, args.clone(), 0)?;
    let path = fs::read_dir(env.root().join("cache/biomcp"))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|e| e == "sqlite"))
        .ok_or("reply cache")?;
    let cache = rusqlite::Connection::open(path).map_err(|e| e.to_string())?;
    cache
        .execute("UPDATE reply SET fresh_until=0", [])
        .map_err(|e| e.to_string())?;
    let stale = invoke(&env, args.clone(), 0)?;
    if stale.get("status") != Some(&json!("partial"))
        || stale.pointer("/sources/0/state") != Some(&json!("stale"))
        || stale.pointer("/sources/0/reason") != Some(&json!("bad-reply"))
        || stale.get("result") != original.get("result")
        || stale.pointer("/problems/0/field") != Some(&json!("record"))
        || stale.pointer("/problems/0/source") != Some(&json!("openfda-label"))
    {
        return Err(format!(
            "rejected identity lost its record problem during stale fallback: {stale}"
        ));
    }
    let corrected = invoke(&env, args.clone(), 0)?;
    if corrected != original
        || super::mcp::call_with_globals(&env, &args, false, &[])? != corrected
        || stub.finish()?.len() != 3
    {
        return Err("rejected stale identity cached, or corrected identity failed to cache".into());
    }
    let full = serde_json::to_vec(&stale).map_err(|e| e.to_string())?.len();
    let comparable = super::source_terms::size(&stale, "")?;
    if full > crate::runner::output_limit("drug-identity-stale-full")?
        || comparable > crate::runner::output_limit("drug-identity-stale-comparable")?
    {
        return Err(format!(
            "stale identity refusal exceeds size ceiling: full={full}, comparable={comparable}"
        ));
    }
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "synthetic stale identity: full={full}, comparable={comparable}"
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
fn corrupt(value: &mut Value, fault: &str) -> Result<(), String> {
    match fault {
        "wrong-therapy" => {
            *value
                .pointer_mut("/data/therapy/id")
                .ok_or("therapy identity")? = json!(84)
        }
        "missing-join" => {
            let therapies = value
                .pointer_mut("/data/evidenceItems/nodes/0/therapies")
                .and_then(Value::as_array_mut)
                .ok_or("claim therapies")?;
            for therapy in therapies {
                therapy.as_object_mut().ok_or("therapy")?.remove("id");
            }
        }
        _ => {
            *value
                .pointer_mut("/results/0/openfda/spl_set_id")
                .ok_or("SPL identity")? = json!(["b118a40d-6b56-cee3-10f6-ded821a97018"])
        }
    }
    Ok(())
}
fn label() -> Value {
    json!({"meta":{"results":{"skip":0,"limit":1,"total":1}},"results":[{
        "id":"492dbdb2-077e-4064-bff3-372d6af0a7a2","set_id":"492dbdb2-077e-4064-bff3-372d6af0a7a2",
        "version":"1","effective_time":"20261024","openfda":{"brand_name":["Synthetic brand"],
        "spl_set_id":["492dbdb2-077e-4064-bff3-372d6af0a7a2"]}}]})
}
