//! A rejected registry cursor offers an explicit uncached first-page request.
use crate::{Action, Ctx, Outcome, next::Step, source_snapshot};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{ReplyData, Request, Source, StepKind, TrialSearch, UsageError};

const PREFIX: &str = "biomcp:ctgov-page:v1:";
const NAMESPACE: &str = "ctgov-continuation/1";

/// The existing capability store authorizes callers and selectors before decoding native cursors.
pub(super) fn restore(
    cx: &Ctx<'_>,
    query: &TrialSearch,
    deadline: Deadline<'_>,
) -> Result<Option<TrialSearch>, RequestError> {
    let Some(token) = query.next_page() else {
        return Ok(Some(query.clone()));
    };
    let Some(id) = token.as_str().strip_prefix(PREFIX) else {
        return Ok(None);
    };
    let key = source_snapshot::key(NAMESPACE, &query.first_page())?;
    let saved = source_snapshot::read::<TrialSearch>(cx, &key, id, deadline)?;
    Ok(saved.filter(|saved| saved.offset() == query.offset() && saved.next_page().is_some()))
}

/// Retain the actual sent query and consumed position, never a caller-supplied native expression.
pub(super) fn retain(
    cx: &Ctx<'_>,
    out: &mut Outcome,
    deadline: Deadline<'_>,
) -> Result<(), RequestError> {
    let sources: Vec<_> = out
        .sources
        .iter()
        .filter(|source| source.state != biomcp_types::SourceState::Skipped)
        .cloned()
        .collect();
    for mut step in std::mem::take(&mut out.next) {
        if step.0 == StepKind::Page
            && let Action::Run {
                request: Request::SearchTrial(query),
                ..
            } = &mut step.1
        {
            let key = source_snapshot::key(NAMESPACE, &query.first_page())?;
            let saved = source_snapshot::write(cx, &key, query, &sources, deadline);
            let Some(id) = crate::publish::keep(saved, out)? else {
                continue;
            };
            let token = biodata::Text::new(&format!("{PREFIX}{id}"))
                .map_err(|_| biomcp_fetch::CacheError::Encoding)?;
            *query = query.at_page(token, query.offset());
        }
        out.next.push(step);
    }
    Ok(())
}

pub(super) fn unavailable(query: &TrialSearch, out: &mut Outcome) {
    out.body = ReplyData::Usage(UsageError::TrialContinuationUnavailable);
    restart(query, out);
}

pub(super) fn rejected(query: &TrialSearch, out: &mut Outcome) {
    out.body = ReplyData::Usage(UsageError::TrialContinuationRejected);
    restart(query, out);
}
fn restart(query: &TrialSearch, out: &mut Outcome) {
    out.next.push(Step::run(
        StepKind::Configure,
        &[Source::ClinicalTrialsGov],
        Request::SearchTrial(query.first_page()),
        true,
    ));
}
