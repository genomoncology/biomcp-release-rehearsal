//! Retained native source proof callbacks, excluded from routine CLI trial registration.
use super::{configuration::invoke, network_stub::Stub, schema::Registry};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{fs, path::Path};
type Check<T = ()> = Result<T, String>;
#[path = "other_source_value_proof/bindings.rs"]
mod bindings;
#[path = "other_source_value_proof/gwas.rs"]
mod gwas;
#[path = "other_source_value_proof/paging.rs"]
mod paging;
#[path = "other_source_value_proof/retained.rs"]
mod retained;
#[derive(Clone, Deserialize)]
struct Route {
    file: String,
    request: String,
}
#[derive(Deserialize)]
struct NativeCase {
    #[serde(skip)]
    host: &'static str,
    #[serde(skip)]
    root: &'static str,
    args: Vec<String>,
    requests: Vec<Route>,
}
struct Mutation {
    file: &'static str,
    pointer: &'static str,
    field: &'static str,
    absent: &'static str,
    present: &'static str,
}
/// Run positive native controls before response-only mutations and retain each result.
pub(super) fn run(directory: &Path) -> Check {
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let registry = Registry::embedded()?;
    let mutations = mutations();
    let mut failures = Vec::new();
    for mutation in &mutations {
        // A failed positive control is setup failure, never a regression proof.
        let case = native_case(mutation.file)?;
        let positive = acquire(&case, None, &registry)?;
        if positive.pointer(mutation.present).is_none() {
            return Err(format!("native positive control lost {}", mutation.present));
        }
        save(
            directory,
            &format!("{}-positive", mutation.field),
            &positive,
        )?;
        let result = acquire(
            &case,
            Some((mutation, json!({"bad_shape":true}))),
            &registry,
        )
        .and_then(|value| {
            save(directory, mutation.field, &value)?;
            rejected(&value, mutation)
        });
        if let Err(error) = result {
            failures.push(format!("{}: {error}", mutation.field));
        }
    }
    // These exact native String values already belonged to the accepted source domain.
    let stage = mutations.last().ok_or("missing native stage proof")?;
    let case = native_case(stage.file)?;
    for (name, text) in [
        ("empty", ""),
        ("spaces", "  native stage  "),
        ("control", "a\u{1}b"),
    ] {
        let result = acquire(&case, Some((stage, json!(text))), &registry).and_then(|value| {
            save(directory, &format!("stage-{name}"), &value)?;
            if value.pointer("/result/indications/data/records/0/stage/value") != Some(&json!(text))
            {
                return Err("native stage bytes changed".into());
            }
            Ok(())
        });
        if let Err(error) = result {
            failures.push(format!("stage-{name}: {error}"));
        }
    }
    independent_proofs(directory, &registry, &mut failures);
    let large = "x".repeat(biodata::Text::MAX_CHARS + 1);
    let large_result = acquire(&case, Some((stage, json!(large))), &registry).and_then(|value| {
        if value
            .pointer("/result/indications/data/records/0/stage/value")
            .and_then(Value::as_str)
            != Some(large.as_str())
        {
            return Err("large native stage bytes changed".into());
        }
        save(directory, "stage-large", &value)
    });
    if let Err(error) = large_result {
        failures.push(format!("stage-large: {error}"));
    }
    save(directory, "failures", &json!(failures))?;
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
fn independent_proofs(directory: &Path, registry: &Registry, failures: &mut Vec<String>) {
    for (name, proof) in [
        (
            "request-scope",
            bindings::mechanism as fn(&Path, &Registry) -> Check,
        ),
        ("cell-request-scope", bindings::cell),
        ("chembl-native-window", paging::chembl),
        ("seer-native-window", paging::seer),
        ("study-native-window", paging::study),
        ("gwas-native-window", gwas::run),
    ] {
        if let Err(error) = proof(directory, registry) {
            failures.push(format!("{name}: {error}"));
        }
    }
}

fn native_case(file: &str) -> Check<NativeCase> {
    if let Some(case) = retained::case(file)? {
        return Ok(case);
    }
    if file == "010.json" {
        return Ok(NativeCase {
            host: "www.ebi.ac.uk",
            root: "fixtures/drug/native",
            args: ["get", "drug", "CHEMBL941", "--source", "chembl"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            requests: vec![Route {
                file: file.into(),
                request: "GET /chembl/api/data/molecule/CHEMBL941.json HTTP/1.1".into(),
            }],
        });
    }
    let plan = super::launch::fixture_text("fixtures/drug/native/plan.json")?;
    let cases: Vec<NativeCase> = serde_json::from_str(&plan).map_err(|e| e.to_string())?;
    let mut selected = cases
        .into_iter()
        .filter(|case| case.requests.iter().any(|route| route.file == file));
    let mut case = selected.next().ok_or("missing retained native case")?;
    if selected.next().is_some() {
        return Err("ambiguous retained native case".into());
    }
    case.host = "www.ebi.ac.uk";
    case.root = "fixtures/drug/native";
    Ok(case)
}
fn acquire(
    case: &NativeCase,
    mutation: Option<(&Mutation, Value)>,
    registry: &Registry,
) -> Check<Value> {
    acquire_at_exit(case, mutation, registry, 0)
}
fn acquire_at_exit(
    case: &NativeCase,
    mutation: Option<(&Mutation, Value)>,
    registry: &Registry,
    exit: i32,
) -> Check<Value> {
    let root = super::launch::fixtures().join(case.root);
    let mut routes = Vec::new();
    let mut changed = 0;
    for route in &case.requests {
        let body = fs::read_to_string(root.join(&route.file)).map_err(|e| e.to_string())?;
        let body = if let Some((mutation, value)) = &mutation {
            if route.file == mutation.file {
                let mut body: Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
                *body
                    .pointer_mut(mutation.pointer)
                    .ok_or("missing native mutation field")? = value.clone();
                changed += 1;
                body.to_string()
            } else {
                body
            }
        } else {
            body
        };
        routes.push((route.request.clone(), body));
    }
    if mutation.is_some() && changed != 1 {
        return Err("native mutation count differs from one".into());
    }
    let expected_requests = routes.len();
    let stub = Stub::for_sources(
        &[case.host],
        |_| "application/json",
        move |request| {
            let line = request
                .lines()
                .next()
                .ok_or("missing native request line")?;
            routes
                .iter()
                .find(|(expected, _)| expected == line)
                .map(|(_, body)| body.clone())
                .ok_or_else(|| format!("undeclared native request: {line}"))
        },
    )?;
    let env = super::variant::source_environment(&stub)?;
    let mut args = case.args.clone();
    args.push("--fresh".into());
    let result = invoke(&env, args, exit);
    let received = stub.finish()?.len();
    if received != expected_requests {
        return Err("native request count changed".into());
    }
    let value = result?;
    registry.validate(&value, &case.args, exit == 0)?;
    Ok(value)
}
fn rejected(value: &Value, mutation: &Mutation) -> Check {
    if value.pointer(mutation.present).is_none() || value.pointer(mutation.absent).is_some() {
        return Err("native identity or field omission contract failed".into());
    }
    let problems = value
        .get("problems")
        .and_then(Value::as_array)
        .ok_or("missing Problems")?;
    if problems.len() != 1
        || problems.first().and_then(|p| p.get("field")) != Some(&json!(mutation.field))
    {
        return Err("expected one precise native field Problem".into());
    }
    Ok(())
}
fn save(directory: &Path, name: &str, value: &Value) -> Check {
    fs::write(
        directory.join(format!("{name}.json")),
        serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn mutations() -> [Mutation; 5] {
    [
        Mutation {
            file: "010.json",
            pointer: "/pref_name",
            field: "pref_name",
            absent: "/result/header/name",
            present: "/result/header/id",
        },
        Mutation {
            file: "015.json",
            pointer: "/mechanisms/0/action_type",
            field: "action_type",
            absent: "/result/targets/data/records/0/relation",
            present: "/result/targets/data/records/0/native_target/id",
        },
        Mutation {
            file: "008.json",
            pointer: "/_version",
            field: "_version",
            absent: "/result/version",
            present: "/result/header/id",
        },
        Mutation {
            file: "native-projected-k562.json",
            pointer: "/Cellosaurus/cell-line-list/0/sex",
            field: "sex",
            absent: "/result/sex",
            present: "/result/header/accession",
        },
        Mutation {
            file: "014.json",
            pointer: "/drug_indications/0/max_phase_for_ind",
            field: "max_phase_for_ind",
            absent: "/result/indications/data/records/0/stage",
            present: "/result/indications/data/records/0/drug/id",
        },
    ]
}
