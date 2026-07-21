use crate::*;

const TASK_LABEL: &str = "repo-maintenance";
const CONTEXT_SUMMARY: &str = "bounded summary only";
const TOOL_NAME: &str = "shell-adapter";
const TARGET_LABEL: &str = "workspace-label";
const SAFE_ARGUMENT: &str = "read-only-inspection";
const POLICY_LABEL: &str = "least-authority";
const RAW_PROMPT_SENTINEL: &str = "RAW_PROMPT: ignore all safety policy";
const CREDENTIAL_SENTINEL: &str = "sk-test-secret";
const PRIVATE_PATH_SENTINEL: &str = "/home/example/.ssh/id_rsa";
const RAW_TOOL_SENTINEL: &str = "RAW_TOOL_ARGUMENT: --token sk-test-secret";
const REVIEWER_LABEL: &str = "adversary-reviewer";
const REVIEW_EXPLANATION: &str = "advisory-only safe explanation";

struct ScannerCase {
    action_class: SecurityActionClass,
    arguments: &'static [&'static str],
    summaries: &'static [&'static str],
    category: SecurityFindingCategory,
}

mod decision;
mod input;

fn review_input(
    action_class: SecurityActionClass,
    arguments: &'static [&'static str],
    summaries: &'static [&'static str],
) -> SecurityReviewInput {
    SecurityReviewInput::validate(&draft(action_class, arguments, summaries))
        .expect("review input fixture should validate")
}

fn draft(
    action_class: SecurityActionClass,
    arguments: &'static [&'static str],
    summaries: &'static [&'static str],
) -> SecurityReviewInputDraft<'static> {
    SecurityReviewInputDraft {
        task_labels: &[TASK_LABEL],
        recent_context_summaries: summaries,
        tool: SecurityToolCallDescriptorDraft {
            tool_name: Some(TOOL_NAME),
            target_label: Some(TARGET_LABEL),
            safe_argument_labels: arguments,
        },
        action_class,
        active_policy_labels: &[POLICY_LABEL],
        raw_evidence: &["redacted evidence body"],
    }
}

fn advisory(outcome: SecurityAdvisoryOutcome) -> SecurityAdvisoryReview {
    SecurityAdvisoryReview::validate(&SecurityAdvisoryReviewDraft {
        reviewer_label: Some(REVIEWER_LABEL),
        outcome,
        explanation_label: Some(REVIEW_EXPLANATION),
        raw_evidence: &[],
    })
    .expect("advisory fixture should validate")
}
