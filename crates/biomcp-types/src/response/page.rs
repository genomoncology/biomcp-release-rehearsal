use serde::{Deserialize, Serialize};
/// Fixed lifetime and size of a stored search snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PoolInfo {
    /// Opaque snapshot identifier.
    pub id: crate::Text,
    /// Number of ordered identities.
    pub size: u32,
    /// Publication wall time in Unix seconds.
    pub created: u64,
    /// Fixed expiry wall time in Unix seconds.
    pub expires: u64,
}
/// Search counts after filtering and deduplication.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct PageInfo {
    /// Skipped results.
    pub offset: u32,
    /// Results returned on this page.
    pub returned: u32,
    /// Total matching identities held by this snapshot.
    pub total: u64,
    /// Optional pooled snapshot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool: Option<PoolInfo>,
    /// Matching source count exceeded the retrieved window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}
