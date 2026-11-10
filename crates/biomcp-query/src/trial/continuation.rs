//! A rejected registry cursor offers an explicit uncached first-page request.
use super::portable_continuation as portable;
use crate::{Action, Ctx, Outcome, next::Step, source_snapshot};
use biomcp_fetch::{Deadline, RequestError};
use biomcp_types::{
    ReplyData, Request, Source, SourceOutcome, SourceState, StepKind, TrialSearch, UsageError,
};

const PREFIX: &str = "biomcp:ctgov-page:v1:";
// Earlier rows could lose dependencies when reissued, so their scope cannot be established.
const NAMESPACE: &str = "ctgov-continuation/1-with-dependencies";

pub(super) struct Restored {
    pub(super) query: TrialSearch,
    dependencies: Dependencies,
}

type StoredCursor = (TrialSearch, Vec<Source>);

enum Dependencies {
    Stored(Vec<SourceOutcome>),
    Portable(Vec<Source>),
}

/// The existing capability store authorizes callers and selectors before decoding native cursors.
pub(super) fn restore(
    cx: &Ctx<'_>,
    query: &TrialSearch,
    deadline: Deadline<'_>,
) -> Result<Option<Restored>, RequestError> {
    let Some(token) = query.next_page() else {
        return Ok(Some(Restored {
            query: query.clone(),
            dependencies: Dependencies::Stored(Vec::new()),
        }));
    };
    if let Some(envelope) = token.as_str().strip_prefix(portable::PREFIX) {
        return Ok(
            portable::restore(cx, query, envelope)?.map(|(query, sources)| Restored {
                query,
                dependencies: Dependencies::Portable(sources),
            }),
        );
    }
    let Some(id) = token.as_str().strip_prefix(PREFIX) else {
        return Ok(None);
    };
    let key = source_snapshot::key(NAMESPACE, &query.first_page())?;
    let saved = source_snapshot::read_with_sources::<StoredCursor>(cx, &key, id, deadline)?;
    Ok(saved
        .filter(|((saved, ids), sources)| {
            saved.offset() == query.offset()
                && saved.next_page().is_some()
                && *ids == source_ids(sources)
        })
        .map(|((query, _), sources)| Restored {
            query,
            dependencies: Dependencies::Stored(sources),
        }))
}

/// Retain the actual sent query and consumed position, never a caller-supplied native expression.
pub(super) fn retain(
    cx: &Ctx<'_>,
    restored: Restored,
    out: &mut Outcome,
    deadline: Deadline<'_>,
) -> Result<(), RequestError> {
    let current = out
        .sources
        .iter()
        .filter(|source| source.state != SourceState::Skipped);
    let dependencies = match restored.dependencies {
        Dependencies::Stored(mut sources) => {
            for source in current {
                if !sources
                    .iter()
                    .any(|previous| previous.source == source.source)
                {
                    sources.push(source.clone());
                }
            }
            Dependencies::Stored(sources)
        }
        Dependencies::Portable(mut sources) => {
            for source in current {
                if !sources.contains(&source.source) {
                    sources.push(source.source);
                }
            }
            Dependencies::Portable(sources)
        }
    };
    for mut step in std::mem::take(&mut out.next) {
        if step.0 == StepKind::Page
            && let Action::Run {
                request: Request::SearchTrial(query),
                ..
            } = &mut step.1
        {
            let saved = retain_token(cx, query, &dependencies, deadline);
            let Some(id) = crate::publish::keep(saved, out)? else {
                continue;
            };
            *query = query.at_page(id, query.offset());
        }
        out.next.push(step);
    }
    Ok(())
}

fn retain_token(
    cx: &Ctx<'_>,
    query: &TrialSearch,
    dependencies: &Dependencies,
    deadline: Deadline<'_>,
) -> Result<biodata::Text, RequestError> {
    if let Some(key) = cx.fetch.trial_cursor_key() {
        let stored_ids;
        let sources = match dependencies {
            Dependencies::Portable(sources) => sources,
            Dependencies::Stored(outcomes) => {
                stored_ids = source_ids(outcomes);
                &stored_ids
            }
        };
        return portable::retain(cx, key, query, sources);
    }
    let Dependencies::Stored(sources) = dependencies else {
        return Err(biomcp_fetch::CacheError::Encoding.into());
    };
    let key = source_snapshot::key(NAMESPACE, &query.first_page())?;
    // The store derives the ID from the key and payload. Bind the dependencies there too.
    let payload = (query, source_ids(sources));
    let id = source_snapshot::write(cx, &key, &payload, sources, deadline)?;
    Ok(biodata::Text::new(&format!("{PREFIX}{id}"))
        .map_err(|_| biomcp_fetch::CacheError::Encoding)?)
}

fn source_ids(outcomes: &[SourceOutcome]) -> Vec<Source> {
    let mut ids: Vec<_> = outcomes.iter().map(|outcome| outcome.source).collect();
    ids.sort_unstable_by_key(|source| Into::<&'static str>::into(*source));
    ids.dedup();
    ids
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
