//! Source refusals survive the shared query request boundary.
use libtest_mimic::Trial;
use replay_support::replay;
use serde_json::Value;
use std::time::Duration;
type Check = Result<(), Box<dyn std::error::Error>>;
fn trials() -> Vec<Trial> {
    vec![Trial::test(
        "gene-identity/query-resolution-keeps-actual-refusals",
        || query_resolution().map_err(|e| e.to_string().into()),
    )]
}
fn query_resolution() -> Check {
    for refused in [false, true] {
        let mut changed = false;
        let (_root, fetch) = replay("gene/gencc", |source, recording| {
            if source == "hgnc"
                && let Some(body) = recording.pointer_mut("/exchanges/0/body/value")
                && let Some(native) = body.as_str()
            {
                changed = true;
                *body = if refused {
                    Value::String(format!(
                        "{native}invalid\tODC1\trefused identity\tprotein-coding gene\tApproved\n"
                    ))
                } else {
                    Value::String(native.replacen(
                        "protein-coding gene",
                        "invalid optional locus group",
                        1,
                    ))
                };
            }
        })?;
        if !changed {
            return Err("HGNC recording did not contain native complete rows".into());
        }
        let query = biomcp_types::GeneGet::new(Box::new(["ODC1".parse()?]))?;
        let mapped = biomcp_query::run(
            &biomcp_query::Ctx {
                fetch: &fetch,
                budget: Duration::from_secs(20),
                fresh: false,
                drug_name_limit: biomcp_query::DRUG_NAME_LIMIT,
                ncbi_email: None,
            },
            &biomcp_types::Request::GetGene(query),
        );
        let expected = if refused {
            biomcp_types::Status::Partial
        } else {
            biomcp_types::Status::Complete
        };
        if mapped.status() != expected
            || !matches!(mapped.body, biomcp_types::ReplyData::GeneCards { body: Some(ref cards), .. } if !cards.items.is_empty())
        {
            return Err(
                "HGNC query aggregation lost a retained gene or its actual refusal fact".into(),
            );
        }
        if !refused
            && !mapped
                .problems
                .iter()
                .any(|p| p.field.as_ref() == "locus_group")
        {
            return Err("HGNC optional-field control did not reach its problem".into());
        }
    }
    Ok(())
}

use biomcp_test_support::replay as replay_support;
fn main() {
    libtest_mimic::run(&libtest_mimic::Arguments::from_args(), trials()).exit();
}
