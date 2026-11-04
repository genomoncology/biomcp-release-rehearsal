//! Feature biomarker-nci: retained native pages and conservative MCP projections.
use super::super::{launch::Environment, nci::invoke};
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    vec![Trial::test(
        crate::runner::pool_contract::BIOMARKER.0,
        || pages().map_err(Into::into),
    )]
}
fn environment() -> Result<Environment, String> {
    let env = Environment::new().map_err(|e| e.to_string())?;
    env.config(&json!({"keys":{"nci_api_key":"synthetic-nci-stored-key"}}))
        .map_err(|e| e.to_string())?;
    Ok(env)
}
fn query() -> Vec<String> {
    [
        "search",
        "biomarker",
        "--source",
        "nci",
        "--name",
        "PD-L1",
        "--limit",
        "2",
    ]
    .map(str::to_owned)
    .to_vec()
}
fn pages() -> Check {
    let env = environment()?;
    let replay = super::super::launch::fixtures().join("replies/biomarker/native");
    let mut args = query();
    args.extend(env.globals(Some(&replay), false));
    let mut answer = invoke(&env, &args, 0)?;
    let mut rows = Vec::new();
    for _ in 0..19 {
        if answer.pointer("/page/total") != Some(&json!(37)) {
            return Err("native total changed while paging".into());
        }
        rows.extend(
            answer
                .pointer("/result/items")
                .and_then(Value::as_array)
                .ok_or("missing biomarker headers")?
                .iter()
                .cloned(),
        );
        let next = answer
            .get("next")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|s| s.get("kind") == Some(&json!("page")));
        let Some(next) = next else {
            break;
        };
        let mut args: Vec<String> =
            serde_json::from_value(next.get("args").cloned().ok_or("page args")?)
                .map_err(|e| e.to_string())?;
        if !args.iter().any(|s| s == "--pool") {
            return Err("page lost its snapshot".into());
        }
        args.extend(env.globals(None, true));
        answer = invoke(&env, &args, 0)?;
    }
    let path = replay.join("nci");
    let mut expected = None;
    for file in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let file = file.map_err(|e| e.to_string())?;
        let record: Value =
            serde_json::from_slice(&std::fs::read(file.path()).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        if record
            .get("url")
            .and_then(Value::as_str)
            .is_some_and(|s| s.ends_with("name=PD-L1"))
        {
            let native = record
                .pointer("/exchanges/0/body/value/data")
                .and_then(Value::as_array)
                .ok_or("native rows")?;
            expected = Some(
                native
                    .iter()
                    .map(|row| {
                        let mut header = json!({
                            "name":row["name"], "codes":row["codes"],
                            "type":row["type"], "count":row["count"]
                        });
                        super::super::assertions::omit_empty_lists(&mut header);
                        header
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
    if expected.as_ref() != Some(&rows) {
        return Err("offline pages changed, skipped or repeated native headers".into());
    }
    Ok(())
}
