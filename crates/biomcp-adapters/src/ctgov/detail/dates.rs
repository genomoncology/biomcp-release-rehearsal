//! Keep registry date precision and the independently supplied qualifier.
use super::{Problems, RawDate};
use crate::raw::Field;
use biodata::TrialDate;

impl RawDate {
    pub(super) fn field(
        field: &str,
        value: Option<Field<Self>>,
        problems: &mut Problems,
    ) -> Option<TrialDate> {
        problems
            .field(field, value)
            .and_then(|date| date.mapped(field, problems))
    }
    fn mapped(self, field: &str, problems: &mut Problems) -> Option<TrialDate> {
        let path = format!("{field}.date");
        let date = problems.field(&path, self.date);
        let kind = problems.field(&format!("{field}.type"), self.kind);
        Some(TrialDate {
            date: problems.required(&path, date)?,
            kind,
        })
    }
}
