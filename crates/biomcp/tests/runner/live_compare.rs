//! Match completed endpoint replies, preserving retry and cache evidence.
use super::drift;
use biomcp_fetch::Observation;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn compare(
    before: &[Observation],
    after: &[Observation],
    changes: &mut Vec<String>,
    sources: &mut BTreeMap<String, Value>,
) -> Value {
    totals(after, sources);
    let mut evidence = Vec::new();
    let mut used = BTreeSet::new();
    let mut found = Vec::new();
    // A retry is evidence, never the successful endpoint contract.
    for old in terminal(before) {
        let exact = terminal(after)
            .into_iter()
            .find(|item| item.identity == old.identity);
        let paired = exact.or_else(|| {
            terminal(after).into_iter().find(|item| {
                item.source == old.source
                    && item.role == old.role
                    && !used.contains(&item.identity)
                    && !before.iter().any(|old| old.identity == item.identity)
            })
        });
        let kind = match paired {
            Some(item) => {
                used.insert(item.identity.clone());
                if item.identity != old.identity {
                    "request-plan-changed"
                } else if item.cached {
                    "cache-reused"
                } else {
                    "same-request"
                }
            }
            None => "request-plan-removed",
        };
        if let Some(item) = paired {
            if item.status != old.status {
                found.push(format!(
                    "{}: HTTP status {:?} -> {:?}",
                    old.identity, old.status, item.status
                ));
            } else if !item.cached && item.identity == old.identity {
                found.extend(
                    drift(&old.shape, &item.shape)
                        .into_iter()
                        .map(|s| format!("{}: {s}", item.identity)),
                );
            }
        }
        if kind.starts_with("request-plan") {
            found.push(format!("{}: {kind}", old.identity));
        }
        evidence.push(json!({"kind": kind, "source": old.source, "role": old.role,
            "before_identity": old.identity, "after_identity": paired.map(|item| &item.identity),
            "before_shape_incomplete": old.shape.incomplete,
            "after_shape_incomplete": paired.map(|item| item.shape.incomplete),
            "shape_comparison": match paired {
                None => "unexamined",
                Some(item) if old.shape.incomplete || item.shape.incomplete => "incomplete",
                Some(_) => "complete"
            },
            "before_statuses": statuses(before, &old.identity),
            "after_statuses": paired.map(|item| statuses(after, &item.identity))}));
    }
    for item in terminal(after)
        .into_iter()
        .filter(|item| !before.iter().any(|old| old.identity == item.identity))
    {
        if !used.contains(&item.identity) {
            found.push(format!("{}: request-plan-added", item.identity));
            evidence.push(json!({"kind": "request-plan-added", "source": item.source, "role": item.role,
                "after_identity": item.identity, "shape_comparison": "unexamined",
                "after_shape_incomplete": item.shape.incomplete, "after_statuses": statuses(after, &item.identity)}));
        }
    }
    changes.extend(found.iter().cloned());
    json!({"items": evidence, "omitted": 0, "drift": found, "drift_omitted": 0})
}
fn terminal(items: &[Observation]) -> Vec<&Observation> {
    let mut result: Vec<&Observation> = Vec::new();
    for item in items {
        if let Some(old) = result.iter_mut().find(|old| old.identity == item.identity) {
            // A cache read cannot erase a transport attempt in this command.
            if !item.cached || old.cached {
                *old = item;
            }
        } else {
            result.push(item);
        }
    }
    result
}
fn statuses(items: &[Observation], identity: &str) -> Value {
    let matching: Vec<_> = items
        .iter()
        .filter(|item| item.identity == identity)
        .collect();
    json!({"values": matching.iter().map(|item| item.status).collect::<Vec<_>>(),
        "omitted": 0})
}

fn totals(after: &[Observation], sources: &mut BTreeMap<String, Value>) {
    for item in after.iter().filter(|item| !item.cached) {
        let name = serde_json::to_value(item.source)
            .unwrap_or(Value::Null)
            .as_str()
            .unwrap_or("unknown")
            .to_owned();
        let totals = sources
            .entry(name)
            .or_insert(json!({"attempts": 0, "bytes": 0, "elapsed_ms": 0}));
        for (key, value) in [
            ("attempts", 1),
            ("bytes", item.bytes),
            ("elapsed_ms", item.elapsed_ms),
        ] {
            let previous = totals.get(key).and_then(Value::as_u64).unwrap_or(0);
            if let Some(slot) = totals.get_mut(key) {
                *slot = json!(previous.saturating_add(value));
            }
        }
    }
}
