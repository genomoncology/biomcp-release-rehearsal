//! Explicit MAUDE native cards and whole device/narrative pages.
use crate::{Ctx, Outcome, status};
use biodata::Text;
use biomcp_adapters::maude;
use biomcp_types::ReplyBody;
use biomcp_types::{AdverseEventGet, AdverseEventSection, ReplyData, Request, Source, StepKind};
mod projection;
mod record;
mod rows;
/// Read exactly one numeric MAUDE key and project only its requested section.
pub(crate) fn get(cx: &Ctx<'_>, query: &AdverseEventGet) -> Outcome {
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let mut out = crate::status::empty(empty(query));
    match record::load(cx, query, deadline, &mut out) {
        Ok(Some((report, positions, snapshot))) => {
            let bound = snapshot
                .clone()
                .map_or_else(|| query.clone(), |t| query.with_snapshot(t));
            match projection::project(query, report, snapshot, positions) {
                Ok((body, total, returned, offset)) => {
                    out.body = body;
                    navigation(&bound, total, returned, offset, &mut out);
                }
                Err(_) => too_large(query, &mut out),
            }
        }
        Ok(None) => {}
        Err(error) => out.local = status::local(error),
    }
    command.finish(deadline.remaining());
    out
}
fn empty(query: &AdverseEventGet) -> ReplyData {
    match query.section() {
        Some(AdverseEventSection::Devices) => {
            biomcp_types::AdverseEventDevices::into_reply(query.clone(), None)
        }
        Some(AdverseEventSection::Narrative) => {
            biomcp_types::AdverseEventNarrative::into_reply(query.clone(), None)
        }
        _ => biomcp_types::AdverseEventMaudeCard::into_reply(query.clone(), None),
    }
}
fn navigation(query: &AdverseEventGet, total: u64, returned: u32, offset: u32, out: &mut Outcome) {
    if query.section().is_some() {
        out.page = Some(biomcp_types::PageInfo {
            offset: query.section_offset(),
            returned,
            total,
            pool: None,
            truncated: None,
        });
        if offset > query.section_offset() && u64::from(offset) < total {
            out.next.push(crate::next::Step::run(
                StepKind::Page,
                &[biomcp_types::Source::Maude],
                Request::GetAdverseEvent(query.at_section_offset(offset)),
                false,
            ));
        }
    } else {
        for section in [AdverseEventSection::Devices, AdverseEventSection::Narrative] {
            if let Some(next) = crate::next::checked(
                query.with_section(section, None),
                biomcp_types::Source::Maude,
                &mut out.problems,
            ) {
                out.next.push(crate::next::Step::run(
                    StepKind::Section,
                    &[biomcp_types::Source::Maude],
                    Request::GetAdverseEvent(next),
                    false,
                ));
            }
        }
    }
}
fn text(value: &str) -> Result<Text, biomcp_types::QueryInvalid> {
    Text::new(value).map_err(|_| biomcp_types::QueryInvalid::default())
}

fn too_large(query: &AdverseEventGet, out: &mut Outcome) {
    crate::status::item(out, false, true, false);
    out.sources = vec![biomcp_adapters::registry::outcome(
        Source::Maude,
        biomcp_types::SourceState::Failed,
        Some(biomcp_types::Reason::TooLarge),
    )];
    out.problems.push(biomcp_types::Problem {
        source: Source::Maude,
        record: Text::new(query.id().as_str()).ok(),
        field: "narrative".into(),
        code: biomcp_types::ProblemCode::ValueUnknown,
    });
}
