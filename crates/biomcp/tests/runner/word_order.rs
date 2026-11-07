//! The published map names commands as typed, and reversed commands name the typed order.
use super::{launch::Environment, mcp};
use serde_json::{Value, json};

type Check<T = ()> = Result<T, String>;

pub(super) fn trial() -> libtest_mimic::Trial {
    super::transcript::trial("schema-discovery/command-word-order", || {
        check().map_err(Into::into)
    })
}

fn cli(env: &Environment, args: &[&str]) -> Check<(Option<i32>, Value)> {
    let args = args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>();
    let output = env
        .command(env!("CARGO_BIN_EXE_biomcp"), &args)
        .output()
        .map_err(|error| error.to_string())?;
    let reply = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("{args:?} did not reply with JSON: {error}"))?;
    Ok((output.status.code(), reply))
}

// Reads the installed skill the way an agent does: each row opens with its commands.
fn map_commands(env: &Environment) -> Check<Vec<String>> {
    let output = env
        .command(
            env!("CARGO_BIN_EXE_biomcp"),
            &["skill".to_owned(), "render".to_owned()],
        )
        .output()
        .map_err(|error| error.to_string())?;
    let skill = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let map = skill.split("\n\n").next().ok_or("skill has no map")?;
    Ok(map
        .lines()
        .take_while(|line| !line.starts_with("Sources/components"))
        .filter_map(|line| line.split(';').next())
        .flat_map(|head| head.split(','))
        .map(String::from)
        .collect())
}

fn single_mcp(env: &Environment, args: &[&str]) -> Check<Value> {
    let replies = mcp::exchange_with_globals(
        env,
        &[mcp::request(
            json!(1),
            "tools/call",
            json!({"name":"biomcp","arguments":{"args":args}}),
        )],
        &["--offline".into()],
    )?;
    replies
        .first()
        .and_then(|reply| reply.pointer("/result/structuredContent"))
        .cloned()
        .ok_or_else(|| format!("{args:?}: missing MCP reply"))
}

fn names_command(reply: &Value, verb: &str, entity: &str) -> Check {
    if reply.pointer("/error/code") == Some(&json!("usage.unknown_command"))
        && reply.pointer("/next/0/args") == Some(&json!(["help", verb, entity]))
        && reply
            .pointer("/next")
            .and_then(Value::as_array)
            .map(Vec::len)
            == Some(1)
    {
        return Ok(());
    }
    Err(format!(
        "{entity} {verb} must name {verb} {entity}: {reply}"
    ))
}

fn retired_names(reply: &Value, verb: &str) -> Check {
    let named = reply
        .pointer("/error/message")
        .and_then(Value::as_str)
        .is_some_and(|message| message.starts_with(&format!("use: {verb} gene ")));
    if named && reply.pointer("/error/code") == Some(&json!("usage.retired")) {
        return Ok(());
    }
    Err(format!("gene {verb} must name {verb} gene: {reply}"))
}

fn check() -> Check {
    let env = Environment::new().map_err(|error| error.to_string())?;
    let shown = map_commands(&env)?;
    for (verb, entity, _) in biomcp_types::command_paths() {
        let command = format!("{verb} {entity}");
        if !shown.contains(&command) {
            return Err(format!("map omits {command}: {shown:?}"));
        }
    }
    for command in &shown {
        let words = command.split(' ').collect::<Vec<_>>();
        let [verb, entity] = words.as_slice() else {
            return Err(format!("map command is not typed in full: {command}"));
        };
        let (code, reply) = cli(&env, &["help", verb, entity])?;
        if code != Some(0) || reply.pointer("/result/commands/0/command") != Some(&json!(words)) {
            return Err(format!("parser refused map command {command}: {reply}"));
        }
        let reversed = [*entity, *verb, "X"];
        let (code, reply) = cli(&env, &reversed)?;
        if code != Some(2) {
            return Err(format!("{reversed:?} must be a usage refusal: {reply}"));
        }
        // The retired gene root names the typed command in its fixed message.
        if *entity == "gene" {
            for reply in [reply, single_mcp(&env, &reversed)?] {
                retired_names(&reply, verb)?;
            }
            continue;
        }
        names_command(&reply, verb, entity)?;
        names_command(&single_mcp(&env, &reversed)?, verb, entity)?;
    }
    let (_, reply) = cli(&env, &["help", "variant", "get"])?;
    names_command(&reply, "get", "variant")?;
    typed_help()
}

fn typed_help() -> Check {
    let typed = Environment::new().map_err(|error| error.to_string())?;
    typed
        .config(&json!({"mcp_tool_mode":"typed"}))
        .map_err(|error| error.to_string())?;
    let replies = mcp::exchange_with_globals(
        &typed,
        &[mcp::request(
            json!(1),
            "tools/call",
            json!({"name":"biomcp_help","arguments":{"command":"variant get"}}),
        )],
        &["--offline".into()],
    )?;
    let reply = replies
        .first()
        .and_then(|reply| reply.pointer("/result/structuredContent"))
        .ok_or("missing typed help reply")?;
    if reply.pointer("/next/0/name") == Some(&json!("biomcp_help"))
        && reply.pointer("/next/0/arguments/command") == Some(&json!("get variant"))
    {
        return Ok(());
    }
    Err(format!("typed help must name get variant: {reply}"))
}
