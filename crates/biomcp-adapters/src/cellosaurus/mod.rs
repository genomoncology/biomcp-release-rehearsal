//! Literal native Cellosaurus queries and request-bound identity checks.
use biomcp_fetch::{Input, Read, Received};
mod map;
mod raw;
mod receive;
mod release;
pub use release::release;
mod sections;
use biodata::{CellLineSourceId, CellosaurusAccession};
use biomcp_fetch::{
    Arg, BadReply, Freshness, InputError, KeyUse, Leg, PathSegment, ReplyMetadata, RequestError,
    SourceSpec, Terms,
};
use biomcp_types::{CellLineCard, CellLineHeader, CellLineSearchCommand, Source, Use};
pub use sections::complete;
use std::time::Duration;
/// Publisher access and attribution policy, shared by admission and health.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Cellosaurus,
    base: "https://api.cellosaurus.org",
    terms: Terms {
        name: "Cellosaurus CC BY 4.0; cite Bairoch (2018)",
        url: "https://www.cellosaurus.org/description.html",
        usage: Use::Open,
    },
    quota: &crate::quotas::CELLOSAURUS,
    key: KeyUse::None,
    deadline: Duration::from_secs(25),
};
const SEARCH_FIELDS: &str = "ac,id,sy,ox,di,ca";
const CARD_FIELDS: &str = "ac,id,ox,di,ca,sx,ag,problematic,caution,dt";
const VERIFY_FIELDS: &str = "ac,id,sy,ox,di,ca,sx,ag,dr,var,prob,caution,dt";
/// Source-ordered checked candidate page and original retrieval metadata.
#[derive(Debug)]
pub struct Page {
    /// Actual record refusals in the native page.
    pub refusals: crate::Refusal,
    /// Refused row identities and rejected native metadata.
    pub problems: Vec<biomcp_types::Problem>,
    /// Native row count before identity refusals.
    pub received: u32,
    /// Whole bounded headers.
    pub items: Vec<CellLineHeader>,
    /// Retrieval metadata, not release metadata.
    pub metadata: ReplyMetadata,
}
/// Selected card and original retrieval metadata.
#[derive(Debug)]
pub struct Card {
    /// Actual refused native records, independent of optional fields.
    pub refusals: crate::Refusal,
    /// Rejected native metadata for the selected record.
    pub problems: Vec<biomcp_types::Problem>,
    /// Absent only for the exact native404 contract.
    pub card: Option<CellLineCard>,
    /// Retrieval metadata.
    pub metadata: ReplyMetadata,
}
/// Escape one literal without normalizing its spelling.
pub fn literal(field: &str, value: &str) -> String {
    format!(
        "{field}:\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"")
    )
}
/// Search literal text and native taxonomy/disease fields within one checked window.
pub fn search(leg: &Leg<'_>, q: &CellLineSearchCommand) -> Result<Page, RequestError> {
    let accession = q
        .query()
        .and_then(|v| v.as_str().parse::<CellosaurusAccession>().ok());
    let mut terms = Vec::new();
    if let Some(text) = q.query() {
        terms.push(literal(
            if accession.is_some() { "ac" } else { "idsy" },
            text.as_str(),
        ));
    }
    // Native fields, including their source ontology identifiers; no local disease inference.
    // https://api.cellosaurus.org/api-fields documents ox and the NCIt-only din field.
    for (field, value) in [("ox", q.species()), ("din", q.disease())] {
        if let Some(value) = value {
            terms.push(literal(field, value.as_str()));
        }
    }
    let query = terms.join(" AND ");
    candidates(
        leg,
        &query,
        q.limit(),
        (accession.is_some() || q.offset() > 0).then_some(q.offset()),
    )
}
/// One exact cross-reference query; its result never verifies identity.
pub fn reverse(leg: &Leg<'_>, id: &CellLineSourceId) -> Result<Page, RequestError> {
    candidates(
        leg,
        &literal("dr", id.accession()),
        5,
        (native_database(id) == Some("ChEMBL-Cells")).then_some(0),
    )
}
fn candidates(
    leg: &Leg<'_>,
    query: &str,
    limit: u32,
    offset: Option<u32>,
) -> Result<Page, RequestError> {
    let endpoint = SPEC.get("/search/cell-line").fresh(Freshness::Days(1));
    let mut args = vec![
        ("q", Arg::Text(query)),
        ("format", Arg::Text("json")),
        ("fields", Arg::Text(SEARCH_FIELDS)),
        ("rows", Arg::Num(limit.into())),
    ];
    if let Some(offset) = offset {
        args.push(("start", Arg::Num(offset.into())));
    }
    let Received {
        value: reply,
        meta: metadata,
    } = leg.call(
        &endpoint,
        Input::Params(&args),
        Read::<raw::Response>::json()
            .check(&|reply| match reply {
                raw::Response::Records { data } if data.rows.len() <= limit as usize => Ok(()),
                _ => Err(BadReply),
            })
            .cache_if(&|reply| {
                let raw::Response::Records { data } = reply else {
                    return false;
                };
                let accepted = data
                    .rows
                    .iter()
                    .filter(|row| row.accession().is_ok())
                    .count();
                let refusals = if accepted < data.rows.len() {
                    crate::Refusal::Refused
                } else {
                    crate::Refusal::None
                };
                refusals.identities_admitted(accepted, data.rows.len())
            }),
    )?;
    let raw::Response::Records { data } = reply else {
        return Err(InputError::Serialization.into());
    };
    let received = data.rows.len() as u32;
    let mut problems = Vec::new();
    let mut items = Vec::new();
    let mut refusals = crate::Refusal::None;
    for mut row in data.rows {
        problems.append(&mut row.problems);
        if row.accession().is_err() {
            refusals = crate::Refusal::Refused;
            crate::mapped::push_problem(&mut problems, Source::Cellosaurus, None, "accession-list");
            continue;
        }
        if row.name().is_none() {
            crate::mapped::push_problem(
                &mut problems,
                Source::Cellosaurus,
                crate::mapped::record(
                    row.accession()
                        .map_err(|_| biomcp_fetch::bad_reply(Source::Cellosaurus, None))?,
                ),
                "name-list",
            );
        }
        items.push(map::header(row)?);
    }
    Ok(Page {
        refusals,
        items,
        problems,
        received,
        metadata,
    })
}
/// Check the primary accession and optional exact qualified cross-reference before cache admission.
pub fn get(
    leg: &Leg<'_>,
    accession: &CellosaurusAccession,
    source: Option<&CellLineSourceId>,
) -> Result<Card, RequestError> {
    fetch_card(leg, accession, source, false)
}
fn fetch_card(
    leg: &Leg<'_>,
    accession: &CellosaurusAccession,
    source: Option<&CellLineSourceId>,
    complete: bool,
) -> Result<Card, RequestError> {
    let endpoint = SPEC
        .get("/cell-line")
        .with_segments(&[PathSegment::new(accession.as_str())?])
        .fresh(Freshness::Days(1))
        .json_absence();
    let fields = if source.is_some() || complete {
        VERIFY_FIELDS
    } else {
        CARD_FIELDS
    };
    let Received {
        value: reply,
        meta: metadata,
    } = leg.call(
        &endpoint,
        Input::Params(&[("format", Arg::Text("json")), ("fields", Arg::Text(fields))]),
        Read::<raw::Response>::json().check(&|reply| check_card(reply, accession, source)),
    )?;
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    let card = match reply {
        raw::Response::Missing { .. } => None,
        raw::Response::Records { data } => data
            .rows
            .into_iter()
            .next()
            .map(|mut row| {
                problems.append(&mut row.problems);
                if row.name().is_none() {
                    crate::mapped::push_problem(
                        &mut problems,
                        Source::Cellosaurus,
                        crate::mapped::record(accession),
                        "name-list",
                    );
                }
                if complete {
                    let mapped = sections::card(row)?;
                    refusals.include(mapped.refusals);
                    problems.extend(mapped.problems);
                    Ok(mapped.value)
                } else {
                    map::card(row)
                }
            })
            .transpose()?,
    };
    Ok(Card {
        refusals,
        card,
        problems,
        metadata,
    })
}
fn check_card(
    reply: &raw::Response,
    accession: &CellosaurusAccession,
    source: Option<&CellLineSourceId>,
) -> Result<(), BadReply> {
    match reply {
        raw::Response::Missing { code: 404, message }
            if message == &format!("Item not found, ac: {accession}") =>
        {
            Ok(())
        }
        raw::Response::Records { data } if data.rows.len() == 1 => {
            let row = data.rows.first().ok_or(BadReply)?;
            if row.accession()? != accession {
                return Err(BadReply);
            }
            if let Some(id) = source {
                let database = native_database(id).ok_or(BadReply)?;
                if !row
                    .xref_list
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .any(|r| {
                        r.database.as_str() == database && r.accession.as_str() == id.accession()
                    })
                {
                    return Err(BadReply);
                }
            }
            Ok(())
        }
        _ => Err(BadReply),
    }
}
/// Public supported source IDs map only to reviewed native database names.
pub fn native_database(id: &CellLineSourceId) -> Option<&'static str> {
    match id.database() {
        "DepMap" => Some("DepMap"),
        "CellModelPassports" | "Cell_Model_Passport" => Some("Cell_Model_Passport"),
        "ChEMBL" | "ChEMBL-Cells" => Some("ChEMBL-Cells"),
        "PharmacoDB" => Some("PharmacoDB"),
        _ => None,
    }
}
/// Complete source policy and command declaration.
pub(crate) mod decl;
