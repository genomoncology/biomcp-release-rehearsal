//! Explicit PDF download orchestration; the binary owns atomic file I/O.
use super::sections::pdf::{load, pdf_error};
use crate::{Ctx, Outcome, status};
use biomcp_types::ReplyBody;

use biomcp_adapters::{pdf, semantic_scholar};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{ArticlePdf, ArticlePdfResult, PdfLink, Reason, ReplyData, Section, Source};
/// Download only when the caller supplied a path and the interface permits local writes.
pub fn download(cx: &Ctx<'_>, query: &ArticlePdf, allow_save: bool) -> Download {
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    let mut out = crate::status::ready(
        (ArticlePdfResult {
            file: Section::NotApplicable,
        })
        .with_query(query.clone()),
    );
    let mut file = None;
    if let Err(error) = execute(cx, query, allow_save, &mut deadline, &mut out, &mut file) {
        out.local = out.local.or(status::local(error));
        crate::status::item(&mut out, false, true, false);
        out.body = (ArticlePdfResult {
            file: Section::unavailable(
                Source::SemanticScholar,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            ),
        })
        .with_query(query.clone());
    }
    command.finish(deadline.remaining());
    Download { out, file }
}
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticlePdf,
    allow_save: bool,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
    file: &mut Option<(PdfLink, Vec<u8>)>,
) -> Result<(), RequestError> {
    let section = load(cx, &query.input, deadline, out)?;
    let Section::Data { data: link, .. } = section else {
        crate::status::item(out, false, true, false);
        out.body = (ArticlePdfResult {
            file: Section::unavailable(Source::SemanticScholar, Reason::NotOpen),
        })
        .with_query(query.clone());
        return Ok(());
    };
    if !allow_save {
        out.body = ReplyData::Usage(biomcp_types::UsageError::PdfSaveRefused(link));
        return Ok(());
    }
    let leg = cx
        .fetch
        .leg(&semantic_scholar::PUBLISHER, *deadline)
        .fresh(true);
    let started = leg.remaining();
    let result = pdf::download(&leg, link.url.as_str()).map_err(|error| {
        if matches!(error, RequestError::Input(_)) {
            pdf_error(Reason::Rejected)
        } else {
            error
        }
    });
    *deadline = super::consume(*deadline, started, leg.remaining());
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
    let bytes = result?;
    if !bytes.starts_with(b"%PDF-") {
        return Err(pdf_error(Reason::NotPdf));
    }
    *file = Some((link, bytes));
    Ok(())
}

/// Bounded download before the binary decides how to install the file.
#[derive(Debug)]
pub struct Download {
    /// Typed source outcome.
    pub out: Outcome,
    /// Valid PDF bytes and source metadata; never cached or parsed.
    pub file: Option<(PdfLink, Vec<u8>)>,
}
