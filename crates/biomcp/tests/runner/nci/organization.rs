//! Native organization continuations retain a complete observation without another request.
use super::super::{launch::Environment, nci::invoke};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), String>;
pub(super) fn trials() -> Vec<Trial> {
    // The test reuses runtime organization pool IDs with foreign selectors and checks retained-page ownership.
    vec![transcript_trial(
        crate::runner::pool_contract::ORGANIZATION.0,
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
        "organization",
        "--source",
        "nci",
        "--name",
        "Dana-Farber",
        "--limit",
        "2",
    ]
    .map(str::to_owned)
    .to_vec()
}
fn pages() -> Check {
    let env = environment()?;
    let replay = super::super::launch::fixtures().join("replies/organization/native");
    let mut args = query();
    args.extend(env.globals(Some(&replay), false));
    let mut answer = invoke(&env, &args, 0)?;
    let mut names = Vec::new();
    let mut counts = Vec::new();
    for _ in 0..6 {
        if answer.pointer("/page/total") != Some(&json!(11)) {
            return Err("organization total changed while paging".into());
        }
        let rows = answer
            .pointer("/result/items")
            .and_then(Value::as_array)
            .ok_or("missing organization rows")?;
        for row in rows {
            names.push(row.get("name").cloned().ok_or("missing name")?);
            counts.push(
                row.get("count")
                    .cloned()
                    .ok_or("missing native trial count")?,
            );
            if row.get("id").is_some() {
                return Err("organization acquired an invented identity".into());
            }
        }
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
            return Err("page lost snapshot identity".into());
        }
        args.extend(env.globals(None, true));
        answer = invoke(&env, &args, 0)?;
    }
    if names.len() != 11
        || counts.get(3) != Some(&json!(3376))
        || names.get(3) != Some(&json!("Dana-Farber Cancer Institute"))
    {
        return Err("local pages skipped, repeated or changed source rows".into());
    }
    Ok(())
}
