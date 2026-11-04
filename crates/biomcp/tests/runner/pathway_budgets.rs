//! Full-envelope command phases; CLI includes its newline and MCP counts compact JSON.
use super::launch::Environment;
use serde_json::Value;
pub(super) fn invoke(
    phase: &str,
    env: &Environment,
    args: Vec<String>,
    exit: i32,
) -> Result<Value, String> {
    let phase = phase_key(phase, &args);
    let value = super::configuration::invoke(env, args, exit)?;
    check(&value, &phase, "cli")?;
    Ok(value)
}
pub(super) fn mcp(
    phase: &str,
    env: &Environment,
    args: &[String],
    error: bool,
    globals: &[String],
) -> Result<Value, String> {
    let value = super::mcp::call_with_globals(env, args, error, globals)?;
    check(&value, &phase_key(phase, args), "mcp")?;
    Ok(value)
}
pub(super) fn check(value: &Value, phase: &str, surface: &str) -> Result<(), String> {
    let measurements: std::collections::BTreeMap<String, usize> = serde_saphyr::from_str(
        &crate::runner::launch::fixture_text("pathway-command-budgets.yaml")?,
    )
    .map_err(|e| e.to_string())?;
    let key = format!("{phase} [{surface}]");
    let measured = measurements
        .get(&key)
        .ok_or_else(|| format!("unmeasured pathway phase: {key}"))?;
    let max_bytes = (measured * 115).div_ceil(100);
    let bytes =
        serde_json::to_vec(value).map_err(|e| e.to_string())?.len() + usize::from(surface == "cli");
    if bytes > max_bytes {
        return Err(format!(
            "pathway {key}: {bytes} bytes exceeds max_bytes={max_bytes}"
        ));
    }
    Ok(())
}

fn phase_key(phase: &str, args: &[String]) -> String {
    let mut words = Vec::new();
    let mut args = args.iter();
    while let Some(word) = args.next() {
        match word.as_str() {
            "--pool" | "--replay" | "--record" | "--case-cache" => {
                args.next();
            }
            "--offline" => {}
            _ => words.push(word.as_str()),
        }
    }
    format!("{phase}: {}", words.join(" "))
}

// The routine gate retains actual help measurements beside the fixed case ceilings.
pub(super) fn help(value: &Value, args: &[String], surface: &str) -> Result<(), String> {
    if args.first().is_some_and(|word| word == "help")
        && (args.len() == 1
            || args
                .iter()
                .any(|word| matches!(word.as_str(), "pathway" | "wikipathways")))
    {
        use std::io::Write;
        let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?.len()
            + usize::from(surface == "cli");
        writeln!(
            std::io::stdout(),
            "PATHWAY_HELP_MEASURE {} {surface} {bytes}",
            args.join(" ")
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
