use super::{SPEC, raw::Native};
use biomcp_fetch::{
    BundleInfo, BundlePart, BundleSet, Fetch, Find, Freshness, Leg, RequestError, RowFormat, SetRow,
};
use biomcp_types::SyncResult;
use std::marker::PhantomData;
const SET: BundleSet<Native> = BundleSet {
    spec: &SPEC,
    name: "fda-cdx",
    fresh: Freshness::Days(7),
    row: PhantomData,
    format: RowFormat::Document {
        media: biomcp_fetch::DocumentMedia::Html,
    },
    parts: &[BundlePart {
        name: "companion",
        path: "/medical-devices/in-vitro-diagnostics/list-fda-authorized-companion-diagnostic-devices-in-vitro-and-imaging-tools",
        format: None,
    }],
};
pub(super) fn read(
    leg: &Leg<'_>,
    find: Find<'_>,
) -> Result<(Vec<SetRow<Native>>, BundleInfo), RequestError> {
    leg.stored_set_rows(&SET, find)
}
/// Replace the complete validated two-table document atomically.
pub fn sync(fetch: &Fetch) -> Result<crate::Mapped<SyncResult>, RequestError> {
    fetch
        .sync_set(&SET)
        .map(|info| crate::mapped::bundle_sync(SPEC.source, info))
}

/// Inspect this source's complete local set without synchronizing it.
pub(crate) fn state(leg: &Leg<'_>) -> Result<biomcp_types::LocalBundleState, RequestError> {
    SET.state(leg)
}
