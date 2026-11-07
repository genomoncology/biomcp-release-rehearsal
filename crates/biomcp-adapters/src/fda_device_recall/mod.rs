//! Native product recall records with exact optional enforcement classification.
use biomcp_fetch::{Input, Read};
mod map;
mod raw;
use biodata::{DeviceRecallNumber, Text};
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError, SourceSpec, Terms};
use biomcp_types::{DeviceRecallCard, DiagnosticCandidate, DiagnosticSearch, Problem, Source, Use};
use raw::Identity;
use serde::de::DeserializeOwned;
use std::time::Duration;
/// Shared openFDA admission policy for recall and its exact enforcement lookup.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::FdaDeviceRecall,
    base: "https://api.fda.gov",
    terms: Terms {
        name: "openFDA terms and license; third-party exceptions",
        url: "https://open.fda.gov/terms/",
        usage: Use::Open,
    },
    quota: &crate::quotas::OPENFDA,
    key: crate::openfda_key::OPTIONAL,
    deadline: Duration::from_secs(15),
};
/// One bounded native recall-record page.
#[derive(Debug)]
pub struct Page {
    /// Native headers, without event deduplication.
    pub items: Box<[DiagnosticCandidate]>,
    /// Native count of recall records.
    pub total: u64,
    /// Rejected optional values with record provenance.
    pub problems: Vec<Problem>,
}
fn literal(value: &Text) -> String {
    format!(
        "\"{}\"",
        value.as_str().replace('\\', "\\\\").replace('"', "\\\"")
    )
}
fn expression(q: &DiagnosticSearch) -> String {
    let mut terms = Vec::new();
    if let Some(v) = q.manufacturer() {
        terms.push(format!("recalling_firm:{}", literal(v)));
    }
    if let Some(v) = q.product_code() {
        terms.push(format!("product_code:{}", literal(v)));
    }
    if q.initiated_from().is_some() || q.initiated_to().is_some() {
        let from = q
            .initiated_from()
            .map_or_else(|| "*".into(), |d| d.to_string());
        let to = q
            .initiated_to()
            .map_or_else(|| "*".into(), |d| d.to_string());
        terms.push(format!("event_date_initiated:[{from} TO {to}]"));
    }
    terms.join(" AND ")
}
/// Search one bounded source-ordered native page.
pub fn search(leg: &Leg<'_>, q: &DiagnosticSearch) -> Result<Page, RequestError> {
    let selector = expression(q);
    let mut args = Vec::new();
    if !selector.is_empty() {
        args.push(("search", Arg::Text(&selector)));
    }
    args.extend([
        ("limit", Arg::Num(q.limit().into())),
        ("skip", Arg::Num(q.offset().into())),
        ("sort", Arg::Text("event_date_initiated:desc")),
    ]);
    let (reply, _) = crate::openfda::page(
        Source::FdaDeviceRecall,
        leg,
        &SPEC
            .get("/device/recall.json")
            .fresh(Freshness::Days(1))
            .bytes(2 * 1024 * 1024)
            .json_absence(),
        &args,
        q.offset(),
        q.limit(),
        &|reply| counts(reply, q.offset(), q.limit(), false),
    )
    .map(|received| (received.value, received.meta))?;
    let mut page = Page {
        items: Box::default(),
        total: 0,
        problems: Vec::new(),
    };
    if let raw::Response::Page { meta, results } = reply {
        page.total = meta.results.total;
        page.items = results
            .into_iter()
            .map(|row| {
                let row = map::mapped(row);
                page.problems.extend(row.problems);
                DiagnosticCandidate::DeviceRecall {
                    header: row.value.header,
                }
            })
            .collect();
    }
    Ok(page)
}
fn counts<T>(reply: &raw::Response<T>, skip: u32, limit: u32, exact: bool) -> Result<(), BadReply> {
    if let raw::Response::Page { meta, .. } = reply
        && (meta.results.skip != skip
            || meta.results.limit != limit
            || exact && meta.results.total != 1)
    {
        return Err(BadReply);
    }
    Ok(())
}
/// An exact lookup failure retains its conflicting identity field.
#[derive(Debug)]
pub struct Failure {
    /// Source or local request failure.
    pub error: RequestError,
    /// Wrong returned recall identity; never admitted to the cache.
    pub identity_field: Option<&'static str>,
}
fn exact<T: DeserializeOwned + Identity + crate::openfda::Item>(
    leg: &Leg<'_>,
    path: &'static str,
    field: &'static str,
    id: &DeviceRecallNumber,
) -> Result<Option<T>, Failure> {
    let selector = format!("{field}.exact:\"{id}\"");
    let endpoint = SPEC
        .get(path)
        .fresh(Freshness::Days(1))
        .bytes(2 * 1024 * 1024)
        .json_absence();
    let identity = |reply: &raw::Response<T>| !matches!(reply,raw::Response::Page {results,..} if results.iter().any(|r|r.id()!=id));
    let reply = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("search", Arg::Text(&selector)),
                ("limit", Arg::Num(1)),
                ("skip", Arg::Num(0)),
            ]),
            Read::<raw::Response<T>>::json()
                .check(&|reply| counts(reply, 0, 1, true))
                .cache_if(&identity),
        )
        .map(|received| received.value)
        .map_err(|error| Failure {
            error,
            identity_field: None,
        })?;
    if !identity(&reply) {
        return Err(Failure {
            error: biomcp_fetch::SourceError {
                source: Source::FdaDeviceRecall,
                reason: biomcp_types::Reason::BadReply,
                status: Some(200),
                tls_cause: None,
                retry_after: None,
            }
            .into(),
            identity_field: Some(field),
        });
    }
    Ok(match reply {
        raw::Response::Page { results, .. } => results.into_iter().next(),
        raw::Response::Absent { .. } => None,
    })
}
/// Get only the selected native product recall record.
pub fn get(
    leg: &Leg<'_>,
    id: &DeviceRecallNumber,
) -> Result<Option<crate::mapped::Mapped<DeviceRecallCard>>, Failure> {
    exact::<raw::Record>(leg, "/device/recall.json", "product_res_number", id)
        .map(|row| row.map(map::mapped))
}
/// One exact enforcement lookup adds only its native classification.
pub fn classification(
    leg: &Leg<'_>,
    id: &DeviceRecallNumber,
) -> Result<crate::mapped::Mapped<Option<Text>>, Failure> {
    exact::<raw::Enforcement>(leg, "/device/enforcement.json", "recall_number", id).map(|row| {
        row.map_or_else(
            || crate::mapped::Mapped {
                refusals: crate::Refusal::None,
                value: None,
                problems: Vec::new(),
            },
            map::classification,
        )
    })
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
