use super::super::*;
use crate::terminal_ide_kit::WorkspaceFocusPolicy;
use crate::terminal_ide_kit::WorkspaceId;
use crate::terminal_ide_kit::WorkspacePaneId;
use crate::terminal_ide_kit::WorkspaceShellCommand;
use crate::terminal_ide_kit::WorkspaceShellDiagnostic;
use crate::terminal_ide_kit::WorkspaceShellDiagnosticCode;
use crate::terminal_ide_kit::WorkspaceShellEffectRequest;
use crate::terminal_ide_kit::WorkspaceShellFocus;
use crate::terminal_ide_kit::WorkspaceShellModel;
use crate::terminal_ide_kit::WorkspaceShellPane;
use crate::terminal_ide_kit::WorkspaceShellTab;
use crate::terminal_ide_kit::WorkspaceShellWorkspace;
use crate::terminal_ide_kit::WorkspaceSplitAxis;
use crate::terminal_ide_kit::WorkspaceTabId;
use crate::terminal_ide_kit::WorkspaceTaskStatus;
use crate::terminal_ide_kit::reduce_workspace_shell_command;
use crate::terminal_ide_kit::validate_workspace_shell;
use crate::terminal_ide_kit::workspace_status_rollup;

const TARGET_WORKSPACE_SHELL: &str = "workspace-shell";
const WORKSPACE_LIVE: &str = "workspace:main";
const WORKSPACE_PERSISTED: &str = "workspace:main:persisted";
const TAB_LIVE: &str = "tab:main";
const TAB_PERSISTED: &str = "tab:main:persisted";
const PANE_EDITOR_LIVE: &str = "pane:editor";
const PANE_EDITOR_PERSISTED: &str = "pane:editor:persisted";
const PANE_TASK_LIVE: &str = "pane:task";
const PANE_TASK_PERSISTED: &str = "pane:task:persisted";
const PANE_STALE_LIVE: &str = "pane:stale";
const ACTION_SPLIT_BACKGROUND: &str = "workspace.split.background";
const ACTION_STATUS_WORKING: &str = "workspace.status.working";
const ACTION_FOCUS_TASK: &str = "workspace.focus.task";
const ACTION_FOCUS_STALE: &str = "workspace.focus.stale";
const ACTION_RESTORE_IMPOSSIBLE: &str = "workspace.restore.impossible";
const ACTION_DISABLED_CLOSE: &str = "workspace.close.disabled";
const SNAPSHOT_WORKSPACE_STATE: &str = "workspace-state";
const STATUS_SEPARATOR: &str = ",";
const EMPTY_MARKER: &str = "-";
const DEFAULT_REPLAY_WIDTH: u16 = 120;
const DEFAULT_REPLAY_HEIGHT: u16 = 40;
const WAIT_BUDGET: usize = 1;
const POSITIVE_FIXTURE_ID: &str = "workspace-shell.positive";
const NEGATIVE_FIXTURE_ID: &str = "workspace-shell.negative";

/// Pure replay target for workspace shell behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceShellReplayTarget {
    model: WorkspaceShellModel,
    commands: std::collections::BTreeMap<String, WorkspaceShellCommand>,
    disabled_actions: std::collections::BTreeSet<String>,
    diagnostics: std::collections::BTreeSet<String>,
    width: u16,
    height: u16,
}

impl WorkspaceShellReplayTarget {
    /// Creates a replay target from a model and command table.
    #[must_use]
    pub fn new(
        model: WorkspaceShellModel,
        commands: std::collections::BTreeMap<String, WorkspaceShellCommand>,
        disabled_actions: std::collections::BTreeSet<String>,
    ) -> Self {
        let diagnostics = diagnostic_codes(&validate_workspace_shell(&model));
        Self {
            model,
            commands,
            disabled_actions,
            diagnostics,
            width: DEFAULT_REPLAY_WIDTH,
            height: DEFAULT_REPLAY_HEIGHT,
        }
    }
}

impl ReplayTarget for WorkspaceShellReplayTarget {
    fn focus_targets(&self) -> std::collections::BTreeSet<String> {
        self.model
            .workspaces
            .iter()
            .flat_map(|workspace| &workspace.tabs)
            .flat_map(|tab| &tab.panes)
            .map(|pane| pane.live_id.to_string())
            .collect()
    }

    fn focused_target(&self) -> Option<String> {
        self.model.focus.as_ref().map(|focus| focus.pane.to_string())
    }

    fn bounds(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn action_known(&self, action_id: &str) -> bool {
        self.commands.contains_key(action_id) || self.disabled_actions.contains(action_id)
    }

    fn action_enabled(&self, action_id: &str) -> bool {
        self.commands.contains_key(action_id)
    }

    fn validate_event(&self, event: &ReplayEvent) -> Vec<ReplayIssue> {
        match event {
            ReplayEvent::Mouse { x, y, .. } if *x >= self.width || *y >= self.height => {
                vec![ReplayIssue::new(
                    ReplayIssueCode::ImpossibleGeometry,
                    TARGET_WORKSPACE_SHELL,
                    "workspace shell replay received impossible geometry",
                )]
            }
            _ => Vec::new(),
        }
    }

    fn apply_event(&mut self, event: &ReplayEvent) {
        match event {
            ReplayEvent::Command { action_id } => self.apply_command(action_id),
            ReplayEvent::Resize { width, height } => {
                self.width = *width;
                self.height = *height;
            }
            _ => {}
        }
    }

    fn snapshot_state(&self) -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::from([
            ("diagnostics".to_string(), join_ids(self.diagnostics.iter())),
            ("focus_pane".to_string(), option_id(self.focused_target())),
            ("layout_shape".to_string(), layout_shape(&self.model)),
            ("pane_content".to_string(), pane_content(&self.model)),
            ("panes".to_string(), join_ids(pane_ids(&self.model).iter())),
            (
                "workspace_status".to_string(),
                self.model
                    .workspaces
                    .first()
                    .map_or(WorkspaceTaskStatus::Unknown, |workspace| workspace_status_rollup(workspace).overall)
                    .label()
                    .to_string(),
            ),
        ])
    }

    fn snapshot_bytes(&self, label: &str) -> Option<Vec<u8>> {
        (label == SNAPSHOT_WORKSPACE_STATE).then(|| state_bytes(&self.snapshot_state()))
    }

    fn diagnostic_events(&self) -> std::collections::BTreeSet<String> {
        self.diagnostics.clone()
    }
}

impl WorkspaceShellReplayTarget {
    fn apply_command(&mut self, action_id: &str) {
        let Some(command) = self.commands.get(action_id) else {
            return;
        };
        let output = reduce_workspace_shell_command(&self.model, command);
        self.model = output.model;
        self.capture_diagnostics(&output.diagnostics);
        self.capture_effects(&output.effects);
    }

    fn capture_diagnostics(&mut self, diagnostics: &[WorkspaceShellDiagnostic]) {
        self.diagnostics.extend(diagnostic_codes(diagnostics));
    }

    fn capture_effects(&mut self, effects: &[WorkspaceShellEffectRequest]) {
        for effect in effects {
            match effect {
                WorkspaceShellEffectRequest::SplitPane { .. } => {
                    self.diagnostics.insert("effect.split-pane".to_string());
                }
                WorkspaceShellEffectRequest::UpdateTaskStatus { .. } => {
                    self.diagnostics.insert("effect.status-update".to_string());
                }
                WorkspaceShellEffectRequest::FocusPane { .. } => {
                    self.diagnostics.insert("effect.focus-pane".to_string());
                }
                WorkspaceShellEffectRequest::CreateTab { .. }
                | WorkspaceShellEffectRequest::ClosePane { .. }
                | WorkspaceShellEffectRequest::RestoreLayout => {}
            }
        }
    }
}

/// Workspace shell replay evidence row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceShellReplayEvidenceRow {
    /// Fixture ID.
    pub fixture_id: String,
    /// Scenario family.
    pub scenario: String,
    /// Deterministic final-state BLAKE3 hash.
    pub state_hash: String,
    /// Deterministic workspace snapshot BLAKE3 hash.
    pub snapshot_hash: String,
    /// Stable diagnostics observed by the fixture.
    pub diagnostics: String,
}

/// Workspace shell replay evidence summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceShellReplayEvidence {
    /// Positive and negative fixture rows.
    pub rows: Vec<WorkspaceShellReplayEvidenceRow>,
}

/// Creates the baseline workspace shell replay target.
#[must_use]
pub fn workspace_shell_replay_target() -> WorkspaceShellReplayTarget {
    WorkspaceShellReplayTarget::new(
        baseline_model(),
        baseline_commands(),
        [ACTION_DISABLED_CLOSE.to_string()].into_iter().collect(),
    )
}

/// Creates a valid workspace shell replay script.
#[must_use]
pub fn workspace_shell_positive_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(POSITIVE_FIXTURE_ID, "workspace shell positive", TARGET_WORKSPACE_SHELL))
        .with_event(ReplayEvent::Command {
            action_id: ACTION_SPLIT_BACKGROUND.to_string(),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::StateContains {
                    key: "panes".to_string(),
                    needle: PANE_TASK_LIVE.to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::StateEquals {
                    key: "focus_pane".to_string(),
                    expected: PANE_EDITOR_LIVE.to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Command {
            action_id: ACTION_STATUS_WORKING.to_string(),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::StateEquals {
                    key: "workspace_status".to_string(),
                    expected: WorkspaceTaskStatus::Working.label().to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Command {
            action_id: ACTION_FOCUS_TASK.to_string(),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::FocusTarget {
                    target: PANE_TASK_LIVE.to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::SnapshotContains {
                    label: SNAPSHOT_WORKSPACE_STATE.to_string(),
                    needle: PANE_TASK_LIVE.to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_assertion(ReplayAssertion::new("focus_pane", PANE_TASK_LIVE))
        .with_assertion(ReplayAssertion::new("workspace_status", WorkspaceTaskStatus::Working.label()))
}

/// Creates an invalid workspace shell replay script with stable diagnostics.
#[must_use]
pub fn workspace_shell_negative_replay_script() -> ReplayScript {
    ReplayScript::new(ReplayMetadata::new(NEGATIVE_FIXTURE_ID, "workspace shell negative", TARGET_WORKSPACE_SHELL))
        .with_event(ReplayEvent::Command {
            action_id: ACTION_DISABLED_CLOSE.to_string(),
        })
        .with_event(ReplayEvent::Command {
            action_id: ACTION_FOCUS_STALE.to_string(),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::DiagnosticPresent {
                    code: format!("{:?}", WorkspaceShellDiagnosticCode::StaleLiveId),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Command {
            action_id: ACTION_RESTORE_IMPOSSIBLE.to_string(),
        })
        .with_event(ReplayEvent::Wait {
            condition: ReplayWaitCondition::new(
                ReplayWaitPredicate::Unsupported {
                    predicate: "wall-clock-output-scrape".to_string(),
                },
                WAIT_BUDGET,
            ),
        })
        .with_event(ReplayEvent::Mouse {
            target: PANE_EDITOR_LIVE.to_string(),
            x: DEFAULT_REPLAY_WIDTH,
            y: DEFAULT_REPLAY_HEIGHT,
        })
}

/// Runs workspace shell replay fixtures and returns deterministic evidence rows.
#[must_use]
pub fn workspace_shell_replay_evidence() -> WorkspaceShellReplayEvidence {
    let positive = run_workspace_fixture(workspace_shell_positive_replay_script());
    let negative = run_workspace_fixture(workspace_shell_negative_replay_script());
    WorkspaceShellReplayEvidence {
        rows: vec![positive, negative],
    }
}

fn run_workspace_fixture(script: ReplayScript) -> WorkspaceShellReplayEvidenceRow {
    let fixture_id = script.metadata.id.clone();
    let scenario = script.metadata.name.clone();
    let mut target = workspace_shell_replay_target();
    let report = run_replay(&script, &mut target);
    let snapshot_hash =
        target.snapshot_bytes(SNAPSHOT_WORKSPACE_STATE).map_or_else(String::new, |bytes| blake3_hex(&bytes));
    WorkspaceShellReplayEvidenceRow {
        fixture_id,
        scenario,
        state_hash: report.state_hash,
        snapshot_hash,
        diagnostics: join_ids(report.issues.iter().map(|issue| format!("{:?}", issue.code))),
    }
}

fn baseline_model() -> WorkspaceShellModel {
    WorkspaceShellModel::new(vec![WorkspaceShellWorkspace::new(
        WORKSPACE_LIVE,
        WORKSPACE_PERSISTED,
        "Workspace",
        vec![WorkspaceShellTab::new(TAB_LIVE, TAB_PERSISTED, "Main", vec![
            WorkspaceShellPane::new(PANE_EDITOR_LIVE, PANE_EDITOR_PERSISTED, "Editor", WorkspaceTaskStatus::Idle)
                .with_content_summary("editor ready")
                .with_diagnostic(WorkspaceShellDiagnostic::new(
                    WorkspaceShellDiagnosticCode::MissingStatusAdapter,
                    PANE_EDITOR_LIVE,
                    "task status adapter is missing",
                )),
        ])],
    )])
    .with_retired_live_id(WorkspacePaneId::new(PANE_STALE_LIVE))
}

fn baseline_commands() -> std::collections::BTreeMap<String, WorkspaceShellCommand> {
    std::collections::BTreeMap::from([
        (ACTION_SPLIT_BACKGROUND.to_string(), WorkspaceShellCommand::SplitPane {
            workspace: WorkspaceId::new(WORKSPACE_LIVE),
            tab: WorkspaceTabId::new(TAB_LIVE),
            target_pane: WorkspacePaneId::new(PANE_EDITOR_LIVE),
            pane: WorkspaceShellPane::new(PANE_TASK_LIVE, PANE_TASK_PERSISTED, "Task", WorkspaceTaskStatus::Idle)
                .with_content_summary("task starting"),
            axis: WorkspaceSplitAxis::Horizontal,
            focus_policy: WorkspaceFocusPolicy::Preserve,
        }),
        (ACTION_STATUS_WORKING.to_string(), WorkspaceShellCommand::UpdateTaskStatus {
            pane: WorkspacePaneId::new(PANE_TASK_LIVE),
            status: WorkspaceTaskStatus::Working,
        }),
        (ACTION_FOCUS_TASK.to_string(), WorkspaceShellCommand::FocusPane {
            workspace: WorkspaceId::new(WORKSPACE_LIVE),
            tab: WorkspaceTabId::new(TAB_LIVE),
            pane: WorkspacePaneId::new(PANE_TASK_LIVE),
        }),
        (ACTION_FOCUS_STALE.to_string(), WorkspaceShellCommand::FocusPane {
            workspace: WorkspaceId::new(WORKSPACE_LIVE),
            tab: WorkspaceTabId::new(TAB_LIVE),
            pane: WorkspacePaneId::new(PANE_STALE_LIVE),
        }),
        (ACTION_RESTORE_IMPOSSIBLE.to_string(), WorkspaceShellCommand::RestoreLayout {
            model: impossible_restore_model(),
        }),
    ])
}

fn impossible_restore_model() -> WorkspaceShellModel {
    WorkspaceShellModel::new(vec![WorkspaceShellWorkspace::new(
        WORKSPACE_LIVE,
        WORKSPACE_PERSISTED,
        "Invalid Workspace",
        vec![WorkspaceShellTab::new(TAB_LIVE, TAB_PERSISTED, "Invalid", vec![
            WorkspaceShellPane::new(PANE_EDITOR_LIVE, PANE_EDITOR_PERSISTED, "Editor", WorkspaceTaskStatus::Idle),
            WorkspaceShellPane::new(PANE_TASK_LIVE, PANE_EDITOR_PERSISTED, "Duplicate", WorkspaceTaskStatus::Idle),
        ])],
    )])
    .with_focus(WorkspaceShellFocus::new(WORKSPACE_LIVE, TAB_LIVE, PANE_STALE_LIVE))
    .with_retired_live_id(WorkspacePaneId::new(PANE_STALE_LIVE))
}

fn pane_ids(model: &WorkspaceShellModel) -> Vec<String> {
    let mut ids = model
        .workspaces
        .iter()
        .flat_map(|workspace| &workspace.tabs)
        .flat_map(|tab| &tab.panes)
        .map(|pane| pane.live_id.to_string())
        .collect::<Vec<_>>();
    ids.sort();
    ids
}

fn pane_content(model: &WorkspaceShellModel) -> String {
    join_ids(
        model
            .workspaces
            .iter()
            .flat_map(|workspace| &workspace.tabs)
            .flat_map(|tab| &tab.panes)
            .map(|pane| format!("{}:{}", pane.live_id, pane.content_summary)),
    )
}

fn layout_shape(model: &WorkspaceShellModel) -> String {
    let workspace_count = model.workspaces.len();
    let tab_count = model.workspaces.iter().map(|workspace| workspace.tabs.len()).sum::<usize>();
    let pane_count = model
        .workspaces
        .iter()
        .flat_map(|workspace| &workspace.tabs)
        .map(|tab| tab.panes.len())
        .sum::<usize>();
    format!("workspaces={workspace_count},tabs={tab_count},panes={pane_count}")
}

fn diagnostic_codes(diagnostics: &[WorkspaceShellDiagnostic]) -> std::collections::BTreeSet<String> {
    diagnostics.iter().map(|diagnostic| format!("{:?}", diagnostic.code)).collect()
}

fn state_bytes(state: &std::collections::BTreeMap<String, String>) -> Vec<u8> {
    state.iter().map(|(key, value)| format!("{key}={value}\n")).collect::<String>().into_bytes()
}

fn join_ids(ids: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    let values = ids.into_iter().map(|id| id.as_ref().to_string()).collect::<Vec<_>>();
    if values.is_empty() {
        EMPTY_MARKER.to_string()
    } else {
        values.join(STATUS_SEPARATOR)
    }
}

fn option_id(id: Option<String>) -> String {
    id.unwrap_or_else(|| EMPTY_MARKER.to_string())
}
