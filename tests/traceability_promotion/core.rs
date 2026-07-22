use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct CoverageReceipt {
    pub profile_id: String,
    pub missing: Vec<String>,
    pub dangling: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PromotionIssue {
    pub code: &'static str,
    pub subject: String,
}

impl PromotionIssue {
    fn new(code: &'static str, subject: impl Into<String>) -> Self {
        Self {
            code,
            subject: subject.into(),
        }
    }
}

// r[verify lattice.repository_validation.reviewed_tracey_promotion]
pub fn validate_promotion(
    baseline: &CoverageReceipt,
    current: &CoverageReceipt,
    promoted_ids: &[&str],
) -> Vec<PromotionIssue> {
    let mut issues = Vec::new();
    let baseline_ids = validate_snapshot("baseline", baseline, &mut issues);
    let current_ids = validate_snapshot("current", current, &mut issues);
    let promoted = unique_promoted_ids(promoted_ids, &mut issues);

    if baseline.profile_id != current.profile_id {
        issues.push(PromotionIssue::new("profile.mismatch", &current.profile_id));
    }

    for identifier in &promoted {
        if !baseline_ids.contains(*identifier) {
            issues.push(PromotionIssue::new("allowlist.not_in_baseline", *identifier));
        }
        if current_ids.contains(*identifier) {
            issues.push(PromotionIssue::new("transition.expected_removal_missing", *identifier));
        }
    }
    for identifier in baseline_ids.difference(&current_ids) {
        if !promoted.contains(identifier.as_str()) {
            issues.push(PromotionIssue::new("transition.unexpected_removed", identifier.as_str()));
        }
    }
    for identifier in current_ids.difference(&baseline_ids) {
        issues.push(PromotionIssue::new("transition.unexpected_added", identifier.as_str()));
    }

    issues.sort();
    issues.dedup();
    issues
}

pub fn marker_is_attached_to_test(source: &str, test_name: &str, requirement_id: &str) -> bool {
    let signature = format!("fn {test_name}(");
    let async_signature = format!("async {signature}");
    let lines = source.lines().collect::<Vec<_>>();
    let Some(mut cursor) = lines.iter().position(|line| {
        let line = line.trim_start();
        line.starts_with(&signature) || line.starts_with(&async_signature)
    }) else {
        return false;
    };
    let expected = format!("// r[verify {requirement_id}]");
    while cursor > 0 {
        cursor = cursor.saturating_sub(1);
        let line = lines[cursor].trim();
        if line == expected {
            return true;
        }
        if line == "#[test]" || line == "#[tokio::test]" || line.starts_with("// r[verify ") {
            continue;
        }
        break;
    }
    false
}

fn validate_snapshot(
    prefix: &'static str,
    receipt: &CoverageReceipt,
    issues: &mut Vec<PromotionIssue>,
) -> BTreeSet<String> {
    if receipt.valid {
        issues.push(PromotionIssue::new(
            if prefix == "baseline" {
                "baseline.expected_invalid"
            } else {
                "current.expected_invalid"
            },
            &receipt.profile_id,
        ));
    }
    if !receipt.dangling.is_empty() {
        issues.push(PromotionIssue::new(
            if prefix == "baseline" {
                "baseline.dangling"
            } else {
                "current.dangling"
            },
            receipt.dangling.len().to_string(),
        ));
    }
    if !strictly_sorted(&receipt.missing) {
        issues.push(PromotionIssue::new(
            if prefix == "baseline" {
                "baseline.unsorted"
            } else {
                "current.unsorted"
            },
            &receipt.profile_id,
        ));
    }
    let mut unique = BTreeSet::new();
    for identifier in &receipt.missing {
        if !unique.insert(identifier.clone()) {
            issues.push(PromotionIssue::new(
                if prefix == "baseline" {
                    "baseline.duplicate_missing"
                } else {
                    "current.duplicate_missing"
                },
                identifier,
            ));
        }
    }
    unique
}

fn unique_promoted_ids<'a>(promoted_ids: &'a [&'a str], issues: &mut Vec<PromotionIssue>) -> BTreeSet<&'a str> {
    let mut unique = BTreeSet::new();
    for identifier in promoted_ids {
        if !unique.insert(*identifier) {
            issues.push(PromotionIssue::new("allowlist.duplicate", *identifier));
        }
    }
    unique
}

fn strictly_sorted(values: &[String]) -> bool {
    values.iter().zip(values.iter().skip(1)).all(|(left, right)| left < right)
}
