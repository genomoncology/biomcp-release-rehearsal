//! One explicit adverse-event source; no inference, drug choice or source probing.
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
mod get;
mod vaers;
use crate::{Ctx, Outcome, status};
use biomcp_adapters::{faers, maude};
use biomcp_types::{
    AdverseEventPage, AdverseEventSearch, PageInfo, ReplyData, ReportUnit, Request, Source,
    StepKind,
};
/// Search report headers through the declared source and its reviewed component admission.
pub(crate) fn search(cx: &Ctx<'_>, query: &AdverseEventSearch) -> Outcome {
    if query.source() == biomcp_types::AdverseEventSource::Vaers {
        return vaers::search(cx, query);
    }
    let source = if query.source() == biomcp_types::AdverseEventSource::Faers {
        Source::Faers
    } else {
        Source::Maude
    };
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let spec = if source == Source::Faers {
        &faers::SPEC
    } else {
        &maude::SPEC
    };
    let leg = cx.fetch.leg(spec, deadline).fresh(cx.fresh);
    let selector = if source == Source::Faers {
        faers::expression(query)
    } else {
        maude::expression(query)
    };
    let reply = if source == Source::Faers {
        faers::search(&leg, query)
    } else {
        maude::search(&leg, query)
    };
    let error = reply.as_ref().err().copied();
    let mut out = crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: (AdverseEventPage {
                reaction_match: query
                    .reaction()
                    .map(|_| biomcp_types::AdverseEventReactionMatch::PhraseMatch),
                source_query: selector.into_boxed_str(),
                source_updated: None,
                fetched: None,
                items: Box::default(),
                unit: ReportUnit::Reports,
                components: Box::default(),
            })
            .with_query(query.clone()),
            page: None,
            sources: vec![status::source(source, leg.finish(), error)],
            problems: Vec::new(),
            next: Vec::new(),
            local: error.and_then(status::local),
        },
    );
    if let Ok(page) = reply {
        let returned = page.items.len() as u32;
        navigation(query, &page, &mut out);
        out.include_refusals(page.refusals);
        out.problems.extend(page.problems);
        out.page = Some(PageInfo {
            offset: query.offset(),
            returned,
            total: page.total,
            pool: None,
            truncated: None,
        });
        crate::status::search(&mut out);
        if let ReplyData::AdverseEventPage {
            body: Some(body), ..
        } = &mut out.body
        {
            body.items = page.items.into();
            body.components = page.components;
            body.source_updated = page.updated.map(Into::into);
            body.fetched = page
                .metadata
                .fetched
                .and_then(|value| u64::try_from(value).ok());
        }
    }
    command.finish(deadline.remaining());
    out
}

/// An unimplemented report get has no source effects until its owning slice lands.
pub(crate) fn unsupported() -> Outcome {
    crate::status::empty(ReplyData::Usage(
        biomcp_types::UsageError::UnsupportedAdverseEvent,
    ))
}

fn navigation(query: &AdverseEventSearch, page: &faers::Page, out: &mut Outcome) {
    let returned = page.items.len() as u32;
    for row in &page.items {
        if let Some(get) = crate::next::checked(
            biomcp_types::AdverseEventGet::new(row.id.clone(), query.source()),
            query.source().source(),
            &mut out.problems,
        ) {
            out.next.push(crate::next::Step::run(
                StepKind::Disambiguate,
                &[query.source().source()],
                Request::GetAdverseEvent(get),
                false,
            ));
        }
    }
    let offset = query.offset().saturating_add(returned);
    if returned > 0 && u64::from(offset) < page.total {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[query.source().source()],
            Request::SearchAdverseEvent(query.at_offset(offset)),
            false,
        ));
    }
}

/// Read an exact FAERS report and its independent retained sections.
pub(crate) fn get(cx: &Ctx<'_>, query: &biomcp_types::AdverseEventGet) -> Outcome {
    if query.source() != biomcp_types::AdverseEventSource::Faers {
        return unsupported();
    }
    get::get(cx, query)
}
