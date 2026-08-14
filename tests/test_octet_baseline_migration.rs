use std::collections::BTreeSet;

const MIGRATION: &str = include_str!("fixtures/octet-baseline-migration.rs.txt");
const REQUIRED_FRAGMENTS: &[&str] = &[
    "const ASSERTION_DENSITY_LINT",
    "const FUNCTION_LENGTH_LINT",
    "const ALLOWED_RESOLVED_FILE: &str = \"src/workflow_validation.rs\"",
    "const ALLOWED_RESOLVED_FUNCTION: &str = \"workflow_diagnostic_from_runner\"",
    "const ALLOWED_RESOLVED_HEADLESS_IMPORTS: &[&str] = &[\"`replay`\", \"`workflow_validation`\"]",
    "const ALLOWED_TYPED_IDENTITY_RESOLUTIONS: &[(&str, &str)]",
    "if !migration.unmatched_candidate.is_empty()",
    "migration.unmatched_old.iter().all(|diagnostic| allowed_resolved_finding(diagnostic))",
    "unapproved resolved import was accepted",
    "typed identity resolution was not accepted",
    "unrelated typed identity resolution was accepted",
    "unapproved resolved finding was accepted",
    "unique.entry(stable_id).or_insert(finding)",
    "duplicate stable finding IDs were not deduplicated",
    "semantic finding sets differ outside the exact resolved-finding allowlist",
];

fn migration_source_issues(source: &str) -> BTreeSet<&'static str> {
    let mut issues = BTreeSet::new();
    for fragment in REQUIRED_FRAGMENTS {
        if !source.contains(fragment) {
            issues.insert("migration.required-fragment");
        }
    }
    if !source.contains("assertion-density identity migration did not normalize line-count drift") {
        issues.insert("migration.positive-test");
    }
    if !source.contains("unapproved resolved finding was accepted")
        || !source.contains("unapproved resolved import was accepted")
        || !source.contains("unrelated typed identity resolution was accepted")
    {
        issues.insert("migration.negative-test");
    }
    if !source.contains("duplicate stable finding IDs were not deduplicated") {
        issues.insert("migration.deduplication-test");
    }
    issues
}

// r[verify lattice.repository_validation.octet_baseline_migration]
// r[verify lattice.repository_validation.octet_baseline_migration.scenario.dynamic]
// r[verify lattice.repository_validation.octet_baseline_migration.scenario.resolved]
#[test]
fn octet_baseline_migration_is_bounded_to_dynamic_counts_and_exact_resolved_findings() {
    let issues = migration_source_issues(MIGRATION);

    assert!(issues.is_empty(), "migration issues: {issues:?}");
}

// r[verify lattice.repository_validation.octet_baseline_migration.scenario.fail_closed]
#[test]
fn octet_baseline_migration_guard_drift_fails_closed() {
    let broadened = MIGRATION.replace("if !migration.unmatched_candidate.is_empty()", "if false");
    let removed_negative_test = MIGRATION.replace("unapproved resolved finding was accepted", "removed guard");

    assert!(migration_source_issues(&broadened).contains("migration.required-fragment"));
    assert!(migration_source_issues(&removed_negative_test).contains("migration.negative-test"));
}
