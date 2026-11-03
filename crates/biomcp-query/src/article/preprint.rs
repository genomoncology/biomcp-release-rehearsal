//! Europe PMC owns preprint cards and document fetching; section dispatch stays shared.
use crate::{Ctx, Outcome, status};

use biomcp_adapters::europepmc;
use biomcp_fetch::Deadline;
use biomcp_types::ArticleInput;
use biomcp_types::{
    ArticleMatch, ArticleResolution, ReplyData, Section, Source, SourceState, Status,
};
pub(super) fn cards<'a>(cx: &Ctx<'a>, deadline: &mut Deadline<'a>, out: &mut Outcome) {
    let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    else {
        return;
    };
    let leg = cx.fetch.leg(&europepmc::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let mut error = None;
    let mut used = false;
    for item in &mut cards.items {
        let ArticleInput::EuropePmc(id) = &item.input else {
            continue;
        };
        used = true;
        match europepmc::core(&leg, id) {
            Ok((card, metadata)) => {
                out.problems.extend(card.problems);
                item.status = crate::status::item_status(true, false, false);
                item.answer = Some(ArticleResolution::Resolved {
                    card: card.value.card(),
                    r#match: ArticleMatch {
                        kind: item.input.kind(),
                    },
                    fetched: metadata.fetched.and_then(|value| u64::try_from(value).ok()),
                });
            }
            Err(e) => {
                error.get_or_insert(e);
                out.local = out.local.or(status::local(e));
                item.status = crate::status::item_status(false, true, false);
                item.answer = None;
            }
        }
    }
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let source = status::source(Source::EuropePmc, leg.finish(), error);
    if used {
        if source.state == SourceState::Stale {
            for item in &mut cards.items {
                if matches!(item.input, ArticleInput::EuropePmc(_))
                    && item.status == Status::Complete
                {
                    item.status = crate::status::item_status(true, false, true);
                }
            }
        }
        out.sources.push(source);
        let state = status::status(cards.items.iter().map(|i| i.status));
        status::get(out, [state].into_iter());
    }
}
pub(super) fn fulltext<'a>(cx: &Ctx<'a>, deadline: &mut Deadline<'a>, out: &mut Outcome) {
    let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    else {
        return;
    };
    let leg = cx.fetch.leg(&europepmc::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let mut error = None;
    let mut used = false;
    for item in &mut cards.items {
        let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
            continue;
        };
        let Some(id) = &card.europepmc_id else {
            continue;
        };
        let Some(sections) = &mut item.sections else {
            continue;
        };
        used = true;
        let result = europepmc::read(&leg, id).map(|mapped| {
            out.dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
            out.problems.extend(mapped.problems);
            mapped.value
        });
        sections.fulltext = Some(match result {
            Ok(Some(document)) => {
                if !document.body.parts.is_empty() {
                    super::sections::fulltext::outline_next(
                        ArticleInput::EuropePmc(id.clone()),
                        &mut out.next,
                        &mut out.problems,
                    );
                }
                Section::Data {
                    source: Source::EuropePmc,
                    data: super::sections::fulltext::outline(&document),
                }
            }
            Ok(None) => Section::unavailable(Source::EuropePmc, biomcp_types::Reason::NotOpen),
            Err(e) => {
                error.get_or_insert(e);
                out.local = out.local.or(status::local(e));
                item.status = crate::status::item_status(true, false, true);
                Section::unavailable(
                    Source::EuropePmc,
                    crate::status::reason(e).unwrap_or(biomcp_types::Reason::Unavailable),
                )
            }
        });
    }
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let source = status::source(Source::EuropePmc, leg.finish(), error);
    if used {
        if source.state == SourceState::Stale {
            for item in &mut cards.items {
                if matches!(item.input, ArticleInput::EuropePmc(_))
                    && item.status == Status::Complete
                {
                    item.status = crate::status::item_status(true, false, true);
                }
            }
        }
        if let Some(previous) = out
            .sources
            .iter_mut()
            .find(|s| s.source == Source::EuropePmc)
        {
            *previous = source;
        } else {
            out.sources.push(source);
        }
        let state = status::status(cards.items.iter().map(|i| i.status));
        status::get(out, [state].into_iter());
    }
}
