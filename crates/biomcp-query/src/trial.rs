//! Trial discovery delegates filters, totals and ranking to the named registry.
use biomcp_types::ReplyBody;
mod condition;
pub mod document;
mod gene_notice;
pub(crate) mod get;
mod nci;
use crate::{Action, Ctx, Outcome, status};

use biomcp_adapters::{ctgov, hgnc};
use biomcp_types::GeneResolution;
use biomcp_types::{
    ReplyData, Request, Source, SourceState, Status, StepKind, TrialPage, TrialSearch,
};
/// Execute a checked native search without locally screening returned studies.
pub(crate) fn search(cx: &Ctx<'_>, query: &TrialSearch) -> Outcome {
    if query.source() == biomcp_types::TrialSource::Nci {
        return nci::search(cx, query);
    }
    let mut outcome = empty(query);
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    let Some(condition) = condition::resolve(cx, query, &mut outcome, &mut deadline) else {
        command.finish(deadline.remaining());
        return outcome;
    };
    let Some(gene) = gene(cx, query, &mut outcome, &mut deadline) else {
        command.finish(deadline.remaining());
        return outcome;
    };
    let leg = cx.fetch.leg(&ctgov::SPEC, deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let conditions = condition
        .as_ref()
        .map_or(query.condition(), std::slice::from_ref);
    let (result, name_notice) = ctgov::search_query(&leg, query, gene.as_deref(), conditions);
    let rejected = name_notice.is_none() && request_rejected(result.as_ref().err());
    let version = match &result {
        Ok(_) => ctgov::version(&leg),
        Err(error) => Err(*error),
    };
    let error = result
        .as_ref()
        .err()
        .or(version.as_ref().err())
        .copied()
        .or_else(|| result.as_ref().ok().and_then(|page| page.count_error));
    deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let mut source = status::source(Source::ClinicalTrialsGov, leg.finish(), error);
    source.notices.extend(name_notice);
    {
        let (found, identity_failed, unavailable) = if result.is_err() {
            (false, true, false)
        } else if version.is_err()
            || error.is_some()
            || source.state == SourceState::Stale
            || outcome
                .sources
                .iter()
                .any(|source| source.state != SourceState::Ok)
        {
            (true, false, true)
        } else {
            (true, false, false)
        };
        crate::status::item(&mut outcome, found, identity_failed, unavailable);
    };
    if let Ok(page) = result {
        if page.condition_word_mismatch {
            condition::word_notice(query, cx.fresh, &mut source, &mut outcome);
        }
        if let Some(gene) = gene.as_deref() {
            gene_notice::append(gene, &page.gene_contexts, &mut source);
        }
        source.count = page.total;
        source.retrieved_count = Some(page.items.len() as u32);
        apply_page(query, cx.fresh, gene.as_deref(), &mut outcome, page);
    }
    if let Ok(version) = version {
        outcome.problems.extend(version.problems);
        if let ReplyData::TrialPage {
            body: Some(body), ..
        } = &mut outcome.body
        {
            body.source_updated = version.value.map(Into::into);
        }
    }
    if rejected {
        outcome.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
    }
    outcome.local = error.and_then(status::local);
    outcome.sources.push(source);
    status::search_for(&mut outcome, &[Source::ClinicalTrialsGov]);
    command.finish(deadline.remaining());
    outcome
}
fn request_rejected(error: Option<&biomcp_fetch::RequestError>) -> bool {
    matches!(error, Some(biomcp_fetch::RequestError::Source(error)) if error.status == Some(400))
}

fn apply_page(
    query: &TrialSearch,
    fresh: bool,
    symbol: Option<&str>,
    outcome: &mut Outcome,
    mut page: ctgov::SearchPage,
) {
    outcome.dropped = page.refusals == biomcp_adapters::Refusal::Refused;
    if outcome.dropped {
        crate::status::item(outcome, true, false, false);
    }
    outcome.problems.append(&mut page.problems);
    let short = page.items.len() < query.limit() as usize
        && page
            .total
            .is_some_and(|total| total < u64::from(query.limit()));
    if query.count_only() {
        page.items.clear();
    }
    if short && outcome.status() == Status::Complete {
        outcome.next.extend(
            query
                .wider_searches(symbol.and_then(|value| biodata::Text::new(value).ok()))
                .into_iter()
                .map(|query| {
                    crate::next::Step::run(
                        StepKind::Disambiguate,
                        &[query.source().source()],
                        Request::SearchTrial(query),
                        fresh,
                    )
                }),
        );
    }
    outcome.page = page.total.map(|total| biomcp_types::PageInfo {
        offset: query.offset(),
        returned: page.items.len() as u32,
        total,
        pool: None,
        truncated: None,
    });
    if let Some(token) = page.next.filter(|_| !query.count_only()) {
        outcome.next.push(crate::next::Step::run(
            StepKind::Page,
            &[query.source().source()],
            Request::SearchTrial(query.at_page(
                token,
                query.offset().saturating_add(page.items.len() as u32),
            )),
            fresh,
        ));
    }
    if let ReplyData::TrialPage {
        body: Some(body), ..
    } = &mut outcome.body
    {
        body.items = page.items.into();
    }
}
fn empty(query: &TrialSearch) -> Outcome {
    crate::status::finish(
        [crate::status::item_status(false, true, false)],
        crate::status::Parts {
            dropped: false,
            body: (TrialPage {
                items: Box::default(),
                source_updated: None,
                attribution:
                    "ClinicalTrials.gov; source-native discovery, no eligibility assessment".into(),
                gene_resolution: None,
                condition_resolution: None,
            })
            .with_query(query.clone()),
            page: None,
            sources: Vec::new(),
            problems: Vec::new(),
            next: Vec::new(),
            local: None,
        },
    )
}
fn gene<'a>(
    cx: &Ctx<'a>,
    query: &TrialSearch,
    outcome: &mut Outcome,
    deadline: &mut biomcp_fetch::Deadline<'a>,
) -> Option<Option<String>> {
    let Some(input) = query.gene() else {
        return Some(None);
    };
    let leg = cx.fetch.leg(&hgnc::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let result = crate::gene::resolve(&leg, input);
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let error = result.as_ref().err().copied();
    outcome
        .sources
        .push(status::source(Source::Hgnc, leg.finish(), error));
    match result {
        Ok(mapped) => {
            outcome.problems.extend(mapped.problems);
            match mapped.value {
                GeneResolution::Resolved { gene, .. } => Some(Some(gene.symbol.to_string())),
                resolution => {
                    crate::status::item(outcome, true, false, true);
                    if let ReplyData::TrialPage {
                        body: Some(body), ..
                    } = &mut outcome.body
                    {
                        body.gene_resolution = Some(resolution);
                    }
                    None
                }
            }
        }
        Err(error) => {
            outcome.local = status::local(error);
            None
        }
    }
}
