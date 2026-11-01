//! Exact get and independent context pages share the native association mapping.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod ancestry;
mod next;
use super::map;
use crate::{Action, Ctx, Outcome, status};
use biomcp_adapters::gwas;
use biomcp_fetch::{Deadline, Leg, RequestError};
use biomcp_types::{
    GwasAncestries, GwasAncestry, GwasAssociation, GwasAssociationCard, GwasAssociationSection,
    GwasGet, GwasLoci, GwasLocus, GwasPage, GwasPageQuery, GwasSearch, GwasSectionPage,
    GwasStudySection, PageInfo, Problem, ProblemCode, Reason, ReplyData, Request, Source,
    SourceOutcome, SourceState, Status, StepKind,
};
/// Read only the explicitly selected source resource.
pub(crate) fn get(cx: &Ctx<'_>, q: &GwasGet) -> Outcome {
    let mut out = crate::status::empty(empty(q));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    if let Err(error) = read(cx, q, deadline, &mut out) {
        out.local = status::local(error);
        if matches!(error, RequestError::Source(e) if e.reason == Reason::NotFound) {
            crate::status::item(&mut out, false, false, false);
        }
    }
    if out
        .sources
        .iter()
        .any(|s| s.reason == Some(Reason::BadReply))
    {
        problem(&mut out, "resource", ProblemCode::ValueUnknown);
        if matches!(
            q,
            GwasGet::Study {
                section: Some(GwasStudySection::Ancestries),
                ..
            }
        ) {
            problem(&mut out, "ancestries", ProblemCode::CollectionIncomplete);
        }
    }
    if out.sources.iter().any(|s| s.state == SourceState::Stale) && out.status() == Status::Complete
    {
        crate::status::item(&mut out, true, false, true);
    }
    if out
        .sources
        .iter()
        .any(|s| matches!(s.state, SourceState::Failed | SourceState::Stale))
    {
        out.next.push(crate::next::Step::new(
            StepKind::Fresh,
            &[biomcp_types::Source::Gwas],
            action(q.restart(), true),
        ));
    }
    if matches!(
        q,
        GwasGet::Study {
            section: Some(GwasStudySection::Ancestries),
            ..
        }
    ) && out
        .sources
        .iter()
        .any(|s| s.reason == Some(Reason::TooLarge))
    {
        problem(&mut out, "ancestries", ProblemCode::CollectionIncomplete);
    }
    command.finish(deadline.remaining());
    out
}
fn empty(q: &GwasGet) -> ReplyData {
    match q {
        GwasGet::Association { section: None, .. } => {
            biomcp_types::GwasAssociationCard::into_reply(q.clone(), None)
        }
        GwasGet::Association { .. } => biomcp_types::GwasLoci::into_reply(q.clone(), None),
        GwasGet::Study { section: None, .. } => {
            biomcp_types::GwasStudy::into_reply(q.clone(), None)
        }
        GwasGet::Study {
            section: Some(GwasStudySection::Associations),
            ..
        } => biomcp_types::GwasPage::into_reply(biomcp_types::GwasPageQuery::Get(q.clone()), None),
        GwasGet::Study { .. } => biomcp_types::GwasAncestries::into_reply(q.clone(), None),
    }
}
fn read(
    cx: &Ctx<'_>,
    q: &GwasGet,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if matches!(
        q,
        GwasGet::Study {
            section: Some(GwasStudySection::Ancestries),
            ..
        }
    ) {
        return ancestry::read(cx, q, deadline, out);
    }
    let leg = cx.fetch.leg(&gwas::SPEC, deadline).fresh(cx.fresh);
    let result = resource(q, &leg, out);
    out.sources.push(status::source(
        Source::Gwas,
        leg.finish(),
        result.as_ref().err().copied(),
    ));
    result
}
fn resource(q: &GwasGet, leg: &Leg<'_>, out: &mut Outcome) -> Result<(), RequestError> {
    match q {
        GwasGet::Association {
            id, section: None, ..
        } => {
            let association =
                map::association(gwas::get::association(leg, *id)?, &mut out.problems);
            next::association(&association, out);
            let mut loci = q.clone();
            if let GwasGet::Association { section, .. } = &mut loci {
                *section = Some(GwasAssociationSection::Loci);
            }
            out.next.push(crate::next::Step::new(
                StepKind::Section,
                &[biomcp_types::Source::Gwas],
                action(loci, false),
            ));
            out.body = (GwasAssociationCard { association }).with_query(q.clone());
        }
        GwasGet::Association {
            id,
            section: Some(GwasAssociationSection::Loci),
            ..
        } => {
            let rows = gwas::get::loci(leg, *id)?;
            out.include_refusals(rows.refusals);
            out.include_refusals(rows.refusals);
            out.problems.extend(rows.problems);
            rejected(out, rows.rejected);
            let (items, continuation) = page(q, &rows.items, out);
            next::loci(&items, out);
            out.body = (GwasLoci {
                association: *id,
                items,
                continuation,
            })
            .with_query(q.clone());
        }
        GwasGet::Study {
            id, section: None, ..
        } => {
            let study = gwas::get::study(leg, id)?;
            out.include_refusals(study.refusals);
            out.problems.extend(study.problems);
            for section in [GwasStudySection::Associations, GwasStudySection::Ancestries] {
                let q = GwasGet::Study {
                    id: id.clone(),
                    section: Some(section),
                    page: GwasSectionPage::default(),
                };
                out.next.push(crate::next::Step::new(
                    StepKind::Section,
                    &[biomcp_types::Source::Gwas],
                    action(q, false),
                ));
            }
            out.body = (study.value).with_query(q.clone());
        }
        GwasGet::Study {
            id,
            section: Some(GwasStudySection::Associations),
            page: window,
        } => {
            associations(q, id, window, leg, out)?;
        }
        GwasGet::Study { .. } => return Ok(()),
    }
    if out.status() != Status::Partial {
        crate::status::item(out, true, false, false);
    }
    Ok(())
}
fn action(q: GwasGet, fresh: bool) -> Action {
    Action::Run {
        request: q.into(),
        fresh,
    }
}
fn problem(out: &mut Outcome, field: &str, code: ProblemCode) {
    out.problems.push(Problem {
        source: Source::Gwas,
        record: None,
        field: field.into(),
        code,
    });
}
fn rejected(out: &mut Outcome, count: usize) {
    if count > 0 {
        crate::status::item(out, true, false, true);
        problem(out, "source_url", ProblemCode::ValueUnknown);
    }
}
fn page<T: Clone>(q: &GwasGet, rows: &[T], out: &mut Outcome) -> (Box<[T]>, Option<GwasGet>) {
    let window = q.page();
    let items: Box<[_]> = rows
        .iter()
        .skip(window.offset() as usize)
        .take(window.limit() as usize)
        .cloned()
        .collect();
    let returned = items.len() as u32;
    let next = window.offset().saturating_add(returned);
    let continuation =
        (returned > 0 && (next as usize) < rows.len()).then(|| q.with_page(window.at_offset(next)));
    if let Some(q) = &continuation {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::Gwas],
            action(q.clone(), false),
        ));
    }
    out.page = Some(PageInfo {
        offset: window.offset(),
        returned,
        total: rows.len() as u64,
        pool: None,
        truncated: None,
    });
    (items, continuation)
}

fn associations(
    q: &GwasGet,
    id: &biodata::GwasStudyId,
    window: &GwasSectionPage,
    leg: &Leg<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let selector = GwasSearch::study_page(id.clone(), window);
    let rows = gwas::search(leg, &selector)?;
    out.include_refusals(rows.refusals);
    out.problems.extend(rows.problems);
    let items: Box<[_]> = rows
        .rows
        .into_iter()
        .map(|row| map::association(row, &mut out.problems))
        .collect();
    let returned = items.len() as u32;
    let continuation = (rows.received > 0
        && u64::from(window.offset() - window.offset() % window.limit())
            + u64::from(rows.received)
            < rows.total)
        .then(|| {
            GwasPageQuery::Get(q.with_page(window.at_offset(
                (window.offset() - window.offset() % window.limit()).saturating_add(rows.received),
            )))
        });
    out.page = Some(PageInfo {
        offset: window.offset(),
        returned,
        total: rows.total,
        pool: None,
        truncated: None,
    });
    if let Some(GwasPageQuery::Get(q)) = &continuation {
        out.next.push(crate::next::Step::new(
            StepKind::Page,
            &[biomcp_types::Source::Gwas],
            action(q.clone(), false),
        ));
    }
    out.body = (GwasPage {
        items,
        page: out.page.clone().ok_or(biomcp_fetch::CacheError::Internal)?,
        qualifying_total: None,
        continuation,
    })
    .with_query(biomcp_types::GwasPageQuery::Get(q.clone()));
    Ok(())
}
