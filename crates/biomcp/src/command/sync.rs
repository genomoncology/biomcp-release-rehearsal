use super::{Encoding, Outcome, bundle_source};
use biomcp_query::Bundle;
use biomcp_types::{Envelope, Status};
use std::io;
pub(super) fn run(
    fetch: &biomcp_fetch::Fetch,
    bundle: Bundle,
    encoding: Encoding,
) -> io::Result<Outcome> {
    match biomcp_query::sources::sync(fetch, bundle) {
        biomcp_query::sources::BundleReply::Standard(result, stale) => {
            let date_notice = result
                .as_ref()
                .ok()
                .and_then(|mapped| mapped.value.data_date.as_deref())
                .map(|date| fetch.data_date_notice(date));
            finish(bundle, result, stale, encoding, date_notice)
        }
        biomcp_query::sources::BundleReply::WikiPathways(result) => {
            finish(bundle, result, false, encoding, None)
        }
    }
}
fn finish<T: biomcp_types::ResponseBody<Query = biomcp_types::SyncQuery>>(
    bundle: Bundle,
    result: Result<biomcp_query::sources::Mapped<T>, biomcp_fetch::RequestError>,
    stale: bool,
    encoding: Encoding,
    date_notice: Option<biomcp_types::SourceNotice>,
) -> io::Result<Outcome> {
    let source = bundle_source(bundle);
    let refusals = result
        .as_ref()
        .map_or(biomcp_query::sources::Refusal::None, |mapped| {
            mapped.refusals
        });
    let local = result
        .as_ref()
        .err()
        .is_some_and(|error| !matches!(error, biomcp_fetch::RequestError::Source(_)));
    let source_error = result.as_ref().err().and_then(|error| match error {
        biomcp_fetch::RequestError::Source(error) => Some(*error),
        _ => None,
    });
    let reason = source_error.map(|error| error.reason);
    let source_status = if reason == Some(biomcp_types::Reason::Disabled) {
        biomcp_types::SourceStatus::Skipped(reason)
    } else if stale {
        biomcp_types::SourceStatus::Stale(reason)
    } else if result.is_ok() {
        biomcp_types::SourceStatus::Ok
    } else if reason == Some(biomcp_types::Reason::Syncing) {
        biomcp_types::SourceStatus::Syncing
    } else {
        biomcp_types::SourceStatus::Failed(reason)
    };
    let local_error = super::local::kind(result.as_ref().err().copied());
    let mut problems = Vec::new();
    let value = result.ok().map(|mapped| {
        problems = mapped.problems;
        mapped.value
    });
    let mut envelope =
        Envelope::<T>::new(Status::Failed, biomcp_types::SyncQuery { source }, value);
    envelope.problems = problems;
    envelope.local_error = local_error;
    let mut source_outcome = biomcp_query::sources::outcome(source, source_status);
    if let Some(error) = source_error {
        source_outcome.notices = biomcp_query::sources::notices(source, reason, error.retry_after);
        source_outcome.notices.extend(error.tls_notice());
        rate_limit_stderr(error, encoding)?;
    }
    source_outcome.notices.extend(date_notice);
    envelope.sources.push(source_outcome);
    let status = biomcp_query::sync_status(&envelope.sources, refusals);
    envelope.status = status;
    if matches!(
        source,
        biomcp_types::Source::Cbioportal | biomcp_types::Source::WikiPathways
    ) && reason == Some(biomcp_types::Reason::BadReply)
    {
        envelope.problems.push(biomcp_types::Problem {
            source,
            record: None,
            field: "catalog".into(),
            code: biomcp_types::ProblemCode::CollectionIncomplete,
        });
    }
    envelope.next = biomcp_query::sources::network(&envelope.sources)
        .iter()
        .map(|step| crate::spell::step(step.0, &step.1))
        .collect::<Result<Vec<_>, _>>()?;
    encoding.encode(
        envelope,
        if local {
            crate::exit::Status::Local
        } else {
            status.into()
        },
    )
}

fn rate_limit_stderr(error: biomcp_fetch::SourceError, encoding: Encoding) -> io::Result<()> {
    if error.reason == biomcp_types::Reason::RateLimited && !matches!(encoding, Encoding::Mcp) {
        use std::io::Write;
        let mut stderr = io::stderr().lock();
        write!(stderr, "{error}")?;
        if let Some(delay) = error.retry_after {
            write!(stderr, "; retry after {delay:?}")?;
        }
        writeln!(stderr)?;
    }
    Ok(())
}
