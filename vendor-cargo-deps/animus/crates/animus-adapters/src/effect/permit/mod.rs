//! Pure permission-mode descriptors and resolution for guarded actions.

mod dialog;
mod evidence;
mod fault;
mod flow;
mod kernel;
mod rule;

#[cfg(test)]
mod tests;

pub use dialog::Decision;
pub use dialog::DecisionDraft;
pub use dialog::DecisionOutcome;
pub use dialog::PromptRequest;
pub use dialog::PromptRequestDraft;
pub use evidence::Receipt;
pub use evidence::ReceiptDraft;
pub use evidence::ReceiptLeak;
pub use evidence::validate_text;
pub use fault::Error;
pub use flow::DecisionReason;
pub use flow::DecisionStatus;
pub use flow::PromptStatus;
pub use flow::Resolution;
pub use flow::ResolutionRequest;
pub use flow::resolve;
pub use kernel::KernelReceipt;
pub use kernel::KernelReceiptDraft;
pub use kernel::ModeDefaultReason;
pub use kernel::ModeDefaultRequest;
pub use kernel::ModeDefaultResolution;
pub use kernel::PatternMatchKind;
pub use kernel::PatternRule;
pub use kernel::PatternRuleDraft;
pub use kernel::PermissionMode;
pub use kernel::RepeatedActionInput;
pub use kernel::RepeatedActionPolicy;
pub use kernel::RepeatedActionResolution;
pub use kernel::RepeatedActionStatus;
pub use kernel::SessionAllowlistEntry;
pub use kernel::SessionAllowlistEntryDraft;
pub use kernel::SessionAllowlistStatus;
pub use kernel::SessionAllowlistValidation;
pub use kernel::detect_repeated_action;
pub use kernel::resolve_mode_default;
pub use kernel::validate_session_allowlist;
pub use rule::Outcome;
pub use rule::Policy;
pub use rule::PolicyDraft;
pub use rule::PromptSummaryPolicy;
pub use rule::TrustScope;
pub use rule::TrustStatus;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

pub(super) const ACTION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("permission_action");
pub(super) const RESOURCE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("permission_resource_label");
pub(super) const OUTCOME_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("permission_outcome");
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
pub(super) const REASON_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("permission_reason");
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

pub(super) fn required_text(field: crate::validation::FieldName, value: Option<&str>) -> Result<Text, Error> {
    match value {
        Some(raw) => required_text_value(field, raw),
        None => Err(Error::MissingField { field: field.as_str() }),
    }
}

pub(super) fn required_text_value(field: crate::validation::FieldName, value: &str) -> Result<Text, Error> {
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
