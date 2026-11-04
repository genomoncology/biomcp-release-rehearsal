//! Local catalog command selectors and their source-owned behavior.
use crate::Mapped;
use biomcp_fetch::{Fetch, Leg, RequestError};
use biomcp_types::{LocalBundleState, Source, SyncResult, WikiPathwaysSync};
/// One source-owned sync response, retaining each public response schema.
#[derive(Debug)]
pub enum BundleReply {
    /// Standard receipt and whether the last admitted catalog is stale.
    Standard(Result<Mapped<SyncResult>, RequestError>, bool),
    /// The two WikiPathways catalogs have their own receipt schema.
    WikiPathways(Result<Mapped<WikiPathwaysSync>, RequestError>),
}
/// One explicitly synchronized local catalog.
#[derive(Debug)]
pub struct BundleDecl {
    /// Public command word.
    pub name: &'static str,
    /// Command help supplied by the adapter.
    pub help: &'static str,
    /// Publisher identity.
    pub source: Source,
    /// Stable command discovery order.
    pub order: usize,
    /// Source-owned synchronization implementation.
    pub sync: fn(&Fetch) -> BundleReply,
    /// Local freshness independent of network reachability.
    pub state: fn(&Leg<'_>) -> Result<LocalBundleState, RequestError>,
}
/// Checked catalog selector. Named selectors live beside their source declarations.
#[derive(Clone, Copy)]
pub struct Bundle(pub(crate) &'static BundleDecl);
impl Bundle {
    /// Resolve a command word against the declaration list.
    pub fn named(name: &str) -> Option<Self> {
        bundles().find(|bundle| bundle.name() == name)
    }
    /// Public command spelling.
    pub fn name(self) -> &'static str {
        self.0.name
    }
    /// Source-owned command help.
    pub fn help(self) -> &'static str {
        self.0.help
    }
    /// Publisher identity.
    pub fn source(self) -> Source {
        self.0.source
    }
    /// Synchronize through the selected adapter.
    pub fn sync(self, fetch: &Fetch) -> BundleReply {
        (self.0.sync)(fetch)
    }
}
impl std::fmt::Debug for Bundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Bundle").field(&self.name()).finish()
    }
}
impl PartialEq for Bundle {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
    }
}
impl Eq for Bundle {}
/// Derive every sync selector in the established help order.
pub fn bundles() -> impl Iterator<Item = Bundle> {
    let mut bundles: Vec<_> = Source::ALL
        .iter()
        .flat_map(|&source| super::decl(source).bundles.iter().copied())
        .collect();
    bundles.sort_by_key(|bundle| bundle.order);
    bundles.into_iter().map(Bundle)
}

/// Shared help for a complete explicit replacement.
pub(crate) const REPLACE_HELP: &str = "Replace the complete validated local bundle atomically. Reads use stored data and never \
    synchronize. Failed replacement preserves the last usable bundle.";
/// Shared help for a bundle that can also refresh during reads.
pub(crate) const REFRESH_HELP: &str = "Synchronize a local source bundle.";
