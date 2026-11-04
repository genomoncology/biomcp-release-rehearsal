//! Two commands share a cache while the recorded peer counts actual source reads.
use crate::runner::{
    assertions, cache_admission, case_data::read_case, launch, network_stub::Stub,
    schema::Registry, transcript,
};
use serde_json::{Value, json};
pub(super) fn check() -> Result<(), String> {
    let case = read_case(&launch::fixtures().join("cases/variant/recurrence-source-query.yaml"))?;
    let registry = Registry::embedded()?;
    let first = case.steps.first().ok_or("recurrence command")?;
    for control in ["empty", "optional", "refused"] {
        let (hosts, mut routes) = cache_admission::routes("variant/recurrence")?;
        let entry = routes
            .iter_mut()
            .find(|(target, _)| target.contains("/mutations?"))
            .ok_or("recorded mutation page")?;
        let target = entry.0.clone();
        let step = prepare(first, control, &mut entry.1)?;
        let stub = Stub::for_sources(
            &hosts.iter().map(String::as_str).collect::<Vec<_>>(),
            |request| {
                if request.contains("hgnc_complete_set") {
                    "text/tab-separated-values"
                } else {
                    "application/json"
                }
            },
            move |request| {
                let line = request.lines().next().ok_or("request line")?;
                routes
                    .iter()
                    .find(|(target, _)| target == line)
                    .map(|(_, body)| body.clone())
                    .ok_or_else(|| format!("unexpected recorded request {line}"))
            },
        )?;
        let env = launch::Environment::new().map_err(|e| e.to_string())?;
        super::config(&env, &stub)?;
        for index in 0..2 {
            transcript::case_step(index);
            crate::runner::run_sequence(
                &step,
                &env,
                &assertions::prepare(&step)?,
                &registry,
                None,
                None,
                None,
            )
            .map_err(|e| format!("mutations {control} call {}: {e}", index + 1))?;
        }
        let requests = stub.finish()?;
        if let Some(transcript) = transcript::current() {
            transcript.record(
                &format!("cache/mutations/{control}/requests"),
                serde_json::to_value(&requests).map_err(|e| e.to_string())?,
            )?;
        }
        let received = requests
            .iter()
            .filter(|(_, request)| request.lines().next() == Some(&target))
            .count();
        let expected = if control == "refused" { 2 } else { 1 };
        if received != expected {
            return Err(format!(
                "mutations {control}: expected {expected} native requests, received {received}"
            ));
        }
    }
    Ok(())
}

fn prepare(
    first: &crate::runner::Step,
    control: &str,
    wire: &mut String,
) -> Result<crate::runner::Step, String> {
    let mut body: Value = serde_json::from_str(wire).map_err(|e| e.to_string())?;
    if control == "empty" {
        body = json!([]);
    } else {
        for row in body.as_array_mut().ok_or("native mutation array")? {
            row[if control == "optional" {
                "chr"
            } else {
                "sampleId"
            }] = json!({});
        }
    }
    *wire = body.to_string();
    let mut step = first.clone();
    step.expect.clear();
    step.expect.insert(
        "/result/examined_rows".into(),
        json!(if control == "empty" { 0 } else { 3 }),
    );
    step.expect
        .insert("/result/more".into(), json!(control != "empty"));
    step.expect
        .insert("/result/all_pages_read".into(), json!(control == "empty"));
    if control != "empty" {
        step.expect.insert(
            "/problems/*/field".into(),
            json!({"includes":if control == "optional" {"chr"} else {"sampleId"}}),
        );
    }
    Ok(step)
}
