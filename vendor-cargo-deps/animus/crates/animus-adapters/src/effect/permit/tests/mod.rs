use super::*;

mod negative;
mod negativekernel;
mod positive;
mod positivekernel;

const ACTION: super::super::GuardedAction = super::super::GuardedAction::Tool;
const RESOURCE: &str = "tool:read-only";
const SESSION: &str = "session:alpha";
const SOURCE: &str = "user-policy";
const SUMMARY: &str = "redacted tool request";
const HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SECOND_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const REDACTION: &str = "raw-prompt-redacted";
const SECOND_REDACTION: &str = "tool-body-redacted";
const BAD_ACTION: &str = "database";
const STALE_TOKEN: &str = "stale-token";
const SESSION_TOKEN: &str = "session-token";
const OTHER_SESSION: &str = "session:beta";
const GLOB_MATCH_KIND: &str = "glob";
const REGEX_MATCH_KIND: &str = "regex";
const UNKNOWN_MATCH_KIND: &str = "prefix";
const GLOB_RESOURCE_PATTERN: &str = "tool:*";
const INVALID_REGEX_RESOURCE_PATTERN: &str = "tool:[";
const UNSUPPORTED_OUTCOME_LABEL: &str = "maybe";
const REPEAT_PRIOR_OBSERVATIONS: u32 = 2;
const REPEAT_THRESHOLD: u32 = 3;
const RAW_PROMPT_RECEIPT: &str = "action=tool raw_prompt=secret";
const CREDENTIAL_RECEIPT: &str = "action=tool credential=token";
const PROVIDER_RECEIPT: &str = "action=tool provider_json={}";
const TOOL_BODY_RECEIPT: &str = "action=tool raw_tool_body=data";
const PATH_RECEIPT: &str = "action=tool path=/home/user/repo";
const OUTPUT_RECEIPT: &str = "action=tool unbounded_output=data";
const SAFE_RECEIPT_TEXT: &str = "action=tool resource_hash=abc redaction=raw-prompt-redacted";

fn policy_draft(outcome: &'static str) -> PolicyDraft<'static> {
    PolicyDraft {
        action: Some(ACTION.as_str()),
        resource_label: Some(RESOURCE),
        outcome: Some(outcome),
        source_identity: Some(SOURCE),
        trust_scope: Some(USER_SCOPE_LABEL),
        trust_status: Some(TRUSTED_LABEL),
        precedence: MIN_PRECEDENCE,
        prompt_summary_policy: Some(REDACTED_SUMMARY_POLICY_LABEL),
    }
}

fn policy(outcome: &'static str) -> Policy {
    Policy::validate(&policy_draft(outcome)).expect("permission policy should validate")
}

fn allow_policy() -> Policy {
    policy(ALLOW_OUTCOME_LABEL)
}

fn ask_policy() -> Policy {
    policy(ASK_OUTCOME_LABEL)
}

fn deny_policy() -> Policy {
    policy(DENY_OUTCOME_LABEL)
}

fn allowed_request(policy: &Policy) -> ResolutionRequest<'_> {
    ResolutionRequest {
        action: ACTION,
        resource_label: RESOURCE,
        authority_status: super::super::AuthorizationStatus::Allowed,
        authority_issue: super::super::AuthorizationIssue::None,
        policy: Some(policy),
        prompt_decision: None,
        session_label: SESSION,
        redacted_summary: SUMMARY,
    }
}

fn denied_authority_request(policy: &Policy) -> ResolutionRequest<'_> {
    ResolutionRequest {
        authority_status: super::super::AuthorizationStatus::Denied,
        authority_issue: super::super::AuthorizationIssue::MissingUcanGrant,
        ..allowed_request(policy)
    }
}

fn safe_hash() -> crate::SafeHash {
    crate::SafeHash::new(HASH).expect("test hash should be valid")
}

fn second_safe_hash() -> crate::SafeHash {
    crate::SafeHash::new(SECOND_HASH).expect("test hash should be valid")
}
