//! CPIC retrieval preserves source contexts and performs no clinical inference.
use biomcp_types::ReplyBody;
mod annotations;
mod annotations_record;
mod next;
use crate::{Ctx, Outcome, status};
pub(crate) use annotations::annotation;
use biomcp_adapters::cpic;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{
    PgxCard, PgxDiplotype, PgxGet, PgxGuidelines, PgxHeader, PgxPage, PgxPageInfo,
    PgxRecommendations, PgxSearch, PgxSection, PgxSource, PgxTotalKind, Problem, ProblemCode,
    Reason, ReplyData, Request, Source, SourceOutcome, SourceState,
};
/// Discover one independent native CPIC pair page.
pub(crate) fn search(cx: &Ctx<'_>, query: &PgxSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&cpic::SPEC, deadline).fresh(cx.fresh);
    let mut problems = Vec::new();
    let mut consumed = 0;
    let mut refusals = biomcp_adapters::Refusal::None;
    let reply = cpic::search(&leg, query).map(|page| {
        refusals.include(page.refusals);
        problems.extend(page.problems);
        consumed = page.consumed;
        let info = page_info(query.offset(), page.items.len(), page.total);
        (PgxPage {
            header: header(query.gene()),
            items: page.items.into(),
            page: info,
        })
        .with_query(query.clone())
    });
    let empty = (PgxPage {
        header: header(query.gene()),
        items: Box::default(),
        page: page_info(query.offset(), 0, 0),
    })
    .with_query(query.clone());
    let mut out = finish(leg, reply, empty, Source::Cpic);
    out.include_refusals(refusals);
    status::append(&mut out.problems, problems);
    out.next = next::steps_with(&Request::SearchPgx(query.clone()), &mut out, Some(consumed));
    command.finish(deadline.remaining());
    out
}
/// Read one explicit card or section with the same source policy on every path.
pub(crate) fn get(cx: &Ctx<'_>, query: &PgxGet) -> Outcome {
    if query.source() == PgxSource::Clinpgx {
        return annotations::get(cx, query);
    }
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&cpic::SPEC, deadline).fresh(cx.fresh);
    let mut problems = Vec::new();
    let mut consumed = 0;
    let mut refusals = biomcp_adapters::Refusal::None;
    let reply = read(&leg, query, &mut problems, &mut consumed, &mut refusals);
    let mut out = finish(leg, reply, empty(query), Source::Cpic);
    out.include_refusals(refusals);
    status::append(&mut out.problems, problems);
    out.next = next::steps_with(&Request::GetPgx(query.clone()), &mut out, Some(consumed));
    command.finish(deadline.remaining());
    out
}
fn read(
    leg: &Leg<'_>,
    q: &PgxGet,
    problems: &mut Vec<Problem>,
    consumed: &mut u32,
    refusals: &mut biomcp_adapters::Refusal,
) -> Result<ReplyData, RequestError> {
    let header = header(q.gene());
    Ok(match q.section() {
        None => {
            let page = cpic::card(leg, q)?;
            *consumed = page.consumed;
            refusals.include(page.refusals);
            problems.extend(page.problems);
            biomcp_types::PgxCard::into_reply(
                q.clone(),
                page.items
                    .into_iter()
                    .next()
                    .map(|identity| PgxCard { header, identity }),
            )
        }
        Some(PgxSection::Recommendations) if q.drug_id().is_some() => {
            let p = cpic::drug_recommendations(leg, q)?;
            *consumed = p.consumed;
            refusals.include(p.refusals);
            problems.extend(p.problems);
            (PgxRecommendations::Table {
                header,
                page: page_info(q.section_offset(), p.items.len(), p.total),
                items: p.items.into(),
            })
            .with_query(q.clone())
        }
        Some(PgxSection::Recommendations) => {
            let p = cpic::recommendations(leg, q)?;
            *consumed = p.consumed;
            refusals.include(p.refusals);
            problems.extend(p.problems);
            (PgxRecommendations::View {
                header,
                page: page_info(q.section_offset(), p.items.len(), p.total),
                items: p.items.into(),
            })
            .with_query(q.clone())
        }
        Some(PgxSection::Guidelines) => {
            let p = cpic::guidelines(leg, q)?;
            *consumed = p.consumed;
            refusals.include(p.refusals);
            problems.extend(p.problems);
            (PgxGuidelines {
                header,
                page: page_info(q.section_offset(), p.items.len(), p.total),
                items: p.items.into(),
            })
            .with_query(q.clone())
        }
        Some(PgxSection::Diplotype) => {
            let page = cpic::diplotype(leg, q)?;
            *consumed = page.consumed;
            refusals.include(page.refusals);
            problems.extend(page.problems);
            (PgxDiplotype {
                header,
                diplotypes: page.items.into(),
            })
            .with_query(q.clone())
        }
        Some(PgxSection::Annotations) => return Err(biomcp_fetch::InputError::Serialization.into()),
    })
}
fn empty(q: &PgxGet) -> ReplyData {
    let header = header(q.gene());
    let page = page_info(q.section_offset(), 0, 0);
    match q.section() {
        None | Some(PgxSection::Annotations) => biomcp_types::PgxCard::into_reply(q.clone(), None),
        Some(PgxSection::Recommendations) if q.drug_id().is_some() => (PgxRecommendations::Table {
            header,
            items: Box::default(),
            page,
        })
        .with_query(q.clone()),
        Some(PgxSection::Recommendations) => (PgxRecommendations::View {
            header,
            items: Box::default(),
            page,
        })
        .with_query(q.clone()),
        Some(PgxSection::Guidelines) => (PgxGuidelines {
            header,
            items: Box::default(),
            page,
        })
        .with_query(q.clone()),
        Some(PgxSection::Diplotype) => (PgxDiplotype {
            header,
            diplotypes: Box::default(),
        })
        .with_query(q.clone()),
    }
}
fn finish(
    leg: Leg<'_>,
    reply: Result<ReplyData, RequestError>,
    empty: ReplyData,
    source: Source,
) -> Outcome {
    let error = reply.as_ref().err().copied();
    let source = status::source(source, leg.finish(), error);
    let state = if reply.is_err() {
        crate::status::item_status(false, true, false)
    } else if source.state == SourceState::Stale {
        crate::status::item_status(true, false, true)
    } else {
        crate::status::item_status(true, false, false)
    };
    crate::status::finish(
        [state],
        crate::status::Parts {
            dropped: false,
            body: reply.unwrap_or(empty),
            page: None,
            problems: record_problem(&source).into_iter().collect(),
            sources: vec![source],
            next: vec![],
            local: error.and_then(status::local),
        },
    )
}
fn header(gene: &biodata::GeneSymbol) -> PgxHeader {
    PgxHeader {
        gene: gene.clone(),
        source: PgxSource::Cpic,
    }
}
fn page_info(offset: u32, returned: usize, total: u64) -> PgxPageInfo {
    PgxPageInfo {
        offset,
        returned: returned as u32,
        total,
        total_kind: PgxTotalKind::Source,
    }
}

fn record_problem(source: &SourceOutcome) -> Option<Problem> {
    (source.reason == Some(Reason::BadReply)).then(|| Problem {
        source: source.source,
        record: None,
        field: "record".into(),
        code: ProblemCode::ValueUnknown,
    })
}
