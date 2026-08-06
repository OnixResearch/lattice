use super::CADENCE_DAILY;
use super::CADENCE_FIELD;
use super::CADENCE_HOURLY;
use super::CADENCE_RUN_NOW;
use super::CONCURRENCY_DENY;
use super::CONCURRENCY_FIELD;
use super::CONCURRENCY_ISOLATE;
use super::CONCURRENCY_SERIALIZE;
use super::EXECUTION_BACKGROUND;
use super::EXECUTION_FIELD;
use super::EXECUTION_FOREGROUND;
use super::ScheduleError;
use super::parse_value;

/// Scheduled workflow cadence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleCadence {
    /// One immediate run.
    RunNow,
    /// Hourly recurring run.
    Hourly,
    /// Daily recurring run.
    Daily,
}

impl ScheduleCadence {
    /// Parses a stable cadence label.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, ScheduleError> {
        parse_value(CADENCE_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            CADENCE_RUN_NOW => Some(Self::RunNow),
            CADENCE_HOURLY => Some(Self::Hourly),
            CADENCE_DAILY => Some(Self::Daily),
            _ => None,
        }
    }
}

/// Scheduled workflow execution mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleExecutionMode {
    /// Host may run through a background queue.
    Background,
    /// Host may run in an explicit foreground flow.
    Foreground,
}

impl ScheduleExecutionMode {
    /// Parses a stable execution mode label.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, ScheduleError> {
        parse_value(EXECUTION_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            EXECUTION_BACKGROUND => Some(Self::Background),
            EXECUTION_FOREGROUND => Some(Self::Foreground),
            _ => None,
        }
    }
}

/// Concurrency policy for overlapping scheduled runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleConcurrencyPolicy {
    /// Serialize overlapping writes.
    Serialize,
    /// Run in isolated workspace when overlap occurs.
    Isolate,
    /// Deny overlapping writes.
    Deny,
}

impl ScheduleConcurrencyPolicy {
    /// Parses a stable concurrency policy label.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, ScheduleError> {
        parse_value(CONCURRENCY_FIELD, value, Self::from_normalized)
    }

    fn from_normalized(value: &str) -> Option<Self> {
        match value {
            CONCURRENCY_SERIALIZE => Some(Self::Serialize),
            CONCURRENCY_ISOLATE => Some(Self::Isolate),
            CONCURRENCY_DENY => Some(Self::Deny),
            _ => None,
        }
    }
}
