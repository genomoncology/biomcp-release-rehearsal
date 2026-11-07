//! Citation contexts belong to the citing paper; abstracts follow source terms.
use super::{Licenses, licenses, permitted, pmid};
use biodata::Pmid;
use biomcp_fetch::{BadReply, Leg};
use biomcp_types::Use;
use serde_json::Value;

pub(crate) fn filter(
    leg: &Leg<'_>,
    body: &[u8],
    parent: Option<Pmid>,
) -> Result<Vec<u8>, BadReply> {
    filter_input(leg, body, parent, false)
}
pub(crate) fn snapshot(leg: &Leg<'_>, body: &[u8]) -> Result<Vec<u8>, BadReply> {
    filter_input(leg, body, None, true)
}
fn filter_input(
    leg: &Leg<'_>,
    body: &[u8],
    parent: Option<Pmid>,
    saved: bool,
) -> Result<Vec<u8>, BadReply> {
    if !leg.disabled_use(Use::PerRecord) {
        return Ok(body.to_vec());
    }
    let root: Value = serde_json::from_slice(body).map_err(|_| BadReply)?;
    if !has_text(&root) {
        return Ok(body.to_vec());
    }
    let mut ids = Vec::new();
    if let Some(parent) = parent {
        ids.push(parent);
    }
    collect(&root, &mut ids, saved);
    let licenses = if ids.is_empty() {
        Licenses::default()
    } else {
        licenses(leg, &ids)
    };
    filter_with_licenses(root, parent, &licenses, saved)
}
pub(super) fn filter_with_licenses(
    mut root: Value,
    parent: Option<Pmid>,
    licenses: &Licenses,
    saved: bool,
) -> Result<Vec<u8>, BadReply> {
    apply(&mut root, parent, licenses, saved)?;
    serde_json::to_vec(&root).map_err(|_| BadReply)
}
fn id(value: &Value, saved: bool) -> Option<Pmid> {
    if !saved {
        return value.pointer("/externalIds/PubMed").and_then(pmid);
    }
    value
        .pointer("/externalIds/PubMed")
        .and_then(pmid)
        .or_else(|| value.get("pmid").and_then(pmid))
        .or_else(|| value.pointer("/header/pmid").and_then(pmid))
        .or_else(|| {
            value
                .pointer("/header/external_ids")
                .and_then(Value::as_array)
                .and_then(|ids| {
                    ids.iter().find_map(|id| {
                        let namespace = id["namespace"].as_str()?;
                        if !namespace.eq_ignore_ascii_case("pubmed") {
                            return None;
                        }
                        super::policy::parse_pmid(id["id"].as_str()?.rsplit(':').next()?)
                    })
                })
        })
}
fn collect(value: &Value, ids: &mut Vec<Pmid>, saved: bool) {
    if let Some(id) = id(value, saved) {
        ids.push(id);
    }
    match value {
        Value::Object(object) => {
            for child in object.values() {
                collect(child, ids, saved);
            }
        }
        Value::Array(rows) => {
            for child in rows {
                collect(child, ids, saved);
            }
        }
        _ => {}
    }
}
fn apply(
    value: &mut Value,
    inherited: Option<Pmid>,
    licenses: &Licenses,
    saved: bool,
) -> Result<(), BadReply> {
    // A nested paper owns its text even when it has no resolvable PMID.
    let own = if value.get("paperId").is_some() || value.get("externalIds").is_some() {
        id(value, saved)
    } else {
        id(value, saved).or(inherited)
    };
    let citing = value
        .get("citingPaper")
        .map_or(own, |paper| id(paper, saved));
    let license = own.and_then(|id| licenses.get(id));
    match value {
        Value::Object(object) => {
            super::abstracts::filter(object, own, license)?;
            let context_license = citing.and_then(|id| licenses.get(id));
            object.remove("biomcpContextLicense");
            if object.contains_key("contexts")
                && let Some(value) = context_license
                    .map(serde_json::to_value)
                    .transpose()
                    .map_err(|_| BadReply)?
            {
                object.insert("biomcpContextLicense".into(), value);
            }
            if !permitted(context_license, false)
                && object.get("contexts").is_some_and(has_contexts)
            {
                let notice = citing
                    .map(|id| {
                        if licenses.unchecked(id) {
                            super::unchecked(id)
                        } else {
                            super::withheld(id)
                        }
                    })
                    .unwrap_or(biomcp_types::WithheldArticleText {
                        reason: biomcp_types::Reason::Disabled,
                        url: None,
                    });
                object.insert(
                    "biomcpTextWithheld".into(),
                    serde_json::to_value(notice).map_err(|_| BadReply)?,
                );
                object.remove("contexts");
            }
            for child in object.values_mut() {
                apply(child, own, licenses, saved)?;
            }
            super::abstracts::lift(object);
        }
        Value::Array(rows) => {
            for child in rows {
                apply(child, own, licenses, saved)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn has_text(value: &Value) -> bool {
    match value {
        Value::Object(object) => {
            ["abstract", "abstract_text"]
                .iter()
                .any(|key| object.get(*key).is_some_and(|value| !value.is_null()))
                || object.get("contexts").is_some_and(has_contexts)
                || object.values().any(has_text)
        }
        Value::Array(rows) => rows.iter().any(has_text),
        _ => false,
    }
}

fn has_contexts(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Array(items) => !items.is_empty(),
        Value::Object(object) => !object.is_empty(),
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}
