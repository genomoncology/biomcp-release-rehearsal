//! Canonical OPENTARGETS source declaration.
use biomcp_fetch::Input;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::Source;
use std::time::Duration;
/// Publisher policy shared by health, request admission and response outcomes.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Opentargets,
    base: "https://api.platform.opentargets.org/api/v4",
    terms: Terms {
        name: crate::drug_sources::OPENTARGETS_TERMS.name,
        url: crate::drug_sources::OPENTARGETS_TERMS.url,
        usage: crate::drug_sources::OPENTARGETS_TERMS.usage,
    },
    quota: &crate::quotas::OPENTARGETS,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};
mod raw;
use biomcp_fetch::{BadReply, Freshness, Leg, RequestError};
pub use map::{Drug, Indications, Mechanisms, Search};
use serde::Serialize;
const GRAPHQL: biomcp_fetch::Endpoint = SPEC.post_json("/graphql").fresh(Freshness::Days(1));
const DETAIL: &str = "query Drug($id:String!){drug(chemblId:$id){id name indications{rows{maxClinicalStage disease{id \
    name}}} mechanismsOfAction{rows{mechanismOfAction actionType targets{id approvedSymbol \
    approvedName}}}}}";
const VERSION: &str = "query DataVersion{meta{dataVersion{year month}}}";
#[derive(Serialize)]
struct Query<'a, T> {
    query: &'a str,
    variables: T,
}
#[derive(Serialize)]
struct SearchVariables<'a> {
    q: &'a str,
}
#[derive(Serialize)]
struct DrugVariables<'a> {
    id: &'a str,
}
/// Fetch one native candidate page in source order with its unchanged native total.
pub fn search(
    leg: &Leg<'_>,
    text: &str,
    index: u32,
    size: u32,
) -> Result<(Search, biomcp_fetch::ReplyMetadata), RequestError> {
    if !(1..=50).contains(&size) {
        return Err(RequestError::Source(biomcp_fetch::SourceError {
            source: Source::Opentargets,
            reason: biomcp_types::Reason::BadReply,
            status: None,
            tls_cause: None,
            retry_after: None,
        }));
    }
    let query = search_query(index, size);
    let body = Query {
        query: &query,
        variables: SearchVariables { q: text },
    };
    leg.call(
        &GRAPHQL,
        Input::json(&[], &body),
        biomcp_fetch::Read::json().check(&|r: &raw::Response<raw::SearchData>| {
            if r.data.search.hits.len() <= size as usize
                && (r.data.search.hits.is_empty()
                    || u64::from(index) * u64::from(size) + r.data.search.hits.len() as u64
                        <= r.data.search.total)
            {
                Ok(())
            } else {
                Err(BadReply)
            }
        }),
    )
    .map(|received| (received.value, received.meta))
    .map(|(r, meta)| (map::Search(r.data.search), meta))
}
/// Read one checked source ChEMBL identity; all section paging occurs over this returned collection.
pub fn drug(
    leg: &Leg<'_>,
    id: &biodata::DrugId,
) -> Result<(Option<Drug>, biomcp_fetch::ReplyMetadata), RequestError> {
    let id = id.to_string();
    let body = Query {
        query: DETAIL,
        variables: DrugVariables { id: &id },
    };
    leg.call(
        &GRAPHQL,
        Input::json(&[], &body),
        biomcp_fetch::Read::json().check(&|r: &raw::Response<raw::DrugData>| {
            if r.data.drug.as_ref().is_none_or(|d| d.id == id) {
                Ok(())
            } else {
                Err(BadReply)
            }
        }),
    )
    .map(|received| (received.value, received.meta))
    .map(|(r, meta)| (r.data.drug.map(map::Drug), meta))
}
/// Retain the platform's published data release on this leg.
/// An unavailable or unreadable release is disclosed and leaves the source's facts readable.
pub fn version(leg: &Leg<'_>) -> Vec<biomcp_types::Problem> {
    let body = Query {
        query: VERSION,
        variables: (),
    };
    let reply = leg.call(
        &GRAPHQL,
        Input::json(&[], &body),
        biomcp_fetch::Read::<raw::Response<raw::VersionData>>::json(),
    );
    let date = reply.map_err(|_| BadReply).and_then(|received| {
        let version = received.value.data.meta.data_version;
        release_month(version.and_then(crate::raw::Field::value).ok_or(BadReply)?)
    });
    let mut problems = crate::mapped::Problems {
        source: Source::Opentargets,
        record: None,
        list: Vec::new(),
    };
    if let Some(date) = problems.optional("meta.dataVersion", Some(date)) {
        leg.data_date(&date);
    }
    problems.list
}
/// Spell a release as a year and month; it has no day, so it carries no age claim.
fn release_month(version: raw::DataVersion) -> Result<String, BadReply> {
    let year: u16 = version.year.parse().map_err(|_| BadReply)?;
    let year = match version.year.len() {
        2 => 2000 + year,
        4 => year,
        _ => return Err(BadReply),
    };
    let month: u8 = version.month.parse().map_err(|_| BadReply)?;
    if (1..=12).contains(&month) {
        Ok(format!("{year}-{month:02}"))
    } else {
        Err(BadReply)
    }
}
mod map;
pub use map::{
    card, header, indications as map_indications, search_headers, targets as map_targets,
};

fn search_query(index: u32, size: u32) -> String {
    format!(
        "query \
        DrugSearch($q:String!){{search(queryString:$q,entityNames:[\"drug\"],page:{{index:{index},size:{size\
        }}}){{total hits{{id name entity}}}}}}"
    )
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
