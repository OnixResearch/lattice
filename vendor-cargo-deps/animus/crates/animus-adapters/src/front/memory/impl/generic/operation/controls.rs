#![allow(unknown_lints)]
#![cfg_attr(animus_octet, allow(tigerstyle::excessive_file_length))]

use super::DESCRIPTOR_ID_FIELD;
use super::GenericMemoryRecordKind;
use super::MAX_MEMORY_DESCRIPTORS;
use super::MEMORY_EVIDENCE_FIELD;
use super::MemoryAdmissionStatus;
use super::MemoryConfidence;
use super::MemoryError;
use super::MemoryFreshness;
use super::MemoryInjectionBudget;
use super::MemoryMatchReason;
use super::MemoryReceiptLeak;
use super::MemoryRecordEnvelope;
use super::MemoryRedactionClass;
use super::MemoryRetrievalOutcomeStatus;
use super::MemoryRetrievalPlan;
use super::MemoryTrustScope;
use super::NonZeroU32;
use super::String;
use super::Vec;
use super::memory_leak;
use super::normalize;

const MEMORY_ADAPTER_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_adapter_label");
const ZERO_MEMORY_UNITS: u32 = 0;
const MEMORY_OPERATION_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_operation_label");
const MEMORY_COMMAND_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_command_label");
const MEMORY_TARGET_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_target_id");
const MEMORY_REASON_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("memory_reason_label");

/// Generic memory operation kind used by safe receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryOperationKind {
    /// Source/import operation.
    SourceImport,
    /// Retrieval or search operation.
    Retrieval,
    /// Candidate extraction operation.
    Extraction,
    /// Candidate admission operation.
    Admission,
    /// Durable store operation.
    Store,
    /// Update, delete, tombstone, or supersession operation.
    UpdateDelete,
    /// Prompt influence operation.
    PromptInfluence,
    /// User or host control command.
    Control,
}

impl MemoryOperationKind {
    /// Returns the stable operation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceImport => "source-import",
            Self::Retrieval => "retrieval",
            Self::Extraction => "extraction",
            Self::Admission => "admission",
            Self::Store => "store",
            Self::UpdateDelete => "update-delete",
            Self::PromptInfluence => "prompt-influence",
            Self::Control => "control",
        }
    }
}

/// Generic safe status for memory operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryOperationStatus {
    /// Operation completed.
    Complete,
    /// Operation was denied.
    Denied,
    /// Required adapter or service was unavailable.
    Unavailable,
    /// Required behavior is unsupported.
    Unsupported,
    /// Operation timed out.
    TimedOut,
    /// Operation completed partially.
    Partial,
    /// Operation was stale and not dispatched.
    Stale,
    /// Candidate or operation was rejected.
    Rejected,
}

impl MemoryOperationStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::TimedOut => "timed-out",
            Self::Partial => "partial",
            Self::Stale => "stale",
            Self::Rejected => "rejected",
        }
    }
}

/// Borrowed safe memory operation receipt metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryOperationReceiptDraft<'a> {
    /// Stable operation label.
    pub operation_label: &'a str,
    /// Adapter family label.
    pub adapter_family_label: &'a str,
    /// Operation kind.
    pub operation_kind: MemoryOperationKind,
    /// Optional safe record identity.
    pub record_id: Option<&'a str>,
    /// Optional safe candidate identity.
    pub candidate_id: Option<&'a str>,
    /// Number of records or candidates affected.
    pub item_count: u32,
    /// Operation status.
    pub status: MemoryOperationStatus,
    /// Redaction class.
    pub redaction_class: MemoryRedactionClass,
    /// Lowercase BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Lowercase BLAKE3 body hash.
    pub body_hash: &'a str,
    /// Raw memory body, forbidden in receipts.
    pub raw_memory_body: Option<&'a str>,
    /// Credential-looking value, forbidden in receipts.
    pub credential: Option<&'a str>,
    /// Provider payload, forbidden in receipts.
    pub provider_payload: Option<&'a str>,
    /// Full host path, forbidden in receipts.
    pub full_path: Option<&'a str>,
    /// Hidden memory body, forbidden in receipts.
    pub hidden_memory_body: Option<&'a str>,
    /// Unbounded prompt text, forbidden in receipts.
    pub unbounded_prompt_text: Option<&'a str>,
}

/// Validated safe memory operation receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryOperationReceipt {
    operation_label: String,
    adapter_family_label: String,
    operation_kind: MemoryOperationKind,
    record_id: Option<String>,
    candidate_id: Option<String>,
    item_count: u32,
    status: MemoryOperationStatus,
    redaction_class: MemoryRedactionClass,
    policy_hash: crate::artifact::Hash,
    body_hash: crate::artifact::Hash,
}

impl MemoryOperationReceipt {
    /// Validates safe operation receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when labels or hashes are invalid, or sensitive material is present.
    // r[impl agent_harness_runtime.memory_adapters.receipts]
    pub fn validate(draft: &MemoryOperationReceiptDraft<'_>) -> Result<Self, MemoryError> {
        reject_operation_receipt_leaks(draft)?;
        Ok(Self {
            operation_label: normalize(MEMORY_OPERATION_LABEL_FIELD, draft.operation_label)?,
            adapter_family_label: normalize(MEMORY_ADAPTER_LABEL_FIELD, draft.adapter_family_label)?,
            operation_kind: draft.operation_kind,
            record_id: normalize_optional(MEMORY_TARGET_FIELD, draft.record_id)?,
            candidate_id: normalize_optional(MEMORY_TARGET_FIELD, draft.candidate_id)?,
            item_count: draft.item_count,
            status: draft.status,
            redaction_class: draft.redaction_class,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash).map_err(MemoryError::InvalidHash)?,
            body_hash: crate::artifact::Hash::new(draft.body_hash).map_err(MemoryError::InvalidHash)?,
        })
    }

    /// Returns operation label.
    #[must_use]
    pub fn operation_label(&self) -> &str {
        self.operation_label.as_str()
    }

    /// Returns adapter family label.
    #[must_use]
    pub fn adapter_family_label(&self) -> &str {
        self.adapter_family_label.as_str()
    }

    /// Returns operation kind.
    #[must_use]
    pub const fn operation_kind(&self) -> MemoryOperationKind {
        self.operation_kind
    }

    /// Returns optional record identity.
    #[must_use]
    pub fn record_id(&self) -> Option<&str> {
        self.record_id.as_deref()
    }

    /// Returns optional candidate identity.
    #[must_use]
    pub fn candidate_id(&self) -> Option<&str> {
        self.candidate_id.as_deref()
    }

    /// Returns item count.
    #[must_use]
    pub const fn item_count(&self) -> u32 {
        self.item_count
    }

    /// Returns status.
    #[must_use]
    pub const fn status(&self) -> MemoryOperationStatus {
        self.status
    }

    /// Returns redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> MemoryRedactionClass {
        self.redaction_class
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }
}

/// Adapter-neutral memory control command kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryControlCommandKind {
    /// List safe memory descriptors.
    List,
    /// Show one safe memory descriptor.
    Show,
    /// Search memories through an injected retrieval adapter.
    Search,
    /// Explain why a memory was included or skipped.
    Why,
    /// Approve an extracted candidate.
    Approve,
    /// Reject an extracted candidate.
    Reject,
    /// Forget a memory record.
    Forget,
    /// Update a memory record.
    Update,
    /// Supersede a memory record.
    Supersede,
    /// Export safe summary metadata.
    ExportSummary,
}

impl MemoryControlCommandKind {
    /// Returns the stable command label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Show => "show",
            Self::Search => "search",
            Self::Why => "why",
            Self::Approve => "approve",
            Self::Reject => "reject",
            Self::Forget => "forget",
            Self::Update => "update",
            Self::Supersede => "supersede",
            Self::ExportSummary => "export-summary",
        }
    }

    /// Returns true when the command can mutate memory state.
    #[must_use]
    pub const fn mutates(self) -> bool {
        matches!(self, Self::Approve | Self::Reject | Self::Forget | Self::Update | Self::Supersede)
    }

    /// Returns true when a command requires a target record or candidate.
    #[must_use]
    pub const fn requires_target(self) -> bool {
        matches!(
            self,
            Self::Show | Self::Why | Self::Approve | Self::Reject | Self::Forget | Self::Update | Self::Supersede
        )
    }

    /// Returns true when remote/headless invocation requires admission before dispatch.
    #[must_use]
    pub const fn requires_remote_admission(self) -> bool {
        self.mutates() || matches!(self, Self::Why | Self::ExportSummary)
    }

    /// Returns true when evidence is required before adapter dispatch.
    #[must_use]
    pub const fn requires_evidence(self) -> bool {
        self.mutates() || matches!(self, Self::Why | Self::ExportSummary)
    }
}

/// Remote admission status for headless or automation-triggered memory controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRemoteAdmissionStatus {
    /// Caller is local or already inside a trusted host path.
    NotRemote,
    /// Remote caller was admitted.
    Admitted,
    /// Remote admission is missing.
    Missing,
    /// Remote admission denied the caller.
    Denied,
    /// Remote admission credential or decision expired.
    Expired,
    /// Remote admission input was malformed.
    Malformed,
}

/// Borrowed memory control request metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryControlRequestDraft<'a> {
    /// Safe command label.
    pub command_label: &'a str,
    /// Command kind.
    pub kind: MemoryControlCommandKind,
    /// Scope requested by the command.
    pub scope: Option<MemoryTrustScope>,
    /// Optional target record or candidate identity.
    pub target_id: Option<&'a str>,
    /// Optional evidence handle for mutation or prompt-visible use.
    pub evidence_handle: Option<&'a str>,
    /// Whether policy grants the required capability.
    pub capability_granted: bool,
    /// Remote admission status.
    pub remote_admission: MemoryRemoteAdmissionStatus,
    /// Freshness of the command target.
    pub freshness: MemoryFreshness,
    /// Safe output budget units.
    pub safe_output_budget_units: u32,
    /// Raw memory body, forbidden in control metadata.
    pub raw_memory_body: Option<&'a str>,
    /// Credential-looking value, forbidden in control metadata.
    pub credential: Option<&'a str>,
    /// Provider payload, forbidden in control metadata.
    pub provider_payload: Option<&'a str>,
    /// Full host path, forbidden in control metadata.
    pub full_path: Option<&'a str>,
}

/// Validated adapter-neutral memory control request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryControlRequest {
    command_label: String,
    kind: MemoryControlCommandKind,
    scope: MemoryTrustScope,
    target_id: Option<String>,
    evidence_handle: Option<String>,
    remote_admission: MemoryRemoteAdmissionStatus,
    freshness: MemoryFreshness,
    safe_output_budget: MemoryInjectionBudget,
}

impl MemoryControlRequest {
    /// Validates memory control metadata before adapter mutation, remote calls, or prompt
    /// influence.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryControlError`] when capability, admission, freshness, evidence, target, safe
    /// output, or redaction validation fails.
    // r[impl agent_harness_runtime.memory_controls.commands]
    // r[impl agent_harness_runtime.memory_controls.validation]
    // r[impl agent_harness_runtime.memory_controls.remote_admission]
    pub fn validate(draft: &MemoryControlRequestDraft<'_>) -> Result<Self, MemoryControlError> {
        reject_control_leaks(draft)?;
        let command_label = normalize(MEMORY_COMMAND_LABEL_FIELD, draft.command_label)?;
        if !draft.capability_granted {
            return Err(MemoryControlError::CapabilityDenied { command_label });
        }
        let scope = draft.scope.ok_or_else(|| MemoryControlError::Memory {
            source: MemoryError::MissingRecordScope {
                record_id: command_label.clone(),
            },
        })?;
        if draft.kind.requires_target() && draft.target_id.is_none() {
            return Err(MemoryControlError::MissingTarget { command_label });
        }
        if draft.kind.requires_evidence() && draft.evidence_handle.is_none() {
            return Err(MemoryControlError::Memory {
                source: MemoryError::MissingEvidence {
                    record_id: command_label.clone(),
                },
            });
        }
        if draft.freshness == MemoryFreshness::Expired
            || (draft.kind.mutates() && draft.freshness == MemoryFreshness::Stale)
        {
            return Err(MemoryControlError::StaleCommand {
                command_label,
                freshness: draft.freshness,
            });
        }
        if draft.kind.requires_remote_admission()
            && !matches!(
                draft.remote_admission,
                MemoryRemoteAdmissionStatus::NotRemote | MemoryRemoteAdmissionStatus::Admitted
            )
        {
            return Err(MemoryControlError::RemoteAdmission {
                command_label,
                status: draft.remote_admission,
            });
        }
        let safe_output_budget = MemoryInjectionBudget::new(draft.safe_output_budget_units)?;
        Ok(Self {
            command_label,
            kind: draft.kind,
            scope,
            target_id: normalize_optional(MEMORY_TARGET_FIELD, draft.target_id)?,
            evidence_handle: normalize_optional(MEMORY_EVIDENCE_FIELD, draft.evidence_handle)?,
            remote_admission: draft.remote_admission,
            freshness: draft.freshness,
            safe_output_budget,
        })
    }

    /// Returns command label.
    #[must_use]
    pub fn command_label(&self) -> &str {
        self.command_label.as_str()
    }

    /// Returns command kind.
    #[must_use]
    pub const fn kind(&self) -> MemoryControlCommandKind {
        self.kind
    }

    /// Returns scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns optional target identity.
    #[must_use]
    pub fn target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    /// Returns optional evidence handle.
    #[must_use]
    pub fn evidence_handle(&self) -> Option<&str> {
        self.evidence_handle.as_deref()
    }

    /// Returns remote admission status.
    #[must_use]
    pub const fn remote_admission(&self) -> MemoryRemoteAdmissionStatus {
        self.remote_admission
    }

    /// Returns target freshness.
    #[must_use]
    pub const fn freshness(&self) -> MemoryFreshness {
        self.freshness
    }

    /// Returns safe output budget.
    #[must_use]
    pub const fn safe_output_budget(&self) -> MemoryInjectionBudget {
        self.safe_output_budget
    }
}

/// Memory control validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemoryControlError {
    /// Underlying memory validation error.
    Memory {
        /// Memory error source.
        source: MemoryError,
    },
    /// Required target identity was missing.
    MissingTarget {
        /// Command label.
        command_label: String,
    },
    /// Capability policy denied the command.
    CapabilityDenied {
        /// Command label.
        command_label: String,
    },
    /// Remote admission failed.
    RemoteAdmission {
        /// Command label.
        command_label: String,
        /// Remote admission status.
        status: MemoryRemoteAdmissionStatus,
    },
    /// Command was stale or expired before dispatch.
    StaleCommand {
        /// Command label.
        command_label: String,
        /// Freshness state.
        freshness: MemoryFreshness,
    },
}

impl core::fmt::Display for MemoryControlError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Memory { source } => write!(formatter, "{source}"),
            Self::MissingTarget { command_label } => {
                write!(formatter, "{command_label} is missing a memory target")
            }
            Self::CapabilityDenied { command_label } => {
                write!(formatter, "{command_label} is denied by memory capability policy")
            }
            Self::RemoteAdmission { command_label, status } => {
                write!(formatter, "{command_label} failed remote memory admission: {status:?}")
            }
            Self::StaleCommand {
                command_label,
                freshness,
            } => write!(formatter, "{command_label} cannot dispatch with {freshness:?} memory freshness"),
        }
    }
}

impl From<MemoryError> for MemoryControlError {
    fn from(source: MemoryError) -> Self {
        Self::Memory { source }
    }
}

/// Safe descriptor returned by list, show, search, or export-summary commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryControlDescriptor {
    record_id: String,
    kind: GenericMemoryRecordKind,
    scope: MemoryTrustScope,
    source_label: String,
    evidence_handle: String,
    freshness: MemoryFreshness,
    redaction_class: MemoryRedactionClass,
    body_hash: crate::artifact::Hash,
    budget_units: NonZeroU32,
}

impl MemoryControlDescriptor {
    /// Returns record identity.
    #[must_use]
    pub fn record_id(&self) -> &str {
        self.record_id.as_str()
    }

    /// Returns record kind.
    #[must_use]
    pub const fn kind(&self) -> GenericMemoryRecordKind {
        self.kind
    }

    /// Returns scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns evidence handle.
    #[must_use]
    pub fn evidence_handle(&self) -> &str {
        self.evidence_handle.as_str()
    }

    /// Returns freshness.
    #[must_use]
    pub const fn freshness(&self) -> MemoryFreshness {
        self.freshness
    }

    /// Returns redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> MemoryRedactionClass {
        self.redaction_class
    }

    /// Returns body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns budget units.
    #[must_use]
    pub const fn budget_units(&self) -> NonZeroU32 {
        self.budget_units
    }
}

/// Projects safe descriptors for memory control list/search responses within a budget.
///
/// # Panics
///
/// Panics if callers provide more than the bounded memory descriptor count.
#[must_use]
// r[impl agent_harness_runtime.memory_controls.commands.safe_listing]
pub fn project_memory_control_listing(
    records: &[MemoryRecordEnvelope],
    safe_output_budget: MemoryInjectionBudget,
) -> Vec<MemoryControlDescriptor> {
    assert!(records.len() <= MAX_MEMORY_DESCRIPTORS);
    assert!(safe_output_budget.units().get() > ZERO_MEMORY_UNITS);
    let mut projected = Vec::with_capacity(records.len());
    let mut consumed_units = ZERO_MEMORY_UNITS;
    for record in records {
        assert!(!record.id().is_empty());
        assert!(record.budget_units().get() > ZERO_MEMORY_UNITS);
        let remaining_units = safe_output_budget.units().get().saturating_sub(consumed_units);
        if record.budget_units().get() > remaining_units {
            continue;
        }
        consumed_units = consumed_units.saturating_add(record.budget_units().get());
        projected.push(MemoryControlDescriptor {
            record_id: String::from(record.id()),
            kind: record.kind(),
            scope: record.scope(),
            source_label: String::from(record.source_label()),
            evidence_handle: String::from(record.evidence_handle()),
            freshness: record.freshness(),
            redaction_class: record.redaction_class(),
            body_hash: record.body_hash().clone(),
            budget_units: record.budget_units(),
        });
    }
    projected
}

/// Review state for an extracted memory candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryCandidateReviewState {
    /// Candidate awaits review.
    Pending,
    /// Candidate was approved.
    Approved,
    /// Candidate was rejected.
    Rejected,
    /// Candidate expired before review.
    Expired,
    /// Candidate was superseded.
    Superseded,
    /// Review service was unavailable.
    Unavailable,
    /// Candidate has been persisted by a store adapter.
    Persisted,
}

/// Borrowed candidate review transition input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryCandidateReviewInput<'a> {
    /// Candidate identity.
    pub candidate_id: &'a str,
    /// Requested review state.
    pub requested_state: MemoryCandidateReviewState,
    /// Candidate scope required for approval or persistence.
    pub scope: Option<MemoryTrustScope>,
    /// Evidence handle required for approval or persistence.
    pub evidence_handle: Option<&'a str>,
    /// Confidence percentage required for approval or persistence.
    pub confidence_percent: u8,
    /// Admission decision status.
    pub admission_status: MemoryAdmissionStatus,
    /// Safe reason label.
    pub reason_label: &'a str,
}

/// Safe candidate review transition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryCandidateReviewTransition {
    candidate_id: String,
    state: MemoryCandidateReviewState,
    scope: Option<MemoryTrustScope>,
    evidence_handle: Option<String>,
    confidence: Option<MemoryConfidence>,
    persistence_allowed: bool,
    reason_label: String,
}

impl MemoryCandidateReviewTransition {
    /// Returns candidate identity.
    #[must_use]
    pub fn candidate_id(&self) -> &str {
        self.candidate_id.as_str()
    }

    /// Returns review state.
    #[must_use]
    pub const fn state(&self) -> MemoryCandidateReviewState {
        self.state
    }

    /// Returns optional scope.
    #[must_use]
    pub const fn scope(&self) -> Option<MemoryTrustScope> {
        self.scope
    }

    /// Returns optional evidence handle.
    #[must_use]
    pub fn evidence_handle(&self) -> Option<&str> {
        self.evidence_handle.as_deref()
    }

    /// Returns optional confidence.
    #[must_use]
    pub const fn confidence(&self) -> Option<MemoryConfidence> {
        self.confidence
    }

    /// Returns true when storage or prompt influence may be planned.
    #[must_use]
    pub const fn persistence_allowed(&self) -> bool {
        self.persistence_allowed
    }

    /// Returns safe reason label.
    #[must_use]
    pub fn reason_label(&self) -> &str {
        self.reason_label.as_str()
    }
}

/// Transitions candidate review state without storing, updating, or injecting memory.
///
/// # Errors
///
/// Returns [`MemoryControlError`] when approval lacks scope, evidence, confidence, or admission.
///
/// # Panics
///
/// Panics only if normalized candidate or reason labels are unexpectedly empty after validation.
// r[impl agent_harness_runtime.memory_controls.review]
pub fn transition_memory_candidate_review(
    input: MemoryCandidateReviewInput<'_>,
) -> Result<MemoryCandidateReviewTransition, MemoryControlError> {
    let candidate_id = normalize(DESCRIPTOR_ID_FIELD, input.candidate_id)?;
    let reason_label = normalize(MEMORY_REASON_FIELD, input.reason_label)?;
    assert!(!candidate_id.is_empty());
    assert!(!reason_label.is_empty());
    let needs_admission =
        matches!(input.requested_state, MemoryCandidateReviewState::Approved | MemoryCandidateReviewState::Persisted);
    if needs_admission && input.admission_status != MemoryAdmissionStatus::Admitted {
        return Ok(MemoryCandidateReviewTransition {
            candidate_id,
            state: MemoryCandidateReviewState::Rejected,
            scope: None,
            evidence_handle: None,
            confidence: None,
            persistence_allowed: false,
            reason_label,
        });
    }
    if needs_admission {
        let scope = input.scope.ok_or_else(|| MemoryControlError::Memory {
            source: MemoryError::MissingRecordScope {
                record_id: candidate_id.clone(),
            },
        })?;
        let evidence_handle = input
            .evidence_handle
            .ok_or_else(|| MemoryControlError::Memory {
                source: MemoryError::MissingEvidence {
                    record_id: candidate_id.clone(),
                },
            })
            .and_then(|value| normalize(MEMORY_EVIDENCE_FIELD, value).map_err(Into::into))?;
        let confidence = MemoryConfidence::new(input.confidence_percent)?;
        return Ok(MemoryCandidateReviewTransition {
            candidate_id,
            state: input.requested_state,
            scope: Some(scope),
            evidence_handle: Some(evidence_handle),
            confidence: Some(confidence),
            persistence_allowed: true,
            reason_label,
        });
    }
    Ok(MemoryCandidateReviewTransition {
        candidate_id,
        state: input.requested_state,
        scope: input.scope,
        evidence_handle: normalize_optional(MEMORY_EVIDENCE_FIELD, input.evidence_handle)?,
        confidence: None,
        persistence_allowed: false,
        reason_label,
    })
}

/// Memory mutation operation kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryMutationOperationKind {
    /// Forget a record.
    Forget,
    /// Update a record.
    Update,
    /// Tombstone a record.
    Tombstone,
    /// Supersede a record.
    Supersede,
}

/// Borrowed mutation receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryMutationReceiptDraft<'a> {
    /// Mutation operation kind.
    pub operation_kind: MemoryMutationOperationKind,
    /// Target record identity.
    pub target_id: &'a str,
    /// Adapter family label.
    pub adapter_family_label: &'a str,
    /// Operation status.
    pub status: MemoryOperationStatus,
    /// Lowercase BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Lowercase BLAKE3 body hash.
    pub body_hash: &'a str,
    /// Raw memory body, forbidden in receipts.
    pub raw_memory_body: Option<&'a str>,
    /// Credential-looking value, forbidden in receipts.
    pub credential: Option<&'a str>,
    /// Provider payload, forbidden in receipts.
    pub provider_payload: Option<&'a str>,
    /// Full host path, forbidden in receipts.
    pub full_path: Option<&'a str>,
}

/// Safe mutation receipt for forget/update/tombstone/supersession outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryMutationReceipt {
    operation_kind: MemoryMutationOperationKind,
    target_id: String,
    adapter_family_label: String,
    status: MemoryOperationStatus,
    policy_hash: crate::artifact::Hash,
    body_hash: crate::artifact::Hash,
}

impl MemoryMutationReceipt {
    /// Validates mutation receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when safe fields fail validation or receipt leaks sensitive data.
    // r[impl agent_harness_runtime.memory_controls.forget_update]
    pub fn validate(draft: &MemoryMutationReceiptDraft<'_>) -> Result<Self, MemoryError> {
        reject_mutation_receipt_leaks(draft)?;
        Ok(Self {
            operation_kind: draft.operation_kind,
            target_id: normalize(MEMORY_TARGET_FIELD, draft.target_id)?,
            adapter_family_label: normalize(MEMORY_ADAPTER_LABEL_FIELD, draft.adapter_family_label)?,
            status: draft.status,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash).map_err(MemoryError::InvalidHash)?,
            body_hash: crate::artifact::Hash::new(draft.body_hash).map_err(MemoryError::InvalidHash)?,
        })
    }

    /// Returns operation kind.
    #[must_use]
    pub const fn operation_kind(&self) -> MemoryMutationOperationKind {
        self.operation_kind
    }

    /// Returns target identity.
    #[must_use]
    pub fn target_id(&self) -> &str {
        self.target_id.as_str()
    }

    /// Returns adapter family label.
    #[must_use]
    pub fn adapter_family_label(&self) -> &str {
        self.adapter_family_label.as_str()
    }

    /// Returns operation status.
    #[must_use]
    pub const fn status(&self) -> MemoryOperationStatus {
        self.status
    }

    /// Returns true only when receipt can claim deletion-like completion.
    #[must_use]
    pub const fn deletion_claimed(&self) -> bool {
        matches!(
            (self.operation_kind, self.status),
            (
                MemoryMutationOperationKind::Forget | MemoryMutationOperationKind::Tombstone,
                MemoryOperationStatus::Complete,
            )
        )
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }
}

/// Safe memory influence explanation entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryInfluenceExplanationEntry {
    record_id: String,
    source_label: String,
    rank: u32,
    match_reason: MemoryMatchReason,
    scope: MemoryTrustScope,
    confidence: MemoryConfidence,
    status: MemoryRetrievalOutcomeStatus,
    consumed_units: u32,
    body_hash: crate::artifact::Hash,
}

impl MemoryInfluenceExplanationEntry {
    /// Returns record identity.
    #[must_use]
    pub fn record_id(&self) -> &str {
        self.record_id.as_str()
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns rank.
    #[must_use]
    pub const fn rank(&self) -> u32 {
        self.rank
    }

    /// Returns match reason.
    #[must_use]
    pub const fn match_reason(&self) -> MemoryMatchReason {
        self.match_reason
    }

    /// Returns scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns confidence.
    #[must_use]
    pub const fn confidence(&self) -> MemoryConfidence {
        self.confidence
    }

    /// Returns retrieval outcome status.
    #[must_use]
    pub const fn status(&self) -> MemoryRetrievalOutcomeStatus {
        self.status
    }

    /// Returns consumed budget units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }

    /// Returns body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }
}

/// Safe memory influence explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryInfluenceExplanation {
    entries: Vec<MemoryInfluenceExplanationEntry>,
}

impl MemoryInfluenceExplanation {
    /// Returns safe explanation entries.
    #[must_use]
    pub fn entries(&self) -> &[MemoryInfluenceExplanationEntry] {
        self.entries.as_slice()
    }
}

/// Builds safe `why` output from a retrieval plan without exposing private bodies.
#[must_use]
// r[impl agent_harness_runtime.memory_controls.influence_explain]
pub fn explain_memory_influence(plan: &MemoryRetrievalPlan) -> MemoryInfluenceExplanation {
    let mut entries = Vec::with_capacity(plan.outcomes().len());
    for outcome in plan.outcomes() {
        entries.push(MemoryInfluenceExplanationEntry {
            record_id: String::from(outcome.record_id()),
            source_label: String::from(outcome.source_label()),
            rank: outcome.rank(),
            match_reason: outcome.match_reason(),
            scope: outcome.scope(),
            confidence: outcome.confidence(),
            status: outcome.status(),
            consumed_units: outcome.consumed_units(),
            body_hash: outcome.body_hash().clone(),
        });
    }
    MemoryInfluenceExplanation { entries }
}

fn normalize_optional(field: crate::validation::FieldName, value: Option<&str>) -> Result<Option<String>, MemoryError> {
    value.map(|inner| normalize(field, inner)).transpose()
}

fn reject_operation_receipt_leaks(draft: &MemoryOperationReceiptDraft<'_>) -> Result<(), MemoryError> {
    if draft.raw_memory_body.is_some() || draft.hidden_memory_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath));
    }
    if draft.unbounded_prompt_text.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::UnboundedPromptText));
    }
    Ok(())
}

fn reject_control_leaks(draft: &MemoryControlRequestDraft<'_>) -> Result<(), MemoryControlError> {
    if draft.raw_memory_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody).into());
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential).into());
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload).into());
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath).into());
    }
    Ok(())
}

fn reject_mutation_receipt_leaks(draft: &MemoryMutationReceiptDraft<'_>) -> Result<(), MemoryError> {
    if draft.raw_memory_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath));
    }
    Ok(())
}
