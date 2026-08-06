use super::*;

#[test]
fn clock_time_rejects_invalid_components_without_clamping() {
    assert_eq!(ClockTime::new(24, 0), Err(ClockTimeError::InvalidHour));
    assert_eq!(ClockTime::new(23, 60), Err(ClockTimeError::InvalidMinute));
    assert_eq!(ClockTime::new(9, 5).unwrap().to_string(), "09:05");
}

#[test]
fn percent_rejects_overfull_values() {
    assert_eq!(Percent::try_new(100), Ok(Percent::FULL));
    assert_eq!(Percent::try_new(101).unwrap_err().kind, ValueErrorKind::OutOfRange);
}

#[test]
fn interval_rejects_non_finite_and_out_of_range_values() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(UnitInterval::try_new(invalid).unwrap_err().kind, ValueErrorKind::NonFinite);
    }
    assert_eq!(UnitInterval::try_new(-0.1).unwrap_err().kind, ValueErrorKind::OutOfRange);
    assert_eq!(UnitInterval::try_new(1.1).unwrap_err().kind, ValueErrorKind::OutOfRange);
}

#[test]
fn positive_ratios_and_pixels_enforce_distinct_zero_policies() {
    assert_eq!(PositiveRatio::try_new(0.0).unwrap_err().kind, ValueErrorKind::NotPositive);
    assert_eq!(NonNegativePixels::try_new(0.0), Ok(NonNegativePixels::ZERO));
    assert_eq!(NonNegativePixels::try_new(f32::NAN).unwrap_err().kind, ValueErrorKind::NonFinite);
}
