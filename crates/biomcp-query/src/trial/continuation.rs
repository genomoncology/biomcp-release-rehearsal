//! A rejected registry cursor offers an explicit uncached first-page request.
use crate::{Outcome, next::Step};
use biomcp_types::{ReplyData, Request, Source, StepKind, TrialSearch, UsageError};

pub(super) fn rejected(query: &TrialSearch, out: &mut Outcome) {
    out.body = ReplyData::Usage(UsageError::TrialContinuationRejected);
    out.next.push(Step::run(
        StepKind::Configure,
        &[Source::ClinicalTrialsGov],
        Request::SearchTrial(query.first_page()),
        true,
    ));
}
