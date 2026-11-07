//! Hosted discovery and follow-ups describe this server's actual authority.
//! live_check: local HTTP/stdio transport replay with recorded public replies; no live calls.
use super::super::{
    http::Server,
    launch::{Environment, fixtures},
    mcp::{exchange_with_globals, request},
};
use serde_json::{Value, json};

#[path = "hosted_surface/components.rs"]
mod components;

type Check = Result<(), String>;

pub(super) fn trials() -> Vec<libtest_mimic::Trial> {
    vec![crate::runner::transcript::trial(
        "mcp-hosted/served-surface-and-source-policy",
        || check().map_err(Into::into),
    )]
}

fn check() -> Check {
    let mut failures = Vec::new();
    for mode in ["single", "typed"] {
        for (claim, result) in [
            ("continuation", continuation(mode)),
            ("components", components::check(mode)),
            ("surface", surface(mode)),
        ] {
            if let Err(error) = result {
                failures.push(format!("{mode}/{claim}: {error}"));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

fn surface(mode: &str) -> Check {
    let (server, config) = server(
        mode,
        &["ctgov", "meddra", "gmdn", "reactome"],
        "replies/drug/openfda-label-page",
    )?;
    parser_recovery(&server, &config, mode)?;
    if mode == "typed" {
        let tools = catalog(&server)?;
        let gene = super::typed_catalog::command(&tools, "get_gene").ok_or("get_gene")?;
        let validator = jsonschema::validator_for(gene.get("inputSchema").ok_or("gene schema")?)
            .map_err(|e| e.to_string())?;
        if !validator.is_valid(&json!({"input":["BRAF"]}))
            || validator.is_valid(&json!({"input":["BRAF"],"section":["oncokb"]}))
        {
            return Err("hosted gene schema disagrees with supported input/OncoKB policy".into());
        }
    }
    let help = offered_help(&server, mode);
    let notices = source_notices(&server, mode);
    match (help, notices) {
        (Err(help), Err(notices)) => Err(format!("{help}\n{notices}")),
        (Err(error), _) | (_, Err(error)) => Err(error),
        _ => Ok(()),
    }
}

fn server(mode: &str, disabled: &[&str], replay: &str) -> Result<(Server, Environment), String> {
    let config = Environment::new().map_err(|e| e.to_string())?;
    config
        .config(
            &json!({"mcp_tool_mode":mode,"mcp_typed_output_schema":true,"hosted_max_use":"restricted","disable_sources":disabled}),
        )
        .map_err(|e| e.to_string())?;
    let server = Server::configured_globals(
        "127.0.0.1:0",
        None,
        Some(fixtures().join(replay)),
        None,
        None,
        &[
            "--hosted".into(),
            "--config".into(),
            config
                .root()
                .join("config/biomcp/config.json")
                .to_string_lossy()
                .into_owned(),
        ],
    )?;
    Ok((server, config))
}

fn parser_recovery(server: &Server, env: &Environment, mode: &str) -> Check {
    let invalid = params(
        mode,
        "article_text",
        &["article", "text", "--unexpected"],
        json!({"unexpected":true}),
    );
    recovery(&call(server, invalid.clone())?)?;
    let stdio = exchange_with_globals(
        env,
        &[request(json!(3), "tools/call", invalid)],
        &["--hosted".into(), "--offline".into()],
    )?;
    recovery(
        stdio
            .first()
            .and_then(|reply| reply.pointer("/result/structuredContent"))
            .ok_or("stdio error reply")?,
    )
}

fn offered_help(server: &Server, mode: &str) -> Check {
    for path in [
        "",
        "get gene",
        "get pathway",
        "get adverse-event",
        "search adverse-event",
        "sources",
        "config",
        "relations",
    ] {
        let reply = help(server, mode, path)?;
        let commands = reply
            .pointer("/result/commands")
            .and_then(Value::as_array)
            .ok_or("help commands")?;
        for command in commands {
            let words = match command.get("command") {
                None => &[][..],
                Some(value) => value.as_array().ok_or("help command words")?.as_slice(),
            };
            if matches!(
                words.first().and_then(Value::as_str),
                Some(
                    "health"
                        | "sync"
                        | "store"
                        | "version"
                        | "serve"
                        | "key"
                        | "mcp-config"
                        | "skill"
                )
            ) {
                return Err(format!("{mode}: unavailable command offered: {words:?}"));
            }
            if command.to_string().contains("--fresh") || command.to_string().contains("--config") {
                return Err(format!("{mode}: host-only option in help {words:?}"));
            }
            if path == "get pathway"
                && command
                    .get("arguments")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .any(|arg| {
                        arg.get("choices")
                            .and_then(Value::as_array)
                            .is_some_and(|choices| choices.contains(&json!("events")))
                    })
            {
                return Err(format!("{mode}: disabled Reactome events section offered"));
            }
        }
        if matches!(path, "sources" | "config") {
            let text = reply.to_string();
            if !text.contains("restricted")
                || !text.contains("operator")
                || !text.contains("self-hosted")
            {
                return Err(format!("{mode}: source help lacks server policy"));
            }
        }
        if path == "relations" {
            let text = reply.to_string();
            if text.contains("pathway (get section pathways)")
                || !text.contains("gene (get section genes)")
            {
                return Err(format!(
                    "{mode}: relation map loses source-specific availability: {reply}"
                ));
            }
        }
    }
    Ok(())
}

fn source_notices(server: &Server, mode: &str) -> Check {
    for (tool, args, input, expected) in [
        (
            "search_drug",
            vec![
                "search",
                "drug",
                "imatinib",
                "--source",
                "openfda-label",
                "--limit",
                "3",
            ],
            json!({"query":"imatinib","source":"openfda-label","limit":3}),
            "next",
        ),
        (
            "search_drug",
            vec!["search", "drug", "artemether", "--source", "who-pq"],
            json!({"query":"artemether","source":"who-pq"}),
            "restricted",
        ),
        (
            "search_trial",
            vec!["search", "trial", "--condition", "melanoma"],
            json!({"condition":["melanoma"]}),
            "operator",
        ),
        (
            "get_gene",
            vec!["get", "gene", "BRAF"],
            json!({"input":["BRAF"]}),
            "",
        ),
    ] {
        let reply = query(server, mode, tool, &args, input)?;
        if expected == "next"
            && reply
                .get("next")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty)
        {
            return Err(format!("{mode}: allowed follow-ups lost: {reply}"));
        }
        if !expected.is_empty()
            && expected != "next"
            && (!reply.to_string().contains(expected) || !reply.to_string().contains("self-hosted"))
        {
            return Err(format!(
                "{mode}: disabled source lacks {expected} explanation: {reply}"
            ));
        }
        offered_next(server, mode, &reply)?;
    }
    Ok(())
}

fn offered_next(server: &Server, mode: &str, reply: &Value) -> Check {
    if mode == "typed" {
        super::typed_followups::check_next(reply, &catalog(server)?, &mut 0)?;
    }
    for step in reply
        .get("next")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if mode == "typed" {
            let action = step
                .pointer("/arguments/action")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let fields = step
                .pointer(&format!("/arguments/{action}"))
                .or_else(|| step.get("arguments"))
                .and_then(Value::as_object)
                .ok_or("typed next arguments")?;
            if ["fresh", "config", "source_pacing"]
                .iter()
                .any(|key| fields.contains_key(*key))
            {
                return Err(format!("{mode}: unavailable next option: {step}"));
            }
        } else {
            let args = step
                .get("args")
                .and_then(Value::as_array)
                .ok_or("next args")?;
            if args.iter().any(|word| {
                matches!(
                    word.as_str(),
                    Some("--fresh" | "--config" | "sync" | "health")
                )
            }) {
                return Err(format!("{mode}: unavailable next: {step}"));
            }
        }
    }
    Ok(())
}

fn continuation(mode: &str) -> Check {
    let (server, _config) = server(
        mode,
        &["myvariant"],
        "replies/variant/civic-evidence-filters",
    )?;
    let reply = query(
        &server,
        mode,
        "get_variant",
        &[
            "get",
            "variant",
            "CA123643",
            "civic",
            "--section-limit",
            "2",
            "--evidence-level",
            "A",
            "--evidence-status",
            "submitted",
        ],
        json!({"input":"CA123643","section":["civic"],"section_limit":2,"evidence_level":["A"],"evidence_status":"submitted"}),
    )?;
    let page = reply
        .pointer("/result/candidates/0/sections/civic/data/profiles/0/evidence_page")
        .ok_or_else(|| format!("{mode}: CIViC page missing: {reply}"))?;
    for (pointer, expected) in [
        ("/total", json!(3)),
        ("/returned", json!(2)),
        ("/next/input", json!("CA123643")),
        ("/next/civic_profile", json!(12)),
        ("/next/civic_kind", json!("evidence")),
        ("/next/evidence_level", json!(["A"])),
        ("/next/civic_cursor", json!("0:E2")),
        ("/next/section", json!(["civic"])),
        ("/next/evidence_status", json!("submitted")),
    ] {
        if page.pointer(pointer) != Some(&expected) {
            return Err(format!(
                "{mode}: CIViC continuation changed at {pointer}: {page}"
            ));
        }
    }
    Ok(())
}

fn recovery(reply: &Value) -> Check {
    if reply.pointer("/error/code") != Some(&json!("usage.arguments")) {
        return Err(format!("parser refusal changed: {reply}"));
    }
    for value in reply
        .pointer("/error/valid_values")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if matches!(
            value.as_str(),
            Some(
                "save"
                    | "replace"
                    | "--save"
                    | "--replace"
                    | "--fresh"
                    | "--config"
                    | "--source_pacing"
            )
        ) {
            return Err(format!("hosted recovery offers refused option: {reply}"));
        }
    }
    Ok(())
}

fn catalog(server: &Server) -> Result<Vec<Value>, String> {
    let (_, reply) = server.request(
        "POST",
        "/mcp",
        &request(json!(2), "tools/list", json!({})).to_string(),
    )?;
    reply
        .pointer("/result/tools")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| format!("missing catalog: {reply}"))
}

fn help(server: &Server, mode: &str, path: &str) -> Result<Value, String> {
    let words = std::iter::once("help")
        .chain(path.split_whitespace())
        .collect::<Vec<_>>();
    query(server, mode, "help", &words, json!({"command":path}))
}

fn params(mode: &str, tool: &str, words: &[&str], input: Value) -> Value {
    if mode == "typed" {
        super::typed_catalog::entity(tool, input)
    } else {
        json!({"name":"biomcp","arguments":{"args":words}})
    }
}

fn query(
    server: &Server,
    mode: &str,
    tool: &str,
    words: &[&str],
    input: Value,
) -> Result<Value, String> {
    let params = if mode == "typed" {
        let tools = catalog(server)?;
        super::typed_catalog::command(&tools, tool).map_or_else(
            || params(mode, tool, words, input.clone()),
            |route| super::typed_catalog::call(&route, &input),
        )
    } else {
        params(mode, tool, words, input)
    };
    call(server, params)
}

fn call(server: &Server, params: Value) -> Result<Value, String> {
    let frame = request(json!(1), "tools/call", params);
    let (_, reply) = server.request("POST", "/mcp", &frame.to_string())?;
    reply
        .pointer("/result/structuredContent")
        .cloned()
        .ok_or_else(|| format!("missing tool reply to {frame}: {reply}"))
}
