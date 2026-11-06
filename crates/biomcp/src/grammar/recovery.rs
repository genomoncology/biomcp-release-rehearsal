//! Suggest only requests accepted by the registry's existing checked constructors.
use biomcp_types::{Rewrite, Step, StepKind};
use serde_json::Value;

pub(super) fn checked(via: Rewrite, path: (&str, &str), value: Value) -> Option<Step> {
    let request = super::rewrite::request(via, path.0, path.1, value).ok()?;
    crate::spell::step(
        StepKind::Configure,
        &biomcp_query::Action::Run {
            request,
            fresh: false,
        },
    )
    .ok()
}

pub(super) fn corrected(
    via: Rewrite,
    path: (&str, &str),
    schema: &Value,
    value: &Value,
    names: &[&str],
    allow_omission: bool,
) -> Option<Step> {
    if let Some(step) = checked(via, path, value.clone()) {
        return Some(step);
    }
    for name in names {
        let property = schema.get("properties")?.get(*name)?;
        if allow_omission && let Some(values) = value.get(*name).and_then(Value::as_array) {
            let accepted: Vec<_> = values
                .iter()
                .filter(|item| {
                    let mut candidate = value.clone();
                    let Some(fields) = candidate.as_object_mut() else {
                        return false;
                    };
                    fields.insert((*name).into(), Value::Array(vec![(*item).clone()]));
                    checked(via, path, candidate).is_some()
                })
                .cloned()
                .collect();
            if !accepted.is_empty() {
                let mut candidate = value.clone();
                candidate
                    .as_object_mut()?
                    .insert((*name).into(), Value::Array(accepted));
                if let Some(step) = checked(via, path, candidate) {
                    return Some(step);
                }
            }
        }
        // Ordinary invalid filters must not disappear from the proposed request.
        if allow_omission {
            let mut candidate = value.clone();
            candidate.as_object_mut()?.remove(*name);
            if let Some(step) = checked(via, path, candidate) {
                return Some(step);
            }
        }
        for replacement in candidates(
            schema,
            property,
            value.get(*name).is_none() || allow_omission,
        ) {
            let mut candidate = value.clone();
            candidate
                .as_object_mut()?
                .insert((*name).into(), replacement);
            if let Some(step) = checked(via, path, candidate) {
                return Some(step);
            }
        }
    }
    None
}

fn candidates(schema: &Value, property: &Value, missing: bool) -> Vec<Value> {
    let resolved = super::validation::resolve(schema, property);
    // A supplied value expresses intent. Only a singleton vocabulary is unambiguous.
    if !missing
        && resolved
            .get("enum")
            .and_then(Value::as_array)
            .is_none_or(|v| v.len() != 1)
    {
        return Vec::new();
    }
    let mut values: Vec<_> = property
        .get("default")
        .filter(|_| missing)
        .into_iter()
        .cloned()
        .chain(
            resolved
                .get("enum")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .cloned(),
        )
        .collect();
    if missing && super::property_type(resolved) == Some("integer") {
        values.push(resolved.get("minimum").cloned().unwrap_or(Value::from(0)));
    }
    if super::property_type(property) == Some("array") {
        values = values
            .into_iter()
            .map(|value| {
                if value.is_array() {
                    value
                } else {
                    Value::Array(vec![value])
                }
            })
            .collect();
    }
    values
}

pub(super) fn next(arguments: &mut biomcp_types::UsageArguments, corrected: Option<Step>) {
    if let Some(step) = corrected {
        arguments.next.insert(0, step);
    }
}

pub(super) fn guidance(next: Step, path: [&str; 2]) -> Vec<Step> {
    let corrected = next
        .args
        .first()
        .is_some_and(|word| word.as_ref() != "help");
    let mut steps = vec![next];
    if corrected {
        steps.push(super::validation::help(path));
    }
    steps
}

/// Restart an unbound trial page without discarding its selection or source filters.
pub(super) fn trial_first_page(
    via: Rewrite,
    path: (&str, &str),
    value: &Value,
    cause: &biomcp_types::QueryInvalid,
) -> Option<Step> {
    if !matches!(
        cause.message(),
        Some(
            biomcp_types::ErrorMessage::TrialSectionPool
                | biomcp_types::ErrorMessage::TrialSearchCursor
        )
    ) {
        return None;
    }
    let mut first = value.clone();
    first
        .as_object_mut()?
        .insert("offset".into(), Value::from(0));
    checked(via, path, first)
}
