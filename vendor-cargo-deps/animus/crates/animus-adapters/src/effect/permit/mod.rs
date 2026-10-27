//! Pure permission-mode descriptors and resolution for guarded actions.

mod dialog;
mod evidence;
mod fault;
mod flow;
mod kernel;
mod rule;

#[cfg(test)]
mod tests;

pub use dialog::{Decision, DecisionDraft, DecisionOutcome, PromptRequest, PromptRequestDraft};
pub use evidence::{Receipt, ReceiptDraft, ReceiptLeak, validate_text};
pub use fault::Error;
pub use flow::{
    DecisionReason, DecisionStatus, PromptStatus, Resolution, ResolutionRequest, resolve,
};
pub use kernel::{
    KernelReceipt, KernelReceiptDraft, ModeDefaultReason, ModeDefaultRequest,
    ModeDefaultResolution, PatternMatchKind, PatternRule, PatternRuleDraft, PermissionMode,
    RepeatedActionInput, RepeatedActionPolicy, RepeatedActionResolution, RepeatedActionStatus,
    SessionAllowlistEntry, SessionAllowlistEntryDraft, SessionAllowlistStatus,
    SessionAllowlistValidation, detect_repeated_action, resolve_mode_default,
    validate_session_allowlist,
};
pub use rule::{Outcome, Policy, PolicyDraft, PromptSummaryPolicy, TrustScope, TrustStatus};

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

pub(super) const ACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_action");
pub(super) const RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_resource_label");
pub(super) const OUTCOME_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_outcome");
pub(super) const SOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_source_identity");
pub(super) const TRUST_SCOPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_trust_scope");
pub(super) const TRUST_STATUS_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_trust_status");
pub(super) const PROMPT_POLICY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_prompt_summary_policy");
pub(super) const PATTERN_MATCH_KIND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_pattern_match_kind");
pub(super) const PATTERN_RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_pattern_resource");
pub(super) const PROMPT_SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_prompt_summary");
pub(super) const SESSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_session_label");
pub(super) const TOKEN_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_prompt_token");
pub(super) const REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_reason");
pub(super) const REDACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_redaction_marker");

pub(super) const ALLOW_OUTCOME_LABEL: &str = "allow";
pub(super) const ASK_OUTCOME_LABEL: &str = "ask";
pub(super) const DENY_OUTCOME_LABEL: &str = "deny";
pub(super) const USER_SCOPE_LABEL: &str = "user";
pub(super) const PROJECT_SCOPE_LABEL: &str = "project";
pub(super) const SESSION_SCOPE_LABEL: &str = "session";
pub(super) const HARNESS_SCOPE_LABEL: &str = "harness";
pub(super) const TRUSTED_LABEL: &str = "trusted";
pub(super) const UNTRUSTED_LABEL: &str = "untrusted";
pub(super) const REDACTED_SUMMARY_POLICY_LABEL: &str = "redacted";
pub(super) const HASH_SCHEMA: &str = "animus.claude-code-permission.v1";
pub(super) const HASH_SEPARATOR: &str = "|";
pub(super) const MIN_PRECEDENCE: u16 = 1;
pub(super) const MAX_PRECEDENCE: u16 = 1_000;
pub(super) const MAX_REDACTIONS: usize = 16;

const _: () = assert!(MIN_PRECEDENCE > 0);
const _: () = assert!(MAX_PRECEDENCE >= MIN_PRECEDENCE);
const _: () = assert!(MAX_REDACTIONS > 0);

pub(super) fn required_text(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Text, Error> {
    match value {
        Some(raw) => required_text_value(field, raw),
        None => Err(Error::MissingField {
            field: field.as_str(),
        }),
    }
}

pub(super) fn required_text_value(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<Text, Error> {
    crate::validation::normalized_adapter_field(field, value).map_err(Error::from)
}

pub(super) fn hash_parts(parts: &[&str]) -> Text {
    let mut text = Text::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}
