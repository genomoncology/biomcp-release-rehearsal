use biomcp_fetch::{LegReport, RequestError};
use biomcp_types::{PageInfo, Problem, ReplyData, Source, SourceOutcome, SourceState, Status};
/// Query result before the interface adds its typed envelope and next spelling.
#[derive(Debug)]
pub struct Outcome {
    /// Overall outcome.
    status: Status,
    pub(crate) publish_error: Option<biomcp_fetch::CacheError>,
    /// At least one native record was refused by its adapter.
    pub(crate) dropped: bool,
    /// Typed response body.
    pub body: ReplyData,
    /// Search counts.
    pub page: Option<PageInfo>,
    /// Sources actually asked.
    pub sources: Vec<SourceOutcome>,
    /// Rejected fields.
    pub problems: Vec<Problem>,
    /// Typed next actions.
    pub next: Vec<crate::next::Step>,
    /// Local errors remain distinct even alongside useful answers.
    pub local: Option<RequestError>,
}

/// Entity-owned response parts without authority to assign an outcome status.
pub(crate) struct Parts {
    pub body: ReplyData,
    pub page: Option<PageInfo>,
    pub sources: Vec<SourceOutcome>,
    pub problems: Vec<Problem>,
    pub next: Vec<crate::next::Step>,
    pub local: Option<RequestError>,
    pub dropped: bool,
}

/// Finish preclassified get items through the shared outcome owner.
pub(crate) fn finish(items: impl IntoIterator<Item = Status>, parts: Parts) -> Outcome {
    let mut out = Outcome {
        status: status(items.into_iter()),
        publish_error: None,
        body: parts.body,
        page: parts.page,
        sources: parts.sources,
        problems: parts.problems,
        next: parts.next,
        local: parts.local,
        dropped: parts.dropped,
    };
    if out.status == Status::Complete {
        item(&mut out, true, false, false);
    }
    out
}

impl Outcome {
    /// Complete section bounding and navigation before the interface spells steps.
    pub fn finish(
        &mut self,
        fetch: &biomcp_fetch::Fetch,
        request: &biomcp_types::Request,
        fresh: bool,
    ) -> Result<(), RequestError> {
        crate::drug::label_bound::bound(fetch, request, self, fresh)?;
        crate::next::finish(fetch, self);
        Ok(())
    }

    /// The shared owner exposes status for readers only.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Preserve actual adapter refusals independently of optional field diagnostics.
    pub(crate) fn include_refusals(&mut self, refusals: biomcp_adapters::Refusal) {
        self.dropped |= refusals == biomcp_adapters::Refusal::Refused;
        if self.status == Status::Complete {
            item(self, true, false, false);
        }
    }

    /// The query layer owns whether a failed typed response remains visible.
    pub fn body_visible(&self) -> bool {
        self.status != Status::Failed
            || match &self.body {
                ReplyData::TrialPage {
                    body: Some(body), ..
                } => body.condition_resolution.is_some(),
                ReplyData::NciTrialPage {
                    body: Some(body), ..
                } => body.condition_resolution.is_some(),
                _ => false,
            }
    }
}

/// Decide an item from identity, requested-section and source observations.
pub(crate) fn item(out: &mut Outcome, found: bool, identity_failed: bool, unavailable: bool) {
    out.status = item_status(
        found,
        identity_failed,
        unavailable
            || out.dropped
            || out.sources.iter().any(|source| {
                matches!(
                    source.state,
                    SourceState::Failed | SourceState::Stale | SourceState::Syncing
                )
            }),
    );
}

/// Combine the independently classified items in one get.
pub(crate) fn get(out: &mut Outcome, items: impl Iterator<Item = Status>) {
    out.status = status(items);
    if out.status == Status::Complete {
        item(out, true, false, false);
    }
}

/// Restore a previously classified stored item without assigning outside this owner.
pub(crate) fn restored(out: &mut Outcome, status: Status) {
    out.status = status;
}

/// Decide a search from its asked providers and adapter refusal flag.
pub(crate) fn search(out: &mut Outcome) {
    out.status = search_status(&out.sources, out.dropped);
}

/// Classify only the asked search providers, keeping resolver reads as provenance.
pub(crate) fn search_for(out: &mut Outcome, providers: &[Source]) {
    let sources: Vec<_> = out
        .sources
        .iter()
        .filter(|s| providers.contains(&s.source))
        .cloned()
        .collect();
    out.status = search_status(&sources, out.dropped);
}

/// Start an unanswered query with one body and no source observations.
pub(crate) fn empty(body: ReplyData) -> Outcome {
    Outcome {
        publish_error: None,
        dropped: false,
        status: Status::Failed,
        body,
        page: None,
        sources: Vec::new(),
        problems: Vec::new(),
        next: Vec::new(),
        local: None,
    }
}

/// Start a body whose successful local construction is already an answer.
pub(crate) fn ready(body: ReplyData) -> Outcome {
    let mut out = empty(body);
    item(&mut out, true, false, false);
    out
}

/// Construct an ordinary native page without snapshot or truncation metadata.
pub(crate) fn page_info(offset: u32, returned: u32, total: u64) -> PageInfo {
    PageInfo {
        offset,
        returned,
        total,
        pool: None,
        truncated: None,
    }
}

/// Classify an identity independently of optional metadata omissions.
pub(crate) fn item_status(found: bool, identity_failed: bool, unavailable: bool) -> Status {
    if identity_failed {
        Status::Failed
    } else if !found {
        Status::Absent
    } else if unavailable {
        Status::Partial
    } else {
        Status::Complete
    }
}

/// A search remains an answer when its providers return no records.
pub(crate) fn search_status(sources: &[SourceOutcome], dropped: bool) -> Status {
    let mut answered = false;
    let mut incomplete = dropped;
    for source in sources {
        match source.state {
            SourceState::Ok => answered = true,
            SourceState::Stale => {
                answered = true;
                incomplete = true;
            }
            SourceState::Failed | SourceState::Syncing => incomplete = true,
            SourceState::Skipped => (),
        }
    }
    if !answered {
        Status::Failed
    } else if incomplete {
        Status::Partial
    } else {
        Status::Complete
    }
}

/// Classify synchronization from source outcomes and actual native row refusals.
pub fn sync_status(sources: &[SourceOutcome], refusals: biomcp_adapters::Refusal) -> Status {
    search_status(sources, refusals == biomcp_adapters::Refusal::Refused)
}

/// Combine input outcomes without hiding a failed input behind a found one.
pub(crate) fn status(items: impl Iterator<Item = Status>) -> Status {
    let values: Vec<_> = items.collect();
    if values.iter().all(|v| *v == Status::Absent) {
        Status::Absent
    } else if values.iter().all(|v| *v == Status::Failed) {
        Status::Failed
    } else if values
        .iter()
        .any(|v| matches!(v, Status::Failed | Status::Partial))
    {
        Status::Partial
    } else {
        Status::Complete
    }
}
pub(crate) fn source(
    source: Source,
    report: LegReport,
    error: Option<RequestError>,
) -> SourceOutcome {
    let reason = error
        .and_then(reason)
        .or_else(|| report.failures.first().map(|failure| failure.reason));
    let mut outcome = biomcp_adapters::registry::outcome(
        source,
        if matches!(
            reason,
            Some(biomcp_types::Reason::Disabled | biomcp_types::Reason::NoKey)
        ) {
            SourceState::Skipped
        } else if report.stale {
            SourceState::Stale
        } else if reason == Some(biomcp_types::Reason::Syncing) {
            SourceState::Syncing
        } else if error.is_some() {
            SourceState::Failed
        } else {
            SourceState::Ok
        },
        reason,
    );
    outcome.notices.extend(biomcp_adapters::guidance::notices(
        source,
        reason,
        error.and_then(|e| match e {
            RequestError::Source(e) => e.retry_after,
            _ => None,
        }),
    ));
    outcome
        .notices
        .extend(report.cache_problems.into_iter().map(|error| {
            biomcp_types::SourceNotice::CacheProblem {
                reason: error.into(),
            }
        }));
    outcome
}
pub(crate) fn local(error: RequestError) -> Option<RequestError> {
    (!matches!(error, RequestError::Source(_))).then_some(error)
}
pub(crate) fn append(
    target: &mut Vec<biomcp_types::Problem>,
    values: impl IntoIterator<Item = biomcp_types::Problem>,
) {
    for value in values {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

/// Preserve a failed or stale source leg when later metadata from it succeeds.
pub(crate) fn retain_source_state(target: &mut SourceOutcome, incoming: &SourceOutcome) {
    for notice in &incoming.notices {
        if !target.notices.contains(notice) {
            target.notices.push(notice.clone());
        }
    }
    if target.state != SourceState::Failed
        && (target.state != SourceState::Stale || incoming.state == SourceState::Failed)
    {
        target.state = incoming.state;
        target.reason = incoming.reason;
    }
}

/// Preserve provider reasons and distinguish absent replay exchanges from local failures.
pub(crate) fn reason(error: RequestError) -> Option<biomcp_types::Reason> {
    match error {
        RequestError::Source(error) => Some(error.reason),
        RequestError::Recording(
            biomcp_fetch::RecordError::Missing | biomcp_fetch::RecordError::Exhausted,
        ) => Some(biomcp_types::Reason::ReplayMiss),
        _ => None,
    }
}
