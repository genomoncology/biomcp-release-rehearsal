//! Bound local cache diagnostics once for every command surface.
use biomcp_fetch::RequestError;
use biomcp_types::{CacheProblem, ErrorEnvelope};

pub(super) fn kind(error: Option<RequestError>) -> Option<CacheProblem> {
    match error? {
        RequestError::Cache(error) => Some(error.into()),
        _ => None,
    }
}

pub(crate) fn startup(
    error: crate::config::StartupError,
    encoding: super::Encoding,
) -> std::io::Result<super::Outcome> {
    let envelope = match error {
        crate::config::StartupError::Cache(error) => {
            let mut envelope = ErrorEnvelope::local();
            envelope.local_error = Some(error.into());
            envelope
        }
        crate::config::StartupError::Configuration(error) => ErrorEnvelope::configuration(error.0),
        crate::config::StartupError::Local => ErrorEnvelope::local(),
    };
    encoding.encode(envelope, crate::exit::Status::Local)
}
