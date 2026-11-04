//! Independent machine-written summaries leave accepted article cards intact.
use crate::{Action, Ctx, Outcome, status};

use biomcp_adapters::semantic_scholar;
use biomcp_fetch::Deadline;
use biomcp_types::ArticleInput;
use biomcp_types::{ArticleResolution, Reason, ReplyData, Section, Source, StepKind};
pub(super) fn run<'a>(cx: &Ctx<'a>, deadline: &mut Deadline<'a>, out: &mut Outcome) {
    let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    else {
        return;
    };
    let leg = cx
        .fetch
        .leg(&semantic_scholar::SPEC, *deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let mut error = None;
    for item in &mut cards.items {
        let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
            continue;
        };
        let input = card
            .pmid
            .map(ArticleInput::Pmid)
            .or_else(|| card.doi.clone().map(ArticleInput::Doi));
        let Some(input) = input else {
            continue;
        };
        let section = match semantic_scholar::summary(&leg, &input) {
            Ok(reply) => reply.section(&mut out.problems),
            Err(e) => {
                error.get_or_insert(e);
                out.local = out.local.or(status::local(e));
                item.status = crate::status::item_status(true, false, true);
                let reason = crate::status::reason(e).unwrap_or(biomcp_types::Reason::Unavailable);
                Section::unavailable(Source::SemanticScholar, reason)
            }
        };
        if let Some(sections) = &mut item.sections {
            sections.tldr = Some(section);
        }
    }
    *deadline = super::super::consume(*deadline, started, leg.remaining());
    let source = status::source(Source::SemanticScholar, leg.finish(), error);
    if source.reason == Some(Reason::RateLimited) {
        out.next.push(crate::next::Step::new(
            StepKind::Key,
            &[biomcp_types::Source::SemanticScholar],
            Action::KeyHelp(biomcp_fetch::SecretName::SemanticScholarApiKey),
        ));
    }
    if source.state == biomcp_types::SourceState::Stale {
        for item in &mut cards.items {
            if item.sections.as_ref().is_some_and(|s| s.tldr.is_some()) {
                item.status = crate::status::item_status(true, false, true);
            }
        }
    }
    out.sources.push(source);
    let state = status::status(cards.items.iter().map(|i| i.status));
    status::get(out, [state].into_iter());
}
