//! Execute canonical case steps and assert their actual emitted bytes.
use super::{Step, Value, assertions, launch, mcp, pathway_budgets, schema};
use std::{io::Write, process::Stdio};
pub(super) fn run_step(
    step: &Step,
    environment: &launch::Environment,
    expectations: &assertions::Expectations<'_>,
    registry: &schema::Registry,
    replay: Option<&std::path::Path>,
    paired_json: Option<&Value>,
) -> Result<Value, String> {
    let globals = environment.globals(replay, step.offline);
    let cli = step
        .run
        .as_ref()
        .map(|args| {
            cli(
                args,
                step,
                environment,
                expectations,
                registry,
                &globals,
                paired_json,
            )
        })
        .transpose()?;
    let mut answer = cli.clone();
    if let Some(args) = &step.mcp {
        let (result, text) =
            mcp::call_text_with_globals(environment, args, step.is_error, &globals)?;
        registry.validate(&result, args, !step.is_error)?;
        super::reply_readback::check(text.as_bytes(), &result, true)?;
        pathway_budgets::help(&result, args, "mcp")?;
        assertions::check(text.as_bytes(), step, expectations)?;
        if step.full_output_bytes {
            writeln!(
                std::io::stderr(),
                "RAW_OUTPUT mcp-text {:?} {}",
                args,
                text.len()
            )
            .map_err(|e| e.to_string())?;
        }
        answer = Some(result.clone());
        if let Some(cli) = cli.as_ref() {
            parity(cli, &result, args)?;
        }
    }
    if let Some(call) = &step.mcp_call {
        answer = Some(named(call, step, environment, expectations, &globals)?);
    }
    answer.ok_or_else(|| "step requires run, mcp or mcp_call".into())
}

fn cli(
    args: &[String],
    step: &Step,
    environment: &launch::Environment,
    expectations: &assertions::Expectations<'_>,
    registry: &schema::Registry,
    globals: &[String],
    paired_json: Option<&Value>,
) -> Result<Value, String> {
    let command_args = launch::arguments(args, globals);
    let mut command = environment.command(env!("CARGO_BIN_EXE_biomcp"), &command_args);
    let output = output(&mut command, step.stdin.as_deref())?;
    if output.status.code() != Some(step.exit) {
        return Err(format!(
            "exit: expected {}, actual {:?}; command: {:?}; stderr: {}; stdout: {}",
            step.exit,
            output.status.code(),
            args,
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(2000)
                .collect::<String>()
        ));
    }
    if !output.stderr.is_empty() {
        return Err("error output: expected empty stderr".into());
    }
    if assertions::markdown(step) {
        assertions::check_markdown(&output.stdout, step, expectations, paired_json)?;
    }
    if let Some(expected) = &step.stdout {
        if serde_json::from_slice::<Value>(&output.stdout).is_ok() {
            return Err("literal stdout requires plain text, not a JSON reply".into());
        }
        if output.stdout != expected.as_bytes() {
            return Err(format!(
                "stdout: expected {expected:?}, actual {:?}",
                String::from_utf8_lossy(&output.stdout)
            ));
        }
        return Ok(Value::String(expected.clone()));
    }
    if assertions::markdown(step) {
        return String::from_utf8(output.stdout)
            .map(Value::String)
            .map_err(|error| error.to_string());
    }
    let value =
        serde_json::from_slice::<Value>(&output.stdout).map_err(|error| error.to_string())?;
    if value.get("schema").is_none()
        && (args.first().is_some_and(|arg| arg == "mcp-config")
            || args.first().is_some_and(|arg| arg == "skill")
                && args
                    .get(1)
                    .is_some_and(|arg| matches!(arg.as_str(), "install" | "status")))
    {
        assertions::check(&output.stdout, step, expectations)?;
        return Ok(value);
    }
    registry.validate(&value, args, output.status.success())?;
    super::reply_readback::check(&output.stdout, &value, false)?;
    pathway_budgets::help(&value, args, "cli")?;
    assertions::check(&output.stdout, step, expectations)?;
    if step.full_output_bytes {
        writeln!(
            std::io::stderr(),
            "RAW_OUTPUT cli {:?} {}",
            args,
            output.stdout.len()
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(value)
}
fn output(
    command: &mut super::capture::Capture,
    input: Option<&str>,
) -> Result<std::process::Output, String> {
    if let Some(input) = input {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| error.to_string())?;
        child
            .stdin
            .take()
            .ok_or("missing CLI input pipe")?
            .write_all(input.as_bytes())
            .map_err(|error| error.to_string())?;
        command.wait_with_output(child)
    } else {
        command.output()
    }
    .map_err(|error| error.to_string())
}

fn parity(cli: &Value, result: &Value, args: &[String]) -> Result<(), String> {
    let mut comparable = result.clone();
    // Repeated explicit syncs may acquire the same source in different seconds.
    if args.first().is_some_and(|verb| verb == "sync")
        && let Some(acquired) = cli.pointer("/result/snapshot/acquired")
        && let Some(value) = comparable.pointer_mut("/result/snapshot/acquired")
    {
        *value = acquired.clone();
    }
    if let Some(diff) = assertions::difference(cli, &comparable, "") {
        return Err(format!("CLI/MCP parity: unequal results at {diff}"));
    }
    Ok(())
}

fn named(
    call: &super::case_data::McpCall,
    step: &Step,
    environment: &launch::Environment,
    expectations: &assertions::Expectations<'_>,
    globals: &[String],
) -> Result<Value, String> {
    let value = super::research::call_checked(
        environment,
        &call.name,
        call.arguments.clone(),
        globals,
        Some(step.is_error),
    )?;
    assertions::check(
        &serde_json::to_vec(&value).map_err(|e| e.to_string())?,
        step,
        expectations,
    )?;
    Ok(value)
}
