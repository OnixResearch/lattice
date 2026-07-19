use super::*;

#[test]
fn days_in_month_leap() {
    assert_eq!(CalDate::new(2024, 2, 1).days_in_month(), 29);
    assert_eq!(CalDate::new(2026, 2, 1).days_in_month(), 28);
    assert_eq!(CalDate::new(1900, 2, 1).days_in_month(), 28);
    assert_eq!(CalDate::new(2000, 2, 1).days_in_month(), 29);
}

#[test]
fn days_in_month_invalid_month_falls_back_to_short_month() {
    assert_eq!(CalDate::new(2026, 13, 1).days_in_month(), 30);
}

#[test]
fn next_day_cross_month() {
    assert_eq!(
        CalDate::new(2026, 3, 31).next_day(),
        CalDate::new(2026, 4, 1)
    );
}

#[test]
fn next_day_cross_year() {
    assert_eq!(
        CalDate::new(2026, 12, 31).next_day(),
        CalDate::new(2027, 1, 1)
    );
}

#[test]
fn prev_day_cross_month() {
    assert_eq!(
        CalDate::new(2026, 4, 1).prev_day(),
        CalDate::new(2026, 3, 31)
    );
}

#[test]
fn next_month_clamps() {
    assert_eq!(
        CalDate::new(2026, 3, 31).next_month(),
        CalDate::new(2026, 4, 30)
    );
}

#[test]
fn prev_month_clamps() {
    assert_eq!(
        CalDate::new(2026, 3, 31).prev_month(),
        CalDate::new(2026, 2, 28)
    );
}

#[test]
fn weekday_known_date() {
    // 2026-04-01 is a Wednesday = 2
    assert_eq!(CalDate::new(2026, 4, 1).weekday(), 2);
}

#[test]
fn month_names() {
    assert_eq!(CalDate::new(2026, 1, 1).month_name(), "January");
    assert_eq!(CalDate::new(2026, 12, 1).month_name(), "December");
    assert_eq!(CalDate::new(2026, 6, 1).month_name_short(), "Jun");
}

#[test]
fn invalid_month_name_uses_placeholder() {
    assert_eq!(CalDate::new(2026, 0, 1).month_name(), "???");
    assert_eq!(CalDate::new(2026, 0, 1).month_name_short(), "???");
}

#[test]
fn invalid_weekday_uses_zero_fallback() {
    assert_eq!(CalDate::new(2026, 0, 1).weekday(), 0);
}

#[test]
fn display_format() {
    assert_eq!(CalDate::new(2026, 4, 1).to_string(), "2026-04-01");
    assert_eq!(CalDate::new(2026, 4, 1).format(), "2026-04-01");
}

#[test]
fn add_days_forward_and_back() {
    let date = CalDate::new(2026, 3, 15);
    assert_eq!(date.add_days(20), CalDate::new(2026, 4, 4));
    assert_eq!(date.add_days(-15), CalDate::new(2026, 2, 28));
}
