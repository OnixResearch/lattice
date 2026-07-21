const RECEIPT_STATUS_ALLOWED: &str = "allowed";
const RECEIPT_STATUS_DENIED: &str = "denied";
const INVALID_PROFILE_LABEL: &str = "invalid-profile";

/// Host actions exposed by the default runtime shell.
pub const DEFAULT_RUNTIME_HOST_ACTIONS: &[super::HostAction] = &[
    super::HostAction::Model,
    super::HostAction::Tool,
    super::HostAction::Respond,
    super::HostAction::Stop,
];

/// Borrowed Steel script selection to activate under policy.
#[derive(Clone, Copy)]
pub struct Script<'a> {
    selected_profile: &'a str,
    label: &'a str,
    source: &'a str,
}

impl<'a> Script<'a> {
    /// Builds a borrowed script selection.
    #[must_use]
    pub const fn new(selected_profile: &'a str, label: &'a str, source: &'a str) -> Self {
        Self {
            selected_profile,
            label,
            source,
        }
    }

    /// Returns the selected profile name.
    #[must_use]
    pub const fn selected_profile(self) -> &'a str {
        self.selected_profile
    }

    /// Returns the safe relative script binding label.
    #[must_use]
    pub const fn label(self) -> &'a str {
        self.label
    }

    /// Returns the script source to load only after activation allows it.
    #[must_use]
    pub const fn source(self) -> &'a str {
        self.source
    }
}

/// Runtime activation inputs checked before Steel evaluation.
pub struct Activation<'a> {
    script: Script<'a>,
    turn_budget: u32,
    requested_host_actions: &'a [super::HostAction],
}

impl<'a> Activation<'a> {
    /// Builds activation inputs for one runtime execution.
    #[must_use]
    pub const fn new(
        script: Script<'a>,
        turn_budget: u32,
        requested_host_actions: &'a [super::HostAction],
    ) -> Self {
        Self {
            script,
            turn_budget,
            requested_host_actions,
        }
    }

    /// Returns the selected script.
    #[must_use]
    pub const fn script(&self) -> Script<'a> {
        self.script
    }

    /// Returns the requested runtime turn budget.
    #[must_use]
    pub const fn turn_budget(&self) -> u32 {
        self.turn_budget
    }

    /// Returns the requested host actions to expose.
    #[must_use]
    pub const fn requested_host_actions(&self) -> &'a [super::HostAction] {
        self.requested_host_actions
    }
}

/// Safe activation status recorded in receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivationStatus {
    /// Policy activation allowed Steel evaluation.
    Allowed,
    /// Policy activation denied Steel evaluation.
    Denied,
}

impl ActivationStatus {
    /// Returns the canonical receipt label for this status.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => RECEIPT_STATUS_ALLOWED,
            Self::Denied => RECEIPT_STATUS_DENIED,
        }
    }
}

/// Redacted receipt emitted for Steel policy activation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationReceipt {
    schema: &'static str,
    status: ActivationStatus,
    profile: String,
    script_hash: String,
    profile_hash: String,
    denied_reason: Option<super::DenyReason>,
    fallback_mode: super::FallbackMode,
    redaction: super::ReceiptRedaction,
    receipt_hash: String,
}

impl ActivationReceipt {
    pub(crate) fn new(
        profile: &super::Profile,
        activation: &Activation<'_>,
        denied_reason: Option<super::DenyReason>,
    ) -> Self {
        let status = activation_status(denied_reason);
        let profile_label = receipt_profile_label(activation.script().selected_profile());
        let script_hash = super::script_hash(activation.script().source());
        let receipt_hash = super::digest::receipt_hash(&super::digest::ReceiptFields {
            status,
            profile: profile_label.as_str(),
            script_hash: script_hash.as_str(),
            profile_hash: profile.profile_hash(),
            denied_reason,
            fallback_mode: profile.fallback_mode(),
            receipt_redaction: profile.receipt_redaction(),
        });
        Self::from_fields(ReceiptParts {
            status,
            profile: profile_label,
            script_hash,
            profile_hash: String::from(profile.profile_hash()),
            denied_reason,
            fallback_mode: profile.fallback_mode(),
            redaction: profile.receipt_redaction(),
            receipt_hash,
        })
    }

    /// Returns the receipt schema label.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns the activation status.
    #[must_use]
    pub const fn status(&self) -> ActivationStatus {
        self.status
    }

    /// Returns true when policy allowed Steel evaluation.
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self.status, ActivationStatus::Allowed)
    }

    /// Returns the safe profile label recorded in the receipt.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.profile.as_str()
    }

    /// Returns the BLAKE3 hash of the observed script source.
    #[must_use]
    pub fn script_hash(&self) -> &str {
        self.script_hash.as_str()
    }

    /// Returns the BLAKE3 hash of checked profile fields.
    #[must_use]
    pub fn profile_hash(&self) -> &str {
        self.profile_hash.as_str()
    }

    /// Returns the denial reason class when activation failed.
    #[must_use]
    pub const fn denied_reason(&self) -> Option<super::DenyReason> {
        self.denied_reason
    }

    /// Returns the declared fallback mode.
    #[must_use]
    pub const fn fallback_mode(&self) -> super::FallbackMode {
        self.fallback_mode
    }

    /// Returns the receipt redaction mode.
    #[must_use]
    pub const fn redaction(&self) -> super::ReceiptRedaction {
        self.redaction
    }

    /// Returns the BLAKE3 hash of safe receipt fields.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }

    /// Returns a bounded safe message for runtime failures.
    #[must_use]
    pub fn denial_message(&self) -> String {
        let reason = self
            .denied_reason
            .map_or("unknown", super::DenyReason::as_str);
        format!("steel policy denied: {reason}")
    }

    fn from_fields(parts: ReceiptParts) -> Self {
        Self {
            schema: super::digest::receipt_schema(),
            status: parts.status,
            profile: parts.profile,
            script_hash: parts.script_hash,
            profile_hash: parts.profile_hash,
            denied_reason: parts.denied_reason,
            fallback_mode: parts.fallback_mode,
            redaction: parts.redaction,
            receipt_hash: parts.receipt_hash,
        }
    }
}

struct ReceiptParts {
    status: ActivationStatus,
    profile: String,
    script_hash: String,
    profile_hash: String,
    denied_reason: Option<super::DenyReason>,
    fallback_mode: super::FallbackMode,
    redaction: super::ReceiptRedaction,
    receipt_hash: String,
}

fn activation_status(denied_reason: Option<super::DenyReason>) -> ActivationStatus {
    if denied_reason.is_some() {
        ActivationStatus::Denied
    } else {
        ActivationStatus::Allowed
    }
}

fn receipt_profile_label(selected_profile: &str) -> String {
    if super::text::is_safe_profile_name(selected_profile) {
        String::from(selected_profile)
    } else {
        String::from(INVALID_PROFILE_LABEL)
    }
}
