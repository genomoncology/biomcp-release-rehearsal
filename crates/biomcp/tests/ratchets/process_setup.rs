//! Keep configuration reads and command tree construction at process setup.
use super::{Check, rust_files, source::tokens};
use std::{fs, path::Path};

// Only key set/remove may reread configuration under their existing write lock.
// This persistence owner preserves concurrent edits and the user's JSON field order.
const KEY_WRITE_OWNER: &str = "config/key_file.rs";
// MCP alone refreshes credentials through the existing reader, preserving live key updates.
const CREDENTIAL_OWNER: &str = "config/credentials.rs";

pub(super) fn check(root: &Path) -> Check {
    let source = root.join("crates/biomcp/src");
    for path in rust_files(&source)? {
        let relative = path
            .strip_prefix(&source)?
            .to_string_lossy()
            .replace('\\', "/");
        // These modules contain only private tests, not product entry points.
        if matches!(
            relative.as_str(),
            "grammar/reference.rs" | "config/credential_reads_tests.rs"
        ) {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        placement(&relative, &text)?;
    }
    Ok(())
}

fn placement(path: &str, text: &str) -> Check {
    let input = product_tokens(text);
    let setup = matches!(path, "main.rs" | "mcp.rs" | "mcp/http.rs");
    let mut import = false;
    for (index, name) in input.iter().enumerate() {
        if name == "key_file"
            && path != "keys.rs"
            && path != KEY_WRITE_OWNER
            && input
                .get(index.wrapping_sub(1))
                .is_none_or(|word| word != "mod")
        {
            return Err(format!("{path}: only key writes may use {KEY_WRITE_OWNER}").into());
        }
        if name == "credentials"
            && path != "command/transport.rs"
            && path != CREDENTIAL_OWNER
            && input
                .get(index.wrapping_sub(1))
                .is_none_or(|word| word != "mod")
        {
            return Err(format!("{path}: only MCP transport may refresh credentials").into());
        }
        match name.as_str() {
            "use" => import = true,
            ";" => import = false,
            "*" if import && !setup => {
                return Err(
                    format!("{path}: wildcard imports hide process setup functions").into(),
                );
            }
            _ => {}
        }
        if name == "Root" {
            root_constructor(path, &input, index)?;
        }
        if !matches!(name.as_str(), "read_file" | "root") {
            continue;
        }
        if input
            .get(index.wrapping_sub(1))
            .is_some_and(|word| word == "fn")
        {
            continue;
        }
        let call = input.get(index + 1).is_some_and(|word| word == "(");
        // A function value can be called through a local binding or passed to a callback.
        let reference = input
            .get(index.wrapping_sub(2)..index)
            .is_some_and(|words| words == [":", ":"]);
        let import = input
            .get(index + 1)
            .is_some_and(|word| matches!(word.as_str(), ";" | "," | "}" | "as"))
            && input
                .get(..index)
                .unwrap_or_default()
                .iter()
                .rev()
                .take_while(|word| word.as_str() != ";")
                .any(|word| word == "use");
        if !(call || reference || import) {
            continue;
        }
        if !(setup || path == CREDENTIAL_OWNER && name == "read_file") {
            return Err(format!("{path}: {name} belongs in main or MCP setup").into());
        }
    }
    Ok(())
}

fn root_constructor(path: &str, input: &[String], index: usize) -> Check {
    let tail = input.get(index + 1..).unwrap_or_default();
    let tail = if tail.first().is_some_and(|word| word == ">") {
        tail.get(1..).unwrap_or_default()
    } else {
        tail
    };
    let constructor = tail
        .iter()
        .take(3)
        .map(String::as_str)
        .eq([":", ":", "new"]);
    let alias = tail.first().is_some_and(|word| word == "as")
        || input
            .get(..index)
            .unwrap_or_default()
            .iter()
            .rev()
            .take_while(|word| word.as_str() != ";")
            .any(|word| word == "type");
    if (constructor || alias)
        && !matches!(
            path,
            "grammar.rs" | "grammar/tree.rs" | "mcp/worker_tests.rs"
        )
    {
        return Err(format!("{path}: Root construction belongs in grammar setup").into());
    }
    Ok(())
}

fn product_tokens(text: &str) -> Vec<String> {
    let input = tokens(text);
    let mut product = Vec::new();
    let mut index = 0;
    while let Some(word) = input.get(index) {
        // Ignore only the test module body, then resume checking product items after it.
        let marker = ["#", "[", "cfg", "(", "test", ")", "]", "mod", "tests", "{"];
        if input
            .get(index..index + marker.len())
            .is_some_and(|words| words.iter().map(String::as_str).eq(marker))
        {
            index += marker.len();
            let mut depth = 1;
            while depth > 0 {
                let Some(word) = input.get(index) else {
                    break;
                };
                match word.as_str() {
                    "{" => depth += 1,
                    "}" => depth -= 1,
                    _ => {}
                }
                index += 1;
            }
        } else {
            product.push(word.clone());
            index += 1;
        }
    }
    product
}

pub(super) fn reject_config_reads_and_tree_builds_in_commands() -> Check {
    for text in [
        "crate::config::file::read_file(None)?;",
        "use crate::config::file::read_file as load; load(None)?;",
        "crate::grammar::root()?;",
        "crate::grammar::Root::new(command);",
        "<crate::grammar::Root>::new(command);",
        "let build = crate::grammar::Root::new; build(command);",
        "use crate::grammar::Root as Tree; Tree::new(command);",
        "type Tree = crate::grammar::Root; Tree::new(command);",
        "use crate::grammar::{root as build}; build()?;",
        "use file::{ConfigFile, read_file}; read_file(None)?;",
        "let mut root = root()?;",
        "let load = crate::config::file::read_file; load(None)?;",
        "let build = crate::grammar::root; build()?;",
        "use crate::grammar as syntax; use syntax::*; let build = root; build()?;",
        "use crate::config::file::*; let load = read_file; load(None)?;",
        "#[cfg(test)]\nmod tests {} fn command() { crate::grammar::root(); }",
        "crate::config::credentials::refresh(globals)?;",
        "use crate::config::credentials as keys; keys::refresh(globals)?;",
        "crate::config::key_file::read(path)?;",
        "use crate::config::key_file as persistence; persistence::read(path)?;",
    ] {
        assert!(placement("command/planted.rs", text).is_err(), "{text}");
    }
    placement(
        "main.rs",
        "config::file::read_file(path)?; grammar::root()?;",
    )?;
    placement(
        "mcp/http/call.rs",
        "#[cfg(test)] mod tests { fn fixture() { crate::grammar::root(); } }",
    )?;
    placement(CREDENTIAL_OWNER, "super::file::read_file(path)?;")?;
    placement(
        "command/transport.rs",
        "crate::config::credentials::refresh(globals)?;",
    )?;
    placement(
        "keys.rs",
        "let lock = config::key_file::lock(path)?; config::key_file::read(path)?;",
    )?;
    Ok(())
}
