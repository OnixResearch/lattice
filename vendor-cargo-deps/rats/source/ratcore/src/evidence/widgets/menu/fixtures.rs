use super::*;

/// Returns the committed leader-key positive replay fixture.
#[must_use]
pub fn leader_key_positive_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(
        POSITIVE_FIXTURE_ID,
        "Leader-key workspace action baseline",
        TARGET_LEADER_KEY,
    ))
    .with_event(ReplayEvent::Key {
        target: TARGET_LEADER_KEY.to_string(),
        key: KEY_WORKSPACE.to_string(),
    })
    .with_event(ReplayEvent::Wait {
        condition: ReplayWaitCondition::new(
            ReplayWaitPredicate::StateEquals {
                key: "current_menu".to_string(),
                expected: SUBMENU_WORKSPACE.to_string(),
            },
            WAIT_BUDGET,
        ),
    })
    .with_event(ReplayEvent::Key {
        target: TARGET_LEADER_KEY.to_string(),
        key: KEY_SAVE.to_string(),
    })
    .with_assertion(ReplayAssertion::new("selected_action", ACTION_WORKSPACE_SAVE))
    .with_assertion(ReplayAssertion::new("visible", false.to_string()))
    .with_snapshot(ReplaySnapshotExpectation::new("state", state_snapshot_hash(&accepted_positive_state())))
}

/// Returns the committed leader-key negative replay fixture.
#[must_use]
pub fn leader_key_negative_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(NEGATIVE_FIXTURE_ID, "Leader-key invalid input baseline", TARGET_LEADER_KEY))
        .with_event(ReplayEvent::Key {
            target: TARGET_LEADER_KEY.to_string(),
            key: KEY_DISABLED.to_string(),
        })
        .with_event(ReplayEvent::Key {
            target: TARGET_STALE_LEADER_KEY.to_string(),
            key: KEY_UNKNOWN.to_string(),
        })
        .with_event(ReplayEvent::Key {
            target: TARGET_LEADER_KEY.to_string(),
            key: KEY_UNKNOWN.to_string(),
        })
        .with_assertion(ReplayAssertion::new("current_menu", MENU_ROOT))
        .with_assertion(ReplayAssertion::new("selected_action", EMPTY_MARKER))
        .with_assertion(ReplayAssertion::new("visible", true.to_string()))
        .with_snapshot(ReplaySnapshotExpectation::new("state", state_snapshot_hash(&accepted_negative_state())))
}

/// Leader-key replay summary row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderReplayRow {
    /// Fixture ID.
    pub fixture_id: String,
    /// Scenario family.
    pub scenario: String,
    /// Deterministic final-state BLAKE3 hash.
    pub state_hash: String,
    /// Stable diagnostics observed by the fixture.
    pub diagnostics: String,
}

/// Leader-key replay summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderReplaySummary {
    /// Positive and negative leader-key fixture rows.
    pub rows: Vec<LeaderReplayRow>,
}

/// Runs leader-key replay fixtures and returns deterministic evidence rows.
#[must_use]
pub fn leader_key_replay_summary() -> LeaderReplaySummary {
    LeaderReplaySummary {
        rows: vec![
            run_leader_fixture(leader_key_positive_replay_script()),
            run_leader_fixture(leader_key_negative_replay_script()),
        ],
    }
}

pub(super) fn baseline() -> crate::leaderkey::LeaderMenu<String> {
    let hidden = std::collections::HashSet::new();
    let (menu, conflicts) = crate::leaderkey::build_from_items(baseline_items(), &hidden);
    debug_assert!(conflicts.is_empty());
    menu
}

fn run_leader_fixture(script: ReplayScript) -> LeaderReplayRow {
    let fixture_id = script.metadata.id.clone();
    let scenario = script.metadata.name.clone();
    let mut target = leader_key_replay_target();
    let report = run_replay(&script, &mut target);
    LeaderReplayRow {
        fixture_id,
        scenario,
        state_hash: report.state_hash,
        diagnostics: join_ids(report.issues.iter().map(|issue| format!("{:?}", issue.code))),
    }
}

fn baseline_items() -> Vec<crate::leaderkey::MenuContribution<String>> {
    vec![
        contribution(
            'w',
            "Workspace",
            crate::leaderkey::LeaderAction::Submenu(SUBMENU_WORKSPACE.to_string()),
            crate::leaderkey::MenuPlacement::Root,
        ),
        contribution(
            'd',
            "Publish docs",
            crate::leaderkey::LeaderAction::Action(ACTION_DISABLED_DOCS.to_string()),
            crate::leaderkey::MenuPlacement::Root,
        ),
        contribution(
            's',
            "Save workspace",
            crate::leaderkey::LeaderAction::Action(ACTION_WORKSPACE_SAVE.to_string()),
            crate::leaderkey::MenuPlacement::Submenu(SUBMENU_WORKSPACE.to_string()),
        ),
    ]
}

fn contribution(
    key: char,
    label: &str,
    action: crate::leaderkey::LeaderAction<String>,
    placement: crate::leaderkey::MenuPlacement,
) -> crate::leaderkey::MenuContribution<String> {
    crate::leaderkey::MenuContribution {
        key,
        label: label.to_string(),
        action,
        placement,
        priority: crate::leaderkey::PRIORITY_BUILTIN,
        source: "leader-replay".to_string(),
    }
}

fn accepted_positive_state() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("breadcrumb".to_string(), EMPTY_MARKER.to_string()),
        ("current_items".to_string(), EMPTY_MARKER.to_string()),
        ("current_menu".to_string(), EMPTY_MARKER.to_string()),
        ("depth".to_string(), "0".to_string()),
        ("selected_action".to_string(), ACTION_WORKSPACE_SAVE.to_string()),
        ("surface".to_string(), TARGET_LEADER_KEY.to_string()),
        ("visible".to_string(), false.to_string()),
    ])
}

fn accepted_negative_state() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("breadcrumb".to_string(), EMPTY_MARKER.to_string()),
        ("current_items".to_string(), "d:Publish docs,w:Workspace".to_string()),
        ("current_menu".to_string(), MENU_ROOT.to_string()),
        ("depth".to_string(), "0".to_string()),
        ("selected_action".to_string(), EMPTY_MARKER.to_string()),
        ("surface".to_string(), TARGET_LEADER_KEY.to_string()),
        ("visible".to_string(), true.to_string()),
    ])
}
