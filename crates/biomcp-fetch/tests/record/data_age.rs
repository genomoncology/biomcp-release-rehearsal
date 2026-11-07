//! Calendar arithmetic needs fixed historical boundaries without a public clock control.
#[test]
fn calendar_days_preserve_leap_years_and_reject_partial_dates() {
    assert_eq!(super::day("1970-01-01"), Some(0));
    assert_eq!(
        super::day("2000-03-01")
            .zip(super::day("2000-02-28"))
            .map(|(a, b)| a - b),
        Some(2)
    );
    assert_eq!(
        super::day("1900-03-01")
            .zip(super::day("1900-02-28"))
            .map(|(a, b)| a - b),
        Some(1)
    );
    assert_eq!(super::day("2026-02-29"), None);
    assert_eq!(super::day("2026-01"), None);
    assert_eq!(super::day("2020-01-01 invalid"), None);
}
