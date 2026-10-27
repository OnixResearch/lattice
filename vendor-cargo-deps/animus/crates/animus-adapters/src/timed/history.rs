use super::{
    Collection, HASH_FIELD, HashInput, OUTCOME_FIELD, PARAMETER_HASH_FIELD, RECIPE_FIELD,
    REDACTION_FIELD, ScheduleError, Text, required_text, text_collection, validate_hash,
    validate_hashes,
};

/// Untrusted scheduled run record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduledRunRecordDraft<'a> {
    /// Schedule identity hash.
    pub schedule_hash: &'a str,
    /// Recipe identity hash.
    pub recipe_hash: &'a str,
    /// Parameter hashes.
    pub parameter_hashes: &'a [&'a str],
    /// Queue or job identity hash.
    pub queue_job_hash: &'a str,
    /// Session ledger hash.
    pub ledger_hash: &'a str,
    /// Retry count used.
    pub retry_count: u32,
    /// Outcome label.
    pub outcome: &'a str,
    /// Safe evidence hashes.
    pub evidence_hashes: &'a [&'a str],
    /// Redaction markers.
    pub redactions: &'a [&'a str],
}

/// Safe scheduled run record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduledRunRecord {
    outcome: Text,
    retry_count: u32,
    parameter_count: u32,
    evidence_count: u32,
    redactions: Collection<Text>,
}

impl ScheduledRunRecord {
    /// Builds a safe scheduled run record.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError`] when hashes, outcome, or redactions are malformed.
    pub fn new(draft: &ScheduledRunRecordDraft<'_>) -> Result<Self, ScheduleError> {
        validate_hash(HashInput {
            field: HASH_FIELD.as_str(),
            value: draft.schedule_hash,
        })?;
        validate_hash(HashInput {
            field: RECIPE_FIELD.as_str(),
            value: draft.recipe_hash,
        })?;
        validate_hashes(PARAMETER_HASH_FIELD, draft.parameter_hashes, true)?;
        validate_hash(HashInput {
            field: HASH_FIELD.as_str(),
            value: draft.queue_job_hash,
        })?;
        validate_hash(HashInput {
            field: HASH_FIELD.as_str(),
            value: draft.ledger_hash,
        })?;
        validate_hashes(HASH_FIELD, draft.evidence_hashes, true)?;
        let outcome = required_text(OUTCOME_FIELD, draft.outcome)?;
        let redactions = text_collection(REDACTION_FIELD, draft.redactions, true)?;
        Ok(Self {
            outcome,
            retry_count: draft.retry_count,
            parameter_count: count_to_u32(PARAMETER_HASH_FIELD, draft.parameter_hashes.len())?,
            evidence_count: count_to_u32(HASH_FIELD, draft.evidence_hashes.len())?,
            redactions,
        })
    }

    /// Returns outcome label.
    #[must_use]
    pub fn outcome(&self) -> &str {
        self.outcome.as_str()
    }

    /// Returns retry count.
    #[must_use]
    pub const fn retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Returns parameter hash count.
    #[must_use]
    pub const fn parameter_count(&self) -> u32 {
        self.parameter_count
    }

    /// Returns evidence hash count.
    #[must_use]
    pub const fn evidence_count(&self) -> u32 {
        self.evidence_count
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redactions(&self) -> &[Text] {
        self.redactions.as_slice()
    }
}

/// Validates rendered scheduled-run record text for raw material.
///
/// # Errors
///
/// Returns [`ScheduleError::RecordLeak`] when raw prompt, secret, path, or tool output appears.
pub fn validate_scheduled_record_text(text: &str) -> Result<(), ScheduleError> {
    let lowered = text.to_ascii_lowercase();
    if contains_record_leak_marker(lowered.as_str()) {
        return Err(ScheduleError::RecordLeak);
    }
    Ok(())
}

fn count_to_u32(
    field_name: crate::validation::FieldName,
    value: usize,
) -> Result<u32, ScheduleError> {
    u32::try_from(value).map_err(|_overflow| ScheduleError::CountOverflow {
        field: field_name.as_str(),
    })
}

fn contains_record_leak_marker(lowered: &str) -> bool {
    if lowered.contains("prompt:") {
        return true;
    }
    if lowered.contains("secret") {
        return true;
    }
    if lowered.contains("/home/") {
        return true;
    }
    lowered.contains("tool output")
}
