//! Native FDA drug enforcement recall records.
use biomcp_fetch::{Input, Read};
mod map;
mod raw;
use biodata::{DrugRecallNumber, Text};
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError, SourceSpec, Terms};
use biomcp_types::{DrugRecallCard, DrugRecallHeader, DrugSearch, Problem, Source, Use};
use std::time::Duration;
/// Shared openFDA admission policy for recall and its exact enforcement lookup.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::FdaDrugRecall,
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
    pub items: Box<[DrugRecallHeader]>,
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
fn expression(q: &DrugSearch) -> String {
    let mut terms = Vec::new();
    if let Some(v) = q.query() {
        let name = literal(v);
        terms.push(format!("(product_description:{name} OR openfda.brand_name:{name} OR openfda.generic_name:{name})"));
    }
    for (field, value) in [
        ("recalling_firm", q.firm()),
        ("status", q.status()),
        ("reason_for_recall", q.reason()),
    ] {
        if let Some(value) = value {
            terms.push(format!("{field}:{}", literal(value)));
        }
    }
    if let Some(value) = q.classification() {
        terms.push(format!("classification:\"{value}\""));
    }
    if q.initiated_from().is_some() || q.initiated_to().is_some() {
        let compact = |d: biodata::PartialDate| d.to_string().replace('-', "");
        let from = q.initiated_from().map_or_else(|| "*".into(), compact);
        let to = q.initiated_to().map_or_else(|| "*".into(), compact);
        terms.push(format!("recall_initiation_date:[{from} TO {to}]"));
    }
    terms.join(" AND ")
}
/// Search one bounded source-ordered native page.
pub fn search(leg: &Leg<'_>, q: &DrugSearch) -> Result<Page, RequestError> {
    let selector = expression(q);
    let mut args = Vec::new();
    if !selector.is_empty() {
        args.push(("search", Arg::Text(&selector)));
    }
    args.extend([
        ("limit", Arg::Num(q.limit().into())),
        ("skip", Arg::Num(q.offset().into())),
        ("sort", Arg::Text("recall_initiation_date:desc")),
    ]);
    let (reply, _) = crate::openfda::page(
        Source::FdaDrugRecall,
        leg,
        &SPEC
            .get("/drug/enforcement.json")
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
                row.value.header
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
fn exact(
    leg: &Leg<'_>,
    path: &'static str,
    field: &'static str,
    id: &DrugRecallNumber,
) -> Result<Option<raw::Record>, Failure> {
    let selector = format!("{field}.exact:\"{id}\"");
    let endpoint = SPEC
        .get(path)
        .fresh(Freshness::Days(1))
        .bytes(2 * 1024 * 1024)
        .json_absence();
    let identity = |reply: &raw::Response<raw::Record>| !matches!(reply,raw::Response::Page {results,..} if results.iter().any(|r|&r.recall_number!=id));
    let reply = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("search", Arg::Text(&selector)),
                ("limit", Arg::Num(1)),
                ("skip", Arg::Num(0)),
            ]),
            Read::<raw::Response<raw::Record>>::json()
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
                source: Source::FdaDrugRecall,
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
/// Fetch one native recall number without accepting another record.
pub fn get(
    leg: &Leg<'_>,
    id: &DrugRecallNumber,
) -> Result<Option<crate::Mapped<DrugRecallCard>>, Failure> {
    exact(leg, "/drug/enforcement.json", "recall_number", id).map(|row| row.map(map::mapped))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
