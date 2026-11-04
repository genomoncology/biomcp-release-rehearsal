use super::{SPEC, map, raw};
use crate::Mapped;
use crate::mapped::problem;
use crate::mapped::record as record_text;
use biodata::{ReactomeId, Text};
use biomcp_fetch::Input;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError};
use biomcp_types::{PathwayCard, PathwayHeader, PathwaySearch, Source};
/// One native page; rejected rows do not change its native position or total.
#[derive(Debug)]
pub struct Page {
    /// Admitted headers and record-local problems.
    pub rows: Mapped<Vec<PathwayHeader>>,
    /// Source-reported unfiltered count or complete observed relation size.
    pub total: u64,
    /// Native rows consumed, independent of admitted rows.
    pub consumed: u32,
}
/// Search exactly the requested native source window, without refill.
pub fn search(leg: &Leg<'_>, q: &PathwaySearch) -> Result<Page, RequestError> {
    let text = q.query().ok_or(biomcp_fetch::InputError::Serialization)?;
    let endpoint = SPEC
        .get("/search/query")
        .fresh(Freshness::Days(1))
        .json_absence();
    let reply = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("query", Arg::Text(text.as_str())),
                ("rows", Arg::Num(q.limit().into())),
                ("Start row", Arg::Num(q.offset().into())),
                ("species", Arg::Text("Homo sapiens")),
                ("types", Arg::Text("Pathway")),
                ("types", Arg::Text("TopLevelPathway")),
                ("Force filters", Arg::Text("true")),
            ]),
            biomcp_fetch::Read::<raw::Search>::json()
                .check(&|v| match v {
                    raw::Search::Page { count, .. } if *count <= q.limit() => Ok(()),
                    raw::Search::Empty(v)
                        if v.messages == [format!("No entries found for query: {text}")] =>
                    {
                        Ok(())
                    }
                    _ => Err(BadReply),
                })
                .cache_if(&|v| match v {
                    raw::Search::Page { results, .. } => results
                        .iter()
                        .flat_map(|g| &g.entries)
                        .all(|r| map::hit(r, &mut map::fields(&r.st_id)).is_some()),
                    raw::Search::Empty(_) => false,
                }),
        )?
        .value;
    let mut rows = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    let (total, consumed) = match reply {
        raw::Search::Page {
            results,
            count,
            total,
        } => {
            for row in results.into_iter().flat_map(|g| g.entries) {
                let mut fields = map::fields(&row.st_id);
                match map::hit(&row, &mut fields) {
                    Some(header) => rows.value.push(header),
                    None => {
                        rows.refusals = crate::Refusal::Refused;
                        if fields.list.is_empty() {
                            rows.problems.push(problem(
                                Source::Reactome,
                                record_text(&row.st_id),
                                "identity",
                            ));
                        }
                    }
                }
                rows.problems.extend(fields.list);
            }
            (total, count)
        }
        raw::Search::Empty(_) => (0, 0),
    };
    Ok(Page {
        rows,
        total,
        consumed,
    })
}
/// Read the complete measured human top-level relation once.
pub fn top_level(leg: &Leg<'_>) -> Result<Mapped<Vec<PathwayHeader>>, RequestError> {
    let endpoint = SPEC
        .get("/data/pathways/top/Homo%20sapiens")
        .fresh(Freshness::Days(7));
    let rows = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Cards>::json().cache_if(&|v| {
                v.0.iter()
                    .all(|r| map::header(r, &mut map::fields(&r.st_id)).is_some())
            }),
        )?
        .value;
    let mut mapped = Mapped {
        refusals: crate::Refusal::None,
        value: Vec::new(),
        problems: Vec::new(),
    };
    for row in rows.0 {
        let mut fields = map::fields(&row.st_id);
        match map::header(&row, &mut fields) {
            Some(v) => mapped.value.push(v),
            None => {
                mapped.refusals = crate::Refusal::Refused;
                if fields.list.is_empty() {
                    mapped.problems.push(problem(
                        Source::Reactome,
                        record_text(&row.st_id),
                        "identity",
                    ));
                }
            }
        }
        mapped.problems.extend(fields.list);
    }
    Ok(mapped)
}
/// Preserve an exact card mismatch as a problem and never cache that body.
pub fn card(
    leg: &Leg<'_>,
    id: &ReactomeId,
    summary: bool,
) -> Result<Mapped<Option<PathwayCard>>, RequestError> {
    let endpoint = SPEC
        .get("/data/query")
        .with_path(format!("/data/query/{id}"))?
        .fresh(Freshness::Days(7));
    let row = match leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Card>::json().cache_if(&|r| {
                r.st_id == id.as_str() && map::header(r, &mut map::fields(&r.st_id)).is_some()
            }),
        )
        .map(|received| received.value)
    {
        Err(RequestError::Source(e)) if e.reason == biomcp_types::Reason::NotFound => {
            return Ok(Mapped {
                refusals: crate::Refusal::None,
                value: None,
                problems: Vec::new(),
            });
        }
        result => result?,
    };
    let mut problems = crate::mapped::Problems {
        source: Source::Reactome,
        record: crate::mapped::text(id.as_str()).ok(),
        list: Vec::new(),
    };
    let mut refusals = crate::Refusal::None;
    let value = if row.st_id == id.as_str() {
        match map::card(row, summary, &mut problems) {
            Some(row) => Some(row),
            None => {
                refusals = crate::Refusal::Refused;
                None
            }
        }
    } else {
        refusals = crate::Refusal::Refused;
        None
    };
    let value = if problems.list.is_empty() {
        problems.required("identity", value)
    } else {
        value
    };
    Ok(Mapped {
        refusals,
        value,
        problems: problems.list,
    })
}
/// Read the source database release independently from pathway version.
pub fn release(leg: &Leg<'_>) -> Result<Text, RequestError> {
    let endpoint = SPEC.get("/data/database/version").fresh(Freshness::Days(1));
    let value = leg
        .call(
            &endpoint,
            Input::Params(&[]),
            biomcp_fetch::Read::<raw::Version>::json_text(),
        )?
        .value;
    Text::new(&value.0.to_string()).map_err(|_| biomcp_fetch::InputError::Serialization.into())
}
