use super::super::order;
use crate::{Ctx, Outcome, status};
use biodata::Pmid;
use biomcp_fetch::Deadline;
use biomcp_types::{ArticleCardItem, ArticleResolution, ReplyData, Section, Source, SourceState};
pub(super) fn run<'a>(cx: &Ctx<'a>, deadline: &mut Deadline<'a>, ids: &[Pmid], out: &mut Outcome) {
    let result = order::fetch(cx, deadline, ids);
    status::append(&mut out.problems, result.problems.clone());
    let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    else {
        return;
    };
    for item in &mut cards.items {
        join(item, &result);
    }
    for row in &result.rows {
        status::append(&mut out.problems, row.problems.clone());
    }
    out.local = out.local.or(result.local);
    out.sources.push(result.source);
    let state = status::status(cards.items.iter().map(|i| i.status));
    status::get(out, [state].into_iter());
}
fn join(item: &mut ArticleCardItem, result: &order::Counts) {
    let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
        return;
    };
    let Some(id) = card.pmid else {
        return;
    };
    let section = if let Some(row) = result.rows.iter().find(|r| r.pmid == id) {
        Section::Data {
            source: Source::Icite,
            data: row.counts.clone(),
        }
    } else if let Some((_, error)) = result.failures.iter().find(|(pmid, _)| *pmid == id) {
        Section::unavailable(
            Source::Icite,
            crate::status::reason(*error).unwrap_or(biomcp_types::Reason::Unavailable),
        )
    } else {
        Section::Empty {
            source: Source::Icite,
            record: None,
        }
    };
    if result.dropped
        || matches!(section, Section::Unavailable { .. })
        || result.source.state == SourceState::Stale
    {
        item.status = crate::status::item_status(true, false, true);
    }
    if let Some(sections) = &mut item.sections {
        sections.citations = Some(section);
    }
}
