mod corporate;
mod credential_containment;
mod drug_identity;
mod key_reload;
mod local_files;
tool_only! { mod protein_commands; }
tool_only! { mod trial_condition; }
tool_only! { pub(super) mod other_source_value_proof; }
mod article_save;
mod article_save_reply;
mod assertions;
pub(super) mod case_main;
tool_only! { pub(super) mod cell_line; }
tool_only! { mod disease_mydisease; }
tool_only! { mod disease_ols; }
tool_only! { mod disease_seer; }
mod execution;
mod faers_search;
mod native_fields;
#[path = "../support/pool_contracts.rs"]
mod pool_contract;
#[path = "../support/synthetic_contracts.rs"]
mod required_contract;
tool_only! { pub(super) mod tools_api; }
mod features;
use crate::runner::transcript::trial as transcript_trial;
use execution::run_step;
mod alphagenome;
mod alphagenome_tls;
mod cache_admission;
mod cache_contention;
mod capture;
mod citation_values;
mod clinpgx;
mod configuration;
mod cpic;
mod diagnostic_cdx;
mod diagnostic_retirements;
mod diagnostic_who;
mod disease_monarch;
mod drug_civic;
mod drug_civic_fixtures;
mod drug_label;
mod drug_next;
mod evidence;
mod faers_get;
tool_only! { mod hgnc_sync; }
mod held_proxy;
mod hosted_deadline;
mod hosted_pacing;
mod housekeeping;
mod http;
mod http_limits;
#[cfg(unix)]
mod key_input;
mod key_registration;
pub(super) mod launch;
mod recurrence;
mod remote_tools;
mod replay;
tool_only! { pub(crate) mod live; }
tool_only! { mod live_follow_proof; }
tool_only! { mod live_proof; }
mod licensed_cache;
mod maude;
mod maude_budget;
mod mcp;
mod mcp_boundaries;
mod mcp_hosted;
mod nci;
mod request_keys;
tool_only! { mod negative_cases; }
use biomcp_test_support::network_stub;
mod normalization;
mod oncokb;
mod oncokb_context;
mod output_stream;
mod pathway_budgets;
mod pathway_commands;
mod pathway_get_pages;
mod pathway_sections;
mod pdf;
mod pgx_retirements;
mod phenotype;
mod process;
pub(super) mod proof;
mod protein_budgets;
mod protein_domains;
mod protein_sections;
mod protocol;
mod publication;
mod reply_comparison;
mod research;
mod response_cases;
mod schema;
mod skill;
tool_only! { mod schema_evolution; }
mod schema_proof;
tool_only! { mod semantic_tables; }
mod cache_errors;
mod snapshot_errors;
mod source_queries;
mod source_terms;
mod store_clean;
mod store_coexistence;
mod study_catalog;
mod transcript;
mod trial_get;
mod trial_sites;
mod vaers;
mod variant;
mod variant_civic;
mod variant_hotspots;
mod variant_population;
mod wikipathways;

pub(super) fn output_limit(name: &str) -> Result<usize, String> {
    let limits: std::collections::BTreeMap<String, usize> =
        serde_saphyr::from_str(&launch::fixture_text("runner-output-budgets.yaml")?)
            .map_err(|error| error.to_string())?;
    limits
        .get(name)
        .copied()
        .ok_or_else(|| format!("missing output budget: {name}"))
}

use libtest_mimic::Trial;
use serde_json::Value;
use std::{fs, path::Path, sync::Arc};

mod case_data;
use case_data::{Case, Step, collect, read_case};

pub(super) fn discover(root: &Path) -> Result<Vec<Trial>, String> {
    let mut paths = Vec::new();
    collect(root, &mut paths).map_err(|error| format!("{}: {error}", root.display()))?;
    paths.sort();
    if paths.is_empty() {
        return Err(format!("{}: no YAML cases", root.display()));
    }
    let registry = schema::Registry::embedded()?;
    let trials = paths
        .into_iter()
        .map(|path| {
            let name = path
                .strip_prefix(root)
                .map_err(|error| error.to_string())?
                .iter()
                .map(|part| part.to_str().ok_or("case path must be UTF-8"))
                .collect::<Result<Vec<_>, _>>()?
                .join("/");
            let case = read_case(&path)?;
            let name = format!("{}/{name}", case.feature);
            let registry = Arc::clone(&registry);
            Ok(transcript_trial(name, move || {
                run_case(&path, &registry).map(|_| ()).map_err(Into::into)
            }))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(trials)
}

pub(super) fn validate_features(trials: &[Trial], complete: bool) -> Result<(), String> {
    features::validate(
        launch::feature_list(),
        trials.iter().map(Trial::name),
        complete,
    )
}

fn run_case(path: &Path, registry: &schema::Registry) -> Result<Value, String> {
    run_case_owned(path, registry, None)
}

fn run_case_owned(
    path: &Path,
    registry: &schema::Registry,
    replay_owner: Option<Arc<tempfile::TempDir>>,
) -> Result<Value, String> {
    let mut case = read_case(path)?;
    if case.story.trim().is_empty() || case.steps.is_empty() {
        return Err("case requires a story and at least one step".into());
    }
    load_floors(&mut case, path)?;
    let expectations = case
        .steps
        .iter()
        .enumerate()
        .map(|(index, step)| {
            if step.mcp_call.is_some()
                && (step.run.is_some()
                    || step.mcp.is_some()
                    || step.follow.is_some()
                    || step.walk.is_some())
            {
                return Err("named MCP call requires its own step".into());
            }
            if step.run.is_none()
                && step.mcp.is_none()
                && step.mcp_call.is_none()
                && step.follow.is_none()
            {
                return Err("step requires run, mcp or mcp_call".into());
            }
            assertions::prepare(step)
                .map_err(|error| format!("{}; step {}: {error}", case.story, index + 1))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut environment = launch::Environment::new().map_err(|error| error.to_string())?;
    if let Some(owner) = replay_owner {
        environment = environment.with_replay_owner(owner);
    }
    if let Some(config) = &case.config {
        environment
            .config(config)
            .map_err(|error| error.to_string())?;
    }
    let replay = response_cases::replay(&environment, case.replay.as_deref(), case.cache_acquired)?;
    let mut previous = None;
    for (index, (step, expectations)) in case.steps.iter().zip(&expectations).enumerate() {
        transcript::case_step(index);
        previous = Some(
            run_sequence(
                step,
                &environment,
                expectations,
                registry,
                replay.as_deref(),
                previous.as_ref(),
                index.checked_sub(1).and_then(|index| case.steps.get(index)),
            )
            .map_err(|error| format!("{}; step {}: {error}", case.story, index + 1))?,
        );
    }
    previous.ok_or_else(|| "case requires at least one step".into())
}

pub(super) fn protocol_trials() -> Vec<Trial> {
    let mut trials = Vec::new();
    trials.extend(protocol::trials());
    trials.extend(configuration::trials());
    trials.extend(recurrence::trials());
    trials.extend(replay::trials());
    trials.extend(output_stream::trials());
    trials.extend(research::trials());
    trials.extend(cache_contention::trials());
    trials.extend(cache_admission::trials());
    trials.extend(corporate::trials());
    trials.extend(drug_identity::trials());
    trials.extend(source_queries::trials());
    trials.extend(key_registration::trials());
    trials.extend(pdf::trials());
    trials.push(skill::trial());
    trials.extend(credential_containment::trials());
    trials.extend(http::trials());
    trials.extend(remote_tools::trials());
    trials.extend(mcp_boundaries::trials());
    trials.extend(mcp_hosted::trials());
    trials.extend(request_keys::trials());
    trials.extend(hosted_pacing::trials());
    trials.extend(hosted_deadline::trials());
    trials.extend(licensed_cache::trials());
    trials.extend(alphagenome_tls::trials());
    trials.extend(housekeeping::trials());
    trials.extend(clinpgx::trials());
    trials.extend(variant_hotspots::trials());
    trials.extend(vaers::trials());
    trials.extend(trial_get::trials());
    trials.push(trial_sites::trial());
    trials.extend(snapshot_errors::trials());
    trials.extend(cache_errors::trials());
    trials.extend(variant_assertion_wire::trials());
    trials.extend(evidence::trials());
    trials.extend(variant::trials());
    trials.extend(variant_population::trials());
    trials.extend(oncokb_context::trials());
    trials.extend(variant_assertions::trials());
    trials.extend(pathway_sections::trials());
    trials.extend(oncokb::trials());
    trials.extend(protein_sections::trials());
    trials.extend(drug_label::trials());
    trials.extend(study_catalog::trials());
    trials.extend(pathway_commands::trials());
    trials.extend(gwas_search::trials());
    trials.extend(diagnostic_who::trials());
    trials.push(wikipathways::trial());
    trials.extend(variant_civic::trials());
    trials.extend(protein_domains::trials());
    trials.extend(faers_get::trials());
    trials.extend(faers_search::trials());
    trials.extend(cpic::trials());
    trials.extend(maude::trials());
    trials.extend(disease_monarch::trials());
    trials.extend(normalization::trials());
    trials.extend(gwas_get::trials());
    trials.extend(phenotype::trials());
    trials.extend(drug_next::trials());
    trials.extend(nci::trials());
    trials.extend(drug_civic::trials());
    trials.push(citation_values::trial());
    trials.push(transcript_trial(pool_contract::ARTICLE_SAVES.0, || {
        article_save::check().map_err(Into::into)
    }));
    trials.push(transcript_trial(
        "mcp-parity/ping-passes-held-provider-call",
        || held_proxy::check().map_err(Into::into),
    ));
    trials
}

fn load_floors(case: &mut Case, path: &Path) -> Result<(), String> {
    for step in &mut case.steps {
        for expected in step.expect.values_mut().filter_map(Value::as_object_mut) {
            for (key, value) in expected.iter_mut() {
                if !matches!(key.as_str(), "includes_all_of" | "equals_file") {
                    continue;
                }
                let Some(file) = value.as_str() else {
                    continue;
                };
                if !Path::new(file)
                    .components()
                    .all(|p| matches!(p, std::path::Component::Normal(_)))
                {
                    return Err("floor must be beside its case".into());
                }
                let floor = path.parent().ok_or("case has no parent")?.join(file);
                *value = serde_json::from_slice(&fs::read(floor).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
fn action(previous: &Value, kind: &str) -> Result<Vec<String>, String> {
    let steps = previous
        .get("next")
        .and_then(Value::as_array)
        .ok_or("follow action absent")?;
    let action = steps
        .iter()
        .find(|s| s.get("kind").and_then(Value::as_str) == Some(kind))
        .ok_or("follow action absent")?;
    action
        .get("args")
        .and_then(Value::as_array)
        .ok_or("follow requires typed arguments")?
        .iter()
        .map(|a| {
            a.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "follow requires typed arguments".into())
        })
        .collect()
}
fn run_sequence(
    step: &Step,
    environment: &launch::Environment,
    expectations: &assertions::Expectations<'_>,
    registry: &schema::Registry,
    replay: Option<&Path>,
    previous: Option<&Value>,
    previous_step: Option<&Step>,
) -> Result<Value, String> {
    let paired = assertions::markdown_pair(step, previous_step, previous)?;
    run_sequence_with(step, expectations, previous, |current, expectations| {
        run_step(current, environment, expectations, registry, replay, paired)
    })
}

fn run_sequence_with(
    step: &Step,
    expectations: &assertions::Expectations<'_>,
    previous: Option<&Value>,
    mut execute: impl FnMut(&Step, &assertions::Expectations<'_>) -> Result<Value, String>,
) -> Result<Value, String> {
    let mut current = step.clone();
    if let Some(kind) = &step.follow {
        current.run = Some(action(
            previous.ok_or("follow requires previous output")?,
            kind,
        )?);
        current.mcp = None;
    }
    let Some(walk) = &step.walk else {
        return execute(&current, expectations);
    };
    if walk.max_pages == 0 || walk.max_pages > 1000 {
        return Err("walk requires a bounded page count".into());
    }
    let mut actions = std::collections::BTreeSet::new();
    let mut offsets = std::collections::BTreeSet::new();
    let mut items = Vec::new();
    let mut aggregate = None;
    for _ in 0..walk.max_pages {
        if !actions.insert(current.run.clone().ok_or("walk requires run")?) {
            return Err("walk repeated action".into());
        }
        let value = execute(&current, &Vec::new())?;
        let offset = value
            .pointer("/page/offset")
            .and_then(Value::as_u64)
            .ok_or("walk requires page offset")?;
        if !offsets.insert(offset) {
            return Err("walk repeated offset".into());
        }
        items.extend(
            value
                .pointer(&walk.items)
                .and_then(Value::as_array)
                .ok_or("walk items must select an array")?
                .iter()
                .cloned(),
        );
        if aggregate.is_none() {
            aggregate = Some(value.clone());
        }
        let next = action(&value, "page");
        if let Ok(args) = next {
            current.run = Some(args);
            current.mcp = None;
        } else {
            let mut result = aggregate.ok_or("walk empty")?;
            *result
                .pointer_mut(&walk.items)
                .ok_or("walk missing aggregate pointer")? = Value::Array(items);
            let next = value
                .get("next")
                .cloned()
                .unwrap_or_else(|| Value::Array(Vec::new()));
            result
                .as_object_mut()
                .ok_or("walk result must be object")?
                .insert("next".into(), next);
            let mut checked = step.clone();
            checked.max_bytes = None;
            checked.max_tokens = None;
            assertions::check(
                &serde_json::to_vec(&result).map_err(|e| e.to_string())?,
                &checked,
                expectations,
            )?;
            return Ok(result);
        }
    }
    Err("walk exceeded max_pages".into())
}

mod variant_assertion_wire;
mod variant_assertions;

mod gwas_budgets;
mod gwas_get;
mod gwas_search;

mod reply_readback;

tool_only! {
    pub(super) fn tool_trials() -> Result<Vec<Trial>, String> {
        let registry = schema::Registry::embedded()?;
        let mut trials = proof::trials();
        trials.push(reply_comparison::trial());
        trials.extend(hgnc_sync::trials());
        trials.extend(schema_proof::trials(registry));
        trials.extend(source_terms::trials());
        trials.extend(live_proof::trials());
        trials.extend(live_follow_proof::trials());
        Ok(trials)
    }
}

use biomcp_test_support::capture as capture_support;
use biomcp_test_support::fixtures as fixture_paths;
use biomcp_test_support::hash as stable_hash;

use biomcp_test_support::recordings as recording_support;
