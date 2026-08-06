#![allow(unknown_lints)]
#![allow(path_segment_repetition, underscore_in_module_filename)]

use super::*;

#[test]
fn standard_mode_allows_after_stronger_gates_pass() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.modes]
    let mode = standard_mode_resolution();

    assert_eq!(mode.status(), DecisionStatus::Execute);
    assert_eq!(mode.reason(), ModeDefaultReason::ModeAllowed);
    assert_eq!(mode.local_outcome(), Outcome::Allow);
    assert!(mode.dispatch_allowed());
}

#[test]
fn trusted_glob_pattern_matches_tool_resource() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.pattern_rules]
    let pattern = trusted_glob_pattern();

    assert_eq!(pattern.match_kind(), PatternMatchKind::Glob);
    assert_eq!(pattern.action(), ACTION);
    assert_eq!(pattern.resource_pattern(), GLOB_RESOURCE_PATTERN);
    assert_eq!(pattern.outcome(), Outcome::Ask);
    assert_eq!(pattern.source_identity(), SOURCE);
    assert_eq!(pattern.trust_scope(), TrustScope::User);
    assert_eq!(pattern.trust_status(), TrustStatus::Trusted);
    assert_eq!(pattern.precedence(), MIN_PRECEDENCE);
    assert!(pattern.matches(ACTION, RESOURCE));
    assert!(!pattern.rule_hash().is_empty());
}

#[test]
fn scoped_allowlist_accepts_current_request() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.session_allowlist]
    let resource_hash = safe_hash();
    let policy_hash = second_safe_hash();
    let allowlist = current_allowlist(&resource_hash, &policy_hash);
    let status = validate_session_allowlist(SessionAllowlistValidation {
        entry: &allowlist,
        action: ACTION,
        resource_hash: &resource_hash,
        policy_hash: &policy_hash,
        session_label: SESSION,
    });

    assert_eq!(allowlist.action(), ACTION);
    assert_eq!(allowlist.resource_hash().as_str(), HASH);
    assert_eq!(allowlist.policy_hash().as_str(), SECOND_HASH);
    assert_eq!(allowlist.session_label(), SESSION);
    assert_eq!(allowlist.token(), SESSION_TOKEN);
    assert_eq!(status, SessionAllowlistStatus::Accepted);
}

#[test]
fn repeated_action_coaches_without_adapter_dispatch() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.repeated_action]
    let resource_hash = safe_hash();
    let repeated = detect_repeated_action(RepeatedActionInput {
        action: ACTION,
        resource_hash: &resource_hash,
        prior_observations: REPEAT_PRIOR_OBSERVATIONS,
        threshold: REPEAT_THRESHOLD,
        destructive: false,
        policy: RepeatedActionPolicy::Coach,
    });

    assert_eq!(repeated.status(), RepeatedActionStatus::Coach);
    assert!(!repeated.adapter_dispatched());
}

#[test]
fn kernel_receipt_records_hashes_and_redactions_only() {
    // r[verify agent_harness_runtime.zerostack_permission_kernel.receipts]
    let resource_hash = safe_hash();
    let policy_hash = second_safe_hash();
    let repeated = repeated_coach(&resource_hash);
    let receipt = KernelReceipt::new(KernelReceiptDraft {
        action: ACTION,
        resource_hash: &resource_hash,
        authority_status: crate::AuthorizationStatus::Allowed,
        mode_outcome: standard_mode_resolution().local_outcome(),
        matched_rule_hash: Some(&policy_hash),
        repeated_action_status: repeated.status(),
        prompt_status: PromptStatus::NotRequired,
        final_status: DecisionStatus::Execute,
        redaction_markers: &[REDACTION, SECOND_REDACTION],
    })
    .expect("kernel receipt should validate");

    assert_eq!(receipt.action(), ACTION);
    assert_eq!(receipt.resource_hash().as_str(), HASH);
    assert_eq!(receipt.authority_status(), crate::AuthorizationStatus::Allowed);
    assert_eq!(receipt.mode_outcome(), Outcome::Allow);
    assert_eq!(receipt.matched_rule_hash().map(crate::SafeHash::as_str), Some(SECOND_HASH));
    assert_eq!(receipt.repeated_action_status(), RepeatedActionStatus::Coach);
    assert_eq!(receipt.prompt_status(), PromptStatus::NotRequired);
    assert_eq!(receipt.final_status(), DecisionStatus::Execute);
    assert_eq!(receipt.redaction_markers(), &[Text::from(REDACTION), Text::from(SECOND_REDACTION)]);
}

fn standard_mode_resolution() -> ModeDefaultResolution {
    resolve_mode_default(ModeDefaultRequest {
        mode: PermissionMode::Standard,
        action: ACTION,
        authority_status: crate::AuthorizationStatus::Allowed,
        sandbox_allows: true,
        redaction_allows: true,
    })
}

fn trusted_glob_pattern() -> PatternRule {
    PatternRule::validate(&PatternRuleDraft {
        match_kind: Some(GLOB_MATCH_KIND),
        action: Some(ACTION.as_str()),
        resource_pattern: Some(GLOB_RESOURCE_PATTERN),
        outcome: Some(ASK_OUTCOME_LABEL),
        source_identity: Some(SOURCE),
        trust_scope: Some(USER_SCOPE_LABEL),
        trust_status: Some(TRUSTED_LABEL),
        precedence: MIN_PRECEDENCE,
    })
    .expect("trusted pattern rule should validate")
}

fn current_allowlist(resource_hash: &crate::SafeHash, policy_hash: &crate::SafeHash) -> SessionAllowlistEntry {
    SessionAllowlistEntry::new(SessionAllowlistEntryDraft {
        action: ACTION,
        resource_hash,
        policy_hash,
        session_label: SESSION,
        token: SESSION_TOKEN,
        invalidated: false,
    })
    .expect("scoped token should validate")
}

fn repeated_coach(resource_hash: &crate::SafeHash) -> RepeatedActionResolution {
    detect_repeated_action(RepeatedActionInput {
        action: ACTION,
        resource_hash,
        prior_observations: REPEAT_PRIOR_OBSERVATIONS,
        threshold: REPEAT_THRESHOLD,
        destructive: false,
        policy: RepeatedActionPolicy::Coach,
    })
}
