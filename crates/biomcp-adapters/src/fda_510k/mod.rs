//! Exact native 510(k) identity is checked before ordinary cache admission.
use biomcp_fetch::{Input, Read};
mod raw;
use biomcp_fetch::{Arg, BadReply, Freshness, Leg, RequestError, SourceSpec, Terms};
use biomcp_types::{DiagnosticClearance, DiagnosticGet, Source, Use};
use std::time::Duration;
/// One source declaration for policy, quota and dispatch.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Fda510k,
    base: "https://api.fda.gov/device/510k.json",
    terms: Terms {
        name: "openFDA terms and license; third-party exceptions",
        url: "https://open.fda.gov/terms/",
        usage: Use::Open,
    },
    quota: &crate::quotas::OPENFDA,
    key: crate::openfda_key::OPTIONAL,
    deadline: Duration::from_secs(15),
};
/// Failed admission retains whether the returned identity conflicted.
#[derive(Debug)]
pub struct Failure {
    /// Ordinary fetch failure and source outcome.
    pub error: RequestError,
    /// A native row claimed another exact K number.
    pub identity_conflict: bool,
}
/// Retrieve only the exact source-native clearance; no harmonized fields enter the projection.
pub fn get(
    leg: &Leg<'_>,
    q: &DiagnosticGet,
) -> Result<Option<crate::Mapped<DiagnosticClearance>>, Failure> {
    let selector = format!("k_number:\"{}\"", q.id());
    let endpoint = SPEC
        .get("")
        .fresh(Freshness::Days(1))
        .bytes(2 * 1024 * 1024)
        .json_absence();
    let reply = leg
        .call(
            &endpoint,
            Input::Params(&[
                ("search", Arg::Text(&selector)),
                ("limit", Arg::Num(1)),
                ("skip", Arg::Num(0)),
            ]),
            Read::<raw::Response>::json()
                .check(&|reply| {
                    if let raw::Response::Page { meta, .. } = reply
                        && (meta.results.skip != 0
                            || meta.results.limit != 1
                            || meta.results.total != 1)
                    {
                        return Err(BadReply);
                    }
                    Ok(())
                })
                .cache_if(&|reply| identity(reply, q)),
        )
        .map(|received| received.value)
        .map_err(|error| Failure {
            error,
            identity_conflict: false,
        })?;
    if !identity(&reply, q) {
        return Err(Failure {
            error: biomcp_fetch::SourceError {
                source: Source::Fda510k,
                reason: biomcp_types::Reason::BadReply,
                status: Some(200),
                tls_cause: None,
                retry_after: None,
            }
            .into(),
            identity_conflict: true,
        });
    }
    Ok(match reply {
        raw::Response::Page { results, .. } => results.into_iter().next().map(raw::Record::mapped),
        raw::Response::Absent { .. } => None,
    })
}

fn identity(reply: &raw::Response, q: &DiagnosticGet) -> bool {
    !matches!(reply,raw::Response::Page{results,..} if results.iter().any(|row|row.k_number.as_str()!=q.id().as_str()))
}

/// Complete source policy and command declaration.
pub(crate) mod decl;

mod search;
pub use search::search;

pub use crate::device_search::Page as SearchPage;
