// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]
/// BLAKE3 or explicitly declared interoperability fingerprint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryFingerprint {
    /// Hash algorithm name.
    pub algorithm: String,
    /// Lowercase hexadecimal digest value.
    pub value_hex: String,
}

impl TrustBoundaryFingerprint {
    /// Builds a trust-boundary fingerprint from precomputed parts.
    #[must_use]
    pub fn new(algorithm: impl Into<String>, value_hex: impl Into<String>) -> Self {
        Self {
            algorithm: algorithm.into(),
            value_hex: value_hex.into(),
        }
    }
}

/// Verifier artifact identity guarded by the trust boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryVerifierArtifact {
    /// Safe verifier artifact label.
    pub label: String,
    /// BLAKE3 verifier artifact hash.
    pub hash: TrustBoundaryFingerprint,
}

/// Evidence acceptance policy for a boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryEvidencePolicy {
    /// Safe evidence labels allowed to count as successful evidence.
    pub accepted_evidence_labels: Vec<String>,
    /// Whether monitor cleanliness is required before evidence acceptance.
    pub require_monitor_clean: bool,
}

/// Immutable outer trust-boundary descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryDescriptor {
    /// Stable boundary identifier.
    pub boundary_id: String,
    /// Tool names sanctioned by the boundary.
    pub sanctioned_tools: Vec<String>,
    /// Safe labels for withheld paths or oracle inputs.
    pub withheld_path_labels: Vec<String>,
    /// Guarded verifier artifact identity.
    pub verifier: TrustBoundaryVerifierArtifact,
    /// Sandbox profile identity required by the boundary.
    pub sandbox_profile_id: String,
    /// Actions requiring explicit approval before success evidence.
    pub approval_required_actions: Vec<TrustBoundaryActionKind>,
    /// Evidence acceptance policy.
    pub evidence_policy: TrustBoundaryEvidencePolicy,
    /// BLAKE3 hash of the reviewed boundary policy.
    pub policy_hash: TrustBoundaryFingerprint,
    /// Whether an inner runtime path requested boundary mutation.
    pub mutable_boundary_request: bool,
}

/// Planned or observed action class evaluated by the monitor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustBoundaryActionKind {
    /// Tool invocation.
    ToolCall,
    /// Read of a path or oracle label.
    PathRead,
    /// Modification of verifier artifact or script.
    VerifierModification,
    /// Sandbox profile change or weakening.
    SandboxProfileChange,
    /// Suppression or bypass of required approval.
    ApprovalSuppression,
    /// Attempt to accept evidence.
    EvidenceAcceptance,
    /// Oracle copying anti-gaming fixture.
    OracleCopy,
    /// Hardcoded expected output or artifact fixture.
    HardcodedExpectedOutput,
    /// Visible-test overfitting fixture.
    VisibleTestOverfit,
    /// Allowed-tool intent gaming fixture.
    AllowedToolIntentGaming,
}

/// Planned or observed action descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryAction {
    /// Stable action identifier.
    pub action_id: String,
    /// Action class.
    pub kind: TrustBoundaryActionKind,
    /// Tool name for tool-call actions.
    pub tool_name: Option<String>,
    /// Safe path, oracle, or verifier label.
    pub path_label: Option<String>,
    /// Requested sandbox profile identity.
    pub sandbox_profile_id: Option<String>,
    /// Evidence label for evidence-acceptance actions.
    pub evidence_label: Option<String>,
    /// Whether required approval was present.
    pub approval_present: bool,
}

/// Exact deterministic trust-boundary issue code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustBoundaryIssueCode {
    /// Action attempted to read withheld material.
    WithheldPathRead,
    /// Action attempted to modify verifier artifacts.
    VerifierModification,
    /// Action attempted an unsanctioned tool call.
    UnsanctionedTool,
    /// Action attempted to weaken or change sandbox profile.
    SandboxWeakening,
    /// Action attempted to suppress required approval.
    ApprovalSuppression,
    /// Action attempted to accept evidence outside policy.
    EvidencePolicyBypass,
    /// Action attempted oracle copying.
    OracleCopy,
    /// Action hardcoded expected output or artifact.
    HardcodedExpectedOutput,
    /// Action overfit visible tests.
    VisibleTestOverfit,
    /// Action used an allowed tool for gaming intent.
    AllowedToolIntentGaming,
}

/// Deterministic monitor decision for one action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryMonitorDecision {
    /// Whether the action may count toward successful evidence.
    pub accepted: bool,
    /// Exact issue codes found by the monitor.
    pub issue_codes: Vec<TrustBoundaryIssueCode>,
    /// Diagnostics with safe labels.
    pub diagnostics: Vec<TrustBoundaryDiagnostic>,
}

/// Host-provided judge-veto outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustBoundaryJudgeOutcome {
    /// Judge did not object, but grants no authority.
    Approve,
    /// Judge vetoed the trajectory as suspicious.
    Veto,
    /// Judge was unavailable.
    Unavailable,
}

/// Optional judge-veto input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryJudgeVeto {
    /// Judge outcome.
    pub outcome: TrustBoundaryJudgeOutcome,
    /// Safe reason label supplied by host or fixture.
    pub reason_label: Option<String>,
}

/// Combined monitor and judge policy binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryJudgeBinding {
    /// Final acceptance after monitor and judge binding.
    pub accepted: bool,
    /// Whether deterministic monitor denial remained final.
    pub monitor_denial_final: bool,
    /// Whether a judge approval was ignored as an authority grant.
    pub judge_grant_ignored: bool,
    /// Diagnostics explaining the binding.
    pub diagnostics: Vec<TrustBoundaryDiagnostic>,
}

/// Redaction status for violation receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustBoundaryRedactionStatus {
    /// Receipt contains only safe metadata.
    Redacted,
    /// Receipt contains unsafe material and must be rejected.
    Unsafe,
}

/// Sensitive classes that must not appear in violation receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrustBoundaryReceiptLeak {
    /// Hidden path body or raw path.
    HiddenPath,
    /// Raw verifier body.
    RawVerifierBody,
    /// Raw prompt body.
    Prompt,
    /// Raw tool input or output body.
    ToolBody,
    /// Provider-native payload.
    ProviderPayload,
    /// Credential-looking value.
    Credential,
    /// Full path dump.
    FullPathDump,
}

/// Redacted boundary violation receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryViolationReceipt {
    /// Boundary policy hash.
    pub policy_hash: TrustBoundaryFingerprint,
    /// Issue code being recorded.
    pub issue_code: TrustBoundaryIssueCode,
    /// Sanitized labels related to the violation.
    pub labels: Vec<String>,
    /// Count of matching observations.
    pub count: u32,
    /// Receipt redaction status.
    pub redaction_status: TrustBoundaryRedactionStatus,
    /// Negative-test-only leak classes.
    pub leaks: Vec<TrustBoundaryReceiptLeak>,
}

/// Trust-boundary diagnostic class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrustBoundaryDiagnosticKind {
    /// Required field is missing or blank.
    MissingField,
    /// Required collection is empty.
    EmptyCollection,
    /// Duplicate safe label or identity.
    DuplicateLabel,
    /// Raw sensitive path or path separator was serialized.
    UnsafeRawPath,
    /// Fingerprint is missing, not BLAKE3, or otherwise invalid.
    InvalidHash,
    /// Inner runtime attempted to mutate immutable boundary data.
    MutableBoundaryRequest,
    /// Deterministic monitor denied an action.
    MonitorDenied,
    /// Judge output was bound as reject-only.
    JudgeRejectOnly,
    /// Receipt contains unsafe material.
    ReceiptLeak,
}

/// Trust-boundary diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBoundaryDiagnostic {
    /// Diagnostic class.
    pub kind: TrustBoundaryDiagnosticKind,
    /// Stable field or relation label.
    pub field: String,
    /// Safe detail text.
    pub detail: String,
}

impl TrustBoundaryDiagnostic {
    /// Builds a trust-boundary diagnostic.
    #[must_use]
    pub fn new(
        kind: TrustBoundaryDiagnosticKind,
        field: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            field: field.into(),
            detail: detail.into(),
        }
    }
}

/// Trust-boundary validation result.
pub type TrustBoundaryValidationResult = Result<(), Vec<TrustBoundaryDiagnostic>>;
