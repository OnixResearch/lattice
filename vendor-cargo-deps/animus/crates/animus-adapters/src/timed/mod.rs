//! Data-only scheduled recipe workflow DTOs and validation helpers.

#![allow(non_trait_imports)]

const ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("schedule_id");
const RECIPE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("recipe_id");
const HASH_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("schedule_hash");
const PARAMETER_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("parameter_hash");
const CADENCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("cadence");
const CALENDAR_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("calendar_policy");
const EXECUTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("execution_mode");
const CONCURRENCY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("concurrency_policy");
const WORKSPACE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("workspace_scope");
const OUTCOME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("outcome");
const REDACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("redaction");
const ACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("control_action");

const CADENCE_RUN_NOW: &str = "run_now";
const CADENCE_HOURLY: &str = "hourly";
const CADENCE_DAILY: &str = "daily";
const EXECUTION_BACKGROUND: &str = "background";
const EXECUTION_FOREGROUND: &str = "foreground";
const CONCURRENCY_SERIALIZE: &str = "serialize";
const CONCURRENCY_ISOLATE: &str = "isolate";
const CONCURRENCY_DENY: &str = "deny";
const CONTROL_PAUSE: &str = "pause";
const CONTROL_RESUME: &str = "resume";
const CONTROL_EDIT: &str = "edit";
const CONTROL_RUN_NOW: &str = "run_now";
const CONTROL_CANCEL: &str = "cancel";
const BLAKE3_HEX_CHARS: usize = 64;
const ASCII_HEX_RADIX: u32 = 16;
const MIN_RETENTION_COUNT: u32 = 1;
const MIN_INTERVAL_TICKS: u64 = 1;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

/// Effects that scheduled recipe validation never performs in reusable core.
pub const PROHIBITED_SCHEDULE_EFFECTS: &[&str] = &[
    "timer start",
    "daemon startup",
    "cron edit",
    "process launch",
    "filesystem mutation",
    "environment read",
];

/// Scheduled recipe validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScheduleError {
    /// Generic adapter contract validation failed.
    Contract(crate::AdapterContractError),
    /// A schedule field contained an unsupported value.
    UnsupportedValue {
        /// Boundary field name.
        field: &'static str,
        /// Unsupported normalized value.
        value: Text,
    },
    /// A required collection was empty.
    EmptyCollection {
        /// Boundary field name.
        field: &'static str,
    },
    /// A collection contained duplicate values.
    DuplicateValue {
        /// Boundary field name.
        field: &'static str,
        /// Duplicated value.
        value: Text,
    },
    /// A collection count overflowed the public DTO bound.
    CountOverflow {
        /// Boundary field name.
        field: &'static str,
    },
    /// A BLAKE3 hash was malformed.
    InvalidHash {
        /// Boundary field name.
        field: &'static str,
    },
    /// Recipe trust was missing or pending.
    UntrustedRecipe,
    /// Retry or retention bounds were invalid.
    InvalidBounds,
    /// Next-run planning received an invalid interval.
    InvalidPlanningInput,
    /// Run admission denied execution.
    RunDenied {
        /// Denial reason.
        reason: ScheduledRunDenial,
    },
    /// Control request lacked host admission or confirmation.
    ControlDenied,
    /// Rendered record leaked raw material.
    RecordLeak,
}

impl From<crate::AdapterContractError> for ScheduleError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

mod command;
mod descriptor;
mod gate;
mod history;
mod kind;
mod overlap;
mod tick;

#[cfg(test)]
mod tests;

pub use command::{ScheduleControlAction, ScheduleControlDraft, ScheduleControlRequest};
pub use descriptor::{ScheduledWorkflow, ScheduledWorkflowDraft};
pub use gate::{
    ScheduledRunAdmissionDraft, ScheduledRunAdmissionStatus, ScheduledRunDenial,
    ScheduledRunPolicyAdmission, ScheduledRunRuntimeAdmission, admit_scheduled_run,
};
pub use history::{ScheduledRunRecord, ScheduledRunRecordDraft, validate_scheduled_record_text};
pub use kind::{ScheduleCadence, ScheduleConcurrencyPolicy, ScheduleExecutionMode};
pub use overlap::{
    ScheduledConcurrencyDecision, ScheduledConcurrencyInput, evaluate_scheduled_concurrency,
};
pub use tick::{NextRunInput, NextRunPlan, plan_next_run};
fn parse_value<T>(
    field_name: crate::validation::FieldName,
    value: &str,
    parser: impl FnOnce(&str) -> Option<T>,
) -> Result<T, ScheduleError> {
    let normalized = crate::validation::normalized_adapter_field(field_name, value)?;
    parser(normalized.as_str()).ok_or(ScheduleError::UnsupportedValue {
        field: field_name.as_str(),
        value: normalized,
    })
}

fn required_text(
    field_name: crate::validation::FieldName,
    value: &str,
) -> Result<Text, ScheduleError> {
    crate::validation::normalized_adapter_field(field_name, value).map_err(ScheduleError::from)
}

fn text_collection(
    field_name: crate::validation::FieldName,
    values: &[&str],
    required: bool,
) -> Result<Collection<Text>, ScheduleError> {
    if required && values.is_empty() {
        return Err(ScheduleError::EmptyCollection {
            field: field_name.as_str(),
        });
    }
    let mut normalized: Collection<Text> = Collection::with_capacity(values.len());
    for value in values {
        let text = required_text(field_name, value)?;
        if normalized.iter().any(|existing| existing == &text) {
            return Err(ScheduleError::DuplicateValue {
                field: field_name.as_str(),
                value: text,
            });
        }
        normalized.push(text);
    }
    Ok(normalized)
}

fn validate_hashes(
    field_name: crate::validation::FieldName,
    values: &[&str],
    required: bool,
) -> Result<(), ScheduleError> {
    assert!(
        !field_name.as_str().is_empty(),
        "hash field names are stable"
    );
    assert!(
        field_name.as_str().len() <= crate::validation::MAX_ADAPTER_FIELD_CHARS,
        "hash field names are bounded"
    );
    if required && values.is_empty() {
        return Err(ScheduleError::EmptyCollection {
            field: field_name.as_str(),
        });
    }
    let mut seen: Collection<Text> = Collection::with_capacity(values.len());
    for value in values {
        let text = required_text(field_name, value)?;
        validate_hash(HashInput {
            field: field_name.as_str(),
            value: text.as_str(),
        })?;
        if seen.iter().any(|existing| existing == &text) {
            return Err(ScheduleError::DuplicateValue {
                field: field_name.as_str(),
                value: text,
            });
        }
        seen.push(text);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct HashInput<'a> {
    field: &'static str,
    value: &'a str,
}

fn validate_hash(input: HashInput<'_>) -> Result<(), ScheduleError> {
    let hash_len = input.value.chars().count();
    if hash_len != BLAKE3_HEX_CHARS {
        return Err(ScheduleError::InvalidHash { field: input.field });
    }
    if input
        .value
        .chars()
        .any(|character| character.to_digit(ASCII_HEX_RADIX).is_none() || character.is_uppercase())
    {
        return Err(ScheduleError::InvalidHash { field: input.field });
    }
    Ok(())
}
