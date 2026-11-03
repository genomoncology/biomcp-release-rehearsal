use crate::{Ctx, Outcome, status};
use biodata::Pmcid;
use biomcp_adapters::pmc_oa;
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::ArticleInput;
use biomcp_types::{
    ArticleResolution, ArticleSupplement, ArticleText, FullText, Reason, Request, Section, Source,
    StepKind, Supplements,
};
type Identity = (Pmcid, Option<biodata::Pmid>);
type Content = Result<Option<(FullText, Supplements)>, Reason>;
/// Load one document for both sections, preserving a single source request sequence.
pub(super) fn run<'a>(
    cx: &Ctx<'a>,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
    fulltext: bool,
    supplements: bool,
) {
    let biomcp_types::ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &mut out.body
    else {
        return;
    };
    let leg = cx.fetch.leg(&pmc_oa::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let mut error = None;
    let mut loaded: Vec<(Identity, Content)> = Vec::new();
    for item in &mut cards.items {
        let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
            continue;
        };
        let Some(sections) = &mut item.sections else {
            continue;
        };
        if card.europepmc_id.is_some() {
            continue;
        }
        let Some(id) = card.pmcid else {
            continue;
        };
        let identity = (id, card.pmid);
        if !loaded.iter().any(|(found, _)| *found == identity) {
            let result = pmc_oa::read(&leg, id, card.pmid).map(|mapped| {
                out.dropped |= mapped.refusals == biomcp_adapters::Refusal::Refused;
                out.problems.extend(mapped.problems);
                mapped.value
            });
            if let Err(e) = result {
                error.get_or_insert(e);
                out.local = out.local.or(status::local(e));
            }
            if let Ok(Some(doc)) = &result {
                out.dropped |= doc.body.refusals == biomcp_adapters::Refusal::Refused;
                out.problems.extend(doc.problems.clone());
            }
            loaded.push((identity, content(result)));
        }
        let Some((_, data)) = loaded.iter().find(|(found, _)| *found == identity) else {
            continue;
        };
        if data.is_err() || matches!(data, Ok(None)) {
            item.status = crate::status::item_status(true, false, true);
        }
        if fulltext && sections.fulltext.is_none() {
            sections.fulltext = Some(section(data, |(text, _)| text.clone()));
            if matches!(data,Ok(Some((text,_))) if !text.parts.is_empty()) && out.next.len() < 5 {
                outline_next(ArticleInput::Pmcid(id), &mut out.next, &mut out.problems);
            }
        }
        if supplements && sections.supplements.is_none() {
            sections.supplements = Some(section(data, |(_, files)| files.clone()));
            if let Ok(Some((_, files))) = data {
                supplement_next(id, files, &mut out.next, &mut out.problems);
            }
        }
    }
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let source = status::source(Source::PmcOa, leg.finish(), error);
    if !loaded.is_empty() {
        if source.state == biomcp_types::SourceState::Stale {
            for item in &mut cards.items {
                if item.sections.as_ref().is_some_and(|s| {
                    matches!(s.fulltext, Some(Section::Data { .. }))
                        || matches!(s.supplements, Some(Section::Data { .. }))
                }) {
                    item.status = crate::status::item_status(true, false, true);
                }
            }
        }
        out.sources.push(source);
    }
    let state = status::status(cards.items.iter().map(|i| i.status));
    status::get(out, [state].into_iter());
}
fn content(result: Result<Option<pmc_oa::Document>, RequestError>) -> Content {
    result
        .map(|doc| {
            doc.map(|doc| {
                let text = outline(&doc);
                let files = Supplements {
                    license: doc.license,
                    files: doc.supplements.into_iter().map(|v| v.file).collect(),
                };
                (text, files)
            })
        })
        .map_err(|e| crate::status::reason(e).unwrap_or(biomcp_types::Reason::Unavailable))
}
fn section<T>(content: &Content, map: impl FnOnce(&(FullText, Supplements)) -> T) -> Section<T> {
    match content {
        Ok(Some(data)) => Section::Data {
            source: Source::PmcOa,
            data: map(data),
        },
        Ok(None) => Section::Unavailable {
            source: Source::PmcOa,
            reason: Reason::NotOpen,
        },
        Err(reason) => Section::unavailable(Source::PmcOa, *reason),
    }
}
fn supplement_next(
    id: Pmcid,
    files: &Supplements,
    out: &mut Vec<crate::next::Step>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    if let Some(file) = files.files.iter().find(|v| v.kind.readable())
        && out.len() < 5
        && let Some(query) = crate::next::checked(
            ArticleSupplement::new(ArticleInput::Pmcid(id), file.name.clone()),
            biomcp_types::Source::PmcOa,
            problems,
        )
    {
        out.push(crate::next::Step::run(
            StepKind::Page,
            &super::super::next::text_sources(query.input()),
            Request::ArticleSupplement(query),
            false,
        ));
    }
}
pub(crate) fn outline(document: &pmc_oa::Document) -> FullText {
    FullText {
        license: document.license,
        processing_version: document.version,
        manuscript: document.manuscript,
        parts: document.body.parts.iter().map(|p| p.meta.clone()).collect(),
        figures: document.body.figures.clone().into(),
    }
}
pub(crate) fn outline_next(
    input: ArticleInput,
    out: &mut Vec<crate::next::Step>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    // This fill-in term advertises literal finding without inventing a research question.
    if let Some(query) = crate::next::checked(
        ArticleText::finding(input, "<text>"),
        Source::PmcOa,
        problems,
    ) {
        out.push(crate::next::Step::run(
            StepKind::Section,
            &super::super::next::text_sources(query.input()),
            Request::ArticleText(query),
            false,
        ));
    }
}
pub(crate) fn next_input(
    input: ArticleInput,
    part: u32,
    out: &mut Vec<crate::next::Step>,
    problems: &mut Vec<biomcp_types::Problem>,
) {
    if let Some(query) = crate::next::checked(
        ArticleText::part(input, part),
        biomcp_types::Source::PmcOa,
        problems,
    ) {
        out.push(crate::next::Step::run(
            StepKind::Page,
            &super::super::next::text_sources(query.input()),
            Request::ArticleText(query),
            false,
        ));
    }
}
