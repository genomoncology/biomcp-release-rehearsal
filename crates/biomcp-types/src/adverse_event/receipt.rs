//! Normalize shared FDA receipt bounds before comparison or request serialization.
use crate::QueryInvalid;
use biodata::PartialDate;
pub(super) fn day(
    date: Option<PartialDate>,
    upper: bool,
    field: &str,
) -> Result<Option<PartialDate>, QueryInvalid> {
    date.map(|date| {
        let month = date.month().ok_or_else(|| QueryInvalid::new(&[field]))?;
        let day = date.day().unwrap_or_else(|| {
            if upper {
                (28..=31)
                    .rev()
                    .find(|day| PartialDate::new(date.year(), Some(month), Some(*day)).is_ok())
                    .unwrap_or(28)
            } else {
                1
            }
        });
        PartialDate::new(date.year(), Some(month), Some(day))
            .map_err(|_| QueryInvalid::new(&[field]))
    })
    .transpose()
}
