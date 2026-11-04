//! Original positions for independently refused native collection members.
use crate::{mapped::Problems, raw::Field};
use serde::{Deserialize, Serialize};
/// Position metadata exists only when a native array lost a member.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NativePositions {
    /// Native position of each admitted row, in unchanged order.
    pub indexes: Box<[u32]>,
    /// Complete native array length before rejecting members.
    pub total: u32,
}
impl NativePositions {
    /// Check retained metadata against its admitted row collection.
    pub fn check(&self, rows: usize) -> bool {
        self.indexes.len() == rows
            && self.indexes.iter().all(|i| *i < self.total)
            && self
                .indexes
                .iter()
                .zip(self.indexes.iter().skip(1))
                .all(|(a, b)| a < b)
    }
    /// Enumerate original positions, including dense legacy collections.
    pub fn enumerate<'a, T>(
        rows: &'a [T],
        positions: Option<&'a Self>,
    ) -> impl Iterator<Item = (u32, &'a T)> {
        rows.iter().enumerate().filter_map(move |(index, row)| {
            let index = match positions {
                Some(p) => p.indexes.get(index).copied()?,
                None => index as u32,
            };
            Some((index, row))
        })
    }
}
impl Problems {
    pub(crate) fn indexed_fields<T>(
        &mut self,
        field: &str,
        value: Option<Field<Vec<Field<T>>>>,
        refusals: &mut crate::Refusal,
    ) -> (Box<[T]>, Option<NativePositions>) {
        let values = self.field(field, value).unwrap_or_default();
        let total = values.len() as u32;
        let mut indexes = Vec::new();
        let rows: Box<[_]> = values
            .into_iter()
            .enumerate()
            .filter_map(|(index, value)| {
                let value = self.record(field, value, refusals)?;
                indexes.push(index as u32);
                Some(value)
            })
            .collect();
        let positions = (rows.len() != total as usize).then_some(NativePositions {
            indexes: indexes.into(),
            total,
        });
        (rows, positions)
    }
}
