use super::{SET, SPEC};
use biomcp_fetch::{BundleInfo, Find, KeyKind, Leg, RequestError};
use biomcp_types::{DiagnosticCandidate, DiagnosticSearch};
use std::collections::HashMap;
/// Bounded WHO product headers and their distinct product total.
#[derive(Debug)]
pub struct Search {
    /// First matching native row per product, in source product order.
    pub items: Box<[DiagnosticCandidate]>,
    /// Distinct matching WHO Product IDs.
    pub total: u64,
    /// Exact stored source snapshot.
    pub info: BundleInfo,
}
/// Match only the two native cells; reads never download or synchronize.
pub fn search(leg: &Leg<'_>, q: &DiagnosticSearch) -> Result<Search, RequestError> {
    let (rows, info) = leg.stored_set_rows(&SET, Find::Prefix(KeyKind::Id, ""))?;
    if q.bundle_revision()
        .is_some_and(|v| Some(v.as_str()) != info.content_revision.as_deref())
    {
        return Err(biomcp_fetch::SourceError {
            source: SPEC.source,
            reason: biomcp_types::Reason::Rejected,
            status: None,
            tls_cause: None,
            retry_after: None,
        }
        .into());
    }
    let mut positions = HashMap::new();
    let mut products: Vec<Option<DiagnosticCandidate>> = Vec::new();
    for row in rows {
        let row = row.value;
        let position = *positions.entry(row.id.clone()).or_insert_with(|| {
            products.push(None);
            products.len() - 1
        });
        if words(&row.marker, q.marker())
            && words(&row.manufacturer, q.manufacturer())
            && let Some(header @ None) = products.get_mut(position)
        {
            *header = Some(DiagnosticCandidate::WhoIvd {
                id: row.id,
                product_name: row.product_name,
                manufacturer: row.manufacturer,
                marker: row.marker,
            });
        }
    }
    let matched: Vec<_> = products.into_iter().flatten().collect();
    let total = matched.len() as u64;
    let items = matched
        .into_iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .collect();
    Ok(Search { items, total, info })
}
fn words(cell: &str, selector: Option<&biodata::Text>) -> bool {
    selector.is_none_or(|value| {
        let cell = cell.to_lowercase();
        let needle = value.as_str().to_lowercase();
        cell.char_indices().any(|(position, _)| {
            if !cell
                .get(position..)
                .is_some_and(|tail| tail.starts_with(&needle))
            {
                return false;
            }
            let before = cell.get(..position).and_then(|v| v.chars().next_back());
            let after = cell
                .get(position + needle.len()..)
                .and_then(|v| v.chars().next());
            before.is_none_or(|c| !c.is_alphanumeric())
                && after.is_none_or(|c| !c.is_alphanumeric())
        })
    })
}
