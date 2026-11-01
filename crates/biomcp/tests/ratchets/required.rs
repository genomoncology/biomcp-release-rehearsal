//! Required public surface comes from shipped help and the advertised MCP registry.
#[path = "../runner/case_data.rs"]
mod case_data;
use biomcp_test_support::environment;
#[path = "../support/help_catalog.rs"]
mod help_catalog;
#[path = "../support/housekeeping_contract.rs"]
mod housekeeping_contract;
#[path = "required/pending.rs"]
mod pending;
#[path = "../support/pool_contracts.rs"]
mod pool_contracts;
#[path = "../runner/process.rs"]
mod process;
#[path = "required/proof.rs"]
mod proof;
#[path = "../support/synthetic_contracts.rs"]
mod synthetic_contracts;
#[path = "required/tally.rs"]
mod tally;
use super::Check;
use serde_json::{Value, json};
use std::{fs, io::Write, path::Path, process::Stdio};

fn metadata() -> Result<(Value, Vec<Value>), Box<dyn std::error::Error>> {
    let home = environment::new()?;
    let binary = Path::new(env!("CARGO_BIN_EXE_biomcp"));
    if !binary.is_file() {
        return Err("required coverage: build the current product binary before ratchets".into());
    }
    let commands = help_catalog::collect(|args| {
        let output = environment::command(binary, home.path(), &args)
            .output()
            .map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err("shipped scoped help failed".into());
        }
        serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())
    })?;
    let help = json!({"result":{"commands":commands}});
    let child = environment::command(binary, home.path(), &["serve".into()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let frames = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2026-07-28","capabilities":{},"clientInfo":{"name":"required-coverage","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    ];
    let (status, stdout, stderr) = process::exchange(child, &frames)?;
    for frame in process::messages(status, &stdout, &stderr)? {
        if frame.get("id") == Some(&json!(2)) {
            return Ok((
                help,
                frame
                    .pointer("/result/tools")
                    .and_then(Value::as_array)
                    .ok_or("MCP tool registry missing")?
                    .clone(),
            ));
        }
    }
    Err("MCP tool registry response missing".into())
}

#[expect(
    clippy::disallowed_methods,
    reason = "Repository ratchets own their working directory boundary"
)]
pub(super) fn required_surface_has_asserted_calls() -> Check {
    let root = std::env::current_dir()?.join("../..");
    let (help, tools) = metadata()?;
    let mut counts = tally::Inventory::new(&help, &tools)?;
    counts.runner(housekeeping_contract::OFFLINE_PROVENANCE.1);
    counts.runner(synthetic_contracts::GWAS_POOL.1);
    counts.runner(synthetic_contracts::GWAS_ANCESTRY.1);
    counts.runner(synthetic_contracts::GWAS_ASSOCIATIONS.1);
    counts.runner(synthetic_contracts::STUDY_PAGES.1);
    counts.runner(synthetic_contracts::STUDY_FILES.1);
    counts.runner(synthetic_contracts::STUDY_SEARCH.1);
    counts.runner(synthetic_contracts::CPIC_VIEW.1);
    counts.runner(synthetic_contracts::CPIC_TABLE.1);
    counts.runner(synthetic_contracts::CPIC_GUIDELINES.1);
    counts.runner(synthetic_contracts::CPIC_DIPLOTYPE.1);
    counts.runner(synthetic_contracts::CPIC_PAIRS.1);
    counts.runner(synthetic_contracts::CLINPGX_CLINICAL.1);
    counts.runner(synthetic_contracts::CLINPGX_KIND.1);
    counts.runner(synthetic_contracts::CLINPGX_ANNOTATION.1);

    counts.runner(pool_contracts::PATHWAY_GENES.1);
    counts.runner(pool_contracts::PATHWAY_SEARCH.1);
    counts.runner(pool_contracts::ORGANIZATION.1);
    counts.runner(pool_contracts::BIOMARKER.1);
    counts.runner(pool_contracts::FEATURES.1);
    counts.runner(pool_contracts::MEMBERS.1);
    counts.runner(pool_contracts::TRIAL.1);
    counts.runner(pool_contracts::ARTICLE_SAVES.1);
    counts.runner(pool_contracts::DOMAIN_PAGES.1);
    counts.runner(pool_contracts::DOMAIN_TYPE.1);
    counts.runner(pool_contracts::ARTICLE_CITATIONS.1);
    counts.runner(pool_contracts::EVIDENCE_FRAGMENTS.1);
    counts.runner(pool_contracts::DRUG_CIVIC.1);
    counts.runner(pool_contracts::WHO_REVISION.1);
    counts.runner(pool_contracts::DRUG_CIVIC_CURSOR.1);
    counts.runner(pool_contracts::VARIANT_CIVIC_CURSOR.1);
    counts.runner(pool_contracts::PATHWAY_EVENTS.1);
    counts.runner(pool_contracts::HTTP_HOST.1);
    counts.runner(pool_contracts::HTTP_CAPACITY.1);
    counts.runner(pool_contracts::HOSTED_MODE.1);
    counts.runner(pool_contracts::HOSTED_PACING.1);
    counts.runner(pool_contracts::SOURCE_PACING.1);

    counts.runner(synthetic_contracts::LABEL_HEADINGS.1);
    counts.runner(synthetic_contracts::ONCOKB.1);
    counts.runner(synthetic_contracts::ONCOKB_ALL.1);
    counts.runner(synthetic_contracts::ALPHAGENOME.1);
    counts.runner(synthetic_contracts::HPO_SNAPSHOT.1);
    counts.runner(synthetic_contracts::MONARCH_XREFS.1);

    counts.runner(synthetic_contracts::FAERS_SECTIONS.1);
    counts.runner(synthetic_contracts::FAERS_IDENTIFIERS.1);
    counts.runner(synthetic_contracts::VALIDATOR_RESOLVER.1);
    counts.runner(synthetic_contracts::VAERS_TABLES.1);
    counts.runner(synthetic_contracts::FAERS_REACTION.1);
    counts.runner(synthetic_contracts::NORMALIZATION_PAGES.1);
    counts.runner(synthetic_contracts::CLINVAR_ASSERTION.1);
    counts.runner(synthetic_contracts::CLINVAR_PAGES.1);
    counts.runner(synthetic_contracts::MONARCH_PHENOTYPES.1);
    counts.runner(synthetic_contracts::POPULATION_SELECTORS.1);
    counts.runner(synthetic_contracts::HOTSPOTS.1);

    counts.runner(synthetic_contracts::MAUDE_SECTIONS.1);

    let mut paths = Vec::new();
    case_data::collect(&root.join("tests/cases"), &mut paths)?;
    paths.sort();
    for path in paths {
        let case = case_data::read_case(&path)?;
        for step in case.steps {
            counts.step(&step);
        }
    }
    let pending = fs::read_to_string(root.join("crates/biomcp/tests/ratchets/required.pending"))?;
    let pending = pending::read(&pending, pending::INITIAL_ADMISSIONS)?;
    let mut output = std::io::stdout().lock();
    for (item, count) in &counts.calls {
        writeln!(output, "required {count:5} {item}")?;
    }
    counts.check(&pending)
}

pub(super) fn controls() -> Check {
    proof::check()?;
    pending::admissions_retire_and_refuse_restoring_a_closed_gap()
}
