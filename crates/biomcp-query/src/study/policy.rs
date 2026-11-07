//! One read-time study policy for catalog cards, saved pages and cohort observations.
use crate::{Ctx, Outcome};
use biomcp_types::{Reason, Source, SourceNotice, SourceStatus};

pub(super) fn excluded(cx: &Ctx<'_>, id: &str) -> bool {
    cx.fetch.commercial_studies_only()
        && biomcp_adapters::cbioportal::commercial_restriction(id).is_some()
}

pub(super) fn notice(out: &mut Outcome, ids: &[&str], skipped: bool) {
    if ids.is_empty() {
        return;
    }
    let status = if skipped {
        SourceStatus::Skipped(Some(Reason::Disabled))
    } else {
        SourceStatus::Ok
    };
    let mut source = biomcp_adapters::registry::outcome(Source::Cbioportal, status);
    for id in ids {
        let Some(reason) = biomcp_adapters::cbioportal::commercial_restriction(id) else {
            continue;
        };
        source.notices.push(SourceNotice::RequestGuidance {
            message: format!("Study {id} was left out in restricted mode: {reason}").into(),
            retry_after_seconds: None,
        });
    }
    out.sources.push(source);
}

/// Withhold every section and recurrence page of a flagged study before source access.
/// Policy omission is the whole answer, so the reply is complete and carries the notice.
pub(crate) fn blocked(cx: &Ctx<'_>, id: &str, out: &mut Outcome) -> bool {
    if !excluded(cx, id) {
        return false;
    }
    notice(out, &[id], true);
    crate::status::item(out, true, false, false);
    true
}
