//! Hold source module debt below the accepted counts.
use super::{Check, rust_files, source::tokens};
use std::{fs, path::Path};

// Lower these ceilings when existing debt is removed. Never raise them.
const PARENTS_WITH_LOGIC: usize = 170;
const MISSING_MODULE_DOCS: usize = 410;

pub(super) fn check(root: &Path) -> Check {
    limits(root, PARENTS_WITH_LOGIC, MISSING_MODULE_DOCS)
}

pub(super) fn limits(root: &Path, parents: usize, docs: usize) -> Check {
    let mut logic = Vec::new();
    let mut undocumented = Vec::new();
    let crates = root.join("crates");
    if !crates.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(crates)? {
        for path in rust_files(&entry?.path().join("src"))? {
            let text = fs::read_to_string(&path)?;
            if !module_doc(&text) {
                undocumented.push(path.clone());
            }
            let (parent, declarations_only) = shape(&tokens(&text));
            if parent && !declarations_only {
                logic.push(path);
            }
        }
    }
    super::diagnostics(vec![
        ceiling(root, "parent modules with logic", &logic, parents),
        ceiling(root, "missing first-line module docs", &undocumented, docs),
    ])
}

fn module_doc(text: &str) -> bool {
    let first = text.lines().next().unwrap_or_default();
    let attribute = tokens(first);
    attribute.starts_with(&["#".into(), "!".into(), "[".into(), "doc".into(), "=".into()])
        || first
            .strip_prefix("//!")
            .is_some_and(|doc| !doc.trim().is_empty())
        || first
            .strip_prefix("/*!")
            .is_some_and(|doc| !doc.trim().trim_end_matches("*/").trim().is_empty())
}

fn ceiling(root: &Path, rule: &str, paths: &[std::path::PathBuf], maximum: usize) -> Check {
    if paths.len() <= maximum {
        return Ok(());
    }
    let examples = paths
        .iter()
        .take(5)
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "{rule}: {} exceeds ceiling {maximum}: {examples}",
        paths.len()
    )
    .into())
}

// Inspect top-level items only. Comments and literals have already been tokenized.
// Attributes, restricted visibility and grouped imports do not introduce logic.
fn shape(input: &[String]) -> (bool, bool) {
    let mut parent = false;
    let mut declarations_only = true;
    let mut index = 0;
    while let Some(token) = input.get(index) {
        if token == "#" {
            index += 1;
            if input.get(index).is_some_and(|token| token == "!") {
                index += 1;
            }
            index = group_end(input, index, "[", "]");
            continue;
        }
        if token == "pub" {
            index += 1;
            if input.get(index).is_some_and(|token| token == "(") {
                index = group_end(input, index, "(", ")");
            }
        }
        let start = index;
        let kind = input.get(index).map(String::as_str).unwrap_or_default();
        parent |= kind == "mod";
        let declaration = matches!(kind, "mod" | "use")
            || kind == "extern" && input.get(index + 1).is_some_and(|token| token == "crate");
        declarations_only &= declaration;
        while let Some(token) = input.get(index) {
            match token.as_str() {
                ";" => {
                    index += 1;
                    break;
                }
                "{" => {
                    index = group_end(input, index, "{", "}");
                    if kind != "use" {
                        declarations_only &= kind != "mod";
                        break;
                    }
                }
                "(" => index = group_end(input, index, "(", ")"),
                "[" => index = group_end(input, index, "[", "]"),
                _ => index += 1,
            }
        }
        if index == start {
            break;
        }
    }
    (parent, declarations_only)
}

fn group_end(input: &[String], start: usize, open: &str, close: &str) -> usize {
    let mut depth = 0;
    for (index, token) in input.iter().enumerate().skip(start) {
        if token == open {
            depth += 1;
        }
        if token == close {
            depth -= 1;
            if depth == 0 {
                return index + 1;
            }
        }
    }
    input.len()
}
