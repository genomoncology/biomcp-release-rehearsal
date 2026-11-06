//! Outside-in helpers use the advertised reference route, never a second entity policy.
use serde_json::{Value, json};
use std::io::Write;

pub(in crate::runner) fn command(tools: &[Value], name: &str) -> Option<Value> {
    if let Some(tool) = tools.iter().find(|tool| tool["name"] == name) {
        return Some(tool.clone());
    }
    let reference = tools
        .iter()
        .find(|tool| tool["name"] == "biomcp_reference")?;
    if !reference
        .pointer("/inputSchema/properties/command/enum")?
        .as_array()?
        .contains(&json!(name))
    {
        return None;
    }
    Some(
        json!({"name":name,"inputSchema":reference.pointer("/inputSchema/properties/arguments")?,"x-reference":true}),
    )
}

pub(in crate::runner) fn call(tool: &Value, arguments: &Value) -> Value {
    if tool["x-reference"] == true {
        json!({"name":"biomcp_reference","arguments":{"command":tool["name"],"arguments":arguments}})
    } else {
        json!({"name":tool["name"],"arguments":arguments})
    }
}

pub(super) fn check(tools: &[Value]) -> Result<(), String> {
    if tools.len() != 19 {
        return Err(format!(
            "approved clinical catalog requires 19 tools, got {}",
            tools.len()
        ));
    }
    for (verb, entity, _) in biomcp_types::command_paths() {
        let name = format!("{verb}_{entity}").replace('-', "_");
        command(tools, &name).ok_or_else(|| format!("unreachable registry command {name}"))?;
    }
    for name in [
        "get_drug",
        "search_drug",
        "get_trial",
        "search_trial",
        "get_article",
        "search_article",
        "get_variant",
        "search_variant",
        "biomcp_reference",
        "get_disease",
        "search_disease",
        "get_diagnostic",
        "search_diagnostic",
        "get_gene",
        "search_adverse_event",
        "biomcp",
        "biomcp_help",
        "search",
        "fetch",
    ] {
        if !tools.iter().any(|tool| tool["name"] == name) {
            return Err(format!("approved advertised tool is missing: {name}"));
        }
    }
    writeln!(
        std::io::stdout(),
        "TYPED_EAGER_CATALOG tools={} definition_bytes={}",
        tools.len(),
        serde_json::to_vec(tools)
            .map_err(|error| error.to_string())?
            .len()
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn budgets(
    env: &crate::runner::launch::Environment,
    tools: &[Value],
) -> Result<(), String> {
    let bytes = serde_json::to_vec(tools)
        .map_err(|error| error.to_string())?
        .len();
    for (config, expected) in [
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_tool_limit":tools.len(),"mcp_typed_definition_bytes":bytes}),
            None,
        ),
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_tool_limit":tools.len()-1}),
            Some("mcp_typed_tool_limit"),
        ),
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_definition_bytes":bytes-1}),
            Some("mcp_typed_definition_bytes"),
        ),
        (
            json!({"mcp_tool_mode":"typed","mcp_typed_output_schema":true,"mcp_typed_definition_bytes":bytes}),
            Some("mcp_typed_definition_bytes"),
        ),
    ] {
        env.config(&config).map_err(|error| error.to_string())?;
        let replies = crate::runner::mcp::exchange_with_globals(
            env,
            &[crate::runner::mcp::request(
                json!(1),
                "tools/list",
                json!({}),
            )],
            &["--offline".into()],
        )?;
        let reply = replies.first().ok_or("budget reply")?;
        if let Some(field) = expected {
            if !reply
                .pointer("/error/message")
                .and_then(Value::as_str)
                .is_some_and(|message| message.contains(field))
            {
                return Err(format!("missing actionable budget error: {reply}"));
            }
        } else if reply.pointer("/result/tools") != Some(&json!(tools)) {
            return Err("exact byte/count budgets changed the catalog".into());
        }
    }
    Ok(())
}
