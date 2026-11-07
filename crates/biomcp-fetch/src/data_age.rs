//! Disclose publisher dataset dates without treating acquisition as publication.
use crate::{Fetch, Leg};
use biomcp_types::{SourceNotice, SourceOutcome};

impl Leg<'_> {
    /// Retain a native dataset date or timestamp after the adapter identifies its role.
    /// Invalid dates and dates without day precision are disclosed without an age claim.
    pub fn data_date(&self, date: &str) {
        let notice = self.fetch.data_date_notice(date);
        let mut report = self
            .report
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !report.data_notices.contains(&notice) {
            report.data_notices.push(notice);
        }
    }
}

impl Fetch {
    /// Attach a retained publisher dataset date without repeating the same fact.
    pub fn retain_data_date(&self, source: &mut SourceOutcome, date: &str) {
        if !source.notices.iter().any(|notice| matches!(notice, SourceNotice::RequestGuidance { data_date: Some(known), .. } if known.as_ref() == date)) {
            source.notices.push(self.data_date_notice(date));
        }
    }
    /// Reassess retained publisher dates when returning stored continuations.
    pub fn refresh_data_dates(&self, sources: &mut [SourceOutcome]) {
        for source in sources {
            for notice in &mut source.notices {
                if let SourceNotice::RequestGuidance {
                    data_date: Some(date),
                    ..
                } = notice
                {
                    *notice = self.data_date_notice(date);
                }
            }
        }
    }

    /// Describe one native publisher dataset date using the current configured age limit.
    pub fn data_date_notice(&self, date: &str) -> SourceNotice {
        let max_days = self.config.source_data_max_age_days;
        let old = day(date).is_some_and(|date| {
            let today = crate::cache::now().div_euclid(86_400);
            today.saturating_sub(date) > i64::from(max_days)
        });
        let message = if old {
            format!(
                "Publisher dataset date: {date}. Old data: exceeds the configured {max_days}-day age limit. Retrieval does not establish a newer data date."
            )
        } else {
            format!(
                "Publisher dataset date: {date}. This is distinct from retrieval and record event dates."
            )
        };
        SourceNotice::RequestGuidance {
            data_date: Some(date.into()),
            message: message.into(),
            retry_after_seconds: None,
        }
    }
}

fn day(value: &str) -> Option<i64> {
    let date = value.get(..10)?;
    if value.len() != 10 {
        value.parse::<biomcp_types::SourceTimestampValue>().ok()?;
    }
    // Reuse the public calendar validator, including leap-year checks.
    format!("{date}T00:00:00Z")
        .parse::<biomcp_types::SourceTimestampValue>()
        .ok()?;
    let mut parts = date.split('-');
    let mut year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    // Gregorian civil date to Unix days, with March starting each leap-year cycle.
    year -= i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month + 2) / 5 + day - 1;
    Some(
        era * 146_097 + year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year
            - 719_468,
    )
}

#[cfg(test)]
#[path = "../tests/record/data_age.rs"]
mod tests;
