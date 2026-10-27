const RECEIPT_SCHEMA: &str = "animus.confirmation-broker.v1";

/// Borrowed fields used to build a safe confirmation receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfirmationReceiptDraft<'a> {
    /// Request that produced the receipt.
    pub request: &'a super::super::ConfirmationRequest,
    /// Final decision status.
    pub decision_status: super::ConfirmationStatus,
    /// Final issue code.
    pub issue: super::ConfirmationIssue,
    /// Broker class that supplied or defaulted the decision.
    pub broker_class: super::ConfirmationBrokerClass,
    /// Timing class for the decision.
    pub timing_class: super::ConfirmationTimingClass,
    /// Expiry class at evaluation time.
    pub expiry_class: super::ConfirmationExpiryClass,
}

/// Safe confirmation receipt with hashes and labels only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationReceipt {
    request_id_hash: super::super::SafeHash,
    action_kind: super::ConfirmationActionKind,
    resource_hash: super::super::SafeHash,
    ability_hash: super::super::SafeHash,
    session_hash: super::super::SafeHash,
    decision_status: super::ConfirmationStatus,
    issue: super::ConfirmationIssue,
    broker_class: super::ConfirmationBrokerClass,
    policy_hash: super::super::SafeHash,
    timing_class: super::ConfirmationTimingClass,
    expiry_class: super::ConfirmationExpiryClass,
    redaction: super::super::RedactionMarker,
    receipt_hash: super::super::SafeHash,
}

impl ConfirmationReceipt {
    /// Builds a safe confirmation receipt from typed request and outcome metadata.
    #[must_use]
    pub fn new(draft: ConfirmationReceiptDraft<'_>) -> Self {
        let request_id_hash = safe_hash_text(draft.request.request_id());
        let resource_hash = safe_hash_text(draft.request.resource());
        let ability_hash = safe_hash_text(draft.request.ability());
        let session_hash = safe_hash_text(draft.request.session_identity());
        let receipt_hash_fields = ReceiptHashFields {
            request: draft.request,
            decision_status: draft.decision_status,
            issue: draft.issue,
            broker_class: draft.broker_class,
            timing_class: draft.timing_class,
            expiry_class: draft.expiry_class,
            request_id_hash: &request_id_hash,
            resource_hash: &resource_hash,
            ability_hash: &ability_hash,
            session_hash: &session_hash,
        };
        let receipt_hash = receipt_hash(&receipt_hash_fields);
        Self {
            request_id_hash,
            action_kind: draft.request.action_kind(),
            resource_hash,
            ability_hash,
            session_hash,
            decision_status: draft.decision_status,
            issue: draft.issue,
            broker_class: draft.broker_class,
            policy_hash: draft.request.policy_hash().clone(),
            timing_class: draft.timing_class,
            expiry_class: draft.expiry_class,
            redaction: draft.request.redaction(),
            receipt_hash,
        }
    }

    /// Returns BLAKE3 hash of the pending request ID.
    #[must_use]
    pub const fn request_id_hash(&self) -> &super::super::SafeHash {
        &self.request_id_hash
    }

    /// Returns protected action kind.
    #[must_use]
    pub const fn action_kind(&self) -> super::ConfirmationActionKind {
        self.action_kind
    }

    /// Returns BLAKE3 hash of the resource identity.
    #[must_use]
    pub const fn resource_hash(&self) -> &super::super::SafeHash {
        &self.resource_hash
    }

    /// Returns BLAKE3 hash of the ability label.
    #[must_use]
    pub const fn ability_hash(&self) -> &super::super::SafeHash {
        &self.ability_hash
    }

    /// Returns BLAKE3 hash of the session or run identity.
    #[must_use]
    pub const fn session_hash(&self) -> &super::super::SafeHash {
        &self.session_hash
    }

    /// Returns confirmation decision status.
    #[must_use]
    pub const fn decision_status(&self) -> super::ConfirmationStatus {
        self.decision_status
    }

    /// Returns issue code.
    #[must_use]
    pub const fn issue(&self) -> super::ConfirmationIssue {
        self.issue
    }

    /// Returns broker class.
    #[must_use]
    pub const fn broker_class(&self) -> super::ConfirmationBrokerClass {
        self.broker_class
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &super::super::SafeHash {
        &self.policy_hash
    }

    /// Returns timing class.
    #[must_use]
    pub const fn timing_class(&self) -> super::ConfirmationTimingClass {
        self.timing_class
    }

    /// Returns expiry class.
    #[must_use]
    pub const fn expiry_class(&self) -> super::ConfirmationExpiryClass {
        self.expiry_class
    }

    /// Returns receipt redaction marker.
    #[must_use]
    pub const fn redaction(&self) -> super::super::RedactionMarker {
        self.redaction
    }

    /// Returns deterministic BLAKE3 hash of safe receipt fields.
    #[must_use]
    pub const fn receipt_hash(&self) -> &super::super::SafeHash {
        &self.receipt_hash
    }
}

struct ReceiptHashFields<'a> {
    request: &'a super::super::ConfirmationRequest,
    decision_status: super::ConfirmationStatus,
    issue: super::ConfirmationIssue,
    broker_class: super::ConfirmationBrokerClass,
    timing_class: super::ConfirmationTimingClass,
    expiry_class: super::ConfirmationExpiryClass,
    request_id_hash: &'a super::super::SafeHash,
    resource_hash: &'a super::super::SafeHash,
    ability_hash: &'a super::super::SafeHash,
    session_hash: &'a super::super::SafeHash,
}

fn receipt_hash(fields: &ReceiptHashFields<'_>) -> super::super::SafeHash {
    safe_hash_parts(&[
        RECEIPT_SCHEMA,
        fields.decision_status.as_str(),
        fields.issue.as_str(),
        fields.broker_class.as_str(),
        fields.timing_class.as_str(),
        fields.expiry_class.as_str(),
        fields.request.action_kind().as_str(),
        fields.request.confirmation_level().as_str(),
        fields.request.redaction().as_str(),
        fields.request.policy_hash().as_str(),
        fields.request_id_hash.as_str(),
        fields.resource_hash.as_str(),
        fields.ability_hash.as_str(),
        fields.session_hash.as_str(),
    ])
}

const RAW_PROMPT_MARKERS: &[&str] = &["RAW_PROMPT", "raw_prompt=", "<raw-prompt>"];
const CREDENTIAL_MARKERS: &[&str] = &[
    "credential=",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "BEGIN PRIVATE KEY",
    "sk-",
];
const FULL_PATH_MARKERS: &[&str] = &["/home/", "C:\\Users\\", "\\Users\\"];
const PROVIDER_PAYLOAD_MARKERS: &[&str] = &["provider_json", "provider payload"];
const RAW_TOOL_PAYLOAD_MARKERS: &[&str] =
    &["RAW_TOOL_PAYLOAD", "raw_tool_payload=", "raw tool payload"];
const SECRET_SUMMARY_MARKERS: &[&str] = &["secret_summary=", "secret summary"];

/// Confirmation receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfirmationReceiptLeak {
    /// Raw prompt text appeared in evidence.
    RawPrompt,
    /// Credential-looking material appeared in evidence.
    Credential,
    /// Full private or absolute host path appeared in evidence.
    FullPath,
    /// Raw provider payload appeared in evidence.
    ProviderPayload,
    /// Raw tool payload appeared in evidence.
    RawToolPayload,
    /// Secret summary appeared in evidence.
    SecretSummary,
}

impl ConfirmationReceiptLeak {
    /// Returns the stable leak label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RawPrompt => "raw_prompt",
            Self::Credential => "credential",
            Self::FullPath => "full_path",
            Self::ProviderPayload => "provider_payload",
            Self::RawToolPayload => "raw_tool_payload",
            Self::SecretSummary => "secret_summary",
        }
    }
}

/// Validates rendered confirmation evidence for unsafe material.
///
/// # Errors
///
/// Returns [`ConfirmationReceiptLeak`] when the evidence includes a raw prompt,
/// credential, full path, provider payload, raw tool payload, or secret summary.
pub fn validate_confirmation_receipt_text(text: &str) -> Result<(), ConfirmationReceiptLeak> {
    first_leak(text).map_or(Ok(()), Err)
}

fn first_leak(text: &str) -> Option<ConfirmationReceiptLeak> {
    LEAK_MARKERS
        .iter()
        .find_map(|(leak, markers)| contains_marker(text, markers).then_some(*leak))
}

fn contains_marker(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| text.contains(marker))
}

const LEAK_MARKERS: &[(ConfirmationReceiptLeak, &[&str])] = &[
    (ConfirmationReceiptLeak::RawPrompt, RAW_PROMPT_MARKERS),
    (ConfirmationReceiptLeak::Credential, CREDENTIAL_MARKERS),
    (ConfirmationReceiptLeak::FullPath, FULL_PATH_MARKERS),
    (
        ConfirmationReceiptLeak::ProviderPayload,
        PROVIDER_PAYLOAD_MARKERS,
    ),
    (
        ConfirmationReceiptLeak::RawToolPayload,
        RAW_TOOL_PAYLOAD_MARKERS,
    ),
    (
        ConfirmationReceiptLeak::SecretSummary,
        SECRET_SUMMARY_MARKERS,
    ),
];

pub(super) fn safe_hash_parts(parts: &[&str]) -> super::super::SafeHash {
    let mut canonical = alloc::string::String::new();
    for part in parts {
        canonical.push_str(part);
        canonical.push('\n');
    }
    safe_hash_text(canonical.as_str())
}

pub(in crate::effect::approval) fn safe_hash_text(text: &str) -> super::super::SafeHash {
    super::super::SafeHash::from_blake3_bytes(text.as_bytes())
}
