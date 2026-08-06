//! Validated Gregorian calendar dates and deterministic calendar arithmetic.
//!
//! Dates are canonical operational values: impossible components cannot be
//! constructed, parsed, compared, or rendered as accepted dates.

use core::str::FromStr;

const DAYS_PER_WEEK: i32 = 7;
const WEEKDAY_MONDAY_OFFSET: i32 = 6;
const LEAP_YEAR_DIVISOR: i32 = 4;
const CENTURY_DIVISOR: i32 = 100;
const GREGORIAN_DIVISOR: i32 = 400;
const FIRST_YEAR: i32 = 1;
const LAST_YEAR: i32 = 9_999;
const FIRST_MONTH: u32 = 1;
const SECOND_MONTH: u32 = 2;
const LAST_MONTH: u32 = 12;
const LAST_MONTH_INDEX: usize = 12;
const DAYS_IN_LONG_MONTH: u32 = 31;
const DAYS_IN_SHORT_MONTH: u32 = 30;
const DAYS_IN_FEBRUARY_COMMON: u32 = 28;
const DAYS_IN_FEBRUARY_LEAP: u32 = 29;
const MONTH_OFFSETS: [i32; LAST_MONTH_INDEX] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
const MONTH_NAMES: [&str; LAST_MONTH_INDEX] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const MONTH_NAMES_SHORT: [&str; LAST_MONTH_INDEX] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Stable calendar date rejection reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CalendarDateError {
    /// Text was not canonical `YYYY-MM-DD`.
    InvalidFormat,
    /// Year was outside the canonical four-digit Gregorian range.
    InvalidYear,
    /// Month was outside `1..=12`.
    InvalidMonth,
    /// Day was zero or exceeded the selected month's length.
    InvalidDay,
}

impl std::fmt::Display for CalendarDateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidFormat => "date must use canonical YYYY-MM-DD form",
            Self::InvalidYear => "date year must be in 1..=9999",
            Self::InvalidMonth => "date month must be in 1..=12",
            Self::InvalidDay => "date day is invalid for its month and year",
        })
    }
}

impl std::error::Error for CalendarDateError {}

fn month_index(month: u32) -> Option<usize> {
    month
        .checked_sub(FIRST_MONTH)
        .and_then(|zero_based| usize::try_from(zero_based).ok())
        .filter(|&index| index < LAST_MONTH_INDEX)
}

fn is_leap_year(year: i32) -> bool {
    year % LEAP_YEAR_DIVISOR == 0 && (year % CENTURY_DIVISOR != 0 || year % GREGORIAN_DIVISOR == 0)
}

fn days_in_month(year: i32, month: u32) -> Option<u32> {
    match month {
        FIRST_MONTH | 3 | 5 | 7 | 8 | 10 | LAST_MONTH => Some(DAYS_IN_LONG_MONTH),
        SECOND_MONTH if is_leap_year(year) => Some(DAYS_IN_FEBRUARY_LEAP),
        SECOND_MONTH => Some(DAYS_IN_FEBRUARY_COMMON),
        4 | 6 | 9 | 11 => Some(DAYS_IN_SHORT_MONTH),
        _ => None,
    }
}

/// Valid Gregorian date with no timezone or time-of-day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[must_use]
pub struct CalDate {
    year: i32,
    month: u32,
    day: u32,
}

impl CalDate {
    /// Validates Gregorian date components.
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, CalendarDateError> {
        if !(FIRST_YEAR..=LAST_YEAR).contains(&year) {
            return Err(CalendarDateError::InvalidYear);
        }
        let Some(month_days) = days_in_month(year, month) else {
            return Err(CalendarDateError::InvalidMonth);
        };
        if day == 0 || day > month_days {
            return Err(CalendarDateError::InvalidDay);
        }
        Ok(Self { year, month, day })
    }

    /// Parses canonical `YYYY-MM-DD` text.
    pub fn parse(value: &str) -> Result<Self, CalendarDateError> {
        value.parse()
    }

    /// Calendar year.
    #[must_use]
    pub const fn year(self) -> i32 {
        self.year
    }

    /// Calendar month in `1..=12`.
    #[must_use]
    pub const fn month(self) -> u32 {
        self.month
    }

    /// Day within the selected month.
    #[must_use]
    pub const fn day(self) -> u32 {
        self.day
    }

    /// Returns the day of week using `0 = Monday` through `6 = Sunday`.
    #[must_use]
    pub fn weekday(self) -> u32 {
        let month_index = month_index(self.month).expect("validated month has an index");
        let day = i32::try_from(self.day).expect("validated day fits i32");
        let mut adjusted_year = self.year;
        if self.month < 3 {
            adjusted_year -= 1;
        }
        let weekday = (adjusted_year + adjusted_year / LEAP_YEAR_DIVISOR - adjusted_year / CENTURY_DIVISOR
            + adjusted_year / GREGORIAN_DIVISOR
            + MONTH_OFFSETS[month_index]
            + day)
            % DAYS_PER_WEEK;
        u32::try_from((weekday + WEEKDAY_MONDAY_OFFSET) % DAYS_PER_WEEK).expect("weekday is nonnegative")
    }

    /// Returns the number of days in the current month.
    #[must_use]
    pub fn days_in_month(self) -> u32 {
        days_in_month(self.year, self.month).expect("validated date has a valid month")
    }

    /// Returns the next calendar day, saturating at `9999-12-31`.
    pub fn next_day(self) -> Self {
        if self.day < self.days_in_month() {
            return Self {
                day: self.day + 1,
                ..self
            };
        }
        if self.month < LAST_MONTH {
            return Self {
                month: self.month + 1,
                day: FIRST_MONTH,
                ..self
            };
        }
        if self.year < LAST_YEAR {
            return Self {
                year: self.year + 1,
                month: FIRST_MONTH,
                day: FIRST_MONTH,
            };
        }
        self
    }

    /// Returns the previous calendar day, saturating at `0001-01-01`.
    pub fn prev_day(self) -> Self {
        if self.day > FIRST_MONTH {
            return Self {
                day: self.day - 1,
                ..self
            };
        }
        if self.month > FIRST_MONTH {
            let month = self.month - 1;
            return Self {
                month,
                day: days_in_month(self.year, month).expect("previous month is valid"),
                ..self
            };
        }
        if self.year > FIRST_YEAR {
            return Self {
                year: self.year - 1,
                month: LAST_MONTH,
                day: DAYS_IN_LONG_MONTH,
            };
        }
        self
    }

    /// Returns the date offset by `n` days, saturating at supported year bounds.
    pub fn add_days(self, n: i32) -> Self {
        let mut date = self;
        for _ in 0..n.unsigned_abs() {
            date = if n.is_negative() {
                date.prev_day()
            } else {
                date.next_day()
            };
        }
        date
    }

    /// Returns the first day of this date's month.
    pub const fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// Returns Monday of the week containing this date.
    pub fn week_start(self) -> Self {
        self.add_days(-i32::try_from(self.weekday()).expect("weekday fits i32"))
    }

    /// Returns the abbreviated weekday name.
    #[must_use]
    pub fn weekday_name(self) -> &'static str {
        const NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        NAMES[usize::try_from(self.weekday()).expect("weekday fits usize")]
    }

    /// Returns the same day in the next month, clamped to the month length.
    pub fn next_month(self) -> Self {
        if self.year == LAST_YEAR && self.month == LAST_MONTH {
            return self;
        }
        let (year, month) = if self.month == LAST_MONTH {
            (self.year + 1, FIRST_MONTH)
        } else {
            (self.year, self.month + 1)
        };
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month).expect("next month is valid")),
        }
    }

    /// Returns the same day in the previous month, clamped to the month length.
    pub fn prev_month(self) -> Self {
        if self.year == FIRST_YEAR && self.month == FIRST_MONTH {
            return self;
        }
        let (year, month) = if self.month == FIRST_MONTH {
            (self.year - 1, LAST_MONTH)
        } else {
            (self.year, self.month - 1)
        };
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month).expect("previous month is valid")),
        }
    }

    /// Returns the full month name, such as `"January"`.
    #[must_use]
    pub fn month_name(self) -> &'static str {
        MONTH_NAMES[month_index(self.month).expect("validated month has an index")]
    }

    /// Returns the abbreviated month name, such as `"Jan"`.
    #[must_use]
    pub fn month_name_short(self) -> &'static str {
        MONTH_NAMES_SHORT[month_index(self.month).expect("validated month has an index")]
    }

    /// Formats the date as canonical `YYYY-MM-DD`.
    #[must_use]
    pub fn format(self) -> String {
        self.to_string()
    }
}

impl FromStr for CalDate {
    type Err = CalendarDateError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 10 || value.as_bytes()[4] != b'-' || value.as_bytes()[7] != b'-' {
            return Err(CalendarDateError::InvalidFormat);
        }
        let year = value[0..4].parse::<i32>().map_err(|_| CalendarDateError::InvalidFormat)?;
        let month = value[5..7].parse::<u32>().map_err(|_| CalendarDateError::InvalidFormat)?;
        let day = value[8..10].parse::<u32>().map_err(|_| CalendarDateError::InvalidFormat)?;
        let date = Self::new(year, month, day)?;
        if date.to_string() != value {
            return Err(CalendarDateError::InvalidFormat);
        }
        Ok(date)
    }
}

impl std::fmt::Display for CalDate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
