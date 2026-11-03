//! Cheap publisher-link sections never download a file.
use crate::{Ctx, Outcome, status};
use biodata::Text;
use biomcp_adapters::semantic_scholar;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::ArticleInput;
use biomcp_types::{
    ArticlePdf, ArticleResolution, OpenAccessStatus, PdfLink, Reason, ReplyData, Request, Section,
    Source, StepKind,
};

pub(super) fn run<'a>(cx: &Ctx<'a>, deadline: &mut Deadline<'a>, out: &mut Outcome) {
    let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &out.body
    else {
        return;
    };
    let inputs: Vec<_> = cards
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let Some(ArticleResolution::Resolved { card, .. }) = &item.answer else {
                return None;
            };
            card.pmid
                .map(ArticleInput::Pmid)
                .or_else(|| card.doi.clone().map(ArticleInput::Doi))
                .map(|input| (index, input))
        })
        .collect();
    for (index, input) in inputs {
        let section = match load(cx, &input, deadline, out) {
            Ok(section) => section,
            Err(error) => {
                out.local = out.local.or(status::local(error));
                Section::unavailable(
                    Source::SemanticScholar,
                    crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
                )
            }
        };
        if matches!(section, Section::Data { .. }) && out.next.len() < 5 {
            let name = ArticlePdf::filename(&input);
            if let Some(save) = crate::next::checked(
                Text::new(&name),
                biomcp_types::Source::SemanticScholar,
                &mut out.problems,
            ) {
                out.next.push(crate::next::Step::run(
                    StepKind::Section,
                    &[biomcp_types::Source::SemanticScholar],
                    Request::ArticlePdf(ArticlePdf {
                        input,
                        save,
                        replace: false,
                    }),
                    false,
                ));
            }
        }
        let ReplyData::ArticleCards {
            body: Some(cards), ..
        } = &mut out.body
        else {
            return;
        };
        if let Some(item) = cards.items.get_mut(index) {
            if matches!(section, Section::Unavailable { reason, .. } if reason != Reason::NotOpen) {
                item.status = crate::status::item_status(true, false, true);
            }
            if let Some(sections) = &mut item.sections {
                sections.pdf = Some(section);
            }
        }
    }
    if let ReplyData::ArticleCards {
        body: Some(cards), ..
    } = &out.body
    {
        let state = status::status(cards.items.iter().map(|i| i.status));
        status::get(out, [state].into_iter());
    }
}
pub(in crate::article) fn load<'a>(
    cx: &Ctx<'a>,
    input: &ArticleInput,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Section<PdfLink>, RequestError> {
    let leg = cx
        .fetch
        .leg(&semantic_scholar::SPEC, *deadline)
        .fresh(cx.fresh);
    let started = leg.remaining();
    let result = semantic_scholar::pdf(&leg, input).and_then(|record| {
        if let Some(link) = &record.open_access_pdf {
            biomcp_adapters::pmc_oa::permitted(&leg, Source::SemanticScholar, link.license())?;
        }
        Ok(record)
    });
    *deadline = super::super::consume(*deadline, started, leg.remaining());
    let source = status::source(
        Source::SemanticScholar,
        leg.finish(),
        result.as_ref().err().copied(),
    );
    if let Some(existing) = out
        .sources
        .iter_mut()
        .find(|s| s.source == Source::SemanticScholar)
    {
        status::retain_source_state(existing, &source);
    } else {
        out.sources.push(source);
    }
    let result = result?;
    out.problems.extend(result.problems);
    let Some(link) = result
        .open_access_pdf
        .filter(|link| !link.url.trim().is_empty() && link.status != OpenAccessStatus::Closed)
    else {
        return Ok(Section::unavailable(
            Source::SemanticScholar,
            Reason::NotOpen,
        ));
    };
    let data = PdfLink {
        url: Text::new(&link.url).map_err(|_| pdf_error(Reason::BadReply))?,
        status: link.status,
        license: link.license(),
    };
    Ok(Section::Data {
        source: Source::SemanticScholar,
        data,
    })
}
pub(in crate::article) fn pdf_error(reason: Reason) -> RequestError {
    SourceError {
        source: Source::SemanticScholar,
        reason,
        status: None,
        retry_after: None,
    }
    .into()
}
