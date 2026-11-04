//! Confined supplement download, local parsing and bounded row selection.
use biomcp_types::ReplyBody;
mod page;
use crate::{Ctx, Outcome, status};
use biodata::Text;
use biomcp_adapters::pmc_oa;
use biomcp_fetch::{Deadline, RequestError, SourceError};
use biomcp_types::ArticleInput;
use biomcp_types::{
    ArticleSupplement, ArticleSupplementResult, Reason, ReplyData, Request, Section, Source,
    StepKind, SupplementContent,
};
use sha2::{Digest, Sha256};
/// Read only a file named by both JATS and its owning bucket record.
pub(crate) fn read(cx: &Ctx<'_>, query: &ArticleSupplement) -> Outcome {
    download(cx, query, false).out
}
/// Return original source bytes for a binary-owned explicit save.
pub fn download(cx: &Ctx<'_>, query: &ArticleSupplement, allow: bool) -> super::save::Download {
    let mut save = super::save::State::new(allow);
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    let mut out = crate::status::ready(
        (ArticleSupplementResult {
            pmcid: None,
            content: Section::NotApplicable,
        })
        .with_query(query.clone()),
    );
    let Ok(selection) = super::protein_change::Selection::new(query.find(), query.variant()) else {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
        return super::save::Download { out, file: None };
    };
    let result = execute(cx, query, &selection, &mut deadline, &mut out, &mut save);
    command.finish(deadline.remaining());
    if let Err(error) = result {
        out.local = out.local.or(status::local(error));
        crate::status::item(&mut out, false, true, false);
        if let ReplyData::ArticleSupplementResult {
            body: Some(body), ..
        } = &mut out.body
        {
            body.content = Section::unavailable(
                Source::PmcOa,
                crate::status::reason(error).unwrap_or(biomcp_types::Reason::Unavailable),
            );
        }
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &super::next::text_sources(query.input()),
            Request::ArticleSupplement(query.clone()),
            true,
        ));
    }
    super::save::Download {
        out,
        file: save.file,
    }
}
fn failure(reason: Reason) -> RequestError {
    SourceError {
        source: Source::PmcOa,
        reason,
        status: None,
        tls_cause: None,
        retry_after: None,
    }
    .into()
}
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticleSupplement,
    selection: &super::protein_change::Selection,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
    save: &mut super::save::State,
) -> Result<(), RequestError> {
    let Some(id) = super::text::resolve(cx, query.input(), deadline, out)? else {
        return Ok(());
    };
    let leg = cx.fetch.leg(&pmc_oa::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let pmid = match query.input() {
        ArticleInput::Pmid(v) => Some(*v),
        _ => None,
    };
    let result = (|| {
        let mapped = pmc_oa::read(&leg, id, pmid)?;
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        let Some(document) = mapped.value else {
            return Ok(Section::Unavailable {
                source: Source::PmcOa,
                reason: Reason::NotOpen,
            });
        };
        out.dropped |= document.body.refusals == biomcp_adapters::Refusal::Refused;
        out.problems.extend(document.problems.clone());
        content(query, selection, &leg, &document, out, save)
    })();
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let source = status::source(Source::PmcOa, leg.finish(), result.as_ref().err().copied());
    if source.state == biomcp_types::SourceState::Stale {
        crate::status::item(out, true, false, true);
    }
    out.sources.push(source);
    let content = result?;
    if !matches!(
        out.body,
        ReplyData::Usage(biomcp_types::UsageError::TextPositionError)
    ) {
        out.body = (ArticleSupplementResult {
            pmcid: Some(id),
            content,
        })
        .with_query(query.clone());
    }
    Ok(())
}

fn empty(license: biodata::License, forms: Option<[String; 4]>) -> SupplementContent {
    SupplementContent {
        saved: None,
        search_message: None,
        license,
        sha256: None,
        url: None,
        reason: None,
        sheets: None,
        rows: None,
        forms,
    }
}

fn content(
    query: &ArticleSupplement,
    selection: &super::protein_change::Selection,
    leg: &biomcp_fetch::Leg<'_>,
    document: &pmc_oa::Document,
    out: &mut Outcome,
    save: &mut super::save::State,
) -> Result<Section<SupplementContent>, RequestError> {
    let Some(file) = document
        .supplements
        .iter()
        .find(|v| v.file.name == *query.file())
    else {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
        return Ok(Section::NotApplicable);
    };
    let mut content = empty(document.license, selection.forms.clone());
    if query.save().is_some() {
        content.url = Some(file.url.clone());
        if save.allow {
            save.file = Some(super::save::File {
                bytes: pmc_oa::download_bytes(leg, file)?,
                url: file.url.clone(),
                parts: None,
            });
        }
    } else if !file.file.kind.readable() {
        content.url = Some(file.url.clone());
        content.reason = Some(Reason::Unsupported);
    } else {
        let downloaded = pmc_oa::download(leg, file)?;
        content.sha256 = Some(
            Text::new(
                &Sha256::digest(&downloaded.bytes)
                    .iter()
                    .map(|v| format!("{v:02x}"))
                    .collect::<String>(),
            )
            .map_err(|_| failure(Reason::BadReply))?,
        );
        if !page::select(
            query,
            &downloaded.sheets,
            &mut content,
            out,
            &selection.needles,
            downloaded.deadline,
        )
        .map_err(failure)?
        {
            out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
            return Ok(Section::NotApplicable);
        }
    }
    Ok(Section::Data {
        source: Source::PmcOa,
        data: content,
    })
}
