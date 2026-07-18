const APPROVED_ROW_COUNT: usize = 10;
const MALFORMED_ROW_COUNT: usize = 5;
const MIN_MALFORMED_ISSUES: usize = 6;
const POLICY_FEATURE: &str = "crossterm";
const POLICY_FAMILY: &str = "app-shell";
const POLICY_OWNER: &str = "rat-showcase owners";
const POLICY_SCOPE: &str = "native terminal shell";
const POLICY_REASON: &str = "terminal shell owns backend behavior";
const POLICY_FALLBACK: &str = "delegate to pure models";
const POLICY_EVIDENCE: &str = "subwayrat/crates/rat-showcase/Cargo.toml";
const POLICY_NEXT_ACTION: &str = "open a focused Cairn";
const STATUS_APPROVED: &str = "approved";
const STATUS_FORBIDDEN: &str = "forbidden";
const STATUS_REQUIRES_CHANGE: &str = "requires-change";
const STATUS_UNKNOWN: &str = "allowed";
const CRATE_RAT_SHOWCASE: &str = "rat-showcase";
const CRATE_RAT_WIDGETS: &str = "rat-widgets";
const CRATE_RAT_RATZILLA: &str = "rat-ratzilla";
const CRATE_RAT_EXPERIMENT: &str = "rat-experiment";
const REUSABLE_WIDGET_FAMILY: &str = "reusable-widget";
const RATZILLA_WEB_FAMILY: &str = "ratzilla-web";
const UNAPPROVED_FEATURE: &str = "unstable-rendered-line-info";

#[test]
fn baseline_feature_policy_rows_are_valid() {
    let rows = crate::ratatui_integration::baseline_ratatui_feature_policy_rows();

    let outcome = crate::ratatui_integration::validate_ratatui_feature_policy(&rows);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.rows.len(), APPROVED_ROW_COUNT);
    assert!(outcome.rows.iter().any(|row| row.feature_name == "serde"));
}

#[test]
fn approved_native_feature_use_passes() {
    let rows = crate::ratatui_integration::baseline_ratatui_feature_policy_rows();
    let feature_uses = vec![crate::ratatui_integration::FeatureUse::new(
        CRATE_RAT_SHOWCASE,
        POLICY_FAMILY,
        POLICY_FEATURE,
        crate::ratatui_integration::FeatureUseTarget::NativeOnly,
    )];

    let outcome = crate::ratatui_integration::validate_ratatui_feature_uses(&rows, &feature_uses);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
}

#[test]
fn malformed_policy_rows_fail_with_deterministic_diagnostics() {
    let approved_without_evidence = feature_policy_row(
        POLICY_FAMILY,
        POLICY_FEATURE,
        STATUS_APPROVED,
        POLICY_OWNER,
        POLICY_SCOPE,
        POLICY_REASON,
        POLICY_FALLBACK,
    );
    let missing_required = feature_policy_row(
        "",
        "missing-owner",
        STATUS_FORBIDDEN,
        "",
        POLICY_SCOPE,
        "",
        "",
    );
    let staged_without_next_action = feature_policy_row(
        "workspace-policy",
        "macros",
        STATUS_REQUIRES_CHANGE,
        POLICY_OWNER,
        POLICY_SCOPE,
        POLICY_REASON,
        POLICY_FALLBACK,
    );
    let unknown = feature_policy_row(
        "workspace-policy",
        "palette",
        STATUS_UNKNOWN,
        POLICY_OWNER,
        POLICY_SCOPE,
        POLICY_REASON,
        POLICY_FALLBACK,
    );

    let outcome = crate::ratatui_integration::validate_ratatui_feature_policy(&[
        approved_without_evidence.clone(),
        approved_without_evidence,
        missing_required,
        staged_without_next_action,
        unknown,
    ]);

    assert!(!outcome.is_valid());
    assert_eq!(outcome.rows.len(), MALFORMED_ROW_COUNT);
    assert!(outcome.issues.len() >= MIN_MALFORMED_ISSUES);
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::DuplicatePolicyKey,
    );
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::MissingRequiredField,
    );
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::MissingEvidence,
    );
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::MissingReason,
    );
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::MissingFallback,
    );
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::MissingNextAction,
    );
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::UnknownStatus,
    );
}

#[test]
fn forbidden_backend_feature_leak_is_rejected() {
    let rows = crate::ratatui_integration::baseline_ratatui_feature_policy_rows();
    let feature_uses = vec![crate::ratatui_integration::FeatureUse::new(
        CRATE_RAT_WIDGETS,
        REUSABLE_WIDGET_FAMILY,
        POLICY_FEATURE,
        crate::ratatui_integration::FeatureUseTarget::NativeOnly,
    )];

    let outcome = crate::ratatui_integration::validate_ratatui_feature_uses(&rows, &feature_uses);

    assert!(!outcome.is_valid());
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::ForbiddenFeatureUse,
    );
}

#[test]
fn web_incompatible_feature_use_is_rejected() {
    let rows = crate::ratatui_integration::baseline_ratatui_feature_policy_rows();
    let feature_uses = vec![crate::ratatui_integration::FeatureUse::new(
        CRATE_RAT_RATZILLA,
        RATZILLA_WEB_FAMILY,
        POLICY_FEATURE,
        crate::ratatui_integration::FeatureUseTarget::WebConsumed,
    )];

    let outcome = crate::ratatui_integration::validate_ratatui_feature_uses(&rows, &feature_uses);

    assert!(!outcome.is_valid());
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::WebIncompatibleFeatureUse,
    );
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::ForbiddenFeatureUse,
    );
}

#[test]
fn unapproved_unstable_feature_use_is_rejected() {
    let rows = crate::ratatui_integration::baseline_ratatui_feature_policy_rows();
    let feature_uses = vec![crate::ratatui_integration::FeatureUse::new(
        CRATE_RAT_EXPERIMENT,
        POLICY_FAMILY,
        UNAPPROVED_FEATURE,
        crate::ratatui_integration::FeatureUseTarget::NativeOnly,
    )];

    let outcome = crate::ratatui_integration::validate_ratatui_feature_uses(&rows, &feature_uses);

    assert!(!outcome.is_valid());
    assert_issue(
        &outcome.issues,
        crate::ratatui_integration::FeaturePolicyIssueCode::UnapprovedFeatureUse,
    );
}

#[test]
fn staged_policy_rows_pass_when_next_action_is_named() {
    let row = feature_policy_row(
        "workspace-policy",
        "macros",
        STATUS_REQUIRES_CHANGE,
        POLICY_OWNER,
        POLICY_SCOPE,
        POLICY_REASON,
        POLICY_FALLBACK,
    )
    .with_next_action(POLICY_NEXT_ACTION);

    let outcome = crate::ratatui_integration::validate_ratatui_feature_policy(&[row]);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
}

#[test]
fn approved_rows_pass_when_evidence_is_named() {
    let row = feature_policy_row(
        POLICY_FAMILY,
        POLICY_FEATURE,
        STATUS_APPROVED,
        POLICY_OWNER,
        POLICY_SCOPE,
        POLICY_REASON,
        POLICY_FALLBACK,
    )
    .with_evidence(POLICY_EVIDENCE);

    let outcome = crate::ratatui_integration::validate_ratatui_feature_policy(&[row]);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
}

fn assert_issue(
    issues: &[crate::ratatui_integration::FeaturePolicyIssue],
    code: crate::ratatui_integration::FeaturePolicyIssueCode,
) {
    assert!(issues.iter().any(|issue| issue.code == code));
}

fn feature_policy_row(
    crate_family: &str,
    feature_name: &str,
    status: &str,
    owner: &str,
    compatibility_scope: &str,
    reason: &str,
    fallback: &str,
) -> crate::ratatui_integration::FeaturePolicyRow {
    crate::ratatui_integration::FeaturePolicyRow::new(
        crate::ratatui_integration::FeaturePolicyRowParts {
            crate_family: crate_family.to_string(),
            feature_name: feature_name.to_string(),
            status: status.to_string(),
            owner: owner.to_string(),
            compatibility_scope: compatibility_scope.to_string(),
            reason: reason.to_string(),
            fallback: fallback.to_string(),
        },
    )
}
