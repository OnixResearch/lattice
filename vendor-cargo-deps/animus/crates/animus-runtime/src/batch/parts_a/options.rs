/// Safe output label for rendered records and receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchOutputLabel {
    value: Text,
}

impl BatchOutputLabel {
    /// Builds a safe output label.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when the label is empty, malformed, or resembles a
    /// remote destination.
    pub fn new(value: &str) -> Result<Self, BatchError> {
        Ok(Self {
            value: validate_safe_label(OUTPUT_LABEL_FIELD, value)?,
        })
    }

    /// Returns the safe output label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

/// Batch output format.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchOutputFormat {
    /// Newline-delimited JSON records.
    Jsonl,
    /// Evaluation-oriented newline-delimited JSON records.
    EvalJsonl,
    /// ShareGPT-style conversation records.
    ShareGpt,
}

impl BatchOutputFormat {
    fn parse(value: &str) -> Result<Self, BatchError> {
        match value {
            FORMAT_JSONL => Ok(Self::Jsonl),
            FORMAT_EVAL_JSONL => Ok(Self::EvalJsonl),
            FORMAT_SHARE_GPT => Ok(Self::ShareGpt),
            _unsupported => Err(unsupported_value(OUTPUT_FORMAT_FIELD, value)),
        }
    }

    /// Returns the canonical output format label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Jsonl => FORMAT_JSONL,
            Self::EvalJsonl => FORMAT_EVAL_JSONL,
            Self::ShareGpt => FORMAT_SHARE_GPT,
        }
    }
}

/// Batch concurrency policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatchConcurrencyPolicy {
    limit: u16,
}

impl BatchConcurrencyPolicy {
    /// Builds a concurrency policy.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when the limit is below the minimum accepted
    /// concurrency.
    pub const fn new(limit: u16) -> Result<Self, BatchError> {
        if limit < MIN_CONCURRENCY_LIMIT {
            return Err(BatchError::InvalidConcurrency {
                field: CONCURRENCY_LIMIT_FIELD,
                limit,
            });
        }
        Ok(Self { limit })
    }

    /// Returns the maximum number of concurrent jobs.
    #[must_use]
    pub const fn limit(self) -> u16 {
        self.limit
    }
}

/// Batch execution mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchExecutionMode {
    /// Execute through a headless automation service.
    Headless,
    /// Execute through a native session service.
    Session,
}

impl BatchExecutionMode {
    fn parse(value: &str) -> Result<Self, BatchError> {
        match value {
            MODE_HEADLESS => Ok(Self::Headless),
            MODE_SESSION => Ok(Self::Session),
            _unsupported => Err(unsupported_value(EXECUTION_MODE_FIELD, value)),
        }
    }

    /// Returns the canonical execution mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Headless => MODE_HEADLESS,
            Self::Session => MODE_SESSION,
        }
    }
}

/// Batch receipt redaction policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchRedactionPolicy {
    /// Receipts include safe labels and hashes only.
    HashOnly,
    /// Receipts include bounded safe metadata and hashes.
    MetadataOnly,
}

impl BatchRedactionPolicy {
    fn parse(value: &str) -> Result<Self, BatchError> {
        match value {
            REDACTION_HASH_ONLY => Ok(Self::HashOnly),
            REDACTION_METADATA_ONLY => Ok(Self::MetadataOnly),
            _unsupported => Err(unsupported_value(REDACTION_POLICY_FIELD, value)),
        }
    }

    /// Returns the canonical redaction policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HashOnly => REDACTION_HASH_ONLY,
            Self::MetadataOnly => REDACTION_METADATA_ONLY,
        }
    }
}

/// Injected batch execution service.
pub trait BatchExecutionService {
    /// Executes one planned job through host-owned automation.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when the host service rejects or cannot complete
    /// the typed job.
    fn execute(&mut self, job: &BatchJob) -> Result<BatchJobOutcome, BatchError>;
}

/// Executes every planned job through an injected service.
///
/// # Errors
///
/// Returns [`BatchError::ExecutionServiceUnavailable`] when no service is
/// supplied, or propagates a typed service error.
pub fn execute_batch_plan(
    plan: &BatchPlan,
    service: Option<&mut dyn BatchExecutionService>,
) -> Result<Collection<BatchJobOutcome>, BatchError> {
    let executor = service.ok_or(BatchError::ExecutionServiceUnavailable {
        reason: MISSING_SERVICE_REASON,
    })?;
    let mut outcomes = Collection::with_capacity(plan.jobs().len());
    for job in plan.jobs() {
        outcomes.push(executor.execute(job)?);
    }
    Ok(outcomes)
}

