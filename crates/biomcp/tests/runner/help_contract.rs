//! Generated help retains usable flag descriptions and entity navigation in both tool modes.
use super::{launch::Environment, mcp};
use serde_json::{Value, json};

pub(super) fn trial() -> libtest_mimic::Trial {
    super::transcript::trial("schema-discovery/help-contract", || {
        check().map_err(Into::into)
    })
}
fn check() -> Result<(), String> {
    let mut failures = Vec::new();
    for mode in ["single", "typed"] {
        let env = Environment::new().map_err(|e| e.to_string())?;
        env.config(&json!({"mcp_tool_mode":mode}))
            .map_err(|e| e.to_string())?;
        for path in [
            "search trial",
            "search adverse-event",
            "drug",
            "search pgx",
            "health",
        ] {
            let (name, arguments) = if mode == "single" {
                let mut args = vec!["help"];
                args.extend(path.split_whitespace());
                ("biomcp", json!({"args":args}))
            } else {
                ("biomcp_help", json!({"command":path}))
            };
            let replies = mcp::exchange_with_globals(
                &env,
                &[mcp::request(
                    json!(1),
                    "tools/call",
                    json!({"name":name,"arguments":arguments}),
                )],
                &["--offline".into()],
            )?;
            let value = replies
                .first()
                .and_then(|r| r.pointer("/result/structuredContent"))
                .ok_or("missing help reply")?;
            let commands = value.pointer("/result/commands").and_then(Value::as_array);
            let valid = match (path, commands) {
                ("drug", Some(commands)) => commands
                    .iter()
                    .any(|c| c.get("command") == Some(&json!(["search", "drug"]))),
                ("search trial" | "search adverse-event", Some(commands)) => descriptions(commands),
                ("search pgx", Some(commands)) => commands
                    .iter()
                    .flat_map(|c| {
                        c.get("arguments")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                    })
                    .any(|arg| {
                        arg.get("name") == Some(&json!("--source"))
                            && arg.get("required") == Some(&json!(true))
                    }),
                ("health", Some(commands)) => !commands.iter().any(|c| {
                    c.get("description")
                        .and_then(Value::as_str)
                        .is_some_and(|s| {
                            s.contains("timed out")
                                || s.contains("deadline exceeded")
                                || s.contains("refused this request")
                        })
                }),
                _ => false,
            };
            if !valid {
                failures.push(format!("{mode} help {path}: {value}"));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
fn descriptions(commands: &[Value]) -> bool {
    commands
        .first()
        .and_then(|c| c.get("arguments"))
        .and_then(Value::as_array)
        .is_some_and(|args| {
            args.iter().all(|arg| {
                arg.get("description")
                    .and_then(Value::as_str)
                    .is_some_and(|s| {
                        !s.is_empty()
                            && (arg
                                .get("choices")
                                .and_then(Value::as_array)
                                .is_none_or(|choices| choices.is_empty())
                                || arg.get("value_type") == Some(&json!("boolean"))
                                || s.contains("Listed choices accept any letter case"))
                            && (arg.get("name") != Some(&json!("--drug"))
                                || (s.contains("FAERS") && s.contains("repeat --drug")))
                    })
            })
        })
}
