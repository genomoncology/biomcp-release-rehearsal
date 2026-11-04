//! Keep disjoint scalar/object alternatives distinct during compatibility checks.
use super::Check;
use serde_json::Value;
use std::collections::BTreeSet;

// The scalar representation permits exactly two disjoint JSON types. Keep
// their paths separate so constraints in one branch cannot hide the other.
pub(super) fn branches<'a>(root: &Value, branches: &'a Value) -> Check<Vec<(String, &'a Value)>> {
    let branches = branches.as_array().ok_or("invalid scalar anyOf")?;
    if branches.len() != 2 {
        return Err("unsupported scalar union".into());
    }
    let mut kinds = BTreeSet::new();
    let mut result = Vec::new();
    for branch in branches {
        let resolved = if let Some(reference) = branch.get("$ref") {
            root.pointer(
                reference
                    .as_str()
                    .and_then(|r| r.strip_prefix('#'))
                    .filter(|r| r.starts_with("/$defs/"))
                    .ok_or("unsupported scalar reference")?,
            )
            .ok_or("unresolved scalar reference")?
        } else {
            branch
        };
        let kind = resolved["type"].as_str().ok_or("missing scalar type")?;
        if !["number", "object", "string"].contains(&kind) || !kinds.insert(kind) {
            return Err("unsupported scalar union".into());
        }
        result.push((kind.to_owned(), branch));
    }
    if !kinds.contains("object") {
        return Err("unsupported scalar union".into());
    }
    Ok(result)
}
