pub(crate) const MIN_RUNTIME_TURNS: u32 = 1;

/// Borrowed profile data exported from Nickel or built by typed test fixtures.
#[derive(Clone, Copy)]
pub struct ProfileDraft<'a> {
    /// Stable profile name selected by callers.
    pub name: &'a str,
    /// Whether the profile is enabled for activation.
    pub enabled: bool,
    /// Whether the profile is the default profile in exported data.
    pub is_default: bool,
    /// Safe relative script binding label.
    pub script_label: &'a str,
    /// Expected lowercase BLAKE3 hash for the script source.
    pub script_hash: &'a str,
    /// Maximum runtime turn budget for this profile.
    pub max_turns: u32,
    /// Declared host actions that may be exposed to Steel.
    pub allowed_host_actions: &'a [&'a str],
    /// Explicit fallback mode label.
    pub fallback_mode: &'a str,
    /// Receipt redaction mode label.
    pub receipt_redaction: &'a str,
}

/// Checked Steel runtime profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    name: String,
    enabled: bool,
    is_default: bool,
    script_label: String,
    script_hash: String,
    max_turns: u32,
    allowed_host_actions: std::collections::BTreeSet<super::HostAction>,
    fallback_mode: super::FallbackMode,
    receipt_redaction: super::ReceiptRedaction,
    hash: String,
}

impl Profile {
    /// Builds a checked profile from exported data or typed fixtures.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when a profile field is unsafe or unsupported.
    pub fn from_draft(draft: &ProfileDraft<'_>) -> Result<Self, super::Error> {
        let name = super::text::validate_profile_name(draft.name)?;
        let script_label = super::text::validate_script_label(draft.script_label)?;
        let script_hash =
            super::text::validate_hash(draft.script_hash, super::text::FieldName::SCRIPT_HASH)?;
        let max_turns = super::text::validate_turn_budget(draft.max_turns)?;
        let allowed_host_actions = super::text::parse_allowed_actions(draft.allowed_host_actions)?;
        let fallback_mode = super::FallbackMode::parse(draft.fallback_mode)?;
        let receipt_redaction = super::ReceiptRedaction::parse(draft.receipt_redaction)?;
        let profile_hash = hash_draft(&CheckedFields {
            draft,
            name: name.as_str(),
            script_label: script_label.as_str(),
            script_hash: script_hash.as_str(),
            max_turns,
            allowed_host_actions: &allowed_host_actions,
            fallback_mode,
            receipt_redaction,
        });

        Ok(Self {
            name,
            enabled: draft.enabled,
            is_default: draft.is_default,
            script_label,
            script_hash,
            max_turns,
            allowed_host_actions,
            fallback_mode,
            receipt_redaction,
            hash: profile_hash,
        })
    }

    /// Returns this profile's stable name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns whether this profile is enabled for activation.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns whether this profile is the default exported profile.
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.is_default
    }

    /// Returns the safe relative script binding label.
    #[must_use]
    pub fn script_label(&self) -> &str {
        self.script_label.as_str()
    }

    /// Returns the expected BLAKE3 script hash.
    #[must_use]
    pub fn script_hash(&self) -> &str {
        self.script_hash.as_str()
    }

    /// Returns the maximum runtime turn budget.
    #[must_use]
    pub const fn max_turns(&self) -> u32 {
        self.max_turns
    }

    /// Returns true when the profile allowlist includes `action`.
    #[must_use]
    pub fn allows_action(&self, action: super::HostAction) -> bool {
        self.allowed_host_actions.contains(&action)
    }

    /// Returns the explicit fallback mode.
    #[must_use]
    pub const fn fallback_mode(&self) -> super::FallbackMode {
        self.fallback_mode
    }

    /// Returns the receipt redaction mode.
    #[must_use]
    pub const fn receipt_redaction(&self) -> super::ReceiptRedaction {
        self.receipt_redaction
    }

    /// Returns the BLAKE3 hash of this profile's checked fields.
    #[must_use]
    pub fn profile_hash(&self) -> &str {
        self.hash.as_str()
    }

    /// Evaluates an activation request and returns a redacted receipt.
    #[must_use]
    pub fn activate(&self, activation: &super::Activation<'_>) -> super::ActivationReceipt {
        super::ActivationReceipt::new(self, activation, self.activation_denial(activation))
    }

    pub(crate) fn activation_denial(
        &self,
        activation: &super::Activation<'_>,
    ) -> Option<super::DenyReason> {
        let script = activation.script();
        if !self.enabled {
            return Some(super::DenyReason::DisabledProfile);
        }
        if !super::text::is_safe_profile_name(script.selected_profile()) {
            return Some(super::DenyReason::BadProfileName);
        }
        self.script_denial(script)
            .or_else(|| self.runtime_denial(activation))
    }

    fn script_denial(&self, script: super::Script<'_>) -> Option<super::DenyReason> {
        if script.selected_profile() != self.name {
            return Some(super::DenyReason::ProfileMismatch);
        }
        if script.source().trim().is_empty() {
            return Some(super::DenyReason::MissingScript);
        }
        if !super::text::is_safe_script_label(script.label()) {
            return Some(super::DenyReason::PathEscape);
        }
        self.script_identity_denial(script)
    }

    fn script_identity_denial(&self, script: super::Script<'_>) -> Option<super::DenyReason> {
        if script.label() != self.script_label {
            return Some(super::DenyReason::MissingScript);
        }
        if super::script_hash(script.source()) != self.script_hash {
            return Some(super::DenyReason::HashMismatch);
        }
        None
    }

    fn runtime_denial(&self, activation: &super::Activation<'_>) -> Option<super::DenyReason> {
        if activation.turn_budget() < MIN_RUNTIME_TURNS || activation.turn_budget() > self.max_turns
        {
            return Some(super::DenyReason::InvalidBudget);
        }
        activation
            .requested_host_actions()
            .iter()
            .copied()
            .find(|action| !self.allows_action(*action))
            .map(|_action| super::DenyReason::HostActionDenied)
    }
}

struct CheckedFields<'a> {
    draft: &'a ProfileDraft<'a>,
    name: &'a str,
    script_label: &'a str,
    script_hash: &'a str,
    max_turns: u32,
    allowed_host_actions: &'a std::collections::BTreeSet<super::HostAction>,
    fallback_mode: super::FallbackMode,
    receipt_redaction: super::ReceiptRedaction,
}

fn hash_draft(fields: &CheckedFields<'_>) -> String {
    super::digest::profile_hash(&super::digest::ProfileFields {
        name: fields.name,
        enabled: fields.draft.enabled,
        is_default: fields.draft.is_default,
        script_label: fields.script_label,
        script_hash: fields.script_hash,
        max_turns: fields.max_turns,
        allowed_host_actions: fields.allowed_host_actions,
        fallback_mode: fields.fallback_mode,
        receipt_redaction: fields.receipt_redaction,
    })
}
