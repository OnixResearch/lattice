use super::super::super::*;
use super::catalog::*;
use super::core::*;

const POSITIVE_FIXTURE_ID: &str = "full-mouse.positive";
const NEGATIVE_FIXTURE_ID: &str = "full-mouse.negative";
const FIXTURE_COMPONENT: &str = "full-mouse-support";
const POSITIVE_NAME: &str = "full mouse positive";
const NEGATIVE_NAME: &str = "full mouse negative";
const WAIT_BUDGET: usize = 1;
const HOVER_X: u16 = 4;
const HOVER_Y: u16 = 2;
const OUT_OF_BOUNDS_X: u16 = DEFAULT_REPLAY_WIDTH;
const OUT_OF_BOUNDS_Y: u16 = DEFAULT_REPLAY_HEIGHT;
const POSITIVE_ACCEPTED_ROUTES: usize = 10;
const NEGATIVE_ACCEPTED_ROUTES: usize = 0;

/// Full mouse replay proof row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerMouseReplayProofRow {
    /// Fixture ID.
    pub fixture_id: String,
    /// Scenario label.
    pub scenario: String,
    /// Final state BLAKE3 hash.
    pub state_hash: String,
    /// Route-summary BLAKE3 hash.
    pub route_summary_hash: String,
    /// Stable replay diagnostics observed for the fixture.
    pub diagnostics: String,
}

/// Full mouse replay proof summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerMouseReplayProof {
    /// Positive and negative full-mouse rows.
    pub rows: Vec<PointerMouseReplayProofRow>,
}

/// Creates the full mouse replay target.
#[must_use]
pub fn pointer_mouse_replay_target() -> PointerMouseReplayTarget {
    PointerMouseReplayTarget::new(baseline_actions(), disabled_actions())
}

/// Creates the valid full mouse replay script.
#[must_use]
pub fn pointer_mouse_positive_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(
        POSITIVE_FIXTURE_ID,
        POSITIVE_NAME,
        FIXTURE_COMPONENT,
    ))
    .with_event(command(ACTION_CLICK_CHROME))
    .with_event(command(ACTION_DOUBLE_COMMAND))
    .with_event(command(ACTION_CONTEXT_TABLE))
    .with_event(ReplayEvent::Mouse {
        target: TARGET_FULL_MOUSE.to_string(),
        x: HOVER_X,
        y: HOVER_Y,
    })
    .with_event(command(ACTION_WHEEL_TREE))
    .with_event(command(ACTION_DRAG_START_DOCKING))
    .with_event(command(ACTION_DRAG_UPDATE_DOCKING))
    .with_event(command(ACTION_DRAG_RELEASE_DOCKING))
    .with_event(command(ACTION_CANCEL_OVERLAY))
    .with_event(command(ACTION_SHIFT_SELECT_SPREADSHEET))
    .with_event(wait_for_released_capture())
    .with_assertion(ReplayAssertion::new(
        "accepted_routes",
        POSITIVE_ACCEPTED_ROUTES.to_string(),
    ))
    .with_assertion(ReplayAssertion::new("captured_target", EMPTY_MARKER))
}

/// Creates the invalid full mouse replay script.
#[must_use]
pub fn pointer_mouse_negative_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(
        NEGATIVE_FIXTURE_ID,
        NEGATIVE_NAME,
        FIXTURE_COMPONENT,
    ))
    .with_event(command(ACTION_DISABLED_REGION))
    .with_event(command(ACTION_MISSING_DRAG_START))
    .with_event(command(ACTION_STALE_CAPTURE))
    .with_event(command(ACTION_UNSUPPORTED_BACKEND))
    .with_event(ReplayEvent::Mouse {
        target: TARGET_FULL_MOUSE.to_string(),
        x: OUT_OF_BOUNDS_X,
        y: OUT_OF_BOUNDS_Y,
    })
    .with_assertion(ReplayAssertion::new(
        "accepted_routes",
        NEGATIVE_ACCEPTED_ROUTES.to_string(),
    ))
    .with_assertion(ReplayAssertion::new("captured_target", EMPTY_MARKER))
}

/// Runs full mouse replay fixtures and returns deterministic proof rows.
#[must_use]
pub fn pointer_mouse_replay_proof() -> PointerMouseReplayProof {
    let positive = run_replay_row(pointer_mouse_positive_replay_script());
    let negative = run_replay_row(pointer_mouse_negative_replay_script());
    PointerMouseReplayProof {
        rows: vec![positive, negative],
    }
}

fn command(action_id: &str) -> ReplayEvent {
    ReplayEvent::Command {
        action_id: action_id.to_string(),
    }
}

fn wait_for_released_capture() -> ReplayEvent {
    ReplayEvent::Wait {
        condition: ReplayWaitCondition::new(
            ReplayWaitPredicate::StateEquals {
                key: "captured_target".to_string(),
                expected: EMPTY_MARKER.to_string(),
            },
            WAIT_BUDGET,
        ),
    }
}

fn run_replay_row(script: ReplayScript) -> PointerMouseReplayProofRow {
    let fixture_id = script.metadata.id.clone();
    let scenario = script.metadata.name.clone();
    let mut target = pointer_mouse_replay_target();
    let report = run_replay(&script, &mut target);
    let route_summary_hash = target
        .snapshot_bytes(SNAPSHOT_ROUTE_SUMMARY)
        .map_or_else(String::new, |bytes| blake3_hex(&bytes));
    let diagnostics = join_issue_codes(&report);
    PointerMouseReplayProofRow {
        fixture_id,
        scenario,
        state_hash: report.state_hash,
        route_summary_hash,
        diagnostics,
    }
}

fn join_issue_codes(report: &ReplayReport) -> String {
    let values = report
        .issues
        .iter()
        .map(|issue| format!("{:?}", issue.code))
        .collect::<Vec<_>>();
    if values.is_empty() {
        EMPTY_MARKER.to_string()
    } else {
        values.join(";")
    }
}
