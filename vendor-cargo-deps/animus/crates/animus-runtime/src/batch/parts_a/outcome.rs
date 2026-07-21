/// Borrowed batch outcome fields before validation.
pub struct BatchJobOutcomeDraft<'a> {
    /// Completed job identity.
    pub job_id: &'a str,
    /// Job status label.
    pub status: &'a str,
    /// Host-observed completion sequence.
    pub completion_sequence: u64,
    /// Optional safe model handle.
    pub model_handle: Option<&'a str>,
    /// Optional safe session handle.
    pub session_handle: Option<&'a str>,
    /// Safe objective summary.
    pub objective_summary: &'a str,
    /// Safe output label.
    pub output_label: &'a str,
    /// Safe output evidence hash.
    pub output_evidence_hash: &'a str,
    /// Redaction marker proving raw payloads were omitted.
    pub redaction_marker: &'a str,
}

/// Validated batch job outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchJobOutcome {
    job_id: Text,
    status: BatchJobStatus,
    completion_sequence: u64,
    model_handle: Option<Text>,
    session_handle: Option<Text>,
    objective_summary: Text,
    output_label: BatchOutputLabel,
    output_evidence_hash: Text,
    redaction_marker: Text,
}

impl BatchJobOutcome {
    /// Validates a host-supplied job outcome.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when identities, status, sequence, safe labels, or
    /// metadata fields are malformed or unsafe.
    pub fn validate(draft: &BatchJobOutcomeDraft<'_>) -> Result<Self, BatchError> {
        let job_id = validate_safe_label(JOB_ID_FIELD, draft.job_id)?;
        let status = BatchJobStatus::parse(draft.status)?;
        if draft.completion_sequence < FIRST_COMPLETION_SEQUENCE {
            return Err(BatchError::InvalidSequence {
                field: COMPLETION_SEQUENCE_FIELD,
            });
        }
        let model_handle = validate_optional_safe_text(MODEL_HANDLE_FIELD, draft.model_handle)?;
        let session_handle =
            validate_optional_safe_text(SESSION_HANDLE_FIELD, draft.session_handle)?;
        let objective_summary =
            validate_safe_metadata(OBJECTIVE_SUMMARY_FIELD, draft.objective_summary)?;
        let output_label = BatchOutputLabel::new(draft.output_label)?;
        let output_evidence_hash =
            validate_safe_metadata(OUTPUT_EVIDENCE_HASH_FIELD, draft.output_evidence_hash)?;
        let redaction_marker =
            validate_safe_metadata(REDACTION_MARKER_FIELD, draft.redaction_marker)?;
        Ok(Self {
            job_id,
            status,
            completion_sequence: draft.completion_sequence,
            model_handle,
            session_handle,
            objective_summary,
            output_label,
            output_evidence_hash,
            redaction_marker,
        })
    }

    /// Returns the job identity.
    #[must_use]
    pub fn job_id(&self) -> &str {
        self.job_id.as_str()
    }

    /// Returns the job status.
    #[must_use]
    pub const fn status(&self) -> BatchJobStatus {
        self.status
    }

    /// Returns the host-observed completion sequence.
    #[must_use]
    pub const fn completion_sequence(&self) -> u64 {
        self.completion_sequence
    }

    /// Returns the safe model handle.
    #[must_use]
    pub fn model_handle(&self) -> Option<&str> {
        self.model_handle.as_deref()
    }

    /// Returns the safe session handle.
    #[must_use]
    pub fn session_handle(&self) -> Option<&str> {
        self.session_handle.as_deref()
    }

    /// Returns the safe objective summary.
    #[must_use]
    pub fn objective_summary(&self) -> &str {
        self.objective_summary.as_str()
    }

    /// Returns the safe output label.
    #[must_use]
    pub const fn output_label(&self) -> &BatchOutputLabel {
        &self.output_label
    }

    /// Returns the output evidence hash.
    #[must_use]
    pub fn output_evidence_hash(&self) -> &str {
        self.output_evidence_hash.as_str()
    }

    /// Returns the redaction marker.
    #[must_use]
    pub fn redaction_marker(&self) -> &str {
        self.redaction_marker.as_str()
    }
}

/// Batch job terminal status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchJobStatus {
    /// Job succeeded.
    Succeeded,
    /// Job failed with safe metadata.
    Failed,
    /// Job was skipped because matching resume evidence was accepted.
    Skipped,
}

impl BatchJobStatus {
    fn parse(value: &str) -> Result<Self, BatchError> {
        match value {
            STATUS_SUCCEEDED => Ok(Self::Succeeded),
            STATUS_FAILED => Ok(Self::Failed),
            STATUS_SKIPPED => Ok(Self::Skipped),
            _unsupported => Err(unsupported_value(STATUS_FIELD, value)),
        }
    }

    /// Returns the canonical status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => STATUS_SUCCEEDED,
            Self::Failed => STATUS_FAILED,
            Self::Skipped => STATUS_SKIPPED,
        }
    }
}

/// Renderer adapter contract over normalized batch output records.
pub trait BatchRenderer {
    /// Renders normalized records into adapter-owned output strings.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when the renderer cannot encode a safe record.
    fn render(&self, records: &[BatchOutputRecord]) -> Result<Collection<Text>, BatchError>;
}

/// Safe normalized output record in deterministic plan order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchOutputRecord {
    job_id: Text,
    status: BatchJobStatus,
    model_handle: Option<Text>,
    session_handle: Option<Text>,
    objective_summary: Text,
    output_label: BatchOutputLabel,
    output_evidence_hash: Text,
    redaction_marker: Text,
}

impl BatchOutputRecord {
    fn from_outcome(outcome: &BatchJobOutcome) -> Self {
        Self {
            job_id: Text::from(outcome.job_id()),
            status: outcome.status(),
            model_handle: outcome.model_handle().map(Text::from),
            session_handle: outcome.session_handle().map(Text::from),
            objective_summary: Text::from(outcome.objective_summary()),
            output_label: outcome.output_label().clone(),
            output_evidence_hash: Text::from(outcome.output_evidence_hash()),
            redaction_marker: Text::from(outcome.redaction_marker()),
        }
    }

    /// Returns the job identity.
    #[must_use]
    pub fn job_id(&self) -> &str {
        self.job_id.as_str()
    }

    /// Returns the job status.
    #[must_use]
    pub const fn status(&self) -> BatchJobStatus {
        self.status
    }

    /// Returns the safe model handle.
    #[must_use]
    pub fn model_handle(&self) -> Option<&str> {
        self.model_handle.as_deref()
    }

    /// Returns the safe session handle.
    #[must_use]
    pub fn session_handle(&self) -> Option<&str> {
        self.session_handle.as_deref()
    }

    /// Returns the safe objective summary.
    #[must_use]
    pub fn objective_summary(&self) -> &str {
        self.objective_summary.as_str()
    }

    /// Returns the safe output label.
    #[must_use]
    pub const fn output_label(&self) -> &BatchOutputLabel {
        &self.output_label
    }

    /// Returns the output evidence hash.
    #[must_use]
    pub fn output_evidence_hash(&self) -> &str {
        self.output_evidence_hash.as_str()
    }

    /// Returns the redaction marker.
    #[must_use]
    pub fn redaction_marker(&self) -> &str {
        self.redaction_marker.as_str()
    }
}

/// Renders outcomes in deterministic plan order.
///
/// # Errors
///
/// Returns [`BatchError`] when an outcome is missing, duplicated, or references
/// a job that is absent from the plan.
pub fn render_plan_order_records(
    plan: &BatchPlan,
    outcomes: &[BatchJobOutcome],
) -> Result<Collection<BatchOutputRecord>, BatchError> {
    reject_unknown_or_duplicate_outcomes(plan, outcomes)?;
    let mut records = Collection::with_capacity(plan.jobs().len());
    for job in plan.jobs() {
        let outcome =
            find_outcome(outcomes, job.job_id()).ok_or_else(|| BatchError::MissingOutcome {
                job_id: Text::from(job.job_id()),
            })?;
        records.push(BatchOutputRecord::from_outcome(outcome));
    }
    Ok(records)
}

