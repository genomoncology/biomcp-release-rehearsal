//! The registry's unusable replies are asked for again; its accepted allele is kept.
use super::routes;
use crate::runner::{
    assertions, case_data::read_case, launch, network_stub::Stub, schema::Registry, transcript,
    variant,
};
use libtest_mimic::Trial;
use serde_json::{Value, json};

pub(super) fn trials() -> Vec<Trial> {
    vec![transcript::trial(
        "variant-normalization/registry-cache-admission",
        || check().map_err(Into::into),
    )]
}

// Each control is run twice against one peer: an accepted allele is requested once,
// an unusable one (wrong identity, or an error code on a 2xx reply) is requested again.
const CONTROLS: &[(&str, i32, usize)] = &[
    ("accepted", 0, 1),
    ("wrong-identity", 4, 2),
    ("error-reply", 4, 2),
];

fn check() -> Result<(), String> {
    let registry = Registry::embedded()?;
    let path = launch::fixtures().join("cases/cache-admission/car-normalize.yaml");
    let case = read_case(&path)?;
    let first = case.steps.first().ok_or("missing registry command")?;
    let mut failures = Vec::new();
    for &(control, exit, expected) in CONTROLS {
        let (hosts, mut routes) = routes("cache-admission")?;
        let entry = routes
            .iter_mut()
            .find(|(request, _)| request.starts_with("GET /allele?hgvs="))
            .ok_or("missing recorded registry route")?;
        let target = entry.0.clone();
        let mut body: Value = serde_json::from_str(&entry.1).map_err(|e| e.to_string())?;
        let mut step = first.clone();
        match control {
            "wrong-identity" => {
                *body.pointer_mut("/@id").ok_or("registry identity")? =
                    json!("http://reg.genome.network/allele/not-an-id");
            }
            "error-reply" => {
                body.as_object_mut()
                    .ok_or("registry allele")?
                    .insert("errorType".into(), json!("InternalServerError"));
            }
            _ => {}
        }
        if control != "accepted" {
            step.expect.clear();
            step.exit = exit;
        }
        entry.1 = body.to_string();
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
            super::super::run_sequence(
                &step,
                &env,
                &assertions::prepare(&step)?,
                &registry,
                None,
                None,
                None,
            )
            .map_err(|error| format!("registry {control} call {}: {error}", index + 1))?;
        }
        let requests = stub.finish()?;
        let received = requests
            .iter()
            .filter(|(_, request)| request.lines().next() == Some(target.as_str()))
            .count();
        if received != expected {
            failures.push(format!(
                "registry {control}: expected {expected} native requests, received {received}"
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
