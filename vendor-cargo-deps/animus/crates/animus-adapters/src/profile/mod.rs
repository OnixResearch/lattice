//! Data-only policy/profile descriptors for prompt guidance composition.

mod compose;
mod data;
mod types;
mod validate;

pub use compose::mutation_requires_runtime_guard;
pub use data::Descriptor;
pub use data::Draft;
pub use data::Kind;
pub use data::SourceScope;
pub use data::TrustStatus;
pub use types::CompositionDiagnostic;
pub use types::CompositionReport;
pub use types::DiagnosticKind;
pub use types::EnforcementBoundary;
pub use types::Error;
pub use types::ProjectTrustMode;
pub use types::PromptHygieneIssue;
pub use types::PromptSegment;
pub use types::RuntimeGuardRequirement;

type Text = alloc::string::String;
type Collection<T> = alloc::vec::Vec<T>;

const KIND_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_kind");
const SCOPE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_scope");
const TRUST_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_trust");
const GOAL_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_goal");
const SUCCESS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_success_criteria");
const CONSTRAINT_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_constraint");
const VALIDATION_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_validation_rule");
const STOP_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_stop_rule");
const PROVENANCE_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_provenance");
const SNIPPET_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("policy_prompt_snippet");
const MIN_POLICY_PRIORITY: u16 = 1;
const MAX_POLICY_PRIORITY: u16 = 1_000;
const MAX_POLICY_ITEMS: usize = 12;
const MAX_POLICY_PROMPT_CHARS: usize = 2_048;
const GLOBAL_SCOPE_RANK: u8 = 0;
const SESSION_SCOPE_RANK: u8 = 1;
const PROJECT_SCOPE_RANK: u8 = 2;
const TOOL_SCOPE_RANK: u8 = 3;
const PROCESS_HEAVY_MARKER: &str = "step-by-step script";
const UNBOUNDED_MARKER: &str = "forever";
const STALE_MODEL_MARKER: &str = "gpt-4 only";
const REQUIRE_APPROVAL_CONSTRAINT: &str = "require-approval-for-destructive-actions";
const ALLOW_DESTRUCTIVE_CONSTRAINT: &str = "allow-destructive-without-approval";
const READ_ONLY_CONSTRAINT: &str = "read-only-session";
const PROMPT_SECTION_SEPARATOR: &str = "\n";
const PROMPT_ITEM_SEPARATOR: &str = "; ";
const FIRST_PROMPT_ITEM_INDEX: usize = 0;
const REMAINING_PROMPT_ITEMS_START: usize = 1;

const _: () = assert!(MIN_POLICY_PRIORITY > 0);
const _: () = assert!(MAX_POLICY_PRIORITY >= MIN_POLICY_PRIORITY);
const _: () = assert!(MAX_POLICY_ITEMS > 0);
const _: () = assert!(MAX_POLICY_PROMPT_CHARS > 0);

/// Composes trusted policy profiles into attributed prompt segments.
#[must_use]
pub fn compose_context(profiles: &[Descriptor], project_trust_mode: ProjectTrustMode) -> CompositionReport {
    compose::context(profiles, project_trust_mode)
}

#[cfg(test)]
mod tests;
