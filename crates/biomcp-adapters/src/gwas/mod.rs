//! Native GWAS Catalog v2 association pages, without clinical interpretation.
use biomcp_fetch::Input;
pub mod get;
mod raw;
mod receive;
mod scope;
mod search;
use crate::quotas;
use biodata::VariantId;
use biomcp_fetch::{Arg, BadReply, Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::Source;
mod map;
pub use map::Association;
pub use search::search;
use std::time::Duration;
/// Current catalog API; retired v1 is never used.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "GWAS Catalog terms and dataset rights",
        url: "https://www.ebi.ac.uk/about/terms-of-use/",
        usage: biomcp_types::Use::Open,
    },
    source: Source::Gwas,
    base: "https://www.ebi.ac.uk/gwas/rest/api/v2",
    key: KeyUse::None,
    quota: &quotas::GWAS,
    deadline: Duration::from_secs(15),
};
const ASSOCIATIONS: Endpoint = SPEC.get("/associations").fresh(Freshness::Days(1));
/// Source-native page and its exact catalog denominator.
#[derive(Debug)]
pub struct Page {
    /// Actual record refusals in the selected source slots.
    pub refusals: crate::Refusal,
    /// Native page rows before identity refusals.
    pub received: u32,
    /// Rejected source fields and row identities.
    pub problems: Vec<biomcp_types::Problem>,
    /// Selected source rows.
    pub rows: Vec<Association>,
    /// Source-reported total associations.
    pub total: u64,
}
/// Read one bounded page by exact rsID, preserving current v2 request identity.
pub fn associations(
    leg: &Leg<'_>,
    rs: &VariantId,
    page: u32,
    size: u32,
    skip: u32,
) -> Result<Page, RequestError> {
    let id = rs.to_string();
    let expected = format!(
        "{}/associations?rs_id={id}&page={page}&size={size}",
        SPEC.base
    );
    let response = leg
        .call(
            &ASSOCIATIONS,
            Input::Params(&[
                ("rs_id", Arg::Text(&id)),
                ("page", Arg::Num(page.into())),
                ("size", Arg::Num(size.into())),
            ]),
            biomcp_fetch::Read::<raw::Response>::json()
                .check(&|r| {
                    if r.matches(&id, page, size, &expected) {
                        Ok(())
                    } else {
                        Err(BadReply)
                    }
                })
                .cache_if(&raw::Response::identities_admitted),
        )?
        .value;
    Ok(self::page(response, skip as usize))
}
fn page(response: raw::Response, skip: usize) -> Page {
    let received_rows = response
        .embedded
        .map(|e| e.associations)
        .unwrap_or_default();
    let received = received_rows.len() as u32;
    let mut problems = crate::mapped::Problems {
        source: Source::Gwas,
        record: None,
        list: Vec::new(),
    };
    let mut rows = Vec::new();
    let mut refusals = crate::Refusal::None;
    for row in received_rows.into_iter().skip(skip) {
        if let Some(mut row) = problems.field("association_id", Some(row.value)) {
            refusals.include(row.refusals);
            row.refusals = crate::Refusal::None;
            problems.list.append(&mut row.problems);
            rows.push(row.into());
        } else {
            refusals = crate::Refusal::Refused;
        }
    }
    Page {
        refusals,
        rows,
        received,
        problems: problems.list,
        total: response.page.total_elements,
    }
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
