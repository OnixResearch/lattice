const REQUEST_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("confirmation_request_id");
const RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("confirmation_resource");
const ABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("confirmation_ability");
const SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("confirmation_summary");
const SESSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("confirmation_session_identity");
const DEFAULT_REQUEST_ID: &str = "compat-confirmation-request";
const DEFAULT_RESOURCE_LABEL: &str = "compat-resource";
const DEFAULT_ABILITY_LABEL: &str = "confirm";
const DEFAULT_SESSION_IDENTITY: &str = "compat-session";
const DEFAULT_POLICY_IDENTITY: &str = "animus.confirmation.default-policy";

/// Borrowed fields required to build a typed confirmation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfirmationRequestDraft<'a> {
    /// Unique pending request identity supplied by the runtime or host shell.
    pub request_id: &'a str,
    /// Protected action class being confirmed.
    pub action_kind: super::ConfirmationActionKind,
    /// Redacted resource label or stable resource identity.
    pub resource: &'a str,
    /// Ability label scoped to the protected action.
    pub ability: &'a str,
    /// Safe human-readable summary; never copied into confirmation receipts.
    pub safe_summary: &'a str,
    /// BLAKE3 hash of the active policy facts that required confirmation.
    pub policy_hash: &'a super::super::SafeHash,
    /// Session or run identity label used to bind approval to one execution.
    pub session_identity: &'a str,
    /// Required confirmation strength.
    pub confirmation_level: super::ConfirmationLevel,
    /// Optional expiry for the pending request.
    pub expiry: Option<super::ConfirmationExpiry>,
    /// Redaction marker carried into safe receipts.
    pub redaction: super::super::RedactionMarker,
}

/// Typed protected-action confirmation request passed to a broker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationRequest {
    request_id: alloc::string::String,
    action_kind: super::ConfirmationActionKind,
    resource: alloc::string::String,
    ability: alloc::string::String,
    safe_summary: alloc::string::String,
    policy_hash: super::super::SafeHash,
    session_identity: alloc::string::String,
    confirmation_level: super::ConfirmationLevel,
    expiry: Option<super::ConfirmationExpiry>,
    redaction: super::super::RedactionMarker,
}

impl ConfirmationRequest {
    /// Builds a compatibility confirmation request from a broad capability and summary.
    ///
    /// Prefer [`Self::from_draft`] for new protected-action boundaries that can
    /// supply explicit action, resource, ability, policy, session, and expiry data.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the summary fails adapter
    /// field validation.
    pub fn new(
        capability: super::super::CapabilityClass,
        summary: &str,
    ) -> Result<Self, crate::AdapterContractError> {
        let policy_hash = super::gate::record::evidence::safe_hash_text(DEFAULT_POLICY_IDENTITY);
        Self::from_draft(ConfirmationRequestDraft {
            request_id: DEFAULT_REQUEST_ID,
            action_kind: super::ConfirmationActionKind::from_capability(capability),
            resource: DEFAULT_RESOURCE_LABEL,
            ability: DEFAULT_ABILITY_LABEL,
            safe_summary: summary,
            policy_hash: &policy_hash,
            session_identity: DEFAULT_SESSION_IDENTITY,
            confirmation_level: super::ConfirmationLevel::Explicit,
            expiry: None,
            redaction: super::super::RedactionMarker::PayloadNotCaptured,
        })
    }

    /// Builds a typed protected-action confirmation request.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when a required string field is
    /// empty, too long, or contains a NUL character.
    pub fn from_draft(
        draft: ConfirmationRequestDraft<'_>,
    ) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            request_id: required_text(REQUEST_ID_FIELD, draft.request_id)?,
            action_kind: draft.action_kind,
            resource: required_text(RESOURCE_FIELD, draft.resource)?,
            ability: required_text(ABILITY_FIELD, draft.ability)?,
            safe_summary: required_text(SUMMARY_FIELD, draft.safe_summary)?,
            policy_hash: draft.policy_hash.clone(),
            session_identity: required_text(SESSION_FIELD, draft.session_identity)?,
            confirmation_level: draft.confirmation_level,
            expiry: draft.expiry,
            redaction: draft.redaction,
        })
    }

    /// Returns the broad capability class for generic service receipts.
    #[must_use]
    pub const fn capability(&self) -> super::super::CapabilityClass {
        self.action_kind.capability()
    }

    /// Returns the pending confirmation request ID.
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.request_id.as_str()
    }

    /// Returns the protected action kind.
    #[must_use]
    pub const fn action_kind(&self) -> super::ConfirmationActionKind {
        self.action_kind
    }

    /// Returns the redacted resource label.
    #[must_use]
    pub fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Returns the protected action ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Returns the safe summary for host-local prompting.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.safe_summary.as_str()
    }

    /// Returns the policy hash that required confirmation.
    #[must_use]
    pub const fn policy_hash(&self) -> &super::super::SafeHash {
        &self.policy_hash
    }

    /// Returns the session or run identity label.
    #[must_use]
    pub fn session_identity(&self) -> &str {
        self.session_identity.as_str()
    }

    /// Returns the required confirmation level.
    #[must_use]
    pub const fn confirmation_level(&self) -> super::ConfirmationLevel {
        self.confirmation_level
    }

    /// Returns the optional request expiry.
    #[must_use]
    pub const fn expiry(&self) -> Option<super::ConfirmationExpiry> {
        self.expiry
    }

    /// Returns the request redaction marker.
    #[must_use]
    pub const fn redaction(&self) -> super::super::RedactionMarker {
        self.redaction
    }
}

/// Borrowed fields used to build an explicit confirmation approval binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConfirmationBindingDraft<'a> {
    /// Pending request identity echoed by the broker.
    pub request_id: &'a str,
    /// Protected action class echoed by the broker.
    pub action_kind: super::ConfirmationActionKind,
    /// Resource label or identity echoed by the broker.
    pub resource: &'a str,
    /// Ability label echoed by the broker.
    pub ability: &'a str,
    /// Policy hash echoed by the broker.
    pub policy_hash: &'a super::super::SafeHash,
    /// Expiry echoed by the broker.
    pub expiry: Option<super::ConfirmationExpiry>,
}

/// Binding fields that an approval must echo to authorize one pending request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmationBinding {
    request_id: alloc::string::String,
    action_kind: super::ConfirmationActionKind,
    resource: alloc::string::String,
    ability: alloc::string::String,
    policy_hash: super::super::SafeHash,
    expiry: Option<super::ConfirmationExpiry>,
}

impl ConfirmationBinding {
    /// Builds an approval binding that exactly matches a pending request.
    #[must_use]
    pub fn from_request(request: &ConfirmationRequest) -> Self {
        Self {
            request_id: alloc::string::String::from(request.request_id()),
            action_kind: request.action_kind(),
            resource: alloc::string::String::from(request.resource()),
            ability: alloc::string::String::from(request.ability()),
            policy_hash: request.policy_hash().clone(),
            expiry: request.expiry(),
        }
    }

    /// Builds an approval binding from explicit fields.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when a required string field is
    /// empty, too long, or contains a NUL character.
    pub fn new(draft: ConfirmationBindingDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            request_id: required_text(REQUEST_ID_FIELD, draft.request_id)?,
            action_kind: draft.action_kind,
            resource: required_text(RESOURCE_FIELD, draft.resource)?,
            ability: required_text(ABILITY_FIELD, draft.ability)?,
            policy_hash: draft.policy_hash.clone(),
            expiry: draft.expiry,
        })
    }

    /// Returns bound request ID.
    #[must_use]
    pub fn request_id(&self) -> &str {
        self.request_id.as_str()
    }

    /// Returns bound action kind.
    #[must_use]
    pub const fn action_kind(&self) -> super::ConfirmationActionKind {
        self.action_kind
    }

    /// Returns bound resource label.
    #[must_use]
    pub fn resource(&self) -> &str {
        self.resource.as_str()
    }

    /// Returns bound ability label.
    #[must_use]
    pub fn ability(&self) -> &str {
        self.ability.as_str()
    }

    /// Returns bound policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &super::super::SafeHash {
        &self.policy_hash
    }

    /// Returns bound expiry.
    #[must_use]
    pub const fn expiry(&self) -> Option<super::ConfirmationExpiry> {
        self.expiry
    }
}

fn required_text(
    field_name: crate::validation::FieldName,
    text: &str,
) -> Result<alloc::string::String, crate::AdapterContractError> {
    crate::validation::normalized_adapter_field(field_name, text)
}
