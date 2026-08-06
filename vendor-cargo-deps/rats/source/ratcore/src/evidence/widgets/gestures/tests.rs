use super::*;
use crate::interaction_replay::ReplayIssueCode;
use crate::interaction_replay::ReplayReport;
use crate::interaction_replay::run_replay;

const EXPECTED_EVIDENCE_ROWS: usize = 2;
const EXPECTED_ARTIFACTS: usize = 4;
const POSITIVE_FIXTURE_ID: &str = "full-mouse.positive";
const NEGATIVE_FIXTURE_ID: &str = "full-mouse.negative";
const TUI_RENDERER: &str = "subwayrat-tui";
const DOM_RENDERER: &str = "meteorite-dom";
const STATE_ACCEPTED_ROUTES: &str = "accepted_routes";
const POSITIVE_ACCEPTED_ROUTES: &str = "10";
const NEGATIVE_ACCEPTED_ROUTES: &str = "0";
const POSITIVE_STATE_HASH: &str = "d8c4a7a4cd5a61ef94ec11ad45d234f738e53ca94d36003bb1c934c3071a965d";
const NEGATIVE_STATE_HASH: &str = "18e427eb6485e0b2fc5c5c6f4e30d47db539abda6f0175e2a5c6e61f4343b934";
const POSITIVE_TUI_ARTIFACT: &str =
    include_str!("../../../../../docs/generated/replay-snapshots/full-mouse-positive.subwayrat.txt");
const POSITIVE_DOM_ARTIFACT: &str =
    include_str!("../../../../../docs/generated/replay-snapshots/full-mouse-positive.meteorite.html");
const NEGATIVE_TUI_ARTIFACT: &str =
    include_str!("../../../../../docs/generated/replay-snapshots/full-mouse-negative.subwayrat.txt");
const NEGATIVE_DOM_ARTIFACT: &str =
    include_str!("../../../../../docs/generated/replay-snapshots/full-mouse-negative.meteorite.html");

#[test]
fn full_mouse_positive_fixture_routes_representative_surfaces() {
    let mut target = pointer_mouse_replay_target();
    let script = pointer_mouse_positive_replay_script();

    let report = run_replay(&script, &mut target);

    assert!(report.is_success(), "issues: {:?}", report.issues);
    assert_eq!(report.state.get(STATE_ACCEPTED_ROUTES), Some(&POSITIVE_ACCEPTED_ROUTES.to_string()));
    assert_eq!(report.issues.len(), 0);
}

#[test]
fn full_mouse_negative_fixture_fails_closed_without_state_mutation() {
    let mut target = pointer_mouse_replay_target();
    let script = pointer_mouse_negative_replay_script();

    let report = run_replay(&script, &mut target);

    assert!(!report.is_success());
    assert_eq!(report.state.get(STATE_ACCEPTED_ROUTES), Some(&NEGATIVE_ACCEPTED_ROUTES.to_string()));
    assert_issue(&report, ReplayIssueCode::DisabledAction);
    assert_issue(&report, ReplayIssueCode::MissingDragStart);
    assert_issue(&report, ReplayIssueCode::StaleCapture);
    assert_issue(&report, ReplayIssueCode::UnsupportedEvent);
    assert_issue(&report, ReplayIssueCode::OutOfBoundsMouse);
    assert_issue(&report, ReplayIssueCode::ImpossibleGeometry);
}

#[test]
fn full_mouse_evidence_rows_bind_state_and_diagnostics() {
    let evidence = pointer_mouse_replay_proof();

    assert_eq!(evidence.rows.len(), EXPECTED_EVIDENCE_ROWS);
    assert!(evidence.rows.iter().any(|row| {
        row.fixture_id == POSITIVE_FIXTURE_ID
            && row.diagnostics == "-"
            && row.state_hash == POSITIVE_STATE_HASH
            && row.route_summary_hash == POSITIVE_STATE_HASH
    }));
    assert!(evidence.rows.iter().any(|row| {
        row.fixture_id == NEGATIVE_FIXTURE_ID
            && row.state_hash == NEGATIVE_STATE_HASH
            && row.route_summary_hash == NEGATIVE_STATE_HASH
            && row.diagnostics.contains("MissingDragStart")
            && row.diagnostics.contains("UnsupportedEvent")
    }));
}

#[test]
fn full_mouse_renderer_artifacts_cover_dom_and_tui() {
    let artifacts = pointer_mouse_renderer_artifacts();

    assert_eq!(artifacts.len(), EXPECTED_ARTIFACTS);
    assert_artifact_bytes(&artifacts, POSITIVE_FIXTURE_ID, TUI_RENDERER, POSITIVE_TUI_ARTIFACT.as_bytes());
    assert_artifact_bytes(&artifacts, POSITIVE_FIXTURE_ID, DOM_RENDERER, POSITIVE_DOM_ARTIFACT.as_bytes());
    assert_artifact_bytes(&artifacts, NEGATIVE_FIXTURE_ID, TUI_RENDERER, NEGATIVE_TUI_ARTIFACT.as_bytes());
    assert_artifact_bytes(&artifacts, NEGATIVE_FIXTURE_ID, DOM_RENDERER, NEGATIVE_DOM_ARTIFACT.as_bytes());
}

fn assert_issue(report: &ReplayReport, code: ReplayIssueCode) {
    assert!(report.issues.iter().any(|issue| issue.code == code));
}

fn assert_artifact_bytes(
    artifacts: &[PointerMouseRendererArtifact],
    fixture_id: &str,
    renderer: &str,
    expected: &[u8],
) {
    let artifact = artifacts
        .iter()
        .find(|artifact| artifact.fixture_id == fixture_id && artifact.renderer == renderer)
        .expect("full mouse artifact exists");
    assert_eq!(artifact.bytes, expected);
    assert!(!artifact.blake3.is_empty());
}
