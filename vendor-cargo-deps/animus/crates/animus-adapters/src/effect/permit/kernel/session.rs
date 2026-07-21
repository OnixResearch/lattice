/// Scoped session allowlist token before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAllowlistEntryDraft<'a> {
    /// Guarded action class.
    pub action: super::super::GuardedAction,
    /// Safe resource identity hash.
    pub resource_hash: &'a crate::SafeHash,
    /// Safe policy identity hash.
    pub policy_hash: &'a crate::SafeHash,
    /// Session label bound to this token.
    pub session_label: &'a str,
    /// Host-issued token label.
    pub token: &'a str,
    /// Whether host policy invalidated the token.
    pub invalidated: bool,
}

/// Validated scoped session allowlist token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionAllowlistEntry {
    action: super::super::GuardedAction,
    resource_hash: crate::SafeHash,
    policy_hash: crate::SafeHash,
    session_label: super::Text,
    token: super::Text,
    invalidated: bool,
}

impl SessionAllowlistEntry {
    /// Builds a scoped session allowlist entry from safe identities.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when the session label or token is malformed.
    // r[impl agent_harness_runtime.zerostack_permission_kernel.session_allowlist]
    pub fn new(draft: SessionAllowlistEntryDraft<'_>) -> Result<Self, super::Error> {
        Ok(Self {
            action: draft.action,
            resource_hash: draft.resource_hash.clone(),
            policy_hash: draft.policy_hash.clone(),
            session_label: super::required_text_value(super::SESSION_FIELD, draft.session_label)?,
            token: super::required_text_value(super::TOKEN_FIELD, draft.token)?,
            invalidated: draft.invalidated,
        })
    }

    /// Returns the guarded action class.
    #[must_use]
    pub const fn action(&self) -> super::super::GuardedAction {
        self.action
    }

    /// Returns the resource hash.
    #[must_use]
    pub const fn resource_hash(&self) -> &crate::SafeHash {
        &self.resource_hash
    }

    /// Returns the policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::SafeHash {
        &self.policy_hash
    }

    /// Returns the session label.
    #[must_use]
    pub fn session_label(&self) -> &str {
        self.session_label.as_str()
    }

    /// Returns token label.
    #[must_use]
    pub fn token(&self) -> &str {
        self.token.as_str()
    }

    /// Returns true when host invalidated this token.
    #[must_use]
    pub const fn invalidated(&self) -> bool {
        self.invalidated
    }
}

/// Borrowed request for validating a session allowlist token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionAllowlistValidation<'a> {
    /// Token under validation.
    pub entry: &'a SessionAllowlistEntry,
    /// Guarded action under evaluation.
    pub action: super::super::GuardedAction,
    /// Safe resource hash under evaluation.
    pub resource_hash: &'a crate::SafeHash,
    /// Safe policy hash under evaluation.
    pub policy_hash: &'a crate::SafeHash,
    /// Session label under evaluation.
    pub session_label: &'a str,
}

/// Token validation outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAllowlistStatus {
    /// Token can satisfy an ask decision.
    Accepted,
    /// Token is stale, mismatched, or invalidated.
    Rejected,
}

/// Validates that a session allowlist token is still bound to the current request.
#[must_use]
// r[impl agent_harness_runtime.zerostack_permission_kernel.session_allowlist.stale]
pub fn validate_session_allowlist(
    request: SessionAllowlistValidation<'_>,
) -> SessionAllowlistStatus {
    if request.entry.invalidated() {
        return SessionAllowlistStatus::Rejected;
    }
    if request.entry.action() != request.action {
        return SessionAllowlistStatus::Rejected;
    }
    if request.entry.resource_hash() != request.resource_hash {
        return SessionAllowlistStatus::Rejected;
    }
    if request.entry.policy_hash() != request.policy_hash {
        return SessionAllowlistStatus::Rejected;
    }
    if request.entry.session_label() != request.session_label {
        return SessionAllowlistStatus::Rejected;
    }
    SessionAllowlistStatus::Accepted
}
