//! Canonical diagnostic publisher declaration; runtime dispatch belongs to later slices.
use biomcp_fetch::{Input, Read};
use biomcp_fetch::{SourceSpec, Terms};
use biomcp_types::{Source, Use};
use std::time::Duration;
/// One publisher declaration for health, outcomes and admission before any I/O.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::FdaPma,
    base: "https://api.fda.gov/device/pma.json",
    terms: Terms {
        name: "openFDA terms and license; third-party exceptions",
        url: "https://open.fda.gov/terms/",
        usage: Use::Open,
    },
    quota: &crate::quotas::OPENFDA,
    key: crate::openfda_key::OPTIONAL,
    deadline: Duration::from_secs(15),
};
mod map;
mod raw;
pub use crate::text_parts::escaped_bytes;
use biodata::DeviceSubmission;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError};
use biomcp_types::DiagnosticGet;
pub use map::{fragments, header, regulatory};
/// Complete source application members or one exact selected supplement.
#[derive(Debug)]
pub struct Records {
    /// Native records in original order, without harmonized values.
    pub rows: Vec<DeviceSubmission>,
    /// Rejected nonidentity source fields.
    pub problems: Vec<biomcp_types::Problem>,
    /// True only when the source application exceeds its single admitted page.
    pub incomplete: bool,
    /// Exact native identity field that conflicts with the request.
    pub identity_conflict: Option<&'static str>,
}
/// Validate exact source identity before any record can enter the ordinary cache.
pub fn get(leg: &Leg<'_>, query: &DiagnosticGet) -> Result<Records, RequestError> {
    let mut selector = format!("pma_number:\"{}\"", query.id());
    if let Some(supplement) = query.supplement() {
        selector.push_str(&format!(" AND supplement_number:\"{supplement}\""));
    }
    let endpoint = SPEC
        .get("")
        .fresh(Freshness::Days(1))
        .bytes(10 * 1024 * 1024)
        .json_absence();
    let reply = leg.call(&endpoint, Input::Params(&[
            ("search", Arg::Text(&selector)),
            ("limit", Arg::Num(100)),
            ("skip", Arg::Num(0)),
        ]), Read::<raw::Response>::json().check(&|reply| {
            if let raw::Response::Page { meta, results } = reply {
                if meta.results.skip != 0 || meta.results.limit != 100 {
                    return Err(BadReply);
                }
                if results.iter().any(|row| row.supplement_number.is_none()) {
                    return Err(BadReply);
                }
                let mut seen = std::collections::BTreeSet::new();
                if results
                    .iter()
                    .any(|row| !seen.insert(row.supplement_number.as_ref().map(|s| s.as_str())))
                {
                    return Err(BadReply);
                }
            }
            Ok(())
        }).cache_if(&|reply| {
            identity(reply, query).is_none()
                && !matches!(reply, raw::Response::Page {meta,..} if meta.results.total > 100)
        }))?.value;
    let identity_conflict = identity(&reply, query);
    let mut problems = Vec::new();
    Ok(match reply {
        raw::Response::Page { meta, results } => Records {
            rows: results
                .into_iter()
                .map(|row| {
                    let mapped = row.mapped();
                    problems.extend(mapped.problems);
                    mapped.value
                })
                .collect(),
            problems,
            incomplete: meta.results.total > 100,
            identity_conflict,
        },
        raw::Response::Absent { .. } => Records {
            rows: vec![],
            problems,
            incomplete: false,
            identity_conflict: None,
        },
    })
}

fn identity(reply: &raw::Response, query: &DiagnosticGet) -> Option<&'static str> {
    let raw::Response::Page { results, .. } = reply else {
        return None;
    };
    if results
        .iter()
        .any(|row| row.pma_number.as_str() != query.id().as_str())
    {
        return Some("pma_number");
    }
    if query.supplement().is_some_and(|wanted| {
        results.iter().any(|row| {
            row.supplement_number
                .as_ref()
                .is_some_and(|cell| cell.as_str() != wanted.as_str())
        })
    }) {
        return Some("supplement_number");
    }
    None
}

/// Complete source policy and command declaration.
pub(crate) mod decl;

mod search;
pub use search::search;

/// FDA document downloads retain their API source identity and never send its key.
pub static DOCUMENTS: biomcp_fetch::SourceSpec = biomcp_fetch::SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: biomcp_types::Source::FdaPma,
    base: "https://www.accessdata.fda.gov",
    terms: SPEC.terms,
    key: biomcp_fetch::KeyUse::None,
    quota: &crate::quotas::OPENFDA,
    deadline: std::time::Duration::from_secs(15),
};

pub use crate::device_search::Page as SearchPage;
