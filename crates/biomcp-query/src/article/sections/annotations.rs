use crate::{Ctx, status};
use biodata::Pmid;
use biomcp_adapters::pubtator;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    Annotations, ArticleCardItem, ArticleResolution, Problem, Section, Source, SourceOutcome,
    SourceState,
};
pub(super) struct Result {
    pub refused: Vec<Pmid>,
    pub values: Vec<(Pmid, Section<Annotations>)>,
    pub source: SourceOutcome,
    pub problems: Vec<Problem>,
    pub local: Option<RequestError>,
}
pub(super) fn ids(items: &[ArticleCardItem]) -> Vec<Pmid> {
    let mut ids = Vec::new();
    for item in items {
        if item
            .sections
            .as_ref()
            .is_none_or(|s| s.annotations.is_some())
        {
            continue;
        }
        if let Some(ArticleResolution::Resolved { card, .. }) = &item.answer
            && let Some(id) = card.pmid
            && !ids.contains(&id)
        {
            ids.push(id);
        }
    }
    ids
}
pub(super) fn run<'a>(cx: &Ctx<'a>, deadline: &mut Deadline<'a>, ids: &[Pmid]) -> Result {
    let leg = cx.fetch.leg(&pubtator::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let chunks: Vec<_> = ids.chunks(pubtator::BATCH_LIMIT).collect();
    let replies = leg.batch(&chunks, |leg, ids| pubtator::annotations(leg, ids));
    let error = replies.iter().find_map(|r| r.as_ref().err().copied());
    let local = replies
        .iter()
        .find_map(|r| r.as_ref().err().copied().and_then(status::local));
    let mut refused = Vec::new();
    let mut values = Vec::new();
    let mut problems = Vec::new();
    for (chunk, reply) in chunks.iter().zip(replies) {
        match reply {
            Ok(rows) => {
                for row in rows {
                    if row.annotations.refusals == biomcp_adapters::Refusal::Refused {
                        refused.push(row.id);
                    }
                    values.push((row.id, row.annotations.value));
                    status::append(&mut problems, row.annotations.problems);
                }
            }
            Err(error) => {
                let reason =
                    crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable);
                values.extend(
                    chunk
                        .iter()
                        .map(|id| (*id, Section::unavailable(Source::PubTator, reason))),
                );
            }
        }
    }
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    Result {
        refused,
        values,
        source: status::source(Source::PubTator, leg.finish(), error),
        problems,
        local,
    }
}
pub(super) fn join(items: &mut [ArticleCardItem], result: &Result) {
    for item in items {
        let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
            continue;
        };
        let Some((id, section)) = result.values.iter().find(|(id, _)| Some(*id) == card.pmid)
        else {
            continue;
        };
        if matches!(section, Section::Unavailable { .. })
            || result.source.state == SourceState::Stale
            || result.refused.contains(id)
        {
            item.status = crate::status::item_status(true, false, true);
        }
        if let Some(sections) = &mut item.sections {
            sections.annotations = Some(section.clone());
        }
    }
}
