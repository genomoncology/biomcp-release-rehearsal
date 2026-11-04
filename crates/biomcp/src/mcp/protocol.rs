//! Shared RPC validation and advertised CLI tool.
use serde_json::{Value, json};

// Official contracts: https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning
// and https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
pub(super) const INSTRUCTIONS: &str = crate::help::FRONT_MAP;
pub(super) const REVISIONS: [&str; 4] = ["2026-07-28", "2025-11-25", "2025-06-18", "2025-03-26"];
pub(super) const VERSION_KEY: &str = "io.modelcontextprotocol/protocolVersion";
pub(super) const CAPABILITIES_KEY: &str = "io.modelcontextprotocol/clientCapabilities";

pub(super) fn info() -> Value {
    json!({"name":"biomcp", "version":env!("CARGO_PKG_VERSION")})
}

pub(super) fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":code,"message":message}})
}
pub(super) fn timeout(id: Value) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":-32000,"message":"Tool deadline exceeded",
        "data":{"reason":biomcp_types::Reason::DeadlineExceeded}}})
}

pub(super) fn valid_id(value: &Value) -> bool {
    value.is_string() || value.is_i64() || value.is_u64()
}

pub(super) fn validate(value: &Value) -> bool {
    value.is_object()
        && value.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
        && value.get("method").is_some_and(Value::is_string)
        && value.get("id").is_none_or(valid_id)
}

pub(super) fn valid_params(method: &str, params: &Value) -> bool {
    params.is_object()
        && params.get("_meta").is_none_or(Value::is_object)
        && (method != "tools/list" || params.get("cursor").is_none_or(Value::is_string))
}

// Tool schema dialect and result contract:
// https://modelcontextprotocol.io/specification/2025-11-25/server/tools
pub(super) const SCHEMA_DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";
const RESULT_CHARACTER_CAP: usize = 150_000;

use crate::config::ReplyForm;

const DESCRIPTION: &str = concat!(
    "Run a BioMCP CLI command. arguments has exactly one field: args, an array of command words (strings), ",
    "without the biomcp executable. Example: {\"args\":[\"help\"]}. Start with help for the command map; ",
    "use help <command> for options and schema <id> for a complete reply schema. ",
    "Replies are JSON source evidence with status, problems and next commands; inspect them before interpreting results."
);

pub(super) fn tool(root: &crate::grammar::Root, read_only: bool) -> std::io::Result<Value> {
    Ok(json!({"name":"biomcp", "description":DESCRIPTION,
        "annotations":{"readOnlyHint":read_only,"destructiveHint":false,"openWorldHint":true},
        "inputSchema":{"$schema":SCHEMA_DIALECT,"type":"object","properties":{
            "args":{"type":"array","items":{"type":"string"},
                "description":"CLI command words, without biomcp. Example: [\"help\"]"}},
            "required":["args"],"additionalProperties":false},"outputSchema":super::output_schema::for_tool(root, "biomcp")?}))
}

pub(super) fn tools(root: &crate::grammar::Root, read_only: bool) -> std::io::Result<Vec<Value>> {
    let mut tools = vec![tool(root, read_only)?];
    tools.extend(super::research::tools(root)?);
    Ok(tools)
}

pub(super) fn result(id: Value, mut value: Value, modern: bool) -> Value {
    if modern && let Some(object) = value.as_object_mut() {
        object.insert("resultType".into(), json!("complete"));
        object.insert(
            "_meta".into(),
            json!({"io.modelcontextprotocol/serverInfo":info()}),
        );
    }
    json!({"jsonrpc":"2.0","id":id,"result":value})
}

// Tool results use the configured content form and negotiated protocol metadata.
// JSON-RPC correlation IDs belong to the outer response, outside CallToolResult.
pub(super) fn tool_result(
    id: Value,
    outcome: &crate::command::Outcome,
    modern: bool,
    form: ReplyForm,
) -> Value {
    let failed = !matches!(
        outcome.status,
        crate::exit::Status::Success | crate::exit::Status::Absent
    );
    let Ok(value) = outcome.value() else {
        return self::error(id, -32603, "Internal error");
    };
    let reply = result(id.clone(), content(value.clone(), failed, form), modern);
    if reply.get("result").is_some_and(within_cap) {
        return reply;
    }
    let Ok(error) = serde_json::to_value(biomcp_types::ErrorEnvelope::from(
        biomcp_types::UsageError::McpResultTooBroad {
            cap: RESULT_CHARACTER_CAP as u64,
        },
    )) else {
        return self::error(id, -32603, "Internal error");
    };
    result(id, content(error, true, form), modern)
}
fn content(value: Value, failed: bool, form: ReplyForm) -> Value {
    match form {
        ReplyForm::Both => json!({"content":[{"type":"text","text":value.to_string()}],
            "structuredContent":value, "isError":failed}),
        ReplyForm::StructuredOnly => {
            json!({"content":[{"type":"text","text":"JSON reply is in structuredContent."}],
            "structuredContent":value, "isError":failed})
        }
        ReplyForm::TextOnly => {
            json!({"content":[{"type":"text","text":value.to_string()}], "isError":failed})
        }
    }
}
pub(super) fn within_cap(result: &Value) -> bool {
    // Count Unicode scalars after JSON escaping, not UTF-8 bytes or source text.
    result
        .to_string()
        .chars()
        .take(RESULT_CHARACTER_CAP)
        .count()
        < RESULT_CHARACTER_CAP
}
