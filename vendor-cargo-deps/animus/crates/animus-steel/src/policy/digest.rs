const PROFILE_SCHEMA: &str = "animus.steel-runtime-policy.v1";
const RECEIPT_SCHEMA: &str = "animus.steel-policy-activation.v1";
const PROFILE_HASH_SEPARATOR: &str = "|";
const RECEIPT_HASH_SEPARATOR: &str = "|";

pub(crate) struct ProfileFields<'a> {
    pub(crate) name: &'a str,
    pub(crate) enabled: bool,
    pub(crate) is_default: bool,
    pub(crate) script_label: &'a str,
    pub(crate) script_hash: &'a str,
    pub(crate) max_turns: u32,
    pub(crate) allowed_host_actions: &'a std::collections::BTreeSet<super::HostAction>,
    pub(crate) fallback_mode: super::FallbackMode,
    pub(crate) receipt_redaction: super::ReceiptRedaction,
}

pub(crate) struct ReceiptFields<'a> {
    pub(crate) status: super::ActivationStatus,
    pub(crate) profile: &'a str,
    pub(crate) script_hash: &'a str,
    pub(crate) profile_hash: &'a str,
    pub(crate) denied_reason: Option<super::DenyReason>,
    pub(crate) fallback_mode: super::FallbackMode,
    pub(crate) receipt_redaction: super::ReceiptRedaction,
}

pub(crate) const fn receipt_schema() -> &'static str {
    RECEIPT_SCHEMA
}

/// Returns the canonical BLAKE3 hash for a Steel script body.
#[must_use]
pub fn script_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

pub(crate) fn profile_hash(fields: &ProfileFields<'_>) -> String {
    let actions = action_labels(fields.allowed_host_actions);
    let max_turns = fields.max_turns.to_string();
    let parts = [
        PROFILE_SCHEMA,
        fields.name,
        bool_label(fields.enabled),
        bool_label(fields.is_default),
        fields.script_label,
        fields.script_hash,
        max_turns.as_str(),
        actions.as_str(),
        fields.fallback_mode.as_str(),
        fields.receipt_redaction.as_str(),
    ];
    text_hash(parts.join(PROFILE_HASH_SEPARATOR).as_str())
}

pub(crate) fn receipt_hash(fields: &ReceiptFields<'_>) -> String {
    let reason = fields
        .denied_reason
        .map_or("none", super::DenyReason::as_str);
    let parts = [
        RECEIPT_SCHEMA,
        fields.status.as_str(),
        fields.profile,
        fields.script_hash,
        fields.profile_hash,
        reason,
        fields.fallback_mode.as_str(),
        fields.receipt_redaction.as_str(),
    ];
    text_hash(parts.join(RECEIPT_HASH_SEPARATOR).as_str())
}

fn action_labels(actions: &std::collections::BTreeSet<super::HostAction>) -> String {
    actions
        .iter()
        .map(|action| action.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn text_hash(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

const fn bool_label(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
