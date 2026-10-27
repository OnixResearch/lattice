#![allow(unknown_lints)]
#![allow(numeric_units, path_segment_repetition, underscore_in_module_filename)]

use super::*;

#[test]
fn yolo_mode_cannot_bypass_missing_authority() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.modes.authority]
    let missing_authority = resolve_mode_default(ModeDefaultRequest {
        mode: PermissionMode::Yolo,
        action: ACTION,
        authority_status: crate::AuthorizationStatus::Denied,
        sandbox_allows: true,
        redaction_allows: true,
    });

    assert_eq!(missing_authority.status(), DecisionStatus::Denied);
    assert_eq!(
        missing_authority.reason(),
        ModeDefaultReason::MissingAuthority
    );
    assert_eq!(missing_authority.local_outcome(), Outcome::Allow);
    assert!(!missing_authority.dispatch_allowed());
}

#[test]
fn pattern_rules_reject_bad_match_kind_and_regex() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.pattern_rules.invalid]
    let unsupported_match_kind =
        pattern_error(Some(UNKNOWN_MATCH_KIND), Some(GLOB_RESOURCE_PATTERN));
    let invalid_regex = pattern_error(Some(REGEX_MATCH_KIND), Some(INVALID_REGEX_RESOURCE_PATTERN));

    assert_eq!(
        unsupported_match_kind,
        Error::UnsupportedValue {
            field: PATTERN_MATCH_KIND_FIELD.as_str(),
            value: Text::from(UNKNOWN_MATCH_KIND),
        }
    );
    assert_eq!(
        invalid_regex,
        Error::InvalidPattern {
            field: PATTERN_RESOURCE_FIELD.as_str(),
        }
    );
}

#[test]
fn pattern_rules_reject_missing_source_outcome_and_action() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.pattern_rules.invalid]
    let missing_source = PatternRule::validate(&PatternRuleDraft {
        source_identity: None,
        ..valid_pattern_draft()
    })
    .expect_err("missing source should fail");
    let unsupported_outcome = PatternRule::validate(&PatternRuleDraft {
        outcome: Some(UNSUPPORTED_OUTCOME_LABEL),
        ..valid_pattern_draft()
    })
    .expect_err("unsupported outcome should fail");
    let malformed_action = PatternRule::validate(&PatternRuleDraft {
        action: Some(BAD_ACTION),
        ..valid_pattern_draft()
    })
    .expect_err("bad action should fail");

    assert_eq!(
        missing_source,
        Error::MissingField {
            field: SOURCE_FIELD.as_str(),
        }
    );
    assert_eq!(
        unsupported_outcome,
        Error::UnsupportedValue {
            field: OUTCOME_FIELD.as_str(),
            value: Text::from(UNSUPPORTED_OUTCOME_LABEL),
        }
    );
    assert_eq!(
        malformed_action,
        Error::UnsupportedValue {
            field: ACTION_FIELD.as_str(),
            value: Text::from(BAD_ACTION),
        }
    );
}

#[test]
fn allowlist_rejects_stale_resource_and_session() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.session_allowlist.stale]
    let resource_hash = safe_hash();
    let other_hash = second_safe_hash();
    let allowlist = allowlist_for(&resource_hash);
    let stale_resource = validate_session_allowlist(SessionAllowlistValidation {
        entry: &allowlist,
        action: ACTION,
        resource_hash: &other_hash,
        policy_hash: &resource_hash,
        session_label: SESSION,
    });
    let stale_session = validate_session_allowlist(SessionAllowlistValidation {
        entry: &allowlist,
        action: ACTION,
        resource_hash: &resource_hash,
        policy_hash: &resource_hash,
        session_label: OTHER_SESSION,
    });

    assert_eq!(stale_resource, SessionAllowlistStatus::Rejected);
    assert_eq!(stale_session, SessionAllowlistStatus::Rejected);
}

#[test]
fn repeated_destructive_action_denies_without_dispatch() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.repeated_action.deny]
    let resource_hash = safe_hash();
    let repeated_denial = detect_repeated_action(RepeatedActionInput {
        action: ACTION,
        resource_hash: &resource_hash,
        prior_observations: REPEAT_PRIOR_OBSERVATIONS,
        threshold: REPEAT_THRESHOLD,
        destructive: true,
        policy: RepeatedActionPolicy::Deny,
    });

    assert_eq!(repeated_denial.status(), RepeatedActionStatus::Denied);
    assert!(!repeated_denial.adapter_dispatched());
}

#[test]
fn kernel_receipt_leak_scan_rejects_raw_prompt() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.receipts.redacted]
    assert_eq!(
        validate_text(RAW_PROMPT_RECEIPT),
        Err(ReceiptLeak::RawPrompt)
    );
}

fn pattern_error(
    match_kind: Option<&'static str>,
    resource_pattern: Option<&'static str>,
) -> Error {
    PatternRule::validate(&PatternRuleDraft {
        match_kind,
        resource_pattern,
        ..valid_pattern_draft()
    })
    .expect_err("pattern should fail")
}

fn valid_pattern_draft() -> PatternRuleDraft<'static> {
    PatternRuleDraft {
        match_kind: Some(GLOB_MATCH_KIND),
        action: Some(ACTION.as_str()),
        resource_pattern: Some(GLOB_RESOURCE_PATTERN),
        outcome: Some(ALLOW_OUTCOME_LABEL),
        source_identity: Some(SOURCE),
        trust_scope: Some(USER_SCOPE_LABEL),
        trust_status: Some(TRUSTED_LABEL),
        precedence: MIN_PRECEDENCE,
    }
}

fn allowlist_for(resource_hash: &crate::SafeHash) -> SessionAllowlistEntry {
    SessionAllowlistEntry::new(SessionAllowlistEntryDraft {
        action: ACTION,
        resource_hash,
        policy_hash: resource_hash,
        session_label: SESSION,
        token: SESSION_TOKEN,
        invalidated: false,
    })
    .expect("allowlist entry should validate")
}
