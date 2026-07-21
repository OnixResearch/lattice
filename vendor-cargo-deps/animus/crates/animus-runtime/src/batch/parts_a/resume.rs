/// Borrowed resume manifest fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResumeManifestDraft<'a> {
    /// Manifest run identity.
    pub run_id: &'a str,
    /// Manifest plan hash.
    pub plan_hash: &'a str,
    /// Manifest output format label.
    pub output_format: &'a str,
    /// Manifest redaction policy label.
    pub redaction_policy: &'a str,
    /// Completed job evidence entries.
    pub completed_jobs: &'a [ResumeCompletedJobDraft<'a>],
}

/// Borrowed completed-job resume evidence before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResumeCompletedJobDraft<'a> {
    /// Completed job identity.
    pub job_id: &'a str,
    /// Job plan hash recorded in the manifest.
    pub plan_hash: &'a str,
    /// Job output format label recorded in the manifest.
    pub output_format: &'a str,
    /// Job redaction policy label recorded in the manifest.
    pub redaction_policy: &'a str,
    /// Job output evidence hash recorded in the manifest.
    pub output_evidence_hash: &'a str,
    /// Resume status label.
    pub status: &'a str,
}

/// Resume decision for every job in a plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchResumePlan {
    decisions: Collection<BatchResumeDecision>,
}

impl BatchResumePlan {
    /// Returns per-job resume decisions in plan order.
    #[must_use]
    pub fn decisions(&self) -> &[BatchResumeDecision] {
        self.decisions.as_slice()
    }
}

/// Per-job resume decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchResumeDecision {
    job_id: Text,
    action: BatchResumeAction,
    stale_reason: Option<ResumeStaleReason>,
}

impl BatchResumeDecision {
    /// Returns the job identity.
    #[must_use]
    pub fn job_id(&self) -> &str {
        self.job_id.as_str()
    }

    /// Returns the resume action.
    #[must_use]
    pub const fn action(&self) -> BatchResumeAction {
        self.action
    }

    /// Returns why a manifest entry was not accepted, when applicable.
    #[must_use]
    pub const fn stale_reason(&self) -> Option<ResumeStaleReason> {
        self.stale_reason
    }
}

/// Resume action for a planned job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BatchResumeAction {
    /// Skip the job because manifest evidence matches.
    Skip,
    /// Run the job because evidence is missing or stale.
    Run,
}

/// Reason resume evidence was not accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResumeStaleReason {
    /// Manifest run identity differs from the current plan.
    RunIdMismatch,
    /// Manifest plan hash differs from the current plan.
    PlanHashMismatch,
    /// Manifest output format differs from the current plan.
    OutputFormatMismatch,
    /// Manifest redaction policy differs from the current plan.
    RedactionPolicyMismatch,
    /// No completed entry exists for the job.
    MissingCompletedEvidence,
    /// Entry status is not completed.
    JobNotCompleted,
    /// Entry output evidence hash differs from expected plan evidence.
    OutputEvidenceMismatch,
}

/// Evaluates resume evidence without executing jobs.
///
/// # Errors
///
/// Returns [`BatchError`] when manifest labels are malformed or repeated.
pub fn evaluate_resume_manifest(
    plan: &BatchPlan,
    manifest: &ResumeManifestDraft<'_>,
) -> Result<BatchResumePlan, BatchError> {
    let manifest_run_id = validate_safe_metadata("manifest_run_id", manifest.run_id)?;
    let manifest_plan_hash = validate_safe_metadata("manifest_plan_hash", manifest.plan_hash)?;
    let manifest_format = BatchOutputFormat::parse(manifest.output_format)?;
    let manifest_redaction = BatchRedactionPolicy::parse(manifest.redaction_policy)?;
    let entries = validate_resume_entries(manifest.completed_jobs)?;
    let mut decisions = Collection::with_capacity(plan.jobs().len());
    for job in plan.jobs() {
        let reason = header_stale_reason(ResumeHeaderInput {
            plan,
            run_id: manifest_run_id.as_str(),
            plan_hash: manifest_plan_hash.as_str(),
            output_format: manifest_format,
            redaction_policy: manifest_redaction,
        })
        .or_else(|| entry_stale_reason(plan, job, entries.as_slice()));
        let action = if reason.is_some() {
            BatchResumeAction::Run
        } else {
            BatchResumeAction::Skip
        };
        decisions.push(BatchResumeDecision {
            job_id: Text::from(job.job_id()),
            action,
            stale_reason: reason,
        });
    }
    Ok(BatchResumePlan { decisions })
}

