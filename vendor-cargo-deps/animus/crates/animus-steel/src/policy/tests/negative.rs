use super::*;

#[test]
fn bad_profile_name_is_rejected() {
    let error = profile_with_name(BAD_PROFILE_NAME).expect_err("bad profile name should fail");

    assert_eq!(error.reason(), super::super::DenyReason::BadProfileName);
    assert_eq!(error.field(), "name");
}

#[test]
fn non_positive_budget_is_rejected() {
    let error = profile_with_budget(ZERO_TURN_BUDGET).expect_err("zero budget should fail");

    assert_eq!(error.reason(), super::super::DenyReason::InvalidBudget);
}

#[test]
fn unknown_host_action_is_rejected() {
    let error =
        profile_with_actions(&["model", UNKNOWN_ACTION]).expect_err("unknown action should fail");

    assert_eq!(error.reason(), super::super::DenyReason::UnknownHostAction);
}

#[test]
fn path_escape_is_rejected() {
    let error = profile_with_script_label(PATH_ESCAPE_LABEL).expect_err("path escape should fail");

    assert_eq!(error.reason(), super::super::DenyReason::PathEscape);
}

#[test]
fn unsupported_fallback_is_rejected() {
    let error = profile_with_fallback(UNSUPPORTED_FALLBACK).expect_err("fallback should fail");

    assert_eq!(
        error.reason(),
        super::super::DenyReason::UnsupportedFallback
    );
}

#[test]
fn unsupported_redaction_is_rejected() {
    let error = profile_with_redaction(UNSUPPORTED_REDACTION).expect_err("redaction should fail");

    assert_eq!(
        error.reason(),
        super::super::DenyReason::UnsupportedReceiptRedaction
    );
}

#[test]
fn invalid_hash_is_rejected() {
    let error = profile_with_hash(INVALID_HASH).expect_err("invalid hash should fail");

    assert_eq!(error.reason(), super::super::DenyReason::HashMismatch);
}

#[test]
fn missing_script_denies_activation() {
    let profile = valid_profile();
    let script = super::super::Script::new(PROFILE_NAME, SCRIPT_LABEL, "");
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::super::ActivationStatus::Denied);
    assert_eq!(
        receipt.denied_reason(),
        Some(super::super::DenyReason::MissingScript)
    );
}

#[test]
fn hash_mismatch_denies_activation() {
    let profile = valid_profile();
    let script = super::super::Script::new(PROFILE_NAME, SCRIPT_LABEL, OTHER_SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::super::ActivationStatus::Denied);
    assert_eq!(
        receipt.denied_reason(),
        Some(super::super::DenyReason::HashMismatch)
    );
}

#[test]
fn oversized_budget_denies_activation() {
    let profile = valid_profile();
    let script = super::super::Script::new(PROFILE_NAME, SCRIPT_LABEL, SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, TWO_TURN_BUDGET);

    assert_eq!(receipt.status(), super::super::ActivationStatus::Denied);
    assert_eq!(
        receipt.denied_reason(),
        Some(super::super::DenyReason::InvalidBudget)
    );
}

#[test]
fn disallowed_action_denies_activation() {
    let profile = profile_with_actions(&["model"]).expect("model-only profile should build");
    let receipt = profile.activate(&valid_activation(&profile));

    assert_eq!(receipt.status(), super::super::ActivationStatus::Denied);
    assert_eq!(
        receipt.denied_reason(),
        Some(super::super::DenyReason::HostActionDenied)
    );
}

#[test]
fn mismatched_profile_denies_activation() {
    let profile = valid_profile();
    let script = super::super::Script::new("other-loop", SCRIPT_LABEL, SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::super::ActivationStatus::Denied);
    assert_eq!(
        receipt.denied_reason(),
        Some(super::super::DenyReason::ProfileMismatch)
    );
}

#[test]
fn path_escape_denies_activation() {
    let profile = valid_profile();
    let script = super::super::Script::new(PROFILE_NAME, PATH_ESCAPE_LABEL, SCRIPT_SOURCE);
    let receipt = activate_script(&profile, script, ONE_TURN_BUDGET);

    assert_eq!(receipt.status(), super::super::ActivationStatus::Denied);
    assert_eq!(
        receipt.denied_reason(),
        Some(super::super::DenyReason::PathEscape)
    );
}
