const RECEIPT_TEXT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("authorization_receipt_text");
const RAW_PROMPT_MARKER: &str = "RAW_PROMPT";
const CREDENTIAL_MARKER: &str = "credential=";
const PRIVATE_HOME_PATH_MARKER: &str = "/home/";
const PROVIDER_PAYLOAD_MARKER: &str = "provider payload";
const RAW_TOOL_BODY_MARKER: &str = "raw tool body";
const RAW_MODEL_OUTPUT_MARKER: &str = "raw model output";

/// Redacted authorization receipt for one guarded action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationReceipt {
    status: super::AuthorizationStatus,
    issue: super::AuthorizationIssue,
    action: super::GuardedAction,
    contract: alloc::string::String,
    ability: alloc::string::String,
    resource_hash: alloc::string::String,
    policy_hash: alloc::string::String,
    checked_grants: u32,
    caveats: super::CaveatSummary,
    sandbox_hash: Option<super::super::SafeHash>,
    receipt_hash: alloc::string::String,
}

impl AuthorizationReceipt {
    /// Builds a receipt from already-authorized safe fields.
    #[must_use]
    pub(super) fn new(fields: ReceiptFields) -> Self {
        Self {
            status: fields.status,
            issue: fields.issue,
            action: fields.action,
            contract: fields.contract,
            ability: fields.ability,
            resource_hash: fields.resource_hash,
            policy_hash: fields.policy_hash,
            checked_grants: fields.checked_grants,
            caveats: fields.caveats,
            sandbox_hash: fields.sandbox_hash,
            receipt_hash: fields.receipt_hash,
        }
    }

    /// Returns true when the guarded action is authorized.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self.status, super::AuthorizationStatus::Allowed)
    }

    /// Returns authorization status.
    #[must_use]
    pub const fn status(&self) -> super::AuthorizationStatus {
        self.status
    }

    /// Returns issue code.
    #[must_use]
    pub const fn issue(&self) -> super::AuthorizationIssue {
        self.issue
    }

    /// Returns guarded action class.
    #[must_use]
    pub const fn action(&self) -> super::GuardedAction {
        self.action
    }

    /// Returns contract identifier.
    #[must_use]
    pub fn contract(&self) -> &str {
        self.contract.as_str()
    }

    /// Returns ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Returns BLAKE3 hash of the resource identifier.
    #[must_use]
    pub fn resource_hash(&self) -> &str {
        self.resource_hash.as_str()
    }

    /// Returns BLAKE3 hash of checked policy fields.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the number of checked matching UCAN grants.
    #[must_use]
    pub const fn checked_grants(&self) -> u32 {
        self.checked_grants
    }

    /// Returns local caveat counts.
    #[must_use]
    pub const fn caveats(&self) -> super::CaveatSummary {
        self.caveats
    }

    /// Returns the sandbox descriptor hash when a sandbox was required.
    #[must_use]
    pub const fn sandbox_hash(&self) -> Option<&super::super::SafeHash> {
        self.sandbox_hash.as_ref()
    }

    /// Returns BLAKE3 hash of safe receipt fields.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

pub(super) struct ReceiptFields {
    pub(super) status: super::AuthorizationStatus,
    pub(super) issue: super::AuthorizationIssue,
    pub(super) action: super::GuardedAction,
    pub(super) contract: alloc::string::String,
    pub(super) ability: alloc::string::String,
    pub(super) resource_hash: alloc::string::String,
    pub(super) policy_hash: alloc::string::String,
    pub(super) checked_grants: u32,
    pub(super) caveats: super::CaveatSummary,
    pub(super) sandbox_hash: Option<super::super::SafeHash>,
    pub(super) receipt_hash: alloc::string::String,
}

/// Validates that textual receipt evidence does not contain known unsafe raw material.
///
/// # Errors
///
/// Returns [`super::AuthorizationLeak`] when the evidence text includes a raw prompt,
/// credential marker, private absolute path, provider payload, raw tool body, or
/// raw model output marker.
pub fn validate_authorization_receipt_text(text: &str) -> Result<(), super::AuthorizationLeak> {
    debug_assert!(!RAW_PROMPT_MARKER.is_empty());
    debug_assert!(!CREDENTIAL_MARKER.is_empty());
    debug_assert!(!PRIVATE_HOME_PATH_MARKER.is_empty());
    crate::validation::normalized_adapter_field(RECEIPT_TEXT_FIELD, text)
        .map_err(|_error| super::AuthorizationLeak::RawPrompt)?;
    if text.contains(RAW_PROMPT_MARKER) {
        return Err(super::AuthorizationLeak::RawPrompt);
    }
    if text.contains(CREDENTIAL_MARKER) {
        return Err(super::AuthorizationLeak::Credential);
    }
    if text.contains(PRIVATE_HOME_PATH_MARKER) {
        return Err(super::AuthorizationLeak::PrivatePath);
    }
    if text.contains(PROVIDER_PAYLOAD_MARKER) {
        return Err(super::AuthorizationLeak::ProviderPayload);
    }
    if text.contains(RAW_TOOL_BODY_MARKER) {
        return Err(super::AuthorizationLeak::RawToolBody);
    }
    if text.contains(RAW_MODEL_OUTPUT_MARKER) {
        return Err(super::AuthorizationLeak::RawModelOutput);
    }
    Ok(())
}

pub(super) struct ReceiptHashFields<'a> {
    pub(super) status: super::AuthorizationStatus,
    pub(super) issue: super::AuthorizationIssue,
    pub(super) request: &'a super::CapabilityRequest,
    pub(super) contract: &'a super::CapabilityPolicyContract,
    pub(super) resource_hash: &'a str,
    pub(super) checked_grants: u32,
    pub(super) caveats: super::CaveatSummary,
    pub(super) sandbox_hash: Option<&'a super::super::SafeHash>,
}

pub(super) fn receipt_hash(fields: &ReceiptHashFields<'_>) -> alloc::string::String {
    debug_assert!(!fields.request.contract().is_empty());
    debug_assert!(!fields.request.resource().is_empty());
    debug_assert!(!fields.resource_hash.is_empty());
    debug_assert!(!fields.contract.policy_hash().is_empty());
    let checked_grants = alloc::format!("{}", fields.checked_grants);
    let caveat_summary = alloc::format!(
        "allow:{};deny:{};ask:{}",
        fields.caveats.allows(),
        fields.caveats.denies(),
        fields.caveats.asks()
    );
    let sandbox = fields
        .sandbox_hash
        .map_or("none", super::super::SafeHash::as_str);
    super::schema::hash_parts(&[
        super::schema::HASH_SCHEMA,
        fields.status.as_str(),
        fields.issue.as_str(),
        fields.request.action().as_str(),
        fields.request.contract(),
        fields.request.ability(),
        fields.resource_hash,
        fields.contract.policy_hash(),
        checked_grants.as_str(),
        caveat_summary.as_str(),
        sandbox,
    ])
}
