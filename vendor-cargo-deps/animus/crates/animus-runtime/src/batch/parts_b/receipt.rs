/// Borrowed batch receipt fields before validation.
pub struct BatchReceiptDraft<'a> {
    /// Receipt run identity.
    pub run_id: &'a str,
    /// Receipt plan hash.
    pub plan_hash: &'a str,
    /// Receipt output format label.
    pub output_format: &'a str,
    /// Receipt redaction policy label.
    pub redaction_policy: &'a str,
    /// Job receipt entries.
    pub jobs: &'a [BatchJobReceiptDraft<'a>],
}

/// Borrowed safe job receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatchJobReceiptDraft<'a> {
    /// Job identity.
    pub job_id: &'a str,
    /// Job status label.
    pub status: &'a str,
    /// Prompt byte count.
    pub prompt_bytes: usize,
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
    /// Redaction marker.
    pub redaction_marker: &'a str,
}

/// Safe batch receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchReceipt {
    run_id: Text,
    plan_hash: Text,
    output_format: BatchOutputFormat,
    redaction_policy: BatchRedactionPolicy,
    jobs: Collection<BatchJobReceipt>,
    receipt_hash: Text,
}

impl BatchReceipt {
    /// Validates safe receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when receipt metadata is malformed or contains raw
    /// prompt, provider payload, credential, full-path, remote destination, or
    /// unbounded-output leak markers.
    pub fn validate(draft: &BatchReceiptDraft<'_>) -> Result<Self, BatchError> {
        let run_id = validate_safe_metadata("receipt_run_id", draft.run_id)?;
        let plan_hash = validate_safe_metadata("receipt_plan_hash", draft.plan_hash)?;
        let output_format = BatchOutputFormat::parse(draft.output_format)?;
        let redaction_policy = BatchRedactionPolicy::parse(draft.redaction_policy)?;
        let jobs = validate_job_receipts(draft.jobs)?;
        let receipt_hash = receipt_hash(ReceiptHashParts {
            run_id: run_id.as_str(),
            plan_hash: plan_hash.as_str(),
            output_format,
            redaction_policy,
            jobs: jobs.as_slice(),
        });
        Ok(Self {
            run_id,
            plan_hash,
            output_format,
            redaction_policy,
            jobs,
            receipt_hash,
        })
    }

    /// Builds safe receipt metadata from a plan and rendered records.
    ///
    /// # Errors
    ///
    /// Returns [`BatchError`] when rendered records do not match planned jobs or
    /// contain unsafe metadata.
    pub fn from_plan_records(
        plan: &BatchPlan,
        records: &[BatchOutputRecord],
    ) -> Result<Self, BatchError> {
        let mut job_drafts = Collection::with_capacity(plan.jobs().len());
        for job in plan.jobs() {
            let record = records
                .iter()
                .find(|candidate| candidate.job_id() == job.job_id())
                .ok_or_else(|| BatchError::MissingOutcome {
                    job_id: Text::from(job.job_id()),
                })?;
            job_drafts.push(BatchJobReceiptDraft {
                job_id: job.job_id(),
                status: record.status().as_str(),
                prompt_bytes: job.prompt_bytes(),
                model_handle: record.model_handle(),
                session_handle: record.session_handle(),
                objective_summary: record.objective_summary(),
                output_label: record.output_label().as_str(),
                output_evidence_hash: record.output_evidence_hash(),
                redaction_marker: record.redaction_marker(),
            });
        }
        Self::validate(&BatchReceiptDraft {
            run_id: plan.identity().run_id(),
            plan_hash: plan.identity().plan_hash(),
            output_format: plan.output_format().as_str(),
            redaction_policy: plan.redaction_policy().as_str(),
            jobs: job_drafts.as_slice(),
        })
    }

    /// Returns the run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    /// Returns the plan hash.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }

    /// Returns the output format.
    #[must_use]
    pub const fn output_format(&self) -> BatchOutputFormat {
        self.output_format
    }

    /// Returns the redaction policy.
    #[must_use]
    pub const fn redaction_policy(&self) -> BatchRedactionPolicy {
        self.redaction_policy
    }

    /// Returns safe job receipts.
    #[must_use]
    pub fn jobs(&self) -> &[BatchJobReceipt] {
        self.jobs.as_slice()
    }

    /// Returns the deterministic receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Safe batch job receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchJobReceipt {
    job_id: Text,
    status: BatchJobStatus,
    prompt_bytes: usize,
    model_handle: Option<Text>,
    session_handle: Option<Text>,
    objective_summary: Text,
    output_label: BatchOutputLabel,
    output_evidence_hash: Text,
    redaction_marker: Text,
}

impl BatchJobReceipt {
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

    /// Returns the prompt byte count.
    #[must_use]
    pub const fn prompt_bytes(&self) -> usize {
        self.prompt_bytes
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

