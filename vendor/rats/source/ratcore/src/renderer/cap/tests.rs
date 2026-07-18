use super::*;

const CAPABILITY_CLICK: &str = "mouse.click.chrome";
const CAPABILITY_WHEEL: &str = "mouse.wheel.tree";
const CAPABILITY_DRAG: &str = "mouse.drag.docking";
const EVENT_CLICK: &str = "click";
const EVENT_WHEEL: &str = "wheel";
const EVENT_DRAG: &str = "drag";
const SURFACE_CHROME: &str = "chrome";
const SURFACE_TREE: &str = "tree";
const SURFACE_DOCKING: &str = "docking";
const OWNER_POINTER: &str = "ratcore::pointer_input_routing";
const DIAGNOSTICS: &str = "positive/negative replay diagnostics";
const NEXT_KEEP_FRESH: &str = "keep evidence fresh when adapters change";
const EVIDENCE_REPLAY: &str = "docs/interaction-replay-fixtures.md#full-mouse";
const REASON_WHEEL: &str = "backend wheel synthesis is unavailable in smoke harness";
const EXPECTED_VALID_ROWS: usize = 2;

#[test]
fn full_mouse_matrix_accepts_complete_and_partial_rows_with_evidence() {
    let rows = vec![
        complete_row(CAPABILITY_CLICK, EVENT_CLICK, SURFACE_CHROME),
        partial_wheel_row(),
    ];

    let report = validate_full_mouse_support_matrix(&rows);

    assert!(report.is_valid(), "issues: {:?}", report.issues);
    assert_eq!(report.rows.len(), EXPECTED_VALID_ROWS);
}

#[test]
fn full_mouse_matrix_rejects_missing_proof_and_unactionable_gaps() {
    let duplicate = FullMouseSupportRow::new(CAPABILITY_DRAG)
        .with_event_class(EVENT_DRAG)
        .with_surface_family(SURFACE_DOCKING)
        .with_shared_core_owner(OWNER_POINTER)
        .with_renderer(FullMouseRendererStatus::new(
            RENDERER_SUBWAYRAT_TERMINAL,
            STATUS_COMPLETE,
        ))
        .with_renderer(FullMouseRendererStatus::new(
            RENDERER_SUBWAYRAT_RATZILLA,
            STATUS_UNSUPPORTED,
        ))
        .with_renderer(FullMouseRendererStatus::new(
            RENDERER_METEORITE_DOM,
            "ready",
        ));

    let report = validate_full_mouse_support_matrix(&[duplicate.clone(), duplicate]);

    assert!(!report.is_valid());
    assert_issue(&report, FullMouseSupportIssueCode::DuplicateCapabilityId);
    assert_issue(&report, FullMouseSupportIssueCode::MissingEvidence);
    assert_issue(&report, FullMouseSupportIssueCode::MissingReason);
    assert_issue(&report, FullMouseSupportIssueCode::UnknownStatus);
    assert_issue(
        &report,
        FullMouseSupportIssueCode::MissingDiagnosticsCoverage,
    );
    assert_issue(&report, FullMouseSupportIssueCode::MissingNextAction);
}

#[test]
fn full_mouse_matrix_reports_missing_renderer_labels() {
    let row = FullMouseSupportRow::new(CAPABILITY_CLICK)
        .with_event_class(EVENT_CLICK)
        .with_surface_family(SURFACE_CHROME)
        .with_shared_core_owner(OWNER_POINTER)
        .with_diagnostics_coverage(DIAGNOSTICS)
        .with_next_action(NEXT_KEEP_FRESH)
        .with_renderer(
            FullMouseRendererStatus::new(RENDERER_SUBWAYRAT_TERMINAL, STATUS_COMPLETE)
                .with_evidence(EVIDENCE_REPLAY),
        );

    let report = validate_full_mouse_support_matrix(&[row]);

    assert!(!report.is_valid());
    assert_issue(&report, FullMouseSupportIssueCode::MissingRenderer);
}

fn complete_row(capability: &str, event_class: &str, surface: &str) -> FullMouseSupportRow {
    FullMouseSupportRow::new(capability)
        .with_event_class(event_class)
        .with_surface_family(surface)
        .with_shared_core_owner(OWNER_POINTER)
        .with_diagnostics_coverage(DIAGNOSTICS)
        .with_next_action(NEXT_KEEP_FRESH)
        .with_renderer(
            FullMouseRendererStatus::new(RENDERER_SUBWAYRAT_TERMINAL, STATUS_COMPLETE)
                .with_evidence(EVIDENCE_REPLAY),
        )
        .with_renderer(
            FullMouseRendererStatus::new(RENDERER_SUBWAYRAT_RATZILLA, STATUS_COMPLETE)
                .with_evidence(EVIDENCE_REPLAY),
        )
        .with_renderer(
            FullMouseRendererStatus::new(RENDERER_METEORITE_DOM, STATUS_COMPLETE)
                .with_evidence(EVIDENCE_REPLAY),
        )
}

fn partial_wheel_row() -> FullMouseSupportRow {
    FullMouseSupportRow::new(CAPABILITY_WHEEL)
        .with_event_class(EVENT_WHEEL)
        .with_surface_family(SURFACE_TREE)
        .with_shared_core_owner(OWNER_POINTER)
        .with_diagnostics_coverage(DIAGNOSTICS)
        .with_next_action(NEXT_KEEP_FRESH)
        .with_renderer(
            FullMouseRendererStatus::new(RENDERER_SUBWAYRAT_TERMINAL, STATUS_COMPLETE)
                .with_evidence(EVIDENCE_REPLAY),
        )
        .with_renderer(
            FullMouseRendererStatus::new(RENDERER_SUBWAYRAT_RATZILLA, STATUS_PARTIAL)
                .with_evidence(EVIDENCE_REPLAY)
                .with_reason(REASON_WHEEL),
        )
        .with_renderer(
            FullMouseRendererStatus::new(RENDERER_METEORITE_DOM, STATUS_COMPLETE)
                .with_evidence(EVIDENCE_REPLAY),
        )
}

fn assert_issue(report: &FullMouseSupportReport, code: FullMouseSupportIssueCode) {
    assert!(report.issues.iter().any(|issue| issue.code == code));
}
