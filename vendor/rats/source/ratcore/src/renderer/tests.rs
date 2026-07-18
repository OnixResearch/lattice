const FEATURE_SHARED: &str = "fixtures.shared";
const FEATURE_TUI: &str = "chrome.overlay";
const FAMILY_FIXTURES: &str = "fixtures";
const FAMILY_CHROME: &str = "chrome";
const OWNER_RATCORE: &str = "ratcore";
const NEXT_DONE: &str = "maintain parity evidence";
const NEXT_METEORITE: &str = "build Meteorite adapter";
const EVIDENCE_DOCS: &str = "docs/showcase-fixture-coverage.md";
const REASON_TERMINAL: &str = "terminal-only chrome primitive";
const SHARED_FEATURE_ACTIONS: &str = "shared.actions";
const SHARED_FEATURE_CHROME: &str = "chrome.menu.navigation";
const SHARED_FEATURE_WORKSPACE: &str = "workspace.shell.core";
const SHARED_STATUS_EXTRACTED: &str = "extracted";
const SHARED_STATUS_RENDERER_OWNED: &str = "renderer-owned";
const SHARED_STATUS_NEEDS_EVIDENCE: &str = "needs-evidence";
const SHARED_STATUS_UNKNOWN: &str = "ready";
const OWNER_RAT_CHROME: &str = "renderer-owned: rat-chrome";
const FEATURE_ANIMATION_FADE: &str = "animation.fade";
const FEATURE_EMBED_CHROME: &str = "meteorite.embedded.chrome-shell";
const FAMILY_ANIMATION: &str = "animation";
const FAMILY_EMBEDDING: &str = "embedding";
const OWNER_ANIMATION: &str = "ratcore::animation_parity";
const OWNER_RATZILLA_EMBED: &str = "ratcore::subwayrat_web_renderer; rat-ratzilla shell";
const EMBED_COLUMN_SUBWAYRAT_TERMINAL: &str = "subwayrat-terminal";
const EMBED_COLUMN_SUBWAYRAT_RATZILLA: &str = "subwayrat-ratzilla";
const EMBED_COLUMN_METEORITE_RATZILLA: &str = "meteorite-embedded-ratzilla";
const EMBED_COLUMN_METEORITE_NATIVE_DOM: &str = "meteorite-native-dom";
const EVIDENCE_TACHYONFX: &str = "rat-showcase effects adapter";
const EVIDENCE_CSS: &str = "met-showcase animation projection";
const EVIDENCE_EMBEDDED: &str = "embedded Ratzilla mount smoke";
const EVIDENCE_NATIVE_DOM: &str = "met-showcase native DOM projection";
const DIAGNOSTICS_EMBEDDED: &str = "missing host, duplicate mount, unsupported fixture diagnostics";
const REASON_NATIVE_PENDING: &str =
    "native DOM component projection remains separate from terminal embedding";
const NEXT_ANIMATION: &str = "keep logical animation fixtures fresh";
const NEXT_NATIVE_DOM: &str = "build native Dioxus projection if parity is required";
const SUBWAYRAT_COMPLETE: &str = "subwayrat complete";
const METEORITE_COMPLETE: &str = "meteorite complete";
const METEORITE_PARTIAL: &str = "meteorite partial";
const SHARED_TEST_EVIDENCE: &str = "positive and negative ratcore tests";
const NEXT_EXTRACT_WORKSPACE: &str = "extract workspace shell modules";
const MALFORMED_ROW_COUNT: usize = 3;
const MIN_MALFORMED_ISSUES: usize = 5;
const SHARED_VALID_ROW_COUNT: usize = 2;
const EMBEDDED_VALID_ROW_COUNT: usize = 1;
const SHARED_MALFORMED_ROW_COUNT: usize = 5;
const MIN_SHARED_MALFORMED_ISSUES: usize = 6;

#[test]
fn complete_shared_rows_pass_with_evidence() {
    let rows = vec![
        crate::renderer_parity::Row::new(FEATURE_SHARED, FAMILY_FIXTURES, OWNER_RATCORE, NEXT_DONE)
            .with_renderer(
                crate::renderer_parity::Support::new("subwayrat", "complete")
                    .with_evidence(EVIDENCE_DOCS),
            )
            .with_renderer(
                crate::renderer_parity::Support::new("meteorite", "complete")
                    .with_evidence(EVIDENCE_DOCS),
            ),
    ];

    let outcome = crate::renderer_parity::validate(&rows);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.rows[0].feature_id, FEATURE_SHARED);
}

#[test]
fn malformed_rows_fail_with_deterministic_diagnostics() {
    let duplicate = crate::renderer_parity::Row::new(FEATURE_TUI, FAMILY_CHROME, OWNER_RATCORE, "")
        .with_renderer(crate::renderer_parity::Support::new(
            "subwayrat",
            "complete",
        ))
        .with_renderer(crate::renderer_parity::Support::new(
            "meteorite",
            "unsupported",
        ));
    let unknown =
        crate::renderer_parity::Row::new("unknown.status", "chrome", "ratcore", NEXT_METEORITE)
            .with_renderer(crate::renderer_parity::Support::new("subwayrat", "ready"))
            .with_renderer(
                crate::renderer_parity::Support::new("meteorite", "divergent")
                    .with_reason(REASON_TERMINAL),
            );

    let outcome = crate::renderer_parity::validate(&[duplicate.clone(), duplicate, unknown]);

    assert!(!outcome.is_valid());
    assert_eq!(outcome.rows.len(), MALFORMED_ROW_COUNT);
    assert!(outcome.issues.len() >= MIN_MALFORMED_ISSUES);
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::DuplicateFeatureId,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingEvidence,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingReason,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingNextAction,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::UnknownStatus,
    );
}

#[test]
fn animation_rows_accept_logical_parity_with_renderer_specific_evidence() {
    let row = crate::renderer_parity::Row::new(
        FEATURE_ANIMATION_FADE,
        FAMILY_ANIMATION,
        OWNER_ANIMATION,
        NEXT_ANIMATION,
    )
    .with_renderer(
        crate::renderer_parity::Support::new("subwayrat", "complete")
            .with_evidence(EVIDENCE_TACHYONFX),
    )
    .with_renderer(
        crate::renderer_parity::Support::new("meteorite", "complete").with_evidence(EVIDENCE_CSS),
    );

    let outcome = crate::renderer_parity::validate(&[row]);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.rows[0].feature_id, FEATURE_ANIMATION_FADE);
}

#[test]
fn animation_rows_reject_missing_evidence_and_gap_actions() {
    let row = crate::renderer_parity::Row::new(
        "animation.missing-evidence",
        FAMILY_ANIMATION,
        OWNER_ANIMATION,
        "",
    )
    .with_renderer(crate::renderer_parity::Support::new(
        "subwayrat",
        "complete",
    ))
    .with_renderer(crate::renderer_parity::Support::new("meteorite", "partial"));

    let outcome = crate::renderer_parity::validate(&[row]);

    assert!(!outcome.is_valid());
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingEvidence,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingNextAction,
    );
}

#[test]
fn embedded_ratzilla_rows_keep_embedded_and_native_dom_columns_separate() {
    let row = crate::renderer_parity::EmbeddedRatzillaSurfaceRow::new(
        FEATURE_EMBED_CHROME,
        FAMILY_EMBEDDING,
        OWNER_RATZILLA_EMBED,
        NEXT_NATIVE_DOM,
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_SUBWAYRAT_TERMINAL, "complete")
            .with_evidence(EVIDENCE_TACHYONFX),
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_SUBWAYRAT_RATZILLA, "complete")
            .with_evidence(EVIDENCE_EMBEDDED),
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_METEORITE_RATZILLA, "complete")
            .with_evidence(EVIDENCE_EMBEDDED),
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_METEORITE_NATIVE_DOM, "partial")
            .with_evidence(EVIDENCE_NATIVE_DOM)
            .with_reason(REASON_NATIVE_PENDING),
    )
    .with_diagnostics_coverage(DIAGNOSTICS_EMBEDDED);

    let outcome = crate::renderer_parity::validate_embedded_ratzilla_surface_rows(&[row]);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.rows.len(), EMBEDDED_VALID_ROW_COUNT);
}

#[test]
fn embedded_ratzilla_rows_reject_embedded_evidence_as_native_dom_completion() {
    let row = crate::renderer_parity::EmbeddedRatzillaSurfaceRow::new(
        FEATURE_EMBED_CHROME,
        FAMILY_EMBEDDING,
        OWNER_RATZILLA_EMBED,
        "",
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_SUBWAYRAT_TERMINAL, "complete")
            .with_evidence(EVIDENCE_TACHYONFX),
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_SUBWAYRAT_RATZILLA, "complete")
            .with_evidence(EVIDENCE_EMBEDDED),
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_METEORITE_RATZILLA, "complete")
            .with_evidence(EVIDENCE_EMBEDDED),
    )
    .with_renderer(
        crate::renderer_parity::Support::new(EMBED_COLUMN_METEORITE_NATIVE_DOM, "complete")
            .with_evidence(EVIDENCE_EMBEDDED),
    );

    let outcome = crate::renderer_parity::validate_embedded_ratzilla_surface_rows(&[row]);

    assert!(!outcome.is_valid());
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingDiagnosticsCoverage,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::InvalidStatusRelationship,
    );
}

#[test]
fn contract_candidates_accept_extracted_and_renderer_owned_decisions() {
    let rows = vec![
        crate::renderer_parity::ContractCandidate::new(
            SHARED_FEATURE_ACTIONS,
            FAMILY_FIXTURES,
            OWNER_RATCORE,
            SHARED_STATUS_EXTRACTED,
        )
        .with_subwayrat_status(SUBWAYRAT_COMPLETE)
        .with_meteorite_status(METEORITE_COMPLETE)
        .with_evidence(EVIDENCE_DOCS)
        .with_test(SHARED_TEST_EVIDENCE),
        crate::renderer_parity::ContractCandidate::new(
            SHARED_FEATURE_CHROME,
            FAMILY_CHROME,
            OWNER_RAT_CHROME,
            SHARED_STATUS_RENDERER_OWNED,
        )
        .with_subwayrat_status(SUBWAYRAT_COMPLETE)
        .with_meteorite_status(METEORITE_PARTIAL)
        .with_reason(REASON_TERMINAL),
    ];

    let outcome = crate::renderer_parity::validate_contract_candidates(&rows);

    assert!(outcome.is_valid(), "issues: {:?}", outcome.issues);
    assert_eq!(outcome.rows.len(), SHARED_VALID_ROW_COUNT);
    assert!(
        outcome
            .rows
            .iter()
            .any(|row| row.feature_id == SHARED_FEATURE_ACTIONS)
    );
}

#[test]
fn contract_candidates_reject_incomplete_inventory_rows() {
    let duplicate = crate::renderer_parity::ContractCandidate::new(
        SHARED_FEATURE_ACTIONS,
        FAMILY_FIXTURES,
        OWNER_RATCORE,
        SHARED_STATUS_EXTRACTED,
    )
    .with_subwayrat_status(SUBWAYRAT_COMPLETE)
    .with_meteorite_status(METEORITE_COMPLETE)
    .with_evidence(EVIDENCE_DOCS);
    let missing_owner = crate::renderer_parity::ContractCandidate::new(
        SHARED_FEATURE_WORKSPACE,
        FAMILY_CHROME,
        "",
        SHARED_STATUS_NEEDS_EVIDENCE,
    )
    .with_subwayrat_status(SUBWAYRAT_COMPLETE)
    .with_meteorite_status(METEORITE_PARTIAL);
    let renderer_owned_without_reason = crate::renderer_parity::ContractCandidate::new(
        SHARED_FEATURE_CHROME,
        FAMILY_CHROME,
        OWNER_RAT_CHROME,
        SHARED_STATUS_RENDERER_OWNED,
    )
    .with_subwayrat_status(SUBWAYRAT_COMPLETE)
    .with_meteorite_status(METEORITE_PARTIAL);
    let unknown = crate::renderer_parity::ContractCandidate::new(
        "shared.unknown",
        FAMILY_CHROME,
        OWNER_RATCORE,
        SHARED_STATUS_UNKNOWN,
    )
    .with_subwayrat_status(SUBWAYRAT_COMPLETE)
    .with_meteorite_status(METEORITE_COMPLETE);

    let outcome = crate::renderer_parity::validate_contract_candidates(&[
        duplicate.clone(),
        duplicate,
        missing_owner,
        renderer_owned_without_reason,
        unknown,
    ]);

    assert!(!outcome.is_valid());
    assert_eq!(outcome.rows.len(), SHARED_MALFORMED_ROW_COUNT);
    assert!(outcome.issues.len() >= MIN_SHARED_MALFORMED_ISSUES);
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::DuplicateFeatureId,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingRequiredField,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingTests,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingReason,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingNextAction,
    );
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::UnknownStatus,
    );
}

#[test]
fn missing_label_is_reported() {
    let row =
        crate::renderer_parity::Row::new(FEATURE_TUI, FAMILY_CHROME, OWNER_RATCORE, NEXT_METEORITE)
            .with_renderer(
                crate::renderer_parity::Support::new("subwayrat", "complete")
                    .with_evidence(EVIDENCE_DOCS),
            );

    let outcome = crate::renderer_parity::validate(&[row]);

    assert!(!outcome.is_valid());
    assert_eq!(outcome.issues[0].subject, "meteorite");
    assert_issue(
        &outcome.issues,
        crate::renderer_parity::IssueCode::MissingRendererLabel,
    );
}

fn assert_issue(issues: &[crate::renderer_parity::Issue], code: crate::renderer_parity::IssueCode) {
    assert!(issues.iter().any(|issue| issue.code == code));
}
