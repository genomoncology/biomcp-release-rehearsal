//! Native SPL identities and unchanged explicit label headings.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::{openfda_label as source, received_date};
use biomcp_fetch::RequestError;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugGet, DrugLabel, DrugPage, DrugSearch, DrugSection, DrugSource, PageInfo, Request, Section,
    Source, StepKind,
};
mod names;
mod search_page;
const SOURCE: Source = Source::OpenfdaLabel;
pub(super) fn search(cx: &Ctx<'_>, query: &DrugSearch) -> Outcome {
    let mut out = super::empty(
        (DrugPage {
            items: Box::default(),
            total: None,
        })
        .with_query(query.clone()),
    );
    let command = cx.fetch.command_deadline(cx.budget);
    let mut deadline = command.deadline();
    let mut expanded = if query.expand_drug_names()
        && (query.name_snapshot().is_some() || !cx.fetch.disabled(SOURCE))
    {
        match names::prepare(cx, query, &mut deadline) {
            Ok(Some(saved)) => Some(saved),
            Ok(None) => {
                names::missing(query, &mut out);
                return out;
            }
            Err(error) => {
                out.local = Some(error);
                return out;
            }
        }
    } else {
        None
    };
    let leg = cx.fetch.leg(&source::SPEC, deadline).fresh(cx.fresh);
    let discovery = if let Some(saved) = expanded.as_mut() {
        names::search(&leg, query, saved)
    } else {
        match (query.indication(), query.query()) {
            (Some(indication), name) => {
                source::search_indication(&leg, indication, name, query.limit(), query.offset())
            }
            (None, Some(name)) => source::search(&leg, name, query.limit(), query.offset()),
            (None, None) => return super::unsupported(query.source(), out),
        }
    };
    let result = discovery.page.and_then(|page| {
        out.problems.extend(page.problems);
        let items = search_page::headers(&page.window.rows, query)?;
        Ok(page.window.with_rows(items))
    });
    let error = result.as_ref().err().copied();
    out.local = error.and_then(status::local);
    let remaining = leg.remaining();
    out.sources
        .push(status::source(SOURCE, leg.finish(), error));
    let sent = names::retain(
        cx,
        query,
        expanded.as_ref(),
        deadline,
        result.is_ok(),
        &mut out,
    );
    disclosure(&mut out, discovery.free_text);
    if let Ok(window) = result {
        let total = window.total().unwrap_or(0);
        search_page::apply(cx, &sent, window, total, &mut out);
    }
    status::search(&mut out);
    names::finish(query, expanded, &mut out);
    command.finish(remaining);
    out
}
fn disclosure(out: &mut Outcome, selector: Option<biodata::Text>) {
    if let Some(selector) = selector
        && let Some(source) = out.sources.first_mut()
    {
        source.term = Some(selector);
        source
            .notices
            .push(biomcp_types::SourceNotice::RequestGuidance {
            data_date: None,
            message:
                "Exact and word name searches found no matching product; searched label free text."
                    .into(),
            retry_after_seconds: None,
        });
    }
}
pub(super) fn get(cx: &Ctx<'_>, query: &DrugGet) -> Outcome {
    let label = matches!(query.sections(), [DrugSection::Label]);
    let mut out = super::empty(if label {
        biomcp_types::DrugLabel::into_reply(query.clone(), None)
    } else {
        biomcp_types::DrugCard::into_reply(query.clone(), None)
    });
    if !query.supports_sections() {
        return super::unsupported(DrugSource::OpenfdaLabel, out);
    }
    let id = match query.native_input() {
        Ok(id) => id,
        Err(_) => return super::rejected_input(query.source(), "input", out),
    };
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&source::SPEC, deadline).fresh(cx.fresh);
    let result = source::get(&leg, &id);
    let error = result.as_ref().err().copied();
    out.local = error.and_then(status::local);
    let remaining = leg.remaining();
    out.sources
        .push(status::source(SOURCE, leg.finish(), error));
    if let Ok(page) = result {
        out.problems.extend(page.problems);
        out.include_refusals(page.window.refusals());
        crate::status::item(&mut out, true, false, false);
        if let Some(source) = out.sources.first_mut() {
            source.count = Some(page.window.total().unwrap_or(0));
            source.retrieved_count = Some(page.window.rows.len() as u32);
        }
        if let Some(record) = page.window.rows.first() {
            if let Err(error) = select(cx, query, record, page.metadata.fetched, &mut out) {
                crate::status::item(&mut out, false, true, false);
                out.local = status::local(error);
            }
        } else {
            crate::status::item(&mut out, false, false, false);
        }
    }
    command.finish(remaining);
    out
}
fn select(
    cx: &Ctx<'_>,
    query: &DrugGet,
    record: &source::Record,
    fetched: Option<i64>,
    out: &mut Outcome,
) -> Result<(), RequestError> {
    if query.sections().is_empty() {
        let card = record.card(received_date(fetched))?;
        for heading in source::HEADINGS
            .into_iter()
            .filter(|h| record.has_heading(h))
        {
            if let Some(heading) = crate::next::checked(
                heading.parse(),
                biomcp_types::Source::OpenfdaLabel,
                &mut out.problems,
            ) && let Some(request) = crate::next::checked(
                query.for_label(heading),
                biomcp_types::Source::OpenfdaLabel,
                &mut out.problems,
            ) {
                out.next.push(crate::next::Step::run(
                    StepKind::Section,
                    &[biomcp_types::Source::OpenfdaLabel],
                    Request::GetDrug(request),
                    cx.fresh,
                ));
            }
        }
        out.body = card.with_query(query.clone());
        return Ok(());
    }
    let Some(heading) = query.label_section().map(|h| h.as_str()) else {
        out.body = section_names(query, record)?;
        return Ok(());
    };
    if !source::HEADINGS.contains(&heading) {
        out.body = biomcp_types::ReplyData::Usage(invalid_heading(record)?);
        return Ok(());
    }
    let offset = query.section_offset().unwrap_or(0);
    let label = match record.label(heading, offset, query.section_limit().unwrap_or(1))? {
        Some(mut data) => {
            data.retrieved = received_date(fetched);
            let returned = data.fragments.len() as u32;
            let total = data.total.unwrap_or(0);
            out.page = Some(page(offset, returned, total));
            if let Some(next) = offset
                .checked_add(returned)
                .filter(|n| returned > 0 && u64::from(*n) < total)
                && let Some(request) = crate::next::checked(
                    query.at_section_offset(next),
                    biomcp_types::Source::OpenfdaLabel,
                    &mut out.problems,
                )
            {
                out.next.push(crate::next::Step::run(
                    StepKind::Page,
                    &[biomcp_types::Source::OpenfdaLabel],
                    Request::GetDrug(request),
                    cx.fresh,
                ));
            }
            Section::Data {
                source: SOURCE,
                data,
            }
        }
        None => Section::Empty {
            source: SOURCE,
            record: None,
        },
    };
    out.body = (DrugLabel {
        identity: record.header()?,
        label,
        sections: Box::default(),
        hint: None,
    })
    .with_query(query.clone());
    Ok(())
}

fn page(offset: u32, returned: u32, total: u64) -> PageInfo {
    PageInfo {
        offset,
        returned,
        total,
        pool: None,
        truncated: None,
    }
}

/// A bare label read: list the section names and say how to read one.
fn section_names(
    query: &DrugGet,
    record: &source::Record,
) -> Result<biomcp_types::ReplyData, RequestError> {
    Ok(DrugLabel {
        identity: record.header()?,
        label: Section::Empty {
            source: SOURCE,
            record: None,
        },
        sections: headings(record)?,
        hint: Some(
            "pass one of sections as label_section (--label-section on the command line) to read its text"
                .parse()
                .map_err(|_| biomcp_fetch::bad_reply(SOURCE, Some(200)))?,
        ),
    }
    .with_query(query.clone()))
}

fn headings(record: &source::Record) -> Result<Box<[biodata::Text]>, RequestError> {
    source::HEADINGS
        .iter()
        .filter(|h| record.has_heading(h))
        .map(|h| h.parse())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| biomcp_fetch::bad_reply(SOURCE, Some(200)))
        .map(Into::into)
}

fn invalid_heading(record: &source::Record) -> Result<biomcp_types::UsageError, RequestError> {
    Ok(biomcp_types::UsageError::SourceSelector {
        source: SOURCE,
        message: biomcp_types::ErrorMessage::LabelSectionMissing,
        option: "--label-section".into(),
        valid_values: headings(record)?,
        next: vec![biomcp_types::Step {
            kind: StepKind::Configure,
            args: ["help", "get", "drug"].map(Into::into).into(),
        }],
    })
}
