//! Existing native reference control and synthetic retained citation boundary.
use super::native_fields::check as native;
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
type Check = Result<(), Box<dyn std::error::Error>>;
pub(super) fn trial() -> Trial {
    transcript_trial(crate::runner::pool_contract::ARTICLE_CITATIONS.0, || {
        run().map_err(|e| e.to_string().into())
    })
}
fn run() -> Check {
    native(
        "article/edges/live",
        &[
            "article",
            "references",
            "33123537",
            "--source",
            "semantic-scholar",
        ],
        "semantic-scholar",
        "/exchanges/0/body/value/data/0/isInfluential",
        "influential",
        json!({"/result/items/0/pmid":"32597970"}),
        "/result/items/0/influential",
    )?;
    retained_citations()
}

fn retained_citations() -> Check {
    let env = crate::runner::launch::Environment::new()?;
    let replay = crate::runner::launch::fixtures().join("replies/article/edges/synthetic-window");
    let globals = env.globals(Some(&replay), false);
    let mut args = [
        "article",
        "citations",
        "23000897",
        "--source",
        "semantic-scholar",
        "--limit",
        "1",
    ]
    .map(str::to_owned)
    .to_vec();
    let mut first_args = args.clone();
    first_args.extend(globals.clone());
    let first = crate::runner::configuration::invoke(&env, first_args, 0)?;
    crate::runner::schema::Registry::embedded()?.validate(&first, &args, true)?;
    let pool = first
        .pointer("/page/pool/id")
        .and_then(Value::as_str)
        .ok_or("citation pool")?;
    args.extend(["--pool".into(), pool.into(), "--offset".into(), "1".into()]);
    let mut retained_args = args.clone();
    retained_args.extend(globals);
    let kept = crate::runner::configuration::invoke(&env, retained_args, 0)?;
    crate::runner::schema::Registry::embedded()?.validate(&kept, &args, true)?;
    if kept.pointer("/result/items/0/header/title") != Some(&json!("Synthetic source page2."))
        || kept.pointer("/page/offset") != Some(&json!(1))
    {
        return Err("retained citation page changed its source order".into());
    }
    Ok(())
}
