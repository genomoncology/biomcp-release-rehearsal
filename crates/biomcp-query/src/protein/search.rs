//! Retained UniProt windows bind literal selectors and hide native cursors.
use super::next;
use crate::{
    Ctx, Outcome, status,
    store::{Pool, Store},
};
use biodata::Text;
use biomcp_adapters::uniprot;
use biomcp_fetch::{CacheError, Deadline, RequestError};
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    PageInfo, ProteinPage, ProteinSearch, ReplyData, Request, Source, SourceOutcome, Status,
    StepKind,
};
use sha2::{Digest, Sha256};
struct Saved {
    dropped: bool,
    page: uniprot::SearchPage,
    sources: Vec<SourceOutcome>,
    problems: Vec<biomcp_types::Problem>,
}
impl Saved {
    fn encode(&self) -> Result<String, CacheError> {
        serde_json::to_string(&(&self.page, &self.sources, &self.problems, self.dropped))
            .map_err(|_| CacheError::Snapshot)
    }
    fn decode(payload: &str) -> Result<Self, CacheError> {
        type Wire = (
            uniprot::SearchPage,
            Vec<SourceOutcome>,
            Vec<biomcp_types::Problem>,
            bool,
        );
        let (page, sources, problems, dropped): Wire =
            serde_json::from_str(payload).map_err(|_| CacheError::Snapshot)?;
        Ok(Self {
            dropped,
            page,
            sources,
            problems,
        })
    }
}
struct Loaded {
    saved: Saved,
    prior: Option<(Pool, String)>,
}
/// Walk the retained collection; an invalid pool never silently refetches.
pub(crate) fn search(cx: &Ctx<'_>, q: &ProteinSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = super::empty(
        (ProteinPage {
            items: Box::default(),
        })
        .with_query(q.clone()),
    );
    if let Err(error) = read(cx, q, deadline, &mut out) {
        out.local = status::local(error);
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &[biomcp_types::Source::UniProt],
            Request::SearchProtein(q.restart()),
            true,
        ));
    }
    command.finish(deadline.remaining());
    out
}
fn key(q: &ProteinSearch) -> Vec<u8> {
    Sha256::digest(format!(
        "biomcp.protein_pool/2\0{}\0{}\0reviewed-human-uniprot",
        q.query().map_or("", Text::as_str),
        q.gene().map_or("", biodata::GeneSymbol::as_str)
    ))
    .to_vec()
}
fn restart(q: &ProteinSearch, out: &mut Outcome, mismatch: bool) {
    out.body = ReplyData::PoolError { mismatch };
    out.next.push(crate::next::Step::run(
        StepKind::Restart,
        &[biomcp_types::Source::UniProt],
        Request::SearchProtein(q.restart()),
        false,
    ));
}
fn read(
    cx: &Ctx<'_>,
    q: &ProteinSearch,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&uniprot::SPEC, deadline).fresh(cx.fresh);
    if cx.fetch.disabled(Source::UniProt) {
        let error = biomcp_fetch::SourceError {
            source: Source::UniProt,
            reason: biomcp_types::Reason::Disabled,
            status: None,
            retry_after: None,
        }
        .into();
        out.sources
            .push(status::source(Source::UniProt, leg.finish(), Some(error)));
        return Err(error);
    }
    if q.pool().is_none() && q.offset() != 0 || q.pool().is_some() && cx.fresh {
        restart(q, out, false);
        return Ok(());
    }
    let mut store = Store::open(cx.fetch.cache_access()?, deadline)?;
    let loaded = load(q, &leg, &store, cx, out);
    let Loaded { mut saved, prior } = match loaded {
        Ok(Some(saved)) => saved,
        Ok(None) => return Ok(()),
        Err(error) => {
            out.sources
                .push(status::source(Source::UniProt, leg.finish(), Some(error)));
            return Err(error);
        }
    };
    if u64::from(q.offset()) > saved.page.total {
        restart(q, out, false);
        return Ok(());
    }
    let error = extend(&leg, q, &mut saved);
    let incoming = status::source(Source::UniProt, leg.finish(), error);
    if let Some(source) = saved.sources.first_mut() {
        status::retain_source_state(source, &incoming);
    }
    out.sources = saved.sources.clone();
    out.problems = saved.problems.clone();
    out.local = error.and_then(status::local);
    let payload = saved.encode()?;
    let size = saved.page.items.len() as u32;
    let pool = match prior {
        Some((pool, old)) if old == payload => pool,
        Some((pool, _)) => store.extend_source_pool(pool, &payload, size, &out.sources)?,
        None => store.publish_source_pool(&key(q), &payload, size, &out.sources)?,
    };
    render(q, saved, pool.info.id.clone(), pool.info, out);
    Ok(())
}
fn load(
    q: &ProteinSearch,
    leg: &biomcp_fetch::Leg<'_>,
    store: &Store<'_>,
    cx: &Ctx<'_>,
    out: &mut Outcome,
) -> Result<Option<Loaded>, RequestError> {
    let pool = if cx.fresh {
        None
    } else {
        store.find(&key(q), q.pool().map(Text::as_str))?
    };
    if let Some(pool) = pool {
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
        let saved = Saved::decode(&payload)?;
        return Ok(Some(Loaded {
            saved,
            prior: Some((pool, payload)),
        }));
    }
    if q.pool().is_some() {
        restart(q, out, false);
        return Ok(None);
    }
    let mapped = uniprot::search(leg, q, None)?;
    Ok(Some(Loaded {
        saved: Saved {
            dropped: mapped.refusals == biomcp_adapters::Refusal::Refused,
            page: mapped.value,
            sources: vec![status::source(Source::UniProt, Default::default(), None)],
            problems: mapped.problems,
        },
        prior: None,
    }))
}
fn extend(
    leg: &biomcp_fetch::Leg<'_>,
    q: &ProteinSearch,
    saved: &mut Saved,
) -> Option<RequestError> {
    let needed = q.offset().saturating_add(q.limit()) as usize;
    while saved.page.items.len() < needed {
        let Some(cursor) = saved.page.cursor.clone() else {
            break;
        };
        let page = match uniprot::search(leg, q, Some(&cursor)) {
            Ok(v) => v,
            Err(e) => return Some(e),
        };
        if saved
            .page
            .items
            .len()
            .saturating_add(page.value.items.len()) as u64
            > saved.page.total
            || page.value.total != saved.page.total
            || page.value.cursor.as_ref() == Some(&cursor)
            || page.value.items.iter().any(|row| {
                saved
                    .page
                    .items
                    .iter()
                    .any(|old| old.protein.accession == row.protein.accession)
            })
        {
            return Some(
                biomcp_fetch::SourceError {
                    source: Source::UniProt,
                    reason: biomcp_types::Reason::BadReply,
                    status: Some(200),
                    retry_after: None,
                }
                .into(),
            );
        }
        saved.dropped |= page.refusals == biomcp_adapters::Refusal::Refused;
        status::append(&mut saved.problems, page.problems);
        saved.page.items.extend(page.value.items);
        saved.page.cursor = page.value.cursor;
        if saved.page.cursor.is_none() {
            break;
        }
    }
    None
}
fn render(
    q: &ProteinSearch,
    saved: Saved,
    pool: Text,
    info: biomcp_types::PoolInfo,
    out: &mut Outcome,
) {
    let rows: Vec<_> = saved
        .page
        .items
        .into_iter()
        .skip(q.offset() as usize)
        .take(q.limit() as usize)
        .collect();
    out.dropped = saved.dropped;
    out.sources = saved.sources;
    status::search(out);
    let returned = rows.len() as u32;
    let offset = q.offset().saturating_add(returned);
    out.next = next::search(
        q,
        &rows,
        pool,
        offset,
        returned > 0 && u64::from(offset) < saved.page.total,
        out.status() != Status::Complete,
        &mut out.problems,
    );
    out.page = Some(PageInfo {
        offset: q.offset(),
        returned,
        total: saved.page.total,
        pool: Some(info),
        truncated: Some(saved.page.cursor.is_some()),
    });
    out.body = (ProteinPage { items: rows.into() }).with_query(q.clone());
}
