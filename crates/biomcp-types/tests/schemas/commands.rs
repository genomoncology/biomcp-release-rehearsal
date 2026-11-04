//! Bind command paths and query schema publishing to the shared command declaration.
use super::*;

macro_rules! query_contracts {
    (rewrites { $($rewrite:ident),+ } $( $(#[$doc:meta])* $variant:ident($query:ident) =>
        ($verb:literal, $entity:literal, $schema_type:ident, $stem:literal, $schema:literal,
         $grammar_type:ident, $grammar_stem:literal, $grammar_schema:literal, $spelling_schema:literal, $order:literal, $via:ident); )+) => {{
        let contracts = contracts! {
            $($schema_type => ($stem, $schema, query),
              $grammar_type => ($grammar_stem, $grammar_schema, query),)+
        };
        let mut seen = std::collections::BTreeSet::new();
        contracts.into_iter().filter(|contract| seen.insert(contract.id)).collect()
    }};
}

pub(super) fn registry() -> Vec<Contract> {
    biomcp_types::command_types!(query_contracts)
}

fn validate(
    rows: &[CommandSpec],
    schemas: &std::collections::BTreeSet<String>,
) -> Result<(), String> {
    for (index, row) in rows.iter().enumerate() {
        if rows.iter().take(index).any(|prior| {
            prior.order == row.order && (prior.verb, prior.entity) != (row.verb, row.entity)
        }) {
            return Err("presentation order names different command paths".into());
        }
        if !schemas.contains(row.schema) {
            return Err(format!("command schema has no file: {}", row.schema));
        }
        if let Some(prior) = rows
            .iter()
            .take(index)
            .find(|prior| (prior.verb, prior.entity) == (row.verb, row.entity))
            && (row.via == Rewrite::Direct
                || row.via != prior.via
                || row.schema != prior.schema
                || row.order != prior.order)
        {
            return Err(format!(
                "duplicate path has no shared rewrite: {} {}",
                row.verb, row.entity
            ));
        }
    }
    Ok(())
}

pub(super) fn every_path_has_one_grammar_or_one_shared_rewrite(
    schemas: &std::collections::BTreeSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    validate(COMMANDS, schemas)?;
    let mut duplicate = COMMANDS.to_vec();
    let first = *COMMANDS.first().ok_or("empty command declaration")?;
    let mut row = first;
    row.via = Rewrite::Direct;
    duplicate.push(row);
    assert!(
        validate(&duplicate, schemas).is_err(),
        "an unshared duplicate path must fail"
    );
    let missing = CommandSpec {
        schema: "biomcp.missing-command/1",
        ..first
    };
    assert!(
        validate(&[missing], schemas).is_err(),
        "a schema without a file must fail"
    );
    Ok(())
}
