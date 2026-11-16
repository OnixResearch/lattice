//! Core application state and event loop.

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crossterm::cursor;
use crossterm::event;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use rat_fuzzy::FuzzyCandidate;
use rat_fuzzy::FuzzySource;
use rat_fuzzy::FuzzyState;
use rat_nodegraph::layout::LayoutConfig;
use rat_nodegraph::layout::auto_layout;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::view::NodeGraphState;
use rat_nodegraph::view::node_bounds;
use rat_streaming::StreamingOutput;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::info;

use crate::agent_node;
use crate::config::AppConfig;
use crate::config::NodeTemplate;
use crate::config::render_template_command;
use crate::control_flow;
use crate::control_flow::ConditionBranch;
use crate::control_flow::ConditionNodeConfig;
use crate::control_flow::MergeNodeConfig;
use crate::control_flow::MergePolicy;
use crate::control_flow::SubworkflowNodeConfig;
use crate::credentials::NodeCredentialRefs;
use crate::credentials::SecretRef;
use crate::engine::executor::Executor;
use crate::engine::executor::NodeStatus;
use crate::engine::executor::RunId;
use crate::engine::executor::StepResult;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;
use crate::engine::workflow::WorkflowNodeIdentity;
use crate::node_kind_provider;
use crate::replay::ReplayCursor;
use crate::replay::ReplayTimeline;
use crate::store::RunDetailRecord;
use crate::store::RunNodeDetail;
use crate::store::RunNodeDetailStatus;
use crate::store::RunRecord;
use crate::store::Store;
use crate::tui::input;
use crate::tui::panels;
use crate::workflow_validation::WorkflowDiagnostic;

const INITIAL_VIEWPORT_WIDTH: u16 = 80;
const INITIAL_VIEWPORT_HEIGHT: u16 = 24;
const NEW_NODE_HORIZONTAL_GAP: i32 = 4;
const RUN_HISTORY_LIMIT: usize = 20;
const APPROVAL_LIST_LIMIT: u32 = 20;
const TRIGGER_STATUS_HISTORY_LIMIT: usize = 100;
const INITIAL_TUI_RUN_ID: u64 = 0;
const ACTIVE_REPLAY_TIMESTAMP_EPOCH_MS: i64 = 0;
const RUN_EVENT_CAPACITY: usize = 32;
const AUTO_LAYOUT_HORIZONTAL_SPACING_COLUMNS: u16 = 6;
const AUTO_LAYOUT_VERTICAL_SPACING_ROWS: u16 = 2;
const AUTO_LAYOUT_NODE_WIDTH_COLUMNS: u16 = 20;
const AUTO_LAYOUT_NODE_HEIGHT_ROWS: u16 = 5;
const CREDENTIAL_ENTRY_SEPARATOR: &str = ",";
const CREDENTIAL_BINDING_SEPARATOR: &str = "=";
const SECRET_IDENTITY_SEPARATOR: &str = ":";
const SECRET_FIELD_SEPARATOR: &str = ".";
const CONDITION_BRANCH_ENTRY_SEPARATOR: &str = ",";
const CONDITION_BRANCH_BINDING_SEPARATOR: &str = "=";
const MERGE_POLICY_INPUT_SEPARATOR: &str = ":";
const MERGE_INPUT_SEPARATOR: &str = ",";
const DEFAULT_CONDITION_BRANCH_LABEL: &str = "match";
const DEFAULT_CONDITION_PREDICATE: &str = "always";
const DEFAULT_MERGE_INPUT_LEFT: &str = "left";
const DEFAULT_MERGE_INPUT_RIGHT: &str = "right";
const DEFAULT_HTTP_DRAFT: &str = "GET https://example.com";
const DEFAULT_FILE_DRAFT: &str = "read ./file.txt";

mod authoring;
pub(in crate::tui) mod editing;
mod lifecycle;
mod model;
mod runtime;

pub use editing::fields::PropertyEditError;
pub use editing::fields::PropertyEditField;
pub use editing::fields::PropertyEditState;
pub use editing::fields::validate_agent_output_mode;
pub use editing::fields::validate_agent_target;
pub use editing::fields::validate_timeout_secs;
pub use model::commands::Action;
pub use model::commands::PaletteData;
pub use model::commands::palette_action;
pub use model::creation::AddNodeFlowState;
pub use model::creation::AddNodeKind;
pub use model::creation::AddNodeStep;
pub use model::creation::normalized_node_label;
pub use model::creation::template_parameter_values;
pub use model::dialogs::DiagnosticsState;
pub use model::dialogs::WorkflowBrowserState;
pub use model::dialogs::WorkflowTemplateBrowseStep;
pub use model::dialogs::WorkflowTemplateBrowserState;
pub use model::dialogs::WorkflowTemplateDraft;
pub use model::live::NodeRunStatus;
pub use model::live::RunState;
pub use model::receipts::ApprovalState;
pub use model::receipts::RunHistoryState;
pub use model::receipts::format_run_record;
pub use model::receipts::run_detail_record_from_results;
pub use model::receipts::trigger_status_lines_from_history;
pub use model::values::RunOutputLine;
pub use model::values::latest_node_output_records;
pub use model::values::rendered_latest_node_output_lines;

#[cfg(test)]
mod tests;
/// Which panel has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Canvas,
    Properties,
    Output,
}

/// Main application state.
pub struct State {
    pub workflow: Workflow,
    pub graph_state: NodeGraphState,
    pub canvas_view: crate::tui::rows::CanvasView,
    pub focus: FocusPanel,
    pub show_palette: bool,
    pub palette_state: FuzzyState,
    pub palette_data: PaletteData,
    pub output: StreamingOutput,
    pub run_output_lines: Vec<RunOutputLine>,
    pub workflow_browser: WorkflowBrowserState,
    pub workflow_template_browser: WorkflowTemplateBrowserState,
    pub workflow_template_draft: Option<WorkflowTemplateDraft>,
    pub property_edit: Option<PropertyEditState>,
    pub authoring: Option<crate::tui::authoring::State>,
    pub add_node_flow: AddNodeFlowState,
    pub node_templates: Vec<NodeTemplate>,
    pub workflow_templates: Vec<crate::workflow_template::ValidatedBundle>,
    pub run_state: RunState,
    pub run_history: RunHistoryState,
    pub approvals: ApprovalState,
    pub diagnostics: DiagnosticsState,
    pub store: Option<Store>,
    active_run: Option<crate::tui::app::model::live::ActiveRun>,
    next_tui_run_id: u64,
    pub config: AppConfig,
    pub last_drag_canvas: Option<(i32, i32)>,
    pub rows_hit_area: Option<ratatui::layout::Rect>,
    pub rows_scroll: usize,
    dirty: bool,
    confirm_quit: bool,
    executor_factory: Arc<dyn Fn() -> Executor + Send + Sync>,
    authority_provider: Arc<dyn crate::engine::effect::execution::RunContextProvider>,
}

#[cfg(test)]
mod rows_tests;
