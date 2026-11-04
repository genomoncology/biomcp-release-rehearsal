/// Convert the original reply retrieval time to its UTC calendar date.
pub fn received_date(fetched: Option<i64>) -> Option<biodata::PartialDate> {
    let fetched = fetched?;
    if fetched < 0 {
        return None;
    }
    let mut days = fetched / 86_400;
    for year in 1970..=9999 {
        let year_days = if biodata::PartialDate::new(year, Some(2), Some(29)).is_ok() {
            366
        } else {
            365
        };
        if days >= year_days {
            days -= year_days;
            continue;
        }
        for month in 1..=12 {
            for day in 1..=31 {
                if let Ok(date) = biodata::PartialDate::new(year, Some(month), Some(day)) {
                    if days == 0 {
                        return Some(date);
                    }
                    days -= 1;
                }
            }
        }
    }
    None
}
