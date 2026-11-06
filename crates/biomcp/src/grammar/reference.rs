//! Render documentation from the parser, schema help and harvested case steps.

use clap::Command;
use std::{collections::BTreeSet, fmt::Write, fs, path::PathBuf};

type Check = Result<(), Box<dyn std::error::Error>>;

fn documents() -> Result<Vec<(&'static str, String)>, Box<dyn std::error::Error>> {
    let parser = super::root().map_err(|_| "command grammar unavailable")?;
    let help = crate::help::reference(&parser)?;
    let mut reference = String::from(
        "# Command reference\n\nGenerated from the command grammar and schema help. \
        Run `make docs-generate` to update.\n\nOptions shown on the root command apply to its subcommands. \
        Query sections and source rules follow each command.\n\n",
    );
    writeln!(reference, "{}\n", super::ENUM_CASE_GUIDANCE)?;
    let mut printed = BTreeSet::new();
    command(&mut reference, &parser, &[], &help, &mut printed)?;
    for entry in &help.commands {
        if !printed.contains(&entry.command.join(" ")) {
            writeln!(reference, "## biomcp {}\n", entry.command.join(" "))?;
            details(&mut reference, entry)?;
        }
    }
    let mut recipes = String::from(
        "# Worked commands\n\nGenerated from recorded public cases. Run `make docs-generate` to update. \
        The documentation check executes these cases through CLI and MCP. Live replies can differ.\n\n",
    );
    for name in ["version", "gene", "article", "trial"] {
        let case: serde_json::Value = serde_saphyr::from_str(&fs::read_to_string(
            root().join(format!("tests/cases/readme/{name}.yaml")),
        )?)?;
        let question = case["story"].as_str().ok_or("recipe question missing")?;
        writeln!(recipes, "## {question}\n\n```sh")?;
        for step in case["steps"].as_array().ok_or("recipe steps missing")? {
            let args: Vec<String> = serde_json::from_value(step["run"].clone())?;
            writeln!(
                recipes,
                "biomcp {}",
                shlex::try_join(args.iter().map(String::as_str))?
            )?;
        }
        writeln!(recipes, "```\n")?;
    }
    let skill = fs::read_to_string(root().join("skills/SKILL.md"))?;
    let (before, rest) = skill
        .split_once("<!-- exit-reference:start -->")
        .ok_or("missing exit reference start")?;
    let (_, after) = rest
        .split_once("<!-- exit-reference:end -->")
        .ok_or("missing exit reference end")?;
    let skill = format!(
        "{before}<!-- exit-reference:start -->\n{}\n<!-- exit-reference:end -->{after}",
        crate::exit::reference()
    );
    Ok(vec![
        ("skills/SKILL.md", skill),
        ("docs/reference/commands.md", reference),
        ("docs/recipes.md", recipes),
        ("docs/reference/source-pacing.md", pacing_document()?),
    ])
}

fn command(
    output: &mut String,
    parser: &Command,
    parents: &[&str],
    help: &biomcp_types::Help,
    printed: &mut BTreeSet<String>,
) -> Result<(), std::fmt::Error> {
    let mut path = parents.to_vec();
    path.push(parser.get_name());
    printed.insert(path.iter().skip(1).copied().collect::<Vec<_>>().join(" "));
    writeln!(output, "## {}\n\n```text", path.join(" "))?;
    let mut printable = parser.clone().bin_name(path.join(" "));
    let rendered = printable.render_long_help().to_string();
    let rendered = rendered
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    writeln!(output, "{rendered}\n\n```\n")?;
    if let Some(entry) = help.commands.iter().find(|entry| {
        entry
            .command
            .iter()
            .map(String::as_str)
            .eq(path.iter().skip(1).copied())
    }) {
        details(output, entry)?;
    }
    for child in parser.get_subcommands() {
        command(output, child, &path, help, printed)?;
    }
    Ok(())
}

fn details(output: &mut String, entry: &biomcp_types::HelpCommand) -> std::fmt::Result {
    writeln!(output, "{}\n", entry.description)?;
    for arg in &entry.arguments {
        writeln!(output, "- `{}`: {}", arg.name, arg.description)?;
    }
    if let Some(rule) = &entry.section_rule {
        writeln!(output, "\n{rule}\n")?;
    }
    for section in &entry.sections {
        writeln!(output, "- `{}`: {}", section.word, section.description)?;
    }
    writeln!(output)
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn reference_matches_grammar() -> Check {
    for (name, expected) in documents()? {
        assert!(
            fs::read_to_string(root().join(name))? == expected,
            "{name}: run make docs-generate"
        );
    }
    Ok(())
}

#[test]
#[ignore = "explicit documentation generation"]
fn write_reference() -> Check {
    generate(&root())
}

fn generate(destination: &std::path::Path) -> Check {
    for (name, text) in documents()? {
        let path = destination.join(name);
        fs::create_dir_all(path.parent().ok_or("documentation directory missing")?)?;
        fs::write(path, text)?;
    }
    Ok(())
}

#[test]
fn documentation_generation_preserves_case_bytes() -> Check {
    let copy = tempfile::tempdir()?;
    let cases = copy.path().join("tests/cases");
    fs::create_dir_all(&cases)?;
    let path = cases.join("help.yaml");
    let mut case: serde_json::Value = serde_saphyr::from_str(&fs::read_to_string(
        root().join("tests/cases/gene/help.yaml"),
    )?)?;
    case.pointer_mut("/steps/0/expect")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("missing help assertions")?
        .insert(
            "/result/commands/0/description".into(),
            "Retained assertion".into(),
        );
    let original = serde_json::to_vec(&case)?;
    fs::write(&path, &original)?;
    generate(copy.path())?;
    assert_eq!(fs::read(path)?, original, "docs generation changed a case");
    Ok(())
}

#[test]
fn unscoped_help_uses_parser_nodes_and_scoped_topics_remain_available() -> Check {
    let parser = super::root().map_err(|_| "command grammar unavailable")?;
    let help = crate::help::reference(&parser)?;
    assert!(
        help.commands
            .iter()
            .any(|entry| entry.command == ["help", "get", "trial"]),
        "nested parser help command missing"
    );
    assert!(
        !help
            .commands
            .iter()
            .any(|entry| entry.command == ["sync", "hgnc"]),
        "positional bundle became a command"
    );
    assert!(
        !help
            .commands
            .iter()
            .any(|entry| entry.command == ["help", "key", "ncbi"]),
        "positional credential topic became a command"
    );
    let scoped = crate::help::dispatch(&parser, &["help".into(), "key".into(), "ncbi".into()])?;
    assert!(
        scoped
            .commands
            .iter()
            .any(|entry| entry.command == ["help", "key", "ncbi"]),
        "scoped credential topic missing"
    );
    let index = help
        .commands
        .iter()
        .find(|entry| entry.command.is_empty())
        .ok_or("root help missing")?;
    assert!(
        index
            .arguments
            .iter()
            .any(|argument| argument.name == "--help" && argument.aliases == ["-h"]),
        "grammar aliases missing"
    );
    Ok(())
}

fn pacing_document() -> Result<String, std::fmt::Error> {
    let mut text = String::from(
        "# Source pacing\n\nGenerated from the source quota registry. Run `make docs-generate` to update.\n\n\
        Defaults equal ceilings. Where no authoritative numeric policy was found, the ceiling remains conservative. \
        Unpublished concurrency keeps the existing local ceiling. All rates space requests evenly without bursts. \
        Daily quotas and shared upstream pools remain separate provider constraints.\n\n\
        | Source | Keyless default / ceiling | Keyed default / ceiling | In flight default / ceiling | Policy checked |\n\
        | --- | --- | --- | --- | --- |\n",
    );
    let rate = |rate: biomcp_fetch::Rate| {
        let (requests, seconds) = rate.ratio();
        format!("{requests}/{seconds}s")
    };
    for policy in biomcp_query::sources::pacing_policies() {
        let quota = policy.quota;
        let keyed = quota
            .keyed_rate()
            .map(&rate)
            .unwrap_or_else(|| "same as keyless".into());
        writeln!(
            text,
            "| `{}` | {} | {} | {} | {} |",
            quota.key().name(),
            rate(quota.rate()),
            keyed,
            quota.in_flight(),
            policy.citation
        )?;
    }
    Ok(text)
}

#[test]
fn every_section_rule_keeps_its_complete_source_text() -> Check {
    let parser = super::root().map_err(|_| "grammar unavailable")?;
    let commands = crate::help::commands(&parser)?;
    let paths = super::command_paths();
    for command in &commands {
        let Some(actual) = &command.section_rule else {
            continue;
        };
        let (_, _, id) = paths
            .iter()
            .find(|(verb, entity, _)| command.command == [*verb, *entity])
            .ok_or("section rule has no schema owner")?;
        let schema: serde_json::Value =
            serde_json::from_str(crate::schema::find(id).ok_or("missing schema")?)?;
        let section = schema
            .pointer("/properties/section")
            .ok_or("missing section")?;
        let original = section
            .get("x-section-rule")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("{id}: missing canonical section rule"))?;
        assert_eq!(actual, &crate::help::source_prose(original), "{id}");
        assert!(
            !actual.contains("biomcp_help"),
            "{id}: compact guidance leaked into rule"
        );
    }
    Ok(())
}

#[test]
fn every_rendered_flag_retains_its_schema_or_parser_description() -> Check {
    let parser = super::root().map_err(|_| "grammar unavailable")?;
    let mut commands = crate::help::commands(&parser)?;
    crate::help::compact(&mut commands);
    let missing = commands
        .iter()
        .flat_map(|command| {
            command
                .arguments
                .iter()
                .filter(|argument| {
                    argument.name.starts_with("--")
                        && argument
                            .description
                            .split(" Value form:")
                            .next()
                            .unwrap_or_default()
                            .trim()
                            .is_empty()
                })
                .map(|argument| format!("{} {}", command.command.join(" "), argument.name))
        })
        .collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "flags lack descriptions: {}",
        missing.join(", ")
    );
    for command in &commands {
        for argument in command
            .arguments
            .iter()
            .filter(|argument| argument.name.starts_with("--"))
        {
            assert!(
                argument.description.contains("Value form:")
                    && argument.description.contains("Example:"),
                "{} {} lacks form/example metadata",
                command.command.join(" "),
                argument.name
            );
        }
    }
    Ok(())
}
