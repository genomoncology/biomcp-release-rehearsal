//! Exact manifest membership precedes the shared original-byte download.
use crate::{Ctx, Outcome, status};
use biodata::TrialDocument;
use biomcp_adapters::ctgov;
use biomcp_types::ReplyBody;
use biomcp_types::{ReplyData, Section, Source, TrialDocumentGet, TrialDocumentResult};
/// A typed source result before the binary writes a caller-owned file.
#[derive(Debug)]
pub struct Download {
    /// Result and attributed request evidence.
    pub out: Outcome,
    /// Original PDF and checked manifest metadata, never cached as a saved file.
    pub file: Option<(TrialDocument, Vec<u8>)>,
}
/// A disallowed surface or destination returns the source URL without downloading.
pub fn download(cx: &Ctx<'_>, query: &TrialDocumentGet, allow: bool) -> Download {
    let command = cx.fetch.command_deadline(cx.budget);
    let leg = cx
        .fetch
        .leg(&ctgov::SPEC, command.deadline())
        .fresh(cx.fresh);
    let mut out = crate::status::ready(
        (TrialDocumentResult {
            file: Section::Empty {
                source: Source::ClinicalTrialsGov,
                record: None,
            },
        })
        .with_query(query.clone()),
    );
    let result = ctgov::detail::get(&leg, query.input).and_then(|(detail, _)| {
        let incomplete_manifest = detail.document_refusals == biomcp_adapters::Refusal::Refused;
        out.include_refusals(detail.refusals);
        crate::status::item(&mut out, true, false, false);
        out.problems.extend(detail.problems);
        let Some(document) = detail
            .documents
            .into_vec()
            .into_iter()
            .find(|d| d.filename == query.file)
        else {
            missing_document(query, &mut out, incomplete_manifest);
            return Ok(None);
        };
        if !allow {
            out.body = ReplyData::Usage(biomcp_types::UsageError::ArticleSaveRefused(document.url));
            return Ok(None);
        }
        ctgov::document::download(&leg, &document).map(|bytes| Some((document, bytes)))
    });
    let error = result.as_ref().err().copied();
    if let Some(error) = error {
        crate::status::item(&mut out, false, true, false);
        out.local = status::local(error);
        out.body = (TrialDocumentResult {
            file: Section::unavailable(
                Source::ClinicalTrialsGov,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            ),
        })
        .with_query(query.clone());
    }
    let remaining = leg.remaining();
    out.sources.push(status::source(
        Source::ClinicalTrialsGov,
        leg.finish(),
        error,
    ));
    command.finish(remaining);
    Download {
        out,
        file: result.ok().flatten(),
    }
}

fn missing_document(query: &TrialDocumentGet, out: &mut Outcome, incomplete_manifest: bool) {
    if incomplete_manifest {
        crate::status::item(out, false, true, false);
        out.body = (TrialDocumentResult {
            file: Section::unavailable(Source::ClinicalTrialsGov, biomcp_types::Reason::BadReply),
        })
        .with_query(query.clone());
    } else {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
    }
}
