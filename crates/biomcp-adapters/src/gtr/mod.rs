//! Stored GTR test/version and relation reports under one atomic snapshot.
mod bundle;
mod get;
mod map;
mod raw;
mod search;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::{Source, Use};
pub(crate) use bundle::state as bundle_state;
pub use bundle::sync;
pub use get::{Record, get};
pub use search::{Search, search};
use std::time::Duration;
/// One publisher declaration for health, outcomes and admission before any I/O.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::Gtr,
    base: "https://ftp.ncbi.nlm.nih.gov/pub/GTR",
    terms: Terms {
        name: "NCBI usage policies; submitter and third-party rights retained",
        url: "https://www.ncbi.nlm.nih.gov/home/about/policies/",
        usage: Use::Open,
    },
    quota: &crate::quotas::GTR,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};

fn revision(
    expected: Option<&biodata::Text>,
    info: &biomcp_fetch::BundleInfo,
) -> Result<(), biomcp_fetch::RequestError> {
    if expected.is_some_and(|v| Some(v.as_str()) != info.content_revision.as_deref()) {
        return Err(biomcp_fetch::SourceError {
            source: Source::Gtr,
            reason: biomcp_types::Reason::Rejected,
            status: None,
            tls_cause: None,
            retry_after: None,
        }
        .into());
    }
    Ok(())
}
fn conflict(id: &biodata::GtrTestId) -> biomcp_types::Problem {
    biomcp_types::Problem {
        source: Source::Gtr,
        record: crate::mapped::record(id),
        field: "test_accession_ver".into(),
        code: biomcp_types::ProblemCode::IdentityConflict,
    }
}

/// Small read-only publisher probe; never an export or synchronization.
pub(crate) const HEALTH_PATH: &str = "/data/";

/// Complete source policy and command declaration.
pub(crate) mod decl;
