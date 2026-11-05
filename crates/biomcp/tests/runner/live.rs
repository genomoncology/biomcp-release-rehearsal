//! Live diagnostics are separate from replay assertions and never join the gate.
use super::{Case, Step, action, collect, launch::Environment, schema};
use biomcp_fetch::{Observation, Shape};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    time::{Duration, Instant},
};
const MAX_CASES: usize = 1024;
#[path = "live_compare.rs"]
mod comparison;
pub(super) use comparison::compare;
const CASE_SAMPLE: usize = 20;
#[path = "live_witness.rs"]
pub(super) mod witness;
const RUN_BUDGET: Duration = Duration::from_secs(3600);

pub(super) fn drift(before: &Shape, after: &Shape) -> Vec<String> {
    let mut changes = Vec::new();
    for key in before
        .fields
        .keys()
        .filter(|key| !after.incomplete && !after.fields.contains_key(*key))
    {
        changes.push(format!("field removed: {key}"));
    }
    for (key, positive) in &before.positive {
        if *positive && after.positive.get(key) == Some(&false) {
            changes.push(format!("count dropped to zero: {key}"));
        }
    }
    if before.header != after.header {
        changes.push("bundle header changed".into());
    }
    changes
}

pub(crate) fn run(list: bool, details: bool) -> Result<Value, String> {
    let root = super::launch::fixtures().join("cases");
    let mut paths = Vec::new();
    collect(&root, &mut paths).map_err(|e| e.to_string())?;
    paths.sort();
    if paths.len() > MAX_CASES {
        return Err("live case discovery limit exceeded".into());
    }
    // Review the whole inventory before any eligible case can contact a source.
    let cases = paths
        .into_iter()
        .map(|path| {
            let case = super::read_case(&path)?;
            Ok((path, case))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let registry = schema::Registry::embedded()?;
    let started = Instant::now();
    let mut results = Vec::new();
    let mut sources = BTreeMap::<String, Value>::new();
    for (path, case) in cases {
        let name = super::case_name(&path, &root)?;
        let skip = eligibility(&case);
        if let Some(reason) = skip {
            results.push(json!({"case": name, "skipped": reason}));
            continue;
        }
        if started.elapsed() >= RUN_BUDGET {
            results
                .push(json!({"case": name, "eligible": true, "skipped": "run budget exhausted"}));
            continue;
        }
        if list {
            results.push(json!({"case": name, "eligible": true}));
            continue;
        }
        let mut captures = Vec::new();
        let result = check(&case, &registry, &mut sources, started, &mut captures);
        let mut result = match result {
            Ok(report) => json!({"case": name, "result": report, "witness": captures}),
            Err(error) => json!({"case": name, "error": error, "witness": captures}),
        };
        witness::retain(std::slice::from_mut(&mut result));
        results.push(result);
    }
    if list {
        let eligible = results
            .iter()
            .filter(|row| row.get("eligible") == Some(&json!(true)))
            .count();
        return Ok(json!({"mode": "live-check-plan", "eligible": eligible, "cases": results}));
    }
    Ok(summary(
        results,
        sources,
        started.elapsed().as_millis(),
        details,
    ))
}
pub(super) fn summary(
    mut results: Vec<Value>,
    sources: BTreeMap<String, Value>,
    elapsed_ms: u128,
    details: bool,
) -> Value {
    witness::retain(&mut results);
    let mut skipped = BTreeMap::<String, usize>::new();
    for result in &results {
        if let Some(reason) = result.get("skipped").and_then(Value::as_str) {
            *skipped.entry(reason.into()).or_default() += 1;
        }
    }
    let mut checked: Vec<_> = results
        .iter()
        .filter(|r| r.get("skipped").is_none())
        .collect();
    let eligible = checked.len()
        + results
            .iter()
            .filter(|row| row.get("eligible") == Some(&json!(true)))
            .count();
    let drift_cases = checked.iter().filter(|r| has_drift(r)).count();
    let error_cases = checked.iter().filter(|r| r.get("error").is_some()).count();
    let retained = if details {
        MAX_CASES
    } else {
        CASE_SAMPLE.max(
            checked
                .iter()
                .filter(|row| has_drift(row) || row.get("error").is_some())
                .count(),
        )
    };
    checked.sort_by_key(|r| !(has_drift(r) || r.get("error").is_some()));
    json!(
        {"mode": "live-check","drift_cases": drift_cases,"error_cases": error_cases,"elapsed_ms": elapsed_ms,"sources": sources,
        "discovered": results.len(),"eligible": eligible,"checked": checked.len(),"skipped": skipped,
        "case_sample": checked.iter().take(retained).collect::<Vec<_>>(),
        "omitted": checked.len().saturating_sub(retained)}
    )
}
fn has_drift(result: &Value) -> bool {
    result
        .pointer("/result/drift")
        .and_then(Value::as_array)
        .is_some_and(|drift| !drift.is_empty())
}
fn eligibility(case: &Case) -> Option<&str> {
    match case._live_check.as_deref() {
        Some("public") if case.replay.is_some() && case.config.is_none() => {
            let skips = step_skips(case);
            if skips.iter().any(Option::is_none) {
                None
            } else if skips.contains(&Some("follow predecessor is ineligible")) {
                Some("follow predecessor is ineligible")
            } else {
                Some("configured, local-only or offline-only case")
            }
        }
        Some("public") => Some("configured, local-only or offline-only case"),
        Some(reason) => Some(reason),
        None => Some("provenance not reviewed for live use"),
    }
}
fn step_skips(case: &Case) -> Vec<Option<&'static str>> {
    let mut predecessor = false;
    case.steps
        .iter()
        .map(|step| {
            let reason = if !eligible_step(step) {
                Some("local validation, presentation or offline step")
            } else if step.follow.is_some() && !predecessor {
                Some("follow predecessor is ineligible")
            } else {
                None
            };
            predecessor = reason.is_none();
            reason
        })
        .collect()
}
fn eligible_step(step: &Step) -> bool {
    if step.offline || step.exit > 1 {
        return false;
    }
    if step.run.as_ref().or(step.mcp.as_ref()).is_some_and(|args| {
        args.windows(2).any(|pair| pair == ["--format", "md"])
            || args.iter().any(|arg| arg == "--format=md")
    }) {
        return false;
    }
    // The reviewed case declares its source operations. Do not maintain another
    // command list that silently drops entity actions or named MCP tools.
    step.follow.is_some() || step.run.is_some() || step.mcp.is_some() || step.mcp_call.is_some()
}
fn check(
    case: &Case,
    registry: &schema::Registry,
    sources: &mut BTreeMap<String, Value>,
    started: Instant,
    captures: &mut Vec<Value>,
) -> Result<Value, String> {
    let replay = super::launch::fixtures()
        .join("replies")
        .join(case.replay.as_ref().ok_or("no recording")?);
    let baseline = Environment::new().map_err(|e| e.to_string())?;
    let live = Environment::new().map_err(|e| e.to_string())?;
    let mut previous = (None, None);
    let mut changes = Vec::new();
    let mut ordinal = 0;
    let mut skipped = Vec::new();
    let mut commands = Vec::new();
    for (index, (step, skip)) in case.steps.iter().zip(step_skips(case)).enumerate() {
        if let Some(reason) = skip {
            previous = (None, None);
            skipped.push(json!({"step": index + 1, "reason": reason}));
            continue;
        }
        let mut args = (
            arguments(step, previous.0.as_ref())?,
            arguments(step, previous.1.as_ref())?,
        );
        let pages = step
            .walk
            .as_ref()
            .map_or(1, |walk| walk.max_pages.min(1000));
        for page in 0..pages {
            if started.elapsed() >= RUN_BUDGET {
                return Err("run budget exhausted".into());
            }
            ordinal += 1;
            if ordinal > 128 {
                return Err("case command budget exhausted".into());
            }
            let ((expected, before), reply) = witness::command(
                &baseline,
                &live,
                (&args.0, &args.1),
                &replay,
                ordinal,
                captures,
                step.mcp_call.as_ref(),
            )?;
            pace(&before, &reply, started);
            let (actual, after) = reply?;
            if step.mcp_call.is_none() && registry.validate(&actual, &args.1, true).is_err() {
                changes.push("output schema failure".into());
            }
            commands.push(command_report(
                ordinal,
                &expected,
                &actual,
                &before,
                &after,
                &mut changes,
                sources,
            ));
            previous = (Some(expected), Some(actual));
            if step.walk.is_none() {
                break;
            }
            let next = (
                action(previous.0.as_ref().ok_or("missing baseline")?, "page"),
                action(previous.1.as_ref().ok_or("missing live result")?, "page"),
            );
            match next {
                (Ok(before), Ok(after)) => args = (before, after),
                (Err(_), Err(_)) => break,
                _ => {
                    changes.push("page continuation changed".into());
                    break;
                }
            }
            if page + 1 == pages {
                changes.push("walk reached its bounded page count".into());
            }
        }
    }
    Ok(json!({"drift": changes, "skipped_steps": skipped, "commands": commands}))
}

fn pace(
    before: &[Observation],
    reply: &Result<(Value, Vec<Observation>), String>,
    started: Instant,
) {
    let observed = reply
        .as_ref()
        .ok()
        .map(|(_, records)| records.as_slice())
        .unwrap_or_default();
    let interval = before
        .iter()
        .chain(observed)
        .map(|o| o.interval_ms)
        .max()
        .unwrap_or(0);
    // Even a killed child may have sent a request just before it died.
    std::thread::sleep(
        Duration::from_millis(interval).min(RUN_BUDGET.saturating_sub(started.elapsed())),
    );
}
fn arguments(step: &Step, previous: Option<&Value>) -> Result<Vec<String>, String> {
    if step.mcp_call.is_some() {
        return Ok(Vec::new());
    }
    match &step.follow {
        Some(kind) => action(previous.ok_or("follow lacks predecessor")?, kind),
        None => step
            .run
            .as_ref()
            .or(step.mcp.as_ref())
            .cloned()
            .ok_or_else(|| "missing command".into()),
    }
}
fn invoke(
    environment: &Environment,
    args: &[String],
    replay: Option<&Path>,
    ordinal: usize,
    call: Option<&super::case_data::McpCall>,
) -> Result<(Value, Vec<Observation>), String> {
    let path = environment
        .root()
        .join(format!("observations-{ordinal}.jsonl"));
    let mut args = args.to_vec();
    let mut globals = environment.globals(replay, false);
    globals.extend(["--observe".into(), path.display().to_string()]);
    if let Some(call) = call {
        // The existing MCP helper validates the named tool's output schema.
        let value =
            super::research::call(environment, &call.name, call.arguments.clone(), &globals)?;
        return Ok((value, observations(environment, ordinal)?));
    }
    let position = args
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(args.len());
    args.splice(position..position, globals);
    let mut command = environment.command(env!("CARGO_BIN_EXE_biomcp"), &args);
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let (_, stdout, _) =
        super::process::Process::new(command.spawn().map_err(|e| e.to_string())?)?.finish()?;
    let value = serde_json::from_slice(&stdout).map_err(|_| "command did not return JSON")?;
    let observations = observations(environment, ordinal)?;
    Ok((value, observations))
}

fn observations(environment: &Environment, ordinal: usize) -> Result<Vec<Observation>, String> {
    let path = environment
        .root()
        .join(format!("observations-{ordinal}.jsonl"));
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if text.len() > 8 * 1024 * 1024 {
        return Err("observation file exceeds bound".into());
    }
    text.lines()
        .map(|line| serde_json::from_str(line).map_err(|e| e.to_string()))
        .collect()
}

pub(super) fn command_report(
    ordinal: usize,
    expected: &Value,
    actual: &Value,
    before: &[Observation],
    after: &[Observation],
    changes: &mut Vec<String>,
    sources: &mut BTreeMap<String, Value>,
) -> Value {
    if expected.get("status") != actual.get("status") {
        changes.push(format!(
            "command {ordinal}: output status changed: {:?} -> {:?}",
            expected.get("status"),
            actual.get("status")
        ));
    }
    let evidence = compare(before, after, changes, sources);
    json!({"ordinal": ordinal, "before_status": expected.get("status"), "after_status": actual.get("status"), "exchanges": evidence})
}
