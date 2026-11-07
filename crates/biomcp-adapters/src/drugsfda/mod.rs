//! Canonical DRUGSFDA source declaration.
use biomcp_fetch::{SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// Publisher policy shared by health, request admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Drugsfda,
    base: "https://api.fda.gov/drug/drugsfda.json",
    terms: Terms {
        name: crate::drug_sources::FDA_TERMS.name,
        url: crate::drug_sources::FDA_TERMS.url,
        usage: crate::drug_sources::FDA_TERMS.usage,
    },
    quota: &crate::quotas::OPENFDA,
    key: crate::openfda_key::OPTIONAL,
    deadline: Duration::from_secs(15),
};
mod admit;
mod map;
mod raw;
mod wire;
use biodata::{DrugId, DrugIdentityLevel, Text};
use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, Leg, ReplyMetadata, RequestError};
use biomcp_types::{DrugHeader, DrugSource};
pub use map::Application;
use raw::Application as RawApplication;
pub(crate) use raw::date;
const APPLICATION: Endpoint = SPEC.get("").fresh(Freshness::Days(1)).json_absence();
/// One native application candidate page and its original retrieval metadata.
#[derive(Debug)]
pub struct Page {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Rejected optional source metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Native applications, without selecting a first product.
    pub records: Vec<Application>,
    /// Native application total, independent of product count.
    pub total: u64,
    /// Actual transport retrieval metadata.
    pub metadata: ReplyMetadata,
    /// Native dataset last_updated date; distinct from record events and retrieval.
    pub updated: Option<biodata::PartialDate>,
}
fn literal(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
/// Discover exact brand or ingredient names, then try their native word fields after a zero total.
pub fn search(leg: &Leg<'_>, text: &Text, limit: u32, offset: u32) -> Result<Page, RequestError> {
    let selector = |suffix: &str| {
        let name = literal(text.as_str());
        format!(
            "products.brand_name{suffix}:{name} OR products.active_ingredients.name{suffix}:{name}"
        )
    };
    let exact = page(leg, &selector(".exact"), limit, offset, None)?;
    if exact.total > 0 {
        return Ok(exact);
    }
    page(leg, &selector(""), limit, offset, None)
}
/// Name resolution uses the existing word selector to retain all exact and approved single-term generic forms.
pub fn search_names(
    leg: &Leg<'_>,
    text: &Text,
    limit: u32,
    offset: u32,
) -> Result<Page, RequestError> {
    let name = literal(text.as_str());
    page(
        leg,
        &format!("products.brand_name:{name} OR products.active_ingredients.name:{name}"),
        limit,
        offset,
        None,
    )
}
/// Read exactly the named native application; reject a wrong identity before caching it.
pub fn get(leg: &Leg<'_>, id: &DrugId) -> Result<Page, RequestError> {
    if !DrugSource::Drugsfda.accepts(id) {
        return Err(biomcp_fetch::bad_reply(Source::Drugsfda, Some(200)));
    }
    page(
        leg,
        &format!("application_number:{}", literal(&id.to_string())),
        1,
        0,
        Some(id),
    )
}
fn page(
    leg: &Leg<'_>,
    selector: &str,
    limit: u32,
    offset: u32,
    id: Option<&DrugId>,
) -> Result<Page, RequestError> {
    if !(1..=50).contains(&limit) {
        return Err(biomcp_fetch::bad_reply(Source::Drugsfda, Some(200)));
    }
    let mut args = vec![
        ("search", Arg::Text(selector)),
        ("limit", Arg::Num(limit.into())),
    ];
    if offset > 0 {
        args.push(("skip", Arg::Num(offset.into())));
    }
    let (response, metadata) = crate::openfda::page(
        Source::Drugsfda,
        leg,
        &APPLICATION,
        &args,
        offset,
        limit,
        &|r: &raw::Response| match r {
            raw::Response::Page { meta, results }
                if meta.results.skip != offset
                    || meta.results.limit != limit
                    || id.is_some_and(|id| {
                        meta.results.total != 1
                            || results.iter().any(|r| r.application_number != *id)
                    }) =>
            {
                Err(BadReply)
            }
            _ => Ok(()),
        },
    )
    .map(|received| (received.value, received.meta))?;
    let mut problems = crate::mapped::Problems {
        source: biomcp_types::Source::Drugsfda,
        record: None,
        list: Vec::new(),
    };
    let (records, total, updated) = match response {
        raw::Response::Page { meta, results } => (results, meta.results.total, meta.last_updated),
        raw::Response::Absent { .. } => (Vec::new(), 0, None),
    };
    problems
        .list
        .extend(records.iter().flat_map(|r| r.problems.iter().cloned()));
    let mut refusals = crate::Refusal::None;
    for record in &records {
        refusals.include(record.refusals);
    }
    Ok(Page {
        refusals,
        problems: problems.list,
        records: records.into_iter().map(map::Application).collect(),
        total,
        metadata,
        updated,
    })
}
impl RawApplication {
    /// An application endpoint establishes application identity, without choosing a brand.
    pub(super) fn header(&self) -> DrugHeader {
        DrugHeader {
            name_resolution: None,
            matched_name: None,
            id: self.application_number.clone(),
            source: DrugSource::Drugsfda,
            name: None,
            identity_level: DrugIdentityLevel::FdaApplication,
            r#type: None,
            version: None,
            url: None,
        }
    }
}

/// Complete source policy and command declaration.
pub(crate) mod decl;

/// FDA document downloads retain their API source identity and never send its key.
pub static DOCUMENTS: biomcp_fetch::SourceSpec = biomcp_fetch::SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: biomcp_types::Source::Drugsfda,
    base: "https://www.accessdata.fda.gov",
    terms: SPEC.terms,
    key: biomcp_fetch::KeyUse::None,
    quota: &crate::quotas::OPENFDA,
    deadline: std::time::Duration::from_secs(15),
};
