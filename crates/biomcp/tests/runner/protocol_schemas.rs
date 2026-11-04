//! Feature mcp-parity: advertised schemas validate actual reply forms and reject unrelated objects.
use super::super::launch::Environment;
use serde_json::{Value, json};
use std::io::Write;

pub(super) fn check(tools: &[Value], reply: &Value) -> Result<(), String> {
    for advertised in tools {
        let schema = advertised
            .get("outputSchema")
            .ok_or("output schema missing")?;
        if schema == &json!({"type":"object"}) {
            return Err("tool advertises a bare object output schema".into());
        }
        local_references(schema, schema)?;
        let validator = jsonschema::validator_for(schema).map_err(|error| error.to_string())?;
        for invalid in [
            json!({"unrelated":true}),
            json!({"$id":"biomcp.health/6", "$schema":"unsupported"}),
            json!({"$id":"unknown.health/6", "$schema":"https://json-schema.org/draft/2020-12/schema"}),
            json!({"schema":"unknown/1", "status":"complete", "query":null, "result":{}}),
            json!({"schema":"unknown/1", "status":"usage", "error":{}}),
            json!({"schema":"biomcp.health/6", "status":"complete", "query":null, "result":[]}),
        ] {
            if validator.is_valid(&invalid) {
                return Err(
                    "output schema accepts an unrelated object or unsupported schema dialect"
                        .into(),
                );
            }
        }
    }
    let main = tools
        .iter()
        .find(|tool| tool.get("name") == Some(&json!("biomcp")))
        .and_then(|tool| tool.get("outputSchema"))
        .ok_or("missing main output schema")?;
    let main_validator = jsonschema::validator_for(main).map_err(|error| error.to_string())?;
    for (id, text) in super::super::schema::ARTIFACTS {
        let canonical: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        main_validator.validate(&canonical).map_err(|error| {
            format!("main tool rejects canonical schema artifact {id}: {error}")
        })?;
    }
    for args in [
        json!(["help"]),
        json!(["schema"]),
        json!(["invalid-command"]),
    ] {
        super::super::research::call(
            &Environment::new().map_err(|e| e.to_string())?,
            "biomcp",
            json!({"args":args}),
            &[],
        )?;
    }
    let env = Environment::new().map_err(|e| e.to_string())?;
    let health = super::super::research::call(
        &env,
        "biomcp",
        json!({"args":["health"]}),
        &["--offline".into()],
    )?;
    let complete = super::super::research::call(
        &env,
        "biomcp",
        json!({"args":["schema", "biomcp.health/6"]}),
        &[],
    )?;
    local_references(&complete, &complete)?;
    jsonschema::validator_for(&complete)
        .map_err(|e| e.to_string())?
        .validate(&health)
        .map_err(|e| e.to_string())?;
    writeln!(
        std::io::stdout(),
        "MCP_TOOLS_BYTES={}",
        serde_json::to_vec(reply).map_err(|e| e.to_string())?.len()
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn local_references(root: &Value, node: &Value) -> Result<(), String> {
    match node {
        Value::Object(fields) => {
            if let Some(reference) = fields.get("$ref").and_then(Value::as_str)
                && reference
                    .strip_prefix('#')
                    .and_then(|pointer| root.pointer(pointer))
                    .is_none()
            {
                return Err(format!(
                    "output schema reference is not locally resolvable: {reference}"
                ));
            }
            for value in fields.values() {
                local_references(root, value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                local_references(root, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
