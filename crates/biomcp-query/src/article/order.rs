use crate::{Ctx, Outcome, status};
use biodata::Pmid;
use biomcp_adapters::icite;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{ArticleItem, ReplyData, Source, SourceOutcome};
/// Application citation enrichment window, shared by searches and graph pages.
const MAX_PMIDS: usize = 10_000;
pub(super) struct Counts {
    pub dropped: bool,
    pub rows: Vec<icite::Publication>,
    pub problems: Vec<biomcp_types::Problem>,
    pub source: SourceOutcome,
    pub local: Option<RequestError>,
    pub error: Option<RequestError>,
    pub failures: Vec<(Pmid, RequestError)>,
}
pub(super) fn fetch<'a>(cx: &Ctx<'a>, deadline: &mut Deadline<'a>, ids: &[Pmid]) -> Counts {
    fetch_selection(cx, deadline, ids, false)
}
pub(super) fn fetch_references<'a>(
    cx: &Ctx<'a>,
    deadline: &mut Deadline<'a>,
    ids: &[Pmid],
) -> Counts {
    fetch_selection(cx, deadline, ids, true)
}
fn fetch_selection<'a>(
    cx: &Ctx<'a>,
    deadline: &mut Deadline<'a>,
    ids: &[Pmid],
    references: bool,
) -> Counts {
    let leg = cx.fetch.leg(&icite::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let mut dropped = false;
    let mut rows = Vec::new();
    let mut problems = Vec::new();
    let mut error = None;
    let mut local = None;
    let mut failures = Vec::new();
    let chunks: Vec<_> = ids.chunks(icite::BATCH_LIMIT).collect();
    // A wave bounds transport callbacks to four, without a second thread policy.
    for wave in chunks.chunks(4) {
        for (ids, result) in wave.iter().zip(leg.batch(wave, |child, ids| {
            if references {
                icite::references(child, ids)
            } else {
                icite::publications(child, ids)
            }
        })) {
            match result {
                Ok(values) => {
                    dropped |= values.refusals == biomcp_adapters::Refusal::Refused;
                    rows.extend(values.value);
                    problems.extend(values.problems);
                }
                Err(e) => {
                    failures.extend(ids.iter().map(|id| (*id, e)));
                    error.get_or_insert(e);
                    local = local.or(status::local(e));
                }
            }
        }
    }
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    Counts {
        dropped,
        rows,
        problems,
        source: status::source(Source::Icite, leg.finish(), error),
        error,
        local,
        failures,
    }
}
pub(super) fn apply<'a>(
    cx: &Ctx<'a>,
    deadline: &mut Deadline<'a>,
    items: &mut [ArticleItem],
    out: &mut Outcome,
) -> Result<bool, RequestError> {
    let ids: Vec<_> = items.iter().filter_map(|i| i.pmid).collect();
    if refuse(ids.len(), out) {
        return Ok(false);
    }
    let result = fetch(cx, deadline, &ids);
    out.dropped |= result.dropped;
    status::append(&mut out.problems, result.problems.clone());
    let stale = out
        .sources
        .iter()
        .any(|s| s.source == Source::Icite && s.state == biomcp_types::SourceState::Stale);
    out.sources.retain(|s| s.source != Source::Icite);
    let mut source = result.source;
    if stale && result.error.is_none() {
        source.set_status(biomcp_types::SourceStatus::Stale(source.reason));
    }
    out.sources.push(source);
    if let Some(error) = result.local {
        return Err(error);
    }
    if result.error.is_some() {
        crate::status::item(out, false, true, false);
        return Ok(false);
    }
    for row in &result.rows {
        status::append(&mut out.problems, row.problems.clone());
    }
    for item in items.iter_mut() {
        item.cited_by = result
            .rows
            .iter()
            .find(|r| Some(r.pmid) == item.pmid)
            .and_then(|r| r.counts.cited_by);
    }
    // Stable sort preserves the native fold order for equal or absent counts.
    items.sort_by_key(|item| std::cmp::Reverse(item.cited_by));
    Ok(true)
}
pub(super) fn refuse(count: usize, out: &mut Outcome) -> bool {
    if count <= MAX_PMIDS {
        return false;
    }
    crate::status::item(out, false, true, false);
    out.body = ReplyData::Usage(biomcp_types::UsageError::TooManyPmids {
        count: count as u64,
    });
    true
}
