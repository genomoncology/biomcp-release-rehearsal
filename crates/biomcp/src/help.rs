//! Schema-fed command help shared with MCP discovery.

#[path = "../coverage.rs"]
pub(crate) mod coverage;
mod discovery;
pub(crate) const FRONT_MAP: &str = include_str!(concat!(env!("OUT_DIR"), "/front-map.txt"));
mod schema_fields;
use schema_fields::{resolve, value_type};
mod public_grammar;
mod source_settings;

use biomcp_types::{Help, HelpArgument, HelpCommand, HelpWord};
use serde_json::Value;
use std::{collections::BTreeMap, io};

pub(crate) fn dispatch(root: &crate::grammar::Root, path: &[String]) -> io::Result<Help> {
    if path == ["get", "clingen"] {
        let mut help = dispatch(root, &["get".into(), "gene".into()])?;
        if let Some(command) = help.commands.first_mut() {
            command
                .description
                .insert_str(0, "ClinGen: get gene ID clingen. ");
            if let Some(example) = command.examples.first_mut() {
                example.push("clingen".into());
            }
        }
        return Ok(help);
    }
    if let [topic] = path
        && matches!(topic.as_str(), "sources" | "config")
    {
        return Ok(source_settings::help(topic));
    }
    if path == ["network"] {
        let mut help = network("ca_bundle");
        help.commands.extend(network("proxy").commands);
        return Ok(help);
    }
    if let [category, setting] = path
        && category == "network"
    {
        return Ok(network(setting));
    }
    let mut commands: Vec<_> = commands(root)?
        .into_iter()
        .filter(|c| {
            path.is_empty()
                || (path == ["help"] && c.command.is_empty())
                || c.command.starts_with(path)
                || (path.len() == 1
                    && path.first().is_some_and(|word| word == "skill")
                    && c.command.first() == path.first())
                || (path.len() == 1 && c.command.get(1) == path.first())
                || (path.len() == 1
                    && path.first().is_some_and(|word| word == "author")
                    && c.command.first() == path.first())
        })
        .collect();
    if !public_grammar::overview(root, &mut commands, path)? {
        compact(&mut commands);
    }
    if path.is_empty() {
        let mut map = topic("map", FRONT_MAP.into());
        map.command.clear();
        commands.insert(0, map);
    }
    Ok(Help { commands })
}
// Large catalogs remain available through the command's complete query schema.
const INLINE_CHOICE_LIMIT: usize = 24;
fn compact(commands: &mut [HelpCommand]) {
    for command in commands {
        command.reply_schemas.clear();
        command.examples.truncate(1);
        for argument in &mut command.arguments {
            let accepted_form = argument
                .description
                .split(". ")
                .next()
                .unwrap_or_default()
                .to_owned();
            if argument.position.is_none() {
                argument.description.clear();
            }
            if !matches!(
                argument.name.as_str(),
                "--source" | "--sources" | "section" | "--section"
            ) && argument.choices.len() > INLINE_CHOICE_LIMIT
                && let Some((_, _, schema)) = crate::grammar::command_paths()
                    .iter()
                    .find(|(verb, entity, _)| command.command == [*verb, *entity])
            {
                argument.description = format!(
                    "{} allowed values; {accepted_form}. Full list: biomcp schema {schema}.",
                    argument.choices.len()
                );
                argument.choices.clear();
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn source_prose(text: &str) -> String {
    discovery::plain(text)
}

#[cfg(test)]
pub(crate) fn reference(root: &crate::grammar::Root) -> io::Result<Help> {
    let mut commands = commands(root)?;
    public_grammar::canonical(root, &mut commands)?;
    Ok(Help { commands })
}
pub(crate) fn commands(root: &crate::grammar::Root) -> io::Result<Vec<HelpCommand>> {
    let examples: BTreeMap<String, Vec<Vec<String>>> =
        serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/examples.json")))?;
    let mut commands = Vec::new();
    for &(verb, entity, id) in crate::grammar::command_paths().iter() {
        let schema: Value = serde_json::from_str(
            crate::schema::find(id).ok_or_else(|| io::Error::other("missing query schema"))?,
        )?;
        let properties = field(&schema, "properties")
            .as_object()
            .ok_or_else(|| io::Error::other("missing properties"))?;
        let arguments = properties
            .iter()
            .flat_map(|(name, node)| arguments(name, node, &schema))
            .collect();
        let sections = section_words(&schema);
        commands.push(HelpCommand {
            reply_schemas: crate::schema::replies(id, verb, entity),
            command: vec![verb.into(), entity.into()],
            description: command_description(verb, entity, &schema),
            arguments,
            section_rule: (!sections.is_empty()).then(|| {
                let section = field(field(&schema, "properties"), "section");
                text(
                    section,
                    if section.get("x-section-rule").is_some() {
                        "x-section-rule"
                    } else {
                        "description"
                    },
                )
            }),
            sections,
            examples: examples
                .get(&format!("{verb} {entity}"))
                .cloned()
                .unwrap_or_default(),
        });
    }
    commands.extend_from_slice(root.housekeeping());
    extra_commands(&mut commands, &examples);
    commands.extend(network("ca_bundle").commands);
    commands.extend(network("proxy").commands);
    commands.push(topic("exit", crate::exit::reference()));

    public_grammar::augment(root, &mut commands)?;
    discovery::complete(&mut commands);
    for command in &mut commands {
        if command.reply_schemas.is_empty() {
            command.reply_schemas = crate::schema::local_replies(&command.command);
        }
    }
    source_settings::timing(&mut commands);
    Ok(commands)
}
fn command_description(verb: &str, entity: &str, schema: &Value) -> String {
    let sources = coverage::command_sources(verb, entity, schema)
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    let summary = biomcp_types::COMMANDS
        .iter()
        .find(|row| row.verb == verb && row.entity == entity)
        .map_or("", |row| row.summary.trim());
    format!("{summary} Sources: {sources}.")
}

fn extra_commands(commands: &mut Vec<HelpCommand>, examples: &BTreeMap<String, Vec<Vec<String>>>) {
    for (command, description) in [
        (
            vec!["store", "clean"],
            "Remove expired replies, pools and snapshots and older incompatible cache files. Return removal \
                counts in one bounded reply. Fresh state and bundles remain.",
        ),
        (
            vec!["health"],
            "Report publisher terms and bounded source reachability; at most one page for each implemented source.",
        ),
        (
            vec!["sync", "[hgnc|clingen|gencc]"],
            "Synchronize a local source bundle.",
        ),
        (
            vec!["help", "[COMMAND...]"],
            "Print generated command help.",
        ),
        (vec!["key", "list"], crate::keys::LIST_HELP),
        (
            vec!["key", "set", "<source>", "[--replace]"],
            "Register one credential from a no-echo prompt or one standard-input line. Use --config PATH to choose the configuration file.",
        ),
        (
            vec!["key", "remove", "<source>"],
            "Remove a stored credential while preserving other settings.",
        ),
        (
            vec!["serve"],
            "Serve MCP over stdio or --http [ADDR]. Non-loopback binds require --allow-host HOST or \
                http.allowed_hosts; neither authenticates callers. HTTP refuses sync, store clean and --fresh. \
                Stdio saves refuse --replace. --hosted refuses file saves, health details, sync, store clean, \
                --fresh and key commands on every transport with a usage.mcp_unavailable error. MCP revisions: 2026-07-28, 2025-11-25, 2025-06-18, 2025-03-26. \
                --rate-share scales only keyed hosted source rates. Hosted HTTP calls have bounded work and one whole-call deadline. \
                Send MCP-Protocol-Version on HTTP requests. Without it, initialize uses its requested legacy revision; \
                other requests default to 2025-03-26. Unsupported headers return HTTP 400.",
        ),
    ] {
        commands.push(HelpCommand {
            reply_schemas: Vec::new(),
            command: command.into_iter().map(String::from).collect(),
            description: description.into(),
            arguments: Vec::new(),
            sections: Vec::new(),
            section_rule: None,
            examples: Vec::new(),
        });
    }
    bundle_commands(commands, examples);
    commands.extend(
        biomcp_query::sources::keys()
            .iter()
            .map(|name| HelpCommand {
                reply_schemas: Vec::new(),
                command: vec![
                    "help".into(),
                    "key".into(),
                    biomcp_query::sources::key_name(*name).into(),
                ],
                description: "Show credential registration and environment fallback names.".into(),
                arguments: Vec::new(),
                sections: Vec::new(),
                section_rule: None,
                examples: Vec::new(),
            }),
    );
}
fn bundle_commands(commands: &mut Vec<HelpCommand>, examples: &BTreeMap<String, Vec<Vec<String>>>) {
    for declaration in biomcp_query::sources::bundles() {
        let bundle = declaration.name();
        commands.push(HelpCommand {
            reply_schemas: Vec::new(),
            command: vec!["sync".into(), bundle.into()],
            description: declaration.help().into(),
            arguments: Vec::new(),
            sections: Vec::new(),
            section_rule: None,
            examples: examples
                .get(&format!("sync {bundle}"))
                .cloned()
                .unwrap_or_default(),
        });
    }
}
fn arguments(name: &str, node: &Value, root: &Value) -> Vec<HelpArgument> {
    if let Some(flags) = node.get("x-cli-flags").and_then(Value::as_object) {
        return flags
            .keys()
            .map(|flag| {
                let mut value = argument(flag, node, root);
                value.value_type = "boolean".into();
                value.choices.clear();
                value
            })
            .collect();
    }
    vec![argument(name, node, root)]
}
fn argument(name: &str, node: &Value, root: &Value) -> HelpArgument {
    let position = field(node, "x-positional").as_u64();
    let resolved = resolve(node, root);
    let choices = if text(resolved, "type") == "array" {
        resolve(field(resolved, "items"), root)
    } else {
        resolved
    };
    HelpArgument {
        name: if position.is_some() {
            name.into()
        } else {
            format!("--{}", name.replace('_', "-"))
        },
        aliases: Vec::new(),
        default: node.get("default").map(|v| {
            v.as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| v.to_string())
        }),
        description: text(node, "description"),
        value_type: value_type(resolved),
        required: field(root, "required")
            .as_array()
            .is_some_and(|v| v.iter().any(|v| v == name)),
        position,
        minimum: field(node, "minimum")
            .as_u64()
            .or(field(node, "minItems").as_u64()),
        maximum: field(node, "maximum")
            .as_u64()
            .or(field(node, "maxItems").as_u64()),
        choices: field(choices, "enum")
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
    }
}
fn text(node: &Value, key: &str) -> String {
    node[key].as_str().unwrap_or_default().into()
}
fn field<'a>(node: &'a Value, key: &str) -> &'a Value {
    node.get(key).unwrap_or(&Value::Null)
}

fn section_words(schema: &Value) -> Vec<HelpWord> {
    let property = field(field(schema, "properties"), "section");
    let section = property
        .get("anyOf")
        .and_then(Value::as_array)
        .and_then(|variants| variants.iter().find(|v| v.get("$ref").is_some()))
        .or_else(|| property.get("items"))
        .unwrap_or(property);
    let section = resolve(section, schema);
    field(section, "enum")
        .as_array()
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(i, v)| {
            Some(HelpWord {
                word: v.as_str()?.into(),
                description: field(section, "x-enum-descriptions")
                    .get(i)
                    .unwrap_or(&Value::Null)
                    .as_str()?
                    .trim()
                    .into(),
            })
        })
        .collect::<Vec<_>>()
}

fn network(setting: &str) -> Help {
    let description = match setting {
        "ca_bundle" => {
            "Set ca_bundle in config.json to a PEM certificate file. These roots add to platform trust for every source. \
        Keep certificate verification enabled."
        }
        _ => {
            "Set proxy in config.json to an HTTP or HTTPS proxy URL. Otherwise https_proxy/HTTPS_PROXY and http_proxy/HTTP_PROXY \
                select routes for their schemes; lower case wins when both names are set. no_proxy/NO_PROXY bypasses \
                exact hosts/IPs, CIDR ranges, .domain, *.domain, optional \
                ports, or *. Bare IPv6 is accepted. Unsupported entries are skipped and health reports configuration notices. \
                Malformed ports and credentials refuse with the variable name. An explicit proxy overrides proxy environment settings. \
                Publisher PDF downloads are refused when a proxy applies."
        }
    };
    Help {
        commands: vec![HelpCommand {
            reply_schemas: crate::schema::local_replies(&["help".into()]),
            command: vec!["help".into(), "network".into(), setting.into()],
            description: description.into(),
            arguments: Vec::new(),
            sections: Vec::new(),
            section_rule: None,
            examples: Vec::new(),
        }],
    }
}

fn topic(name: &str, description: String) -> HelpCommand {
    HelpCommand {
        command: vec!["help".into(), name.into()],
        description,
        reply_schemas: crate::schema::local_replies(&["help".into()]),
        arguments: Vec::new(),
        sections: Vec::new(),
        section_rule: None,
        examples: Vec::new(),
    }
}
