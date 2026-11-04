//! Recorded replies must read into their published body type and write unchanged.
use biomcp_test_support::readers;
use serde_json::Value;

pub(super) fn check(bytes: &[u8], value: &Value, mcp: bool) -> Result<(), String> {
    let Some(schema) = value.get("schema").and_then(Value::as_str) else {
        // A raw schema command returns the requested schema artifact.
        return Ok(());
    };
    let rewritten = if let Some(reply) = readers::READERS.iter().find(|reply| reply.0 == schema) {
        (reply.1)(bytes)
    } else {
        // Error replies have their own checked schema singleton rather than a body.
        serde_json::from_slice::<biomcp_types::ErrorEnvelope>(bytes)
            .and_then(|reply| serde_json::to_vec(&reply))
    }
    .map_err(|error| format!("reply readback {schema}: {error}"))?;
    let expected = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let rewritten = if mcp {
        // MCP text is serialized from structured content, which sorts object keys.
        serde_json::from_slice::<Value>(&rewritten)
            .and_then(|reply| serde_json::to_vec(&reply))
            .map_err(|error| error.to_string())?
    } else {
        rewritten
    };
    if rewritten != expected {
        return Err(format!("reply readback {schema}: written bytes changed"));
    }
    Ok(())
}
