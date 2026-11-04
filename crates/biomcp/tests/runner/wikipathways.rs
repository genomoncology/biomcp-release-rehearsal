//! Feature pathway-wikipathways: explicit atomic catalog replacement and stored-only reads.
use super::launch::Environment;
use super::pathway_budgets as budgets;
mod concurrent;
type Check = Result<(), String>;
pub(super) fn trial() -> libtest_mimic::Trial {
    super::transcript::trial(
        "pathway-wikipathways/concurrent-readers-during-pair-replacement",
        || concurrent::run().map_err(Into::into),
    )
}
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

const SEARCH: &str = "https://www.wikipathways.org/json/findPathwaysByText.json";
const GET: &str = "https://www.wikipathways.org/json/getPathwayInfo.json";

pub(super) fn catalog(search: bool, changed: bool) -> Value {
    json!(
        {"pathwayInfo":(0..13).map(|n| { let mut row = json!({"id":format!("WP{}",n+1),
        "url":format!("https://www.wikipathways.org/instance/WP{}",n+1),
        "name":if changed {format!("New Atlas {n}")} else {format!("Literal Atlas {n}")},
        "species":if n==12 {"Mus musculus"} else {"Homo sapiens"},"revision":"2025-05-22","authors":"Source Author",
        "description":"Native source description...",
        "citedIn":""}); if search && let Some(map)=row.as_object_mut() { map.insert("datanodes".into(),
        json!("TP53, unchecked label")); map.insert("annotations".into(),json!("native annotation")); } row }).collect::<Vec<_>>()}
    )
}
fn recording(root: &Path, url: &str) -> PathBuf {
    use sha2::{Digest, Sha256};
    let key = Sha256::digest(format!("GET\0{url}").as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    root.join("wikipathways").join(format!("{key}.json"))
}
pub(super) fn seed(
    env: &Environment,
    search: Value,
    get: Value,
    status: u16,
) -> Result<PathBuf, String> {
    let root = env.root().join("replay");
    let dir = root.join("wikipathways");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (url, body, code) in [(SEARCH, search, 200), (GET, get, status)] {
        let exchange = json!(
            {"status":code,"content_type":"application/json","delay_ms":0,"retry_after_ms":null,"body":{"kind":"text","value":body.to_string()}}
        );
        let exchanges = vec![exchange; if code == 500 { 3 } else { 1 }];
        let wire = json!({"source":"wikipathways","method":"GET","url":url,"exchanges":exchanges});
        fs::write(recording(&root, url), wire.to_string()).map_err(|e| e.to_string())?;
    }
    Ok(root)
}

tool_only! {}

fn call(
    phase: &str,
    env: &Environment,
    replay: &Path,
    args: &[&str],
    exit: i32,
) -> Result<Value, String> {
    let mut command = env.globals(Some(replay), false);
    command.extend(args.iter().map(|s| (*s).into()));
    budgets::invoke(phase, env, command, exit)
}
