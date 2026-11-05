//! One help entry point delegates command paths to the existing CLI help owner.
use serde_json::{Value, json};

pub(super) fn tool() -> Value {
    json!({"name":"biomcp_help",
        "description":"Read BioMCP command help, argument schema, sections and examples for a command path.",
        "inputSchema":{"$schema":super::super::protocol::SCHEMA_DIALECT,"type":"object",
            "properties":{"command":{"type":"string","description":"Command path, such as search trial, get gene or sources; empty for the map."}},
            "required":["command"],"additionalProperties":false},
        "annotations":{"readOnlyHint":true,"destructiveHint":false,"openWorldHint":true}})
}
pub(super) fn parse(params: &Value) -> Option<super::super::research::Call> {
    let fields = params.get("arguments")?.as_object()?;
    if fields.len() != 1 {
        return None;
    }
    let command = fields.get("command")?.as_str()?;
    let mut args = vec![std::ffi::OsString::from("help")];
    args.extend(command.split_whitespace().map(std::ffi::OsString::from));
    Some(super::super::research::Call {
        args,
        projection: super::super::research::Projection::Cli,
    })
}
// Translate only CLI-owned help steps, preserving their complete command path.
pub(super) fn next(root: &crate::grammar::Root, args: &[&str]) -> Option<Value> {
    let (verb, path) = args.split_first()?;
    if *verb != "help" {
        return None;
    }
    let argv = args
        .iter()
        .copied()
        .map(std::ffi::OsString::from)
        .collect::<Vec<_>>();
    crate::grammar::parse(root, &argv).ok()?;
    Some(json!({"name":"biomcp_help","arguments":{"command":path.join(" ")}}))
}
