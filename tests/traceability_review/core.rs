use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRow {
    pub requirement_id: String,
    pub subsystem: String,
    pub classification: String,
    pub rationale_code: String,
    pub evidence_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedDecision {
    pub requirement_id: &'static str,
    pub classification: &'static str,
    pub rationale_code: &'static str,
    pub evidence_paths: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageSnapshot {
    pub missing: Vec<String>,
    pub dangling: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReviewIssue {
    pub code: &'static str,
    pub subject: String,
}

impl ReviewIssue {
    fn new(code: &'static str, subject: impl Into<String>) -> Self {
        Self {
            code,
            subject: subject.into(),
        }
    }
}

// r[verify lattice.repository_validation.traceability_tranche_review]
pub fn validate_tranche(rows: &[ReviewRow], subsystem: &str, expected: &[ExpectedDecision]) -> Vec<ReviewIssue> {
    let mut issues = Vec::new();
    let mut expected_by_id = BTreeMap::new();
    for decision in expected {
        if expected_by_id.insert(decision.requirement_id, decision).is_some() {
            issues.push(ReviewIssue::new("expectation.duplicate", decision.requirement_id));
        }
    }

    let mut observed_by_id = BTreeMap::new();
    for row in rows.iter().filter(|row| row.subsystem == subsystem) {
        if observed_by_id.insert(row.requirement_id.as_str(), row).is_some() {
            issues.push(ReviewIssue::new("review.duplicate", &row.requirement_id));
        }
        if !expected_by_id.contains_key(row.requirement_id.as_str()) {
            issues.push(ReviewIssue::new("review.unexpected", &row.requirement_id));
        }
    }

    for (requirement_id, decision) in expected_by_id {
        let Some(row) = observed_by_id.get(requirement_id) else {
            issues.push(ReviewIssue::new("review.missing", requirement_id));
            continue;
        };
        if row.classification != decision.classification {
            issues.push(ReviewIssue::new("review.classification", requirement_id));
        }
        if row.rationale_code != decision.rationale_code {
            issues.push(ReviewIssue::new("review.rationale", requirement_id));
        }
        let mut observed_paths = row.evidence_paths.clone();
        observed_paths.sort();
        let mut expected_paths = decision.evidence_paths.to_vec();
        expected_paths.sort();
        if observed_paths != expected_paths {
            issues.push(ReviewIssue::new("review.evidence_paths", requirement_id));
        }
    }

    sort_issues(&mut issues);
    issues
}

#[allow(dead_code)]
pub fn validate_no_selected_markers(sources: &[&str], selected_ids: &[&str]) -> Vec<ReviewIssue> {
    let mut issues = Vec::new();
    let mut unique = BTreeSet::new();
    for identifier in selected_ids {
        if !unique.insert(*identifier) {
            issues.push(ReviewIssue::new("marker.duplicate_selected", *identifier));
        }
        let implementation_marker = format!("r[impl {identifier}]");
        let verification_marker = format!("r[verify {identifier}]");
        if sources
            .iter()
            .any(|source| source.contains(&implementation_marker) || source.contains(&verification_marker))
        {
            issues.push(ReviewIssue::new("marker.promoted", *identifier));
        }
    }
    sort_issues(&mut issues);
    issues
}

pub fn validate_unchanged_coverage(baseline: &CoverageSnapshot, current: &CoverageSnapshot) -> Vec<ReviewIssue> {
    let mut issues = Vec::new();
    validate_snapshot("baseline", baseline, &mut issues);
    validate_snapshot("current", current, &mut issues);
    let baseline_set = baseline.missing.iter().collect::<BTreeSet<_>>();
    let current_set = current.missing.iter().collect::<BTreeSet<_>>();
    for identifier in baseline_set.difference(&current_set) {
        issues.push(ReviewIssue::new("coverage.removed", identifier.as_str()));
    }
    for identifier in current_set.difference(&baseline_set) {
        issues.push(ReviewIssue::new("coverage.added", identifier.as_str()));
    }
    sort_issues(&mut issues);
    issues
}

fn validate_snapshot(prefix: &str, snapshot: &CoverageSnapshot, issues: &mut Vec<ReviewIssue>) {
    if snapshot.valid {
        issues.push(ReviewIssue::new(
            if prefix == "baseline" {
                "coverage.baseline_valid"
            } else {
                "coverage.current_valid"
            },
            prefix,
        ));
    }
    if !snapshot.dangling.is_empty() {
        issues.push(ReviewIssue::new(
            if prefix == "baseline" {
                "coverage.baseline_dangling"
            } else {
                "coverage.current_dangling"
            },
            snapshot.dangling.len().to_string(),
        ));
    }
    if !strictly_sorted(&snapshot.missing) {
        issues.push(ReviewIssue::new(
            if prefix == "baseline" {
                "coverage.baseline_unsorted"
            } else {
                "coverage.current_unsorted"
            },
            prefix,
        ));
    }
    let mut unique = BTreeSet::new();
    for identifier in &snapshot.missing {
        if !unique.insert(identifier) {
            issues.push(ReviewIssue::new(
                if prefix == "baseline" {
                    "coverage.baseline_duplicate"
                } else {
                    "coverage.current_duplicate"
                },
                identifier,
            ));
        }
    }
}

fn strictly_sorted(values: &[String]) -> bool {
    values.iter().zip(values.iter().skip(1)).all(|(left, right)| left < right)
}

fn sort_issues(issues: &mut Vec<ReviewIssue>) {
    issues.sort();
    issues.dedup();
}
