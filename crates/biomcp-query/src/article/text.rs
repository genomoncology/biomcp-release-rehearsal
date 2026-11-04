//! Bounded body selection under one command budget, including PMID resolution.
pub use super::save::Download;
use crate::{Ctx, Outcome, status};
use biodata::{Pmcid, Text};
use biomcp_types::ReplyBody;

use biomcp_adapters::{europepmc, jats, pmc_oa, pubmed};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::ArticleInput;
use biomcp_types::{
    ArticleText, ArticleTextResult, PageInfo, Reason, ReplyData, Request, Section, Source,
    StepKind, TextContent, TextHit,
};
/// Read a bounded part or find page through the same transport and command budget.
pub(crate) fn read(cx: &Ctx<'_>, query: &ArticleText) -> Outcome {
    download(cx, query, false).out
}
/// Obtain the whole document for an explicit binary-owned save.
pub fn download(cx: &Ctx<'_>, query: &ArticleText, allow: bool) -> super::save::Download {
    let mut save = super::save::State::new(allow);
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    let mut out = crate::status::ready(
        (ArticleTextResult {
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
        if let ReplyData::ArticleTextResult {
            body: Some(body), ..
        } = &mut out.body
        {
            let default_source = if matches!(query.input(), ArticleInput::EuropePmc(_)) {
                Source::EuropePmc
            } else {
                Source::PmcOa
            };
            let source = match error {
                RequestError::Source(e) => e.source,
                _ => default_source,
            };
            let reason = crate::status::reason(error).unwrap_or(Reason::Unavailable);
            body.content = Section::unavailable(source, reason);
        }
        out.next.push(crate::next::Step::run(
            StepKind::Fresh,
            &super::next::text_sources(query.input()),
            Request::ArticleText(query.clone()),
            true,
        ));
    }
    super::next::text(query, &mut out);
    super::save::Download {
        out,
        file: save.file,
    }
}
pub(super) fn resolve<'a>(
    cx: &Ctx<'a>,
    input: &ArticleInput,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
) -> Result<Option<Pmcid>, RequestError> {
    match input {
        ArticleInput::Pmcid(id) => Ok(Some(*id)),
        ArticleInput::Pmid(id) => {
            let leg = cx.fetch.leg(&pubmed::SPEC, *deadline).fresh(cx.fresh);
            let started = leg.remaining();
            let result = pubmed::summaries(&leg, &[*id], cx.ncbi_email.as_ref());
            *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
            let source =
                status::source(Source::PubMed, leg.finish(), result.as_ref().err().copied());
            if source.state == biomcp_types::SourceState::Stale {
                crate::status::item(out, true, false, true);
            }
            out.sources.push(source);
            let mapped = result?;
            out.include_refusals(mapped.refusals);
            if out.dropped {
                crate::status::item(out, true, false, true);
            }
            status::append(&mut out.problems, mapped.problems);
            let header = mapped
                .value
                .into_iter()
                .find(|h| h.pmid == Some(*id))
                .ok_or(biomcp_fetch::SourceError {
                    source: Source::PubMed,
                    reason: Reason::BadReply,
                    status: None,
                    tls_cause: None,
                    retry_after: None,
                })?;
            Ok(header.pmcid)
        }
        ArticleInput::Doi(_) | ArticleInput::EuropePmc(_) => Ok(None),
    }
}
fn execute<'a>(
    cx: &Ctx<'a>,
    query: &ArticleText,
    selection: &super::protein_change::Selection,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
    save: &mut super::save::State,
) -> Result<(), RequestError> {
    if let ArticleInput::EuropePmc(id) = query.input() {
        return preprint(cx, query, selection, id, deadline, out, save);
    }
    let Some(id) = resolve(cx, query.input(), deadline, out)? else {
        return Ok(());
    };
    let leg = cx.fetch.leg(&pmc_oa::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let pmid = match query.input() {
        ArticleInput::Pmid(id) => Some(*id),
        _ => None,
    };
    let result = pmc_oa::read(&leg, id, pmid).map(|mapped| {
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        mapped.value
    });
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let source = status::source(Source::PmcOa, leg.finish(), result.as_ref().err().copied());
    if source.state == biomcp_types::SourceState::Stale {
        crate::status::item(out, true, false, true);
    }
    out.sources.push(source);
    let content = match result? {
        None => Section::unavailable(Source::PmcOa, Reason::NotOpen),
        Some(document) => {
            out.dropped |= document.body.refusals == biomcp_adapters::Refusal::Refused;
            out.problems.extend(document.problems.clone());
            let Some(content) = select(
                query,
                selection,
                ArticleInput::Pmcid(id),
                &document,
                out,
                save,
            ) else {
                out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
                return Ok(());
            };
            Section::Data {
                source: Source::PmcOa,
                data: content,
            }
        }
    };
    out.body = (ArticleTextResult {
        pmcid: Some(id),
        content,
    })
    .with_query(query.clone());
    Ok(())
}
fn select(
    query: &ArticleText,
    selection: &super::protein_change::Selection,
    input: ArticleInput,
    document: &pmc_oa::Document,
    out: &mut Outcome,
    save: &mut super::save::State,
) -> Option<TextContent> {
    let mut content = TextContent {
        saved: None,
        url: None,
        forms: selection.forms.clone(),
        license: document.license,
        processing_version: document.version,
        manuscript: document.manuscript,
        scope: Text::new(if matches!(query.input(), ArticleInput::EuropePmc(_)) {
            "Europe PMC body sections and tables"
        } else {
            "PMC body sections and tables"
        })
        .ok()?,
        part: None,
        markdown: None,
        heading_path: None,
        hits: None,
    };
    if query.save().is_some() {
        content.url = Some(document.url.clone());
        save.text(query.input(), document)?;
    } else if let Some(number) = query.selected_part() {
        let part = document
            .body
            .parts
            .iter()
            .find(|p| p.meta.number == number)?;
        content.part = Some(number);
        content.heading_path = Some(part.heading_path.clone());
        content.markdown = biodata::Markdown::new(
            &part
                .rows
                .iter()
                .map(|r| r.text.as_str())
                .collect::<Vec<_>>()
                .join(part.separator),
        )
        .ok();
        if part.continuation {
            super::sections::fulltext::next_input(
                input,
                number.checked_add(1)?,
                &mut out.next,
                &mut out.problems,
            );
        }
    } else {
        content.hits = Some(find(query, selection, document, out)?);
    }
    Some(content)
}

fn find(
    query: &ArticleText,
    selection: &super::protein_change::Selection,
    document: &pmc_oa::Document,
    out: &mut Outcome,
) -> Option<Box<[TextHit]>> {
    let needles: Vec<_> = selection.needles.iter().map(|v| v.to_lowercase()).collect();
    let matching: Vec<_> = document
        .body
        .rows
        .iter()
        .map(|r| {
            needles
                .iter()
                .any(|needle| r.plain.to_lowercase().contains(needle))
        })
        .collect();
    let hits = matches(document, &matching);
    let offset = usize::try_from(query.offset()).ok()?;
    if offset > hits.len() {
        return None;
    }
    let mut page = Vec::new();
    let mut size = 0;
    for (part, path, row) in hits.iter().skip(offset).take(25) {
        let bytes = jats::escaped_bytes(row) + serde_json::to_vec(path).ok()?.len() + 128;
        if !page.is_empty() && size + bytes > jats::TEXT_BYTES {
            break;
        }
        size += bytes;
        page.push(TextHit {
            part: *part,
            markdown: biodata::Markdown::new(row).ok()?,
            heading_path: path.clone(),
        });
    }
    let returned = u32::try_from(page.len()).ok()?;
    let total = u64::try_from(hits.len()).ok()?;
    let following = query.offset().checked_add(returned)?;
    if returned > 0 && u64::from(following) < total {
        out.next.push(crate::next::Step::run(
            StepKind::Page,
            &super::next::text_sources(query.input()),
            Request::ArticleText(query.page(following)),
            false,
        ));
    }
    out.page = Some(PageInfo {
        offset: query.offset(),
        returned,
        total,
        pool: None,
        truncated: None,
    });
    Some(page.into())
}

type Hit = (u32, Box<[Text]>, String);
fn matches(document: &pmc_oa::Document, matching: &[bool]) -> Vec<Hit> {
    let mut hits = Vec::new();
    for part in &document.body.parts {
        let matching_rows: Vec<_> = part
            .rows
            .iter()
            .filter(|r| matching.get(r.source_row).copied().unwrap_or(false))
            .collect();
        if matching_rows.is_empty() {
            continue;
        }
        if let Some(context) = &part.context {
            let data: Vec<_> = matching_rows
                .iter()
                .filter(|r| {
                    document
                        .body
                        .rows
                        .get(r.source_row)
                        .is_some_and(|r| !r.context)
                })
                .map(|r| r.text.as_str())
                .collect();
            let markdown = if data.is_empty() {
                context.clone()
            } else {
                format!("{context}\n{}", data.join("\n"))
            };
            hits.push((part.meta.number, part.heading_path.clone(), markdown));
        } else {
            hits.extend(
                matching_rows
                    .into_iter()
                    .map(|r| (part.meta.number, part.heading_path.clone(), r.text.clone())),
            );
        }
    }
    hits
}

fn preprint<'a>(
    cx: &Ctx<'a>,
    query: &ArticleText,
    selection: &super::protein_change::Selection,
    id: &biodata::EuropePmcId,
    deadline: &mut Deadline<'a>,
    out: &mut Outcome,
    save: &mut super::save::State,
) -> Result<(), RequestError> {
    let leg = cx.fetch.leg(&europepmc::SPEC, *deadline).fresh(cx.fresh);
    let started = leg.remaining();
    let result = europepmc::read(&leg, id);
    *deadline = deadline.after_stage(started.saturating_sub(leg.remaining()));
    let source = status::source(
        Source::EuropePmc,
        leg.finish(),
        result.as_ref().err().copied(),
    );
    if source.state == biomcp_types::SourceState::Stale {
        crate::status::item(out, true, false, true);
    }
    out.sources.push(source);
    let mapped = result?;
    out.include_refusals(mapped.refusals);
    out.problems.extend(mapped.problems);
    let Some(document) = mapped.value else {
        out.body = (ArticleTextResult {
            pmcid: None,
            content: Section::unavailable(Source::EuropePmc, Reason::NotOpen),
        })
        .with_query(query.clone());
        return Ok(());
    };
    let Some(content) = select(
        query,
        selection,
        query.input().clone(),
        &document,
        out,
        save,
    ) else {
        out.body = ReplyData::Usage(biomcp_types::UsageError::TextPositionError);
        return Ok(());
    };
    out.body = (ArticleTextResult {
        pmcid: None,
        content: Section::Data {
            source: Source::EuropePmc,
            data: content,
        },
    })
    .with_query(query.clone());
    Ok(())
}
