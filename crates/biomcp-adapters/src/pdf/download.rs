//! Credential-free bounded publisher downloads; PDF content is never parsed.
use biomcp_fetch::Input;
/// Maximum publisher bytes independent of ordinary API response limits.
pub const MAX_BYTES: u64 = 25 * 1024 * 1024;
/// Read a publisher file without caching or forwarding an API credential.
pub fn download(
    leg: &biomcp_fetch::Leg<'_>,
    url: &str,
) -> Result<Vec<u8>, biomcp_fetch::RequestError> {
    from_source(leg, &crate::semantic_scholar::PUBLISHER, url)
}

pub(super) fn from_source(
    leg: &biomcp_fetch::Leg<'_>,
    spec: &'static biomcp_fetch::SourceSpec,
    url: &str,
) -> Result<Vec<u8>, biomcp_fetch::RequestError> {
    let endpoint = spec.download(url)?.bytes(MAX_BYTES);
    leg.call(&endpoint, Input::Params(&[]), biomcp_fetch::Read::bytes())
        .map(|received| received.value.0)
        .map_err(|error| match error {
            biomcp_fetch::RequestError::Source(mut source)
                if source.status.is_some_and(|s| (400..500).contains(&s)) =>
            {
                source.reason = biomcp_types::Reason::Rejected;
                source.into()
            }
            error => error,
        })
}
