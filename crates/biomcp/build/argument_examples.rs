//! Retain recorded positive examples that demonstrate otherwise unrepresented flags.
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::Path,
};

pub(super) fn append(
    directory: &Path,
    examples: &mut BTreeMap<String, Vec<Vec<String>>>,
) -> Result<(), Box<dyn Error>> {
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            append(&path, examples)?;
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "yaml") {
            continue;
        }
        println!("cargo:rerun-if-changed={}", path.display());
        let case: serde_json::Value = serde_saphyr::from_str(&fs::read_to_string(path)?)?;
        for step in case
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            if step
                .get("exit")
                .and_then(serde_json::Value::as_i64)
                .is_some_and(|exit| exit != 0)
                || step.get("is_error").and_then(serde_json::Value::as_bool) == Some(true)
            {
                continue;
            }
            let Some(run) = step
                .get("run")
                .and_then(|run| serde_json::from_value::<Vec<String>>(run.clone()).ok())
            else {
                continue;
            };
            if run
                .iter()
                .any(|word| word.contains("{{") || word.contains("${"))
                || !biomcp_types::COMMANDS.iter().any(|command| {
                    run.first().is_some_and(|verb| verb == command.verb)
                        && run.get(1).is_some_and(|entity| entity == command.entity)
                })
            {
                continue;
            }
            let command = run.get(..2).ok_or("command path missing")?.join(" ");
            let retained = examples.entry(command).or_default();
            let known = retained
                .iter()
                .flat_map(|run| flags(run))
                .collect::<BTreeSet<_>>();
            if flags(&run).any(|flag| !known.contains(flag)) {
                retained.push(run);
            }
        }
    }
    Ok(())
}
fn flags(run: &[String]) -> impl Iterator<Item = &str> {
    run.iter()
        .filter(|word| word.starts_with("--"))
        .map(|word| word.split_once('=').map_or(word.as_str(), |(flag, _)| flag))
}
