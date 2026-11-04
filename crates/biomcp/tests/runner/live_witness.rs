//! Keep bounded diagnostic context only when a case drifts or fails.
use super as live;
use super::super::launch::Environment;
use biomcp_fetch::{Observation, Shape};
use biomcp_types::Use;
use serde_json::{Value, json};
use std::path::Path;

/// Stop a case before another command when its retained evidence exceeds this budget.
/// The final command remains whole, so an error never destroys the evidence it reports.
const MAX_CASE_BYTES: usize = 16 * 1024 * 1024;

pub(in crate::runner) fn within_budget(captures: &[Value]) -> Result<(), String> {
    let bytes = serde_json::to_vec(captures).map_err(|error| error.to_string())?;
    if bytes.len() > MAX_CASE_BYTES {
        return Err("case evidence budget exhausted; complete commands retained".into());
    }
    Ok(())
}

pub(super) fn retain(results: &mut [Value]) {
    for result in results {
        let wanted = live::has_drift(result) || result.get("error").is_some();
        if !wanted && let Some(map) = result.as_object_mut() {
            map.remove("witness");
        }
    }
}

type Reply = (Value, Vec<Observation>);
pub(super) fn command(
    baseline: &Environment,
    live: &Environment,
    args: (&[String], &[String]),
    replay: &Path,
    ordinal: usize,
    captures: &mut Vec<Value>,
    call: Option<&super::super::case_data::McpCall>,
) -> Result<(Reply, Result<Reply, String>), String> {
    within_budget(captures).map_err(|error| format!("{error}; command {ordinal} was not run"))?;
    captures.push(json!({"ordinal":ordinal}));
    let capture = captures.last_mut().ok_or("missing command capture")?;
    let expected = invoke_call(
        baseline,
        args.0,
        Some(replay),
        ordinal,
        capture,
        "baseline",
        call,
    )?;
    if ordinal == 1 && expected.1.is_empty() {
        return Err("recorded command has no provider observations".into());
    }
    let actual = invoke_call(live, args.1, None, ordinal, capture, "live", call);
    within_budget(captures)
        .map_err(|error| format!("{error}; stopped after complete command {ordinal}"))?;
    Ok((expected, actual))
}

pub(in crate::runner) fn invoke(
    environment: &Environment,
    args: &[String],
    replay: Option<&Path>,
    ordinal: usize,
    capture: &mut Value,
    side: &str,
) -> Result<(Value, Vec<Observation>), String> {
    invoke_call(environment, args, replay, ordinal, capture, side, None)
}

fn invoke_call(
    environment: &Environment,
    args: &[String],
    replay: Option<&Path>,
    ordinal: usize,
    capture: &mut Value,
    side: &str,
    call: Option<&super::super::case_data::McpCall>,
) -> Result<(Value, Vec<Observation>), String> {
    let result = live::invoke(environment, args, replay, ordinal, call);
    let context = match &result {
        Ok((output, observations)) => context(output, observations),
        Err(error) => {
            let observations = live::observations(environment, ordinal).unwrap_or_default();
            json!({"error":error,"requests":observations})
        }
    };
    if let Some(map) = capture.as_object_mut() {
        map.insert(side.into(), context);
    }
    result
}

pub(in crate::runner) fn context(output: &Value, observations: &[Observation]) -> Value {
    // Open-only retention is conservative for mixed, restricted, unknown and
    // noncommercial terms. Never retain raw provider bodies or any headers.
    let permitted = observations.iter().all(|observation| {
        biomcp_query::sources::terms(observation.source)
            .is_some_and(|terms| terms.usage == Use::Open)
    }) && open_labels(output);
    let mut ids = Vec::new();
    identities(output, &mut ids, 0);
    json!({"requests":observations,"result_ids":ids,
        "output_shape":Shape::inspect(output.to_string().as_bytes(),"application/json"),
        "output":if permitted {output.clone()} else {json!({"omitted":"source terms require shape and identities only"})}})
}

fn open_labels(value: &Value) -> bool {
    match value {
        Value::Object(map) => map.iter().all(|(key, value)| {
            (!matches!(key.as_str(), "use" | "usage") || value.as_str() == Some("open"))
                && open_labels(value)
        }),
        Value::Array(items) => items.iter().all(open_labels),
        _ => true,
    }
}

fn identities(value: &Value, ids: &mut Vec<Value>, depth: usize) {
    if depth > 12 || ids.len() >= 64 {
        return;
    }
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if matches!(
                    key.as_str(),
                    "id" | "ids"
                        | "accession"
                        | "pmid"
                        | "pmcid"
                        | "doi"
                        | "gene"
                        | "symbol"
                        | "pma_number"
                        | "supplement_number"
                ) || key.ends_with("_id")
                    || key.ends_with("_ids")
                {
                    identifier_values(key, item, ids);
                }
                identities(item, ids, depth + 1);
            }
        }
        Value::Array(items) => {
            for item in items.iter().take(64) {
                identities(item, ids, depth + 1);
            }
        }
        _ => {}
    }
}

fn identifier_values(key: &str, item: &Value, ids: &mut Vec<Value>) {
    if ids.len() >= 64 {
        return;
    }
    if (item.is_string() || item.is_number()) && item.to_string().len() <= 256 {
        ids.push(json!({"field":key,"value":item}));
    } else if let Some(values) = item.as_array() {
        for value in values.iter().take(64) {
            if (value.is_string() || value.is_number())
                && ids.len() < 64
                && value.to_string().len() <= 256
            {
                ids.push(json!({"field":key,"value":value}));
            }
        }
    }
}
