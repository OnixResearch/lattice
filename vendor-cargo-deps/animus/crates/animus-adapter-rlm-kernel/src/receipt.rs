use alloc::string::String;
use alloc::vec::Vec;

const MAX_SAFE_LABEL_CHARS: u32 = 256;
const MAX_RECEIPT_ITEMS: usize = 32;
const RECEIPT_IDENTITY_DOMAIN: &str = "animus.rlm-kernel-receipt.v1";
const PROHIBITED_CLAIMS: &[&str] = &[
    "security_sandbox",
    "security_sandboxing",
    "model_correctness",
    "task_success",
    "task_completion",
    "production_readiness",
    "release_eligibility",
];

/// RLM kernel receipt status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RlmKernelStatus {
    /// A bounded operation completed.
    Completed,
    /// Admission denied the operation before effects.
    Denied,
    /// Evaluation failed at the Steel boundary.
    Failed,
    /// Compaction preserved kernel state.
    Preserved,
    /// An explicit host clear reset the kernel.
    Cleared,
}

impl RlmKernelStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Denied => "denied",
            Self::Failed => "failed",
            Self::Preserved => "preserved",
            Self::Cleared => "cleared",
        }
    }
}

/// Prohibited receipt content class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptLeakClass {
    /// A raw kernel variable value was supplied.
    VariableValue,
    /// A model-written code body was supplied.
    CodeBody,
    /// A child transcript was supplied.
    ChildTranscript,
    /// Credential material was supplied.
    Credential,
    /// A raw host path was supplied.
    RawPath,
}

impl ReceiptLeakClass {
    /// Returns the stable leak label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::VariableValue => "variable_value",
            Self::CodeBody => "code_body",
            Self::ChildTranscript => "child_transcript",
            Self::Credential => "credential",
            Self::RawPath => "raw_path",
        }
    }
}

/// One bounded metadata-only receipt count.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReceiptCount {
    label: String,
    value: u64,
}

impl ReceiptCount {
    /// Builds a safe count.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the label is malformed.
    pub fn new(label: &str, value: u64) -> Result<Self, crate::Error> {
        Ok(Self {
            label: safe_label("count_label", label)?,
            value,
        })
    }

    /// Returns the safe count label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the bounded count value.
    #[must_use]
    pub const fn value(&self) -> u64 {
        self.value
    }
}

/// Borrowed RLM kernel receipt facts before metadata-only validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptDraft<'a> {
    /// Receipt status.
    pub status: RlmKernelStatus,
    /// Domain-separated BLAKE3 policy identity.
    pub policy_identity: &'a str,
    /// Observed operation classes.
    pub operation_classes: &'a [crate::HostOperationClass],
    /// Bounded structural counts.
    pub counts: &'a [ReceiptCount],
    /// Safe child-lineage BLAKE3 identities.
    pub child_lineage_hashes: &'a [&'a str],
    /// Safe context or artifact BLAKE3 identities.
    pub artifact_hashes: &'a [&'a str],
    /// Whether kernel state survived compaction.
    pub kernel_state_preserved: bool,
    /// Required non-claims.
    pub non_claims: &'a [&'a str],
    /// Optional raw variable value. Validation always rejects this field.
    pub variable_value: Option<&'a str>,
    /// Optional model-written code body. Validation always rejects this field.
    pub code_body: Option<&'a str>,
    /// Optional child transcript. Validation always rejects this field.
    pub child_transcript: Option<&'a str>,
    /// Optional credential. Validation always rejects this field.
    pub credential: Option<&'a str>,
    /// Optional raw path. Validation always rejects this field.
    pub raw_path: Option<&'a str>,
    /// Optional positive claim labels checked against prohibited claims.
    pub claims: &'a [&'a str],
}

/// Validated metadata-only RLM kernel receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    status: RlmKernelStatus,
    policy_identity: String,
    operation_classes: Vec<crate::HostOperationClass>,
    counts: Vec<ReceiptCount>,
    child_lineage_hashes: Vec<String>,
    artifact_hashes: Vec<String>,
    kernel_state_preserved: bool,
    non_claims: Vec<String>,
    identity: String,
}

impl Receipt {
    /// Validates metadata-only receipt facts without retaining raw content.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the draft leaks payloads, contains a
    /// prohibited claim, has malformed hashes, omits non-claims, or contradicts
    /// its preservation status.
    pub fn from_draft(draft: &ReceiptDraft<'_>) -> Result<Self, crate::Error> {
        reject_optional_leak(draft.variable_value, ReceiptLeakClass::VariableValue)?;
        reject_optional_leak(draft.code_body, ReceiptLeakClass::CodeBody)?;
        reject_optional_leak(draft.child_transcript, ReceiptLeakClass::ChildTranscript)?;
        reject_optional_leak(draft.credential, ReceiptLeakClass::Credential)?;
        reject_optional_leak(draft.raw_path, ReceiptLeakClass::RawPath)?;
        reject_prohibited_claims(draft.claims)?;
        validate_hash("policy_identity", draft.policy_identity)?;
        let operation_classes = canonical_operations(draft.operation_classes)?;
        let counts = canonical_counts(draft.counts)?;
        let child_lineage_hashes = canonical_hashes("child_lineage_hashes", draft.child_lineage_hashes)?;
        let artifact_hashes = canonical_hashes("artifact_hashes", draft.artifact_hashes)?;
        let non_claims = canonical_non_claims(draft.non_claims)?;
        validate_preservation(draft.status, draft.kernel_state_preserved)?;
        let identity = receipt_identity(&ReceiptIdentityInputs {
            draft,
            operations: &operation_classes,
            counts: &counts,
            child_hashes: &child_lineage_hashes,
            artifact_hashes: &artifact_hashes,
            non_claims: &non_claims,
        });
        Ok(Self {
            status: draft.status,
            policy_identity: String::from(draft.policy_identity),
            operation_classes,
            counts,
            child_lineage_hashes,
            artifact_hashes,
            kernel_state_preserved: draft.kernel_state_preserved,
            non_claims,
            identity,
        })
    }

    /// Returns the receipt schema.
    #[must_use]
    pub const fn schema() -> &'static str {
        crate::RLM_KERNEL_RECEIPT_SCHEMA
    }

    /// Returns the receipt status.
    #[must_use]
    pub const fn status(&self) -> RlmKernelStatus {
        self.status
    }

    /// Returns the policy identity.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        self.policy_identity.as_str()
    }

    /// Returns canonical observed operation classes.
    #[must_use]
    pub fn operation_classes(&self) -> &[crate::HostOperationClass] {
        self.operation_classes.as_slice()
    }

    /// Returns canonical bounded counts.
    #[must_use]
    pub fn counts(&self) -> &[ReceiptCount] {
        self.counts.as_slice()
    }

    /// Returns safe child-lineage identities.
    #[must_use]
    pub fn child_lineage_hashes(&self) -> &[String] {
        self.child_lineage_hashes.as_slice()
    }

    /// Returns safe artifact identities.
    #[must_use]
    pub fn artifact_hashes(&self) -> &[String] {
        self.artifact_hashes.as_slice()
    }

    /// Returns whether compaction preserved kernel state.
    #[must_use]
    pub const fn kernel_state_preserved(&self) -> bool {
        self.kernel_state_preserved
    }

    /// Returns required non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[String] {
        self.non_claims.as_slice()
    }

    /// Returns the domain-separated BLAKE3 receipt identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

fn reject_optional_leak(value: Option<&str>, class: ReceiptLeakClass) -> Result<(), crate::Error> {
    if value.is_some() {
        return Err(crate::Error::ReceiptLeak { class });
    }
    Ok(())
}

fn reject_prohibited_claims(claims: &[&str]) -> Result<(), crate::Error> {
    for claim in claims {
        let normalized = safe_label("claim", claim)?;
        if PROHIBITED_CLAIMS.iter().any(|prohibited| normalized == *prohibited) {
            return Err(crate::Error::ProhibitedClaim);
        }
    }
    Ok(())
}

fn canonical_operations(
    operations: &[crate::HostOperationClass],
) -> Result<Vec<crate::HostOperationClass>, crate::Error> {
    crate::ensure_at_most(
        "receipt_operation_classes",
        crate::usize_to_u64(operations.len()),
        crate::usize_to_u64(MAX_RECEIPT_ITEMS),
    )?;
    let mut values = operations.to_vec();
    values.sort();
    values.dedup();
    Ok(values)
}

fn canonical_counts(values: &[ReceiptCount]) -> Result<Vec<ReceiptCount>, crate::Error> {
    crate::ensure_at_most("receipt_counts", crate::usize_to_u64(values.len()), crate::usize_to_u64(MAX_RECEIPT_ITEMS))?;
    let mut canonical = values.to_vec();
    canonical.sort();
    let original_len = canonical.len();
    canonical.dedup_by(|left, right| left.label == right.label);
    if canonical.len() != original_len {
        return Err(crate::Error::DuplicateIdentity {
            field: "receipt_counts",
        });
    }
    Ok(canonical)
}

fn canonical_hashes(field: &'static str, values: &[&str]) -> Result<Vec<String>, crate::Error> {
    crate::ensure_at_most(field, crate::usize_to_u64(values.len()), crate::usize_to_u64(MAX_RECEIPT_ITEMS))?;
    let mut canonical = values
        .iter()
        .map(|value| {
            validate_hash(field, value)?;
            Ok(String::from(*value))
        })
        .collect::<Result<Vec<_>, crate::Error>>()?;
    canonical.sort();
    canonical.dedup();
    Ok(canonical)
}

fn canonical_non_claims(values: &[&str]) -> Result<Vec<String>, crate::Error> {
    let mut canonical = values.iter().map(|value| safe_label("non_claim", value)).collect::<Result<Vec<_>, _>>()?;
    canonical.sort();
    canonical.dedup();
    for required in crate::REQUIRED_NON_CLAIMS {
        if !canonical.iter().any(|value| value == required) {
            return Err(crate::Error::MissingNonClaim { non_claim: required });
        }
    }
    Ok(canonical)
}

fn validate_preservation(status: RlmKernelStatus, preserved: bool) -> Result<(), crate::Error> {
    if (status == RlmKernelStatus::Preserved && !preserved) || (status == RlmKernelStatus::Cleared && preserved) {
        return Err(crate::Error::InconsistentReceipt);
    }
    Ok(())
}

fn safe_label(field: &'static str, value: &str) -> Result<String, crate::Error> {
    let label = crate::required_text(field, value, MAX_SAFE_LABEL_CHARS)?;
    if label.starts_with('/')
        || label.contains("/home/")
        || label.contains("token=")
        || label.contains("api_key")
        || label.contains("(define ")
        || label.contains("assistant:")
    {
        return Err(crate::Error::ReceiptLeak {
            class: ReceiptLeakClass::RawPath,
        });
    }
    Ok(label)
}

fn validate_hash(field: &'static str, value: &str) -> Result<(), crate::Error> {
    if crate::is_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(crate::Error::InvalidBlake3Identity { field })
    }
}

struct ReceiptIdentityInputs<'a> {
    draft: &'a ReceiptDraft<'a>,
    operations: &'a [crate::HostOperationClass],
    counts: &'a [ReceiptCount],
    child_hashes: &'a [String],
    artifact_hashes: &'a [String],
    non_claims: &'a [String],
}

fn receipt_identity(inputs: &ReceiptIdentityInputs<'_>) -> String {
    let draft = inputs.draft;
    let operation_text = inputs.operations.iter().map(|operation| operation.as_str()).collect::<Vec<_>>().join(",");
    let count_text = inputs
        .counts
        .iter()
        .map(|count| alloc::format!("{}={}", count.label, count.value))
        .collect::<Vec<_>>()
        .join(",");
    let child_text = inputs.child_hashes.join(",");
    let artifact_text = inputs.artifact_hashes.join(",");
    let non_claim_text = inputs.non_claims.join(",");
    let preserved = if draft.kernel_state_preserved {
        "preserved"
    } else {
        "not-preserved"
    };
    crate::hash_parts(RECEIPT_IDENTITY_DOMAIN, &[
        draft.status.as_str(),
        draft.policy_identity,
        operation_text.as_str(),
        count_text.as_str(),
        child_text.as_str(),
        artifact_text.as_str(),
        preserved,
        non_claim_text.as_str(),
    ])
}
