#![allow(unknown_lints)]
#![cfg_attr(animus_octet, allow(tigerstyle::excessive_file_length))]

use super::{
    GenericMemoryAdapterCapability, GenericMemoryAdapterRole, MEMORY_SOURCE_FIELD, MemoryError,
    MemoryOperationStatus, MemoryReceiptLeak, MemoryRetrievalOutcomeStatus, String, Vec,
    memory_leak, normalize,
};

const MEMORY_ADAPTER_OWNER_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_adapter_owner");
const MEMORY_REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_reason_label");
const MEMORY_CONFORMANCE_CASE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_conformance_case");
const MEMORY_CONFORMANCE_EVIDENCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_conformance_evidence");
const MEMORY_SERVICE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_service_label");
const MEMORY_REMOTE_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_remote_id_label");

/// Memory backend family tracked by the conformance rail.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryBackendFamily {
    /// Null, disabled, or no-op memory backend.
    NullNoOp,
    /// Local file or markdown host-owned backend.
    LocalFileMarkdown,
    /// SQL or document store backend.
    SqlDocument,
    /// Vector or hybrid search backend.
    VectorHybrid,
    /// MCP or remote memory backend.
    McpRemote,
    /// Candidate extractor backend.
    Extractor,
    /// Host admission backend.
    Admission,
}

impl MemoryBackendFamily {
    /// Returns the stable backend-family label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NullNoOp => "null-no-op",
            Self::LocalFileMarkdown => "local-file-markdown",
            Self::SqlDocument => "sql-document",
            Self::VectorHybrid => "vector-hybrid",
            Self::McpRemote => "mcp-remote",
            Self::Extractor => "extractor",
            Self::Admission => "admission",
        }
    }
}

/// Applicability status for a conformance matrix row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryConformanceApplicability<'a> {
    /// Row applies to the backend family.
    Applicable,
    /// Row does not apply and carries a safe reason.
    NotApplicable {
        /// Safe not-applicable reason.
        reason: &'a str,
    },
}

/// Borrowed conformance matrix row before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryConformanceMatrixRowDraft<'a> {
    /// Backend family.
    pub backend_family: MemoryBackendFamily,
    /// Adapter role covered by the row.
    pub adapter_role: GenericMemoryAdapterRole,
    /// Declared capabilities for this row.
    pub capabilities: &'a [GenericMemoryAdapterCapability],
    /// Safe owner label.
    pub owner_label: &'a str,
    /// Applicability status.
    pub applicability: MemoryConformanceApplicability<'a>,
    /// Positive behavior case label.
    pub positive_case: Option<&'a str>,
    /// Negative behavior case label.
    pub negative_case: Option<&'a str>,
    /// Safe evidence label.
    pub safe_evidence_label: Option<&'a str>,
}

/// Validated memory conformance matrix row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryConformanceMatrixRow {
    backend_family: MemoryBackendFamily,
    adapter_role: GenericMemoryAdapterRole,
    capabilities: Vec<GenericMemoryAdapterCapability>,
    owner_label: String,
    applicability: Option<String>,
    positive_case: Option<String>,
    negative_case: Option<String>,
    safe_evidence_label: Option<String>,
}

impl MemoryConformanceMatrixRow {
    /// Validates one conformance matrix row.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryConformanceError`] when owner, capability, positive, negative, evidence, or not-applicable metadata is missing.
    // r[impl agent_harness_runtime.memory_conformance.matrix]
    pub fn validate(
        draft: &MemoryConformanceMatrixRowDraft<'_>,
    ) -> Result<Self, MemoryConformanceError> {
        if draft.capabilities.is_empty() {
            return Err(MemoryConformanceError::MissingCapability {
                family: draft.backend_family,
            });
        }
        let owner_label = normalize(MEMORY_ADAPTER_OWNER_FIELD, draft.owner_label)?;
        match draft.applicability {
            MemoryConformanceApplicability::Applicable => {
                let positive_case = required_conformance_label(
                    draft.backend_family,
                    draft.positive_case,
                    MemoryConformanceMissingField::Positive,
                )?;
                let negative_case = required_conformance_label(
                    draft.backend_family,
                    draft.negative_case,
                    MemoryConformanceMissingField::Negative,
                )?;
                let safe_evidence_label = required_conformance_label(
                    draft.backend_family,
                    draft.safe_evidence_label,
                    MemoryConformanceMissingField::Evidence,
                )?;
                Ok(Self {
                    backend_family: draft.backend_family,
                    adapter_role: draft.adapter_role,
                    capabilities: draft.capabilities.to_vec(),
                    owner_label,
                    applicability: None,
                    positive_case: Some(positive_case),
                    negative_case: Some(negative_case),
                    safe_evidence_label: Some(safe_evidence_label),
                })
            }
            MemoryConformanceApplicability::NotApplicable { reason } => Ok(Self {
                backend_family: draft.backend_family,
                adapter_role: draft.adapter_role,
                capabilities: draft.capabilities.to_vec(),
                owner_label,
                applicability: Some(normalize(MEMORY_REASON_FIELD, reason)?),
                positive_case: normalize_optional(
                    MEMORY_CONFORMANCE_CASE_FIELD,
                    draft.positive_case,
                )?,
                negative_case: normalize_optional(
                    MEMORY_CONFORMANCE_CASE_FIELD,
                    draft.negative_case,
                )?,
                safe_evidence_label: normalize_optional(
                    MEMORY_CONFORMANCE_EVIDENCE_FIELD,
                    draft.safe_evidence_label,
                )?,
            }),
        }
    }

    /// Returns backend family.
    #[must_use]
    pub const fn backend_family(&self) -> MemoryBackendFamily {
        self.backend_family
    }

    /// Returns adapter role.
    #[must_use]
    pub const fn adapter_role(&self) -> GenericMemoryAdapterRole {
        self.adapter_role
    }

    /// Returns declared capabilities.
    #[must_use]
    pub fn capabilities(&self) -> &[GenericMemoryAdapterCapability] {
        self.capabilities.as_slice()
    }

    /// Returns owner label.
    #[must_use]
    pub fn owner_label(&self) -> &str {
        self.owner_label.as_str()
    }

    /// Returns not-applicable reason, if any.
    #[must_use]
    pub fn not_applicable_reason(&self) -> Option<&str> {
        self.applicability.as_deref()
    }

    /// Returns positive case label, if any.
    #[must_use]
    pub fn positive_case(&self) -> Option<&str> {
        self.positive_case.as_deref()
    }

    /// Returns negative case label, if any.
    #[must_use]
    pub fn negative_case(&self) -> Option<&str> {
        self.negative_case.as_deref()
    }

    /// Returns safe evidence label, if any.
    #[must_use]
    pub fn safe_evidence_label(&self) -> Option<&str> {
        self.safe_evidence_label.as_deref()
    }
}

/// Memory conformance rail validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemoryConformanceError {
    /// Underlying memory validation error.
    Memory {
        /// Memory error source.
        source: MemoryError,
    },
    /// Row omitted declared capabilities.
    MissingCapability {
        /// Backend family.
        family: MemoryBackendFamily,
    },
    /// Applicable row omitted a positive case.
    MissingPositiveCase {
        /// Backend family.
        family: MemoryBackendFamily,
    },
    /// Applicable row omitted a negative case.
    MissingNegativeCase {
        /// Backend family.
        family: MemoryBackendFamily,
    },
    /// Applicable row omitted safe evidence.
    MissingEvidence {
        /// Backend family.
        family: MemoryBackendFamily,
    },
    /// Remote failure row tried to claim success.
    UnsafeSuccessClaim {
        /// Backend family.
        family: MemoryBackendFamily,
    },
}

impl core::fmt::Display for MemoryConformanceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Memory { source } => write!(formatter, "{source}"),
            Self::MissingCapability { family } => {
                write!(
                    formatter,
                    "{} conformance row lacks capability",
                    family.as_str()
                )
            }
            Self::MissingPositiveCase { family } => write!(
                formatter,
                "{} conformance row lacks a positive case",
                family.as_str()
            ),
            Self::MissingNegativeCase { family } => write!(
                formatter,
                "{} conformance row lacks a negative case",
                family.as_str()
            ),
            Self::MissingEvidence { family } => write!(
                formatter,
                "{} conformance row lacks safe evidence",
                family.as_str()
            ),
            Self::UnsafeSuccessClaim { family } => write!(
                formatter,
                "{} failure fixture claimed a successful memory operation",
                family.as_str()
            ),
        }
    }
}

impl From<MemoryError> for MemoryConformanceError {
    fn from(source: MemoryError) -> Self {
        Self::Memory { source }
    }
}

/// Enablement state for a null/no-op backend input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryNullBackendToggle {
    /// Behavior is enabled and the null backend must report unavailable.
    Enabled,
    /// Behavior is disabled and the null backend remains unsupported/no-op.
    Disabled,
}

/// Required rule presence outside memory state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRequiredRulePresence {
    /// Required rule is present.
    Present,
    /// Required rule is absent.
    Absent,
}

/// Null-backend side-effect planning state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryNullSideEffectState {
    /// A side effect is planned.
    Planned,
    /// A side effect is not planned.
    NotPlanned,
}

/// Required rule activity state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRequiredRuleState {
    /// Required guidance or policy remains active.
    Active,
    /// No required guidance or policy is active.
    Inactive,
}

/// Input for null/no-op memory backend evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryNullBackendInput {
    /// Memory-use enablement.
    pub memory_use: MemoryNullBackendToggle,
    /// Memory-generation enablement.
    pub memory_generation: MemoryNullBackendToggle,
    /// Required guidance presence outside memory.
    pub guidance: MemoryRequiredRulePresence,
    /// Required policy presence outside memory.
    pub policy: MemoryRequiredRulePresence,
}

/// Null/no-op backend plan proving memory performs no side effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryNullBackendPlan {
    status: MemoryOperationStatus,
    storage: MemoryNullSideEffectState,
    retrieval: MemoryNullSideEffectState,
    prompt_injection: MemoryNullSideEffectState,
    extraction: MemoryNullSideEffectState,
    mutation: MemoryNullSideEffectState,
    required_rules: MemoryRequiredRuleState,
}

impl MemoryNullBackendPlan {
    /// Returns status.
    #[must_use]
    pub const fn status(self) -> MemoryOperationStatus {
        self.status
    }

    /// Returns true when storage was planned.
    #[must_use]
    pub const fn storage_planned(self) -> bool {
        matches!(self.storage, MemoryNullSideEffectState::Planned)
    }

    /// Returns true when retrieval was planned.
    #[must_use]
    pub const fn retrieval_planned(self) -> bool {
        matches!(self.retrieval, MemoryNullSideEffectState::Planned)
    }

    /// Returns true when prompt injection was planned.
    #[must_use]
    pub const fn prompt_injection_planned(self) -> bool {
        matches!(self.prompt_injection, MemoryNullSideEffectState::Planned)
    }

    /// Returns true when extraction was planned.
    #[must_use]
    pub const fn extraction_planned(self) -> bool {
        matches!(self.extraction, MemoryNullSideEffectState::Planned)
    }

    /// Returns true when mutation was planned.
    #[must_use]
    pub const fn mutation_planned(self) -> bool {
        matches!(self.mutation, MemoryNullSideEffectState::Planned)
    }

    /// Returns true when required guidance or policy remains active.
    #[must_use]
    pub const fn required_rules_active(self) -> bool {
        matches!(self.required_rules, MemoryRequiredRuleState::Active)
    }
}

/// Evaluates a null/no-op memory backend without planning side effects.
///
/// # Panics
///
/// Panics only if the computed null-backend status or rule state falls outside expected enums.
#[must_use]
// r[impl agent_harness_runtime.memory_conformance.null_backend]
pub fn evaluate_null_memory_backend(input: MemoryNullBackendInput) -> MemoryNullBackendPlan {
    let status = if matches!(input.memory_use, MemoryNullBackendToggle::Enabled)
        || matches!(input.memory_generation, MemoryNullBackendToggle::Enabled)
    {
        MemoryOperationStatus::Unavailable
    } else {
        MemoryOperationStatus::Unsupported
    };
    let required_rules = if matches!(input.guidance, MemoryRequiredRulePresence::Present)
        || matches!(input.policy, MemoryRequiredRulePresence::Present)
    {
        MemoryRequiredRuleState::Active
    } else {
        MemoryRequiredRuleState::Inactive
    };
    assert!(matches!(
        status,
        MemoryOperationStatus::Unavailable | MemoryOperationStatus::Unsupported
    ));
    assert!(matches!(
        required_rules,
        MemoryRequiredRuleState::Active | MemoryRequiredRuleState::Inactive
    ));
    MemoryNullBackendPlan {
        status,
        storage: MemoryNullSideEffectState::NotPlanned,
        retrieval: MemoryNullSideEffectState::NotPlanned,
        prompt_injection: MemoryNullSideEffectState::NotPlanned,
        extraction: MemoryNullSideEffectState::NotPlanned,
        mutation: MemoryNullSideEffectState::NotPlanned,
        required_rules,
    }
}

/// Borrowed file-backed evidence metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryFilesystemEvidenceDraft<'a> {
    /// Safe source label.
    pub source_label: &'a str,
    /// BLAKE3 body hash.
    pub body_hash: &'a str,
    /// Full path, forbidden in reusable evidence.
    pub full_path: Option<&'a str>,
    /// Raw memory body, forbidden in reusable evidence.
    pub raw_memory_body: Option<&'a str>,
}

/// Safe reusable file-backed memory evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryFilesystemEvidence {
    source_label: String,
    body_hash: crate::artifact::Hash,
}

impl MemoryFilesystemEvidence {
    /// Validates file-backed evidence after host-owned import redacts paths and bodies.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when evidence leaks a path, raw body, or malformed hash.
    ///
    /// # Panics
    ///
    /// Panics only if leak checks or validated safe-label/hash invariants are violated.
    // r[impl agent_harness_runtime.memory_conformance.filesystem_backend]
    pub fn validate(draft: &MemoryFilesystemEvidenceDraft<'_>) -> Result<Self, MemoryError> {
        if draft.full_path.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::FullPath));
        }
        if draft.raw_memory_body.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
        }
        assert!(draft.full_path.is_none());
        assert!(draft.raw_memory_body.is_none());
        let source_label = normalize(MEMORY_SOURCE_FIELD, draft.source_label)?;
        let body_hash =
            crate::artifact::Hash::new(draft.body_hash).map_err(MemoryError::InvalidHash)?;
        assert!(!source_label.is_empty());
        assert!(!body_hash.as_str().is_empty());
        Ok(Self {
            source_label,
            body_hash,
        })
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns BLAKE3 body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }
}

/// Input for semantic backend unavailable planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemorySemanticBackendFallbackInput {
    /// Backend status.
    pub status: MemoryOperationStatus,
    /// Whether an alternate adapter was explicitly injected and admitted.
    pub explicit_alternate_adapter: bool,
}

/// Semantic backend fallback plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemorySemanticBackendFallbackPlan {
    status: MemoryOperationStatus,
    ambient_fallback_attempted: bool,
    explicit_alternate_attempted: bool,
}

impl MemorySemanticBackendFallbackPlan {
    /// Returns backend status.
    #[must_use]
    pub const fn status(self) -> MemoryOperationStatus {
        self.status
    }

    /// Returns true when an ambient fallback was attempted.
    #[must_use]
    pub const fn ambient_fallback_attempted(self) -> bool {
        self.ambient_fallback_attempted
    }

    /// Returns true when an explicit alternate adapter may be tried.
    #[must_use]
    pub const fn explicit_alternate_attempted(self) -> bool {
        self.explicit_alternate_attempted
    }
}

/// Plans semantic backend failure behavior without ambient keyword/provider/network/index fallback.
#[must_use]
// r[impl agent_harness_runtime.memory_conformance.semantic_backend]
pub fn plan_semantic_backend_fallback(
    input: MemorySemanticBackendFallbackInput,
) -> MemorySemanticBackendFallbackPlan {
    MemorySemanticBackendFallbackPlan {
        status: input.status,
        ambient_fallback_attempted: false,
        explicit_alternate_attempted: input.explicit_alternate_adapter
            && input.status != MemoryOperationStatus::Complete,
    }
}

/// Borrowed remote memory backend failure fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRemoteBackendFailureDraft<'a> {
    /// Safe service label.
    pub service_label: &'a str,
    /// Safe remote ID label.
    pub remote_id_label: &'a str,
    /// Failure status.
    pub status: MemoryOperationStatus,
    /// Credential-looking value, forbidden in receipts.
    pub credential: Option<&'a str>,
    /// Provider payload, forbidden in receipts.
    pub provider_payload: Option<&'a str>,
    /// Transport envelope, forbidden in receipts.
    pub transport_envelope: Option<&'a str>,
    /// Unbounded remote output, forbidden in receipts.
    pub unbounded_remote_output: Option<&'a str>,
}

/// Safe remote memory backend failure receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRemoteBackendFailure {
    service_label: String,
    remote_id_label: String,
    status: MemoryOperationStatus,
}

impl MemoryRemoteBackendFailure {
    /// Validates an MCP/remote memory failure receipt.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryConformanceError`] when the fixture leaks sensitive data or claims success.
    ///
    /// # Panics
    ///
    /// Panics only if failure-status or redaction invariants are violated after validation.
    // r[impl agent_harness_runtime.memory_conformance.remote_backend]
    pub fn validate(
        draft: &MemoryRemoteBackendFailureDraft<'_>,
    ) -> Result<Self, MemoryConformanceError> {
        if draft.status == MemoryOperationStatus::Complete {
            return Err(MemoryConformanceError::UnsafeSuccessClaim {
                family: MemoryBackendFamily::McpRemote,
            });
        }
        if draft.credential.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::Credential).into());
        }
        if draft.provider_payload.is_some() || draft.transport_envelope.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::ProviderPayload).into());
        }
        if draft.unbounded_remote_output.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::UnboundedPromptText).into());
        }
        assert_ne!(draft.status, MemoryOperationStatus::Complete);
        assert!(draft.credential.is_none());
        assert!(draft.provider_payload.is_none());
        assert!(draft.transport_envelope.is_none());
        assert!(draft.unbounded_remote_output.is_none());
        let service_label = normalize(MEMORY_SERVICE_LABEL_FIELD, draft.service_label)?;
        let remote_id_label = normalize(MEMORY_REMOTE_ID_FIELD, draft.remote_id_label)?;
        assert!(!service_label.is_empty());
        assert!(!remote_id_label.is_empty());
        Ok(Self {
            service_label,
            remote_id_label,
            status: draft.status,
        })
    }

    /// Returns safe service label.
    #[must_use]
    pub fn service_label(&self) -> &str {
        self.service_label.as_str()
    }

    /// Returns safe remote ID label.
    #[must_use]
    pub fn remote_id_label(&self) -> &str {
        self.remote_id_label.as_str()
    }

    /// Returns failure status.
    #[must_use]
    pub const fn status(&self) -> MemoryOperationStatus {
        self.status
    }
}

/// Shared memory failure taxonomy.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryFailureClass {
    /// Adapter or backend unavailable.
    Unavailable,
    /// Policy or admission denied the operation.
    Denied,
    /// Adapter or backend timed out.
    TimedOut,
    /// Record was malformed.
    MalformedRecord,
    /// Record or command was stale.
    Stale,
    /// Record exceeded budget.
    OverBudget,
    /// Record was redacted.
    Redacted,
    /// Record conflicted with another record.
    Conflicting,
    /// Receipt was unsafe.
    UnsafeReceipt,
    /// Receipt or DTO leaked sensitive data.
    Leak,
}

impl MemoryFailureClass {
    /// Returns the stable failure class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Denied => "denied",
            Self::TimedOut => "timed-out",
            Self::MalformedRecord => "malformed-record",
            Self::Stale => "stale",
            Self::OverBudget => "over-budget",
            Self::Redacted => "redacted",
            Self::Conflicting => "conflicting",
            Self::UnsafeReceipt => "unsafe-receipt",
            Self::Leak => "leak",
        }
    }
}

/// Maps retrieval outcomes into the shared memory failure taxonomy.
#[must_use]
// r[impl agent_harness_runtime.memory_conformance.failure_taxonomy]
pub const fn memory_failure_for_retrieval_status(
    status: MemoryRetrievalOutcomeStatus,
) -> Option<MemoryFailureClass> {
    match status {
        MemoryRetrievalOutcomeStatus::Selected => None,
        MemoryRetrievalOutcomeStatus::SkippedScope | MemoryRetrievalOutcomeStatus::Denied => {
            Some(MemoryFailureClass::Denied)
        }
        MemoryRetrievalOutcomeStatus::Stale | MemoryRetrievalOutcomeStatus::Superseded => {
            Some(MemoryFailureClass::Stale)
        }
        MemoryRetrievalOutcomeStatus::Redacted => Some(MemoryFailureClass::Redacted),
        MemoryRetrievalOutcomeStatus::Conflicting
        | MemoryRetrievalOutcomeStatus::DiversityDuplicate => Some(MemoryFailureClass::Conflicting),
        MemoryRetrievalOutcomeStatus::OverBudget => Some(MemoryFailureClass::OverBudget),
    }
}

/// Maps operation statuses into the shared memory failure taxonomy.
#[must_use]
pub const fn memory_failure_for_operation_status(
    status: MemoryOperationStatus,
) -> Option<MemoryFailureClass> {
    match status {
        MemoryOperationStatus::Complete => None,
        MemoryOperationStatus::Denied | MemoryOperationStatus::Rejected => {
            Some(MemoryFailureClass::Denied)
        }
        MemoryOperationStatus::Unavailable | MemoryOperationStatus::Unsupported => {
            Some(MemoryFailureClass::Unavailable)
        }
        MemoryOperationStatus::TimedOut => Some(MemoryFailureClass::TimedOut),
        MemoryOperationStatus::Partial | MemoryOperationStatus::Stale => {
            Some(MemoryFailureClass::Stale)
        }
    }
}

fn normalize_optional(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<String>, MemoryError> {
    value.map(|inner| normalize(field, inner)).transpose()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MemoryConformanceMissingField {
    Positive,
    Negative,
    Evidence,
}

fn required_conformance_label(
    family: MemoryBackendFamily,
    value: Option<&str>,
    field: MemoryConformanceMissingField,
) -> Result<String, MemoryConformanceError> {
    let Some(value) = value else {
        return Err(match field {
            MemoryConformanceMissingField::Positive => {
                MemoryConformanceError::MissingPositiveCase { family }
            }
            MemoryConformanceMissingField::Negative => {
                MemoryConformanceError::MissingNegativeCase { family }
            }
            MemoryConformanceMissingField::Evidence => {
                MemoryConformanceError::MissingEvidence { family }
            }
        });
    };
    let field_name = match field {
        MemoryConformanceMissingField::Evidence => MEMORY_CONFORMANCE_EVIDENCE_FIELD,
        MemoryConformanceMissingField::Positive | MemoryConformanceMissingField::Negative => {
            MEMORY_CONFORMANCE_CASE_FIELD
        }
    };
    assert!(matches!(
        field,
        MemoryConformanceMissingField::Positive
            | MemoryConformanceMissingField::Negative
            | MemoryConformanceMissingField::Evidence
    ));
    let label = normalize(field_name, value)?;
    assert!(!label.is_empty());
    Ok(label)
}
