//! Feature-rule validation.

const DUPLICATE_KEY_SUBJECT: &str = "policy_key";
const ROW_SUBJECT: &str = "row";
const REASON_SUBJECT: &str = "reason";
const FALLBACK_SUBJECT: &str = "fallback";
const STATUS_SUBJECT: &str = "status";
const EVIDENCE_SUBJECT: &str = "evidence";
const NEXT_ACTION_SUBJECT: &str = "next_action";

/// Validates checked-in `Ratatui` feature policy rows.
#[must_use]
pub fn validate_ratatui_feature_policy(
    rows: &[super::types::FeaturePolicyRow],
) -> super::types::FeaturePolicyReport {
    let mut issues = Vec::with_capacity(rows.len());
    let mut seen_keys = std::collections::BTreeSet::new();
    debug_assert!(issues.capacity() >= rows.len());
    debug_assert!(seen_keys.len() <= rows.len());
    for row in rows {
        let key = rule_key(KeyParts {
            crate_family: &row.crate_family,
            feature_name: &row.feature_name,
        });
        if !seen_keys.insert(key.clone()) {
            issues.push(super::types::FeaturePolicyIssue::new(
                super::types::FeaturePolicyIssueCode::DuplicatePolicyKey,
                key.clone(),
                DUPLICATE_KEY_SUBJECT,
                "duplicate Ratatui feature policy key",
            ));
        }
        issues.extend(row_issues(row, &key));
    }
    let mut sorted_rows = rows.to_vec();
    sorted_rows.sort_by(|left, right| {
        rule_key(KeyParts {
            crate_family: &left.crate_family,
            feature_name: &left.feature_name,
        })
        .cmp(&rule_key(KeyParts {
            crate_family: &right.crate_family,
            feature_name: &right.feature_name,
        }))
    });
    super::types::FeaturePolicyReport {
        issues,
        rows: sorted_rows,
    }
}

/// Validates observed feature uses against accepted feature policy rows.
#[must_use]
pub fn validate_ratatui_feature_uses(
    policy_rows: &[super::types::FeaturePolicyRow],
    uses: &[super::types::FeatureUse],
) -> super::types::FeaturePolicyReport {
    let validated = validate_ratatui_feature_policy(policy_rows);
    let accepted_rules = accepted_rule_map(&validated.rows);
    let issue_bound = validated.issues.len().saturating_add(uses.len());
    let mut issues = Vec::with_capacity(issue_bound);
    issues.extend(validated.issues);
    debug_assert!(issues.capacity() >= uses.len());
    for feature_use in uses {
        issues.extend(feature_use_issues(feature_use, &accepted_rules));
    }
    super::types::FeaturePolicyReport {
        issues,
        rows: validated.rows,
    }
}

fn row_issues(
    row: &super::types::FeaturePolicyRow,
    key: &str,
) -> Vec<super::types::FeaturePolicyIssue> {
    let mut issues = Vec::with_capacity(super::types::KNOWN_POLICY_STATUSES.len());
    debug_assert!(issues.capacity() == super::types::KNOWN_POLICY_STATUSES.len());
    if has_missing_row_field(row) {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::MissingRequiredField,
            key,
            ROW_SUBJECT,
            "crate family, feature name, status, and owner are required",
        ));
    }
    if row.reason.trim().is_empty() {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::MissingReason,
            key,
            REASON_SUBJECT,
            "Ratatui feature policy rows require a reason",
        ));
    }
    if row.fallback.trim().is_empty() {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::MissingFallback,
            key,
            FALLBACK_SUBJECT,
            "Ratatui feature policy rows require fallback behavior",
        ));
    }
    let status = row.status.as_str();
    if !super::types::KNOWN_POLICY_STATUSES.contains(&status) {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::UnknownStatus,
            key,
            STATUS_SUBJECT,
            format!("unknown Ratatui feature policy status `{status}`"),
        ));
        return issues;
    }
    if status == super::types::POLICY_APPROVED
        && row.evidence.iter().all(|entry| entry.trim().is_empty())
    {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::MissingEvidence,
            key,
            EVIDENCE_SUBJECT,
            "approved Ratatui feature policy rows require evidence",
        ));
    }
    if status == super::types::POLICY_REQUIRES_CHANGE && row.next_action.trim().is_empty() {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::MissingNextAction,
            key,
            NEXT_ACTION_SUBJECT,
            "staged Ratatui feature policy rows require a next action",
        ));
    }
    issues
}

fn feature_use_issues(
    feature_use: &super::types::FeatureUse,
    rules: &std::collections::BTreeMap<(String, String), String>,
) -> Vec<super::types::FeaturePolicyIssue> {
    let mut issues = Vec::with_capacity(super::types::KNOWN_POLICY_STATUSES.len());
    debug_assert!(issues.capacity() == super::types::KNOWN_POLICY_STATUSES.len());
    let key = rule_key(KeyParts {
        crate_family: &feature_use.crate_family,
        feature_name: &feature_use.feature_name,
    });
    if has_missing_use_field(feature_use) {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::MissingRequiredField,
            key.clone(),
            feature_use.crate_name.clone(),
            "crate name, crate family, and feature name are required",
        ));
    }
    if is_web_incompatible(feature_use) {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::WebIncompatibleFeatureUse,
            key.clone(),
            feature_use.crate_name.clone(),
            "web-consumed crate uses a web-incompatible Ratatui feature",
        ));
    }
    let Some(status) = rules.get(&(
        feature_use.crate_family.clone(),
        feature_use.feature_name.clone(),
    )) else {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::UnapprovedFeatureUse,
            key,
            feature_use.crate_name.clone(),
            "Ratatui feature use has no accepted policy row",
        ));
        return issues;
    };
    if status == super::types::POLICY_FORBIDDEN {
        issues.push(super::types::FeaturePolicyIssue::new(
            super::types::FeaturePolicyIssueCode::ForbiddenFeatureUse,
            key,
            feature_use.crate_name.clone(),
            "Ratatui feature use is explicitly forbidden by policy",
        ));
    }
    issues
}

fn has_missing_row_field(row: &super::types::FeaturePolicyRow) -> bool {
    let is_family_empty = row.crate_family.trim().is_empty();
    let is_feature_empty = row.feature_name.trim().is_empty();
    let is_status_empty = row.status.trim().is_empty();
    let is_owner_empty = row.owner.trim().is_empty();
    is_family_empty || is_feature_empty || is_status_empty || is_owner_empty
}

fn has_missing_use_field(feature_use: &super::types::FeatureUse) -> bool {
    let is_name_empty = feature_use.crate_name.trim().is_empty();
    let is_family_empty = feature_use.crate_family.trim().is_empty();
    let is_feature_empty = feature_use.feature_name.trim().is_empty();
    is_name_empty || is_family_empty || is_feature_empty
}

fn is_web_incompatible(feature_use: &super::types::FeatureUse) -> bool {
    let is_web_target = feature_use.target == super::types::FeatureUseTarget::WebConsumed;
    let is_blocked_feature =
        super::types::WEB_INCOMPATIBLE_FEATURES.contains(&feature_use.feature_name.as_str());
    is_web_target && is_blocked_feature
}

fn accepted_rule_map(
    rows: &[super::types::FeaturePolicyRow],
) -> std::collections::BTreeMap<(String, String), String> {
    rows.iter()
        .filter(|row| super::types::KNOWN_POLICY_STATUSES.contains(&row.status.as_str()))
        .map(|row| {
            (
                (row.crate_family.clone(), row.feature_name.clone()),
                row.status.clone(),
            )
        })
        .collect()
}

struct KeyParts<'a> {
    crate_family: &'a str,
    feature_name: &'a str,
}

fn rule_key(parts: KeyParts<'_>) -> String {
    format!("{}:{}", parts.crate_family, parts.feature_name)
}
