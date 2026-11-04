pub use biodata::{PathwayRef, ReactomeId};
use serde::{Deserialize, Serialize};
/// A bounded list with the total before its cap was applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "Capped_{T}"))]
pub struct Capped<T> {
    /// References in stable source order.
    #[serde(default, skip_serializing_if = "<[_]>::is_empty")]
    pub items: Box<[T]>,
    /// Distinct references before capping.
    pub total: u64,
}
