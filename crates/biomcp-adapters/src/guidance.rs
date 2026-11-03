//! Reviewed source timing guidance shared by replies and command help.
use biomcp_types::{Reason, Source, SourceNotice};
/// Explain a source deadline without inferring an outage or a provider completion guarantee.
pub fn deadline(source: Source) -> Option<&'static str> {
    crate::decl::decl(source).guidance.deadline
}
pub use crate::vaers::decl::WONDER_QUOTA;
/// Preserve a latency or admission reason and attach the corresponding next step.
pub fn notices(
    source: Source,
    reason: Option<Reason>,
    retry: Option<std::time::Duration>,
) -> Vec<SourceNotice> {
    let message = match reason {
        Some(Reason::DeadlineExceeded) => deadline(source),
        Some(Reason::RateLimited) => {
            Some("The source rate-limited this request; retry after the reported delay.")
        }
        Some(Reason::LocalQuota) => crate::decl::decl(source).guidance.quota,
        _ => None,
    };
    message
        .map(|message| SourceNotice::RequestGuidance {
            message: message.into(),
            retry_after_seconds: retry.map(|d| d.as_secs() + u64::from(d.subsec_nanos() != 0)),
        })
        .into_iter()
        .collect()
}
