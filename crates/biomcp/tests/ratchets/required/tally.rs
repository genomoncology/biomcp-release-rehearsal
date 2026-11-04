use super::{Check, Value, case_data::Step};
use std::collections::{BTreeMap, BTreeSet};

const FAILURES: &[(&str, &str)] = &[
    ("not-found", "not-found"),
    ("unavailable", "unavailable"),
    ("bad-source-data", "bad-reply"),
    ("offline-miss", "offline"),
    ("expired-pool", "pool.expired"),
];

pub(super) struct Inventory {
    pub(super) calls: BTreeMap<String, usize>,
    commands: Vec<Value>,
}
impl Inventory {
    pub(super) fn new(help: &Value, tools: &[Value]) -> Result<Self, Box<dyn std::error::Error>> {
        let commands = help
            .pointer("/result/commands")
            .and_then(Value::as_array)
            .ok_or("help commands missing")?
            .clone();
        let mut inventory = Self {
            calls: BTreeMap::new(),
            commands,
        };
        let globals: BTreeSet<_> = inventory
            .commands
            .iter()
            .find(|c| words(&c["command"]).is_empty())
            .into_iter()
            .flat_map(|c| rows(c, "arguments"))
            .filter_map(|a| a["name"].as_str())
            .collect();
        for command in &inventory.commands {
            let path = words(&command["command"]).join(" ");
            if !path.is_empty() {
                inventory.calls.insert(format!("command {path}"), 0);
                inventory.calls.insert(format!("refusal {path}"), 0);
            }
            for argument in rows(command, "arguments") {
                if let Some(name) = argument["name"].as_str().filter(|s| s.starts_with('-')) {
                    if !path.is_empty() && globals.contains(name) {
                        continue;
                    }
                    let item = if path.is_empty() {
                        format!("global {name}")
                    } else {
                        format!("option {path} {name}")
                    };
                    inventory.calls.insert(item, 0);
                }
            }
            for section in rows(command, "sections") {
                inventory.calls.insert(
                    format!(
                        "section {path} {}",
                        section["word"].as_str().ok_or("section word missing")?
                    ),
                    0,
                );
            }
        }
        for tool in tools {
            inventory.calls.insert(
                format!("tool {}", tool["name"].as_str().ok_or("tool name missing")?),
                0,
            );
        }
        for (failure, _) in FAILURES {
            inventory.calls.insert(format!("failure {failure}"), 0);
            for family in ["trial", "variant", "label", "article"] {
                if *failure == "expired-pool" && matches!(family, "variant" | "label") {
                    continue;
                }
                inventory
                    .calls
                    .insert(format!("failure {family} {failure}"), 0);
            }
        }
        Ok(inventory)
    }
    pub(super) fn step(&mut self, step: &Step) {
        // Only explicit CLI argv count. MCP-only and dynamic follow/walk expansions do not.
        if step.follow.is_some() {
            return;
        }
        if !step.expect.is_empty() {
            if let Some(call) = &step.mcp_call {
                self.increment(BTreeSet::from([format!("tool {}", call.name)]));
            } else if step.mcp.is_some() {
                self.increment(BTreeSet::from(["tool biomcp".into()]));
            }
        }
        let Some(argv) = &step.run else {
            return;
        };
        if step.expect.is_empty() && step.stdout.is_none() {
            return;
        }
        let (tokens, mut hits) = self.arguments(argv);
        if let Some(command) = self.command(&tokens) {
            let path_words = words(&command["command"]);
            let path = path_words.join(" ");
            if step.exit == 2
                && step.expect.get("/error/code").is_some_and(|v| {
                    [
                        "usage.arguments",
                        "usage.schema",
                        "usage.unknown_command",
                        "usage.retired",
                    ]
                    .iter()
                    .any(|code| asserts(v, code))
                })
            {
                hits.insert(format!("refusal {path}"));
            }
            for parent in &self.commands {
                let words = words(&parent["command"]);
                if !words.is_empty() && path_words.starts_with(&words) {
                    hits.insert(format!("command {}", words.join(" ")));
                }
            }
            if !hits.contains(&format!("option {path} --section")) {
                // The grammar splits only a suffix and keeps the first input as an identity.
                for token in tokens.iter().skip(path_words.len() + 1).rev() {
                    if rows(command, "sections")
                        .iter()
                        .any(|section| section["word"] == *token)
                    {
                        hits.insert(format!("section {path} {token}"));
                    } else {
                        break;
                    }
                }
            }
            self.failures(step, &path_words, &tokens, &mut hits);
        }
        self.increment(hits);
    }
    fn increment(&mut self, hits: BTreeSet<String>) {
        for item in hits {
            if let Some(count) = self.calls.get_mut(&item) {
                *count += 1;
            }
        }
    }
    pub(super) fn runner(&mut self, items: &[&str]) {
        self.increment(items.iter().map(|item| (*item).to_owned()).collect());
    }
    fn command(&self, tokens: &[String]) -> Option<&Value> {
        self.commands
            .iter()
            .filter(|c| !words(&c["command"]).is_empty())
            .filter(|c| tokens.starts_with(&words(&c["command"])))
            .max_by_key(|c| words(&c["command"]).len())
    }
    fn arguments(&self, argv: &[String]) -> (Vec<String>, BTreeSet<String>) {
        let globals = self
            .commands
            .iter()
            .find(|c| words(&c["command"]).is_empty());
        let mut hits = BTreeSet::new();
        let mut tokens = Vec::new();
        let mut index = 0;
        let mut positional = false;
        while let Some(token) = argv.get(index) {
            if token == "--" {
                positional = true;
                index += 1;
                continue;
            }
            let flag = token.split('=').next().unwrap_or(token);
            let scoped = self.command(&tokens);
            let argument = globals
                .into_iter()
                .chain(scoped)
                .flat_map(|c| rows(c, "arguments"))
                .find(|a| a["name"] == flag || words(&a["aliases"]).iter().any(|a| a == flag));
            if let Some(argument) = argument.filter(|_| !positional && token.starts_with('-')) {
                let name = argument["name"].as_str().unwrap_or_default();
                if globals.is_some_and(|g| rows(g, "arguments").iter().any(|a| a["name"] == name)) {
                    hits.insert(format!("global {name}"));
                } else if let Some(scoped) = scoped {
                    hits.insert(format!(
                        "option {} {name}",
                        words(&scoped["command"]).join(" ")
                    ));
                }
                if name == "--section"
                    && let Some(scoped) = scoped
                {
                    section_argument(scoped, token, argv.get(index + 1), &mut hits);
                }
                let next_flag = argv
                    .get(index + 1)
                    .map(|token| token.split('=').next().unwrap_or(token));
                let next_is_flag = next_flag.is_some_and(|flag| {
                    globals
                        .into_iter()
                        .chain(scoped)
                        .flat_map(|c| rows(c, "arguments"))
                        .any(|a| {
                            a["name"] == flag || words(&a["aliases"]).iter().any(|a| a == flag)
                        })
                });
                if !token.contains('=') && argument["value_type"] != "boolean" && !next_is_flag {
                    index += 1;
                }
            } else {
                tokens.push(token.clone());
            }
            index += 1;
        }
        (tokens, hits)
    }
    fn failures(
        &self,
        step: &Step,
        path: &[String],
        tokens: &[String],
        hits: &mut BTreeSet<String>,
    ) {
        let family = if path.iter().any(|s| s == "trial") {
            Some("trial")
        } else if path.iter().any(|s| s == "variant") {
            Some("variant")
        } else if path.iter().any(|s| s == "article") {
            Some("article")
        } else if path == ["get", "drug"] && tokens.iter().skip(3).any(|s| s == "label")
            || path == ["search", "drug"]
                && step.run.as_ref().is_some_and(|args| {
                    args.windows(2)
                        .any(|pair| pair == ["--source", "openfda-label"])
                })
        {
            Some("label")
        } else {
            None
        };
        if family == Some("label")
            && step
                .expect
                .get("/status")
                .is_some_and(|value| asserts(value, "absent"))
            && step
                .expect
                .get("/sources/0/source")
                .is_some_and(|value| asserts(value, "openfda-label"))
            && step
                .expect
                .get("/sources/0/state")
                .is_some_and(|value| asserts(value, "ok"))
        {
            hits.insert("failure label not-found".into());
        }
        // A checked registry absence is a successful source with an absent item, not a source failure.
        if path == ["get", "trial"]
            && [
                ("/status", "absent"),
                ("/result/items/0/status", "absent"),
                ("/sources/0/source", "ctgov"),
                ("/sources/0/state", "ok"),
            ]
            .iter()
            .all(|(pointer, value)| {
                step.expect
                    .get(*pointer)
                    .is_some_and(|assertion| asserts(assertion, value))
            })
        {
            hits.insert("failure trial not-found".into());
        }
        for (failure, wire) in FAILURES {
            if step.expect.iter().any(|(pointer, value)| {
                (pointer.ends_with("/reason") || pointer.ends_with("/code")) && asserts(value, wire)
            }) {
                hits.insert(format!("failure {failure}"));
                if let Some(family) = family {
                    hits.insert(format!("failure {family} {failure}"));
                }
            }
        }
    }
    pub(super) fn check(&self, pending: &BTreeSet<String>) -> Check {
        let missing: BTreeSet<_> = self
            .calls
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(item, _)| item.clone())
            .collect();
        let new: Vec<_> = missing.difference(pending).cloned().collect();
        let stale: Vec<_> = pending.difference(&missing).cloned().collect();
        if new.is_empty() && stale.is_empty() {
            return Ok(());
        }
        Err(format!("required coverage: uncovered items {new:?}; remove covered or retired pending items {stale:?}").into())
    }
}
fn words(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
fn rows<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .map_or(&[], Vec::as_slice)
}

fn section_argument(
    command: &Value,
    token: &str,
    next: Option<&String>,
    hits: &mut BTreeSet<String>,
) {
    let value = token
        .split_once('=')
        .map(|(_, value)| value)
        .or_else(|| next.map(String::as_str));
    for section in rows(command, "sections") {
        if section["word"].as_str() == value {
            hits.insert(format!(
                "section {} {}",
                words(&command["command"]).join(" "),
                value.unwrap_or_default()
            ));
        }
    }
}

fn asserts(value: &Value, expected: &str) -> bool {
    value == expected || value.get("equals").is_some_and(|value| value == expected)
}
