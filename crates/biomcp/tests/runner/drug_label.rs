//! Feature drug-label: captured identity subsets with explicitly synthetic label prose.
//! Whole native label bodies remain outside Git in the reviewed source evidence.
use super::{configuration::invoke, launch::Environment, network_stub::Stub};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), String>;
mod published;

pub(super) fn trials() -> Vec<Trial> {
    // The peer checks literal source label requests and counts reads after retaining all native heading pages.
    vec![
        transcript_trial(crate::runner::required_contract::LABEL_HEADINGS.0, || {
            headings().map_err(Into::into)
        }),
        transcript_trial("drug-label/bare-label-lists-headings", || {
            bare_label().map_err(Into::into)
        }),
        transcript_trial("drug-label/published-sections", || {
            published::sections().map_err(Into::into)
        }),
    ]
}
fn bare_label() -> Check {
    let id = identity(4)?;
    let (env, _stub) = get_setup(&id, record(&id))?;
    let mut args = get_args(&id)?;
    args.push("label".into());
    let cli = invoke(&env, args.clone(), 0)?;
    let mcp = super::mcp::call_with_globals(&env, &args, false, &[])?;
    for value in [&cli, &mcp] {
        let body = value.pointer("/result").unwrap_or(value);
        let listed = body
            .pointer("/sections")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("no sections: {value}"))?;
        if !listed.contains(&json!("boxed_warning"))
            || !listed.contains(&json!("indications_and_usage"))
        {
            return Err(format!("section names missing: {value}"));
        }
        let hint = body
            .pointer("/hint")
            .and_then(Value::as_str)
            .ok_or("no hint")?;
        if !hint.contains("label_section") || !hint.contains("--label-section") {
            return Err(format!("hint lacks the option: {hint}"));
        }
    }
    let mut unknown = args.clone();
    unknown.extend(["--label-section".into(), "effective_time".into()]);
    let refused_mcp = super::mcp::call_with_globals(&env, &unknown, true, &[])?;
    for refused in [invoke(&env, unknown, 2)?, refused_mcp] {
        let message = refused
            .pointer("/error/message")
            .and_then(Value::as_str)
            .ok_or("no refusal message")?;
        if !message.contains("--label-section") || refused.pointer("/error/valid_values").is_none()
        {
            return Err(format!("unknown section refusal unclear: {refused}"));
        }
    }
    env.config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|e| e.to_string())?;
    // The typed drug tool: a bad section on the label source, and a source with no label.
    for (source, option, valid) in [
        ("openfda-label", "label_section", "boxed_warning"),
        ("mychem", "source", "openfda-label"),
    ] {
        let typed = super::mcp::exchange_with_globals(
            &env,
            &[super::mcp::request(
                json!(1),
                "tools/call",
                json!({"name":"drug","arguments":{"action":"get","get":{
                    "input":field(&id,"set_id"),"source":source,
                    "section":["label"],"label_section":"effective_time"}}}),
            )],
            &[],
        )?;
        let refused = typed
            .first()
            .and_then(|r| r.pointer("/result/structuredContent"))
            .ok_or_else(|| format!("no typed reply: {typed:?}"))?;
        let message = refused.pointer("/error/message").and_then(Value::as_str);
        let listed = refused
            .pointer("/error/valid_values")
            .and_then(Value::as_array);
        let named = message.is_some_and(|m| m.contains("--label-section"))
            || refused.pointer("/error/option").and_then(Value::as_str) == Some(option);
        if !named || !listed.is_some_and(|l| l.contains(&json!(valid))) {
            return Err(format!("typed refusal unclear for {source}: {refused}"));
        }
    }
    Ok(())
}
fn identity(n: usize) -> Result<Value, String> {
    let rows: Vec<Value> = serde_json::from_str(&crate::runner::launch::fixture_text(
        "fixtures/drug/openfda/identities.json",
    )?)
    .map_err(|e| e.to_string())?;
    rows.get(n).cloned().ok_or("identity index".into())
}
fn record(identity: &Value) -> Value {
    json!(
        {"id":field(identity,"id"),"set_id":field(identity,"set_id"),"version":field(identity,"version"),"effective_time":field(identity,
        "effective_time"),"openfda":{"brand_name":[field(identity,"brand")],"spl_set_id":[field(identity,"set_id")]},
        "boxed_warning":["Synthetic warning fixture."],"dosage_and_administration":[" Synthetic dose fixture.\n"],
        "drug_interactions":["Synthetic interaction fixture."],"indications_and_usage":["Synthetic indication fixture."],
        "warnings_and_cautions":["Synthetic caution fixture."]}
    )
}
fn setup(body: Value, selector: String, status: u16) -> Result<(Environment, Stub), String> {
    setup_raw(body.to_string(), selector, status)
}
fn setup_raw(body: String, selector: String, status: u16) -> Result<(Environment, Stub), String> {
    let expected = format!(
        "GET /drug/label.json?search={selector}&limit=1&sort=effective_time%3Adesc HTTP/1.1"
    );
    let stub = Stub::for_source_status(
        "api.fda.gov",
        "application/json",
        move |request| {
            if request.lines().next() != Some(expected.as_str()) {
                return Err(format!("unexpected request: {request}"));
            }
            Ok(body.clone())
        },
        move |_| status,
    )?;
    environment(stub)
}
fn environment(stub: Stub) -> Result<(Environment, Stub), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let cert = env.root().join("ca.pem");
    stub.retain(env.shared_root());
    std::fs::write(&cert, &stub.certificate).map_err(|e| e.to_string())?;
    env.config(&json!({"ca_bundle":cert,"proxy":format!("http://{}",stub.address)}))
        .map_err(|e| e.to_string())?;
    Ok((env, stub))
}
fn page(record: Value) -> Value {
    json!({"meta":{"results":{"skip":0,"limit":1,"total":1}},"results":[record]})
}
fn get_args(identity: &Value) -> Result<Vec<String>, String> {
    Ok(vec![
        "get".into(),
        "drug".into(),
        field(identity, "set_id").as_str().ok_or("set")?.into(),
        "--source".into(),
        "openfda-label".into(),
    ])
}
fn get_setup(identity: &Value, record: Value) -> Result<(Environment, Stub), String> {
    setup(
        page(record),
        format!(
            "openfda.spl_set_id%3A%22{}%22",
            field(identity, "set_id").as_str().ok_or("set")?
        ),
        200,
    )
}
fn both(env: &Environment, args: Vec<String>, exit: i32) -> Result<Value, String> {
    let value = invoke(env, args.clone(), exit)?;
    super::schema::Registry::embedded()?.validate(&value, &args, true)?;
    let mcp = super::mcp::call_with_globals(env, &args, exit >= 2, &[])?;
    if value != mcp {
        return Err("CLI and MCP differ".into());
    }
    measure(&value)?;
    Ok(value)
}
fn measure(value: &Value) -> Check {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    let comparable = super::source_terms::size(value, "")?;
    // Peak retained full-body replay is 7661 comparable bytes; allow 15% growth.
    if comparable > crate::runner::output_limit("label-comparable")? {
        return Err(format!(
            "label exceeds measured comparable budget {comparable}/8811"
        ));
    }
    if bytes > crate::runner::output_limit("label-full")? {
        return Err(format!("label exceeds full allocation {bytes}/8192"));
    }
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "drug-label: full {bytes} bytes/{} tokens, comparable {comparable} bytes/{} tokens",
        bytes.div_ceil(4),
        comparable.div_ceil(4)
    )
    .map_err(|e| e.to_string())
}

fn headings() -> Check {
    let id = identity(4)?;
    let record = record(&id);
    let (env, stub) = get_setup(&id, record.clone())?;
    for (index, heading) in [
        "boxed_warning",
        "dosage_and_administration",
        "drug_interactions",
        "indications_and_usage",
        "warnings_and_cautions",
    ]
    .into_iter()
    .enumerate()
    {
        let mut args = get_args(&id)?;
        args.extend(["label".into(), "--label-section".into(), heading.into()]);
        let value = both(&env, args, 0)?;
        let measured = [1208, 1232, 1220, 1227, 1224]
            .get(index)
            .copied()
            .ok_or("heading budget")?;
        budget(&value, measured)?;
        if field(&value, "schema") != "biomcp.drug-label/7"
            || value.pointer("/result/label/data/text") != record.pointer(&format!("/{heading}/0"))
            || value.pointer("/result/label/data/retrieved").is_none()
        {
            return Err(format!("native heading or text changed: {heading}"));
        }
    }
    if stub.finish()?.len() != 1 {
        return Err("cached field selections fetched the label again".into());
    }
    Ok(())
}

fn field<'a>(value: &'a Value, key: &str) -> &'a Value {
    value.get(key).unwrap_or(&Value::Null)
}

fn budget(value: &Value, measured: usize) -> Check {
    let actual = super::source_terms::size(value, "")?;
    let bound = (measured * 115).div_ceil(100);
    if actual > bound {
        return Err(format!(
            "label case exceeds measured comparable budget {actual}/{bound}"
        ));
    }
    Ok(())
}
