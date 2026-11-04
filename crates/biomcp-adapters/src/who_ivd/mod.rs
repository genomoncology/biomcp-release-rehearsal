//! Complete stored WHO export; product identities preserve every native CSV row.
mod raw;
mod search;
use biomcp_fetch::{KeyUse, SourceSpec, Terms};
use biomcp_types::{Source, Use};
pub use search::{Search, search};
use std::time::Duration;
/// One publisher declaration for health, outcomes and admission before any I/O.
pub static SPEC: SourceSpec = SourceSpec {
    destinations: biomcp_fetch::Destinations::Primary,
    user_agent: None,
    source: Source::WhoIvd,
    base: "https://extranet.who.int/prequal",
    terms: Terms {
        name: "WHO terms of use; export has no narrower license; commercial and substantial reproduction reserved",
        url: "https://www.who.int/about/policies/terms-of-use",
        usage: Use::Noncommercial,
    },
    quota: &crate::quotas::WHO,
    key: KeyUse::None,
    deadline: Duration::from_secs(15),
};

const SET: biomcp_fetch::BundleSet<raw::Native> = biomcp_fetch::BundleSet {
    spec: &SPEC,
    name: "who-ivd",
    parts: &[biomcp_fetch::BundlePart {
        name: "products",
        path: "/vitro-diagnostics/prequalified/in-vitro-diagnostics/export?page&_format=csv",
        format: None,
    }],
    format: biomcp_fetch::RowFormat::NativeCsv {
        max_bytes: 2 * 1024 * 1024,
    },
    // Local expiry policy, not a WHO publication cadence. Reads never synchronize.
    fresh: biomcp_fetch::Freshness::Days(1),
    row: std::marker::PhantomData,
};

/// Validate every native row before publishing one complete replacement snapshot.
pub fn sync(
    fetch: &biomcp_fetch::Fetch,
) -> Result<crate::Mapped<biomcp_types::SyncResult>, biomcp_fetch::RequestError> {
    fetch
        .sync_set(&SET)
        .map(|info| crate::mapped::bundle_sync(SPEC.source, info))
}

/// Read all source-ordered rows for one exact known product without provider I/O.
pub fn get(
    leg: &biomcp_fetch::Leg<'_>,
    query: &biomcp_types::DiagnosticGet,
) -> Result<(Option<biodata::WhoIvdProduct>, biomcp_fetch::BundleInfo), biomcp_fetch::RequestError>
{
    let (rows, info) = leg.stored_set_rows(
        &SET,
        biomcp_fetch::Find::Exact(biomcp_fetch::KeyKind::Id, query.id().as_str()),
    )?;
    if query
        .bundle_revision()
        .is_some_and(|v| Some(v.as_str()) != info.content_revision.as_deref())
    {
        return Err(biomcp_fetch::SourceError {
            source: Source::WhoIvd,
            reason: biomcp_types::Reason::Rejected,
            status: None,
            tls_cause: None,
            retry_after: None,
        }
        .into());
    }
    let id = rows.first().map(|row| row.value.id.clone());
    let product = id.map(|id| biodata::WhoIvdProduct {
        id,
        native_rows: rows.into_iter().map(|row| row.value.into()).collect(),
    });
    Ok((product, info))
}

/// Small read-only publisher probe; never an export or synchronization.
pub(crate) const HEALTH_PATH: &str = "/vitro-diagnostics/prequalified/in-vitro-diagnostics";

/// Inspect this source's complete local set without synchronizing it.
pub(crate) fn bundle_state(
    leg: &biomcp_fetch::Leg<'_>,
) -> Result<biomcp_types::LocalBundleState, biomcp_fetch::RequestError> {
    SET.state(leg)
}

/// Complete source policy and command declaration.
pub(crate) mod decl;
