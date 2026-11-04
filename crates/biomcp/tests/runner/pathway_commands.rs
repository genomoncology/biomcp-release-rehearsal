//! Feature pathway-reactome: retain a runtime pool token while changing caller selectors.
use super::launch::{Environment, fixtures};
use crate::runner::pathway_budgets as budgets;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    // The test mutates emitted pool arguments to check that foreign selectors cannot reuse a runtime snapshot.
    vec![transcript_trial(
        crate::runner::pool_contract::PATHWAY_SEARCH.0,
        || bindings().map_err(Into::into),
    )]
}
fn call(
    phase: &str,
    env: &Environment,
    args: &[String],
    globals: &[String],
    exit: i32,
) -> Result<Value, String> {
    let mut command = args.to_vec();
    command.extend_from_slice(globals);
    budgets::invoke(phase, env, command, exit)
}
fn bindings() -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replies = fixtures().join("replies/pathway/native");
    let globals = env.globals(Some(&replies), false);
    let first = call(
        "pathway_commands/bindings/1",
        &env,
        &["search".into(), "pathway".into(), "MAPK signaling".into()],
        &globals,
        0,
    )?;
    let page = first
        .get("next")
        .and_then(Value::as_array)
        .and_then(|steps| steps.iter().find(|s| s.get("kind") == Some(&json!("page"))))
        .and_then(|step| step.get("args"))
        .ok_or("missing native continuation")?;
    let page: Vec<String> = serde_json::from_value(page.clone()).map_err(|e| e.to_string())?;
    let mut foreign = page.clone();
    *foreign.get_mut(2).ok_or("missing text selector")? = "Signaling by EGFR".into();
    let rejected = call("pathway_commands/bindings/2", &env, &foreign, &globals, 3)?;
    if rejected.pointer("/error/code") != Some(&json!("pool.expired"))
        || rejected.pointer("/next/0/kind") != Some(&json!("restart"))
    {
        return Err(format!("foreign query reused retained page: {rejected}"));
    }
    let mut disabled = page.clone();
    disabled.extend(["--disable-sources".into(), "reactome".into()]);
    let rejected = call("pathway_commands/bindings/3", &env, &disabled, &globals, 4)?;
    if rejected.pointer("/sources/0/state") != Some(&json!("skipped"))
        || rejected.pointer("/sources/0/reason") != Some(&json!("disabled"))
    {
        return Err(format!(
            "disabled source restored retained page: {rejected}"
        ));
    }
    let next = call("pathway_commands/bindings/4", &env, &page, &globals, 0)?;
    if next.pointer("/result/page/offset") != Some(&json!(5))
        || next.pointer("/result/items/0/id/id") != Some(&json!("R-HSA-5674135"))
    {
        return Err(format!("native page lost its request binding: {next}"));
    }
    Ok(())
}
