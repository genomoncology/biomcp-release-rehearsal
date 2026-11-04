//! Native reviewed-human pages, keeping only a validated opaque cursor.
use super::{SPEC, entry};
use crate::Mapped;
use biodata::{ProteinHeader, Text};
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, Reply, RequestError};
use biomcp_fetch::{Input, Read};
use biomcp_types::{Problem, ProblemCode, ProteinSearch, Source};
use serde::{Deserialize, Serialize};
const PATH: &str = "/uniprotkb/search";
const FIELDS: &str = "accession,id,protein_name,gene_names,organism_id,reviewed,length";
/// One admitted native window and its exact source count.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchPage {
    /// Native source order, without choosing among candidates.
    pub items: Vec<ProteinHeader>,
    /// Native total of the same search.
    pub total: u64,
    /// Opaque next cursor, never a user-facing URL.
    pub cursor: Option<Text>,
}
#[derive(Debug, Deserialize)]
struct Rows {
    results: Vec<crate::raw::Field<entry::raw::Entry>>,
}
impl Reply for Rows {
    fn check(&self) -> Result<(), BadReply> {
        if self.results.len() > 5 {
            return Err(BadReply);
        }
        Ok(())
    }
}
/// Send the exact proved search shape and admit only reviewed human rows.
pub fn search(
    leg: &Leg<'_>,
    q: &ProteinSearch,
    cursor: Option<&Text>,
) -> Result<Mapped<SearchPage>, RequestError> {
    let query = native_query(q);
    let mut args = vec![
        ("query", Arg::Text(&query)),
        ("format", Arg::Text("json")),
        ("size", Arg::Num(5)),
    ];
    if q.query().is_some() {
        args.push(("fields", Arg::Text(FIELDS)));
    }
    if let Some(cursor) = cursor {
        args.push(("cursor", Arg::Text(cursor.as_str())));
    }
    let ep = SPEC.get(PATH).identity_encoding().fresh(Freshness::Days(1));
    let (raw, meta) = leg
        .call(&ep, Input::Params(&args), Read::<Rows>::json())
        .map(|received| (received.value, received.meta))?;
    let page = meta
        .pagination
        .ok_or_else(|| biomcp_fetch::bad_reply(Source::UniProt, Some(200)))?;
    let returned = raw.results.len() as u64;
    if returned > page.total
        || returned == 0 && page.total > 0
        || cursor.is_none() && returned < page.total && page.next.is_none()
    {
        return Err(biomcp_fetch::bad_reply(Source::UniProt, Some(200)));
    }
    let cursor = page
        .next
        .as_deref()
        .map(|url| {
            SPEC.next_cursor(url, PATH, &args, "cursor")
                .map_err(|_| biomcp_fetch::bad_reply(Source::UniProt, Some(200)))
                .and_then(|value| {
                    Text::new(&value)
                        .map_err(|_| biomcp_fetch::bad_reply(Source::UniProt, Some(200)))
                })
        })
        .transpose()?;
    let mut items = Vec::new();
    let mut problems = Vec::new();
    let mut refusals = crate::Refusal::None;
    for raw in raw.results {
        let Some(raw) = raw.value() else {
            refusals = crate::Refusal::Refused;
            crate::mapped::push_problem(&mut problems, Source::UniProt, None, "identity");
            continue;
        };
        let id = raw.primary_accession.clone();
        let mapped = entry::map::entry(raw, &id);
        refusals.include(mapped.refusals);
        problems.extend(mapped.problems);
        if let Some(row) = mapped.value {
            if items
                .iter()
                .any(|h: &ProteinHeader| h.protein.accession == id)
            {
                refusals = crate::Refusal::Refused;
                problems.push(Problem {
                    source: Source::UniProt,
                    record: crate::mapped::record(&id),
                    field: "primaryAccession".into(),
                    code: ProblemCode::ValueUnknown,
                });
            } else {
                items.push(row.header);
            }
        }
    }
    Ok(Mapped {
        refusals,
        value: SearchPage {
            items,
            total: page.total,
            cursor,
        },
        problems,
    })
}
fn native_query(q: &ProteinSearch) -> String {
    let literal = q
        .gene()
        .map(|v| format!("gene_exact:{v}"))
        .unwrap_or_else(|| q.query().map_or("", Text::as_str).into());
    format!("({literal}) AND (organism_id:9606) AND (reviewed:true)")
}
