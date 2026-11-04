/// Counts removed by explicit local store cleanup. Fresh state and bundles remain.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StoreClean {
    /// Expired source replies.
    pub replies: u64,
    /// Expired search pools.
    pub pools: u64,
    /// Items belonging to removed pools.
    pub items: u64,
    /// Expired trial and citation snapshots.
    pub snapshots: u64,
    /// Older incompatible cache files.
    pub files: u64,
}
impl super::ResponseBody for StoreClean {
    const SCHEMA: &'static str = "biomcp.store-clean/4";
    type Query = super::EmptyQuery;
}

/// Measured lengths of owned files, including unused database pages and present sidecars.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct StoreSize {
    /// All measured owned files, including retained generations and partial downloads.
    pub total_bytes: u64,
    /// Active database plus its present WAL and SHM files, counted once each.
    pub active_bytes: u64,
    /// Retained database generations plus their present WAL and SHM files.
    pub retained_bytes: u64,
    /// Incomplete download files belonging to active or retained databases.
    pub partial_download_bytes: u64,
}
biodata::wire_enum! {
    /// Why physical bytes cannot be attributed to one local bundle.
    BundleSize {
        /// Bundles share database pages, indexes and journal files with other stored state.
        SharedStorage => "shared-storage",
    }
}
