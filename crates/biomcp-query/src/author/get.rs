use super::{empty, recovery};
use crate::{Ctx, Outcome, status};
use biodata::AuthorId;
use biomcp_adapters::semantic_scholar::{self, author};
use biomcp_types::ReplyBody;
use biomcp_types::{
    AuthorCardItem, AuthorCards, AuthorGet, Reason, Request, Source, SourceState, Status,
};
/// Read each exact requested record once and preserve every input slot.
pub(crate) fn get(cx: &Ctx<'_>, q: &AuthorGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = empty(cards(q, Vec::new()));
    let mut items = Vec::new();
    for input in q.ids() {
        let (spec, source) = match input {
            AuthorId::SemanticScholar(_) => (&semantic_scholar::SPEC, Source::SemanticScholar),
            AuthorId::Orcid(_) => (&biomcp_adapters::orcid::SPEC, Source::Orcid),
        };
        let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
        let reply = match input {
            AuthorId::SemanticScholar(id) => {
                author::card(&leg, id).map(|value| biomcp_adapters::Mapped {
                    refusals: biomcp_adapters::Refusal::None,
                    value: Some(value.value),
                    problems: value.problems,
                })
            }
            AuthorId::Orcid(id) => biomcp_adapters::orcid::card(&leg, id),
        };
        let mut state = match &reply {
            Ok(mapped) if mapped.value.is_some() => crate::status::item_status(true, false, false),
            Ok(_) => crate::status::item_status(false, false, false),
            Err(biomcp_fetch::RequestError::Source(e)) if e.reason == Reason::NotFound => {
                crate::status::item_status(false, false, false)
            }
            Err(e) => {
                out.local = out.local.or(status::local(*e));
                crate::status::item_status(false, true, false)
            }
        };
        let outcome = status::source(
            source,
            leg.finish(),
            super::source_error(reply.as_ref().err().copied()),
        );
        if outcome.state == SourceState::Stale && state == Status::Complete {
            state = crate::status::item_status(true, false, true);
        }
        if let Some(existing) = out.sources.iter_mut().find(|s| s.source == source) {
            status::retain_source_state(existing, &outcome);
        } else {
            out.sources.push(outcome);
        }
        let card = reply.ok().and_then(|mapped| {
            out.include_refusals(mapped.refusals);
            out.problems.extend(mapped.problems);
            mapped.value
        });
        items.push(AuthorCardItem {
            input: input.clone(),
            status: state,
            card,
        });
    }
    status::get(&mut out, items.iter().map(|i| i.status));
    recovery(cx, &mut out, Request::GetAuthor(q.clone()));
    for item in &items {
        if item.card.is_some() && out.next.len() < 5 {
            let step = crate::next::Step::new(
                biomcp_types::StepKind::Pivot,
                &[super::next::author(&item.input)],
                crate::Action::AuthorPapers(biomcp_types::AuthorPapersQuery::new(
                    item.input.clone(),
                )),
            );
            if !out.next.contains(&step) {
                out.next.push(step);
            }
        }
    }
    out.body = cards(q, items);
    out.next.truncate(5);
    command.finish(deadline.remaining());
    out
}

fn cards(query: &AuthorGet, items: Vec<AuthorCardItem>) -> biomcp_types::ReplyData {
    AuthorCards {
        items: items.into(),
    }
    .with_query(query.clone())
}
