//! Feature protein-search: dynamic pool bindings and lifetimes through the shipped command.
use super::{configuration::invoke, launch::Environment};
use serde_json::Value;

// Measure the ordinary CLI and MCP responses against the accepted external replay.
tool_only! {
pub(super) fn native(root: &std::path::Path) -> Result<(), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let globals = env.globals(Some(&root.join("replay")), false);
    for words in [
        vec!["get", "protein", "P00533"],
        vec!["get", "protein", "Q9UIF7"],
        vec!["get", "protein", "P38398-2"],
        vec!["get", "protein", "P42771", "Q8N726"],
        vec!["get", "protein", "BRAF"],
        vec!["get", "protein", "CDKN2A"],
        vec!["search", "protein", "--gene", "BRAF"],
        vec!["search", "protein", "--gene", "CDKN2A"],
        vec!["search", "protein", "EGFR"],
    ] {
        let args: Vec<String> = words.iter().map(|s| (*s).into()).collect();
        let cli = measure_native(&env, &globals, &args)?;
        if words == ["search", "protein", "EGFR"] {
            measure_native(&env, &globals, &super::action(&cli, "page")?)?;
        }
    }
    Ok(())
}
}
tool_only! {
fn measure_native(env: &Environment, globals: &[String], args: &[String]) -> Result<Value, String> {
    use std::io::Write;
    let mut command = globals.to_vec();
    command.extend_from_slice(args);
    let cli = invoke(env, command.clone(), 0)?;
    let mcp = super::mcp::call_with_globals(env, args, false, globals)?;
    for key in ["status", "result", "query", "page"] {
        if cli.get(key) != mcp.get(key) {
            return Err(format!("protein CLI/MCP {key} differs for {args:?}"));
        }
    }
    for (kind, value) in [("CLI", &cli), ("MCP", &mcp)] {
        let comparable = super::source_terms::size(value, "")?;
        writeln!(
            std::io::stderr(),
            "protein {kind} {args:?}: full={} comparable={comparable} ceiling={}",
            serde_json::to_vec(value).map_err(|e| e.to_string())?.len(),
            (comparable * 115).div_ceil(100)
        )
        .map_err(|e| e.to_string())?;
    }
    command.extend(["--format".into(), "md".into()]);
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &command)
        .output()
        .map_err(|e| e.to_string())?;
    let markdown = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let identities = cli
        .pointer("/result/items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| {
            item.pointer("/header/protein/accession")
                .or_else(|| item.pointer("/protein/accession"))
                .and_then(Value::as_str)
        });
    if !output.status.success()
        || markdown.trim().is_empty()
        || identities
            .into_iter()
            .any(|id| !markdown.contains(&super::source_terms::text::escape(id)))
    {
        return Err(format!("protein Markdown failed for {args:?}"));
    }
    Ok(cli)
}
}
