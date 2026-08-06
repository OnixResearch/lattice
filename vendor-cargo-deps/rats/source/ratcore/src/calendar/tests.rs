use super::CalDate;
use super::CalendarDateError;

fn date(year: i32, month: u32, day: u32) -> CalDate {
    CalDate::new(year, month, day).expect("test date is valid")
}

#[test]
fn leap_year_month_lengths_are_gregorian() {
    assert_eq!(date(2024, 2, 1).days_in_month(), 29);
    assert_eq!(date(2026, 2, 1).days_in_month(), 28);
    assert_eq!(date(1900, 2, 1).days_in_month(), 28);
    assert_eq!(date(2000, 2, 1).days_in_month(), 29);
}

#[test]
fn impossible_components_are_rejected() {
    assert_eq!(CalDate::new(2026, 13, 1), Err(CalendarDateError::InvalidMonth));
    assert_eq!(CalDate::new(2026, 0, 1), Err(CalendarDateError::InvalidMonth));
    assert_eq!(CalDate::new(2026, 2, 29), Err(CalendarDateError::InvalidDay));
    assert_eq!(CalDate::new(2024, 2, 30), Err(CalendarDateError::InvalidDay));
    assert_eq!(CalDate::new(0, 1, 1), Err(CalendarDateError::InvalidYear));
}

#[test]
fn next_and_previous_day_cross_month_and_year() {
    assert_eq!(date(2026, 3, 31).next_day(), date(2026, 4, 1));
    assert_eq!(date(2026, 12, 31).next_day(), date(2027, 1, 1));
    assert_eq!(date(2026, 4, 1).prev_day(), date(2026, 3, 31));
}

#[test]
fn month_arithmetic_clamps_only_derived_day() {
    assert_eq!(date(2026, 3, 31).next_month(), date(2026, 4, 30));
    assert_eq!(date(2026, 3, 31).prev_month(), date(2026, 2, 28));
}

#[test]
fn weekday_and_names_are_defined_for_valid_dates() {
    assert_eq!(date(2026, 4, 1).weekday(), 2);
    assert_eq!(date(2026, 1, 1).month_name(), "January");
    assert_eq!(date(2026, 12, 1).month_name(), "December");
    assert_eq!(date(2026, 6, 1).month_name_short(), "Jun");
}

#[test]
fn canonical_parse_display_and_order_round_trip() {
    let parsed = CalDate::parse("2026-04-01").unwrap();
    assert_eq!(parsed.to_string(), "2026-04-01");
    assert_eq!(parsed.format(), "2026-04-01");
    assert_eq!(CalDate::parse("2026-4-01"), Err(CalendarDateError::InvalidFormat));
    assert_eq!(CalDate::parse("2026-02-29"), Err(CalendarDateError::InvalidDay));
    assert!(parsed < date(2026, 4, 2));
}

#[test]
fn day_offsets_cross_boundaries() {
    let current = date(2026, 3, 15);
    assert_eq!(current.add_days(20), date(2026, 4, 4));
    assert_eq!(current.add_days(-15), date(2026, 2, 28));
}
