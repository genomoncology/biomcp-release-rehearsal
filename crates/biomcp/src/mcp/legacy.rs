//! Initialization-based protocol edge.
use super::protocol;
use serde_json::{Value, json};

pub(super) fn initialize(
    params: &Value,
    mode: crate::config::ToolMode,
    context: crate::command::Context<'_>,
) -> Option<Value> {
    let revision = revision(params.get("protocolVersion")?.as_str()?);
    params.get("capabilities")?.as_object()?;
    let info = params.get("clientInfo")?;
    info.get("name")?.as_str()?;
    info.get("version")?.as_str()?;
    Some(json!({"protocolVersion":revision,
        "capabilities":{"tools":{}}, "serverInfo":protocol::info(), "instructions":protocol::instructions(mode, context)}))
}

// Pinned lifecycle version negotiation preserves a supported requested revision.
// https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle#version-negotiation
pub(super) fn revision(requested: &str) -> &'static str {
    protocol::REVISIONS[1..]
        .iter()
        .copied()
        .find(|revision| *revision == requested)
        .unwrap_or(protocol::REVISIONS[1])
}
