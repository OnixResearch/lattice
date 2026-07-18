use super::*;

const TARGET_EDITOR: &str = "editor";
const TARGET_TREE: &str = "tree";
const TARGET_COMMAND_PALETTE: &str = "command-palette";
const ACTION_SAVE: &str = "save";
const ACTION_DISABLED: &str = "disabled";
const ACTION_UNKNOWN: &str = "unknown";
const SNAPSHOT_RENDER: &str = "render";
const DIAGNOSTIC_READY: &str = "demo.ready";
const WAIT_BUDGET: usize = 1;
const EMPTY_WAIT_BUDGET: usize = 0;
const DEMO_WIDTH: u16 = 10;
const DEMO_HEIGHT: u16 = 4;
const OUT_OF_BOUNDS_COORDINATE: u16 = 99;
const EXPECTED_WIDGET_REPLAY_ISSUES: usize = 3;

#[derive(Clone)]
struct DemoTarget {
    focus: String,
    text: String,
    width: u16,
    height: u16,
}

impl DemoTarget {
    fn new() -> Self {
        Self {
            focus: TARGET_EDITOR.to_string(),
            text: String::new(),
            width: DEMO_WIDTH,
            height: DEMO_HEIGHT,
        }
    }
}

impl ReplayTarget for DemoTarget {
    fn focus_targets(&self) -> std::collections::BTreeSet<String> {
        [TARGET_EDITOR.to_string(), TARGET_TREE.to_string()]
            .into_iter()
            .collect()
    }

    fn focused_target(&self) -> Option<String> {
        Some(self.focus.clone())
    }

    fn bounds(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn action_known(&self, action_id: &str) -> bool {
        matches!(action_id, ACTION_SAVE | ACTION_DISABLED)
    }

    fn action_enabled(&self, action_id: &str) -> bool {
        action_id == ACTION_SAVE
    }

    fn apply_event(&mut self, event: &ReplayEvent) {
        match event {
            ReplayEvent::Text { value, .. } => self.text.push_str(value),
            ReplayEvent::Key { key, .. } if key == "Tab" => self.focus = TARGET_TREE.to_string(),
            ReplayEvent::Resize { width, height } => {
                self.width = *width;
                self.height = *height;
            }
            ReplayEvent::Command { action_id } if action_id == ACTION_SAVE => {
                self.text.push('!');
            }
            _ => {}
        }
    }

    fn snapshot_state(&self) -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::from([
            ("focus".to_string(), self.focus.clone()),
            ("text".to_string(), self.text.clone()),
            (
                "size".to_string(),
                format!("{}x{}", self.width, self.height),
            ),
        ])
    }

    fn snapshot_bytes(&self, label: &str) -> Option<Vec<u8>> {
        (label == SNAPSHOT_RENDER).then(|| format!("render:{}", self.text).into_bytes())
    }

    fn diagnostic_events(&self) -> std::collections::BTreeSet<String> {
        (!self.text.is_empty())
            .then(|| DIAGNOSTIC_READY.to_string())
            .into_iter()
            .collect()
    }
}

#[test]
fn replays_valid_script_and_hashes_snapshots() {
    let mut target = DemoTarget::new();
    let expected_render = blake3_hex(b"render:hi!");
    let script = ReplayScript::new(ReplayMetadata::new("case", "case", "demo"))
        .with_event(ReplayEvent::Text {
            target: TARGET_EDITOR.to_string(),
            value: "hi".to_string(),
        })
        .with_event(ReplayEvent::Command {
            action_id: ACTION_SAVE.to_string(),
        })
        .with_assertion(ReplayAssertion::new("text", "hi!"))
        .with_snapshot(ReplaySnapshotExpectation::new(
            SNAPSHOT_RENDER,
            expected_render,
        ));

    let outcome = run_replay(&script, &mut target);

    assert!(outcome.is_success());
    assert_eq!(outcome.state.get("text"), Some(&"hi!".to_string()));
    assert_eq!(outcome.snapshots.len(), 2);
}

#[test]
fn rejects_malformed_script_before_mutation() {
    let mut target = DemoTarget::new();
    let script = ReplayScript::new(ReplayMetadata::new("bad", "", "demo"))
        .with_event(ReplayEvent::Unsupported {
            kind: "browser-pointer".to_string(),
        })
        .with_event(ReplayEvent::Text {
            target: TARGET_EDITOR.to_string(),
            value: String::new(),
        });

    let outcome = run_replay(&script, &mut target);

    assert_eq!(outcome.issues.len(), 3);
    assert_eq!(outcome.state.get("text"), Some(&String::new()));
}

#[test]
fn reports_stale_focus_disabled_action_and_bounds() {
    let mut target = DemoTarget::new();
    let script = ReplayScript::new(ReplayMetadata::new("bad", "bad", "demo"))
        .with_event(ReplayEvent::Key {
            target: TARGET_TREE.to_string(),
            key: "Enter".to_string(),
        })
        .with_event(ReplayEvent::Command {
            action_id: ACTION_DISABLED.to_string(),
        })
        .with_event(ReplayEvent::Mouse {
            target: TARGET_EDITOR.to_string(),
            x: OUT_OF_BOUNDS_COORDINATE,
            y: OUT_OF_BOUNDS_COORDINATE,
        });

    let outcome = run_replay(&script, &mut target);

    assert!(
        outcome
            .issues
            .iter()
            .any(|issue| issue.code == ReplayIssueCode::StaleFocus)
    );
    assert!(
        outcome
            .issues
            .iter()
            .any(|issue| issue.code == ReplayIssueCode::DisabledAction)
    );
    assert!(
        outcome
            .issues
            .iter()
            .any(|issue| issue.code == ReplayIssueCode::OutOfBoundsMouse)
    );
}

#[test]
fn waits_on_supplied_state_snapshots_focus_and_diagnostics() {
    let mut target = DemoTarget::new();
    let script = ReplayScript::new(ReplayMetadata::new("wait", "wait", "demo"))
        .with_event(ReplayEvent::Text {
            target: TARGET_EDITOR.to_string(),
            value: "hi".to_string(),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::StateEquals {
                    key: "text".to_string(),
                    expected: "hi".to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::SnapshotContains {
                    label: SNAPSHOT_RENDER.to_string(),
                    needle: "hi".to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::FocusTarget {
                    target: TARGET_EDITOR.to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::DiagnosticPresent {
                    code: DIAGNOSTIC_READY.to_string(),
                },
                WAIT_BUDGET,
            ),
        });

    let outcome = run_replay(&script, &mut target);

    assert!(outcome.is_success());
    assert_eq!(outcome.state.get("text"), Some(&"hi".to_string()));
}

#[test]
fn failed_waits_preserve_last_accepted_state() {
    let mut target = DemoTarget::new();
    let script = ReplayScript::new(ReplayMetadata::new("wait.bad", "bad wait", "demo"))
        .with_event(ReplayEvent::Text {
            target: TARGET_EDITOR.to_string(),
            value: "hi".to_string(),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::StateEquals {
                    key: "missing".to_string(),
                    expected: "value".to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::StateContains {
                    key: "text".to_string(),
                    needle: "absent".to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::Unsupported {
                    predicate: "wall-clock-sleep".to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::DiagnosticPresent {
                    code: "missing.diagnostic".to_string(),
                },
                EMPTY_WAIT_BUDGET,
            ),
        });

    let outcome = run_replay(&script, &mut target);

    assert_eq!(outcome.state.get("text"), Some(&"hi".to_string()));
    assert_issue(&outcome, ReplayIssueCode::MissingWaitTarget);
    assert_issue(&outcome, ReplayIssueCode::WaitConditionUnmet);
    assert_issue(&outcome, ReplayIssueCode::UnsupportedWaitPredicate);
}

#[test]
fn reports_assertion_and_snapshot_mismatch() {
    let mut target = DemoTarget::new();
    let script = ReplayScript::new(ReplayMetadata::new("bad", "bad", "demo"))
        .with_assertion(ReplayAssertion::new("text", "expected"))
        .with_snapshot(ReplaySnapshotExpectation::new("state", "not-a-hash"));

    let outcome = run_replay(&script, &mut target);

    assert!(
        outcome
            .issues
            .iter()
            .any(|issue| issue.code == ReplayIssueCode::AssertionMismatch)
    );
    assert!(
        outcome
            .issues
            .iter()
            .any(|issue| issue.code == ReplayIssueCode::SnapshotMismatch)
    );
}

fn assert_issue(outcome: &ReplayReport, code: ReplayIssueCode) {
    assert!(outcome.issues.iter().any(|issue| issue.code == code));
}

#[path = "widget/tests.rs"]
mod widget;
