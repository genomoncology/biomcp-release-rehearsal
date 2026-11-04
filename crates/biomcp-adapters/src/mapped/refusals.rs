//! Shared facts for actually declined native records.
use super::{Field, Problems};

/// Whether a mapper refused a record, independent of field diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize)]
pub enum Refusal {
    /// No record was refused.
    None,
    /// At least one record was refused at its mapping boundary.
    Refused,
}
impl Refusal {
    /// Preserve an actual refusal from either mapping stage.
    pub fn include(&mut self, other: Self) {
        if other == Self::Refused {
            *self = other;
        }
    }
}

impl Problems {
    /// Decline an examined native record while preserving its field diagnostic.
    pub(crate) fn record<T>(
        &mut self,
        field: &str,
        value: Field<T>,
        refusals: &mut Refusal,
    ) -> Option<T> {
        match value {
            Field::Value(value) => Some(value),
            invalid @ Field::Invalid(_) => {
                *refusals = Refusal::Refused;
                self.field(field, Some(invalid))
            }
        }
    }
    /// Inspect members only after admitting their optional collection container.
    pub(crate) fn records<T>(
        &mut self,
        field: &str,
        value: Option<Field<Vec<Field<T>>>>,
        refusals: &mut Refusal,
    ) -> Box<[T]> {
        self.field(field, value)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|row| self.record(field, row, refusals))
            .collect()
    }
}
