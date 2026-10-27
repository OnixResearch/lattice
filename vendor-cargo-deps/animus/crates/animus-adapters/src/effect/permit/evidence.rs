const RAW_PROMPT_MARKER: &str = "raw_prompt=";
const CREDENTIAL_MARKER: &str = "credential=";
const SECRET_MARKER: &str = "secret=";
const PROVIDER_PAYLOAD_MARKER: &str = "provider_json";
const RAW_TOOL_BODY_MARKER: &str = "raw_tool_body=";
const UNIX_PATH_MARKER: &str = "/home/";
const WINDOWS_PATH_MARKER: &str = ":\\";
const UNBOUNDED_OUTPUT_MARKER: &str = "unbounded_output=";

/// Borrowed receipt fields for safe permission evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptDraft<'a> {
    /// Guarded action class.
    pub action: super::super::GuardedAction,
    /// Safe BLAKE3 resource hash.
    pub resource_hash: &'a crate::SafeHash,
    /// Authority status from Nickel/UCAN/sandbox checks.
    pub authority_status: super::super::AuthorizationStatus,
    /// Authority issue from Nickel/UCAN/sandbox checks.
    pub authority_issue: super::super::AuthorizationIssue,
    /// Selected local outcome.
    pub local_outcome: Option<super::Outcome>,
    /// Prompt status after resolution.
    pub prompt_status: super::PromptStatus,
    /// BLAKE3 hash of selected local policy.
    pub policy_hash: &'a crate::SafeHash,
    /// Decision reason label.
    pub reason: &'a str,
    /// Redaction markers proving unsafe material was omitted.
    pub redaction_markers: &'a [&'a str],
}

/// Safe permission receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    action: super::super::GuardedAction,
    resource_hash: crate::SafeHash,
    authority_status: super::super::AuthorizationStatus,
    authority_issue: super::super::AuthorizationIssue,
    local_outcome: Option<super::Outcome>,
    prompt_status: super::PromptStatus,
    policy_hash: crate::SafeHash,
    reason: super::Text,
    redaction_markers: super::Collection<super::Text>,
}

impl Receipt {
    /// Builds safe permission evidence from labels, statuses, hashes, and redactions.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when reason or redaction labels are malformed or
    /// exceed collection bounds.
    pub fn new(draft: ReceiptDraft<'_>) -> Result<Self, super::Error> {
        validate_redaction_count(draft.redaction_markers.len())?;
        Ok(Self {
            action: draft.action,
            resource_hash: draft.resource_hash.clone(),
            authority_status: draft.authority_status,
            authority_issue: draft.authority_issue,
            local_outcome: draft.local_outcome,
            prompt_status: draft.prompt_status,
            policy_hash: draft.policy_hash.clone(),
            reason: super::required_text_value(super::REASON_FIELD, draft.reason)?,
            redaction_markers: redactions(draft.redaction_markers)?,
        })
    }

    /// Returns the action class.
    #[must_use]
    pub const fn action(&self) -> super::super::GuardedAction {
        self.action
    }

    /// Returns the resource hash.
    #[must_use]
    pub const fn resource_hash(&self) -> &crate::SafeHash {
        &self.resource_hash
    }

    /// Returns authority status.
    #[must_use]
    pub const fn authority_status(&self) -> super::super::AuthorizationStatus {
        self.authority_status
    }

    /// Returns authority issue.
    #[must_use]
    pub const fn authority_issue(&self) -> super::super::AuthorizationIssue {
        self.authority_issue
    }

    /// Returns local outcome.
    #[must_use]
    pub const fn local_outcome(&self) -> Option<super::Outcome> {
        self.local_outcome
    }

    /// Returns prompt status.
    #[must_use]
    pub const fn prompt_status(&self) -> super::PromptStatus {
        self.prompt_status
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::SafeHash {
        &self.policy_hash
    }

    /// Returns decision reason.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }

    /// Returns sorted redaction markers.
    #[must_use]
    pub fn redaction_markers(&self) -> &[super::Text] {
        self.redaction_markers.as_slice()
    }
}

/// Unsafe permission receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptLeak {
    /// Raw prompt text leaked.
    RawPrompt,
    /// Credential-looking material leaked.
    Credential,
    /// Raw provider payload leaked.
    ProviderPayload,
    /// Raw tool body leaked.
    RawToolBody,
    /// Full private path leaked.
    FullPath,
    /// Unbounded output leaked.
    UnboundedOutput,
}

/// Validates rendered permission receipt text for unsafe material.
///
/// # Errors
///
/// Returns [`ReceiptLeak`] when known unsafe sentinels appear.
pub fn validate_text(text: &str) -> Result<(), ReceiptLeak> {
    assert!(
        text.len() >= text.chars().count(),
        "text length bounds chars"
    );
    assert!(
        !RAW_PROMPT_MARKER.is_empty(),
        "leak marker must not be empty"
    );
    if text.contains(RAW_PROMPT_MARKER) {
        return Err(ReceiptLeak::RawPrompt);
    }
    if text.contains(CREDENTIAL_MARKER) || text.contains(SECRET_MARKER) {
        return Err(ReceiptLeak::Credential);
    }
    if text.contains(PROVIDER_PAYLOAD_MARKER) {
        return Err(ReceiptLeak::ProviderPayload);
    }
    if text.contains(RAW_TOOL_BODY_MARKER) {
        return Err(ReceiptLeak::RawToolBody);
    }
    if text.contains(UNIX_PATH_MARKER) || text.contains(WINDOWS_PATH_MARKER) {
        return Err(ReceiptLeak::FullPath);
    }
    if text.contains(UNBOUNDED_OUTPUT_MARKER) {
        return Err(ReceiptLeak::UnboundedOutput);
    }
    Ok(())
}

pub(super) fn redactions(values: &[&str]) -> Result<super::Collection<super::Text>, super::Error> {
    let mut redaction_markers = super::Collection::with_capacity(values.len());
    for marker in values {
        redaction_markers.push(super::required_text_value(super::REDACTION_FIELD, marker)?);
    }
    redaction_markers.sort();
    redaction_markers.dedup();
    Ok(redaction_markers)
}

pub(super) fn validate_redaction_count(count: usize) -> Result<(), super::Error> {
    if count > super::MAX_REDACTIONS {
        return Err(super::Error::TooManyRedactions {
            max_items: super::MAX_REDACTIONS,
            actual_items: count,
        });
    }
    Ok(())
}
