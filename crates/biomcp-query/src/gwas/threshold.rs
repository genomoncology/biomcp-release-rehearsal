//! An exact threshold walk retains received identities without scanning ahead.
use super::{map, next};
use crate::{Ctx, Outcome, source_snapshot, status, store::Pool};
use biomcp_adapters::gwas;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    GwasAssociation, GwasPage, GwasPageQuery, GwasSearch, PageInfo, Reason, ReplyData, Request,
    Source, SourceOutcome, StepKind,
};

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Saved {
    dropped: bool,
    rows: Vec<GwasAssociation>,
    received: u32,
    problems: Vec<biomcp_types::Problem>,
    total: Option<u64>,
    done: bool,
    sources: Vec<SourceOutcome>,
}
impl Saved {
    fn incomplete(&self) -> bool {
        self.dropped
            || self.problems.iter().any(|problem| {
                matches!(
                    problem.field.as_ref(),
                    "pvalue_mantissa" | "pvalue_exponent"
                )
            })
            || self.sources.iter().any(|source| {
                matches!(
                    source.state,
                    biomcp_types::SourceState::Failed
                        | biomcp_types::SourceState::Stale
                        | biomcp_types::SourceState::Syncing
                )
            })
    }
}
pub(super) fn search(cx: &Ctx<'_>, q: &GwasSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = crate::status::empty(biomcp_types::GwasPage::into_reply(
        biomcp_types::GwasPageQuery::Search(q.clone()),
        None,
    ));
    if let Err(error) = read(cx, q, deadline, &mut out) {
        out.local = status::local(error);
        out.body = biomcp_types::GwasPage::into_reply(
            biomcp_types::GwasPageQuery::Search(q.clone()),
            None,
        );
        out.page = None;
        out.sources = vec![status::source(
            Source::Gwas,
            Default::default(),
            Some(error),
        )];
        if out
            .sources
            .iter()
            .any(|s| s.reason == Some(Reason::BadReply))
        {
            out.problems.push(biomcp_types::Problem {
                source: Source::Gwas,
                record: None,
                field: "association_page".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            });
        }
        out.next = next::search(
            &q.restart(),
            None,
            biomcp_types::SourceState::Failed,
            cx.fresh,
            &mut out.problems,
        );
    }
    command.finish(deadline.remaining());
    out
}
fn key(q: &GwasSearch) -> Result<Vec<u8>, RequestError> {
    source_snapshot::key("gwas-threshold", &q.restart())
}

fn restart(q: &GwasSearch, out: &mut Outcome, mismatch: bool) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[biomcp_types::Source::Gwas],
        Request::SearchGwas(q.restart()),
        false,
    ));
}
fn read(
    cx: &Ctx<'_>,
    q: &GwasSearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&gwas::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::Gwas) {
        let error = biomcp_fetch::SourceError::disabled(Source::Gwas).into();
        out.sources
            .push(status::source(Source::Gwas, leg.finish(), Some(error)));
        return Err(error);
    }
    if q.pool().is_none() && q.offset() != 0 || q.pool().is_some() && cx.fresh {
        restart(q, out, false);
        return Ok(());
    }
    let key = key(q)?;
    let (prior, mismatch) = if cx.fresh {
        (None, false)
    } else {
        source_snapshot::find::<Saved>(cx, &key, q.pool(), deadline)?
    };
    let (mut saved, prior) = match prior {
        Some((saved, pool)) => (saved, Some(pool)),
        None if q.pool().is_some() => {
            restart(q, out, mismatch);
            return Ok(());
        }
        None => (Saved::default(), None),
    };
    if q.offset() as usize > saved.rows.len() {
        restart(q, out, false);
        return Ok(());
    }
    let needed = q.offset().saturating_add(q.limit()) as usize;
    if !saved.done && saved.rows.len() < needed {
        let result = gwas::search(&leg, &q.at_offset(saved.received));
        let error = result.as_ref().err().copied();
        let mut incoming = status::source(Source::Gwas, leg.finish(), error);
        if let Ok(page) = &result {
            incoming.retrieved_count = Some(page.rows.len() as u32);
            incoming.count = Some(page.total);
        }
        out.sources = vec![incoming.clone()];
        let page = result?;
        append(q, &mut saved, page)?;
        if let Some(source) = saved.sources.first_mut() {
            status::retain_source_state(source, &incoming);
        } else {
            saved.sources = vec![incoming];
        }
        out.sources = saved.sources.clone();
    } else {
        out.sources = saved.sources.clone();
    }
    let size = saved.rows.len() as u32;
    let pool = crate::publish::keep(
        source_snapshot::publish(cx, &key, &saved, size, &saved.sources, prior, deadline),
        out,
    )?;
    render(cx, q, saved, pool, out);
    Ok(())
}
fn append(q: &GwasSearch, saved: &mut Saved, page: gwas::Page) -> Result<(), RequestError> {
    if saved.total.is_some_and(|total| total != page.total) {
        return Err(biomcp_fetch::bad_reply(Source::Gwas, Some(200)));
    }
    let threshold = q
        .p_value()
        .ok_or_else(|| biomcp_fetch::bad_reply(Source::Gwas, Some(200)))?;
    saved.received = saved
        .received
        .checked_add(page.received)
        .ok_or_else(|| biomcp_fetch::bad_reply(Source::Gwas, Some(200)))?;
    if u64::from(saved.received) > page.total {
        return Err(biomcp_fetch::bad_reply(Source::Gwas, Some(200)));
    }
    saved.dropped |= page.refusals == biomcp_adapters::Refusal::Refused;
    saved.problems.extend(page.problems);
    saved.total = Some(page.total);
    for row in page.rows {
        let Some(exact) = row.value.exact_p_value.clone() else {
            saved.dropped = true;
            let problem = biomcp_types::Problem {
                source: Source::Gwas,
                record: row.value.id.get().to_string().parse().ok(),
                field: "pvalue_mantissa".into(),
                code: biomcp_types::ProblemCode::ValueUnknown,
            };
            saved.problems.extend(row.problems);
            if !saved.problems.iter().any(|p| {
                p.record == problem.record
                    && matches!(p.field.as_ref(), "pvalue_mantissa" | "pvalue_exponent")
            }) {
                saved.problems.push(problem);
            }
            continue;
        };
        if saved.rows.iter().any(|old| old.id == row.value.id)
            || saved
                .rows
                .last()
                .and_then(|old| old.exact_p_value.as_ref())
                .is_some_and(|last| last > &exact)
        {
            return Err(biomcp_fetch::bad_reply(Source::Gwas, Some(200)));
        }
        if exact > *threshold {
            saved.done = true;
            break;
        }
        saved.rows.push(map::association(row, &mut saved.problems));
    }
    if saved.rows.len() as u64 > page.total {
        return Err(biomcp_fetch::bad_reply(Source::Gwas, Some(200)));
    }
    saved.done |= u64::from(saved.received) == page.total;
    if !saved.done && page.received == 0 {
        return Err(biomcp_fetch::bad_reply(Source::Gwas, Some(200)));
    }
    Ok(())
}
fn render(cx: &Ctx<'_>, q: &GwasSearch, saved: Saved, pool: Option<Pool>, out: &mut Outcome) {
    let incomplete = saved.incomplete();
    out.problems = saved.problems.clone();
    let items: Box<[_]> = saved
        .rows
        .iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .cloned()
        .collect();
    let returned = items.len() as u32;
    let offset = q.offset().saturating_add(returned);
    let continuation = pool
        .as_ref()
        .filter(|_| !saved.done || (offset as usize) < saved.rows.len())
        .map(|pool| GwasPageQuery::Search(q.in_pool(pool.info.id.clone(), offset)));
    let page = PageInfo {
        offset: q.offset(),
        returned,
        total: saved.total.unwrap_or(0),
        pool: pool.map(|p| p.info),
        truncated: Some(!saved.done),
    };
    let body = GwasPage {
        items,
        page: page.clone(),
        continuation,
        qualifying_total: (saved.done && !incomplete).then_some(saved.rows.len() as u64),
    };
    out.next = next::search(
        q,
        Some(&body),
        biomcp_types::SourceState::Ok,
        cx.fresh,
        &mut out.problems,
    );
    out.page = Some(page);
    out.body = body.with_query(biomcp_types::GwasPageQuery::Search(q.clone()));
    out.dropped = incomplete;
    status::search(out);
}

impl crate::source_snapshot::Observation for Saved {
    fn whole(&self) -> bool {
        !self.incomplete()
    }
}
