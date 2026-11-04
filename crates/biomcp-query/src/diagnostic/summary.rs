//! Source-owned FDA device search and document access.
use super::{empty, next};
use crate::{Ctx, Outcome, status};
use biomcp_adapters::{fda_pma, pdf};
use biomcp_types::{
    DiagnosticGet, DiagnosticSummary, PageInfo, PdfTextState, ReplyBody, Request, StepKind,
};
pub(super) fn get(cx: &Ctx<'_>, q: &DiagnosticGet) -> Outcome {
    let mut out = empty(DiagnosticSummary::into_reply(q.clone(), None));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&fda_pma::DOCUMENTS, deadline).fresh(cx.fresh);
    // FDA's legacy archive uses pdf/ (P010013, P970043); later folders are unpadded (P020014 -> pdf2/).
    // See the Summary links on those FDA PMA records; supplements have independent documents.
    let id = q.id().as_str();
    let year = id.get(1..3).unwrap_or_default();
    let directory = if ("02"..="69").contains(&year) {
        format!("pdf{}", year.trim_start_matches('0'))
    } else {
        "pdf".into()
    };
    let url = format!("https://www.accessdata.fda.gov/cdrh_docs/{directory}/{id}B.pdf").parse();
    let result = url
        .map_err(|_| biomcp_fetch::bad_reply(q.source().source(), None))
        .and_then(|url| pdf::read(&leg, &fda_pma::DOCUMENTS, &url, q.section_offset()));
    let error = match result {
        Ok(document) => {
            let returned = u32::from(document.text.is_some());
            let unavailable = document.state != PdfTextState::Text;
            status::item(&mut out, true, false, unavailable);
            out.page = Some(PageInfo {
                offset: q.section_offset(),
                returned,
                total: document.total.into(),
                pool: None,
                truncated: None,
            });
            if returned > 0 && q.section_offset() + returned < document.total {
                next::push(
                    &mut out,
                    StepKind::Page,
                    &[q.source().source()],
                    Request::GetDiagnostic(q.at_section_offset(q.section_offset() + returned)),
                    cx.fresh,
                );
            }
            out.body = DiagnosticSummary { document }.with_query(q.clone());
            None
        }
        Err(error) => Some(error),
    };
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(q.source().source(), leg.finish(), error));
    if error.is_some() {
        status::item(&mut out, false, true, false);
    }
    command.finish(deadline.remaining());
    out
}
