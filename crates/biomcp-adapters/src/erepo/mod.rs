//! ClinGen evidence documents by exact CAid, without gene-wide fallback.
use biomcp_fetch::Input;
mod history;
mod raw;
use crate::{Mapped, mapped::Problems, quotas};
use biomcp_fetch::{Arg, Endpoint, Freshness, KeyUse, Leg, RequestError, SourceSpec};
use biomcp_types::{Source, VariantAssertion};
use std::time::Duration;
/// Exact public evidence repository policy; record rights remain source-specific.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    terms: biomcp_fetch::Terms {
        name: "ClinGen sharing policy; record rights unverified",
        url: "https://clinicalgenome.org/about/terms-of-use/",
        usage: biomcp_types::Use::Restricted,
    },
    source: Source::Erepo,
    base: "https://erepo.clinicalgenome.org",
    key: KeyUse::None,
    quota: &quotas::CLINGEN,
    deadline: Duration::from_secs(8),
};
const FETCH: Endpoint = SPEC
    .get("/evrepo/api/summary/classifications")
    .json_absence()
    .fresh(Freshness::Days(1));
/// Read native pages until completion, retaining source order and checked request identity.
pub fn assertions(
    leg: &Leg<'_>,
    caid: &str,
    selection: Option<(&biomcp_types::AssertionId, Option<&str>)>,
) -> Result<Mapped<Vec<VariantAssertion>>, RequestError> {
    let mapped = records(leg, caid)?;
    let records = mapped.value;
    let mut refusals = mapped.refusals;
    let mut cx = Problems {
        source: Source::Erepo,
        record: None,
        list: mapped.problems,
    };
    if let Some((id, Some(version))) = selection {
        let id = id.as_str();
        let record = records
            .iter()
            .find(|r| r.id == id)
            .ok_or_else(|| biomcp_fetch::bad_reply(Source::Erepo, None))?;
        // The summary names its own exact current document version.
        // Older selections use their own SEPIO body and cannot borrow these facts.
        if record.version != version {
            let uuid = record
                .uuid
                .as_ref()
                .and_then(crate::raw::Field::as_ref)
                .ok_or_else(|| biomcp_fetch::bad_reply(Source::Erepo, None))?;
            if !record
                .versions
                .as_ref()
                .and_then(crate::raw::Field::as_ref)
                .is_some_and(|v| {
                    v.iter()
                        .filter_map(crate::raw::Field::as_ref)
                        .any(|v| v.as_str() == version)
                })
            {
                return Err(biomcp_fetch::bad_reply(Source::Erepo, None));
            }
            let mapped = history::fetch(leg, caid, id, uuid, version)?;
            refusals.include(mapped.refusals);
            cx.list.extend(mapped.problems);
            return Ok(Mapped {
                refusals,
                value: mapped.value.into_iter().collect(),
                problems: cx.list,
            });
        }
    }
    let value = records
        .into_iter()
        .filter_map(|row| match row.map(caid, &mut cx) {
            Some(row) => Some(row),
            None => {
                refusals = crate::Refusal::Refused;
                None
            }
        })
        .collect();
    Ok(Mapped {
        refusals,
        value,
        problems: cx.list,
    })
}
fn records(leg: &Leg<'_>, caid: &str) -> Result<Mapped<Vec<raw::Record>>, RequestError> {
    let mut rows = Vec::new();
    let mut refusals = crate::Refusal::None;
    let mut cx = Problems {
        source: Source::Erepo,
        record: None,
        list: Vec::new(),
    };
    let mut page = 1;
    loop {
        let params = [
            ("columns", Arg::Text("caId")),
            ("values", Arg::Text(caid)),
            ("matchTypes", Arg::Text("exact")),
            ("pgSize", Arg::Num(3)),
            ("pg", Arg::Num(page)),
        ];
        let (reply, _) = leg
            .call(
                &FETCH,
                Input::Params(&params),
                biomcp_fetch::Read::<raw::ReplyBody>::json().cache_if(&|reply| {
                    reply.data.as_ref().is_some_and(|rows| {
                        !rows.is_empty()
                            && rows
                                .iter()
                                .all(|row| row.as_ref().is_some_and(|row| row.matches(caid)))
                    })
                }),
            )
            .map(|received| (received.value, received.meta))?;
        let data = reply.data.unwrap_or_default();
        let complete = data.len() < 3;
        for raw in data {
            cx.record = None;
            let Some(row) = cx.field("record", Some(raw)) else {
                refusals = crate::Refusal::Refused;
                continue;
            };
            cx.record = cx.text("PCERDocID", Some(&row.id));
            let version = cx.text("docVersion", Some(&row.version));
            let allele = cx.optional("caId", Some(row.ca_id.parse::<biodata::VariantId>()));
            if cx.record.is_some() && version.is_some() && allele.is_some() {
                if row.ca_id == caid {
                    rows.push(row);
                } else {
                    refusals = crate::Refusal::Refused;
                    cx.required::<()>("caId", None);
                }
            } else {
                refusals = crate::Refusal::Refused;
            }
        }
        if complete {
            return Ok(Mapped {
                refusals,
                value: rows,
                problems: cx.list,
            });
        }
        page = page
            .checked_add(1)
            .ok_or_else(|| biomcp_fetch::bad_reply(Source::Erepo, None))?;
    }
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
