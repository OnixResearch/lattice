//! Metadata-only evaluation adapter receipt drafts.

use alloc::{format, string::String, vec, vec::Vec};

use animus_eval_core::{hash_identity, validate_identity};

use crate::MAX_ADAPTER_RECEIPT_LABELS;

/// Maximum safe label bytes in one adapter receipt.
pub const MAX_ADAPTER_RECEIPT_LABEL_BYTES: usize = 96;

/// Receipt subject class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterReceiptKind {
    /// Service admission.
    ServiceAdmission,
    /// Typed request.
    ServiceRequest,
    /// Bounded observation.
    ServiceObservation,
    /// Model gateway projection or ordering gate.
    Gateway,
    /// Artifact transfer metadata.
    Artifact,
    /// Verifier metadata.
    Verifier,
    /// Storage metadata.
    Storage,
}

impl AdapterReceiptKind {
    const fn label(self) -> &'static str {
        match self {
            Self::ServiceAdmission => "service_admission",
            Self::ServiceRequest => "service_request",
            Self::ServiceObservation => "service_observation",
            Self::Gateway => "gateway",
            Self::Artifact => "artifact",
            Self::Verifier => "verifier",
            Self::Storage => "storage",
        }
    }
}

/// Receipt status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterReceiptStatus {
    /// Service or request was admitted.
    Admitted,
    /// Bounded success was observed.
    Observed,
    /// Call remains blocked.
    Blocked,
    /// Call failed under supplied host evidence.
    Failed,
}

impl AdapterReceiptStatus {
    const fn label(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Observed => "observed",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

/// Unsupported claim class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterReceiptClaim {
    /// A bounded metadata observation only.
    BoundedObservation,
    /// Exactly-once remote effects.
    ExactlyOnceRemoteEffects,
    /// Verifier soundness.
    VerifierSoundness,
    /// Artifact authenticity.
    ArtifactAuthenticity,
    /// Complete model-call capture.
    CompleteCapture,
    /// Benchmark validity.
    BenchmarkValidity,
    /// Authoritative completion.
    AuthoritativeCompletion,
    /// Production readiness.
    ProductionReadiness,
    /// Release eligibility.
    ReleaseEligibility,
}

/// Explicit prohibited-content facts supplied by the projector.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AdapterReceiptLeakFlags {
    /// Raw prompt is present.
    pub prompt: bool,
    /// Hidden test is present.
    pub hidden_test: bool,
    /// Reasoning is present.
    pub reasoning: bool,
    /// Provider body is present.
    pub provider_body: bool,
    /// Tool body is present.
    pub tool_body: bool,
    /// Credential is present.
    pub credential: bool,
    /// Environment value is present.
    pub environment_value: bool,
    /// Physical path is present.
    pub physical_path: bool,
    /// Unbounded log is present.
    pub unbounded_log: bool,
}

impl AdapterReceiptLeakFlags {
    const fn any(self) -> bool {
        self.prompt
            || self.hidden_test
            || self.reasoning
            || self.provider_body
            || self.tool_body
            || self.credential
            || self.environment_value
            || self.physical_path
            || self.unbounded_log
    }
}

/// Borrowed adapter receipt input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterReceiptDraft<'a> {
    /// Receipt subject kind.
    pub kind: AdapterReceiptKind,
    /// Evaluation identity.
    pub evaluation_identity: &'a str,
    /// Plan identity.
    pub plan_identity: &'a str,
    /// Service or gateway identity.
    pub subject_identity: &'a str,
    /// Exact policy identity.
    pub policy_identity: &'a str,
    /// Exact implementation identity.
    pub implementation_identity: &'a str,
    /// Receipt status.
    pub status: AdapterReceiptStatus,
    /// Bounded safe labels.
    pub labels: &'a [&'a str],
    /// Bounded item count.
    pub item_count: u32,
    /// Redacted item count.
    pub redaction_count: u32,
    /// Explicit prohibited content facts.
    pub leaks: AdapterReceiptLeakFlags,
    /// Requested claim classes.
    pub claims: &'a [AdapterReceiptClaim],
}

/// Validated metadata-only receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterReceipt {
    /// Canonical receipt identity.
    pub identity: String,
    /// Receipt kind.
    pub kind: AdapterReceiptKind,
    /// Evaluation identity.
    pub evaluation_identity: String,
    /// Plan identity.
    pub plan_identity: String,
    /// Subject identity.
    pub subject_identity: String,
    /// Policy identity.
    pub policy_identity: String,
    /// Implementation identity.
    pub implementation_identity: String,
    /// Status.
    pub status: AdapterReceiptStatus,
    /// Safe labels.
    pub labels: Vec<String>,
    /// Bounded item count.
    pub item_count: u32,
    /// Redaction count.
    pub redaction_count: u32,
    /// Explicit claim boundaries.
    pub non_claims: Vec<&'static str>,
}

/// Receipt validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterReceiptError {
    /// Identity is malformed.
    Identity,
    /// Label or count exceeds a bound.
    Bound,
    /// Receipt contains prohibited private content.
    Leak,
    /// Receipt requests an unsupported claim.
    Overclaim,
}

/// Project a deterministic metadata-only adapter receipt.
///
/// # Errors
/// Returns identity, bound, leak, or overclaim before emitting a receipt identity.
pub fn project_adapter_receipt(
    draft: &AdapterReceiptDraft<'_>,
) -> Result<AdapterReceipt, AdapterReceiptError> {
    for identity in [
        draft.evaluation_identity,
        draft.plan_identity,
        draft.subject_identity,
        draft.policy_identity,
        draft.implementation_identity,
    ] {
        validate_identity(identity).map_err(|_| AdapterReceiptError::Identity)?;
    }
    if draft.labels.len() > MAX_ADAPTER_RECEIPT_LABELS || draft.redaction_count > draft.item_count {
        return Err(AdapterReceiptError::Bound);
    }
    if draft.leaks.any() {
        return Err(AdapterReceiptError::Leak);
    }
    if draft
        .claims
        .iter()
        .any(|claim| *claim != AdapterReceiptClaim::BoundedObservation)
    {
        return Err(AdapterReceiptError::Overclaim);
    }
    for label in draft.labels {
        validate_label(label)?;
    }
    let labels = draft.labels.join(",");
    let counts = format!("{}:{}", draft.item_count, draft.redaction_count);
    let identity = hash_identity(
        "animus.native-evaluation.adapter-receipt.v1",
        &[
            draft.kind.label(),
            draft.evaluation_identity,
            draft.plan_identity,
            draft.subject_identity,
            draft.policy_identity,
            draft.implementation_identity,
            draft.status.label(),
            &labels,
            &counts,
        ],
    );
    Ok(AdapterReceipt {
        identity,
        kind: draft.kind,
        evaluation_identity: draft.evaluation_identity.into(),
        plan_identity: draft.plan_identity.into(),
        subject_identity: draft.subject_identity.into(),
        policy_identity: draft.policy_identity.into(),
        implementation_identity: draft.implementation_identity.into(),
        status: draft.status,
        labels: draft
            .labels
            .iter()
            .map(|label| String::from(*label))
            .collect(),
        item_count: draft.item_count,
        redaction_count: draft.redaction_count,
        non_claims: vec![
            "not-host-effect-execution",
            "not-exactly-once-remote-effects",
            "not-verifier-soundness-or-artifact-authenticity",
            "not-complete-capture-or-benchmark-validity",
            "not-authoritative-completion",
            "not-production-or-release-readiness",
        ],
    })
}

fn validate_label(label: &str) -> Result<(), AdapterReceiptError> {
    if label.is_empty() || label.len() > MAX_ADAPTER_RECEIPT_LABEL_BYTES {
        return Err(AdapterReceiptError::Bound);
    }
    if !label.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
    }) {
        return Err(AdapterReceiptError::Leak);
    }
    Ok(())
}
