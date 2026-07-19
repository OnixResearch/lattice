//! Calendar date type and month math.
//!
//! Zero-dependency date helpers: weekday calculation, leap year handling,
//! and day or month arithmetic with clamping.

const DAYS_PER_WEEK: i32 = 7;
const WEEKDAY_MONDAY_OFFSET: i32 = 6;
const LEAP_YEAR_DIVISOR: i32 = 4;
const CENTURY_DIVISOR: i32 = 100;
const GREGORIAN_DIVISOR: i32 = 400;
const FIRST_MONTH: u32 = 1;
const SECOND_MONTH: u32 = 2;
const LAST_MONTH: u32 = 12;
const LAST_MONTH_INDEX: usize = 12;
const DAYS_IN_LONG_MONTH: u32 = 31;
const DAYS_IN_SHORT_MONTH: u32 = 30;
const DAYS_IN_FEBRUARY_COMMON: u32 = 28;
const DAYS_IN_FEBRUARY_LEAP: u32 = 29;
const UNKNOWN_MONTH_NAME: &str = "???";
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

fn month_index(month: u32) -> Option<usize> {
    month
        .checked_sub(FIRST_MONTH)
        .and_then(|zero_based| usize::try_from(zero_based).ok())
        .filter(|&index| index < LAST_MONTH_INDEX)
}

fn is_leap_year(year: i32) -> bool {
    year % LEAP_YEAR_DIVISOR == 0 && (year % CENTURY_DIVISOR != 0 || year % GREGORIAN_DIVISOR == 0)
}

fn common_days_in_month(month: u32) -> u32 {
    match month {
        FIRST_MONTH | 3 | 5 | 7 | 8 | 10 | LAST_MONTH => DAYS_IN_LONG_MONTH,
        SECOND_MONTH => DAYS_IN_FEBRUARY_COMMON,
        _ => DAYS_IN_SHORT_MONTH,
    }
}

/// Simple date value with no timezone and no time-of-day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[must_use]
pub struct CalDate {
    /// Calendar year.
    pub year: i32,
    /// Calendar month, usually in the range `1..=12`.
    pub month: u32,
    /// Day within the month, usually in the range `1..=31`.
    pub day: u32,
}

impl CalDate {
    /// Creates a new calendar date.
    ///
    /// This constructor stores the provided components without validation.
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Returns the day of week using `0 = Monday` through `6 = Sunday`.
    ///
    /// Invalid month or day values return `0`.
    #[must_use]
    pub fn weekday(&self) -> u32 {
        let Some(month_index) = month_index(self.month) else {
            return 0;
        };
        let Ok(day) = i32::try_from(self.day) else {
            return 0;
        };

        let mut adjusted_year = self.year;
        if self.month < 3 {
            adjusted_year -= 1;
        }

        let weekday = (adjusted_year + adjusted_year / LEAP_YEAR_DIVISOR
            - adjusted_year / CENTURY_DIVISOR
            + adjusted_year / GREGORIAN_DIVISOR
            + MONTH_OFFSETS[month_index]
            + day)
            % DAYS_PER_WEEK;

        let Ok(weekday) = u32::try_from((weekday + WEEKDAY_MONDAY_OFFSET) % DAYS_PER_WEEK) else {
            return 0;
        };
        weekday
    }

    /// Returns the number of days in the current month.
    ///
    /// Invalid months fall back to the short-month length of `30` days.
    #[must_use]
    pub fn days_in_month(&self) -> u32 {
        if self.month == SECOND_MONTH && is_leap_year(self.year) {
            DAYS_IN_FEBRUARY_LEAP
        } else {
            common_days_in_month(self.month)
        }
    }

    /// Returns the next calendar day.
    pub fn next_day(self) -> Self {
        if self.day < self.days_in_month() {
            Self {
                day: self.day.saturating_add(1),
                ..self
            }
        } else if self.month < LAST_MONTH {
            Self {
                month: self.month.saturating_add(1),
                day: FIRST_MONTH,
                ..self
            }
        } else {
            Self {
                year: self.year + 1,
                month: FIRST_MONTH,
                day: FIRST_MONTH,
            }
        }
    }

    /// Returns the previous calendar day.
    pub fn prev_day(self) -> Self {
        if self.day > FIRST_MONTH {
            Self {
                day: self.day.saturating_sub(1),
                ..self
            }
        } else if self.month > FIRST_MONTH {
            let previous_month = Self {
                month: self.month.saturating_sub(1),
                day: FIRST_MONTH,
                ..self
            };
            Self {
                day: previous_month.days_in_month(),
                ..previous_month
            }
        } else {
            Self {
                year: self.year - 1,
                month: LAST_MONTH,
                day: DAYS_IN_LONG_MONTH,
            }
        }
    }

    /// Returns the date offset by `n` days.
    ///
    /// Positive values move forward in time; negative values move backward.
    pub fn add_days(self, n: i32) -> Self {
        let mut date = self;
        let day_count = n.unsigned_abs();

        if n.is_negative() {
            for _ in 0..day_count {
                date = date.prev_day();
            }
        } else {
            for _ in 0..day_count {
                date = date.next_day();
            }
        }

        date
    }

    /// Returns the same day in the next month, clamped to the month length.
    pub fn next_month(self) -> Self {
        let (year, month) = if self.month == LAST_MONTH {
            (self.year + 1, FIRST_MONTH)
        } else {
            (self.year, self.month.saturating_add(1))
        };
        let days_in_month = Self::new(year, month, FIRST_MONTH).days_in_month();
        Self::new(year, month, self.day.min(days_in_month))
    }

    /// Returns the same day in the previous month, clamped to the month length.
    pub fn prev_month(self) -> Self {
        let (year, month) = if self.month == FIRST_MONTH {
            (self.year - 1, LAST_MONTH)
        } else {
            (self.year, self.month.saturating_sub(1))
        };
        let days_in_month = Self::new(year, month, FIRST_MONTH).days_in_month();
        Self::new(year, month, self.day.min(days_in_month))
    }

    /// Returns the full month name, such as `"January"`.
    #[must_use]
    pub fn month_name(&self) -> &'static str {
        month_index(self.month).map_or(UNKNOWN_MONTH_NAME, |index| MONTH_NAMES[index])
    }

    /// Returns the abbreviated month name, such as `"Jan"`.
    #[must_use]
    pub fn month_name_short(&self) -> &'static str {
        month_index(self.month).map_or(UNKNOWN_MONTH_NAME, |index| MONTH_NAMES_SHORT[index])
    }

    /// Formats the date as `YYYY-MM-DD`.
    #[must_use]
    pub fn format(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl std::fmt::Display for CalDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
