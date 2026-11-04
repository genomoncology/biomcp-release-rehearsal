//! Derive mapping construction decisions from the source that owns them.
use super::super::{Check, rust_files};
use super::tokens;
use std::{fs, io::Write, path::Path};

pub(super) fn check(root: &Path) -> Check {
    let mut inventory = Vec::new();
    for directory in ["crates/biomcp-adapters/src", "crates/biomcp-query/src"] {
        for path in rust_files(&root.join(directory))? {
            let source = tokens(&fs::read_to_string(&path)?);
            for (index, token) in source.iter().enumerate() {
                if token != "Mapped" {
                    continue;
                }
                let mut opening = index + 1;
                if source
                    .get(opening..opening + 3)
                    .is_some_and(|v| v == [":", ":", "<"])
                {
                    opening += 3;
                    let mut generic = 1;
                    while generic > 0 {
                        match source.get(opening).map(String::as_str) {
                            Some("<") => generic += 1,
                            Some(">") => generic -= 1,
                            None => break,
                            _ => {}
                        }
                        opening += 1;
                    }
                }
                if source.get(opening).is_none_or(|v| v != "{") {
                    continue;
                }
                let mut depth = 1;
                let mut decision = false;
                for (offset, token) in source.iter().skip(opening + 1).enumerate() {
                    match token.as_str() {
                        "{" | "(" | "[" => depth += 1,
                        "}" | ")" | "]" => depth -= 1,
                        "refusals" if depth == 1 => {
                            decision = source
                                .get(opening + 2 + offset)
                                .is_some_and(|v| matches!(v.as_str(), ":" | "," | "}"));
                        }
                        _ => {}
                    }
                    if depth == 0 {
                        break;
                    }
                }
                if !decision {
                    return Err(format!(
                        "{}: Mapped construction at token {index} requires an explicit refusal decision",
                        path.strip_prefix(root)?.display()
                    )
                    .into());
                }
                inventory.push(format!(
                    "{}: Mapped at token {index}: explicit refusal decision",
                    path.strip_prefix(root)?.display()
                ));
            }
        }
    }
    let mut output = std::io::stdout().lock();
    for site in &inventory {
        writeln!(output, "{site}")?;
    }
    writeln!(output, "Mapped construction decisions: {}", inventory.len())?;
    Ok(())
}
