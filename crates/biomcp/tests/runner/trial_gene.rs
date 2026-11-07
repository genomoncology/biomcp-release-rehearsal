//! The same trial notice expectations cross the single and typed MCP boundaries.
use super::{assertions, case_data, launch, mcp};
use crate::runner::transcript::trial;
use libtest_mimic::Trial;
use serde_json::json;

pub(super) fn trials() -> Vec<Trial> {
    ["single", "typed"]
        .into_iter()
        .map(|mode| {
            trial(
                format!("trial-search/trial/gene-notices-mcp-{mode}-mode"),
                move || check(mode).map_err(Into::into),
            )
        })
        .collect()
}

fn check(mode: &str) -> Result<(), String> {
    let fixtures = launch::fixtures();
    let path = if mode == "typed" {
        "cases/trial/gene-notices-typed.json"
    } else {
        "cases/trial/gene-notices.yaml"
    };
    let case = case_data::read_case(&fixtures.join(path))?;
    let step = case.steps.first().ok_or("missing trial case step")?;
    let env = launch::Environment::new().map_err(|e| e.to_string())?;
    case.config.as_ref().ok_or("missing tool mode")?;
    env.case_config(&case).map_err(|e| e.to_string())?;
    let parameters = if let Some(call) = &step.mcp_call {
        json!({"name": call.name, "arguments": call.arguments})
    } else {
        json!({"name":"biomcp", "arguments":{"args":step.run}})
    };
    let replay = fixtures
        .join("replies")
        .join(case.replay.as_ref().ok_or("missing replay")?);
    let replies = mcp::exchange_with_globals(
        &env,
        &[mcp::request(json!(1), "tools/call", parameters)],
        &env.globals(Some(&replay), false),
    )?;
    let result = replies
        .iter()
        .find(|row| row.get("id") == Some(&json!(1)))
        .and_then(|row| row.get("result"))
        .ok_or("missing trial tool result")?;
    if result.get("isError") != Some(&json!(false)) {
        return Err(format!("{mode} trial search failed: {result}"));
    }
    let value = result
        .get("structuredContent")
        .ok_or("missing structured trial result")?;
    let expected = assertions::prepare(step)?;
    assertions::check(
        &serde_json::to_vec(value).map_err(|e| e.to_string())?,
        step,
        &expected,
    )
}
