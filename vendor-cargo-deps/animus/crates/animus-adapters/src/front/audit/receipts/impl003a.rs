
impl AuditWindow {
    /// Validates event window identity.
    ///
    /// # Errors
    ///
    /// Returns [`AuditReceiptError`] when the safe window label or hash is invalid.
    pub fn validate(draft: &AuditWindowDraft<'_>) -> Result<Self, AuditReceiptError> {
        Ok(Self {
            window_id: normalize(WINDOW_ID_FIELD, draft.window_id)?,
            event_window_hash: crate::artifact::Hash::new(draft.event_window_hash)
                .map_err(AuditReceiptError::InvalidHash)?,
        })
    }

    /// Returns the safe event window identifier.
    #[must_use]
    pub fn window_id(&self) -> &str {
        self.window_id.as_str()
    }

    /// Returns the event window hash.
    #[must_use]
    pub const fn event_window_hash(&self) -> &crate::artifact::Hash {
        &self.event_window_hash
    }
}

/// Host-provided bounded work counts before projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditWorkDraft {
    /// Work kind.
    pub kind: AuditWorkKind,
    /// Internal pending count. Only the clamped public count is exposed.
    pub pending_count: u32,
    /// Completed count.
    pub completed_count: u32,
    /// Failed count.
    pub failed_count: u32,
    /// Host-summarized timing bucket.
    pub timing_bucket: AuditTimingBucket,
}

/// Host-provided failure class count before projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditFailureDraft {
    /// Failure class.
    pub failure_class: AuditFailureClass,
    /// Count for the failure class.
    pub count: u32,
}

/// Borrowed audit projection inputs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditProjectionInput<'a> {
    /// Projection policy.
    pub policy: &'a AuditProjectionPolicy,
    /// Event window identity.
    pub window: &'a AuditWindow,
    /// Safe work count drafts.
    pub work: &'a [AuditWorkDraft],
    /// Safe failure count drafts.
    pub failures: &'a [AuditFailureDraft],
}

/// Projected work summary with clamped pending counts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditWorkSummary {
    kind: AuditWorkKind,
    status: AuditStatusClass,
    pending_public_count: u32,
    pending_total_count: u32,
    completed_count: u32,
    failed_count: u32,
    timing_bucket: AuditTimingBucket,
    pending_over_limit: bool,
}

impl AuditWorkSummary {
    /// Returns the work kind.
    #[must_use]
    pub const fn kind(&self) -> AuditWorkKind {
        self.kind
    }

    /// Returns the safe status class.
    #[must_use]
    pub const fn status(&self) -> AuditStatusClass {
        self.status
    }

    /// Returns the clamped public pending count.
    #[must_use]
    pub const fn pending_public_count(&self) -> u32 {
        self.pending_public_count
    }

    /// Returns the total pending count without private IDs or payloads.
    #[must_use]
    pub const fn pending_total_count(&self) -> u32 {
        self.pending_total_count
    }

    /// Returns the completed count.
    #[must_use]
    pub const fn completed_count(&self) -> u32 {
        self.completed_count
    }

    /// Returns the failed count.
    #[must_use]
    pub const fn failed_count(&self) -> u32 {
        self.failed_count
    }

    /// Returns the host-summarized timing bucket.
    #[must_use]
    pub const fn timing_bucket(&self) -> AuditTimingBucket {
        self.timing_bucket
    }

    /// Returns true when pending count exceeded the public limit.
    #[must_use]
    pub const fn pending_over_limit(&self) -> bool {
        self.pending_over_limit
    }
}

/// Projected failure class summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditFailureSummary {
    failure_class: AuditFailureClass,
    count: u32,
}

impl AuditFailureSummary {
    /// Returns the failure class.
    #[must_use]
    pub const fn failure_class(&self) -> AuditFailureClass {
        self.failure_class
    }

    /// Returns the count for the failure class.
    #[must_use]
    pub const fn count(&self) -> u32 {
        self.count
    }
}

/// Bounded audit snapshot over one event window.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditSnapshot {
    schema_version: NonZeroU16,
    policy_hash: crate::artifact::Hash,
    window_id: String,
    event_window_hash: crate::artifact::Hash,
    work: Vec<AuditWorkSummary>,
    failures: Vec<AuditFailureSummary>,
}

impl AuditSnapshot {
    /// Returns the audit schema version.
    #[must_use]
    pub const fn schema_version(&self) -> NonZeroU16 {
        self.schema_version
    }

    /// Returns the policy identity.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns the safe event window identifier.
    #[must_use]
    pub fn window_id(&self) -> &str {
        self.window_id.as_str()
    }

    /// Returns the event window hash.
    #[must_use]
    pub const fn event_window_hash(&self) -> &crate::artifact::Hash {
        &self.event_window_hash
    }

    /// Returns bounded work summaries.
    #[must_use]
    pub fn work(&self) -> &[AuditWorkSummary] {
        self.work.as_slice()
    }

    /// Returns failure-class summaries.
    #[must_use]
    pub fn failures(&self) -> &[AuditFailureSummary] {
        self.failures.as_slice()
    }
}

/// Projects host-provided safe counts into a bounded audit snapshot.
///
/// # Errors
///
/// Returns [`AuditReceiptError`] when count arrays exceed public adapter bounds.
// r[impl agent_harness_runtime.audit_receipts.snapshots]
// r[impl agent_harness_runtime.audit_receipts.clamping]
// r[impl agent_harness_runtime.audit_receipts.host_owned]
// r[impl agent_harness_runtime.audit_receipts.composition]
pub fn project_audit_snapshot(
    input: AuditProjectionInput<'_>,
) -> Result<AuditSnapshot, AuditReceiptError> {
    validate_item_count("audit_work", input.work.len(), MAX_AUDIT_WORK_SUMMARIES)?;
    validate_item_count(
        "audit_failure",
        input.failures.len(),
        MAX_AUDIT_FAILURE_SUMMARIES,
    )?;

    let mut work = Vec::with_capacity(input.work.len());
    for draft in input.work {
        work.push(project_work_summary(
            draft,
            input.policy.public_pending_limit(),
        ));
    }
    work.sort_by_key(AuditWorkSummary::kind);

    let mut failures = summarize_failures(input.failures);
    failures.sort_by_key(AuditFailureSummary::failure_class);

    Ok(AuditSnapshot {
        schema_version: input.policy.schema_version(),
        policy_hash: input.policy.policy_hash().clone(),
        window_id: String::from(input.window.window_id()),
        event_window_hash: input.window.event_window_hash().clone(),
        work,
        failures,
    })
}
