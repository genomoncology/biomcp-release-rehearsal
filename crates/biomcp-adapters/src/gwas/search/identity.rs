//! Pure reply admission before source cache publication.
use crate::gwas::{SPEC, scope::Selectors};
use biomcp_types::GwasSearch;
use std::collections::BTreeMap;
pub(super) fn self_link(url: &str, expected: &BTreeMap<String, String>) -> bool {
    resource_link(url, "associations", expected)
}
pub(in crate::gwas) fn resource_link(
    url: &str,
    resource: &str,
    expected: &BTreeMap<String, String>,
) -> bool {
    let prefix = format!("{}/{resource}?", SPEC.base);
    let Some(query) = url.strip_prefix(&prefix) else {
        return false;
    };
    let mut actual = BTreeMap::new();
    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            return false;
        };
        let (Some(key), Some(value)) = (decode(key), decode(value)) else {
            return false;
        };
        if actual.insert(key, value).is_some() {
            return false;
        }
    }
    actual == *expected
}
fn decode(text: &str) -> Option<String> {
    let mut bytes = text.bytes();
    let mut decoded = Vec::new();
    while let Some(byte) = bytes.next() {
        decoded.push(match byte {
            b'%' => {
                let a = char::from(bytes.next()?).to_digit(16)?;
                let b = char::from(bytes.next()?).to_digit(16)?;
                (a * 16 + b) as u8
            }
            b'+' => b' ',
            b'#' => return None,
            b => b,
        });
    }
    String::from_utf8(decoded).ok()
}
pub(super) fn row(row: &Selectors, q: &GwasSearch) -> bool {
    if q.study().is_some_and(|id| {
        row.accession_id
            .as_ref()
            .and_then(crate::raw::Field::as_ref)
            .is_none_or(|v| v.as_str() != id.to_string())
    }) || q.pubmed().is_some_and(|id| {
        row.pubmed_id
            .as_ref()
            .and_then(crate::raw::Field::as_ref)
            .map(String::as_str)
            != Some(id.to_string().as_str())
    }) || q
        .variant()
        .is_some_and(|id| !row.alleles().any(|a| a.rs_id.as_str() == id.to_string()))
        || q.trait_id().is_some_and(|id| {
            !q.include_child_traits() && !row.traits().any(|t| t.efo_id.as_str() == id.to_string())
        })
        || q.trait_label()
            .is_some_and(|label| !row.traits().any(|t| t.efo_trait.as_ref() == Some(label)))
    {
        return false;
    }
    true
}
