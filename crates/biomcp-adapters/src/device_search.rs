//! Shared native openFDA device search filters and bounded page admission.
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError, SourceSpec};
use biomcp_types::{DiagnosticSearch, Problem};
use serde::de::DeserializeOwned;
/// Source-owned diagnostic submission candidates.
#[derive(Debug)]
pub struct Page {
    /// Original source records, without grouping applications.
    pub items: Box<[biomcp_types::DiagnosticCandidate]>,
    /// Native submission count.
    pub total: u64,
    /// Rejected optional source fields.
    pub problems: Vec<Problem>,
}
fn literal(value: &biodata::Text) -> String {
    format!(
        "\"{}\"",
        value.as_str().replace('\\', "\\\\").replace('"', "\\\"")
    )
}
fn expression(q: &DiagnosticSearch, name: &str) -> String {
    let mut terms = Vec::new();
    if let Some(value) = q.name() {
        terms.push(format!("{name}:{}", literal(value)));
    }
    if let Some(value) = q.manufacturer() {
        terms.push(format!("applicant:{}", literal(value)));
    }
    if let Some(value) = q.product_code() {
        terms.push(format!("product_code:{}", literal(value)));
    }
    if q.decision_from().is_some() || q.decision_to().is_some() {
        let from = q
            .decision_from()
            .map_or_else(|| "*".into(), |d| d.to_string());
        let to = q
            .decision_to()
            .map_or_else(|| "*".into(), |d| d.to_string());
        terms.push(format!("decision_date:[{from} TO {to}]"));
    }
    terms.join(" AND ")
}
pub(crate) fn page<T: DeserializeOwned + crate::openfda::Item>(
    leg: &Leg<'_>,
    spec: &'static SourceSpec,
    q: &DiagnosticSearch,
    name: &str,
) -> Result<(Vec<T>, u64), RequestError> {
    let selector = expression(q, name);
    let mut args = vec![
        ("limit", Arg::Num(q.limit().into())),
        ("skip", Arg::Num(q.offset().into())),
        ("sort", Arg::Text("decision_date:desc")),
    ];
    if !selector.is_empty() {
        args.push(("search", Arg::Text(&selector)));
    }
    let reply = crate::openfda::page(
        spec.source,
        leg,
        &spec
            .get("")
            .fresh(Freshness::Days(1))
            .bytes(2 * 1024 * 1024)
            .json_absence(),
        &args,
        q.offset(),
        q.limit(),
        &|reply| {
            if let crate::openfda::Envelope::Page { meta, .. } = reply
                && (meta.results.skip != q.offset() || meta.results.limit != q.limit())
            {
                return Err(BadReply);
            }
            Ok(())
        },
    )?
    .value;
    Ok(match reply {
        crate::openfda::Envelope::Page { meta, results } => (results, meta.results.total),
        crate::openfda::Envelope::Absent { .. } => (vec![], 0),
    })
}
