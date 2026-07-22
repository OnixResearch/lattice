#[path = "traceability_review/core.rs"]
mod core;

use core::CoverageSnapshot;
use core::ExpectedDecision;
use core::ReviewRow;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-agent-runtime-promotion-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-workflow-validation-audit-current.json");
const REVIEWED: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed.json");
const WORKFLOW: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-workflow-validation.txt");
const N8N: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-n8n-json.txt");
const REPLAY: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-replay.txt");
const CREDENTIALS: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-credentials.txt");
const CONTROL_FLOW: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-control-flow.txt");
const NODE_KIND: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-node-kind-provider.txt");
const RUNNER_CORE: &str = include_str!("fixtures/traceability-workflow-validation-audit-reviewed-runner-core.txt");

const WORKFLOW_PATH: &str = "tests/test_workflow_validation.rs";
const N8N_PATH: &str = "tests/test_n8n_json.rs";
const REPLAY_PATH: &str = "tests/test_replay.rs";
const CREDENTIALS_PATH: &str = "tests/test_credentials.rs";
const CONTROL_FLOW_PATH: &str = "tests/test_control_flow.rs";
const NODE_KIND_PATH: &str = "tests/test_node_kind_provider.rs";
const RUNNER_CORE_PATH: &str = "tests/test_runner_core.rs";
const SUBSYSTEM: &str = "workflow-validation";
const EXACT: &str = "exact_maintained_evidence";
const PARTIAL: &str = "partial_evidence_only";
const ABSENT: &str = "no_exact_evidence_identified";
const SELECTED_COUNT: usize = 29;
const UNRELATED: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

const EXPECTED: &[ExpectedDecision] = &[
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.credentials.providers.config"),
        classification: PARTIAL,
        rationale_code: "gap.pre-effect-blocking-unasserted",
        evidence_paths: &[CREDENTIALS_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.credentials.providers.field"),
        classification: PARTIAL,
        rationale_code: "gap.fatal-severity-unasserted",
        evidence_paths: &[CREDENTIALS_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.credentials.providers.key"),
        classification: ABSENT,
        rationale_code: "gap.malformed-provider-key-evidence-absent",
        evidence_paths: &[],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.diagnostics.core.empty_command"),
        classification: EXACT,
        rationale_code: "assertions.empty-command-fatal-code",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.diagnostics.core.missing_config"),
        classification: EXACT,
        rationale_code: "assertions.missing-config-fatal-code",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.diagnostics.core.valid"),
        classification: EXACT,
        rationale_code: "assertions.valid-workflow-no-fatal",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.diagnostics.negative.invalid_edge"),
        classification: EXACT,
        rationale_code: "assertions.invalid-edge-stable-code",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.generic_runner_admission.effects"),
        classification: ABSENT,
        rationale_code: "gap.effect-resource-admission-evidence-absent",
        evidence_paths: &[],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.generic_runner_admission.graph"),
        classification: PARTIAL,
        rationale_code: "gap.complete-graph-cases-and-zero-effects-unasserted",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.generic_runner_admission.terminal_state"),
        classification: PARTIAL,
        rationale_code: "gap.persisted-config-immutability-unasserted",
        evidence_paths: &[RUNNER_CORE_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_json.export_limits.node"),
        classification: EXACT,
        rationale_code: "assertions.unsupported-export-refused",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.connection"),
        classification: EXACT,
        rationale_code: "assertions.malformed-connection-refused",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.expression"),
        classification: EXACT,
        rationale_code: "assertions.unsupported-expression-refused",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_json.unsupported.node_kind"),
        classification: EXACT,
        rationale_code: "assertions.unsupported-node-refused",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.malformed_connection"),
        classification: EXACT,
        rationale_code: "assertions.parity-malformed-golden-diagnostic",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.plaintext_credential"),
        classification: EXACT,
        rationale_code: "assertions.parity-plaintext-safe-diagnostic",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.unsupported_expression"),
        classification: EXACT,
        rationale_code: "assertions.parity-expression-golden-diagnostic",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.n8n_parity_diagnostics.unsupported_node"),
        classification: EXACT,
        rationale_code: "assertions.parity-node-golden-diagnostic",
        evidence_paths: &[N8N_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.node_kind_provider"),
        classification: PARTIAL,
        rationale_code: "gap.all-builtins-and-runtime-provider-contract-unasserted",
        evidence_paths: &[NODE_KIND_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.replay_records.artifact"),
        classification: EXACT,
        rationale_code: "assertions.missing-artifact-recoverable-placeholder",
        evidence_paths: &[REPLAY_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.replay_records.order"),
        classification: EXACT,
        rationale_code: "assertions.duplicate-out-of-order-fatal-codes",
        evidence_paths: &[REPLAY_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.replay_records.terminal"),
        classification: EXACT,
        rationale_code: "assertions.terminal-replay-mutation-rejected",
        evidence_paths: &[REPLAY_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.run_gate.block"),
        classification: EXACT,
        rationale_code: "assertions.fatal-blocks-tui-and-headless-effects",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.run_gate.warning"),
        classification: EXACT,
        rationale_code: "assertions.warning-allows-run-gate",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.subworkflows.mapping"),
        classification: ABSENT,
        rationale_code: "gap.subworkflow-mapping-evidence-absent",
        evidence_paths: &[],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.subworkflows.missing"),
        classification: EXACT,
        rationale_code: "assertions.missing-subworkflow-rejected",
        evidence_paths: &[CONTROL_FLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.subworkflows.recursive"),
        classification: EXACT,
        rationale_code: "assertions.recursive-subworkflow-rejected",
        evidence_paths: &[CONTROL_FLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.tui.panel.empty"),
        classification: PARTIAL,
        rationale_code: "gap.ready-message-unasserted",
        evidence_paths: &[WORKFLOW_PATH],
    },
    ExpectedDecision {
        requirement_id: concat!("n8n", "clone.workflow_validation.tui.panel.open"),
        classification: PARTIAL,
        rationale_code: "gap.location-and-actionable-text-unasserted",
        evidence_paths: &[WORKFLOW_PATH],
    },
];

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
        WORKFLOW_PATH => WORKFLOW,
        N8N_PATH => N8N,
        REPLAY_PATH => REPLAY,
        CREDENTIALS_PATH => CREDENTIALS,
        CONTROL_FLOW_PATH => CONTROL_FLOW,
        NODE_KIND_PATH => NODE_KIND,
        RUNNER_CORE_PATH => RUNNER_CORE,
        _ => panic!("unexpected evidence path: {path}"),
    }
}

fn selected() -> Vec<&'static str> {
    EXPECTED.iter().map(|decision| decision.requirement_id).collect()
}
fn codes(issues: &[core::ReviewIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.workflow_validation_audit]
// r[verify lattice.repository_validation.workflow_validation_audit.scenario.complete]
// r[verify lattice.repository_validation.workflow_validation_audit.scenario.evidence]
#[test]
fn workflow_validation_audit_is_complete_byte_bound_and_unpromoted() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let selected = selected();
    let review = core::validate_tranche(&rows(&report), SUBSYSTEM, EXPECTED);
    let unchanged = core::validate_unchanged_coverage(&coverage(BASELINE), &coverage(CURRENT));
    let markers = core::validate_no_selected_markers(
        &[WORKFLOW, N8N, REPLAY, CREDENTIALS, CONTROL_FLOW, NODE_KIND, RUNNER_CORE],
        &selected,
    );
    assert!(review.is_empty(), "review issues: {review:?}");
    assert!(unchanged.is_empty(), "coverage issues: {unchanged:?}");
    assert!(markers.is_empty(), "marker issues: {markers:?}");
    assert_eq!(selected.len(), SELECTED_COUNT);
    for row in report.rows.iter().filter(|row| selected.contains(&row.requirement_id.as_str())) {
        for evidence in &row.evidence {
            assert_eq!(blake3::hash(source(&evidence.path).as_bytes()).to_hex().to_string(), evidence.blake3);
        }
    }
}

// r[verify lattice.repository_validation.workflow_validation_audit.scenario.fail_closed]
#[test]
fn workflow_validation_audit_drift_fails_closed() {
    let report: InventoryReport = serde_json::from_str(REVIEWED).expect("inventory decodes");
    let mut review_rows = rows(&report);
    review_rows.retain(|row| row.requirement_id != EXPECTED[0].requirement_id);
    let row = review_rows
        .iter_mut()
        .find(|row| row.requirement_id == EXPECTED[1].requirement_id)
        .expect("review row");
    row.classification = EXACT.to_string();
    row.rationale_code = "invalid.rationale".to_string();
    row.evidence_paths = vec!["../unsafe".to_string()];
    let review_issues = core::validate_tranche(&review_rows, SUBSYSTEM, EXPECTED);
    let review_codes = codes(&review_issues);
    for code in [
        "review.missing",
        "review.classification",
        "review.rationale",
        "review.evidence_paths",
    ] {
        assert!(review_codes.contains(code));
    }

    let marker = format!("// r[verify {}]", EXPECTED[0].requirement_id);
    let marker_issues = core::validate_no_selected_markers(&[marker.as_str()], &[
        EXPECTED[0].requirement_id,
        EXPECTED[0].requirement_id,
    ]);
    let marker_codes = codes(&marker_issues);
    assert!(marker_codes.contains("marker.promoted"));
    assert!(marker_codes.contains("marker.duplicate_selected"));

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
