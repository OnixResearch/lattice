//! Pure exec-policy contracts, validation, and evaluation.

mod eval;
mod fault;
mod shell;
mod types;

#[cfg(test)]
mod tests;

pub use eval::Evaluation;
pub use eval::Request;
pub use eval::RuleEvidence;
pub use eval::evaluate;
pub use fault::Error;
pub use shell::Analysis;
pub use shell::AnalysisKind;
pub use shell::analyze_wrapper;
pub use types::CommandIntent;
pub use types::CommandIntentDraft;
pub use types::Decision;
pub use types::Rule;
pub use types::RuleDraft;
pub use types::SandboxContext;
pub use types::SandboxContextDraft;

pub(super) type Collection<T> = alloc::vec::Vec<T>;
pub(super) type Text = alloc::string::String;

pub(super) const COMMAND_ARG_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_command_arg");
pub(super) const RULE_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("exec_policy_rule_id");
pub(super) const DECISION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_decision");
pub(super) const RATIONALE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_rationale");
pub(super) const PREFIX_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_command_prefix");
pub(super) const POSITIVE_EXAMPLES_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_positive_examples");
pub(super) const NEGATIVE_EXAMPLES_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_negative_examples");
pub(super) const ACTIVE_SANDBOX_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_active_sandbox");
pub(super) const REQUESTED_CAPABILITY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_requested_capability");
pub(super) const ESCALATION_REASON_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("exec_policy_escalation_reason");

pub(super) const ALLOW_LABEL: &str = "allow";
pub(super) const PROMPT_LABEL: &str = "prompt";
pub(super) const FORBID_LABEL: &str = "forbid";
pub(super) const HASH_SCHEMA: &str = "animus.exec-policy.v1";
pub(super) const HASH_SEPARATOR: &str = "|";
pub(super) const HASH_LENGTH_SEPARATOR: &str = ":";
pub(super) const COMMAND_HASH_KIND: &str = "command";
pub(super) const RULE_HASH_KIND: &str = "rule";
pub(super) const REDACTION_COMMAND_HASH_ONLY: &str = "command_hash_only";
pub(super) const NO_MATCHING_RULE_ID: &str = "no_matching_rule";
pub(super) const MAX_COMMAND_WORDS: usize = 64;
pub(super) const MAX_RULE_EXAMPLES: usize = 16;
pub(super) const DECISION_RANK_ALLOW: u8 = 1;
pub(super) const DECISION_RANK_PROMPT: u8 = 2;
pub(super) const DECISION_RANK_FORBID: u8 = 3;

const _: () = assert!(MAX_COMMAND_WORDS > 0);
const _: () = assert!(MAX_RULE_EXAMPLES > 0);
const _: () = assert!(DECISION_RANK_ALLOW < DECISION_RANK_PROMPT);
const _: () = assert!(DECISION_RANK_PROMPT < DECISION_RANK_FORBID);

pub(super) fn required_text(field: crate::validation::FieldName, value: Option<&str>) -> Result<Text, Error> {
    match value {
        Some(raw) => normalized_text(field, raw),
        None => Err(Error::MissingField { field: field.as_str() }),
    }
}

pub(super) fn normalized_text(field: crate::validation::FieldName, value: &str) -> Result<Text, Error> {
    crate::validation::normalized_adapter_field(field, value).map_err(Error::from)
}

pub(super) fn hash_parts(kind: &str, parts: &[&str]) -> Text {
    use alloc::string::ToString;

    let mut canonical = Text::new();
    canonical.push_str(HASH_SCHEMA);
    canonical.push_str(HASH_SEPARATOR);
    canonical.push_str(kind);
    canonical.push_str(HASH_SEPARATOR);
    canonical.push_str(&parts.len().to_string());
    for part in parts {
        canonical.push_str(HASH_SEPARATOR);
        canonical.push_str(&part.len().to_string());
        canonical.push_str(HASH_LENGTH_SEPARATOR);
        canonical.push_str(part);
    }
    alloc::format!("{}", blake3::hash(canonical.as_bytes()).to_hex())
}
