//! Native SPL identities and unchanged explicit label headings.
use crate::{Ctx, Outcome, status};
use biomcp_adapters::{openfda_label as source, received_date};
use biomcp_fetch::RequestError;
use biomcp_types::Paged;
use biomcp_types::ReplyBody;
use biomcp_types::{
    DrugGet, DrugLabel, DrugPage, DrugSearch, DrugSection, DrugSource, PageInfo, Request, Section,
    Source, StepKind,
};
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
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&source::SPEC, deadline).fresh(cx.fresh);
    let discovery = match (query.indication(), query.query()) {
        (Some(indication), name) => {
            source::search_indication(&leg, indication, name, query.limit(), query.offset())
        }
        (None, Some(name)) => source::search(&leg, name, query.limit(), query.offset()),
        (None, None) => return super::unsupported(query.source(), out),
    };
    let result = discovery.page.and_then(|page| {
        out.problems.extend(page.problems);
        let items = page
            .window
            .rows
            .iter()
            .map(|r| {
                let mut header = r.header()?;
                header.matched_name = query.query().and_then(|name| r.name_match(name));
                Ok(header)
            })
            .collect::<Result<Vec<_>, RequestError>>()?;
        Ok(page.window.with_rows(items))
    });
    let error = result.as_ref().err().copied();
    out.local = error.and_then(status::local);
    let remaining = leg.remaining();
    out.sources
        .push(status::source(SOURCE, leg.finish(), error));
    disclosure(&mut out, discovery.free_text);
    if let Ok(window) = result {
        let total = window.total().unwrap_or(0);
        let next_offset = window.next_offset();
        let items = window.rows;
        let returned = items.len() as u32;
        crate::status::item(&mut out, returned != 0, false, false);
        if let Some(source) = out.sources.first_mut() {
            source.count = Some(total);
            source.retrieved_count = Some(returned);
        }
        out.page = Some(page(query.offset(), returned, total));
        for item in &items {
            if let Some(get) = crate::next::checked(
                DrugGet::new(item.id.clone(), DrugSource::OpenfdaLabel),
                biomcp_types::Source::OpenfdaLabel,
                &mut out.problems,
            ) {
                out.next.push(crate::next::Step::run(
                    StepKind::Disambiguate,
                    &[biomcp_types::Source::OpenfdaLabel],
                    Request::GetDrug(get),
                    cx.fresh,
                ));
            }
        }
        if let Some(offset) = next_offset {
            out.next.push(crate::next::Step::run(
                StepKind::Page,
                &[biomcp_types::Source::OpenfdaLabel],
                Request::SearchDrug(query.at_offset(offset)),
                cx.fresh,
            ));
        }
        out.body = (DrugPage {
            items: items.into(),
            total: Some(total),
        })
        .with_query(query.clone());
    }
    status::search(&mut out);
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
    let heading = query.label_section().map_or("", |h| h.as_str());
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

fn invalid_heading(record: &source::Record) -> Result<biomcp_types::UsageError, RequestError> {
    Ok(biomcp_types::UsageError::SourceSelector {
        source: SOURCE,
        option: "--label-section".into(),
        valid_values: source::HEADINGS
            .iter()
            .filter(|h| record.has_heading(h))
            .map(|h| h.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| biomcp_fetch::bad_reply(SOURCE, Some(200)))?
            .into(),
        next: vec![biomcp_types::Step {
            kind: StepKind::Configure,
            args: ["help", "get", "drug"].map(Into::into).into(),
        }],
    })
}
