//! ChEMBL metadata and independent assay-count disclosure.
use super::{CellLineGet, Ctx, Outcome, Request, Source, StepKind, empty, recovery, run, status};
use biodata::CellosaurusAccession;
use biomcp_adapters::chembl;
use biomcp_types::ReplyBody;
use biomcp_types::ReplyData;
use biomcp_types::{
    Capped, CellLineAssayCount, CellLineAssayCountQuery, CellLineChembl, Problem, ProblemCode,
};
/// One native metadata page, never an assay request.
pub(super) fn get(cx: &Ctx<'_>, query: &CellLineGet, accession: &CellosaurusAccession) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let out = get_at(cx, query, accession, deadline);
    command.finish(deadline.remaining());
    out
}
pub(super) fn get_at(
    cx: &Ctx<'_>,
    query: &CellLineGet,
    accession: &CellosaurusAccession,
    deadline: biomcp_fetch::Deadline<'_>,
) -> Outcome {
    let leg = cx.fetch.leg(&chembl::SPEC, deadline).fresh(cx.fresh);
    let mut body = CellLineChembl {
        accession: accession.clone(),
        items: Capped {
            items: Box::default(),
            total: 0,
        },
        offset: query.section_offset(),
        release: None,
        fetched: None,
    };
    let limit = query.section_limit().unwrap_or(5);
    let reply = chembl::cells(&leg, accession, limit, body.offset);
    let error = reply.as_ref().err().copied();
    let mut out = empty((body.clone()).with_query(query.clone()));
    out.sources
        .push(status::source(Source::Chembl, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok(page) = reply {
        out.problems.extend(page.problems);
        let total = page.window.total().unwrap_or(0);
        let next_offset = page.window.next_offset();
        let more = page.window.has_more();
        out.include_refusals(page.window.refusals());
        let items = page.window.rows;
        let metadata = page.metadata;
        crate::status::item(&mut out, total != 0, false, more);
        if let Some(offset) = next_offset {
            out.next.push(crate::next::Step::new(
                StepKind::Page,
                &[biomcp_types::Source::Chembl],
                run(Request::GetCellLine(query.at_section_offset(offset))),
            ));
        }
        for row in &items {
            out.next.push(crate::next::Step::new(
                StepKind::Section,
                &[biomcp_types::Source::Chembl],
                run(Request::CellLineAssayCount(CellLineAssayCountQuery {
                    id: row.id.clone(),
                })),
            ));
        }
        body.items = Capped {
            items: items.into(),
            total,
        };
        body.fetched = metadata.fetched.and_then(|value| u64::try_from(value).ok());
        out.body = body.with_query(query.clone());
        attach(cx, deadline, &mut out);
    } else if error.is_some() {
        out.problems.push(problem("cellosaurus_id"));
    }
    recovery(&mut out, Request::GetCellLine(query.clone()));
    out
}
/// One exact assay request with its native total, including zero.
pub(crate) fn assay_count(cx: &Ctx<'_>, query: &CellLineAssayCountQuery) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&chembl::SPEC, deadline).fresh(cx.fresh);
    let reply = chembl::assay_count(&leg, &query.id);
    let error = reply.as_ref().err().copied();
    let mut out = empty(
        (CellLineAssayCount {
            id: query.id.clone(),
            assay_count: 0,
            release: None,
            fetched: None,
        })
        .with_query(query.clone()),
    );
    out.sources
        .push(status::source(Source::Chembl, leg.finish(), error));
    out.local = error.and_then(status::local);
    if let Ok((assay_count, metadata)) = reply {
        crate::status::item(&mut out, true, false, false);
        out.body = (CellLineAssayCount {
            id: query.id.clone(),
            assay_count,
            release: None,
            fetched: metadata.fetched.and_then(|value| u64::try_from(value).ok()),
        })
        .with_query(query.clone());
        attach(cx, deadline, &mut out);
    } else if error.is_some() {
        out.problems.push(problem("cell_chembl_id"));
    }
    recovery(&mut out, Request::CellLineAssayCount(query.clone()));
    command.finish(deadline.remaining());
    out
}
fn problem(field: &str) -> Problem {
    Problem {
        source: Source::Chembl,
        record: None,
        field: field.into(),
        code: ProblemCode::ValueUnknown,
    }
}
fn attach(cx: &Ctx<'_>, deadline: biomcp_fetch::Deadline<'_>, out: &mut Outcome) {
    let leg = cx.fetch.leg(&chembl::SPEC, deadline).fresh(cx.fresh);
    let release = chembl::release(&leg);
    let error = release.as_ref().err().copied();
    let incoming = status::source(Source::Chembl, leg.finish(), error);
    if let Some(source) = out.sources.last_mut() {
        status::retain_source_state(source, &incoming);
    }
    if error.is_some() {
        crate::status::item(out, true, false, true);
        out.local = error.and_then(status::local);
        out.problems.push(problem("release"));
    }
    let release = release.ok().map(|mapped| {
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        mapped.value
    });
    match &mut out.body {
        ReplyData::CellLineChembl {
            body: Some(body), ..
        } => body.release = release.clone(),
        ReplyData::CellLineAssayCount {
            body: Some(body), ..
        } => body.release = release.clone(),
        _ => {}
    }
    if out
        .sources
        .iter()
        .any(|s| s.state == biomcp_types::SourceState::Stale)
    {
        crate::status::item(out, true, false, true);
    }
}
