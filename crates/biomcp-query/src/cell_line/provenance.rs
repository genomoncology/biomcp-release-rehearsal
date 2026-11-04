use super::{Ctx, Outcome, Source, cellosaurus, status};
use biomcp_types::ReplyData;
/// Release metadata is an independent cacheable read, with its own failure state.
pub(super) fn attach(cx: &Ctx<'_>, deadline: biomcp_fetch::Deadline<'_>, out: &mut Outcome) {
    let leg = cx.fetch.leg(&cellosaurus::SPEC, deadline).fresh(cx.fresh);
    let release = cellosaurus::release(&leg);
    let error = release.as_ref().err().copied();
    let incoming = status::source(Source::Cellosaurus, leg.finish(), error);
    if let Some(source) = out.sources.first_mut() {
        status::retain_source_state(source, &incoming);
    }
    if error.is_some() {
        crate::status::item(out, true, false, true);
        out.local = error.and_then(status::local);
        out.problems.push(biomcp_types::Problem {
            source: Source::Cellosaurus,
            record: None,
            field: "release".into(),
            code: biomcp_types::ProblemCode::ValueUnknown,
        });
    }
    let release = release.ok().map(|mapped| {
        out.include_refusals(mapped.refusals);
        out.problems.extend(mapped.problems);
        mapped.value
    });
    match &mut out.body {
        ReplyData::CellLinePage {
            body: Some(page), ..
        } => page.release = release.clone(),
        ReplyData::CellLineFilteredPage {
            body: Some(page), ..
        } => page.0.release = release.clone(),
        ReplyData::CellLineCard {
            body: Some(card), ..
        } => card.release = release.clone(),
        _ => {}
    }
    if out
        .sources
        .iter()
        .any(|s| s.state == biomcp_types::SourceState::Stale)
    {
        crate::status::item(out, true, false, true);
    }
}
