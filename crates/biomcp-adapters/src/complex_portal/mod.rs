//! Exact native Complex Portal participants; no inferred membership.
use biomcp_fetch::Input;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::{Source, Use};
use std::time::Duration;
/// One publisher declaration owns admission, terms and keyless request policy.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::ComplexPortal,
    base: "https://www.ebi.ac.uk/intact/complex-ws",
    terms: Terms {
        name: "CC0",
        url: "https://pmc.ncbi.nlm.nih.gov/articles/PMC11701666/",
        usage: Use::Open,
    },
    key: KeyUse::None,
    quota: &crate::quotas::COMPLEX_PORTAL,
    deadline: Duration::from_secs(8),
};
mod map;
mod raw;
use biodata::{ComplexPortalId, ProteinComplexHeader, ProteinComplexMember, UniProtIsoform};
use biomcp_fetch::{Arg, Freshness, Leg, RequestError};
use serde::{Deserialize, Serialize};
/// An admitted complex and its whole native participant list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Complex {
    /// Actual refused native records, independent of optional fields.
    #[serde(
        default = "crate::mapped::refusal_serde::none",
        with = "crate::mapped::refusal_serde"
    )]
    pub refusals: crate::Refusal,

    /// Exact native header.
    pub header: ProteinComplexHeader,
    /// Source participants, including non-proteins.
    pub members: Vec<ProteinComplexMember>,
    /// Rejected optional source fields retained with the complex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problems: Option<Vec<biomcp_types::Problem>>,
}
/// A complete admitted collection with an independent source search count.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Collection {
    /// Actual refused native records, independent of optional fields.
    #[serde(
        default = "crate::mapped::refusal_serde::none",
        with = "crate::mapped::refusal_serde"
    )]
    pub refusals: crate::Refusal,

    /// Exact-participant admissions in native order.
    pub items: Vec<Complex>,
    /// Native text-search total, never an admitted membership count.
    pub native_total: u64,
}
const HUMAN: &str = "species_f:(\"Homo sapiens\")";
/// Scan the exact accession query without admitting description matches.
pub fn search(leg: &Leg<'_>, anchor: &UniProtIsoform) -> Result<Collection, RequestError> {
    let endpoint = SPEC
        .get("")
        .with_path(format!("/search/{anchor}"))?
        .identity_encoding()
        .fresh(Freshness::Days(1));
    let mut offset = 0;
    let mut items = Vec::new();
    let mut total = None;
    loop {
        let mut args = vec![("number", Arg::Num(25)), ("filters", Arg::Text(HUMAN))];
        if offset > 0 {
            args.push(("first", Arg::Num(offset)));
        }
        let page = leg
            .call(
                &endpoint,
                Input::Params(&args),
                biomcp_fetch::Read::<raw::Search>::json(),
            )?
            .value;
        if total.is_some_and(|n| n != page.total_number_of_results) {
            return Err(biomcp_fetch::bad_reply(Source::ComplexPortal, Some(200)));
        }
        total = Some(page.total_number_of_results);
        let returned = page.elements.len() as u64;
        if offset + returned > page.total_number_of_results
            || returned == 0 && offset < page.total_number_of_results
        {
            return Err(biomcp_fetch::bad_reply(Source::ComplexPortal, Some(200)));
        }
        for row in page.elements {
            if row.interactors.is_none() {
                let detail = detail(leg, &row.id)?;
                if let Some(complex) = map::complex(detail, anchor)? {
                    items.push(complex);
                }
            } else if let Some(complex) = map::complex(row, anchor)? {
                items.push(complex);
            }
        }
        offset += returned;
        if offset == page.total_number_of_results {
            break;
        }
    }
    let mut ids = std::collections::BTreeSet::new();
    if items.iter().any(|c| !ids.insert(c.header.id.to_string())) {
        return Err(biomcp_fetch::bad_reply(Source::ComplexPortal, Some(200)));
    }
    let mut refusals = crate::Refusal::None;
    for complex in &items {
        refusals.include(complex.refusals);
    }
    Ok(Collection {
        refusals,
        items,
        native_total: total
            .ok_or_else(|| biomcp_fetch::bad_reply(Source::ComplexPortal, Some(200)))?,
    })
}
fn detail(leg: &Leg<'_>, id: &ComplexPortalId) -> Result<raw::Complex, RequestError> {
    let endpoint = SPEC
        .get("")
        .with_path(format!("/complex/{id}"))?
        .identity_encoding()
        .fresh(Freshness::Days(1));
    let d = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Detail>::json().cache_if(&|d| d.id == *id),
        )?
        .value;
    if d.id != *id {
        return Err(biomcp_fetch::bad_reply(Source::ComplexPortal, Some(200)));
    }
    Ok(raw::Complex {
        id: d.id,
        name: d.name,
        organism_name: d.species,
        predicted_complex: d.predicted_complex,
        interactors: Some(d.participants),
    })
}
/// Read one exact complex and require an actual protein participant as anchor.
pub fn members(
    leg: &Leg<'_>,
    id: &ComplexPortalId,
    anchor: &UniProtIsoform,
) -> Result<Option<Complex>, RequestError> {
    map::complex(detail(leg, id)?, anchor)
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
