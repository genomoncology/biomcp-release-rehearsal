//! Shared RPC validation and advertised CLI tool.
use serde_json::{Value, json};

// Official contracts: https://modelcontextprotocol.io/specification/2026-07-28/basic/versioning
// and https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
pub(super) const INSTRUCTIONS: &str = crate::help::FRONT_MAP;
pub(super) fn instructions(
    mode: crate::config::ToolMode,
    context: crate::command::Context<'_>,
) -> String {
    let mut instructions = if mode == crate::config::ToolMode::Typed {
        super::typed::instructions(context)
            .unwrap_or_else(|_| "Use help for command arguments.".into())
    } else if context.globals.hosted {
        super::hosted::instructions(context)
            .unwrap_or_else(|_| "Use help for command arguments.".into())
    } else {
        INSTRUCTIONS.into()
    };
    instructions.push('\n');
    instructions.push_str(crate::grammar::ENUM_CASE_GUIDANCE);
    instructions
}
pub(super) const REVISIONS: [&str; 4] = ["2026-07-28", "2025-11-25", "2025-06-18", "2025-03-26"];
pub(super) const VERSION_KEY: &str = "io.modelcontextprotocol/protocolVersion";
pub(super) const CAPABILITIES_KEY: &str = "io.modelcontextprotocol/clientCapabilities";

pub(super) fn info() -> Value {
    json!({"name":"biomcp", "version":env!("CARGO_PKG_VERSION")})
}

pub(super) fn error(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0", "id":id, "error":{"code":code,"message":message}})
}
/// Keeps the code and leading text hosting relies on, then says what to do.
pub(super) fn capacity(id: Value, limit: usize) -> Value {
    let message = format!(
        "Tool capacity exceeded: this server runs at most {limit} tool calls at once. \
         Retry this call after one of your running calls returns, \
         or after a few seconds if none are running."
    );
    error(id, -31000, &message)
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
    if within_cap(value) {
        return result(id, content(value.clone(), failed, form), modern);
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
// A summary line names where this form puts the JSON, so it never names an absent field.
pub(super) fn reply_note(form: ReplyForm) -> &'static str {
    match form {
        ReplyForm::StructuredOnly => "JSON reply is in structuredContent.",
        ReplyForm::Both | ReplyForm::TextOnly => "JSON reply follows.",
    }
}
fn content(value: Value, failed: bool, form: ReplyForm) -> Value {
    let mut result = match form {
        ReplyForm::Both => json!({"content":[{"type":"text","text":value.to_string()}],
            "structuredContent":value.clone(), "isError":failed}),
        ReplyForm::StructuredOnly => {
            json!({"content":[{"type":"text","text":reply_note(form)}],
            "structuredContent":value.clone(), "isError":failed})
        }
        ReplyForm::TextOnly => {
            json!({"content":[{"type":"text","text":value.to_string()}], "isError":failed})
        }
    };
    crate::view::protein::append_mcp(&mut result, &value);
    result
}
pub(super) fn within_cap(value: &Value) -> bool {
    // Admit the JSON value once, independent of its configured wire representation.
    // Count Unicode scalars after JSON escaping, not UTF-8 bytes or source text.
    value.to_string().chars().take(RESULT_CHARACTER_CAP).count() < RESULT_CHARACTER_CAP
}
