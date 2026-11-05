//! Count buckets keep their own shape instead of pretending to be report pages.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::faers;
use biomcp_types::{
    AdverseEventPage, AdverseEventReactionCounts, AdverseEventSearch, ReplyBody, ReportUnit, Source,
};

pub(super) fn search(cx: &Ctx<'_>, query: &AdverseEventSearch) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&faers::SPEC, deadline).fresh(cx.fresh);
    let mut body = AdverseEventPage {
        reaction_counts: None,
        reaction_match: query
            .reaction()
            .map(|_| biomcp_types::AdverseEventReactionMatch::PhraseMatch),
        source_query: query.faers_expression().into_boxed_str(),
        source_updated: None,
        fetched: None,
        items: Box::default(),
        unit: ReportUnit::Reports,
        components: Box::default(),
    };
    let mapped = faers::reactions(&leg, query).and_then(|page| {
        body.fetched = page
            .metadata
            .fetched
            .map(u64::try_from)
            .transpose()
            .map_err(|_| biomcp_fetch::InputError::Serialization)?;
        if !page.items.is_empty() {
            body.reaction_counts = Some(AdverseEventReactionCounts {
                limit_reached: page.items.len() == query.limit() as usize,
                items: page.items.into(),
            });
        }
        body.source_updated = page.updated.map(Into::into);
        body.components = page.components;
        Ok(())
    });
    let error = mapped.err();
    let mut out = status::finish(
        [status::item_status(false, true, false)],
        status::Parts {
            body: body.with_query(query.clone()),
            page: None,
            sources: vec![status::source(Source::Faers, leg.finish(), error)],
            problems: Vec::new(),
            next: Vec::new(),
            local: error.and_then(status::local),
            dropped: false,
        },
    );
    status::search(&mut out);
    command.finish(deadline.remaining());
    out
}
