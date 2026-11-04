//! Select one native application document, without guessing a submission or product.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::{drugsfda, pdf};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{DrugFdaSubmission, DrugGet, PageInfo, PdfTextState, Request, Source, StepKind};
pub(super) fn select(
    cx: &Ctx<'_>,
    query: &DrugGet,
    row: &drugsfda::Application,
    deadline: Deadline<'_>,
    out: &mut Outcome,
) -> Result<Option<Box<[DrugFdaSubmission]>>, RequestError> {
    let Some(id) = query.document_id() else {
        return Ok(None);
    };
    let mut selected = Vec::new();
    for mut submission in row.submissions(0, u32::MAX)? {
        let Some(docs) = submission.application_docs.take() else {
            continue;
        };
        for document in docs.into_vec().into_iter().filter(|d| &d.id == id) {
            let mut submission = submission.clone();
            submission.application_docs = Some(vec![document].into());
            selected.push(submission);
        }
    }
    if selected.len() != 1 {
        return Err(biomcp_fetch::bad_reply(Source::Drugsfda, Some(200)));
    }
    let document = selected
        .first_mut()
        .and_then(|s| s.application_docs.as_mut())
        .and_then(|docs| docs.first_mut())
        .ok_or_else(|| biomcp_fetch::bad_reply(Source::Drugsfda, Some(200)))?;
    if document
        .r#type
        .as_ref()
        .is_none_or(|kind| kind.as_str() != "Letter")
    {
        return Err(biomcp_fetch::bad_reply(Source::Drugsfda, Some(200)));
    }
    let url = document
        .url
        .as_ref()
        .ok_or_else(|| biomcp_fetch::bad_reply(Source::Drugsfda, Some(200)))?;
    let leg = cx.fetch.leg(&drugsfda::DOCUMENTS, deadline).fresh(cx.fresh);
    let offset = query.section_offset().unwrap_or(0);
    let result = pdf::read(&leg, &drugsfda::DOCUMENTS, url, offset);
    let error = result.as_ref().err().copied();
    let report = status::source(Source::Drugsfda, leg.finish(), error);
    // Both requests belong to Drugs@FDA; keep one source outcome and the document failure.
    if error.is_some()
        && let Some(source) = out
            .sources
            .iter_mut()
            .find(|source| source.source == Source::Drugsfda)
    {
        *source = report;
    }
    let text = result?;
    let returned = u32::from(text.text.is_some());
    status::item(out, true, false, text.state != PdfTextState::Text);
    out.page = Some(PageInfo {
        offset,
        returned,
        total: text.total.into(),
        pool: None,
        truncated: None,
    });
    if returned > 0
        && offset + returned < text.total
        && let Some(request) = crate::next::checked(
            query.at_section_offset(offset + returned),
            Source::Drugsfda,
            &mut out.problems,
        )
    {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &[Source::Drugsfda],
            Request::GetDrug(request),
            cx.fresh,
        ));
    }
    document.text = Some(text);
    Ok(Some(selected.into()))
}
