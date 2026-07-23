use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct CoverageReceipt {
    pub profile_id: String,
    pub requirements: usize,
    pub referenced: usize,
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
#[allow(dead_code)]
pub fn validate_promotion(
    baseline: &CoverageReceipt,
    current: &CoverageReceipt,
    promoted_ids: &[&str],
) -> Vec<PromotionIssue> {
    validate_transition(baseline, current, promoted_ids, false)
}

#[allow(dead_code)]
pub fn validate_final_promotion(
    baseline: &CoverageReceipt,
    current: &CoverageReceipt,
    promoted_ids: &[&str],
) -> Vec<PromotionIssue> {
    validate_transition(baseline, current, promoted_ids, true)
}

fn validate_transition(
    baseline: &CoverageReceipt,
    current: &CoverageReceipt,
    promoted_ids: &[&str],
    expect_complete_current: bool,
) -> Vec<PromotionIssue> {
    let mut issues = Vec::new();
    let baseline_ids = validate_snapshot("baseline", baseline, false, &mut issues);
    let current_ids = validate_snapshot("current", current, expect_complete_current, &mut issues);
    let promoted = unique_promoted_ids(promoted_ids, &mut issues);

    if expect_complete_current && !current_ids.is_empty() {
        issues.push(PromotionIssue::new("current.expected_complete", current_ids.len().to_string()));
    }
    if expect_complete_current && current.requirements != current.referenced {
        issues.push(PromotionIssue::new(
            "current.expected_full_reference",
            format!("{}:{}", current.requirements, current.referenced),
        ));
    }
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

#[allow(dead_code)]
pub fn validate_marker_bindings(source: &str, bindings: &[(&str, &str)]) -> Vec<PromotionIssue> {
    let mut issues = Vec::new();
    for (requirement_id, test_name) in bindings {
        if !marker_is_attached_to_test(source, test_name, requirement_id) {
            issues.push(PromotionIssue::new("binding.missing", format!("{requirement_id}:{test_name}")));
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

#[allow(dead_code)]
pub fn validate_marker_occurrences(sources: &[&str], bindings: &[(&str, &str)]) -> Vec<PromotionIssue> {
    let mut expected_counts = BTreeMap::new();
    for (requirement_id, _) in bindings {
        *expected_counts.entry(*requirement_id).or_insert(0_usize) += 1;
    }

    let mut issues = Vec::new();
    for (requirement_id, expected_count) in expected_counts {
        let marker = format!("// r[verify {requirement_id}]");
        let actual_count = sources.iter().map(|source| source.matches(&marker).count()).sum::<usize>();
        if actual_count != expected_count {
            issues.push(PromotionIssue::new(
                "binding.occurrence_count",
                format!("{requirement_id}:{expected_count}:{actual_count}"),
            ));
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

#[allow(dead_code)]
pub fn validate_policy_excludes_promoted_ids(policy: &str, promoted_ids: &[&str]) -> Vec<PromotionIssue> {
    let mut issues = Vec::new();
    for identifier in promoted_ids {
        if policy.contains(identifier) {
            issues.push(PromotionIssue::new("policy.promoted_present", *identifier));
        }
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
    expected_valid: bool,
    issues: &mut Vec<PromotionIssue>,
) -> BTreeSet<String> {
    if receipt.valid != expected_valid {
        issues.push(PromotionIssue::new(
            match (prefix, expected_valid) {
                ("baseline", false) => "baseline.expected_invalid",
                ("current", false) => "current.expected_invalid",
                ("current", true) => "current.expected_valid",
                _ => "baseline.expected_valid",
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
