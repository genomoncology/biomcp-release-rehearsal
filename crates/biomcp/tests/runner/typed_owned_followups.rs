//! Schema-owned follow-ups retain data while arrays and singular Steps become calls.
//! live_check: synthetic drug replay and retained native public organization replay; no live requests.
use crate::runner::{
    launch::{Environment, fixtures},
    mcp::{exchange_with_globals, request},
    transcript::trial,
};
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    vec![
        trial("mcp-parity/typed-owned-drug-alternatives", || {
            drug().map_err(Into::into)
        }),
        trial("mcp-parity/typed-owned-organization-pivot", || {
            organization().map_err(Into::into)
        }),
        trial("mcp-parity/typed-owned-native-json-boundary", || {
            native_boundary().map_err(Into::into)
        }),
        trial("mcp-parity/typed-owned-branch-deduplication", || {
            branch_deduplication().map_err(Into::into)
        }),
        trial("mcp-parity/typed-owned-research-text", || {
            research_text().map_err(Into::into)
        }),
        trial("mcp-parity/typed-owned-fresh-unavailable", || {
            fresh().map_err(Into::into)
        }),
    ]
}
fn drug() -> Result<(), String> {
    for (value, tools) in responses(
        "drug/name-get",
        json!({}),
        "get_drug",
        json!({"input":"Example ingredient","section":["approvals"]}),
    )? {
        if value.pointer("/result/identity/id") != Some(&json!("NDA021335"))
            || value.pointer("/result/identity/name_resolution/input")
                != Some(&json!("Example ingredient"))
        {
            return Err(format!("drug source data changed: {value}"));
        }
        call(
            value
                .pointer("/result/identity/name_resolution/alternatives/0")
                .ok_or("drug alternatives")?,
            "get_drug",
            json!({"input":"NDA021588","source":"drugsfda"}),
            &tools,
        )?;
    }
    Ok(())
}
fn organization() -> Result<(), String> {
    for (value, tools) in responses(
        "organization/native",
        json!({"keys":{"nci_api_key":"synthetic-nci-replay"}}),
        "search_organization",
        json!({"source":"nci","name":"Dana-Farber","limit":5}),
    )? {
        if value.pointer("/result/items/3/name") != Some(&json!("Dana-Farber Cancer Institute"))
            || value.pointer("/result/items/3/count") != Some(&json!(3376))
        {
            return Err(format!("organization source data changed: {value}"));
        }
        call(
            value
                .pointer("/result/items/3/next")
                .ok_or("organization pivot")?,
            "search_trial",
            json!({"facility":"Dana-Farber Cancer Institute","source":"nci"}),
            &tools,
        )?;
    }
    Ok(())
}
fn call(step: &Value, name: &str, arguments: Value, tools: &[Value]) -> Result<(), String> {
    if step.get("args").is_some()
        || step.get("name") != Some(&json!(name))
        || step.get("arguments") != Some(&arguments)
    {
        return Err(format!(
            "CLI follow-up survived or typed call changed: {step}"
        ));
    }
    let tool = tools
        .iter()
        .find(|tool| tool["name"] == name)
        .ok_or("unadvertised nested call")?;
    jsonschema::validator_for(&tool["inputSchema"])
        .map_err(|e| e.to_string())?
        .validate(&arguments)
        .map_err(|e| e.to_string())
}
fn responses(
    replay: &str,
    config: Value,
    name: &str,
    arguments: Value,
) -> Result<Vec<(Value, Vec<Value>)>, String> {
    ["both", "structured-only", "text-only"]
        .into_iter()
        .map(|form| {
            let mut config = config.clone();
            config
                .as_object_mut()
                .ok_or("configuration object")?
                .insert("mcp_reply_form".into(), json!(form));
            response(replay, config, name, arguments.clone())
        })
        .collect()
}

fn response(
    replay: &str,
    mut config: Value,
    name: &str,
    arguments: Value,
) -> Result<(Value, Vec<Value>), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let fields = config.as_object_mut().ok_or("configuration object")?;
    fields.insert("mcp_tool_mode".into(), json!("typed"));
    fields.insert("mcp_typed_output_schema".into(), json!(true));
    env.config(&config).map_err(|e| e.to_string())?;
    let globals = env.globals(Some(&fixtures().join("replies").join(replay)), false);
    let replies = exchange_with_globals(
        &env,
        &[request(json!(1), "tools/list", json!({}))],
        &globals,
    )?;
    let tools = replies
        .iter()
        .find(|v| v["id"] == 1)
        .and_then(|v| v.pointer("/result/tools"))
        .and_then(Value::as_array)
        .ok_or("catalog")?
        .clone();
    let route = super::super::typed_catalog::command(&tools, name).ok_or("nested request route")?;
    let replies = exchange_with_globals(
        &env,
        &[request(
            json!(2),
            "tools/call",
            super::super::typed_catalog::call(&route, &arguments),
        )],
        &globals,
    )?;
    let reply = replies
        .iter()
        .find(|v| v["id"] == 2)
        .ok_or("nested reply")?;
    let value = reply
        .pointer("/result/structuredContent")
        .cloned()
        .or_else(|| {
            let text = reply.pointer("/result/content/0/text")?.as_str()?;
            serde_json::from_str(text.split_once('\n')?.1).ok()
        })
        .ok_or("nested body")?;
    Ok((value, tools))
}
fn research_text() -> Result<(), String> {
    for (value, _) in responses("gene/main", json!({}), "fetch", json!({"id":"gene BRAF"}))? {
        let text = value
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("missing research text: {value}"))?;
        if !text.contains("BRAF")
            || !text.contains("HGNC:1097")
            || !text.contains("\"name\":\"get_gene\"")
            || text.contains("biomcp get gene")
            || text.contains("biomcp help")
        {
            return Err(format!(
                "research text lost native data or retained CLI follow-ups: {text}"
            ));
        }
    }
    Ok(())
}

fn fresh() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"mcp_tool_mode":"typed","mcp_typed_output_schema":true}))
        .map_err(|e| e.to_string())?;
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &["get", "gene", "BRAF", "--offline"].map(str::to_owned),
        )
        .output()
        .map_err(|e| e.to_string())?;
    let cli: Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    if !cli
        .get("next")
        .and_then(Value::as_array)
        .is_some_and(|steps| steps.iter().any(|step| step["kind"] == "fresh"))
    {
        return Err(format!("CLI supplied no force-refresh witness: {cli}"));
    }
    let replies = exchange_with_globals(
        &env,
        &[request(
            json!(1),
            "tools/call",
            json!({"name":"get_gene","arguments":{"input":["BRAF"]}}),
        )],
        &["--offline".into()],
    )?;
    let value = replies
        .first()
        .and_then(|v| v.pointer("/result/structuredContent"))
        .ok_or("fresh reply")?;
    if value
        .get("next")
        .and_then(Value::as_array)
        .is_some_and(|steps| {
            steps
                .iter()
                .any(|step| step["kind"] == "fresh" || step.get("args").is_some())
        })
    {
        return Err(format!(
            "unavailable force-refresh survived typed output: {value}"
        ));
    }
    Ok(())
}

// Direct schema-boundary checks cannot be injected through the closed command models.
fn native_boundary() -> Result<(), String> {
    let schema = json!({"$defs":{"Reply_Step":{"type":"object"},
        "Cycle":{"$ref":"#/$defs/Cycle"}},"type":"object","properties":{
        "owned":{"type":"object","properties":{"next":{"$ref":"#/$defs/Reply_Step"}}},
        "native":{"type":"object","additionalProperties":true},"cycle":{"$ref":"#/$defs/Cycle"}}});
    let native = json!({"next":{"kind":"pivot","args":["KEEP LITERAL"]},
        "alternatives":[{"kind":"pivot","args":["KEEP TOO"]}]});
    let mut value = json!({"owned":{"next":{"kind":"pivot","args":["get","gene","BRAF"]}},
        "native":native,"cycle":{"next":{"args":["KEEP CYCLE"]}}});
    let native_schema = schema
        .pointer("/properties/native")
        .ok_or("native schema")?;
    let native_value = value.get("native").ok_or("native JSON")?;
    if super::step_paths::paths(&schema, &schema, &value) != ["/owned/next"]
        || !super::step_paths::paths(&schema, native_schema, native_value).is_empty()
    {
        return Err("native JSON or nonproductive reference became executable control".into());
    }
    super::step_paths::remove(&mut value, "/owned/next");
    if value.get("native") != Some(&native)
        || value.pointer("/cycle/next/args") != Some(&json!(["KEEP CYCLE"]))
    {
        return Err("native JSON changed while removing an owned Step".into());
    }
    Ok(())
}
fn branch_deduplication() -> Result<(), String> {
    let schema = json!({"$defs":{"Reply_Step":{"type":"object"}},"properties":{
        "next":{"items":{"anyOf":[{"$ref":"#/$defs/Reply_Step"},{"$ref":"#/$defs/Reply_Step"}]}}}});
    let value = json!({"next":[{"kind":"pivot","args":["a"]},{"kind":"pivot","args":["b"]}]});
    if super::step_paths::paths(&schema, &schema, &value) != ["/next/0", "/next/1"] {
        return Err("overlapping branches did not retain one visit per owned Step".into());
    }
    Ok(())
}
