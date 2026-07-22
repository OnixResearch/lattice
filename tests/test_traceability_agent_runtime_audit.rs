#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-repository-validation-promotion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-agent-runtime-audit-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-agent-runtime-audit-reviewed.json");
const CORE: &str = include_str!("fixtures/traceability-agent-runtime-audit-reviewed-core.txt");
const SHELL: &str = include_str!("fixtures/traceability-agent-runtime-audit-reviewed-shell.txt");
const SHELL_TESTS: &str = include_str!("fixtures/traceability-agent-runtime-audit-reviewed-shell-tests.txt");
const ANIMUS_TESTS: &str = include_str!("fixtures/traceability-agent-runtime-audit-reviewed-animus-tests.txt");
const OPENAI_TESTS: &str = include_str!("fixtures/traceability-agent-runtime-audit-reviewed-openai-execution.txt");
const CORE_PATH: &str = "src/animus_agent/core.rs";
const SHELL_PATH: &str = "src/animus_agent/shell.rs";
const SHELL_TESTS_PATH: &str = "src/animus_agent/shell/tests.rs";
const ANIMUS_TESTS_PATH: &str = "tests/test_animus_agent.rs";
const OPENAI_TESTS_PATH: &str = "tests/openai/execution.rs";
const SUBSYSTEM: &str = "agent-runtime";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const SELECTED_COUNT: usize = 14;

const COMPAT_MISSING: &str = "lattice.agent_runtime.animus_compatibility.scenario.missing";
const COMPAT_ROUTES: &str = "lattice.agent_runtime.animus_compatibility.scenario.routes";
const EFFECT_CREDENTIAL: &str = "lattice.agent_runtime.animus_effects.scenario.credential";
const EFFECT_FAILURE: &str = "lattice.agent_runtime.animus_effects.scenario.provider-failure";
const EFFECT_TOOL: &str = "lattice.agent_runtime.animus_effects.scenario.tool-denied";
const FABRIC_COMPLETE: &str = "lattice.agent_runtime.animus_fabric.scenario.complete";
const FABRIC_EXHAUSTION: &str = "lattice.agent_runtime.animus_fabric.scenario.exhaustion";
const FABRIC_IDENTITY: &str = "lattice.agent_runtime.animus_fabric.scenario.identity";
const LEAF_AUTHORITY: &str = "lattice.agent_runtime.animus_leaf.scenario.authority";
const LEAF_EXECUTE: &str = "lattice.agent_runtime.animus_leaf.scenario.execute";
const OUTPUT_NON_AUTH: &str = "lattice.agent_runtime.animus_output.scenario.non-authoritative";
const OUTPUT_TYPED: &str = "lattice.agent_runtime.animus_output.scenario.typed";
const OUTPUT_UNSUPPORTED: &str = "lattice.agent_runtime.animus_output.scenario.unsupported";
const REASONING_PLAIN: &str = "lattice.agent_runtime.inline_reasoning_boundary.scenario.plain";
const SELECTED: &[&str] = &[
    COMPAT_MISSING,
    COMPAT_ROUTES,
    EFFECT_CREDENTIAL,
    EFFECT_FAILURE,
    EFFECT_TOOL,
    FABRIC_COMPLETE,
    FABRIC_EXHAUSTION,
    FABRIC_IDENTITY,
    LEAF_AUTHORITY,
    LEAF_EXECUTE,
    OUTPUT_NON_AUTH,
    OUTPUT_TYPED,
    OUTPUT_UNSUPPORTED,
    REASONING_PLAIN,
];
const UNRELATED: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

#[derive(Debug, serde::Deserialize)]
struct InventoryReport {
    rows: Vec<InventoryRow>,
}
#[derive(Debug, serde::Deserialize)]
struct InventoryRow {
    requirement_id: String,
    subsystem: String,
    classification: String,
    evidence: Vec<Evidence>,
    rationale_code: String,
}
#[derive(Debug, serde::Deserialize)]
struct Evidence {
    path: String,
    blake3: String,
}
#[derive(Debug, serde::Deserialize)]
struct Coverage {
    missing: Vec<String>,
    dangling: Vec<String>,
    valid: bool,
}

fn expected() -> Vec<ExpectedDecision> {
    vec![
        ExpectedDecision {
            requirement_id: COMPAT_MISSING,
            classification: PARTIAL,
            rationale_code: "gap.workflow-and-orchestration-state-unasserted",
            evidence_paths: &[ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: COMPAT_ROUTES,
            classification: EXACT,
            rationale_code: "assertions.existing-route-bypasses-animus",
            evidence_paths: &[ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: EFFECT_CREDENTIAL,
            classification: PARTIAL,
            rationale_code: "gap.events-and-output-credential-omission-unasserted",
            evidence_paths: &[SHELL_PATH, ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: EFFECT_FAILURE,
            classification: PARTIAL,
            rationale_code: "gap.redacted-no-fallback-failure-unasserted",
            evidence_paths: &[SHELL_PATH, ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: EFFECT_TOOL,
            classification: EXACT,
            rationale_code: "assertions.absent-tool-adapter-fails-without-model-effect",
            evidence_paths: &[SHELL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: FABRIC_COMPLETE,
            classification: EXACT,
            rationale_code: "assertions.fixed-loop-dispatches-once-and-returns",
            evidence_paths: &[ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: FABRIC_EXHAUSTION,
            classification: EXACT,
            rationale_code: "assertions.turn-ceiling-blocks-additional-dispatch",
            evidence_paths: &[SHELL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: FABRIC_IDENTITY,
            classification: PARTIAL,
            rationale_code: "gap.identity-rejection-before-transport-unasserted",
            evidence_paths: &[CORE_PATH, ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: LEAF_AUTHORITY,
            classification: PARTIAL,
            rationale_code: "gap.authority-bearing-output-inertness-unasserted",
            evidence_paths: &[CORE_PATH, ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: LEAF_EXECUTE,
            classification: EXACT,
            rationale_code: "assertions.runner-executes-one-typed-animus-node",
            evidence_paths: &[ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: OUTPUT_NON_AUTH,
            classification: PARTIAL,
            rationale_code: "gap.incomplete-status-mapping-unasserted",
            evidence_paths: &[SHELL_PATH, SHELL_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: OUTPUT_TYPED,
            classification: EXACT,
            rationale_code: "assertions.text-json-values-and-safe-metadata",
            evidence_paths: &[ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: OUTPUT_UNSUPPORTED,
            classification: PARTIAL,
            rationale_code: "gap.unsupported-zero-dispatch-unasserted",
            evidence_paths: &[CORE_PATH, ANIMUS_TESTS_PATH],
        },
        ExpectedDecision {
            requirement_id: REASONING_PLAIN,
            classification: EXACT,
            rationale_code: "assertions.plain-text-and-json-preserved",
            evidence_paths: &[OPENAI_TESTS_PATH],
        },
    ]
}

fn rows(report: &InventoryReport) -> Vec<ReviewRow> {
    report
        .rows
        .iter()
        .map(|row| ReviewRow {
            requirement_id: row.requirement_id.clone(),
            subsystem: row.subsystem.clone(),
            classification: row.classification.clone(),
            rationale_code: row.rationale_code.clone(),
            evidence_paths: row.evidence.iter().map(|evidence| evidence.path.clone()).collect(),
        })
        .collect()
}

fn coverage(source: &str) -> CoverageSnapshot {
    let value: Coverage = serde_json::from_str(source).expect("coverage decodes");
    CoverageSnapshot {
        missing: value.missing,
        dangling: value.dangling,
        valid: value.valid,
    }
}

fn source(path: &str) -> &'static str {
    match path {
        CORE_PATH => CORE,
        SHELL_PATH => SHELL,
        SHELL_TESTS_PATH => SHELL_TESTS,
        ANIMUS_TESTS_PATH => ANIMUS_TESTS,
        OPENAI_TESTS_PATH => OPENAI_TESTS,
        _ => panic!("unexpected path: {path}"),
    }
}

fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.agent_runtime_audit]
// r[verify lattice.repository_validation.agent_runtime_audit.scenario.complete]
// r[verify lattice.repository_validation.agent_runtime_audit.scenario.evidence]
#[test]
fn agent_runtime_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let review = core::validate_tranche(&rows(&report), SUBSYSTEM, &expected());
    let coverage = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let markers = core::validate_no_selected_markers(&[CORE, SHELL, SHELL_TESTS, ANIMUS_TESTS, OPENAI_TESTS], SELECTED);
    assert!(review.is_empty(), "review issues: {review:?}");
    assert!(coverage.is_empty(), "coverage issues: {coverage:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
    assert_eq!(
        report.rows.iter().filter(|row| SELECTED.contains(&row.requirement_id.as_str())).count(),
        SELECTED_COUNT
    );
    for row in report.rows.iter().filter(|row| SELECTED.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.agent_runtime_audit.scenario.fail_closed]
#[test]
fn agent_runtime_audit_drift_fails_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let mut review_rows = rows(&report);
    review_rows.retain(|row| row.requirement_id != COMPAT_MISSING);
    let identity = review_rows.iter_mut().find(|row| row.requirement_id == FABRIC_IDENTITY).expect("identity row");
    identity.classification = EXACT.to_string();
    identity.rationale_code = "invalid.rationale".to_string();
    identity.evidence_paths = vec!["../unsafe".to_string()];
    let review_issues = core::validate_tranche(&review_rows, SUBSYSTEM, &expected());
    let review_codes = codes(&review_issues);
    let marker = format!("// r[verify {COMPAT_MISSING}]");
    let marker_issues = core::validate_no_selected_markers(&[marker.as_str()], &[COMPAT_MISSING, COMPAT_MISSING]);
    let marker_codes = codes(&marker_issues);
    let baseline = coverage(BASELINE);
    let mut current = coverage(CURRENT);
    current.missing.retain(|id| id != UNRELATED);
    current.missing.push(ADDED.to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let coverage_issues = core::validate_unchanged_coverage(&baseline, &current);
    let coverage_codes = codes(&coverage_issues);
    for code in [
        "review.missing",
        "review.classification",
        "review.rationale",
        "review.evidence_paths",
    ] {
        assert!(review_codes.contains(code));
    }
    assert!(marker_codes.contains("marker.promoted"));
    assert!(marker_codes.contains("marker.duplicate_selected"));
    for code in [
        "coverage.removed",
        "coverage.added",
        "coverage.current_duplicate",
        "coverage.current_unsorted",
        "coverage.current_dangling",
        "coverage.current_valid",
    ] {
        assert!(coverage_codes.contains(code));
    }
}
