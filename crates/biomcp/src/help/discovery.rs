//! Build navigation from the same commands and sections used for execution.
use biomcp_types::HelpCommand;

pub(crate) fn relations(commands: &[HelpCommand]) -> String {
    let entities: std::collections::BTreeSet<_> = commands
        .iter()
        .filter(|c| c.command.first().is_some_and(|v| v == "get"))
        .filter_map(|c| c.command.get(1))
        .cloned()
        .collect();
    let mut links = std::collections::BTreeMap::<String, Vec<String>>::new();
    for command in commands.iter() {
        let Some(name) = command.command.get(1) else {
            continue;
        };
        if command.command.first().is_some_and(|v| v == "get") {
            for section in &command.sections {
                if let Some(entity) = entities
                    .iter()
                    .find(|entity| section.word == **entity || section.word == format!("{entity}s"))
                {
                    links
                        .entry(name.clone())
                        .or_default()
                        .push(format!("{entity} (get section {})", section.word));
                }
            }
        }
        if command.command.first().is_some_and(|v| v == "search") {
            for entity in &entities {
                for argument in command.arguments.iter().filter(|a| {
                    a.name == format!("--{entity}") || a.name == format!("--{entity}-id")
                }) {
                    links
                        .entry(entity.clone())
                        .or_default()
                        .push(format!("{} (search {})", name, argument.name));
                }
            }
        }
    }
    let relations = links
        .into_iter()
        .map(|(entity, mut targets)| {
            targets.sort();
            targets.dedup();
            format!("{entity} -> {}", targets.join(", "))
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "Follow source relationships through a named get section or a search filter. \
        These links retrieve source records without deciding their clinical meaning.\n{relations}"
    )
}

pub(super) fn complete(commands: &mut Vec<HelpCommand>) {
    commands.push(super::topic("relations", relations(commands)));
    complete_examples(commands);
    if let Some(index) = commands.iter().position(|c| c.command.is_empty()) {
        let index = commands.remove(index);
        commands.insert(0, index);
    }
}

fn complete_examples(commands: &mut [HelpCommand]) {
    let examples: Vec<_> = commands
        .iter()
        .filter_map(|command| {
            command
                .examples
                .first()
                .map(|example| (command.command.clone(), example.clone()))
        })
        .collect();
    for command in commands.iter_mut() {
        command.description = plain(&command.description);
        for argument in &mut command.arguments {
            argument.description = plain(&argument.description);
        }
        if let Some(rule) = &mut command.section_rule {
            *rule = plain(rule);
        }
        for section in &mut command.sections {
            section.description = plain(&section.description);
        }
        if command.examples.is_empty() {
            let example = if command.command.is_empty() {
                vec!["help".into()]
            } else if let Some((_, example)) = examples
                .iter()
                .find(|(path, _)| path.starts_with(&command.command))
            {
                example.clone()
            } else if command.command == ["serve"] {
                vec!["help".into(), "serve".into()]
            } else {
                command.command.clone()
            };
            command.examples.push(example);
        }
    }
}

pub(super) fn plain(text: &str) -> String {
    let mut out = String::new();
    for token in text.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_') {
        let word = token.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
        out.push_str(match word {
            "native" | "provider" => "source",
            "Native" | "Provider" => "Source",
            "literal" => "exact",
            "Literal" => "Exact",
            _ => word,
        });
        out.push_str(&token[word.len()..]);
    }
    out.replace("source source", "source")
        .replace("source-source", "source")
}
