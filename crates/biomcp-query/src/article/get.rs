//! PubMed card resolution under one command deadline.
use crate::{Ctx, Outcome, status};
use biodata::Pmid;
use biomcp_adapters::pubmed;
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::ReplyBody;
use biomcp_types::{Article, ArticleInput};
use biomcp_types::{
    ArticleCardItem, ArticleCards, ArticleGet, ArticleMatch, ArticleResolution, ReplyData, Source,
    SourceState, Status,
};
/// Resolve and hydrate article cards under one absolute command deadline.
pub(crate) fn get(cx: &Ctx<'_>, query: &ArticleGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&pubmed::SPEC, deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let mut resolutions = Vec::new();
    let mut ids = Vec::new();
    let mut error = None;
    let mut local = None;
    for input in query.input() {
        let result = pubmed::resolve_ids(&leg, input, cx.ncbi_email.as_ref());
        if let Err(e) = result {
            error.get_or_insert(e);
            local = local.or(status::local(e));
        }
        if let Ok(found) = &result {
            for id in found {
                if !ids.contains(id) {
                    ids.push(*id);
                }
            }
        }
        resolutions.push(result);
    }
    let (records, failures) = fetch(&leg, &ids, cx);
    for (_, e) in &failures {
        error.get_or_insert(*e);
        local = local.or(status::local(*e));
    }
    let remaining = leg.remaining();
    let source = status::source(Source::PubMed, leg.finish(), error);
    let stale = source.state == SourceState::Stale;
    let mut items = Vec::new();
    for (input, resolution) in query.input().iter().zip(resolutions) {
        match resolution {
            Err(_) => items.push(item(
                input,
                crate::status::item_status(false, true, false),
                None,
            )),
            Ok(ids) => append(input, &ids, &records, &failures, stale, &mut items),
        }
    }
    let mut out = crate::status::finish(
        [status::status(items.iter().map(|i| i.status))],
        crate::status::Parts {
            dropped: records
                .iter()
                .any(|(record, _)| record.refusals == biomcp_adapters::Refusal::Refused),
            body: (ArticleCards {
                items: items.into(),
            })
            .with_query(query.clone()),
            sources: sources(query, source),
            page: None,
            problems: records
                .iter()
                .flat_map(|(record, _)| record.problems.iter().cloned())
                .collect(),
            next: Vec::new(),
            local,
        },
    );
    let mut deadline = deadline.after_stage(started.saturating_sub(remaining));
    super::preprint::cards(cx, &mut deadline, &mut out);
    let requested = if let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    {
        super::sections::prepare(query, &mut cards.items)
    } else {
        false
    };
    if requested {
        super::sections::indexing::join(query, &records, &mut out);
        super::sections::run(cx, &mut deadline, query, &mut out);
    }
    super::next::get(query, &mut out);
    command.finish(deadline.remaining());
    out
}
pub(super) type Record = (pubmed::FetchedArticle, Option<i64>);
type Failure = (Pmid, RequestError);
fn fetch(leg: &Leg<'_>, ids: &[Pmid], cx: &Ctx<'_>) -> (Vec<Record>, Vec<Failure>) {
    let chunks: Vec<_> = ids.chunks(200).collect();
    let results = leg.batch(&chunks, |child, ids| {
        pubmed::fetch_articles(child, ids, cx.ncbi_email.as_ref())
    });
    let mut records = Vec::new();
    let mut failures = Vec::new();
    for (ids, result) in chunks.into_iter().zip(results) {
        match result {
            Ok((rows, metadata)) => {
                for row in rows {
                    if row.article.pmid.is_some_and(|id| ids.contains(&id))
                        && !records
                            .iter()
                            .any(|(r, _): &Record| r.article.pmid == row.article.pmid)
                    {
                        records.push((row, metadata.fetched));
                    }
                }
            }
            Err(error) => failures.extend(ids.iter().map(|id| (*id, error))),
        }
    }
    (records, failures)
}
fn agrees(input: &ArticleInput, card: &Article) -> bool {
    match input {
        ArticleInput::Pmid(id) => Some(*id) == card.pmid,
        ArticleInput::Pmcid(id) => card.pmcid.as_ref() == Some(id),
        ArticleInput::Doi(id) => card.doi.as_ref() == Some(id),
        ArticleInput::EuropePmc(id) => card.europepmc_id.as_ref() == Some(id),
    }
}
fn item(
    input: &ArticleInput,
    status: Status,
    answer: Option<ArticleResolution>,
) -> ArticleCardItem {
    ArticleCardItem {
        input: input.clone(),
        status,
        answer,
        sections: None,
        more: Box::default(),
    }
}
fn append(
    input: &ArticleInput,
    ids: &[Pmid],
    records: &[Record],
    failures: &[Failure],
    stale: bool,
    items: &mut Vec<ArticleCardItem>,
) {
    let mut found = false;
    for id in ids {
        if let Some((card, fetched)) = records
            .iter()
            .find(|(card, _)| card.article.pmid == Some(*id) && agrees(input, &card.article))
        {
            found = true;
            items.push(item(
                input,
                if stale || card.refusals == biomcp_adapters::Refusal::Refused {
                    crate::status::item_status(true, false, true)
                } else {
                    crate::status::item_status(true, false, false)
                },
                Some(ArticleResolution::Resolved {
                    card: card.article.clone().card(),
                    r#match: ArticleMatch { kind: input.kind() },
                    fetched: fetched.and_then(|value| u64::try_from(value).ok()),
                }),
            ));
        }
    }
    let failed = ids
        .iter()
        .any(|id| failures.iter().any(|(failed, _)| id == failed));
    if failed {
        items.push(item(
            input,
            crate::status::item_status(false, true, false),
            None,
        ));
    } else if !found {
        items.push(item(
            input,
            crate::status::item_status(false, false, false),
            Some(ArticleResolution::NotFound),
        ));
    }
}

fn sources(
    query: &ArticleGet,
    source: biomcp_types::SourceOutcome,
) -> Vec<biomcp_types::SourceOutcome> {
    if query
        .input()
        .iter()
        .any(|i| !matches!(i, ArticleInput::EuropePmc(_)))
    {
        vec![source]
    } else {
        Vec::new()
    }
}
