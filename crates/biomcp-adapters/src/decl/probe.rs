//! Explicit bounded request shapes for source health.
use biomcp_fetch::{Arg, Leg, RequestError, SourceSpec};
/// Source-owned health behavior.
#[derive(Debug)]
pub enum Probe {
    /// A bounded GET endpoint and fixed parameters.
    Get(&'static str, &'static [(&'static str, Arg<'static>)]),
    /// A read-only GraphQL type query.
    GraphQl(&'static str),
    /// Adapter behavior that needs a declared endpoint or admission check.
    Custom(fn(&Leg<'_>) -> Result<(), RequestError>),
    /// No public probe is supported by the source.
    Unavailable,
    /// Publisher identity carried by another source, with no direct request.
    Component,
}
impl Probe {
    /// Run the declared request without choosing a source-specific fallback.
    pub fn run(&self, spec: &'static SourceSpec, leg: &Leg<'_>) -> Result<(), RequestError> {
        match self {
            Self::Get(path, params) => {
                leg.probe(&spec.get(path), biomcp_fetch::Input::Params(params))
            }
            Self::GraphQl(path) => leg.probe(
                &spec.post_json(path).idempotent(),
                biomcp_fetch::Input::json(
                    &[],
                    &ProbeQuery {
                        query: "{ __typename }",
                    },
                ),
            ),
            Self::Custom(probe) => probe(leg),
            Self::Unavailable | Self::Component => Err(biomcp_fetch::SourceError {
                source: spec.source,
                reason: biomcp_types::Reason::Rejected,
                status: None,
                tls_cause: None,
                retry_after: None,
            }
            .into()),
        }
    }
}
#[derive(serde::Serialize)]
struct ProbeQuery {
    query: &'static str,
}
