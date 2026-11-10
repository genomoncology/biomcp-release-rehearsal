//! Native registry discovery; no local clinical interpretation or reordering.
use biomcp_fetch::Input;
pub mod detail;
pub mod document;
mod filters;
mod gene;
mod raw;
mod refusals;
mod version;
use biodata::Text;
use biomcp_fetch::{Arg, Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::TrialHeader;
use biomcp_types::TrialSearch;
pub use gene::{GeneContext, GeneMention};
use raw::{FirstPage, RawPage};
use std::{collections::BTreeMap, time::Duration};
pub use version::version;
/// Reviewed document download origin.
pub(crate) const DOCUMENT_ORIGIN: &str = "https://cdn.clinicaltrials.gov";
/// Publisher terms apply to this registry; individual records are not a blanket CC0 grant.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Additional(&[DOCUMENT_ORIGIN]),
    user_agent: None,
    source: biomcp_types::Source::ClinicalTrialsGov,
    base: "https://clinicaltrials.gov/api/v2",
    key: KeyUse::None,
    quota: &crate::quotas::CTGOV,
    deadline: Duration::from_secs(15),
    terms: biomcp_fetch::Terms {
        name: "ClinicalTrials.gov attribution and source processing date; publisher terms",
        url: "https://clinicaltrials.gov/about-site/terms-conditions",
        usage: biomcp_types::Use::Attribution,
    },
};
const SEARCH: Endpoint = SPEC.get("/studies").fresh(Freshness::Days(1));
/// Small source processing version endpoint, also the bounded health probe.
pub const VERSION: Endpoint = SPEC.get("/version").fresh(Freshness::Days(1));
/// A native search page with opaque source cursor.
#[derive(Debug)]
pub struct SearchPage {
    /// Actual refused studies or native child records, independent of field problems.
    pub refusals: crate::Refusal,
    /// Native studies consumed before refused identities are filtered.
    pub received: u32,
    /// Source order, untouched.
    pub items: Vec<TrialHeader>,
    /// Rejected nonidentity fields from retained studies.
    pub problems: Vec<biomcp_types::Problem>,
    /// At least one complete native condition list lacks the searched words.
    pub condition_word_mismatch: bool,
    /// Literal gene context in retained source records, without an eligibility assessment.
    pub gene_contexts: Vec<GeneContext>,
    /// Exact native total.
    pub total: Option<u64>,
    /// Native count lookup failure while page rows remain usable.
    pub count_error: Option<RequestError>,
    /// Exact fallback count parameters, only when the source omitted continuation totals.
    pub count_request: Option<BTreeMap<String, String>>,
    /// Opaque next page.
    pub next: Option<Text>,
}
/// Exact parameters; resolved gene text is explicit and attributed to HGNC by the caller.
pub fn parameters(query: &TrialSearch, gene: Option<&str>) -> BTreeMap<String, String> {
    name_parameters(query, gene, query.broader_names())
}
/// Provider-native name search, preserving every other registry filter.
pub fn broader_parameters(query: &TrialSearch, gene: Option<&str>) -> BTreeMap<String, String> {
    name_parameters(query, gene, true)
}
fn name_parameters(
    query: &TrialSearch,
    gene: Option<&str>,
    broader_names: bool,
) -> BTreeMap<String, String> {
    let mut p = filters::base_parameters(query);
    for (name, values) in [
        ("query.cond", query.condition()),
        ("query.intr", query.intervention()),
    ] {
        if !values.is_empty() {
            p.insert(name.into(), alternatives(values));
        }
    }
    filters::name_parameters(query, broader_names, &mut p);
    let mut text = Vec::new();
    if let Some(value) = query.query() {
        text.push(value.as_str().to_owned());
    }
    if let Some(value) = query.variant() {
        text.push(value.as_str().to_owned());
    }
    if let Some(value) = gene {
        text.push(value.to_owned());
    }
    if !query.keyword().is_empty() {
        let keywords = if text.is_empty() {
            alternatives(query.keyword())
        } else {
            for value in &mut text {
                *value = format!("({value})");
            }
            format!("({})", alternatives(query.keyword()))
        };
        text.push(keywords);
    }
    if !text.is_empty() {
        p.insert("query.term".into(), text.join(" AND "));
    }
    if let Some(status) = query.status() {
        p.insert("filter.overallStatus".into(), status.as_str().to_owned());
    }
    let advanced = filters::advanced(query, broader_names);
    if !advanced.is_empty() {
        p.insert("filter.advanced".into(), advanced);
    }
    let mut facets = Vec::new();
    if let Some(sex) = query.sex() {
        facets.push(format!(
            "sex:{}",
            match sex {
                biodata::TrialSex::Female => "f",
                biodata::TrialSex::Male => "m",
                biodata::TrialSex::All => "a",
            }
        ));
    }
    if let Some(sponsor) = query.sponsor_type() {
        facets.push(format!("funderType:{}", sponsor.as_str()));
    }
    if !facets.is_empty() {
        p.insert("aggFilters".into(), facets.join(","));
    }
    if let (Some(lat), Some(lon), Some(distance)) = (query.lat(), query.lon(), query.distance()) {
        p.insert(
            "filter.geo".into(),
            format!(
                "distance({},{},{})",
                lat.as_str(),
                lon.as_str(),
                distance.as_str()
            ),
        );
    }
    p
}
/// Read and validate native discovery results before shared cache admission.
pub fn search(
    leg: &Leg<'_>,
    params: &BTreeMap<String, String>,
    conditions: &[Text],
    countries: &[Box<str>],
) -> Result<SearchPage, RequestError> {
    search_with_gene(leg, params, conditions, countries, None)
}

pub(super) fn search_with_gene(
    leg: &Leg<'_>,
    params: &BTreeMap<String, String>,
    conditions: &[Text],
    countries: &[Box<str>],
    gene: Option<&str>,
) -> Result<SearchPage, RequestError> {
    let raw = fetch_page(leg, params)?;
    if !countries.is_empty()
        && raw
            .studies
            .as_deref()
            .unwrap_or_default()
            .iter()
            .filter_map(crate::raw::Field::as_ref)
            .any(|row| !row.protocol_section.has_country(countries))
    {
        return Err(biomcp_fetch::bad_reply(
            biomcp_types::Source::ClinicalTrialsGov,
            Some(200),
        ));
    }
    let mut total = raw.total_count;
    let mut count_error = None;
    let mut count_request = None;
    if total.is_none() {
        let mut count = params.clone();
        count.remove("pageToken");
        count.insert("pageSize".into(), "1".into());
        match fetch_page(leg, &count) {
            Ok(page) => total = page.total_count,
            Err(error) => count_error = Some(error),
        }
        count_request = Some(count);
    }
    let (condition_word_mismatch, gene_contexts) = raw.search_contexts(conditions, gene);
    let received = raw.studies.as_ref().map_or(0, |rows| rows.len() as u32);
    let mut rejected = Vec::new();
    let mut refusals = crate::Refusal::None;
    let mapped = raw
        .studies
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| match row.value() {
            Some(row) => Some(row.protocol_section.header()),
            None => {
                refusals = crate::Refusal::Refused;
                rejected.push(crate::mapped::problem(SPEC.source, None, "nctId"));
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| biomcp_fetch::bad_reply(biomcp_types::Source::ClinicalTrialsGov, Some(200)))?;
    let (items, problems): (Vec<_>, Vec<_>) = mapped
        .into_iter()
        .map(|row| {
            refusals.include(row.refusals);
            (row.value, row.problems)
        })
        .unzip();
    rejected.extend(problems.into_iter().flatten());
    Ok(SearchPage {
        received,
        refusals,
        problems: rejected,
        condition_word_mismatch,
        gene_contexts,
        items,
        total,
        count_error,
        count_request,
        next: raw.next_page_token,
    })
}
fn fetch_page(leg: &Leg<'_>, params: &BTreeMap<String, String>) -> Result<RawPage, RequestError> {
    let args: Vec<_> = params
        .iter()
        .map(|(k, v)| (k.as_str(), Arg::Text(v)))
        .collect();
    if params.contains_key("pageToken") {
        leg.call(
            &SEARCH,
            Input::Params(&args),
            biomcp_fetch::Read::<RawPage>::json(),
        )
        .map(|received| received.value)
    } else {
        leg.call(
            &SEARCH,
            Input::Params(&args),
            biomcp_fetch::Read::<FirstPage>::json(),
        )
        .map(|received| received.value)
        .map(|page| page.0)
    }
}

fn alternatives(values: &[Text]) -> String {
    values
        .iter()
        .map(|value| format!("({})", value.as_str()))
        .collect::<Vec<_>>()
        .join(" OR ")
}

mod search_query;
pub use search_query::{name_fits, search_query};

/// Complete source policy and command declaration.
pub(crate) mod decl;
