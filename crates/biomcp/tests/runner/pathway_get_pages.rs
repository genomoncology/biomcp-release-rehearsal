//! Exact native event rows through the card's retained section selectors.
use super::{configuration::invoke, launch::Environment};
use serde_json::{Value, json};
pub(super) fn check(env: &Environment, globals: &[String]) -> Result<(), String> {
    let mut pool: Option<String> = None;
    for (offset, id, name) in [
        (0, "R-HSA-5675206", "PEA15 binds MAPK monomers and dimers"),
        (4, "R-HSA-5675433", "PP5 dephosphorylates RAF1 S338"),
    ] {
        let mut args = [
            "get",
            "pathway",
            "R-HSA-5673001",
            "events",
            "--section-limit",
            "1",
            "--section-offset",
        ]
        .map(str::to_owned)
        .to_vec();
        args.push(offset.to_string());
        if let Some(pool) = &pool {
            args.extend(["--pool".into(), pool.clone()]);
        }
        let mut command = args.clone();
        command.extend_from_slice(globals);
        let value = invoke(env, command, 0)?;
        super::schema::Registry::embedded()?.validate(&value, &args, true)?;
        for (path, expected) in [
            ("/result/pathway/id/id", json!("R-HSA-5673001")),
            ("/result/events/data/items/0/id", json!(id)),
            ("/result/events/data/items/0/name", json!(name)),
            ("/result/events/data/page/offset", json!(offset)),
            ("/result/events/data/page/total", json!(81)),
        ] {
            if value.pointer(path) != Some(&expected) {
                return Err(format!("retained event differs at {path}: {value}"));
            }
        }
        let observed = value
            .pointer("/result/events/data/page/pool/id")
            .and_then(Value::as_str)
            .ok_or("card event pool")?;
        if pool.as_deref().is_some_and(|expected| expected != observed) {
            return Err("card section replaced its retained pool".into());
        }
        pool = Some(observed.to_owned());
    }
    Ok(())
}
