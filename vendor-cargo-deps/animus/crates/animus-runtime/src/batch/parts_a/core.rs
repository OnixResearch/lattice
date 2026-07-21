/// Borrowed batch request fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatchRequestDraft<'a> {
    /// Stable caller-visible run label.
    pub run_label: &'a str,
    /// Local input location label supplied by the shell.
    pub source_location: &'a str,
    /// Local output location label supplied by the shell.
    pub output_location: &'a str,
    /// Output format label.
    pub output_format: &'a str,
    /// Maximum number of concurrent jobs.
    pub concurrency_limit: u16,
    /// Execution mode label.
    pub execution_mode: &'a str,
    /// Receipt redaction policy label.
    pub redaction_policy: &'a str,
    /// Borrowed job drafts.
    pub jobs: &'a [BatchJobDraft<'a>],
}

/// Borrowed batch job fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatchJobDraft<'a> {
    /// Stable job identity.
    pub job_id: &'a str,
    /// Prompt body sent only to the injected execution service.
    pub prompt: &'a str,
    /// Safe objective label for receipts.
    pub objective_label: &'a str,
    /// Safe output label for rendered records and receipts.
    pub output_label: &'a str,
    /// Optional safe model handle supplied by the shell.
    pub model_handle: Option<&'a str>,
    /// Optional safe session handle supplied by the shell.
    pub session_handle: Option<&'a str>,
}

/// Validated batch plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchPlan {
    identity: BatchRunIdentity,
    source_location: Text,
    output_location: Text,
    output_format: BatchOutputFormat,
    concurrency: BatchConcurrencyPolicy,
    execution_mode: BatchExecutionMode,
    redaction_policy: BatchRedactionPolicy,
    jobs: Collection<BatchJob>,
}

impl BatchPlan {
    /// Validates a batch request into a deterministic local-only plan.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when required fields are empty, locations are
    /// remote, labels are malformed, jobs are empty or duplicated, or enum-like
    /// fields are unsupported.
    pub fn validate(draft: &BatchRequestDraft<'_>) -> Result<Self, BatchError> {
        let run_label = validate_safe_label(RUN_LABEL_FIELD, draft.run_label)?;
        let source_location =
            validate_local_location(SOURCE_LOCATION_FIELD, draft.source_location)?;
        let output_location =
            validate_local_location(OUTPUT_LOCATION_FIELD, draft.output_location)?;
        let output_format = BatchOutputFormat::parse(draft.output_format)?;
        let concurrency = BatchConcurrencyPolicy::new(draft.concurrency_limit)?;
        let execution_mode = BatchExecutionMode::parse(draft.execution_mode)?;
        let redaction_policy = BatchRedactionPolicy::parse(draft.redaction_policy)?;
        let jobs = validate_jobs(draft.jobs)?;
        let plan_hash = plan_hash(PlanHashParts {
            source_location: source_location.as_str(),
            output_location: output_location.as_str(),
            output_format,
            concurrency,
            execution_mode,
            redaction_policy,
            jobs: jobs.as_slice(),
        });
        let run_id = hash_parts(&[BATCH_RUN_SCHEMA, run_label.as_str(), plan_hash.as_str()]);
        let identity = BatchRunIdentity::new(run_label, run_id, plan_hash);
        Ok(Self {
            identity,
            source_location,
            output_location,
            output_format,
            concurrency,
            execution_mode,
            redaction_policy,
            jobs,
        })
    }

    /// Returns deterministic run identity.
    #[must_use]
    pub const fn identity(&self) -> &BatchRunIdentity {
        &self.identity
    }

    /// Returns the local source location label.
    #[must_use]
    pub fn source_location(&self) -> &str {
        self.source_location.as_str()
    }

    /// Returns the local output location label.
    #[must_use]
    pub fn output_location(&self) -> &str {
        self.output_location.as_str()
    }

    /// Returns the output format.
    #[must_use]
    pub const fn output_format(&self) -> BatchOutputFormat {
        self.output_format
    }

    /// Returns the concurrency policy.
    #[must_use]
    pub const fn concurrency(&self) -> BatchConcurrencyPolicy {
        self.concurrency
    }

    /// Returns the execution mode.
    #[must_use]
    pub const fn execution_mode(&self) -> BatchExecutionMode {
        self.execution_mode
    }

    /// Returns the redaction policy.
    #[must_use]
    pub const fn redaction_policy(&self) -> BatchRedactionPolicy {
        self.redaction_policy
    }

    /// Returns ordered jobs in deterministic plan order.
    #[must_use]
    pub fn jobs(&self) -> &[BatchJob] {
        self.jobs.as_slice()
    }

    /// Returns expected output evidence hash for a job in this plan.
    #[must_use]
    pub fn expected_output_evidence_hash(&self, job: &BatchJob) -> Text {
        hash_parts(&[
            BATCH_OUTPUT_SCHEMA,
            self.identity.plan_hash(),
            job.job_hash(),
            self.output_format.as_str(),
            self.redaction_policy.as_str(),
            job.output_label().as_str(),
        ])
    }
}

/// Deterministic run identity for a batch plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchRunIdentity {
    label: Text,
    run_id: Text,
    plan_hash: Text,
}

impl BatchRunIdentity {
    fn new(label: Text, run_id: Text, plan_hash: Text) -> Self {
        Self {
            label,
            run_id,
            plan_hash,
        }
    }

    /// Returns the safe run label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the deterministic run id.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    /// Returns the deterministic plan hash.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }
}

/// Validated batch job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchJob {
    id: Text,
    prompt: Text,
    objective_label: Text,
    output_label: BatchOutputLabel,
    model_handle: Option<Text>,
    session_handle: Option<Text>,
    prompt_bytes: usize,
    job_hash: Text,
}

impl BatchJob {
    /// Returns the job identity.
    #[must_use]
    pub fn job_id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns the raw prompt for the injected execution service only.
    #[must_use]
    pub fn prompt(&self) -> &str {
        self.prompt.as_str()
    }

    /// Returns prompt byte length for safe metadata.
    #[must_use]
    pub const fn prompt_bytes(&self) -> usize {
        self.prompt_bytes
    }

    /// Returns the safe objective label.
    #[must_use]
    pub fn objective_label(&self) -> &str {
        self.objective_label.as_str()
    }

    /// Returns the safe output label.
    #[must_use]
    pub const fn output_label(&self) -> &BatchOutputLabel {
        &self.output_label
    }

    /// Returns the safe model handle, when supplied.
    #[must_use]
    pub fn model_handle(&self) -> Option<&str> {
        self.model_handle.as_deref()
    }

    /// Returns the safe session handle, when supplied.
    #[must_use]
    pub fn session_handle(&self) -> Option<&str> {
        self.session_handle.as_deref()
    }

    /// Returns the deterministic job hash.
    #[must_use]
    pub fn job_hash(&self) -> &str {
        self.job_hash.as_str()
    }
}

