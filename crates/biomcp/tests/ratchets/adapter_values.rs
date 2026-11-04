//! Classify adapter and query value drops beside the existing source ratchet.
#[path = "private_refusal.rs"]
pub(crate) mod private_refusal;
use super::super::{Check, rust_files};
use super::tokens;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
#[derive(Deserialize)]
pub(crate) struct Audit {
    sites: Vec<Site>,
}
#[derive(Deserialize)]
struct Site {
    path: String,
    site: String,
    maximum: usize,
    class: String,
    owner: String,
    reason: String,
}
pub(super) fn check(root: &Path) -> Check {
    let audit = validate(include_str!("adapter_value_audit.json"))?;
    let mut violations = Vec::new();
    for (relative, site, count) in sites(root)? {
        if !audit
            .sites
            .iter()
            .any(|entry| entry.path == relative && entry.site == site && count <= entry.maximum)
        {
            let rule = if site.starts_with("adapter serde refusal:") {
                "raw serde default or deny_unknown_fields forbidden"
            } else {
                "classify new optional-value drop or file rule"
            };
            violations.push(format!("{relative}: {rule}: {site}"));
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations.join("\n").into())
    }
}
pub(crate) fn validate(text: &str) -> Result<Audit, Box<dyn std::error::Error>> {
    let audit: Audit = serde_json::from_str(text)?;
    let mut seen = BTreeSet::new();
    for entry in &audit.sites {
        let adapter = entry
            .path
            .strip_prefix("crates/biomcp-adapters/src/")
            .or_else(|| entry.path.strip_prefix("crates/biomcp-query/src/"))
            .and_then(|path| path.split('/').next())
            .unwrap_or("");
        if !seen.insert((&entry.path, &entry.site))
            || entry.maximum == 0
            || entry.owner != adapter
            || entry.reason.is_empty()
            || !matches!(
                entry.class.as_str(),
                "migrated" | "identity_refusal" | "legitimate_absence" | "pending_audit"
            )
            || !initial_entry(entry)
        {
            return Err(format!(
                "{}: new or invalid pending entry: {}",
                entry.path, entry.site
            )
            .into());
        }
    }
    Ok(audit)
}
fn initial_entry(entry: &Site) -> bool {
    let identity = format!("{}\n{}", entry.path, entry.site);
    let hash: String = Sha256::digest(identity.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    INITIAL_IDENTITIES
        .lines()
        .filter_map(|line| line.split_once(' '))
        .any(|(allowed, maximum)| {
            allowed == hash
                && maximum
                    .parse::<usize>()
                    .is_ok_and(|maximum| entry.maximum <= maximum)
        })
}
fn audit() -> Result<Audit, Box<dyn std::error::Error>> {
    Ok(serde_json::from_str(include_str!(
        "adapter_value_audit.json"
    ))?)
}
pub(super) fn pending(
    path: &str,
    site: &str,
    count: usize,
) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(audit()?
        .sites
        .iter()
        .any(|entry| entry.path == path && entry.site == site && count <= entry.maximum))
}
type DropSite = (String, String, usize);
pub(crate) fn sites(root: &Path) -> Result<Vec<DropSite>, Box<dyn std::error::Error>> {
    let mut result = Vec::new();
    for directory in ["crates/biomcp-adapters/src", "crates/biomcp-query/src"] {
        let directory = root.join(directory);
        if !directory.exists() {
            continue;
        }
        for path in rust_files(&directory)? {
            let relative = path
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            let text = fs::read_to_string(&path)?;
            let mut found = fingerprints(root, &tokens(&text), &relative);
            if relative.starts_with("crates/biomcp-adapters/src/")
                && text.lines().count() > super::ADAPTER_LIMIT
            {
                found.insert("file line limit".into(), text.lines().count());
            }
            for (site, count) in found {
                result.push((relative.clone(), site, count));
            }
        }
    }
    Ok(result)
}

fn fingerprints(root: &Path, tokens: &[String], path: &str) -> BTreeMap<String, usize> {
    let mut result = BTreeMap::new();
    let mut function = "module";
    for (index, token) in tokens.iter().enumerate() {
        if token == "fn"
            && let Some(name) = tokens.get(index + 1)
        {
            function = name;
        }
        if path.starts_with("crates/biomcp-adapters/src/")
            && let Some(site) = attribute_site(tokens, index)
            && !private_refusal::allowed(root, path, tokens, index)
        {
            *result.entry(site).or_default() += 1;
        }
        let tail = tokens.get(index..).unwrap_or_default();
        let ok = tail.starts_with(&[".".into(), "ok".into(), "(".into(), ")".into()]);
        let pattern = matches!(token.as_str(), "if" | "let")
            && (tail.starts_with(&["if".into(), "let".into(), "Ok".into()])
                || token == "let"
                    && index
                        .checked_sub(1)
                        .and_then(|i| tokens.get(i))
                        .is_none_or(|t| t != "if")
                    && tail.get(1).is_some_and(|t| t == "Ok"));
        // Treat defaults throughout raw/map files conservatively, and also catch
        // parse receivers (including a result assigned earlier in the function).
        let start = tokens
            .get(..index)
            .unwrap_or_default()
            .iter()
            .rposition(|t| t == "fn")
            .unwrap_or(0);
        let default = tail.starts_with(&[
            ".".into(),
            "unwrap_or_default".into(),
            "(".into(),
            ")".into(),
        ]) && (path.ends_with("/raw.rs")
            || path.ends_with("/map.rs")
            || tokens
                .get(start..index)
                .unwrap_or_default()
                .iter()
                .any(|t| {
                    matches!(
                        t.as_str(),
                        "parse" | "from_str" | "from_slice" | "from_value" | "from_reader"
                    )
                }));
        if ok || default || pattern {
            let end = if pattern {
                (index + 16).min(tokens.len())
            } else {
                index + 4
            };
            let context = tokens
                .get(index.saturating_sub(12)..end)
                .unwrap_or_default()
                .join(" ");
            *result.entry(format!("{function}: {context}")).or_default() += 1;
        }
    }
    result
}
pub(super) fn attribute_site(tokens: &[String], index: usize) -> Option<String> {
    if tokens.get(index)? != "#" {
        return None;
    }
    let tail = tokens.get(index + 1..)?;
    let end = tail.iter().position(|t| t == "]")?;
    let attribute = tail.get(..end)?;
    if !attribute.iter().any(|t| t == "serde")
        || !attribute
            .iter()
            .any(|t| t == "default" || t == "deny_unknown_fields")
    {
        return None;
    }
    Some(format!(
        "adapter serde refusal: {}",
        tail.get(..(end + 13).min(tail.len()))?.join(" ")
    ))
}

// Initial path/fingerprint/count identities form a fixed ceiling. The active
// inventory may remove entries, lower counts or reclassify without changing it.
const INITIAL_IDENTITIES: &str = include_str!("adapter_value_ceilings.txt");
