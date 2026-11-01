//! Exercise diagnostic dependency planning through the runner, without live sources.
use super::launch::{Environment, fixtures};
use crate::runner::transcript::trial as transcript_trial;
use libtest_mimic::Trial;
use serde_json::{Value, json};
use std::fs;

pub(super) fn trials() -> Vec<Trial> {
    vec![transcript_trial(
        "live-source-checks/follow-predecessors",
        || check().map_err(|e| e.to_string().into()),
    )]
}
fn check() -> Result<(), Box<dyn std::error::Error>> {
    let env = Environment::new()?;
    let root = env.root().join("fixtures");
    fs::create_dir_all(root.join("cases"))?;
    let witnessed = fixtures().join("cases/article/search/checked-request.yaml");
    let mut checked: Value = serde_saphyr::from_str(&fs::read_to_string(&witnessed)?)?;
    checked
        .as_object_mut()
        .ok_or("missing case object")?
        .insert("live_check".into(), json!("public"));
    checked
        .get_mut("steps")
        .ok_or("missing steps")?
        .as_array_mut()
        .ok_or("missing steps")?
        .push(json!({"follow":"page"}));
    fs::write(
        root.join("cases/checked.yaml"),
        serde_json::to_vec(&checked)?,
    )?;
    fs::write(
        root.join("cases/excluded.yaml"),
        serde_json::to_vec(&json!({
            "feature":"live-source-checks", "live_check": "public", "story": "An offline source predecessor stays excluded.",
            "replay": "article/search/live", "steps": [
                {"run": ["search", "article", "AKT1 E17K"], "offline": true},
                {"follow": "page"}, {"follow": "page"}
            ]
        }))?,
    )?;
    fs::write(
        root.join("cases/eligible.yaml"),
        serde_json::to_vec(&json!({
            "feature":"live-source-checks", "live_check": "public", "story": "An eligible source permits its follow.",
            "replay": "article/search/live", "steps": [
                {"run": ["search", "article", "AKT1 E17K"]}, {"follow": "page"}
            ]
        }))?,
    )?;
    let plan = diagnostic(&env, &root, true)?;
    let cases = plan
        .get("cases")
        .and_then(Value::as_array)
        .ok_or("missing inventory")?;
    for name in ["checked.yaml", "excluded.yaml"] {
        if !cases.iter().any(|case| {
            case.get("case") == Some(&json!(name))
                && case.get("skipped") == Some(&json!("follow predecessor is ineligible"))
        }) {
            return Err(format!("dependent case not excluded: {plan}").into());
        }
    }
    if cases.len() != 3
        || plan.get("eligible") != Some(&json!(1))
        || !cases.iter().any(|case| {
            case.get("case") == Some(&json!("eligible.yaml"))
                && case.get("eligible") == Some(&json!(true))
        })
    {
        return Err(format!("eligible follow inventory changed: {plan}").into());
    }
    // Remove the live case before execution: this proof never contacts a source.
    fs::remove_file(root.join("cases/eligible.yaml"))?;
    let report = diagnostic(&env, &root, false)?;
    let excluded_plan = diagnostic(&env, &root, true)?;
    if report.get("discovered") != Some(&json!(2))
        || report.get("checked") != excluded_plan.get("eligible")
        || report.get("eligible") != excluded_plan.get("eligible")
        || report.get("error_cases") != Some(&json!(0))
        || report.pointer("/skipped/follow predecessor is ineligible") != Some(&json!(2))
    {
        return Err(format!("skipped diagnostic counts changed: {report}").into());
    }
    // Ordinary replay still checks the local error through the kept case.
    super::run_case(&witnessed, super::schema::Registry::embedded()?.as_ref())?;
    missing_marker(&env, &root)?;
    continuation_context()?;
    Ok(())
}
fn missing_marker(
    env: &Environment,
    root: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Plant an actual kept recording case, then remove only its declaration.
    let path = fixtures().join("cases/readme/gene.yaml");
    let mut case: Value = serde_saphyr::from_str(&fs::read_to_string(path)?)?;
    case.as_object_mut()
        .ok_or("missing case object")?
        .remove("live_check");
    fs::write(
        root.join("cases/unreviewed.yaml"),
        serde_json::to_vec(&case)?,
    )?;
    let output = env
        .command(
            std::env::current_exe()?,
            &["--live-check".into(), "--list".into()],
        )
        .env("BIOMCP_TEST_FIXTURES", root)
        .output()?;
    if output.status.success()
        || !String::from_utf8_lossy(&output.stderr).contains("unreviewed.yaml: missing live_check")
    {
        return Err("discovery accepted a real recording without its declaration".into());
    }
    Ok(())
}
fn continuation_context() -> Result<(), Box<dyn std::error::Error>> {
    let env = Environment::new()?;
    let replay = fixtures().join("replies/trial/walk");
    let mut capture = json!({});
    let (first, observations) = super::live::witness::invoke(
        &env,
        &[
            "search".into(),
            "trial".into(),
            "--condition".into(),
            "melanoma".into(),
        ],
        Some(&replay),
        1,
        &mut capture,
        "baseline",
    )?;
    let token = observations
        .iter()
        .find_map(|row| row.provenance.discovery.as_ref())
        .ok_or("missing checked registry token discovery")?;
    let args = super::action(&first, "page")?;
    let (_, observations) =
        super::live::witness::invoke(&env, &args, Some(&replay), 2, &mut capture, "baseline")?;
    let request = observations
        .iter()
        .find(|row| row.role.ends_with("::RawPage"))
        .ok_or("missing registry continuation observation")?;
    let inputs = request
        .provenance
        .inputs
        .as_ref()
        .ok_or("missing continuation token input")?;
    if token.namespace != "ctgov-page"
        || token.ids.len() != 1
        || inputs.namespace != token.namespace
        || inputs.ids != token.ids
        || request.provenance.input_context.is_none()
    {
        return Err("continuation provenance did not follow the checked first page".into());
    }
    Ok(())
}
fn diagnostic(
    env: &Environment,
    root: &std::path::Path,
    list: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut args = vec!["--live-check".into()];
    if list {
        args.push("--list".into());
    }
    let output = env
        .command(std::env::current_exe()?, &args)
        .env("BIOMCP_TEST_FIXTURES", root)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "diagnostic failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}
