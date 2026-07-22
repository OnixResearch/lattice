#[path = "../scripts/traceability_inventory/core.rs"]
mod core;

use core::Classification;
use core::CoverageReceipt;
use core::EvidenceReference;
use core::InventoryReport;
use core::ReviewDecision;
use core::ReviewPolicy;
use core::build_inventory;
use core::expected_evidence;
use core::missing_set_hash;
use core::verify_report;
use std::collections::BTreeMap;

const SCHEMA_VERSION: &str = "lattice.traceability-debt-inventory.v1";
const HASH_ALGORITHM: &str = "BLAKE3";
const PROFILE_ID: &str = "cairn-default";
const FIRST_ID: &str = "lattice.example.alpha.scenario.valid";
const SECOND_ID: &str = "lattice.example.beta.scenario.invalid";
const STALE_ID: &str = "lattice.example.stale.scenario.removed";
const EVIDENCE_PATH: &str = "tests/test_example.rs";
const RATIONALE_CODE: &str = "assertions.example";
const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const INVALID_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SAMPLE_REQUIREMENTS: usize = 3;
const SAMPLE_REFERENCED: usize = 1;
const SAMPLE_MISSING_COUNT: usize = 2;
const REVIEWED_COUNT: usize = 1;
const SAMPLE_UNASSESSED_COUNT: usize = 1;
const CURRENT_MISSING_COUNT: usize = 317;
const CURRENT_REVIEWED_COUNT: usize = 2;
const CURRENT_UNASSESSED_COUNT: usize = 315;
const CURRENT_EXACT_COUNT: usize = 1;
const CURRENT_NO_EXACT_COUNT: usize = 1;
const CURRENT_MISSING_HASH: &str = "a4526fac7d7f14c594c1a900a74dd4fd118028c8855ce773b114587aa81bf3bd";
const GENERATED_INVENTORY: &str = include_str!("../cairn-policy/generated/traceability-debt-inventory.json");
const REQUIRED_UNASSESSED_NON_CLAIM: &str = "unassessed does not mean behavior or evidence is absent";

fn sample_coverage() -> CoverageReceipt {
    CoverageReceipt {
        profile_id: PROFILE_ID.to_string(),
        requirements: SAMPLE_REQUIREMENTS,
        referenced: SAMPLE_REFERENCED,
        missing: vec![FIRST_ID.to_string(), SECOND_ID.to_string()],
        dangling: Vec::new(),
        policy_hash: VALID_HASH.to_string(),
        input_hash: VALID_HASH.to_string(),
        receipt_hash: VALID_HASH.to_string(),
        valid: false,
    }
}

fn sample_policy(coverage: &CoverageReceipt) -> ReviewPolicy {
    ReviewPolicy {
        schema_version: SCHEMA_VERSION.to_string(),
        hash_algorithm: HASH_ALGORITHM.to_string(),
        expected_missing_count: coverage.missing.len(),
        expected_missing_hash: missing_set_hash(&coverage.missing),
        reviewed: vec![ReviewDecision {
            requirement_id: FIRST_ID.to_string(),
            classification: Classification::ExactMaintainedEvidence,
            evidence: vec![EvidenceReference {
                path: EVIDENCE_PATH.to_string(),
                blake3: VALID_HASH.to_string(),
            }],
            rationale_code: RATIONALE_CODE.to_string(),
        }],
        non_claims: vec![
            "inventory rows do not satisfy requirements or create Tracey evidence".to_string(),
            REQUIRED_UNASSESSED_NON_CLAIM.to_string(),
            "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness".to_string(),
        ],
    }
}

fn sample_observed_evidence() -> BTreeMap<String, String> {
    BTreeMap::from([(EVIDENCE_PATH.to_string(), VALID_HASH.to_string())])
}

fn issue_codes(report: &InventoryReport) -> Vec<&str> {
    report.issues.iter().map(|issue| issue.code.as_str()).collect()
}

// r[verify lattice.repository_validation.traceability_debt_inventory.scenario.complete]
#[test]
fn deterministic_inventory_partitions_every_missing_id_once() {
    let coverage = sample_coverage();
    let policy = sample_policy(&coverage);
    let observed_evidence = sample_observed_evidence();
    let first = build_inventory(&coverage, &policy, &observed_evidence);
    let second = build_inventory(&coverage, &policy, &observed_evidence);

    assert!(first.valid, "issues: {:?}", first.issues);
    assert_eq!(first, second);
    assert_eq!(first.summary.total, SAMPLE_MISSING_COUNT);
    assert_eq!(first.summary.reviewed, REVIEWED_COUNT);
    assert_eq!(first.summary.unassessed, SAMPLE_UNASSESSED_COUNT);
    assert_eq!(first.rows[0].requirement_id, FIRST_ID);
    assert_eq!(first.rows[1].requirement_id, SECOND_ID);
    assert_eq!(expected_evidence(&policy).expect("review evidence references must validate"), vec![
        EvidenceReference {
            path: EVIDENCE_PATH.to_string(),
            blake3: VALID_HASH.to_string(),
        }
    ]);
    assert!(verify_report(&first).is_empty());
}

// r[verify lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed]
#[test]
fn duplicate_stale_and_unsafe_review_inputs_fail_closed() {
    let mut coverage = sample_coverage();
    coverage.missing.push(FIRST_ID.to_string());
    let mut policy = sample_policy(&coverage);
    policy.reviewed.push(ReviewDecision {
        requirement_id: STALE_ID.to_string(),
        classification: Classification::PartialEvidenceOnly,
        evidence: vec![EvidenceReference {
            path: "../outside.rs".to_string(),
            blake3: VALID_HASH.to_string(),
        }],
        rationale_code: RATIONALE_CODE.to_string(),
    });
    let report = build_inventory(&coverage, &policy, &sample_observed_evidence());
    let codes = issue_codes(&report);

    assert!(!report.valid);
    assert!(codes.contains(&"coverage.duplicate_missing"));
    assert!(codes.contains(&"review.id_not_missing"));
    assert!(codes.contains(&"review.evidence_path"));
    assert!(codes.contains(&"review.evidence_unavailable"));
}

// r[verify lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed]
#[test]
fn changed_missing_identity_and_tampered_report_are_rejected() {
    let coverage = sample_coverage();
    let mut policy = sample_policy(&coverage);
    policy.expected_missing_hash = INVALID_HASH.to_string();
    let observed_evidence = sample_observed_evidence();
    let invalid = build_inventory(&coverage, &policy, &observed_evidence);
    assert!(issue_codes(&invalid).contains(&"policy.expected_missing_hash"));

    let mut valid = build_inventory(&coverage, &sample_policy(&coverage), &observed_evidence);
    valid.rows[0].classification = Classification::NoExactEvidenceIdentified;
    let verification_codes = verify_report(&valid).into_iter().map(|issue| issue.code).collect::<Vec<_>>();
    assert!(verification_codes.iter().any(|code| code == "report.inventory_hash"));
    assert!(verification_codes.iter().any(|code| code == "report.summary"));
}

// r[verify lattice.repository_validation.traceability_debt_inventory.scenario.fail_closed]
#[test]
fn evidence_bytes_and_coverage_provenance_are_bound() {
    let coverage = sample_coverage();
    let policy = sample_policy(&coverage);
    let changed_evidence = BTreeMap::from([(EVIDENCE_PATH.to_string(), INVALID_HASH.to_string())]);
    let invalid = build_inventory(&coverage, &policy, &changed_evidence);
    assert!(issue_codes(&invalid).contains(&"review.evidence_hash"));

    let baseline = build_inventory(&coverage, &policy, &sample_observed_evidence());
    let mut changed_coverage = coverage;
    changed_coverage.input_hash = INVALID_HASH.to_string();
    let changed = build_inventory(&changed_coverage, &policy, &sample_observed_evidence());
    assert!(changed.valid, "issues: {:?}", changed.issues);
    assert_ne!(baseline.inventory_hash, changed.inventory_hash);
}

// r[verify lattice.repository_validation.traceability_debt_inventory.scenario.non_claim]
#[test]
fn missing_decision_remains_unassessed_without_absence_claim() {
    let coverage = sample_coverage();
    let report = build_inventory(&coverage, &sample_policy(&coverage), &sample_observed_evidence());
    let unassessed = report
        .rows
        .iter()
        .find(|row| row.requirement_id == SECOND_ID)
        .expect("second requirement must be inventoried");

    assert_eq!(unassessed.classification, Classification::Unassessed);
    assert!(unassessed.evidence.is_empty());
    assert_eq!(unassessed.rationale_code, "not-reviewed");
    assert!(report.non_claims.iter().any(|value| value == REQUIRED_UNASSESSED_NON_CLAIM));
}

// r[verify lattice.repository_validation.traceability_debt_inventory.scenario.complete]
// r[verify lattice.repository_validation.traceability_debt_inventory.scenario.non_claim]
#[test]
fn checked_inventory_pins_current_debt_without_promoting_it() {
    let report: InventoryReport = serde_json::from_str(GENERATED_INVENTORY).expect("generated inventory must decode");

    assert!(report.valid, "issues: {:?}", report.issues);
    assert_eq!(report.summary.total, CURRENT_MISSING_COUNT);
    assert_eq!(report.summary.reviewed, CURRENT_REVIEWED_COUNT);
    assert_eq!(report.summary.unassessed, CURRENT_UNASSESSED_COUNT);
    assert_eq!(report.missing_set_hash, CURRENT_MISSING_HASH);
    assert_eq!(report.summary.by_classification.get("exact_maintained_evidence"), Some(&CURRENT_EXACT_COUNT));
    assert!(!report.summary.by_classification.contains_key("partial_evidence_only"));
    assert_eq!(report.summary.by_classification.get("no_exact_evidence_identified"), Some(&CURRENT_NO_EXACT_COUNT));
    assert_eq!(report.summary.by_classification.get("unassessed"), Some(&CURRENT_UNASSESSED_COUNT));
    assert!(verify_report(&report).is_empty());
}
