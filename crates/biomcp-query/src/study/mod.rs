//! Literal study catalog reads with checked source admission and fixed continuation state.
use biomcp_types::ReplyBody;
mod page;
pub(super) use crate::status::empty;
use crate::{Action, Bundle, Ctx, Outcome, status};
use biomcp_adapters::cbioportal::{SPEC, studies};
use biomcp_fetch::{Leg, RequestError};
use biomcp_types::{Source, SourceState, Status, StepKind, StudySearch};
fn finish(out: &mut Outcome, leg: Leg<'_>, error: Option<RequestError>) {
    out.local = error.and_then(status::local);
    out.sources
        .push(status::source(Source::Cbioportal, leg.finish(), error));
    if error.is_some() {
        crate::status::item(out, false, true, false)
    } else if out.status() == Status::Complete
        && (out.dropped || out.sources.iter().any(|s| s.state == SourceState::Stale))
    {
        crate::status::item(out, true, false, true)
    }
    if out
        .sources
        .iter()
        .any(|s| matches!(s.state, SourceState::Stale | SourceState::Syncing))
    {
        out.next.push(crate::next::Step::new(
            StepKind::Sync,
            &[biomcp_types::Source::Cbioportal],
            Action::Sync {
                bundle: Bundle::CBIOPORTAL_STUDIES,
            },
        ));
    }
}
/// Search only the last accepted metadata catalog; no implicit refresh.
pub(crate) fn search(cx: &Ctx<'_>, q: &StudySearch) -> Outcome {
    let mut out = empty(biomcp_types::StudyPage::into_reply(q.clone(), None));
    let command = cx.fetch.command_deadline(cx.budget);
    let deadline = command.deadline();
    let leg = cx.fetch.leg(&SPEC, deadline);
    let result = studies::search(&leg, q).and_then(|(rows, info)| {
        out.include_refusals(rows.refusals);
        out.problems.extend(rows.problems);
        page::search(cx, q, rows.value, info, deadline, &mut out)
    });
    let error = result.err();
    finish(&mut out, leg, error);
    command.finish(deadline.remaining());
    out
}
mod get;
pub(crate) use get::get;
