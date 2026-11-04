//! Only reviewed stored Refusal fields may restore a missing legacy flag.
use super::tokens;
use std::{fs, path::Path};

const CODEC: &str = "crate::mapped::refusal_serde";
const OWNERS: &[(&str, &str, &str)] = &[
    (
        "cbioportal/studies/raw.rs",
        "ReceivedCatalogRow",
        "refusals",
    ),
    ("ctgov/detail.rs", "Detail", "refusals"),
    ("ctgov/detail.rs", "Detail", "document_refusals"),
    ("nci/mod.rs", "Detail", "refusals"),
    ("semantic_scholar/author/papers.rs", "RawPage", "refusals"),
    (
        "semantic_scholar/author/papers/row.rs",
        "Input::Cached",
        "refusals",
    ),
    ("complex_portal/mod.rs", "Complex", "refusals"),
    ("complex_portal/mod.rs", "Collection", "refusals"),
];

pub(in super::super) fn allowed(root: &Path, path: &str, input: &[String], index: usize) -> bool {
    let Some(relative) = path.strip_prefix("crates/biomcp-adapters/src/") else {
        return false;
    };
    let Some(owner) = owner(input, index) else {
        return false;
    };
    let Some(tail) = input.get(index..) else {
        return false;
    };
    let Some(end) = tail.iter().position(|token| token == "]") else {
        return false;
    };
    let mut attribute: Vec<_> = tail.iter().take(end + 1).map(String::as_str).collect();
    if let Some(comma) = attribute.len().checked_sub(3)
        && attribute.get(comma) == Some(&",")
    {
        attribute.remove(comma);
    }
    let codec = if relative == "ctgov/detail.rs" {
        if !alias(root) {
            return false;
        }
        "super::refusals"
    } else {
        CODEC
    };
    let default = format!("\"{codec}::none\"");
    let with = format!("\"{codec}\"");
    if attribute
        != [
            "#", "[", "serde", "(", "default", "=", &default, ",", "with", "=", &with, ")", "]",
        ]
    {
        return false;
    }
    let Some(field) = tail.get(end + 1..) else {
        return false;
    };
    let field = if field.first().is_some_and(|token| token == "pub") {
        field.get(1..).unwrap_or_default()
    } else {
        field
    };
    OWNERS.iter().any(|(file, container, name)| {
        *file == relative
            && *container == owner
            && field
                .iter()
                .take(7)
                .map(String::as_str)
                .eq([*name, ":", "crate", ":", ":", "Refusal", ","])
    })
}

fn owner(input: &[String], index: usize) -> Option<String> {
    let prefix = input.get(..index)?;
    let start = prefix
        .iter()
        .rposition(|token| token == "struct" || token == "enum")?;
    let name = prefix.get(start + 1)?;
    let body = prefix.get(start + 2..)?;
    let open = body.iter().position(|token| token == "{")?;
    let mut stack = Vec::new();
    for (index, token) in body.iter().enumerate().skip(open) {
        if token == "{" {
            stack.push(index);
        } else if token == "}" {
            stack.pop()?;
        }
    }
    match stack.as_slice() {
        [_] => Some(name.clone()),
        [_, variant] if prefix.get(start).is_some_and(|token| token == "enum") => {
            Some(format!("{name}::{}", body.get(variant.checked_sub(1)?)?))
        }
        _ => None,
    }
}

fn alias(root: &Path) -> bool {
    let Ok(text) = fs::read_to_string(root.join("crates/biomcp-adapters/src/ctgov/refusals.rs"))
    else {
        return false;
    };
    let expected =
        tokens("pub(super) use crate::mapped::refusal_serde::{deserialize, none, serialize};");
    tokens(&text).starts_with(&expected)
}
