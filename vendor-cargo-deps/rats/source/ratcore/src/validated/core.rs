//! Focused validated values shared by renderer-neutral model boundaries.
//!
//! These types cover only domains used by more than one workspace consumer.
//! Domain-specific units such as cache byte sizes and Typst measurements stay
//! in their owning crates.

/// Stable reason a primitive value was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueErrorKind {
    /// A floating-point input was NaN or infinite.
    NonFinite,
    /// A value fell outside the domain's inclusive range.
    OutOfRange,
    /// A value required a strictly positive number.
    NotPositive,
}

/// Deterministic validation error for a focused numeric value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError {
    /// Domain being constructed.
    pub domain: &'static str,
    /// Stable rejection reason.
    pub kind: ValueErrorKind,
    /// Required range or policy.
    pub expected: &'static str,
}

impl ValueError {
    const fn new(domain: &'static str, kind: ValueErrorKind, expected: &'static str) -> Self {
        Self { domain, kind, expected }
    }
}

impl core::fmt::Display for ValueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{} value is {:?}; expected {}", self.domain, self.kind, self.expected)
    }
}

impl std::error::Error for ValueError {}

/// Rejected 24-hour clock component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockTimeError {
    /// Hour was outside `0..=23`.
    InvalidHour,
    /// Minute was outside `0..=59`.
    InvalidMinute,
}

impl core::fmt::Display for ClockTimeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidHour => "hour must be in 0..=23",
            Self::InvalidMinute => "minute must be in 0..=59",
        })
    }
}

impl std::error::Error for ClockTimeError {}

/// Valid minute-resolution time on a 24-hour clock.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClockTime {
    hour: u8,
    minute: u8,
}

impl ClockTime {
    /// Validates hour and minute without clamping.
    pub const fn new(hour: u8, minute: u8) -> Result<Self, ClockTimeError> {
        if hour > 23 {
            return Err(ClockTimeError::InvalidHour);
        }
        if minute > 59 {
            return Err(ClockTimeError::InvalidMinute);
        }
        Ok(Self { hour, minute })
    }

    /// Hour in `0..=23`.
    #[must_use]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// Minute in `0..=59`.
    #[must_use]
    pub const fn minute(self) -> u8 {
        self.minute
    }
}

impl core::fmt::Display for ClockTime {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// Human percentage in the inclusive range `0..=100`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Percent(u8);

impl Percent {
    /// Zero percent.
    pub const ZERO: Self = Self(0);
    /// One hundred percent.
    pub const FULL: Self = Self(100);

    /// Validates a percentage.
    pub const fn try_new(value: u8) -> Result<Self, ValueError> {
        if value <= Self::FULL.0 {
            Ok(Self(value))
        } else {
            Err(ValueError::new("percent", ValueErrorKind::OutOfRange, "0..=100"))
        }
    }

    /// Returns the percentage scalar.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl core::fmt::Display for Percent {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "{}%", self.0)
    }
}

/// Finite normalized scalar in the inclusive range `0.0..=1.0`.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct UnitInterval(f64);

impl UnitInterval {
    /// Empty interval value.
    pub const ZERO: Self = Self(0.0);
    /// Half interval value.
    pub const HALF: Self = Self(0.5);
    /// Full interval value.
    pub const FULL: Self = Self(1.0);

    /// Validates a normalized scalar.
    pub fn try_new(value: f64) -> Result<Self, ValueError> {
        if !value.is_finite() {
            return Err(ValueError::new("unit interval", ValueErrorKind::NonFinite, "a finite value in 0.0..=1.0"));
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ValueError::new("unit interval", ValueErrorKind::OutOfRange, "0.0..=1.0"));
        }
        Ok(Self(value))
    }

    /// Returns the normalized scalar.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }

    /// Converts to a rounded human percentage.
    #[must_use]
    pub fn rounded_percent(self) -> Percent {
        let value = (self.0 * 100.0).round() as u8;
        Percent::try_new(value).unwrap_or(Percent::FULL)
    }
}

/// Finite strictly-positive ratio.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct PositiveRatio(f32);

impl PositiveRatio {
    /// Validates a positive ratio.
    pub fn try_new(value: f32) -> Result<Self, ValueError> {
        if !value.is_finite() {
            return Err(ValueError::new(
                "positive ratio",
                ValueErrorKind::NonFinite,
                "a finite value greater than zero",
            ));
        }
        if value <= 0.0 {
            return Err(ValueError::new("positive ratio", ValueErrorKind::NotPositive, "> 0"));
        }
        Ok(Self(value))
    }

    /// Returns the ratio scalar.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

/// Finite nonnegative pixel extent.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct NonNegativePixels(f32);

impl NonNegativePixels {
    /// Zero pixels.
    pub const ZERO: Self = Self(0.0);

    /// Validates a pixel extent.
    pub fn try_new(value: f32) -> Result<Self, ValueError> {
        if !value.is_finite() {
            return Err(ValueError::new("pixel extent", ValueErrorKind::NonFinite, "a finite nonnegative value"));
        }
        if value < 0.0 {
            return Err(ValueError::new("pixel extent", ValueErrorKind::OutOfRange, ">= 0"));
        }
        Ok(Self(value))
    }

    /// Returns the pixel scalar.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
