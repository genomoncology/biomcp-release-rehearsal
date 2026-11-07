//! Hosted adverse-event capabilities follow native component admission.
use super::{Check, Server, catalog, help, query, recovery, server};
use serde_json::{Value, json};

pub(super) fn check(mode: &str) -> Check {
    let (server, _config) = server(
        mode,
        &["meddra", "gmdn"],
        "replies/adverse-event/faers-synthetic",
    )?;
    if mode == "typed" {
        component_catalog(&server)?;
    }
    component_help(&server, mode)?;
    component_recovery(&server, mode)?;
    component_card(&server, mode)?;
    aggregate_component(&server, mode)
}

fn component_catalog(server: &Server) -> Check {
    let tools = catalog(server)?;
    let route = super::super::typed_catalog::command(&tools, "get_adverse_event")
        .ok_or("get adverse event action")?;
    let entity = tools
        .iter()
        .find(|tool| tool["name"] == "adverse_event")
        .ok_or("adverse event tool")?;
    let schema = entity.get("inputSchema").ok_or("adverse event schema")?;
    let validator = jsonschema::validator_for(schema).map_err(|error| error.to_string())?;
    for input in [
        json!({"id":"100","source":"faers","section":"drugs"}),
        json!({"id":"123","source":"maude","section":"narrative"}),
    ] {
        let call = super::super::typed_catalog::call(&route, &input);
        if !validator.is_valid(call.get("arguments").ok_or("entity arguments")?) {
            return Err("component-free report call disappeared".into());
        }
    }
    // Each action's schema filters component-disabled sections.
    if ["reactions", "devices"]
        .iter()
        .any(|section| schema.to_string().contains(&format!("\"{section}\"")))
    {
        return Err("adverse event schema advertises a component-disabled section".into());
    }
    let search = super::super::typed_catalog::command(&tools, "search_adverse_event")
        .ok_or("search adverse event")?;
    let validator = jsonschema::validator_for(search.get("inputSchema").ok_or("search schema")?)
        .map_err(|e| e.to_string())?;
    if !validator.is_valid(&json!({"text":"Synthetic medicine","source":"faers"}))
        || validator.is_valid(&json!({"text":"device","source":"maude"}))
        || search.pointer("/inputSchema/properties/reaction").is_some()
        || ["aggregate", "manufacturer", "product_code", "problem"]
            .iter()
            .any(|field| {
                search
                    .pointer(&format!("/inputSchema/properties/{field}"))
                    .is_some()
            })
    {
        return Err("search component capabilities disagree with admission".into());
    }
    Ok(())
}

fn component_help(server: &Server, mode: &str) -> Check {
    for path in ["get adverse-event", "search adverse-event"] {
        let help = help(server, mode, path)?;
        let commands = help
            .pointer("/result/commands")
            .and_then(Value::as_array)
            .filter(|commands| !commands.is_empty())
            .ok_or_else(|| format!("{mode}: missing component help: {help}"))?;
        for command in commands {
            for section in command
                .get("sections")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if matches!(
                    section.get("word").and_then(Value::as_str),
                    Some("reactions" | "devices")
                ) {
                    return Err(format!("{mode}: component-disabled section in help"));
                }
            }
            for argument in command
                .get("arguments")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let choices = argument.get("choices").and_then(Value::as_array);
                let section = choices.is_some_and(|choices| {
                    choices
                        .iter()
                        .any(|word| matches!(word.as_str(), Some("reactions" | "devices")))
                });
                let source = path == "search adverse-event"
                    && choices.is_some_and(|choices| choices.contains(&json!("maude")));
                if matches!(
                    argument.get("name").and_then(Value::as_str),
                    Some(
                        "--reaction"
                            | "--aggregate"
                            | "--manufacturer"
                            | "--product_code"
                            | "--problem"
                    )
                ) || section
                    || source
                {
                    return Err(format!(
                        "{mode}: component-disabled option in help: {argument}"
                    ));
                }
            }
        }
    }
    Ok(())
}

fn component_recovery(server: &Server, mode: &str) -> Check {
    for (tool, args, input) in [
        (
            "search_adverse_event",
            vec!["search", "adverse-event", "x", "--source", "bogus"],
            json!({"text":"x","source":"bogus"}),
        ),
        (
            "get_adverse_event",
            vec!["get", "adverse-event", "100", "--source", "faers", "bogus"],
            json!({"id":"100","source":"faers","section":"bogus"}),
        ),
    ] {
        let reply = query(server, mode, tool, &args, input)?;
        recovery(&reply)?;
        if reply
            .pointer("/error/valid_values")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .any(|word| matches!(word.as_str(), Some("maude" | "reactions" | "devices")))
        {
            return Err(format!(
                "{mode}: recovery offers a component-disabled choice: {reply}"
            ));
        }
    }
    Ok(())
}

fn aggregate_component(server: &Server, mode: &str) -> Check {
    for aggregate in ["bogus", "reactions"] {
        let reply = query(
            server,
            mode,
            "search_adverse_event",
            &[
                "search",
                "adverse-event",
                "Synthetic medicine",
                "--aggregate",
                aggregate,
            ],
            json!({"text":"Synthetic medicine","aggregate":aggregate}),
        )?;
        if aggregate == "bogus" {
            recovery(&reply)?;
            if reply.pointer("/error/valid_values").is_some() {
                return Err(format!("disabled aggregation offered in recovery: {reply}"));
            }
        } else if reply.pointer("/sources/0/reason") != Some(&json!("disabled"))
            || !reply.to_string().contains("operator")
        {
            return Err(format!(
                "aggregation component refusal/cause changed: {reply}"
            ));
        }
    }
    Ok(())
}

fn component_card(server: &Server, mode: &str) -> Check {
    let card = query(
        server,
        mode,
        "get_adverse_event",
        &["get", "adverse-event", "100"],
        json!({"id":"100"}),
    )?;
    if card.get("status") != Some(&json!("complete")) {
        return Err(format!("card failed: {card}"));
    }
    if card
        .get("next")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|step| {
            step.get("args")
                .and_then(Value::as_array)
                .is_some_and(|args| args.contains(&json!("reactions")))
                || step.pointer("/arguments/section") == Some(&json!("reactions"))
                || step.pointer("/arguments/get/section") == Some(&json!("reactions"))
        })
    {
        return Err("disabled reaction section offered by next".into());
    }
    for (source, section) in [("faers", "reactions"), ("maude", "devices")] {
        let reply = query(
            server,
            mode,
            "get_adverse_event",
            &["get", "adverse-event", "100", section, "--source", source],
            json!({"id":"100","source":source,"section":section}),
        )?;
        if reply.pointer("/sources/0/reason") != Some(&json!("disabled"))
            || !reply.to_string().contains("operator")
        {
            return Err(format!("component refusal/cause changed: {reply}"));
        }
    }
    if mode == "single" {
        let reply = query(
            server,
            mode,
            "get_adverse_event",
            &["get", "adverse-event", "100", "--disable-sources", "faers"],
            json!({"id":"100"}),
        )?;
        let message = reply
            .pointer("/sources/0/notices")
            .and_then(Value::as_array)
            .and_then(|notices| notices.last())
            .and_then(|notice| notice.get("message"))
            .and_then(Value::as_str)
            .ok_or("request exclusion notice")?;
        if !message.contains("request")
            || message.contains("operator")
            || message.contains("ceiling")
        {
            return Err(format!(
                "unrelated MedDRA exclusion blamed for caller's card exclusion: {reply}"
            ));
        }
    }
    Ok(())
}
