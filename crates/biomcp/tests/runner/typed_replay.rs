#[path = "typed_replay_arguments.rs"]
mod arguments;
#[path = "typed_replay_volatile.rs"]
mod volatile;
use arguments::{input, json_globals, query_arguments};
// Reuse existing recorded caller cases without maintaining a second entity inventory.
use super::super::super::{
    case_data,
    launch::Environment,
    mcp::{exchange_with_globals, request},
    response_cases,
};
use serde_json::{Value, json};

#[derive(Clone, Copy)]
struct CaseContext<'a> {
    id: &'a str,
    label: &'a str,
    delayed_control: bool,
}

pub(super) fn check(tool: &Value, case_index: &mut usize) -> Result<(), String> {
    let name = tool
        .get("name")
        .and_then(Value::as_str)
        .ok_or("tool name")?;
    let (verb, entity, _) = biomcp_types::command_paths()
        .into_iter()
        .find(|(verb, entity, _)| format!("{verb}_{entity}").replace('-', "_") == name)
        .ok_or_else(|| format!("{name}: absent from query registry"))?;
    let mut paths = Vec::new();
    let cases = super::super::super::launch::fixtures().join("cases");
    case_data::collect(&cases, &mut paths).map_err(|e| e.to_string())?;
    paths.sort();
    let mut calls = 0;
    let mut excluded = 0;
    let mut failures = Vec::new();
    for path in paths {
        let case = case_data::read_case(&path)?;
        for (index, step) in case.steps.iter().enumerate() {
            let Some(args) = &step.run else { continue };
            let (query, _, _) = query_arguments(args);
            if query.first().map(String::as_str) != Some(verb)
                || query.get(1).map(String::as_str) != Some(entity)
            {
                continue;
            }
            let label = format!("{name} {} step {}", path.display(), index + 1);
            super::super::super::transcript::case_step(*case_index);
            use std::io::Write;
            writeln!(
                std::io::stdout(),
                "TYPED_REPLAY_CASE index={case_index} case={label}"
            )
            .map_err(|e| e.to_string())?;
            *case_index += 1;
            let id = path
                .strip_prefix(&cases)
                .map_err(|error| error.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            let context = CaseContext {
                id: &id,
                label: &label,
                delayed_control: false,
            };
            match candidate(tool, args, &case, index, context, &mut calls) {
                Ok(true) => {}
                Ok(false) => excluded += 1,
                Err(error) => failures.push(format!("{label}: {error}")),
            }
        }
    }
    if calls == 0 {
        failures.push(format!("{name}: no recorded typed call; no exemption"));
    }
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "TYPED_REPLAY tool={name} calls={calls} excluded={excluded} failures={}",
        failures.len()
    )
    .map_err(|e| e.to_string())?;
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn prepare(
    case: &case_data::Case,
    index: usize,
) -> Result<(Environment, Vec<String>, Value), String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    let (globals, config) = prepare_in(
        &env,
        case,
        index,
        std::path::Path::new(env!("CARGO_BIN_EXE_biomcp")),
    )?;
    Ok((env, globals, config))
}

fn prepare_in(
    env: &Environment,
    case: &case_data::Case,
    index: usize,
    binary: &std::path::Path,
) -> Result<(Vec<String>, Value), String> {
    let config = case.config.clone().unwrap_or_else(|| json!({}));
    env.config(&config).map_err(|e| e.to_string())?;
    let replay = response_cases::replay(env, case.replay.as_deref(), case.cache_acquired)?;
    let environment = env.using_executable(binary);
    let registry = super::super::super::schema::Registry::embedded()?;
    let mut previous = None;
    let mut previous_step = None;
    for step in case.steps.get(..index).ok_or("case step index")? {
        previous = Some(super::super::super::run_sequence(
            step,
            &environment,
            &Vec::new(),
            registry.as_ref(),
            replay.as_deref(),
            previous.as_ref(),
            previous_step,
        )?);
        previous_step = Some(step);
    }
    let step = case.steps.get(index).ok_or("case step index")?;
    // An explicit offline request must not receive the fixture's mutually exclusive replay mode.
    let offline = step.offline
        || step
            .run
            .as_deref()
            .is_some_and(|args| arguments::offline(args).unwrap_or(false));
    let globals = env.globals(replay.as_deref(), offline);
    Ok((globals, config))
}

fn candidate(
    tool: &Value,
    args: &[String],
    case: &case_data::Case,
    index: usize,
    context: CaseContext<'_>,
    calls: &mut usize,
) -> Result<bool, String> {
    let label = context.label;
    let (env, fixture_globals, config) = prepare(case, index)?;
    let (args, mut globals, process_validation) = query_arguments(args);
    globals.extend(fixture_globals);
    let step = case.steps.get(index).ok_or("case step index")?;
    let expected = cli_case(&env, &args, &globals, step)?;
    if !process_valid(process_validation, &expected, label)? {
        return Ok(false);
    }
    let query = query_schema(&env, &args)?;
    let Some(input) = input(&args, &query, &expected) else {
        if expected.get("status") != Some(&json!("usage")) {
            return Err(format!("valid CLI cannot translate: {args:?}"));
        }
        use std::io::Write;
        writeln!(std::io::stdout(), "TYPED_REPLAY_CLI_SYNTAX case={label} step={} args={args:?} reason={} \
             CLI_witness={expected} typed_coverage=typed_tools::rejections(unknown-key/non-object/malformed-value)",
            index + 1, expected.get("error").unwrap_or(&Value::Null)).map_err(|e| e.to_string())?;
        return Ok(false);
    };
    let schema_failure = schema_failure(tool, &input, &expected, label, index)?;
    let frozen_parity = single_parity(&env, case, index, &args, &globals, context);
    // Keep this comparison independent even when a frozen setup or call failed.
    reset_prepare(
        &env,
        case,
        index,
        std::path::Path::new(env!("CARGO_BIN_EXE_biomcp")),
    )?;
    let expected = cli_case(&env, &args, &globals, step)?;
    reset_prepare(
        &env,
        case,
        index,
        std::path::Path::new(env!("CARGO_BIN_EXE_biomcp")),
    )?;
    typed_config(&env, config)?;
    let expected = mcp_expected(&input, expected, &frozen_parity);
    let typed_parity = compare(
        tool,
        &input,
        &expected,
        (context.id, index + 1),
        &env,
        &globals,
        calls,
    );
    let mut failures = Vec::new();
    if let Some(error) = schema_failure {
        failures.push(error);
    }
    if let Err(error) = frozen_parity {
        failures.push(error);
    }
    if let Err(error) = typed_parity {
        failures.push(error);
    }
    if failures.is_empty() {
        Ok(true)
    } else {
        Err(failures.join("\n"))
    }
}

// README.md:61 and command/mcp_policy.rs forbid replacement in every MCP call.
fn schema_failure(
    tool: &Value,
    input: &Value,
    expected: &Value,
    label: &str,
    index: usize,
) -> Result<Option<String>, String> {
    let Err(error) = validate_input(tool, input) else {
        return Ok(None);
    };
    if expected.get("status") != Some(&json!("usage")) {
        return Ok(Some(error));
    }
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "TYPED_REPLAY_EXPECTED_INVALID case={label} step={} {error}; CLI_witness={expected}",
        index + 1
    )
    .map_err(|error| error.to_string())?;
    Ok(None)
}

fn query_schema(env: &Environment, args: &[String]) -> Result<Value, String> {
    let verb = args.first().ok_or("command verb")?;
    let entity = args.get(1).ok_or("command entity")?;
    let (_, _, id) = biomcp_types::command_paths()
        .into_iter()
        .find(|(v, e, _)| *v == verb.as_str() && *e == entity.as_str())
        .ok_or("registry command")?;
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &["schema".into(), id.into()])
        .output()
        .map_err(|error| error.to_string())?;
    serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())
}

fn mcp_expected(input: &Value, cli: Value, single: &Result<Value, String>) -> Value {
    let refusal = json!({"error":{"code":"usage.arguments",
        "message":"expected a supported command with checked arguments"},
        "schema":"biomcp.error/4","status":"usage"});
    if input.get("replace") == Some(&json!(true))
        && cli.pointer("/query/replace") == Some(&json!(true))
        && cli.get("status") == Some(&json!("complete"))
        && single.as_ref().is_ok_and(|reply| reply == &refusal)
    {
        refusal
    } else {
        cli
    }
}

fn typed_config(env: &Environment, mut config: Value) -> Result<(), String> {
    config
        .as_object_mut()
        .ok_or("configuration object")?
        .insert("mcp_tool_mode".into(), json!("typed"));
    env.config(&config).map_err(|e| e.to_string())
}

fn process_valid(
    validation: Result<(), String>,
    expected: &Value,
    label: &str,
) -> Result<bool, String> {
    let Err(error) = validation else {
        return Ok(true);
    };
    if expected.get("status") != Some(&json!("usage")) {
        return Err(error);
    }
    use std::io::Write;
    writeln!(
        std::io::stdout(),
        "TYPED_REPLAY_CLI_SYNTAX case={label} shared_process_parser={error}; \
         CLI_witness={expected}; coverage=same-global-parser-for-CLI-and-serve"
    )
    .map_err(|e| e.to_string())?;
    Ok(false)
}

fn cli_case(
    env: &Environment,
    args: &[String],
    globals: &[String],
    step: &case_data::Step,
) -> Result<Value, String> {
    let mut command = env.command(
        env!("CARGO_BIN_EXE_biomcp"),
        &super::super::super::launch::arguments(args, &json_globals(globals)),
    );
    let output = super::super::super::execution::output(&mut command, step.stdin.as_deref())?;
    if output.status.code() != Some(step.exit) {
        return Err(format!(
            "CLI exit changed: expected {}, actual {:?}: {args:?}",
            step.exit,
            output.status.code()
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
}

fn reset_prepare(
    env: &Environment,
    case: &case_data::Case,
    index: usize,
    binary: &std::path::Path,
) -> Result<(), String> {
    for entry in std::fs::read_dir(env.root()).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        }
        .map_err(|e| e.to_string())?;
    }
    for directory in ["home", "config", "cache"] {
        std::fs::create_dir(env.root().join(directory)).map_err(|e| e.to_string())?;
    }
    prepare_in(env, case, index, binary).map(|_| ())
}

fn single_parity(
    env: &Environment,
    case: &case_data::Case,
    index: usize,
    args: &[String],
    globals: &[String],
    context: CaseContext<'_>,
) -> Result<Value, String> {
    let frozen = super::super::super::launch::typed_baseline().ok_or("missing frozen baseline")?;
    let current = std::path::Path::new(env!("CARGO_BIN_EXE_biomcp"));
    let request_args =
        super::super::super::launch::arguments(args, &arguments::request_globals(globals));
    let frames = [request(
        json!(1),
        "tools/call",
        json!({"name":"biomcp","arguments":{"args":request_args}}),
    )];
    let mut server_args = vec!["serve".into()];
    server_args.extend_from_slice(globals);
    reset_prepare(env, case, index, &frozen)?;
    let old = super::baseline::exchange(env, &frozen, &server_args, &frames)?;
    reset_prepare(env, case, index, current)?;
    let new = super::baseline::exchange(env, current, &server_args, &frames)?;
    let mut approved = false;
    if old != new
        && old.len() == 1
        && new.len() == 1
        && let (Some(before), Some(after)) = (old.values().next(), new.values().next())
    {
        approved = volatile::raw(context.id, index + 1, before, after)?;
    }
    let mut failure = None;
    if (old != new && !approved) || context.delayed_control {
        if context.delayed_control {
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }
        reset_prepare(env, case, index, current)?;
        let control = super::baseline::exchange(env, current, &server_args, &frames)?;
        if context.delayed_control {
            tool_only! { volatile::clock(
                context.id,
                index + 1,
                new.get("1").ok_or("CURRENT clock frame")?,
                control.get("1").ok_or("CURRENT control clock frame")?,
            )?; }
        } else {
            failure = Some(if new == control {
                "frozen/current deterministic single RAW mismatch; exact bytes are in the case transcript".to_owned()
            } else {
                "CURRENT/CURRENT nondeterministic single RAW mismatch; exact bytes are in the case transcript; no fields exempted".to_owned()
            });
        }
    }
    match failure {
        Some(error) => Err(error),
        None => {
            let reply: Value = serde_json::from_str(new.get("1").ok_or("single reply")?)
                .map_err(|error| error.to_string())?;
            reply
                .pointer("/result/structuredContent")
                .cloned()
                .ok_or_else(|| "single structured reply".into())
        }
    }
}

fn compare(
    tool: &Value,
    input: &Value,
    expected: &Value,
    case: (&str, usize),
    env: &Environment,
    globals: &[String],
    calls: &mut usize,
) -> Result<(), String> {
    let name = tool
        .get("name")
        .and_then(Value::as_str)
        .ok_or("tool name")?;
    let mut typed_globals = globals.to_vec();
    typed_globals.extend(["--tool-mode".into(), "typed".into()]);
    let reply = exchange_with_globals(
        env,
        &[request(
            json!(1),
            "tools/call",
            json!({"name":name,"arguments":input}),
        )],
        &typed_globals,
    )?;
    let reply = reply.first().ok_or("missing typed reply")?;
    *calls += 1;
    let actual = reply
        .pointer("/result/structuredContent")
        .ok_or_else(|| format!("{name}: {reply}"))?;
    let checked = volatile::expected(case.0, case.1, expected, actual)?;
    let expected = checked.as_ref().unwrap_or(expected);
    if serde_json::to_vec(actual).map_err(|e| e.to_string())?
        != serde_json::to_vec(expected).map_err(|e| e.to_string())?
    {
        return Err(format!(
            "{name} decoded CLI JSON mismatch: {actual} versus {expected}"
        ));
    }
    super::super::super::schema::Registry::embedded()?.validate(
        actual,
        &[],
        expected.get("status") != Some(&json!("usage")),
    )?;
    if let Some(schema) = tool.get("outputSchema")
        && !jsonschema::validator_for(schema)
            .map_err(|e| e.to_string())?
            .is_valid(actual)
    {
        return Err(format!("{name} output schema mismatch"));
    }
    Ok(())
}

fn validate_input(tool: &Value, input: &Value) -> Result<(), String> {
    let schema = tool.get("inputSchema").ok_or("missing input schema")?;
    let validator = jsonschema::validator_for(schema).map_err(|e| e.to_string())?;
    validator
        .validate(input)
        .map_err(|e| format!("typed schema rejected recorded case: {e}; input={input}"))
}

pub(super) fn rejects_overstrict_schema(tool: &Value) -> Result<(), String> {
    let mut tool = tool.clone();
    tool.as_object_mut()
        .ok_or("tool object")?
        .insert("inputSchema".into(), json!(false));
    let error = match check(&tool, &mut 0) {
        Ok(()) => return Err("over-strict schema silently skipped recorded cases".into()),
        Err(error) => error,
    };
    if !error.contains("typed schema rejected recorded case") {
        return Err(format!(
            "over-strict replay failed for the wrong reason: {error}"
        ));
    }
    Ok(())
}

tool_only! {
// This diagnostic mode proves clock fields; it does not grant main replay allowances.
pub(super) fn clock_controls() -> Result<(), String> {
    let mut failures = Vec::new();
    let cases = super::super::super::launch::fixtures().join("cases");
    for (sequence, key) in volatile::clock_cases()?.iter().enumerate() {
        let (id, number) = key.rsplit_once(" step ").ok_or("clock case identifier")?;
        let index = number
            .parse::<usize>()
            .map_err(|error| error.to_string())?
            .checked_sub(1)
            .ok_or("clock step index")?;
        let case = case_data::read_case(&cases.join(id))?;
        let args = case
            .steps
            .get(index)
            .and_then(|step| step.run.as_ref())
            .ok_or("clock command")?;
        let (args, extra, validation) = query_arguments(args);
        validation?;
        if !biomcp_types::command_paths()
            .iter()
            .any(|(verb, entity, _)| {
                args.first().map(String::as_str) == Some(*verb)
                    && args.get(1).map(String::as_str) == Some(*entity)
            })
        {
            return Err(format!("clock case is not a registry query: {key}"));
        }
        super::super::super::transcript::case_step(sequence);
        let (env, fixture_globals, _) = prepare(&case, index)?;
        let mut globals = extra;
        globals.extend(fixture_globals);
        use std::io::Write;
        writeln!(
            std::io::stdout(),
            "TYPED_CLOCK_CASE index={sequence} case={key}"
        )
        .map_err(|error| error.to_string())?;
        let context = CaseContext {
            id,
            label: key,
            delayed_control: true,
        };
        if let Err(error) = single_parity(&env, &case, index, &args, &globals, context) {
            failures.push(format!("{key}: {error}"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

}
