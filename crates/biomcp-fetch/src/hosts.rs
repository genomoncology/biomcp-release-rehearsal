//! Source-owned request origins checked before storage or connection admission.
use crate::{InputError, SourceSpec};

/// Outbound policy beside the primary origin in a source declaration.
#[derive(Clone, Copy, Debug)]
pub enum Destinations {
    /// Requests stay on the declared base origin.
    Primary,
    /// Credential-free downloads may use these additional exact origins.
    Additional(&'static [&'static str]),
    /// Publisher PDF downloads keep their separate public-address and redirect guards.
    PublisherPdf,
}
impl SourceSpec {
    pub(crate) fn admit(&self, raw: &str, download: bool) -> Result<(), InputError> {
        let request = host(raw)?;
        if download && matches!(self.destinations, Destinations::PublisherPdf) {
            return crate::destination::checked(raw).map(|_| ());
        }
        if !self.base.is_empty() && request.origin() == host(self.base)?.origin() {
            return Ok(());
        }
        if download && let Destinations::Additional(origins) = self.destinations {
            for origin in origins {
                if request.origin() == secondary(origin)?.origin() {
                    return Ok(());
                }
            }
        }
        Err(InputError::Host)
    }
}
fn host(raw: &str) -> Result<url::Url, InputError> {
    let parsed = url::Url::parse(raw).map_err(|_| InputError::Host)?;
    if !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
        || parsed
            .host_str()
            .is_none_or(|host| host.is_empty() || host.contains('*'))
    {
        return Err(InputError::Host);
    }
    Ok(parsed)
}
fn secondary(raw: &str) -> Result<url::Url, InputError> {
    let parsed = host(raw)?;
    if parsed.path() != "/" || parsed.query().is_some() {
        return Err(InputError::Host);
    }
    Ok(parsed)
}

#[cfg(test)]
#[path = "hosts_tests.rs"]
mod tests;
