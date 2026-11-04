//! Exact therapy selection preserves independently filtered native claim streams.
use super::{
    drug_map, drug_raw as raw, drug_request::fetch, variant::failure, variant_raw::Connection,
};
use crate::Mapped;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{
    CivicClaimKind, CivicClaimPage, DrugCivic, DrugCivicPage, DrugCivicStreams, DrugGet, Reason,
    Section, Source,
};

fn seek<T>(
    leg: &Leg<'_>,
    id: std::num::NonZeroU32,
    query: &DrugGet,
    kind: CivicClaimKind,
    mut rows: Connection<T>,
    extract: fn(raw::Data) -> Option<Connection<T>>,
) -> Result<Connection<T>, RequestError> {
    let offset = if query.civic_cursor().is_some() {
        0
    } else {
        query.section_offset().unwrap_or(0)
    };
    let mut skipped = 0_u32;
    while skipped < offset {
        if !rows.page_info.has_next_page {
            rows.nodes.clear();
            return Ok(rows);
        }
        skipped = skipped
            .checked_add(rows.nodes.len() as u32)
            .ok_or_else(|| failure(Reason::TooLarge))?;
        let count = if skipped < offset {
            (offset - skipped).min(50)
        } else {
            query.section_limit().unwrap_or(1)
        };
        let data = fetch(
            leg,
            id,
            query,
            Some(kind),
            rows.page_info.end_cursor.as_ref(),
            count,
        )?;
        rows = extract(data).ok_or_else(|| failure(Reason::BadReply))?;
    }
    Ok(rows)
}
fn page<T>(
    rows: &Connection<T>,
    query: &DrugGet,
    kind: CivicClaimKind,
) -> Result<CivicClaimPage<DrugGet>, RequestError> {
    let next = if rows.page_info.has_next_page {
        Some(
            query
                .with_civic_stream(kind, rows.page_info.end_cursor.clone())
                .map_err(|_| failure(Reason::BadReply))?,
        )
    } else {
        None
    };
    Ok(CivicClaimPage {
        returned: rows.nodes.len() as u32,
        total: rows.total_count,
        next,
    })
}
/// Retrieve only the exact native therapy and its requested native claim collections.
/// Claim profiles retain their own native context; collection totals never become profile counts.
pub fn drug_civic(
    leg: &Leg<'_>,
    query: &DrugGet,
    id: std::num::NonZeroU32,
) -> Result<Mapped<Option<DrugCivic>>, RequestError> {
    if query.source() != biomcp_types::DrugSource::Civic {
        return Err(failure(Reason::BadReply));
    }
    let count = first_count(query);
    let data = fetch(
        leg,
        id,
        query,
        query.civic_kind(),
        query.civic_cursor(),
        count,
    )?;
    let mut cx = crate::mapped::Problems {
        source: Source::Civic,
        record: None,
        list: Vec::new(),
    };
    let identity = data
        .therapy
        .ok_or_else(|| failure(Reason::NotFound))?
        .header(&mut cx);
    let Some(identity) = identity else {
        return Ok(Mapped {
            refusals: crate::Refusal::Refused,
            value: None,
            problems: cx.list,
        });
    };
    if identity.id.to_string() != query.input().to_string() {
        cx.required::<()>("id", None);
        return Ok(Mapped {
            refusals: crate::Refusal::Refused,
            value: None,
            problems: cx.list,
        });
    }
    let mut refusals = crate::Refusal::None;
    let evidence = data
        .evidence_items
        .map(|rows| {
            collection(
                leg,
                query,
                (id, CivicClaimKind::Evidence, rows),
                |d| d.evidence_items,
                drug_map::evidence,
                &mut cx,
                &mut refusals,
            )
        })
        .transpose()?;
    let assertions = data
        .assertions
        .map(|rows| {
            collection(
                leg,
                query,
                (id, CivicClaimKind::Assertions, rows),
                |d| d.assertions,
                drug_map::assertion,
                &mut cx,
                &mut refusals,
            )
        })
        .transpose()?;
    Ok(Mapped {
        refusals,
        value: Some(DrugCivic {
            identity,
            civic: Section::Data {
                source: Source::Civic,
                data: DrugCivicStreams {
                    evidence,
                    assertions,
                },
            },
        }),
        problems: cx.list,
    })
}

fn first_count(query: &DrugGet) -> u32 {
    let offset = if query.civic_cursor().is_some() {
        0
    } else {
        query.section_offset().unwrap_or(0)
    };
    if offset > 0 {
        offset.min(50)
    } else {
        query.section_limit().unwrap_or(1)
    }
}

fn collection<T>(
    leg: &Leg<'_>,
    query: &DrugGet,
    stream: (std::num::NonZeroU32, CivicClaimKind, Connection<T>),
    extract: fn(raw::Data) -> Option<Connection<T>>,
    map: fn(T, &DrugGet, &mut crate::mapped::Problems) -> Option<biomcp_types::DrugCivicClaim>,
    cx: &mut crate::mapped::Problems,
    refusals: &mut crate::Refusal,
) -> Result<DrugCivicPage, RequestError> {
    let (id, kind, rows) = stream;
    let rows = seek(leg, id, query, kind, rows, extract)?;
    let mut page = page(&rows, query, kind)?;
    let records: Box<[_]> = rows
        .nodes
        .into_iter()
        .filter_map(|row| {
            cx.record = None;
            match map(row, query, cx) {
                Some(row) => Some(row),
                None => {
                    *refusals = crate::Refusal::Refused;
                    None
                }
            }
        })
        .collect();
    page.returned = records.len() as u32;
    Ok(DrugCivicPage { records, page })
}
