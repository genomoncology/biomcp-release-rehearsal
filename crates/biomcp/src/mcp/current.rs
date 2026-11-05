//! Stateless 2026-07-28 protocol edge.
use super::protocol::{self, CAPABILITIES_KEY, REVISIONS, VERSION_KEY};
use serde_json::{Value, json};

pub(super) fn identified(params: &Value) -> bool {
    params
        .get("_meta")
        .and_then(Value::as_object)
        .is_some_and(|meta| {
            [
                VERSION_KEY,
                CAPABILITIES_KEY,
                "io.modelcontextprotocol/clientInfo",
            ]
            .iter()
            .any(|key| meta.contains_key(*key))
        })
}

pub(super) fn validate(id: &Value, params: &Value) -> Result<(), Value> {
    let meta = params.get("_meta");
    let version = meta
        .and_then(|m| m.get(VERSION_KEY))
        .and_then(Value::as_str);
    if version.is_none()
        || meta
            .and_then(|m| m.get(CAPABILITIES_KEY))
            .is_none_or(|v| !v.is_object())
    {
        return Err(protocol::error(id.clone(), -32602, "Invalid params"));
    }
    if version != REVISIONS.first().copied() {
        return Err(json!({"jsonrpc":"2.0", "id":id, "error":{
            "code":-32022,"message":"Unsupported protocol version",
            "data":{"supported":REVISIONS,"requested":version}}}));
    }
    Ok(())
}

pub(super) fn discover(
    mode: crate::config::ToolMode,
    context: crate::command::Context<'_>,
) -> Value {
    json!({"supportedVersions":REVISIONS,"instructions":protocol::instructions(mode, context),"capabilities":{"tools":{}}})
}
