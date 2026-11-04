//! Preserve native finite numbers and explicit nonfinite scalar categories in JSON.
use serde::{Deserialize, Serialize};
biodata::wire_enum! {
    /// Native nonfinite scalar category, without assigning a missing-value meaning.
    AlphaGenomeNonfiniteKind {
        /// Native not-a-number value.
        Nan => "nan",
        /// Native positive infinity.
        PositiveInfinity => "positive-infinity",
        /// Native negative infinity.
        NegativeInfinity => "negative-infinity",
    }
}
/// One native scalar; finite values remain JSON numbers and nonfinite values stay explicit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum AlphaGenomeScalar {
    /// A finite JSON number.
    Finite(biodata::FiniteNumber),
    /// A native nonfinite value without invented units, payload meaning or missingness.
    Nonfinite {
        /// Exact scalar category.
        kind: AlphaGenomeNonfiniteKind,
    },
}
impl AlphaGenomeScalar {
    /// Preserve a native FLOAT32 scalar without converting nonfinite values to null.
    pub fn from_float32(value: f32) -> Result<Self, biodata::Error> {
        let kind = if value.is_nan() {
            Some(AlphaGenomeNonfiniteKind::Nan)
        } else if value == f32::INFINITY {
            Some(AlphaGenomeNonfiniteKind::PositiveInfinity)
        } else if value == f32::NEG_INFINITY {
            Some(AlphaGenomeNonfiniteKind::NegativeInfinity)
        } else {
            None
        };
        match kind {
            Some(kind) => Ok(Self::Nonfinite { kind }),
            None => f64::from(value)
                .try_into()
                .map(Self::Finite)
                .map_err(|_| biodata::Error::NumberNonFinite),
        }
    }
}
