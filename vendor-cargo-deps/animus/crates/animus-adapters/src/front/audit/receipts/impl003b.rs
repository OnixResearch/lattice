
fn project_work_summary(draft: &AuditWorkDraft, public_pending_limit: u32) -> AuditWorkSummary {
    let pending_public_count = core::cmp::min(draft.pending_count, public_pending_limit);
    AuditWorkSummary {
        kind: draft.kind,
        status: status_for_counts(
            draft.pending_count,
            draft.completed_count,
            draft.failed_count,
        ),
        pending_public_count,
        pending_total_count: draft.pending_count,
        completed_count: draft.completed_count,
        failed_count: draft.failed_count,
        timing_bucket: draft.timing_bucket,
        pending_over_limit: draft.pending_count > public_pending_limit,
    }
}

const fn status_for_counts(
    pending_count: u32,
    completed_count: u32,
    failed_count: u32,
) -> AuditStatusClass {
    if failed_count > ZERO_AUDIT_COUNT {
        return AuditStatusClass::Failed;
    }
    if pending_count > ZERO_AUDIT_COUNT {
        return AuditStatusClass::Pending;
    }
    if completed_count > ZERO_AUDIT_COUNT {
        return AuditStatusClass::Completed;
    }
    AuditStatusClass::Idle
}

fn summarize_failures(failures: &[AuditFailureDraft]) -> Vec<AuditFailureSummary> {
    let mut summaries: Vec<AuditFailureSummary> = Vec::with_capacity(failures.len());
    for draft in failures {
        if draft.count == ZERO_AUDIT_COUNT {
            continue;
        }
        if let Some(summary) = summaries
            .iter_mut()
            .find(|summary| summary.failure_class == draft.failure_class)
        {
            summary.count = summary.count.saturating_add(draft.count);
            continue;
        }
        summaries.push(AuditFailureSummary {
            failure_class: draft.failure_class,
            count: draft.count,
        });
    }
    summaries
}

/// Borrowed audit receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuditReceiptDraft<'a> {
    /// Safe projected snapshot.
    pub snapshot: &'a AuditSnapshot,
    /// Raw prompt text, which is forbidden.
    pub raw_prompt: Option<&'a str>,
    /// Tool arguments, which are forbidden.
    pub tool_arguments: Option<&'a str>,
    /// Tool output, which is forbidden.
    pub tool_output: Option<&'a str>,
    /// Provider payload, which is forbidden.
    pub provider_payload: Option<&'a str>,
    /// Credential-looking value, which is forbidden.
    pub credential: Option<&'a str>,
    /// Authorization header, which is forbidden.
    pub authorization_header: Option<&'a str>,
    /// Environment value, which is forbidden.
    pub environment_value: Option<&'a str>,
    /// Full host path, which is forbidden.
    pub full_path: Option<&'a str>,
    /// Unbounded logs, which are forbidden.
    pub unbounded_log: Option<&'a str>,
}

/// Safe audit receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditReceipt {
    snapshot: AuditSnapshot,
    receipt_hash: crate::artifact::Hash,
}

impl AuditReceipt {
    /// Validates a safe audit receipt and computes its BLAKE3 receipt hash.
    ///
    /// # Errors
    ///
    /// Returns [`AuditReceiptError`] when forbidden sensitive receipt fields are present.
    // r[impl agent_harness_runtime.audit_receipts.receipts]
    pub fn validate(draft: &AuditReceiptDraft<'_>) -> Result<Self, AuditReceiptError> {
        reject_receipt_leaks(draft)?;
        Ok(Self {
            snapshot: draft.snapshot.clone(),
            receipt_hash: receipt_hash(draft.snapshot),
        })
    }

    /// Returns the safe audit snapshot.
    #[must_use]
    pub const fn snapshot(&self) -> &AuditSnapshot {
        &self.snapshot
    }

    /// Returns the BLAKE3 receipt hash.
    #[must_use]
    pub const fn receipt_hash(&self) -> &crate::artifact::Hash {
        &self.receipt_hash
    }
}

fn reject_receipt_leaks(draft: &AuditReceiptDraft<'_>) -> Result<(), AuditReceiptError> {
    if draft.raw_prompt.is_some() {
        return Err(leak(AuditReceiptLeak::RawPrompt));
    }
    if draft.tool_arguments.is_some() {
        return Err(leak(AuditReceiptLeak::ToolArguments));
    }
    if draft.tool_output.is_some() {
        return Err(leak(AuditReceiptLeak::ToolOutput));
    }
    if draft.provider_payload.is_some() {
        return Err(leak(AuditReceiptLeak::ProviderPayload));
    }
    if draft.credential.is_some() {
        return Err(leak(AuditReceiptLeak::Credential));
    }
    if draft.authorization_header.is_some() {
        return Err(leak(AuditReceiptLeak::AuthorizationHeader));
    }
    if draft.environment_value.is_some() {
        return Err(leak(AuditReceiptLeak::EnvironmentValue));
    }
    if draft.full_path.is_some() {
        return Err(leak(AuditReceiptLeak::FullPath));
    }
    if draft.unbounded_log.is_some() {
        return Err(leak(AuditReceiptLeak::UnboundedLog));
    }
    Ok(())
}

const fn leak(leak: AuditReceiptLeak) -> AuditReceiptError {
    AuditReceiptError::ReceiptLeak { leak }
}

fn receipt_hash(snapshot: &AuditSnapshot) -> crate::artifact::Hash {
    let mut canonical = String::from(CANONICAL_VERSION_PREFIX);
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(&snapshot.schema_version().get().to_string());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(snapshot.policy_hash().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(snapshot.window_id());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(snapshot.event_window_hash().as_str());
    for summary in snapshot.work() {
        append_work_summary(&mut canonical, summary);
    }
    for summary in snapshot.failures() {
        append_failure_summary(&mut canonical, summary);
    }
    let digest = blake3::hash(canonical.as_bytes());
    crate::artifact::Hash::from_bytes(digest.as_bytes())
}

fn append_work_summary(canonical: &mut String, summary: &AuditWorkSummary) {
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(summary.kind().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(summary.status().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(&summary.pending_public_count().to_string());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(&summary.pending_total_count().to_string());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(&summary.completed_count().to_string());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(&summary.failed_count().to_string());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(summary.timing_bucket().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(if summary.pending_over_limit() {
        "over-limit"
    } else {
        "within-limit"
    });
}

fn append_failure_summary(canonical: &mut String, summary: &AuditFailureSummary) {
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(summary.failure_class().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(&summary.count().to_string());
}

/// Audit receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditReceiptLeak {
    /// Raw prompt text was present.
    RawPrompt,
    /// Tool arguments were present.
    ToolArguments,
    /// Tool output was present.
    ToolOutput,
    /// Provider payload was present.
    ProviderPayload,
    /// Credential-looking value was present.
    Credential,
    /// Authorization header was present.
    AuthorizationHeader,
    /// Environment value was present.
    EnvironmentValue,
    /// Full host path was present.
    FullPath,
    /// Unbounded log material was present.
    UnboundedLog,
}

/// Audit projection or receipt validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditReceiptError {
    /// A safe text field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 hash failed validation.
    InvalidHash(crate::artifact::Error),
    /// Audit schema version was zero.
    ZeroSchemaVersion,
    /// A bounded item list exceeded the allowed item count.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum supported items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// Audit receipt contained forbidden sensitive material.
    ReceiptLeak {
        /// Leak class.
        leak: AuditReceiptLeak,
    },
}

impl core::fmt::Display for AuditReceiptError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidHash(error) => write!(formatter, "{error}"),
            Self::ZeroSchemaVersion => formatter.write_str("audit schema version must be non-zero"),
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} supports at most {max_items} items; got {actual_items}"
            ),
            Self::ReceiptLeak { leak } => write!(formatter, "audit receipt leak: {leak:?}"),
        }
    }
}

impl From<crate::AdapterContractError> for AuditReceiptError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

fn validate_item_count(
    field: &'static str,
    actual_items: usize,
    max_items: usize,
) -> Result<(), AuditReceiptError> {
    if actual_items > max_items {
        return Err(AuditReceiptError::TooManyItems {
            field,
            max_items,
            actual_items,
        });
    }
    Ok(())
}

fn normalize(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<String, AuditReceiptError> {
    crate::validation::normalized_adapter_field(field, value).map_err(AuditReceiptError::Contract)
}

