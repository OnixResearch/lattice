const RAW_PROMPT_RECEIPT_MARKER: &str = "raw_prompt=";
const PROMPT_BODY_RECEIPT_MARKER: &str = "prompt_body=";
const UNIX_HOME_PATH_MARKER: &str = "/home/";
const MAC_HOME_PATH_MARKER: &str = "/Users/";
const WINDOWS_DRIVE_PATH_MARKER: &str = ":\\";
const SECRET_ASSIGNMENT_MARKER: &str = "secret=";
const CREDENTIAL_ASSIGNMENT_MARKER: &str = "credential=";
const ACCESS_KEY_MARKER: &str = "AKIA";
const PROVIDER_JSON_MARKER: &str = "provider_json";
const PROVIDER_FIELD_MARKER: &str = "\"provider\"";
const HIDDEN_MEMORY_MARKER: &str = "hidden_memory=";
const MEMORY_BODY_MARKER: &str = "memory_body=";

/// Borrowed safe activation/discovery receipt fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReceiptDraft<'a> {
    /// Agent name recorded in the receipt.
    pub definition_name: &'a str,
    /// Source scope recorded in the receipt.
    pub source_scope: super::SourceScope,
    /// Trust status recorded in the receipt.
    pub trust_status: super::TrustStatus,
    /// Admission status recorded in the receipt.
    pub admission_status: super::AdmissionStatus,
    /// Effective capability ceiling labels recorded in the receipt.
    pub capability_ceilings: &'a [&'a str],
    /// Selected model role label recorded in the receipt.
    pub selected_model_role: Option<&'a str>,
    /// BLAKE3 hash of the canonical manifest bytes.
    pub manifest_hash: &'a crate::SafeHash,
    /// BLAKE3 hash of the active policy/admission facts.
    pub policy_hash: &'a crate::SafeHash,
    /// Redaction markers proving omitted unsafe payload classes.
    pub redaction_markers: &'a [&'a str],
}

/// Safe agent-definition receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    definition_name: super::Text,
    source_scope: super::SourceScope,
    trust_status: super::TrustStatus,
    admission_status: super::AdmissionStatus,
    capability_ceilings: super::Collection<super::Text>,
    selected_model_role: Option<super::Text>,
    manifest_hash: crate::SafeHash,
    policy_hash: crate::SafeHash,
    redaction_markers: super::Collection<super::Text>,
}

impl Receipt {
    /// Builds a receipt that contains metadata, labels, hashes, and redactions only.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when labels or redaction markers are malformed,
    /// missing, or exceed bounded collection limits.
    pub fn new(draft: ReceiptDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            definition_name: super::required_text_value(super::NAME_FIELD, draft.definition_name)?,
            source_scope: draft.source_scope,
            trust_status: draft.trust_status,
            admission_status: draft.admission_status,
            capability_ceilings: super::optional_label_set(
                super::CAPABILITY_FIELD,
                draft.capability_ceilings,
            )?,
            selected_model_role: selected_model_role(draft.selected_model_role)?,
            manifest_hash: draft.manifest_hash.clone(),
            policy_hash: draft.policy_hash.clone(),
            redaction_markers: super::required_label_set(
                super::REDACTION_MARKER_FIELD,
                draft.redaction_markers,
            )?,
        })
    }

    /// Returns the safe agent name.
    #[must_use]
    pub fn definition_name(&self) -> &str {
        self.definition_name.as_str()
    }

    /// Returns the source scope.
    #[must_use]
    pub const fn source_scope(&self) -> super::SourceScope {
        self.source_scope
    }

    /// Returns the trust status.
    #[must_use]
    pub const fn trust_status(&self) -> super::TrustStatus {
        self.trust_status
    }

    /// Returns the admission status.
    #[must_use]
    pub const fn admission_status(&self) -> super::AdmissionStatus {
        self.admission_status
    }

    /// Returns effective capability ceilings.
    #[must_use]
    pub fn capability_ceilings(&self) -> &[super::Text] {
        self.capability_ceilings.as_slice()
    }

    /// Returns the selected model role label.
    #[must_use]
    pub fn selected_model_role(&self) -> Option<&str> {
        self.selected_model_role.as_deref()
    }

    /// Returns the canonical manifest hash.
    #[must_use]
    pub const fn manifest_hash(&self) -> &crate::SafeHash {
        &self.manifest_hash
    }

    /// Returns the active policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::SafeHash {
        &self.policy_hash
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redaction_markers(&self) -> &[super::Text] {
        self.redaction_markers.as_slice()
    }
}

/// Receipt leak classes rejected by rendered receipt validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptLeak {
    /// Receipt included raw prompt material.
    RawPromptBody,
    /// Receipt included a host/project full path.
    FullPath,
    /// Receipt included credential-looking material.
    Credential,
    /// Receipt included raw provider payload data.
    ProviderPayload,
    /// Receipt included hidden memory body text.
    HiddenMemoryBody,
}

/// Validates rendered receipt text for unsafe payload sentinels.
///
/// # Errors
///
/// Returns [`ReceiptLeak`] when the rendered receipt includes a known unsafe
/// prompt, path, credential, provider-payload, or memory-body marker.
pub fn validate_text(text: &str) -> Result<(), ReceiptLeak> {
    assert!(
        text.len() >= text.chars().count(),
        "text length bounds chars"
    );
    assert!(
        !RAW_PROMPT_RECEIPT_MARKER.is_empty(),
        "receipt leak markers must not be empty"
    );
    if has_prompt_leak(text) {
        return Err(ReceiptLeak::RawPromptBody);
    }
    if has_path_leak(text) {
        return Err(ReceiptLeak::FullPath);
    }
    if has_credential_leak(text) {
        return Err(ReceiptLeak::Credential);
    }
    if has_provider_leak(text) {
        return Err(ReceiptLeak::ProviderPayload);
    }
    if has_memory_leak(text) {
        return Err(ReceiptLeak::HiddenMemoryBody);
    }
    Ok(())
}

fn selected_model_role(value: Option<&str>) -> Result<Option<super::Text>, super::Error> {
    value
        .map(|actual| super::required_text_value(super::MODEL_FIELD, actual))
        .transpose()
}

fn has_prompt_leak(text: &str) -> bool {
    text.contains(RAW_PROMPT_RECEIPT_MARKER) || text.contains(PROMPT_BODY_RECEIPT_MARKER)
}

fn has_path_leak(text: &str) -> bool {
    text.contains(UNIX_HOME_PATH_MARKER)
        || text.contains(MAC_HOME_PATH_MARKER)
        || text.contains(WINDOWS_DRIVE_PATH_MARKER)
}

fn has_credential_leak(text: &str) -> bool {
    text.contains(SECRET_ASSIGNMENT_MARKER)
        || text.contains(CREDENTIAL_ASSIGNMENT_MARKER)
        || text.contains(ACCESS_KEY_MARKER)
}

fn has_provider_leak(text: &str) -> bool {
    text.contains(PROVIDER_JSON_MARKER) || text.contains(PROVIDER_FIELD_MARKER)
}

fn has_memory_leak(text: &str) -> bool {
    text.contains(HIDDEN_MEMORY_MARKER) || text.contains(MEMORY_BODY_MARKER)
}
