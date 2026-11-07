//! Shared typed YAML case parsing and file discovery.
use serde::Deserialize;
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

// The shared typed YAML parser
// rejects unknown fields so a misspelled assertion cannot silently pass.
// https://docs.rs/serde-saphyr/latest/serde_saphyr/#using-serializer-or-deserializer-specifically
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Case {
    #[serde(rename = "live_check")]
    pub(crate) _live_check: Option<String>,
    pub(crate) feature: String,
    pub(crate) story: String,
    pub(crate) replay: Option<String>,
    pub(crate) cache_acquired: Option<i64>,
    pub(crate) config: Option<Value>,
    #[serde(rename = "harvest", default)]
    pub(crate) _harvest: Option<String>,
    #[serde(rename = "verified", default)]
    pub(crate) _verified: Option<String>,
    pub(crate) steps: Vec<Step>,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Step {
    pub(crate) run: Option<Vec<String>>,
    pub(crate) stdout: Option<String>,
    pub(crate) stdin: Option<String>,
    pub(crate) follow: Option<String>,
    pub(crate) walk: Option<Walk>,
    #[serde(default)]
    pub(crate) offline: bool,
    pub(crate) mcp: Option<Vec<String>>,
    pub(crate) mcp_call: Option<McpCall>,
    #[serde(default)]
    pub(crate) is_error: bool,
    #[serde(default)]
    pub(crate) exit: i32,
    #[serde(default)]
    pub(crate) full_output_bytes: bool,
    pub(crate) max_bytes: Option<usize>,
    pub(crate) max_tokens: Option<usize>,
    #[serde(default)]
    pub(crate) expect: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct McpCall {
    pub(crate) name: String,
    pub(crate) arguments: Value,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Walk {
    pub(crate) items: String,
    pub(crate) max_pages: u32,
}

pub(crate) fn collect(root: &Path, paths: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, paths)?;
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            paths.push(path);
        }
    }
    Ok(())
}

pub(crate) fn read_case(path: &Path) -> Result<Case, String> {
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_saphyr::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}
