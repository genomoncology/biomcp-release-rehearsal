//! Actual research tools retain source identity and the bounded CLI record.
use super::{Environment, Value, Write, budget, json, path};
pub(super) fn run(measure: bool) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let replay = super::super::launch::fixtures().join("replies/cell-line/examples");
    let globals = env.globals(Some(&replay), false);
    let searched = super::super::research::call(
        &env,
        "search",
        json!({"query":"cell-line MOLM13"}),
        &globals,
    )?;
    measure_output("research/search/MOLM13", &searched, measure)?;
    let hits = searched
        .get("results")
        .and_then(Value::as_array)
        .ok_or("research results")?;
    if hits.len() != 1 {
        return Err("research source order/count changed".into());
    }
    let hit = hits.first().ok_or("research hit")?;
    path(hit, "/title", json!("MOLM-13"))?;
    path(hit, "/url", json!("https://www.cellosaurus.org/CVCL_2119"))?;
    path(
        &searched,
        "/metadata/envelope/result/items/0/accession",
        json!("CVCL_2119"),
    )?;
    let id = hit
        .get("id")
        .and_then(Value::as_str)
        .ok_or("research fetch identity")?;
    let fetched = super::super::research::call(&env, "fetch", json!({"id":id}), &globals)?;
    measure_output("research/fetch/CVCL_2119", &fetched, measure)?;
    path(
        &fetched,
        "/url",
        json!("https://www.cellosaurus.org/CVCL_2119"),
    )?;
    path(&fetched, "/title", json!("MOLM-13"))?;
    let mut args = vec!["get".to_string()];
    args.extend(shlex::split(id).ok_or("opaque fetch args")?);
    let card = super::super::trial_get::command(&env, &args, &globals, 0)?;
    if fetched.get("metadata") != Some(&super::super::research::fetch_metadata(&card)) {
        return Err("research fetch differs from exact CLI card".into());
    }
    if fetched
        .get("text")
        .and_then(Value::as_str)
        .is_none_or(|text| !text.contains("MOLM-13") || !text.contains("CVCL_2119"))
    {
        return Err("research fetch lost its readable cell-line card".into());
    }
    Ok(())
}
fn measure_output(phase: &str, value: &Value, measure: bool) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len();
    writeln!(std::io::stderr(), "CELL_LINE_MEASURE {phase} [mcp] {bytes}")
        .map_err(|e| e.to_string())?;
    if !measure {
        budget(phase, "mcp", value)?;
    }
    Ok(())
}
