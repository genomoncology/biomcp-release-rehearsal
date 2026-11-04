//! Recorded command cases need a local peer to observe cache reuse across processes.
//! Separate replay steps cannot witness a second provider request.
mod cases;
mod controls;
use super::{
    assertions, case_data::read_case, launch, network_stub::Stub, schema::Registry, transcript,
    variant,
};
use libtest_mimic::Trial;
use serde_json::Value;
use std::fs;
type Routes = Vec<(String, String)>;
#[derive(Clone, Copy)]
struct Owner {
    name: &'static str,
    feature: &'static str,
    recording: &'static str,
    rows: &'static str,
    identity: &'static str,
    accepted_identity: &'static str,
    count: &'static str,
    total: &'static str,
    total_field: &'static str,
    string: bool,
    optional: &'static str,
}
pub(super) fn trials() -> Vec<Trial> {
    cases::OWNERS
        .iter()
        .copied()
        .map(|owner| {
            transcript::trial(
                format!("{}/{}-cache-admission", owner.feature, owner.name),
                move || check(&owner).map_err(Into::into),
            )
        })
        .collect()
}
fn check(owner: &Owner) -> Result<(), String> {
    let registry = Registry::embedded()?;
    let path = launch::fixtures().join(format!("cases/cache-admission/{}.yaml", owner.name));
    let case = read_case(&path)?;
    let first = case.steps.first().ok_or("missing cache command")?;
    let recording: Value =
        serde_json::from_str(&launch::fixture_text(owner.recording)?).map_err(|e| e.to_string())?;
    let url = recording
        .get("url")
        .and_then(Value::as_str)
        .ok_or("recorded URL")?;
    let parsed = url::Url::parse(url).map_err(|e| e.to_string())?;
    let target = format!(
        "GET {}{} HTTP/1.1",
        parsed.path(),
        parsed
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default()
    );
    let mut failures = Vec::new();
    // Positive controls qualify the exact command and peer before the regression witness.
    for &control in controls::kinds(owner) {
        let (hosts, mut routes) = routes("cache-admission")?;
        let entry = routes
            .iter_mut()
            .find(|(request, _)| request == &target)
            .ok_or("missing recorded route")?;
        let (step, body) = controls::prepare(owner, first, control, &entry.1)?;
        entry.1 = body;
        let stub = Stub::for_sources(
            &hosts.iter().map(String::as_str).collect::<Vec<_>>(),
            |_| "application/json",
            move |request| {
                let line = request.lines().next().ok_or("missing request line")?;
                routes
                    .iter()
                    .find(|(target, _)| target == line)
                    .map(|(_, body)| body.clone())
                    .ok_or_else(|| format!("unexpected native request {line}"))
            },
        )?;
        let env = variant::source_environment(&stub)?;
        for index in 0..2 {
            transcript::case_step(index);
            super::run_sequence(
                &step,
                &env,
                &assertions::prepare(&step)?,
                &registry,
                None,
                None,
                None,
            )
            .map_err(|error| format!("{} {control} call {}: {error}", owner.name, index + 1))?;
        }
        let requests = stub.finish()?;
        if let Some(failure) = controls::requests(owner, control, &target, &requests)? {
            failures.push(failure);
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
pub(super) fn routes(recording: &str) -> Result<(Vec<String>, Routes), String> {
    let mut routes = Vec::new();
    let mut hosts = Vec::new();
    for source in fs::read_dir(launch::fixtures().join("replies").join(recording))
        .map_err(|e| e.to_string())?
    {
        let source = source.map_err(|e| e.to_string())?.path();
        if !source.is_dir() {
            continue;
        }
        for file in fs::read_dir(source).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            let record: Value =
                serde_json::from_slice(&fs::read(file.path()).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let url = url::Url::parse(
                record
                    .get("url")
                    .and_then(Value::as_str)
                    .ok_or("recorded URL")?,
            )
            .map_err(|e| e.to_string())?;
            let host = url.host_str().ok_or("recorded host")?.to_owned();
            if !hosts.contains(&host) {
                hosts.push(host);
            }
            let target = format!(
                "GET {}{} HTTP/1.1",
                url.path(),
                url.query()
                    .map(|query| format!("?{query}"))
                    .unwrap_or_default()
            );
            let body = record
                .pointer("/exchanges/0/body/value")
                .ok_or("recorded body")?;
            let body = if record
                .pointer("/exchanges/0/body/kind")
                .and_then(Value::as_str)
                == Some("text")
            {
                body.as_str().ok_or("recorded text body")?.to_owned()
            } else {
                body.to_string()
            };
            routes.push((target, body));
        }
    }
    Ok((hosts, routes))
}
