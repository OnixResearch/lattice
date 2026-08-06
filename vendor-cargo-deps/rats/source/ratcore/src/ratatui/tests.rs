const FEATURE_BUFFER: &str = "ratatui.buffer";
const FEATURE_BUILTIN_DATA_VIZ: &str = "ratatui.builtin-data-viz";
const FEATURE_BUILTIN_CANVAS: &str = "ratatui.builtin-canvas";
const FEATURE_BUILTIN_CALENDAR: &str = "ratatui.builtin-calendar";
const FEATURE_BUILTIN_DECORATION: &str = "ratatui.builtin-decoration";
const FEATURE_BUILTIN_PROGRESS_SCROLL: &str = "ratatui.builtin-progress-scroll";
const FEATURE_BUILTIN_COLLECTIONS: &str = "ratatui.builtin-collections";
const FEATURE_BUILTIN_BRANDING: &str = "ratatui.builtin-branding";
const FEATURE_CHART: &str = "ratatui.chart";
const FAMILY_WIDGET: &str = "Chart and Gauge";
const OWNER_SHOWCASE: &str = "rat-showcase";
const STATUS_PASS_THROUGH: &str = "pass-through";
const STATUS_SHOWCASE_ONLY: &str = "showcase-only";
const STATUS_NEEDS_EVIDENCE: &str = "needs-evidence";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_UNKNOWN: &str = "ready";
const NEXT_SHOWCASE: &str = "keep demo fixture fresh";
const EVIDENCE_SHOWCASE: &str = "subwayrat/crates/rat-showcase/src/main.rs";
const REASON_UNSUPPORTED: &str = "no accepted owner";
const BUILTIN_ROW_COUNT: usize = 7;
const BASELINE_ROW_COUNT: usize = 17;
const MALFORMED_ROW_COUNT: usize = 5;
const MIN_MALFORMED_ISSUES: usize = 6;
const BUILTIN_FEATURE_IDS: [&str; BUILTIN_ROW_COUNT] = [
    FEATURE_BUILTIN_DATA_VIZ,
    FEATURE_BUILTIN_CANVAS,
    FEATURE_BUILTIN_CALENDAR,
    FEATURE_BUILTIN_DECORATION,
    FEATURE_BUILTIN_PROGRESS_SCROLL,
    FEATURE_BUILTIN_COLLECTIONS,
    FEATURE_BUILTIN_BRANDING,
];

#[test]
fn baseline_coverage_inventory_is_valid() {
    let rows = crate::ratatui_integration::baseline_ratatui_coverage_rows();

    let outcome = crate::ratatui_integration::validate_ratatui_coverage(&rows);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.rows.len(), BASELINE_ROW_COUNT);
    assert!(outcome.rows.iter().any(|row| row.feature_id == FEATURE_BUFFER));
}

#[test]
fn baseline_coverage_lists_built_in_widget_families() {
    let rows = crate::ratatui_integration::baseline_ratatui_coverage_rows();
    let outcome = crate::ratatui_integration::validate_ratatui_coverage(&rows);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    for feature_id in BUILTIN_FEATURE_IDS {
        let row = outcome
            .rows
            .iter()
            .find(|row| row.feature_id == feature_id)
            .expect("built-in family row is present");
        assert!(!row.evidence.is_empty());
        assert!(row.status == STATUS_SHOWCASE_ONLY || row.status == STATUS_PASS_THROUGH);
    }
}

#[test]
fn complete_showcase_rows_pass_with_evidence() {
    let rows = vec![
        crate::ratatui_integration::CoverageRow::new(
            FEATURE_CHART,
            FAMILY_WIDGET,
            OWNER_SHOWCASE,
            STATUS_SHOWCASE_ONLY,
            NEXT_SHOWCASE,
        )
        .with_evidence(EVIDENCE_SHOWCASE),
    ];

    let outcome = crate::ratatui_integration::validate_ratatui_coverage(&rows);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.rows[0].feature_id, FEATURE_CHART);
}

#[test]
fn malformed_inventory_rows_fail_with_deterministic_diagnostics() {
    let duplicate = crate::ratatui_integration::CoverageRow::new(
        FEATURE_CHART,
        FAMILY_WIDGET,
        OWNER_SHOWCASE,
        STATUS_SHOWCASE_ONLY,
        NEXT_SHOWCASE,
    );
    let needs_evidence_without_next_action = crate::ratatui_integration::CoverageRow::new(
        "ratatui.pending",
        FAMILY_WIDGET,
        OWNER_SHOWCASE,
        STATUS_NEEDS_EVIDENCE,
        "",
    );
    let unsupported_without_reason = crate::ratatui_integration::CoverageRow::new(
        "ratatui.unsupported",
        FAMILY_WIDGET,
        OWNER_SHOWCASE,
        STATUS_UNSUPPORTED,
        NEXT_SHOWCASE,
    );
    let unknown = crate::ratatui_integration::CoverageRow::new(
        "ratatui.unknown",
        FAMILY_WIDGET,
        OWNER_SHOWCASE,
        STATUS_UNKNOWN,
        NEXT_SHOWCASE,
    );

    let outcome = crate::ratatui_integration::validate_ratatui_coverage(&[
        duplicate.clone(),
        duplicate,
        needs_evidence_without_next_action,
        unsupported_without_reason,
        unknown,
    ]);

    assert!(!outcome.is_valid());
    assert_eq!(outcome.rows.len(), MALFORMED_ROW_COUNT);
    assert!(outcome.issues.len() >= MIN_MALFORMED_ISSUES);
    assert_issue(&outcome.issues, crate::ratatui_integration::CoverageIssueCode::DuplicateFeatureId);
    assert_issue(&outcome.issues, crate::ratatui_integration::CoverageIssueCode::MissingEvidence);
    assert_issue(&outcome.issues, crate::ratatui_integration::CoverageIssueCode::MissingNextAction);
    assert_issue(&outcome.issues, crate::ratatui_integration::CoverageIssueCode::MissingReason);
    assert_issue(&outcome.issues, crate::ratatui_integration::CoverageIssueCode::UnknownStatus);
}

#[test]
fn required_fields_are_checked_before_status_policy() {
    let row = crate::ratatui_integration::CoverageRow::new("", "", "", STATUS_SHOWCASE_ONLY, NEXT_SHOWCASE)
        .with_evidence(EVIDENCE_SHOWCASE);

    let outcome = crate::ratatui_integration::validate_ratatui_coverage(&[row]);

    assert!(!outcome.is_valid());
    assert_issue(&outcome.issues, crate::ratatui_integration::CoverageIssueCode::MissingRequiredField);
}

#[test]
fn unsupported_rows_pass_with_reason_without_evidence() {
    let row = crate::ratatui_integration::CoverageRow::new(
        "ratatui.experimental",
        FAMILY_WIDGET,
        OWNER_SHOWCASE,
        STATUS_UNSUPPORTED,
        NEXT_SHOWCASE,
    )
    .with_reason(REASON_UNSUPPORTED);

    let outcome = crate::ratatui_integration::validate_ratatui_coverage(&[row]);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
}

fn assert_issue(
    issues: &[crate::ratatui_integration::CoverageIssue],
    code: crate::ratatui_integration::CoverageIssueCode,
) {
    assert!(issues.iter().any(|issue| issue.code == code));
}
