//! An exact threshold walk retains received identities without scanning ahead.
use super::{map, next};
use crate::{
    Ctx, Outcome, status,
    store::{Pool, Store},
};
use biomcp_adapters::gwas;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    GwasAssociation, GwasPage, GwasPageQuery, GwasSearch, PageInfo, Reason, ReplyData, Request,
    Source, SourceOutcome, StepKind,
};
use sha2::{Digest, Sha256};

#[derive(Default)]
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
    fn encode(&self) -> Result<String, CacheError> {
        serde_json::to_string(&(
            &self.rows,
            self.total,
            self.done,
            &self.sources,
            self.received,
            &self.problems,
            self.dropped,
        ))
        .map_err(|_| CacheError::Snapshot)
    }
    fn decode(payload: &str) -> Result<Self, CacheError> {
        let (rows, total, done, sources, received, problems, dropped) =
            serde_json::from_str(payload).map_err(|_| CacheError::Snapshot)?;
        Ok(Self {
            dropped,
            rows,
            received,
            problems,
            total,
            done,
            sources,
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
fn key(q: &GwasSearch) -> Vec<u8> {
    Sha256::digest(format!(
        "biomcp.gwas_threshold/4\0{}",
        serde_json::to_string(&q.restart()).unwrap_or_default()
    ))
    .to_vec()
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
        let error = biomcp_fetch::SourceError {
            source: Source::Gwas,
            reason: Reason::Disabled,
            status: None,
            retry_after: None,
        }
        .into();
        out.sources
            .push(status::source(Source::Gwas, leg.finish(), Some(error)));
        return Err(error);
    }
    if q.pool().is_none() && q.offset() != 0 || q.pool().is_some() && cx.fresh {
        restart(q, out, false);
        return Ok(());
    }
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let prior = if cx.fresh {
        None
    } else {
        store.find(&key(q), q.pool().map(biodata::Text::as_str))?
    };
    let Some(mut saved) = load(cx, q, &store, prior.as_ref(), out)? else {
        return Ok(());
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
        saved.sources = vec![incoming];
    } else {
        out.sources = saved.sources.clone();
    }
    let payload = saved.encode()?;
    let size = saved.rows.len() as u32;
    let pool = match prior {
        Some(pool) => store.extend_source_pool(pool, &payload, size, &saved.sources)?,
        None => store.publish_source_pool(&key(q), &payload, size, &saved.sources)?,
    };
    render(cx, q, saved, pool, out);
    Ok(())
}
fn load(
    cx: &Ctx<'_>,
    q: &GwasSearch,
    store: &Store<'_>,
    prior: Option<&Pool>,
    out: &mut Outcome,
) -> Result<Option<Saved>, RequestError> {
    let Some(pool) = prior else {
        if q.pool().is_some() {
            restart(q, out, false);
            return Ok(None);
        }
        return Ok(Some(Saved::default()));
    };
    if pool.key != key(q) {
        restart(q, out, true);
        return Ok(None);
    }
    let Some(payload) =
        store.source_snapshot(pool.info.id.as_str(), &key(q), |s| cx.fetch.disabled(s))?
    else {
        restart(q, out, false);
        return Ok(None);
    };
    Ok(Some(Saved::decode(&payload)?))
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
fn render(cx: &Ctx<'_>, q: &GwasSearch, saved: Saved, pool: Pool, out: &mut Outcome) {
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
    let continuation = (!saved.done || (offset as usize) < saved.rows.len())
        .then(|| GwasPageQuery::Search(q.in_pool(pool.info.id.clone(), offset)));
    let page = PageInfo {
        offset: q.offset(),
        returned,
        total: saved.total.unwrap_or(0),
        pool: Some(pool.info),
        truncated: Some(!saved.done),
    };
    let body = GwasPage {
        items,
        page: page.clone(),
        continuation,
        qualifying_total: saved.done.then_some(saved.rows.len() as u64),
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
    out.dropped = saved.dropped;
    status::search(out);
}
