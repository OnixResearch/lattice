//! Core application state and event loop.

use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crossterm::cursor;
use crossterm::event;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::{self};
use rat_fuzzy::FuzzyCandidate;
use rat_fuzzy::FuzzySource;
use rat_fuzzy::FuzzyState;
use rat_nodegraph::layout::LayoutConfig;
use rat_nodegraph::layout::auto_layout;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;
use rat_nodegraph::model::PortSpec;
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
use crate::agent_node::AgentOutputMode;
use crate::agent_node::AgentTarget;
use crate::config::AppConfig;
use crate::config::NodeTemplate;
use crate::config::render_template_command;
use crate::credentials::NodeCredentialRefs;
use crate::credentials::SecretRef;
use crate::engine::executor::Executor;
use crate::engine::executor::NodeStatus;
use crate::engine::executor::RunId;
use crate::engine::executor::StepResult;
use crate::engine::value::Value;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;
use crate::store::RunDetailRecord;
use crate::store::RunNodeDetail;
use crate::store::RunNodeDetailStatus;
use crate::store::RunOutputDetail;
use crate::store::RunRecord;
use crate::store::Store;
use crate::tui::input;
use crate::tui::panels;
use crate::workflow_validation;
use crate::workflow_validation::WorkflowDiagnostic;

const INITIAL_VIEWPORT_WIDTH: u16 = 80;
const INITIAL_VIEWPORT_HEIGHT: u16 = 24;
const NEW_NODE_HORIZONTAL_GAP: i32 = 4;
const RUN_HISTORY_LIMIT: usize = 20;
const INITIAL_TUI_RUN_ID: u64 = 0;
const CREDENTIAL_ENTRY_SEPARATOR: &str = ",";
const CREDENTIAL_BINDING_SEPARATOR: &str = "=";
const SECRET_IDENTITY_SEPARATOR: &str = ":";
const SECRET_FIELD_SEPARATOR: &str = ".";

/// Which panel has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusPanel {
    Canvas,
    Properties,
    Output,
}

/// High-level actions produced by input handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    ConfirmQuit,
    CancelQuit,
    RunWorkflow,
    CancelRun,
    OpenRunHistory,
    OpenDiagnostics,
    SaveWorkflow,
    AddNode,
    OpenWorkflow,
    AutoLayout,
    EditNodeLabel,
    EditNodeCommand,
    EditNodeTimeout,
}

/// Command palette data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRunState {
    pub active: bool,
    pub node_statuses: HashMap<u64, NodeRunStatus>,
}

impl AppRunState {
    fn idle() -> Self {
        Self {
            active: false,
            node_statuses: HashMap::new(),
        }
    }
}

struct ActiveRun {
    run_id: RunId,
    workflow: Workflow,
    receiver: mpsc::UnboundedReceiver<RunEvent>,
    handle: JoinHandle<()>,
}

#[derive(Debug)]
enum RunEvent {
    Completed {
        run_id: RunId,
        workflow: Workflow,
        results: Vec<StepResult>,
    },
}

pub struct PaletteData {
    pub items: Vec<FuzzyCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyEditField {
    Label,
    Command,
    TimeoutSecs,
    ImagePath,
    AgentPrompt,
    AgentTarget,
    AgentOutputMode,
    AgentModelHint,
    AgentToolPolicy,
    AgentContextRefs,
    AgentCwd,
    AgentSession,
    AgentCredentials,
}

impl PropertyEditField {
    fn label(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::Command => "command",
            Self::TimeoutSecs => "timeout",
            Self::ImagePath => "image path",
            Self::AgentPrompt => "agent prompt",
            Self::AgentTarget => "agent target",
            Self::AgentOutputMode => "agent output mode",
            Self::AgentModelHint => "agent model hint",
            Self::AgentToolPolicy => "agent tool policy",
            Self::AgentContextRefs => "agent context refs",
            Self::AgentCwd => "agent cwd",
            Self::AgentSession => "agent session",
            Self::AgentCredentials => "agent credentials",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyEditState {
    pub node_id: NodeId,
    pub field: PropertyEditField,
    pub original: String,
    pub draft: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyEditError {
    MissingSelection,
    NodeNotFound,
    MissingConfig,
    InvalidTimeout,
    InvalidAgentTarget,
    InvalidAgentOutputMode,
    InvalidAgentCredentials,
}

pub fn validate_timeout_secs(input: &str) -> Result<u64, PropertyEditError> {
    input.trim().parse::<u64>().map_err(|_| PropertyEditError::InvalidTimeout)
}

pub fn validate_agent_target(input: &str) -> Result<AgentTarget, PropertyEditError> {
    agent_node::parse_agent_target(input.trim()).map_err(|_| PropertyEditError::InvalidAgentTarget)
}

pub fn validate_agent_output_mode(input: &str) -> Result<AgentOutputMode, PropertyEditError> {
    agent_node::parse_agent_output_mode(input.trim()).map_err(|_| PropertyEditError::InvalidAgentOutputMode)
}

fn optional_draft_value(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn context_refs_from_draft(input: &str) -> Vec<String> {
    input.split(',').map(str::trim).filter(|value| !value.is_empty()).map(str::to_string).collect()
}

fn credentials_from_draft(input: &str) -> Result<NodeCredentialRefs, PropertyEditError> {
    let mut refs = NodeCredentialRefs::default();
    for entry in input.split(CREDENTIAL_ENTRY_SEPARATOR).map(str::trim).filter(|entry| !entry.is_empty()) {
        let Some((env_name, identity)) = entry.split_once(CREDENTIAL_BINDING_SEPARATOR) else {
            return Err(PropertyEditError::InvalidAgentCredentials);
        };
        let Some((provider, key_with_field)) = identity.trim().split_once(SECRET_IDENTITY_SEPARATOR) else {
            return Err(PropertyEditError::InvalidAgentCredentials);
        };
        let (key, field) = match key_with_field.split_once(SECRET_FIELD_SEPARATOR) {
            Some((key, field)) => (key, optional_draft_value(field)),
            None => (key_with_field, None),
        };
        let env_name = env_name.trim();
        let provider = provider.trim();
        let key = key.trim();
        if env_name.is_empty() || provider.is_empty() || key.is_empty() {
            return Err(PropertyEditError::InvalidAgentCredentials);
        }
        refs.env.insert(env_name.to_string(), SecretRef::new(provider, key, field));
    }
    Ok(refs)
}

pub(crate) fn credentials_to_draft(credentials: Option<&NodeCredentialRefs>) -> String {
    let mut refs = credentials.map(|credentials| credentials.env.iter().collect::<Vec<_>>()).unwrap_or_default();
    refs.sort_by_key(|(env_name, _)| *env_name);
    refs.into_iter()
        .map(|(env_name, secret_ref)| format!("{}={}", env_name, secret_ref.identity()))
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn normalized_node_label(input: &str, default_label: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        default_label.to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn template_parameter_values(template: &NodeTemplate, values: &[String]) -> HashMap<String, String> {
    template
        .parameters
        .iter()
        .zip(values.iter())
        .map(|(parameter, value)| (parameter.name.clone(), value.clone()))
        .collect()
}

pub fn format_run_record(record: &RunRecord) -> String {
    format!(
        "{} │ {} nodes │ {} failures │ {} cache hits",
        record.run_id, record.node_count, record.failures, record.cache_hits,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutputLine {
    pub run_id: RunId,
    pub node_id: NodeId,
    pub port_id: PortId,
    pub port_label: String,
    pub text: String,
}

impl RunOutputLine {
    fn rendered(&self) -> String {
        format!("  │ {}: {}", self.port_label, self.text)
    }
}

pub fn latest_node_output_records(
    records: &[RunOutputLine],
    workflow: &Workflow,
    node_id: NodeId,
) -> Vec<RunOutputLine> {
    let mut latest_by_port = HashMap::new();
    for record in records.iter().filter(|record| record.node_id == node_id) {
        latest_by_port.insert(record.port_id, record.clone());
    }

    if let Some(node) = workflow.graph.node(node_id) {
        return node.output_ports.iter().filter_map(|port| latest_by_port.remove(&port.id)).collect();
    }

    let mut latest_records = latest_by_port.into_values().collect::<Vec<_>>();
    latest_records.sort_by_key(|record| record.port_id.0);
    latest_records
}

pub fn rendered_latest_node_output_lines(
    records: &[RunOutputLine],
    workflow: &Workflow,
    node_id: NodeId,
) -> Vec<String> {
    latest_node_output_records(records, workflow, node_id)
        .into_iter()
        .map(|record| record.rendered())
        .collect()
}

fn completed_output_records(
    run_id: RunId,
    workflow: &Workflow,
    node_id: NodeId,
    outputs: &HashMap<PortId, Value>,
) -> Vec<RunOutputLine> {
    let mut records = Vec::new();

    if let Some(node) = workflow.graph.node(node_id) {
        for port in &node.output_ports {
            if let Some(value) = outputs.get(&port.id) {
                append_output_value_records(&mut records, run_id, node_id, port.id, &port.label, value);
            }
        }
        return records;
    }

    let mut fallback_outputs = outputs.iter().collect::<Vec<_>>();
    fallback_outputs.sort_by_key(|(port_id, _)| port_id.0);
    for (port_id, value) in fallback_outputs {
        append_output_value_records(&mut records, run_id, node_id, *port_id, &format!("port {}", port_id.0), value);
    }

    records
}

fn append_output_value_records(
    records: &mut Vec<RunOutputLine>,
    run_id: RunId,
    node_id: NodeId,
    port_id: PortId,
    port_label: &str,
    value: &Value,
) {
    let display = value.as_display_string();
    if display.is_empty() {
        return;
    }

    records.extend(display.lines().map(|line| RunOutputLine {
        run_id,
        node_id,
        port_id,
        port_label: port_label.to_string(),
        text: line.to_string(),
    }));
}

// r[impl n8nclone.tui_ux.run_history.details.persist]
pub fn run_detail_record_from_results(run_id: RunId, workflow: &Workflow, results: &[StepResult]) -> RunDetailRecord {
    let nodes = results.iter().map(|step| run_node_detail_from_step(run_id, workflow, step)).collect::<Vec<_>>();
    RunDetailRecord { run_id, nodes }
}

fn run_node_detail_from_step(run_id: RunId, workflow: &Workflow, step: &StepResult) -> RunNodeDetail {
    let node_label =
        workflow.graph.node(step.node_id).map(|node| node.label.clone()).unwrap_or_else(|| "?".to_string());
    match &step.status {
        NodeStatus::Completed { outputs, cached } => RunNodeDetail {
            node_id: step.node_id,
            node_label,
            status: RunNodeDetailStatus::Completed,
            cached: *cached,
            outputs: completed_output_records(run_id, workflow, step.node_id, outputs)
                .into_iter()
                .map(run_output_line_to_detail)
                .collect(),
            error: None,
        },
        NodeStatus::Failed { error } => RunNodeDetail {
            node_id: step.node_id,
            node_label,
            status: RunNodeDetailStatus::Failed,
            cached: false,
            outputs: Vec::new(),
            error: Some(error.clone()),
        },
        NodeStatus::Skipped => RunNodeDetail {
            node_id: step.node_id,
            node_label,
            status: RunNodeDetailStatus::Skipped,
            cached: false,
            outputs: Vec::new(),
            error: None,
        },
        NodeStatus::Pending => RunNodeDetail {
            node_id: step.node_id,
            node_label,
            status: RunNodeDetailStatus::Pending,
            cached: false,
            outputs: Vec::new(),
            error: None,
        },
        NodeStatus::Running => RunNodeDetail {
            node_id: step.node_id,
            node_label,
            status: RunNodeDetailStatus::Running,
            cached: false,
            outputs: Vec::new(),
            error: None,
        },
    }
}

fn run_output_line_to_detail(line: RunOutputLine) -> RunOutputDetail {
    RunOutputDetail {
        port_id: line.port_id,
        port_label: line.port_label,
        text: line.text,
    }
}

fn canceled_run_detail_record(
    run_id: RunId,
    workflow: &Workflow,
    statuses: &HashMap<u64, NodeRunStatus>,
) -> RunDetailRecord {
    let nodes = workflow
        .graph
        .nodes()
        .map(|node| {
            let status = statuses.get(&node.id.0).copied().unwrap_or(NodeRunStatus::Skipped);
            RunNodeDetail {
                node_id: node.id,
                node_label: node.label.clone(),
                status: run_detail_status_from_node_run_status(status),
                cached: false,
                outputs: Vec::new(),
                error: None,
            }
        })
        .collect::<Vec<_>>();
    RunDetailRecord { run_id, nodes }
}

fn run_detail_status_from_node_run_status(status: NodeRunStatus) -> RunNodeDetailStatus {
    match status {
        NodeRunStatus::Pending => RunNodeDetailStatus::Pending,
        NodeRunStatus::Running => RunNodeDetailStatus::Running,
        NodeRunStatus::Completed => RunNodeDetailStatus::Completed,
        NodeRunStatus::Failed => RunNodeDetailStatus::Failed,
        NodeRunStatus::Skipped => RunNodeDetailStatus::Skipped,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddNodeStep {
    Kind,
    TemplateSelect,
    TemplateParameter,
    Label,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddNodeKind {
    Shell,
    ImageInput,
    PiAgent,
    ClankersAgent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddNodeFlowState {
    pub open: bool,
    pub step: AddNodeStep,
    pub kind: AddNodeKind,
    pub label: String,
    pub command: String,
    pub selected_template: usize,
    pub parameter_values: Vec<String>,
    pub parameter_index: usize,
}

impl AddNodeFlowState {
    fn closed() -> Self {
        Self {
            open: false,
            step: AddNodeStep::Label,
            kind: AddNodeKind::Shell,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    fn kind() -> Self {
        Self {
            open: true,
            step: AddNodeStep::Kind,
            kind: AddNodeKind::Shell,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    fn shell(default_label: String) -> Self {
        Self {
            open: true,
            step: AddNodeStep::Label,
            kind: AddNodeKind::Shell,
            label: default_label,
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    fn typed(kind: AddNodeKind, default_label: String) -> Self {
        Self {
            open: true,
            step: AddNodeStep::Label,
            kind,
            label: default_label,
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    fn templates() -> Self {
        Self {
            open: true,
            step: AddNodeStep::TemplateSelect,
            kind: AddNodeKind::Shell,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsState {
    pub open: bool,
    pub diagnostics: Vec<WorkflowDiagnostic>,
}

impl DiagnosticsState {
    fn closed() -> Self {
        Self {
            open: false,
            diagnostics: Vec::new(),
        }
    }

    fn open(diagnostics: Vec<WorkflowDiagnostic>) -> Self {
        Self {
            open: true,
            diagnostics,
        }
    }
}

pub struct WorkflowBrowserState {
    pub open: bool,
    pub names: Vec<String>,
    pub selected: usize,
    pub message: Option<String>,
}

impl WorkflowBrowserState {
    fn closed() -> Self {
        Self {
            open: false,
            names: Vec::new(),
            selected: 0,
            message: None,
        }
    }

    pub fn open(names: Vec<String>, message: Option<String>) -> Self {
        Self {
            open: true,
            names,
            selected: 0,
            message,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunHistoryState {
    pub open: bool,
    pub records: Vec<RunRecord>,
    pub selected: usize,
    pub detail: Option<String>,
    pub detail_record: Option<RunDetailRecord>,
    pub selected_detail_node: usize,
    pub message: Option<String>,
}

impl RunHistoryState {
    fn closed() -> Self {
        Self {
            open: false,
            records: Vec::new(),
            selected: 0,
            detail: None,
            detail_record: None,
            selected_detail_node: 0,
            message: None,
        }
    }

    fn open(records: Vec<RunRecord>, message: Option<String>) -> Self {
        Self {
            open: true,
            records,
            selected: 0,
            detail: None,
            detail_record: None,
            selected_detail_node: 0,
            message,
        }
    }
}

impl PaletteData {
    fn new() -> Self {
        let items = vec![
            FuzzyCandidate {
                id: "add_node".into(),
                text: "Add Shell Node".into(),
                context: None,
                icon: Some('+'),
            },
            FuzzyCandidate {
                id: "edit_command".into(),
                text: "Edit Selected Node Command (e)".into(),
                context: None,
                icon: Some('✎'),
            },
            FuzzyCandidate {
                id: "edit_label".into(),
                text: "Rename Selected Node (l)".into(),
                context: None,
                icon: Some('✎'),
            },
            FuzzyCandidate {
                id: "edit_timeout".into(),
                text: "Edit Selected Node Timeout (t)".into(),
                context: None,
                icon: Some('✎'),
            },
            FuzzyCandidate {
                id: "run".into(),
                text: "Run Workflow (Ctrl-R)".into(),
                context: None,
                icon: Some('▶'),
            },
            FuzzyCandidate {
                id: "cancel_run".into(),
                text: "Cancel Run (Ctrl-X)".into(),
                context: None,
                icon: Some('■'),
            },
            FuzzyCandidate {
                id: "history".into(),
                text: "Run History (Ctrl-H)".into(),
                context: None,
                icon: Some('◷'),
            },
            FuzzyCandidate {
                id: "diagnostics".into(),
                text: "Workflow Diagnostics (Ctrl-D)".into(),
                context: None,
                icon: Some('!'),
            },
            FuzzyCandidate {
                id: "save".into(),
                text: "Save Workflow (Ctrl-S)".into(),
                context: None,
                icon: Some('💾'),
            },
            FuzzyCandidate {
                id: "open".into(),
                text: "Open Workflow (Ctrl-O)".into(),
                context: None,
                icon: Some('📂'),
            },
            FuzzyCandidate {
                id: "layout".into(),
                text: "Auto-Layout (Ctrl-L)".into(),
                context: None,
                icon: Some('⊞'),
            },
            FuzzyCandidate {
                id: "quit".into(),
                text: "Quit (Ctrl-Q)".into(),
                context: None,
                icon: Some('✕'),
            },
        ];
        Self { items }
    }
}

impl FuzzySource for PaletteData {
    fn candidates(&self) -> &[FuzzyCandidate] {
        &self.items
    }
}

/// Resolve a palette selection to an AppAction.
pub fn palette_action(id: &str) -> Option<AppAction> {
    match id {
        "add_node" => Some(AppAction::AddNode),
        "run" => Some(AppAction::RunWorkflow),
        "cancel_run" => Some(AppAction::CancelRun),
        "history" => Some(AppAction::OpenRunHistory),
        "diagnostics" => Some(AppAction::OpenDiagnostics),
        "save" => Some(AppAction::SaveWorkflow),
        "open" => Some(AppAction::OpenWorkflow),
        "layout" => Some(AppAction::AutoLayout),
        "edit_label" => Some(AppAction::EditNodeLabel),
        "edit_command" => Some(AppAction::EditNodeCommand),
        "edit_timeout" => Some(AppAction::EditNodeTimeout),
        "quit" => Some(AppAction::Quit),
        _ => None,
    }
}

/// Main application state.
pub struct App {
    pub workflow: Workflow,
    pub graph_state: NodeGraphState,
    pub focus: FocusPanel,
    pub show_palette: bool,
    pub palette_state: FuzzyState,
    pub palette_data: PaletteData,
    pub output: StreamingOutput,
    pub run_output_lines: Vec<RunOutputLine>,
    pub workflow_browser: WorkflowBrowserState,
    pub property_edit: Option<PropertyEditState>,
    pub add_node_flow: AddNodeFlowState,
    pub node_templates: Vec<NodeTemplate>,
    pub run_state: AppRunState,
    pub run_history: RunHistoryState,
    pub diagnostics: DiagnosticsState,
    pub store: Option<Store>,
    active_run: Option<ActiveRun>,
    next_tui_run_id: u64,
    pub config: AppConfig,
    pub last_drag_canvas: Option<(i32, i32)>,
    dirty: bool,
    confirm_quit: bool,
    executor_factory: Arc<dyn Fn() -> Executor + Send + Sync>,
}

impl App {
    /// Create a new app with an empty workflow.
    pub fn new(config: AppConfig) -> Self {
        let workflow = Workflow::new("untitled");
        let graph_state = NodeGraphState::new(workflow.graph.clone(), INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT);
        let agent_runtime = config.agent_runtime.clone();

        let store = match Store::new(config.data_dir.join("n8nclone.redb")) {
            Ok(s) => Some(s),
            Err(e) => {
                error!(error = %e, "failed to open database, running without persistence");
                None
            }
        };

        let next_tui_run_id = store
            .as_ref()
            .and_then(|store| store.next_run_id().ok())
            .map(|run_id| run_id.0)
            .unwrap_or(INITIAL_TUI_RUN_ID);
        let palette_state = FuzzyState::new();
        let palette_data = PaletteData::new();

        Self {
            workflow,
            graph_state,
            focus: FocusPanel::Canvas,
            show_palette: false,
            palette_state,
            palette_data,
            output: StreamingOutput::new(),
            run_output_lines: Vec::new(),
            workflow_browser: WorkflowBrowserState::closed(),
            property_edit: None,
            add_node_flow: AddNodeFlowState::closed(),
            node_templates: Vec::new(),
            run_state: AppRunState::idle(),
            run_history: RunHistoryState::closed(),
            diagnostics: DiagnosticsState::closed(),
            store,
            active_run: None,
            next_tui_run_id,
            config,
            last_drag_canvas: None,
            dirty: false,
            confirm_quit: false,
            executor_factory: Arc::new(move || Executor::new_with_agent_runtime_config(agent_runtime.clone())),
        }
    }

    #[cfg(test)]
    fn set_executor_factory(&mut self, factory: Arc<dyn Fn() -> Executor + Send + Sync>) {
        self.executor_factory = factory;
    }

    /// Return whether the workflow has unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        self.dirty
    }

    /// Return whether quit confirmation is currently active.
    pub fn is_confirming_quit(&self) -> bool {
        self.confirm_quit
    }

    /// Mark the current workflow as dirty after a user-visible mutation.
    pub fn mark_workflow_dirty(&mut self) {
        self.workflow.touch();
        self.dirty = true;
    }

    /// Clear dirty state after a successful save or clean workflow load.
    pub fn clear_workflow_dirty(&mut self) {
        self.dirty = false;
        self.confirm_quit = false;
    }

    /// Request quit. Returns true when the event loop may exit immediately.
    pub fn request_quit(&mut self) -> bool {
        if self.has_unsaved_changes() {
            self.confirm_quit = true;
            self.output.push_line("Unsaved changes. Press y/Enter to quit or n/Esc to stay.");
            false
        } else {
            true
        }
    }

    /// Cancel a pending quit confirmation.
    pub fn cancel_quit_confirmation(&mut self) {
        self.confirm_quit = false;
        self.output.push_line("Quit canceled");
    }

    /// Return whether the add-node flow is open.
    pub fn is_add_node_flow_open(&self) -> bool {
        self.add_node_flow.open
    }

    /// Return whether the diagnostics modal is open.
    pub fn is_diagnostics_open(&self) -> bool {
        self.diagnostics.open
    }

    /// Open workflow diagnostics for the current workflow snapshot.
    pub fn open_diagnostics(&mut self) {
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.add_node_flow = AddNodeFlowState::closed();
        self.run_history = RunHistoryState::closed();
        self.diagnostics = DiagnosticsState::open(workflow_validation::validate_workflow(&self.workflow));
    }

    /// Close the diagnostics modal without mutating workflow state.
    pub fn close_diagnostics(&mut self) {
        self.diagnostics.open = false;
    }

    /// Replace the configured node templates.
    pub fn set_node_templates(&mut self, templates: Vec<NodeTemplate>) {
        self.node_templates = templates;
    }

    /// Start the guided add-node flow.
    pub fn open_add_node_flow(&mut self) {
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.diagnostics.open = false;
        self.property_edit = None;
        self.add_node_flow = AddNodeFlowState::kind();
    }

    /// Cancel guided add-node state without mutating the workflow.
    pub fn cancel_add_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::closed();
    }

    /// Switch from template selection to manual shell-node entry.
    pub fn start_manual_shell_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::shell(self.next_shell_node_label());
    }

    pub fn start_template_node_flow(&mut self) {
        if self.node_templates.is_empty() {
            self.output.push_line("No node templates configured");
            return;
        }
        self.add_node_flow = AddNodeFlowState::templates();
    }

    pub fn start_image_input_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::ImageInput, self.next_image_node_label());
    }

    pub fn start_pi_agent_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::PiAgent, self.next_agent_node_label());
    }

    pub fn start_clankers_agent_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::ClankersAgent, self.next_agent_node_label());
    }

    /// Move add-node template selection down.
    pub fn select_next_node_template(&mut self) {
        if self.node_templates.is_empty() {
            return;
        }
        let last_index = self.node_templates.len() - 1;
        self.add_node_flow.selected_template = (self.add_node_flow.selected_template + 1).min(last_index);
    }

    /// Move add-node template selection up.
    pub fn select_previous_node_template(&mut self) {
        self.add_node_flow.selected_template = self.add_node_flow.selected_template.saturating_sub(1);
    }

    /// Append one character to the active add-node draft field.
    pub fn push_add_node_char(&mut self, ch: char) {
        match self.add_node_flow.step {
            AddNodeStep::Kind => {}
            AddNodeStep::Label => self.add_node_flow.label.push(ch),
            AddNodeStep::Command => self.add_node_flow.command.push(ch),
            AddNodeStep::TemplateParameter => {
                if let Some(value) = self.add_node_flow.parameter_values.get_mut(self.add_node_flow.parameter_index) {
                    value.push(ch);
                }
            }
            AddNodeStep::TemplateSelect => {}
        }
    }

    /// Remove one character from the active add-node draft field.
    pub fn pop_add_node_char(&mut self) {
        match self.add_node_flow.step {
            AddNodeStep::Kind => {}
            AddNodeStep::Label => {
                self.add_node_flow.label.pop();
            }
            AddNodeStep::Command => {
                self.add_node_flow.command.pop();
            }
            AddNodeStep::TemplateParameter => {
                if let Some(value) = self.add_node_flow.parameter_values.get_mut(self.add_node_flow.parameter_index) {
                    value.pop();
                }
            }
            AddNodeStep::TemplateSelect => {}
        }
    }

    /// Advance or finish the guided add-node flow.
    pub fn confirm_add_node_flow(&mut self) {
        match self.add_node_flow.step {
            AddNodeStep::Kind => self.start_manual_shell_node_flow(),
            AddNodeStep::TemplateSelect => self.confirm_template_node(),
            AddNodeStep::TemplateParameter => self.confirm_template_parameter(),
            AddNodeStep::Label => self.add_node_flow.step = AddNodeStep::Command,
            AddNodeStep::Command => self.confirm_typed_node_from_flow(),
        }
    }

    fn confirm_typed_node_from_flow(&mut self) {
        match self.add_node_flow.kind {
            AddNodeKind::Shell => self.confirm_shell_node_from_flow(),
            AddNodeKind::ImageInput => self.confirm_image_input_node_from_flow(),
            AddNodeKind::PiAgent => self.confirm_agent_node_from_flow(AgentTarget::Pi),
            AddNodeKind::ClankersAgent => self.confirm_agent_node_from_flow(AgentTarget::Clankers),
        }
    }

    fn confirm_shell_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, &self.next_shell_node_label());
        let command = self.add_node_flow.command.clone();
        self.add_shell_node(&label, &command);
        self.cancel_add_node_flow();
    }

    fn confirm_image_input_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, &self.next_image_node_label());
        let path = self.add_node_flow.command.clone();
        self.add_image_input_node(&label, &path);
        self.cancel_add_node_flow();
    }

    fn confirm_agent_node_from_flow(&mut self, target: AgentTarget) {
        let label = normalized_node_label(&self.add_node_flow.label, &self.next_agent_node_label());
        let prompt = self.add_node_flow.command.clone();
        self.add_agent_node(&label, target, &prompt, AgentOutputMode::Image);
        self.cancel_add_node_flow();
    }

    fn confirm_template_node(&mut self) {
        let Some(template) = self.node_templates.get(self.add_node_flow.selected_template).cloned() else {
            return;
        };
        if template.parameters.is_empty() {
            if self.add_template_node(&template) {
                self.cancel_add_node_flow();
            }
            return;
        }
        self.start_template_parameter_flow(&template);
    }

    fn start_template_parameter_flow(&mut self, template: &NodeTemplate) {
        self.add_node_flow.step = AddNodeStep::TemplateParameter;
        self.add_node_flow.parameter_index = 0;
        self.add_node_flow.parameter_values = template
            .parameters
            .iter()
            .map(|parameter| parameter.default.as_ref().map_or_else(String::new, |default| default.as_raw_string()))
            .collect();
    }

    fn confirm_template_parameter(&mut self) {
        let Some(template) = self.node_templates.get(self.add_node_flow.selected_template).cloned() else {
            return;
        };
        let last_parameter_index = template.parameters.len().saturating_sub(1);
        if self.add_node_flow.parameter_index < last_parameter_index {
            self.add_node_flow.parameter_index += 1;
            return;
        }
        if self.add_template_node(&template) {
            self.cancel_add_node_flow();
        }
    }

    fn add_template_node(&mut self, template: &NodeTemplate) -> bool {
        if template.kind != "shell" {
            self.output.push_line(&format!("Unsupported template kind: {}", template.kind));
            return false;
        }

        let inputs = template
            .input_ports
            .iter()
            .map(|port| PortSpec::new(port.label.clone(), port.type_tag.clone()))
            .collect::<Vec<_>>();
        let outputs = template
            .output_ports
            .iter()
            .map(|port| PortSpec::new(port.label.clone(), port.type_tag.clone()))
            .collect::<Vec<_>>();
        let parameter_values = template_parameter_values(template, &self.add_node_flow.parameter_values);
        let command = match render_template_command(template, &parameter_values) {
            Ok(command) => command,
            Err(error) => {
                self.output.push_line(&format!("Template parameter error: {error}"));
                return false;
            }
        };
        let id = self.workflow.graph.add_node(template.name.clone(), &inputs, &outputs);
        self.workflow.set_node_config(id, NodeConfig::shell(command));
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
        true
    }

    fn next_shell_node_label(&self) -> String {
        format!("Shell {}", self.workflow.graph.node_count() + 1)
    }

    fn next_image_node_label(&self) -> String {
        format!("Image {}", self.workflow.graph.node_count() + 1)
    }

    fn next_agent_node_label(&self) -> String {
        format!("Agent {}", self.workflow.graph.node_count() + 1)
    }

    /// Return whether the run-history modal is open.
    pub fn is_run_history_open(&self) -> bool {
        self.run_history.open
    }

    /// Open recent run history.
    pub fn open_run_history(&mut self) {
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.diagnostics.open = false;
        self.add_node_flow = AddNodeFlowState::closed();
        let Some(store) = &self.store else {
            let message = "No database available".to_string();
            self.output.push_line(&message);
            self.run_history = RunHistoryState::open(Vec::new(), Some(message));
            return;
        };

        match store.list_runs(RUN_HISTORY_LIMIT) {
            Ok(records) => {
                let message = if records.is_empty() {
                    Some("No run history available".to_string())
                } else {
                    None
                };
                if let Some(message) = &message {
                    self.output.push_line(message);
                }
                self.run_history = RunHistoryState::open(records, message);
            }
            Err(e) => {
                let message = format!("Run history failed: {e}");
                self.output.push_line(&message);
                self.run_history = RunHistoryState::open(Vec::new(), Some(message));
            }
        }
    }

    pub fn close_run_history(&mut self) {
        self.run_history = RunHistoryState::closed();
    }

    pub fn select_next_run_history(&mut self) {
        if self.run_history.records.is_empty() {
            return;
        }
        let last_index = self.run_history.records.len() - 1;
        if let Some(detail_record) = &self.run_history.detail_record {
            let last_node_index = detail_record.nodes.len().saturating_sub(1);
            self.run_history.selected_detail_node = (self.run_history.selected_detail_node + 1).min(last_node_index);
            return;
        }
        self.run_history.selected = (self.run_history.selected + 1).min(last_index);
        self.run_history.detail = None;
    }

    pub fn select_previous_run_history(&mut self) {
        if self.run_history.detail_record.is_some() {
            self.run_history.selected_detail_node = self.run_history.selected_detail_node.saturating_sub(1);
            return;
        }
        self.run_history.selected = self.run_history.selected.saturating_sub(1);
        self.run_history.detail = None;
    }

    pub fn inspect_selected_run_history(&mut self) {
        let Some(record) = self.run_history.records.get(self.run_history.selected) else {
            return;
        };
        self.run_history.detail = Some(format_run_record(record));
        self.run_history.selected_detail_node = 0;

        let Some(store) = &self.store else {
            let message = "No database available".to_string();
            self.run_history.detail = Some(message.clone());
            self.run_history.message = Some(message);
            self.output.push_line("No database available");
            return;
        };
        match store.load_run_details(record.run_id) {
            Ok(Some(detail_record)) => self.run_history.detail_record = Some(detail_record),
            Ok(None) => {
                let message = "No run details available".to_string();
                self.run_history.detail_record = None;
                self.run_history.detail = Some(message.clone());
                self.run_history.message = Some(message);
            }
            Err(error) => {
                let message = format!("Run details failed: {error}");
                self.output.push_line(&message);
                self.run_history.detail_record = None;
                self.run_history.detail = Some(message.clone());
                self.run_history.message = Some(message);
            }
        }
    }

    pub fn rerun_selected_run_history(&mut self) -> bool {
        let Some(record) = self.run_history.records.get(self.run_history.selected).cloned() else {
            return false;
        };
        let Some(store) = &self.store else {
            self.output.push_line("No database available");
            return false;
        };
        let workflow = match store.load_workflow(&record.workflow_name) {
            Ok(Some(workflow)) => workflow,
            Ok(None) => {
                self.output.push_line(&format!("Workflow not found: {}", record.workflow_name));
                return false;
            }
            Err(error) => {
                self.output.push_line(&format!("Rerun failed: {error}"));
                return false;
            }
        };
        self.start_workflow_snapshot(workflow, "▶ Rerunning workflow from history…")
    }

    /// Return whether the workflow browser modal is open.
    pub fn is_workflow_browser_open(&self) -> bool {
        self.workflow_browser.open
    }

    /// Open the saved-workflow browser.
    pub fn open_workflow_browser(&mut self) {
        self.show_palette = false;
        self.diagnostics.open = false;
        let Some(store) = &self.store else {
            let message = "No database available".to_string();
            self.output.push_line(&message);
            self.workflow_browser = WorkflowBrowserState::open(Vec::new(), Some(message));
            return;
        };

        match store.list_workflows() {
            Ok(mut names) => {
                names.sort();
                let message = if names.is_empty() {
                    Some("No saved workflows available".to_string())
                } else {
                    None
                };
                if let Some(message) = &message {
                    self.output.push_line(message);
                }
                self.workflow_browser = WorkflowBrowserState::open(names, message);
            }
            Err(e) => {
                let message = format!("Workflow browser failed: {e}");
                self.output.push_line(&message);
                self.workflow_browser = WorkflowBrowserState::open(Vec::new(), Some(message));
            }
        }
    }

    /// Close the saved-workflow browser.
    pub fn close_workflow_browser(&mut self) {
        self.workflow_browser = WorkflowBrowserState::closed();
    }

    /// Move browser selection to the next saved workflow.
    pub fn select_next_workflow(&mut self) {
        if self.workflow_browser.names.is_empty() {
            return;
        }
        let last_index = self.workflow_browser.names.len() - 1;
        self.workflow_browser.selected = (self.workflow_browser.selected + 1).min(last_index);
    }

    /// Move browser selection to the previous saved workflow.
    pub fn select_previous_workflow(&mut self) {
        self.workflow_browser.selected = self.workflow_browser.selected.saturating_sub(1);
    }

    /// Confirm the selected workflow in the browser.
    pub fn confirm_workflow_browser_selection(&mut self) {
        let Some(name) = self.workflow_browser.names.get(self.workflow_browser.selected).cloned() else {
            return;
        };

        if self.load_workflow_by_name(&name) {
            self.close_workflow_browser();
        }
    }

    /// Load a saved workflow by name, preserving the current workflow on failure.
    pub fn load_workflow_by_name(&mut self, name: &str) -> bool {
        let Some(store) = &self.store else {
            self.output.push_line("No database available");
            return false;
        };

        match store.load_workflow(name) {
            Ok(Some(workflow)) => {
                self.workflow = workflow;
                self.sync_graph_state();
                self.clear_workflow_dirty();
                self.output.push_line(&format!("Opened workflow '{name}'"));
                true
            }
            Ok(None) => {
                self.output.push_line(&format!("Workflow not found: {name}"));
                false
            }
            Err(e) => {
                self.output.push_line(&format!("Open failed: {e}"));
                false
            }
        }
    }

    /// Start editing a selected shell-node property.
    // r[impl n8nclone.tui_ux.node_kind_provider.unsupported_edit]
    pub fn start_property_edit(&mut self, field: PropertyEditField) -> bool {
        let Some(node_id) = self.selected_node_id() else {
            self.output.push_line("Select one node to edit properties");
            return false;
        };

        let Some(draft) = self.property_value(node_id, field) else {
            self.output.push_line("Selected node property is unavailable");
            return false;
        };

        self.property_edit = Some(PropertyEditState {
            node_id,
            field,
            original: draft.clone(),
            draft,
        });
        true
    }

    /// Append one character to the active property draft.
    pub fn push_property_edit_char(&mut self, ch: char) {
        if let Some(edit) = &mut self.property_edit {
            edit.draft.push(ch);
        }
    }

    /// Remove one character from the active property draft.
    pub fn pop_property_edit_char(&mut self) {
        if let Some(edit) = &mut self.property_edit {
            edit.draft.pop();
        }
    }

    /// Cancel active property editing without mutating workflow state.
    pub fn cancel_property_edit(&mut self) {
        self.property_edit = None;
    }

    /// Save active property editing, preserving previous state on validation failure.
    pub fn save_property_edit(&mut self) -> bool {
        let Some(edit) = self.property_edit.clone() else {
            return false;
        };

        match self.apply_property_edit(&edit) {
            Ok(()) => {
                self.property_edit = None;
                self.mark_workflow_dirty();
                self.output.push_line(&format!("Updated node {}", edit.field.label()));
                true
            }
            Err(PropertyEditError::InvalidTimeout) => {
                self.output.push_line("Invalid timeout; enter whole seconds");
                false
            }
            Err(PropertyEditError::InvalidAgentTarget) => {
                self.output.push_line("Invalid agent target; enter pi or clankers");
                false
            }
            Err(PropertyEditError::InvalidAgentOutputMode) => {
                self.output.push_line("Invalid agent output mode; enter text, json, image, or artifact");
                false
            }
            Err(PropertyEditError::InvalidAgentCredentials) => {
                self.output.push_line("Invalid agent credentials; use ENV=provider:key[, ENV=provider:key.field]");
                false
            }
            Err(_) => {
                self.output.push_line("Property edit failed");
                false
            }
        }
    }

    fn selected_node_id(&self) -> Option<NodeId> {
        if self.graph_state.selected.len() != 1 {
            return None;
        }
        self.graph_state.selected.iter().next().copied()
    }

    fn property_value(&self, node_id: NodeId, field: PropertyEditField) -> Option<String> {
        match field {
            PropertyEditField::Label => self.workflow.graph.node(node_id).map(|node| node.label.clone()),
            PropertyEditField::Command => match self.workflow.node_config(node_id)? {
                NodeConfig::Shell { command, .. } => Some(command.clone()),
                NodeConfig::Http { .. }
                | NodeConfig::File { .. }
                | NodeConfig::ImageInput { .. }
                | NodeConfig::Agent(_) => None,
            },
            PropertyEditField::TimeoutSecs => match self.workflow.node_config(node_id)? {
                NodeConfig::Shell { timeout_secs, .. } | NodeConfig::Http { timeout_secs, .. } => {
                    Some(timeout_secs.to_string())
                }
                NodeConfig::File { .. } | NodeConfig::ImageInput { .. } | NodeConfig::Agent(_) => None,
            },
            PropertyEditField::ImagePath => match self.workflow.node_config(node_id)? {
                NodeConfig::ImageInput { path, .. } => Some(path.clone()),
                _ => None,
            },
            PropertyEditField::AgentPrompt => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.prompt.clone()),
                _ => None,
            },
            PropertyEditField::AgentTarget => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(agent_node::agent_target_label(settings.target).to_string()),
                _ => None,
            },
            PropertyEditField::AgentOutputMode => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.output_mode.as_str().to_string()),
                _ => None,
            },
            PropertyEditField::AgentModelHint => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.model_hint.clone().unwrap_or_default()),
                _ => None,
            },
            PropertyEditField::AgentToolPolicy => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.tool_policy.clone()),
                _ => None,
            },
            PropertyEditField::AgentContextRefs => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.context_refs.join(", ")),
                _ => None,
            },
            PropertyEditField::AgentCwd => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.cwd.clone().unwrap_or_default()),
                _ => None,
            },
            PropertyEditField::AgentSession => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.session.clone().unwrap_or_default()),
                _ => None,
            },
            PropertyEditField::AgentCredentials => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(_) => Some(credentials_to_draft(self.workflow.node_credentials(node_id))),
                _ => None,
            },
        }
    }

    fn apply_property_edit(&mut self, edit: &PropertyEditState) -> Result<(), PropertyEditError> {
        match edit.field {
            PropertyEditField::Label => {
                let Some(node) = self.workflow.graph.node_mut(edit.node_id) else {
                    return Err(PropertyEditError::NodeNotFound);
                };
                node.label = edit.draft.clone();
                self.sync_graph_state();
                self.graph_state.selected.insert(edit.node_id);
                self.graph_state.focused = Some(edit.node_id);
                Ok(())
            }
            PropertyEditField::Command => {
                let Some(config) = self.workflow.node_config(edit.node_id).cloned() else {
                    return Err(PropertyEditError::MissingConfig);
                };
                match config {
                    NodeConfig::Shell {
                        cwd,
                        env,
                        timeout_secs,
                        pipe_stdin,
                        ..
                    } => {
                        self.workflow.set_node_config(edit.node_id, NodeConfig::Shell {
                            command: edit.draft.clone(),
                            cwd,
                            env,
                            timeout_secs,
                            pipe_stdin,
                        });
                        Ok(())
                    }
                    NodeConfig::Http { .. }
                    | NodeConfig::File { .. }
                    | NodeConfig::ImageInput { .. }
                    | NodeConfig::Agent(_) => Err(PropertyEditError::MissingConfig),
                }
            }
            PropertyEditField::TimeoutSecs => {
                let timeout_secs = validate_timeout_secs(&edit.draft)?;
                let Some(config) = self.workflow.node_config(edit.node_id).cloned() else {
                    return Err(PropertyEditError::MissingConfig);
                };
                match config {
                    NodeConfig::Shell {
                        command,
                        cwd,
                        env,
                        pipe_stdin,
                        ..
                    } => {
                        self.workflow.set_node_config(edit.node_id, NodeConfig::Shell {
                            command,
                            cwd,
                            env,
                            timeout_secs,
                            pipe_stdin,
                        });
                        Ok(())
                    }
                    NodeConfig::Http {
                        method,
                        url,
                        headers,
                        body,
                        ..
                    } => {
                        self.workflow.set_node_config(edit.node_id, NodeConfig::Http {
                            method,
                            url,
                            headers,
                            body,
                            timeout_secs,
                        });
                        Ok(())
                    }
                    NodeConfig::File { .. } | NodeConfig::ImageInput { .. } | NodeConfig::Agent(_) => {
                        Err(PropertyEditError::MissingConfig)
                    }
                }
            }
            PropertyEditField::ImagePath => {
                let Some(config) = self.workflow.node_config(edit.node_id).cloned() else {
                    return Err(PropertyEditError::MissingConfig);
                };
                match config {
                    NodeConfig::ImageInput {
                        allowed_roots,
                        max_bytes,
                        ..
                    } => {
                        self.workflow.set_node_config(edit.node_id, NodeConfig::ImageInput {
                            path: edit.draft.clone(),
                            allowed_roots,
                            max_bytes,
                        });
                        Ok(())
                    }
                    _ => Err(PropertyEditError::MissingConfig),
                }
            }
            PropertyEditField::AgentPrompt
            | PropertyEditField::AgentTarget
            | PropertyEditField::AgentOutputMode
            | PropertyEditField::AgentModelHint
            | PropertyEditField::AgentToolPolicy
            | PropertyEditField::AgentContextRefs
            | PropertyEditField::AgentCwd
            | PropertyEditField::AgentSession => {
                let Some(NodeConfig::Agent(mut settings)) = self.workflow.node_config(edit.node_id).cloned() else {
                    return Err(PropertyEditError::MissingConfig);
                };
                match edit.field {
                    PropertyEditField::AgentPrompt => settings.prompt = edit.draft.clone(),
                    PropertyEditField::AgentTarget => settings.target = validate_agent_target(&edit.draft)?,
                    PropertyEditField::AgentOutputMode => {
                        settings.output_mode = validate_agent_output_mode(&edit.draft)?;
                    }
                    PropertyEditField::AgentModelHint => settings.model_hint = optional_draft_value(&edit.draft),
                    PropertyEditField::AgentToolPolicy => settings.tool_policy = edit.draft.clone(),
                    PropertyEditField::AgentContextRefs => settings.context_refs = context_refs_from_draft(&edit.draft),
                    PropertyEditField::AgentCwd => settings.cwd = optional_draft_value(&edit.draft),
                    PropertyEditField::AgentSession => settings.session = optional_draft_value(&edit.draft),
                    PropertyEditField::Label
                    | PropertyEditField::Command
                    | PropertyEditField::TimeoutSecs
                    | PropertyEditField::ImagePath
                    | PropertyEditField::AgentCredentials => return Err(PropertyEditError::MissingConfig),
                }
                self.workflow.set_node_config(edit.node_id, NodeConfig::Agent(settings));
                Ok(())
            }
            PropertyEditField::AgentCredentials => match self.workflow.node_config(edit.node_id) {
                Some(NodeConfig::Agent(_)) => {
                    self.workflow.set_node_credentials(edit.node_id, credentials_from_draft(&edit.draft)?);
                    Ok(())
                }
                _ => Err(PropertyEditError::MissingConfig),
            },
        }
    }

    /// Cycle focus to the next panel.
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Canvas => FocusPanel::Properties,
            FocusPanel::Properties => FocusPanel::Output,
            FocusPanel::Output => FocusPanel::Canvas,
        };
    }

    /// Add a new shell node to the workflow.
    pub fn add_shell_node(&mut self, label: &str, command: &str) {
        let id = self.workflow.add_shell_node(label, command);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    pub fn add_image_input_node(&mut self, label: &str, path: &str) {
        let id = self.workflow.add_image_input_node(label, path);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    pub fn add_agent_node(&mut self, label: &str, target: AgentTarget, prompt: &str, output_mode: AgentOutputMode) {
        let id = self.workflow.add_agent_node(label, target, prompt, output_mode);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    fn place_new_node(&mut self, id: NodeId) {
        // Position the new node so it doesn't overlap existing ones.
        // Find the rightmost edge of all existing nodes, then place after a gap.
        let mut max_right: i32 = 0;
        let mut stack_y: i32 = 0;
        for node in self.workflow.graph.nodes() {
            if node.id == id {
                continue;
            }
            if let Some(bounds) = node_bounds(&self.workflow.graph, node.id) {
                let right = bounds.x + bounds.width as i32;
                if right > max_right {
                    max_right = right;
                    stack_y = bounds.y;
                }
            }
        }
        let has_others = self.workflow.graph.node_count() > 1;
        if let Some(node) = self.workflow.graph.node_mut(id)
            && has_others
        {
            node.x = max_right + NEW_NODE_HORIZONTAL_GAP;
            node.y = stack_y;
        }
    }

    fn select_node(&mut self, id: NodeId) {
        self.graph_state.selected.clear();
        self.graph_state.selected.insert(id);
        self.graph_state.focused = Some(id);
    }

    /// Run auto-layout on the graph.
    pub fn auto_layout(&mut self) {
        auto_layout(&mut self.workflow.graph, &LayoutConfig::default());
        self.sync_graph_state();
        self.mark_workflow_dirty();
        self.output.push_line("Auto-layout applied");
    }

    /// Start the current workflow in the background.
    pub fn start_workflow_run(&mut self) -> bool {
        if self.run_state.active {
            self.output.push_line("Workflow already running");
            return false;
        }

        let diagnostics = workflow_validation::validate_workflow(&self.workflow);
        self.diagnostics.diagnostics = diagnostics.clone();
        self.diagnostics.open = false;
        if workflow_validation::has_fatal_diagnostics(&diagnostics) {
            self.diagnostics = DiagnosticsState::open(diagnostics.clone());
            self.output.push_line("Workflow validation failed; open diagnostics for details");
            for diagnostic in workflow_validation::fatal_diagnostics(&diagnostics) {
                self.output.push_line(&workflow_validation::format_diagnostic(&diagnostic));
            }
            return false;
        }
        for diagnostic in &diagnostics {
            self.output.push_line(&workflow_validation::format_diagnostic(diagnostic));
        }

        let workflow = self.workflow.clone();
        self.start_workflow_snapshot(workflow, "▶ Running workflow…")
    }

    fn start_workflow_snapshot(&mut self, workflow: Workflow, message: &str) -> bool {
        if self.run_state.active {
            self.output.push_line("Workflow already running");
            return false;
        }

        let run_id = RunId(self.next_tui_run_id);
        self.next_tui_run_id += 1;
        let (sender, receiver) = mpsc::unbounded_channel();
        let node_statuses =
            workflow.graph.nodes().map(|node| (node.id.0, NodeRunStatus::Pending)).collect::<HashMap<_, _>>();

        self.run_state = AppRunState {
            active: true,
            node_statuses,
        };
        self.output.push_line(message);

        let active_workflow = workflow.clone();
        let executor_factory = self.executor_factory.clone();
        let handle = tokio::spawn(async move {
            let mut executor = executor_factory();
            let cache = HashMap::new();
            let (_executor_run_id, results) = executor.run(&workflow, &cache).await;
            let _ = sender.send(RunEvent::Completed {
                run_id,
                workflow,
                results,
            });
        });
        self.active_run = Some(ActiveRun {
            run_id,
            workflow: active_workflow,
            receiver,
            handle,
        });
        true
    }

    /// Cancel the active background run, if any.
    pub fn cancel_active_run(&mut self) -> bool {
        let Some(active_run) = self.active_run.take() else {
            self.output.push_line("No active workflow run");
            return false;
        };

        active_run.handle.abort();
        for status in self.run_state.node_statuses.values_mut() {
            if matches!(*status, NodeRunStatus::Pending | NodeRunStatus::Running) {
                *status = NodeRunStatus::Skipped;
            }
        }
        self.run_state.active = false;
        let workflow_name = active_run.workflow.meta.name.clone();
        self.save_run_record(active_run.run_id, &workflow_name, 0, 0, self.run_state.node_statuses.len());
        self.save_run_detail_record(canceled_run_detail_record(
            active_run.run_id,
            &active_run.workflow,
            &self.run_state.node_statuses,
        ));
        self.output.push_line("Workflow run canceled");
        true
    }

    /// Poll background run events and apply completed results.
    pub fn poll_run_events(&mut self) {
        let mut completed = Vec::new();
        if let Some(active_run) = &mut self.active_run {
            while let Ok(event) = active_run.receiver.try_recv() {
                completed.push(event);
            }
        }

        for event in completed {
            match event {
                RunEvent::Completed {
                    run_id,
                    workflow,
                    results,
                } => {
                    self.finish_workflow_run(run_id, &workflow, results);
                    self.active_run = None;
                }
            }
        }
    }

    fn finish_workflow_run(&mut self, run_id: RunId, run_workflow: &Workflow, results: Vec<StepResult>) {
        for step in &results {
            let node_label = run_workflow.graph.node(step.node_id).map(|n| n.label.as_str()).unwrap_or("?");
            match &step.status {
                NodeStatus::Completed { outputs, cached, .. } => {
                    self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Completed);
                    let cache_tag = if *cached { " (cached)" } else { "" };
                    self.output.push_line(&format!("✓ {node_label}{cache_tag} [{}]", step.input_hash));

                    for record in completed_output_records(run_id, run_workflow, step.node_id, outputs) {
                        self.output.push_line(&record.rendered());
                        self.run_output_lines.push(record);
                    }

                    if let Some(ref store) = self.store {
                        let _ = store.put_cache(&step.input_hash, outputs);
                    }
                }
                NodeStatus::Failed { error } => {
                    self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Failed);
                    self.output.push_line(&format!("✗ {node_label}: {error}"));
                }
                NodeStatus::Skipped => {
                    self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Skipped);
                }
                NodeStatus::Pending | NodeStatus::Running => {
                    self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Running);
                }
            }
        }

        self.run_state.active = false;
        let failures = results.iter().filter(|step| matches!(step.status, NodeStatus::Failed { .. })).count();
        let cache_hits = results
            .iter()
            .filter(|step| matches!(step.status, NodeStatus::Completed { cached: true, .. }))
            .count();
        self.save_run_record(run_id, &run_workflow.meta.name, cache_hits, failures, results.len());
        self.save_run_detail_record(run_detail_record_from_results(run_id, run_workflow, &results));
        self.output.push_line(&format!("── run {} complete ({} steps) ──", run_id, results.len()));
    }

    fn save_run_record(
        &mut self,
        run_id: RunId,
        workflow_name: &str,
        cache_hits: usize,
        failures: usize,
        node_count: usize,
    ) {
        let Some(store) = &self.store else {
            return;
        };
        let now = chrono::Utc::now().timestamp_millis();
        let record = RunRecord {
            run_id,
            workflow_name: workflow_name.to_string(),
            started_epoch_ms: now,
            finished_epoch_ms: Some(now),
            node_count,
            cache_hits,
            failures,
        };
        if let Err(e) = store.save_run(&record) {
            self.output.push_line(&format!("Run history save failed: {e}"));
        }
    }

    fn save_run_detail_record(&mut self, record: RunDetailRecord) {
        let Some(store) = &self.store else {
            return;
        };
        if let Err(error) = store.save_run_details(&record) {
            self.output.push_line(&format!("Run details save failed: {error}"));
        }
    }

    /// Save the current workflow to the store.
    pub fn save_workflow(&mut self) {
        if let Some(ref store) = self.store {
            match store.save_workflow(&self.workflow) {
                Ok(()) => {
                    self.clear_workflow_dirty();
                    self.output.push_line(&format!("Saved '{}'", self.workflow.meta.name));
                }
                Err(e) => {
                    self.output.push_line(&format!("Save failed: {e}"));
                }
            }
        } else {
            self.output.push_line("No database available");
        }
    }

    /// Rebuild graph_state from workflow.graph.
    fn sync_graph_state(&mut self) {
        let old_viewport = self.graph_state.viewport.clone();
        let mut graph = self.workflow.graph.clone();
        // Re-apply the "any" type compatibility function (lost on clone).
        graph.set_compatibility(Box::new(|src, tgt| src == tgt || src == "any" || tgt == "any"));
        self.graph_state = NodeGraphState::new(graph, old_viewport.width, old_viewport.height);
        self.graph_state.viewport = old_viewport;
    }

    /// Run the TUI event loop.
    pub async fn run_tui(mut self) -> io::Result<()> {
        // Terminal setup.
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        crossterm::execute!(stdout, event::EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        info!("TUI started");

        loop {
            self.poll_run_events();

            // Render.
            terminal.draw(|frame| {
                panels::render(&mut self, frame.area(), frame.buffer_mut());
            })?;

            // Poll for events (16ms → ~60fps).
            if event::poll(std::time::Duration::from_millis(16))? {
                let ev = event::read()?;
                if let Some(action) = input::handle_event(&mut self, &ev) {
                    match action {
                        AppAction::Quit => {
                            if self.request_quit() {
                                break;
                            }
                        }
                        AppAction::ConfirmQuit => break,
                        AppAction::CancelQuit => {
                            self.cancel_quit_confirmation();
                        }
                        AppAction::RunWorkflow => {
                            self.start_workflow_run();
                        }
                        AppAction::CancelRun => {
                            self.cancel_active_run();
                        }
                        AppAction::OpenRunHistory => {
                            self.open_run_history();
                        }
                        AppAction::OpenDiagnostics => {
                            self.open_diagnostics();
                        }
                        AppAction::SaveWorkflow => {
                            self.save_workflow();
                        }
                        AppAction::AddNode => {
                            self.open_add_node_flow();
                        }
                        AppAction::OpenWorkflow => {
                            self.open_workflow_browser();
                        }
                        AppAction::AutoLayout => {
                            self.auto_layout();
                        }
                        AppAction::EditNodeLabel => {
                            self.start_property_edit(PropertyEditField::Label);
                        }
                        AppAction::EditNodeCommand => {
                            self.start_property_edit(PropertyEditField::Command);
                        }
                        AppAction::EditNodeTimeout => {
                            self.start_property_edit(PropertyEditField::TimeoutSecs);
                        }
                    }
                }
            }
        }

        // Cleanup.
        crossterm::execute!(terminal.backend_mut(), event::DisableMouseCapture)?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;

        info!("TUI exited");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rat_streaming::streaming_output::DisplayLine;
    use tempfile::TempDir;

    use super::*;
    use crate::agent_node::AgentAdapter;
    use crate::agent_node::AgentError;
    use crate::agent_node::AgentRequest;
    use crate::agent_node::AgentResponse;
    use crate::config::PortTemplate;
    use crate::config::TemplateParameter;
    use crate::config::TemplateParameterDefault;
    use crate::config::TemplateParameterType;
    use crate::engine::value::ContentHash;
    use crate::engine::value::ImageMetadata;
    use crate::engine::value::ImageValue;
    use crate::image_node::MemoryImageIo;

    const DEFAULT_SHELL_TIMEOUT_SECS: u64 = 60;
    const RUN_EVENT_POLL_LIMIT: usize = 50;
    const RUN_EVENT_POLL_DELAY_MS: u64 = 10;
    const SHELL_STDOUT_OUTPUT_INDEX: usize = 0;
    const SHELL_STDERR_OUTPUT_INDEX: usize = 1;
    const SHELL_EXIT_OUTPUT_INDEX: usize = 2;
    const AGENT_IMAGE_OUTPUT_INDEX: usize = 2;
    const EXIT_CODE_SUCCESS: i32 = 0;
    const EXIT_CODE_GREP_NO_MATCH: i32 = 1;
    const SNAPSHOT_RUN_ID: u64 = 0;
    const SNAPSHOT_WORKFLOW_NAME: &str = "snapshot";
    const SNAPSHOT_NODE_LABEL: &str = "Snapshot label";
    const MUTATED_NODE_LABEL: &str = "Mutated label";
    const SNAPSHOT_COMMAND: &str = "echo snapshot";
    const SNAPSHOT_STDOUT_TEXT: &str = "snapshot output";
    const SNAPSHOT_STDOUT: &str = "snapshot output\n";
    const SNAPSHOT_INPUT_HASH_BYTES: &[u8] = b"snapshot-log";
    const SNAPSHOT_OUTPUT_RECORD_COUNT: usize = 2;
    const SNAPSHOT_SECOND_RUN_ID: u64 = 1;
    const OLDER_STDOUT_TEXT: &str = "older output";
    const OTHER_NODE_TEXT: &str = "other node output";
    const TEMPLATE_PARAMETER_COUNT: usize = 1;
    const TEMPLATE_FIRST_PARAMETER_INDEX: usize = 0;
    const TEMPLATE_NUMBER_DEFAULT: f64 = 10.0;
    const TEMPLATE_NUMBER_MIN: f64 = 1.0;
    const TEMPLATE_NUMBER_MAX: f64 = 10.0;
    const IMAGE_PREVIEW_BYTES: &[u8] = b"raw-image-bytes";
    const TUI_LOCAL_IMAGE_FILE_NAME: &str = "tui-selfie.png";
    const TUI_LOCAL_PNG_FIXTURE: &[u8] = b"\x89PNG\r\n\x1a\ntui";
    const TUI_MISSING_IMAGE_PATH: &str = "/tui/missing.png";
    const TUI_BAD_IMAGE_PATH: &str = "/tui/bad.png";
    const TUI_BAD_IMAGE_BYTES: &[u8] = b"not an image";

    #[derive(Debug)]
    struct TuiErrorAgentAdapter {
        error: AgentError,
    }

    impl AgentAdapter for TuiErrorAgentAdapter {
        fn execute(&self, _request: &AgentRequest) -> Result<AgentResponse, AgentError> {
            Err(self.error.clone())
        }
    }

    fn test_config(temp_dir: &TempDir) -> AppConfig {
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir).expect("create app data directory");
        AppConfig {
            data_dir,
            ..AppConfig::default()
        }
    }

    fn output_text_lines(app: &App) -> Vec<String> {
        (0..app.output.display_line_count())
            .filter_map(|line_index| match app.output.get_display_line(line_index) {
                DisplayLine::Text(line) => Some(line),
                DisplayLine::Omitted(_) => None,
            })
            .collect()
    }

    fn image_executor_factory(files: HashMap<String, Vec<u8>>) -> Arc<dyn Fn() -> Executor + Send + Sync> {
        Arc::new(move || Executor::new_with_image_io(Box::new(MemoryImageIo::new(files.clone()))))
    }

    fn agent_error_executor_factory(error: AgentError) -> Arc<dyn Fn() -> Executor + Send + Sync> {
        Arc::new(move || Executor::new_with_agent_adapter(Box::new(TuiErrorAgentAdapter { error: error.clone() })))
    }

    async fn drain_run(app: &mut App) {
        for _ in 0..RUN_EVENT_POLL_LIMIT {
            app.poll_run_events();
            if !app.run_state.active {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(RUN_EVENT_POLL_DELAY_MS)).await;
        }
    }

    fn parameterized_grep_template() -> NodeTemplate {
        NodeTemplate {
            name: "grep".to_string(),
            description: "Filter text".to_string(),
            kind: "shell".to_string(),
            command: Some("grep -E '${pattern}'".to_string()),
            parameters: vec![TemplateParameter {
                name: "pattern".to_string(),
                type_tag: TemplateParameterType::String,
                description: "Search pattern".to_string(),
                default: None,
                required: true,
                min: None,
                max: None,
            }],
            input_ports: vec![PortTemplate {
                label: "stdin".to_string(),
                type_tag: "string".to_string(),
            }],
            output_ports: vec![PortTemplate {
                label: "stdout".to_string(),
                type_tag: "string".to_string(),
            }],
        }
    }

    fn rendered_output_records(records: Vec<RunOutputLine>) -> Vec<String> {
        records.into_iter().map(|record| record.rendered()).collect()
    }

    #[test]
    fn mutation_marks_workflow_dirty() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        assert!(!app.has_unsaved_changes());
        app.add_shell_node("Example", "echo example");

        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn successful_save_clears_dirty_state() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Example", "echo example");

        assert!(app.has_unsaved_changes());
        app.save_workflow();

        assert!(!app.has_unsaved_changes());
        assert!(!app.is_confirming_quit());
    }

    #[test]
    fn failed_save_keeps_dirty_state() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Example", "echo example");
        app.store = None;

        app.save_workflow();

        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn dirty_quit_requests_confirmation() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Example", "echo example");

        assert!(!app.request_quit());
        assert!(app.is_confirming_quit());

        app.cancel_quit_confirmation();
        assert!(!app.is_confirming_quit());
    }

    #[test]
    fn clean_quit_exits_without_confirmation() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        assert!(app.request_quit());
        assert!(!app.is_confirming_quit());
    }

    #[test]
    fn workflow_browser_lists_saved_workflows() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.workflow.meta.name = "saved".to_string();
        app.save_workflow();

        app.open_workflow_browser();

        assert!(app.is_workflow_browser_open());
        assert_eq!(app.workflow_browser.names, vec!["saved".to_string()]);
        assert_eq!(app.workflow_browser.selected, 0);
    }

    #[test]
    fn workflow_browser_reports_empty_store() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        app.open_workflow_browser();

        assert!(app.is_workflow_browser_open());
        assert!(app.workflow_browser.names.is_empty());
        assert_eq!(app.workflow_browser.message.as_deref(), Some("No saved workflows available"),);
    }

    #[test]
    fn workflow_browser_loads_selected_workflow() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.workflow.meta.name = "saved".to_string();
        app.add_shell_node("Saved", "echo saved");
        app.save_workflow();
        app.workflow = Workflow::new("current");
        app.sync_graph_state();

        app.open_workflow_browser();
        app.confirm_workflow_browser_selection();

        assert_eq!(app.workflow.meta.name, "saved");
        assert_eq!(app.workflow.graph.node_count(), 1);
        assert!(!app.is_workflow_browser_open());
        assert!(!app.has_unsaved_changes());
    }

    #[test]
    fn workflow_browser_load_failure_preserves_current_workflow() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.workflow = Workflow::new("current");
        app.sync_graph_state();

        assert!(!app.load_workflow_by_name("missing"));

        assert_eq!(app.workflow.meta.name, "current");
        assert_eq!(app.workflow.graph.node_count(), 0);
    }

    #[test]
    fn property_edit_updates_shell_command() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Example", "echo old");
        app.clear_workflow_dirty();
        let node_id = app.selected_node_id().expect("selected shell node");

        assert!(app.start_property_edit(PropertyEditField::Command));
        app.property_edit.as_mut().expect("property edit").draft = "echo new".to_string();
        assert!(app.save_property_edit());

        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "echo new"),
            _ => panic!("expected shell config"),
        }
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn property_edit_rejects_invalid_timeout_without_mutating() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Example", "echo example");
        app.clear_workflow_dirty();
        let node_id = app.selected_node_id().expect("selected shell node");

        assert!(app.start_property_edit(PropertyEditField::TimeoutSecs));
        app.property_edit.as_mut().expect("property edit").draft = "not seconds".to_string();
        assert!(!app.save_property_edit());

        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { timeout_secs, .. } => {
                assert_eq!(*timeout_secs, DEFAULT_SHELL_TIMEOUT_SECS)
            }
            _ => panic!("expected shell config"),
        }
        assert!(app.property_edit.is_some());
        assert!(!app.has_unsaved_changes());
    }

    #[test]
    fn property_edit_cancel_preserves_original_value() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Original", "echo example");
        app.clear_workflow_dirty();
        let node_id = app.selected_node_id().expect("selected shell node");

        assert!(app.start_property_edit(PropertyEditField::Label));
        app.property_edit.as_mut().expect("property edit").draft = "Changed".to_string();
        app.cancel_property_edit();

        let label = app.workflow.graph.node(node_id).expect("node").label.as_str();
        assert_eq!(label, "Original");
        assert!(app.property_edit.is_none());
        assert!(!app.has_unsaved_changes());
    }

    #[test]
    fn timeout_validation_accepts_whole_seconds_only() {
        assert_eq!(validate_timeout_secs("42"), Ok(42));
        assert_eq!(validate_timeout_secs("nope"), Err(PropertyEditError::InvalidTimeout));
    }

    #[test]
    fn add_node_flow_creates_shell_node_from_entered_fields() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        app.open_add_node_flow();
        app.start_manual_shell_node_flow();
        app.add_node_flow.label = "Custom".to_string();
        app.confirm_add_node_flow();
        app.add_node_flow.command = "echo custom".to_string();
        app.confirm_add_node_flow();

        assert_eq!(app.workflow.graph.node_count(), 1);
        let node_id = app.graph_state.focused.expect("new node focused");
        assert_eq!(app.workflow.graph.node(node_id).expect("node").label, "Custom");
        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "echo custom"),
            _ => panic!("expected shell config"),
        }
        assert!(!app.is_add_node_flow_open());
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn add_node_flow_creates_image_input_node() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        app.open_add_node_flow();
        app.start_image_input_node_flow();
        app.add_node_flow.label = "Selfie".to_string();
        app.confirm_add_node_flow();
        app.add_node_flow.command = "/tmp/selfie.png".to_string();
        app.confirm_add_node_flow();

        let node_id = app.graph_state.focused.expect("new node focused");
        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::ImageInput { path, .. } => assert_eq!(path, "/tmp/selfie.png"),
            _ => panic!("expected image input config"),
        }
        assert!(!app.is_add_node_flow_open());
        assert!(app.has_unsaved_changes());
    }

    #[test]
    fn add_node_flow_creates_pi_and_clankers_agent_nodes() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        app.open_add_node_flow();
        app.start_pi_agent_node_flow();
        app.add_node_flow.label = "Pi image agent".to_string();
        app.confirm_add_node_flow();
        app.add_node_flow.command = "make art".to_string();
        app.confirm_add_node_flow();
        let pi_node_id = app.graph_state.focused.expect("pi node focused");
        match app.workflow.node_config(pi_node_id).expect("pi node config") {
            NodeConfig::Agent(settings) => {
                assert_eq!(settings.target, AgentTarget::Pi);
                assert_eq!(settings.prompt, "make art");
            }
            _ => panic!("expected agent config"),
        }

        app.open_add_node_flow();
        app.start_clankers_agent_node_flow();
        app.add_node_flow.label = "Clankers image agent".to_string();
        app.confirm_add_node_flow();
        app.add_node_flow.command = "make ink art".to_string();
        app.confirm_add_node_flow();
        let clankers_node_id = app.graph_state.focused.expect("clankers node focused");
        match app.workflow.node_config(clankers_node_id).expect("clankers node config") {
            NodeConfig::Agent(settings) => {
                assert_eq!(settings.target, AgentTarget::Clankers);
                assert_eq!(settings.output_mode, AgentOutputMode::Image);
            }
            _ => panic!("expected agent config"),
        }
    }

    #[test]
    fn image_and_agent_property_edits_update_config_and_reject_invalid_values() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_image_input_node("Selfie", "/tmp/old.png");
        let image_node_id = app.graph_state.focused.expect("image node focused");
        assert!(app.start_property_edit(PropertyEditField::ImagePath));
        app.property_edit.as_mut().expect("image edit").draft = "/tmp/new.png".to_string();
        assert!(app.save_property_edit());
        match app.workflow.node_config(image_node_id).expect("image config") {
            NodeConfig::ImageInput { path, .. } => assert_eq!(path, "/tmp/new.png"),
            _ => panic!("expected image input config"),
        }

        app.add_agent_node("Agent", AgentTarget::Pi, "old prompt", AgentOutputMode::Text);
        let agent_node_id = app.graph_state.focused.expect("agent node focused");
        assert!(app.start_property_edit(PropertyEditField::AgentPrompt));
        app.property_edit.as_mut().expect("prompt edit").draft = "new prompt".to_string();
        assert!(app.save_property_edit());
        assert!(app.start_property_edit(PropertyEditField::AgentTarget));
        app.property_edit.as_mut().expect("target edit").draft = "clankers".to_string();
        assert!(app.save_property_edit());
        assert!(app.start_property_edit(PropertyEditField::AgentOutputMode));
        app.property_edit.as_mut().expect("mode edit").draft = "image".to_string();
        assert!(app.save_property_edit());
        assert!(app.start_property_edit(PropertyEditField::AgentContextRefs));
        app.property_edit.as_mut().expect("context edit").draft = "@one.png, @two.png".to_string();
        assert!(app.save_property_edit());
        assert!(app.start_property_edit(PropertyEditField::AgentSession));
        app.property_edit.as_mut().expect("session edit").draft = "session-1".to_string();
        assert!(app.save_property_edit());
        assert!(app.start_property_edit(PropertyEditField::AgentCredentials));
        app.property_edit.as_mut().expect("credential edit").draft = "TOKEN=env:artist-token".to_string();
        assert!(app.save_property_edit());
        match app.workflow.node_config(agent_node_id).expect("agent config") {
            NodeConfig::Agent(settings) => {
                assert_eq!(settings.prompt, "new prompt");
                assert_eq!(settings.target, AgentTarget::Clankers);
                assert_eq!(settings.output_mode, AgentOutputMode::Image);
                assert_eq!(settings.context_refs, vec!["@one.png".to_string(), "@two.png".to_string()]);
                assert_eq!(settings.session.as_deref(), Some("session-1"));
            }
            _ => panic!("expected agent config"),
        }

        let credentials = app.workflow.node_credentials(agent_node_id).expect("credentials");
        assert_eq!(credentials.env.get("TOKEN").expect("token ref").identity(), "env:artist-token");

        assert!(app.start_property_edit(PropertyEditField::AgentTarget));
        app.property_edit.as_mut().expect("bad target edit").draft = "bad-target".to_string();
        assert!(!app.save_property_edit());
        assert!(app.property_edit.is_some());
        app.cancel_property_edit();

        assert!(app.start_property_edit(PropertyEditField::AgentCredentials));
        app.property_edit.as_mut().expect("bad credentials edit").draft = "not-a-ref".to_string();
        assert!(!app.save_property_edit());
        assert!(app.property_edit.is_some());
    }

    #[test]
    fn add_node_flow_creates_node_from_template() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.set_node_templates(vec![NodeTemplate {
            name: "grep".to_string(),
            description: "Filter text".to_string(),
            kind: "shell".to_string(),
            command: Some("grep needle".to_string()),
            parameters: Vec::new(),
            input_ports: vec![PortTemplate {
                label: "stdin".to_string(),
                type_tag: "string".to_string(),
            }],
            output_ports: vec![PortTemplate {
                label: "stdout".to_string(),
                type_tag: "string".to_string(),
            }],
        }]);

        app.open_add_node_flow();
        app.start_template_node_flow();
        app.confirm_add_node_flow();

        assert_eq!(app.workflow.graph.node_count(), 1);
        let node_id = app.graph_state.focused.expect("new node focused");
        let node = app.workflow.graph.node(node_id).expect("node");
        assert_eq!(node.label, "grep");
        assert_eq!(node.input_ports.len(), 1);
        assert_eq!(node.output_ports.len(), 1);
        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "grep needle"),
            _ => panic!("expected shell config"),
        }
    }

    #[test]
    fn add_node_flow_cancel_preserves_workflow() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        app.open_add_node_flow();
        app.add_node_flow.label = "Should not exist".to_string();
        app.cancel_add_node_flow();

        assert_eq!(app.workflow.graph.node_count(), 0);
        assert!(!app.is_add_node_flow_open());
        assert!(!app.has_unsaved_changes());
    }

    #[test]
    fn add_node_flow_prompts_and_renders_template_parameters() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.set_node_templates(vec![parameterized_grep_template()]);

        app.open_add_node_flow();
        app.start_template_node_flow();
        app.confirm_add_node_flow();

        assert_eq!(app.add_node_flow.step, AddNodeStep::TemplateParameter);
        assert_eq!(app.add_node_flow.parameter_values.len(), TEMPLATE_PARAMETER_COUNT);
        assert_eq!(app.add_node_flow.parameter_index, TEMPLATE_FIRST_PARAMETER_INDEX);
        app.add_node_flow.parameter_values[TEMPLATE_FIRST_PARAMETER_INDEX] = "needle".to_string();
        app.confirm_add_node_flow();

        let node_id = app.graph_state.focused.expect("new node focused");
        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "grep -E 'needle'"),
            _ => panic!("expected shell config"),
        }
        assert!(!app.is_add_node_flow_open());
    }

    #[test]
    fn cancel_template_parameter_entry_preserves_workflow() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.set_node_templates(vec![parameterized_grep_template()]);

        app.open_add_node_flow();
        app.start_template_node_flow();
        app.confirm_add_node_flow();
        app.cancel_add_node_flow();

        assert_eq!(app.workflow.graph.node_count(), 0);
        assert!(!app.is_add_node_flow_open());
        assert!(!app.has_unsaved_changes());
    }

    #[test]
    fn invalid_template_parameter_keeps_flow_recoverable_without_node() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        let mut template = parameterized_grep_template();
        template.command = Some("head -n ${limit}".to_string());
        template.parameters = vec![TemplateParameter {
            name: "limit".to_string(),
            type_tag: TemplateParameterType::Number,
            description: "Line limit".to_string(),
            default: Some(TemplateParameterDefault::Number(TEMPLATE_NUMBER_DEFAULT)),
            required: true,
            min: Some(TEMPLATE_NUMBER_MIN),
            max: Some(TEMPLATE_NUMBER_MAX),
        }];
        app.set_node_templates(vec![template]);

        app.open_add_node_flow();
        app.start_template_node_flow();
        app.confirm_add_node_flow();
        app.add_node_flow.parameter_values[TEMPLATE_FIRST_PARAMETER_INDEX] = "not-a-number".to_string();
        app.confirm_add_node_flow();

        assert_eq!(app.workflow.graph.node_count(), 0);
        assert_eq!(app.add_node_flow.step, AddNodeStep::TemplateParameter);
        assert!(app.is_add_node_flow_open());
        assert!(output_text_lines(&app).iter().any(|line| line.contains("expects a number")));
    }

    #[test]
    fn empty_add_node_label_uses_default() {
        assert_eq!(normalized_node_label("", "Shell 1"), "Shell 1");
        assert_eq!(normalized_node_label("  Custom  ", "Shell 1"), "Custom");
    }

    #[test]
    fn completed_output_lines_are_port_labeled_and_ordered() {
        let mut workflow = Workflow::new("display");
        let node_id = workflow.add_shell_node("grep", "grep world");
        let node = workflow.graph.node(node_id).expect("node exists");
        let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
        let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
        let mut outputs = HashMap::new();
        outputs.insert(exit_port, Value::from_exit_code(EXIT_CODE_SUCCESS));
        outputs.insert(stdout_port, Value::from_string("hello world\n".to_string()));

        assert_eq!(
            rendered_output_records(completed_output_records(RunId(SNAPSHOT_RUN_ID), &workflow, node_id, &outputs)),
            vec![
                "  │ stdout: hello world".to_string(),
                "  │ exit: exit code: 0".to_string()
            ],
        );
    }

    #[test]
    fn completed_image_outputs_render_bounded_preview_metadata() {
        let mut workflow = Workflow::new("image-preview");
        let node_id = workflow.add_agent_node("Image agent", AgentTarget::Clankers, "make art", AgentOutputMode::Image);
        let node = workflow.graph.node(node_id).expect("node exists");
        let image_port = node.output_ports[AGENT_IMAGE_OUTPUT_INDEX].id;
        let mut outputs = HashMap::new();
        outputs.insert(
            image_port,
            Value::from_image(ImageValue::new(
                IMAGE_PREVIEW_BYTES.to_vec(),
                "image/png".to_string(),
                ImageMetadata::default(),
            )),
        );

        let rendered =
            rendered_output_records(completed_output_records(RunId(SNAPSHOT_RUN_ID), &workflow, node_id, &outputs));

        assert!(rendered.iter().any(|line| line.contains("image: image/png image")));
        assert!(!rendered.iter().any(|line| line.contains("raw-image-bytes")));
    }

    #[test]
    fn completed_output_records_preserve_run_node_and_port_metadata() {
        let mut workflow = Workflow::new(SNAPSHOT_WORKFLOW_NAME);
        let node_id = workflow.add_shell_node(SNAPSHOT_NODE_LABEL, SNAPSHOT_COMMAND);
        let node = workflow.graph.node(node_id).expect("node exists");
        let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
        let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
        let mut outputs = HashMap::new();
        outputs.insert(stdout_port, Value::from_string(SNAPSHOT_STDOUT.to_string()));
        outputs.insert(exit_port, Value::from_exit_code(EXIT_CODE_SUCCESS));

        let records = completed_output_records(RunId(SNAPSHOT_RUN_ID), &workflow, node_id, &outputs);

        assert_eq!(records.len(), SNAPSHOT_OUTPUT_RECORD_COUNT);
        let stdout_record = records.iter().find(|record| record.port_id == stdout_port).expect("stdout record");
        assert_eq!(stdout_record.run_id, RunId(SNAPSHOT_RUN_ID));
        assert_eq!(stdout_record.node_id, node_id);
        assert_eq!(stdout_record.port_label, "stdout");
        assert_eq!(stdout_record.text, SNAPSHOT_STDOUT_TEXT);
    }

    #[test]
    fn latest_node_output_records_return_latest_lines_in_port_order() {
        let mut workflow = Workflow::new(SNAPSHOT_WORKFLOW_NAME);
        let node_id = workflow.add_shell_node(SNAPSHOT_NODE_LABEL, SNAPSHOT_COMMAND);
        let other_node_id = workflow.add_shell_node(MUTATED_NODE_LABEL, SNAPSHOT_COMMAND);
        let node = workflow.graph.node(node_id).expect("node exists");
        let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
        let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
        let other_stdout_port =
            workflow.graph.node(other_node_id).expect("other node exists").output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
        let records = vec![
            RunOutputLine {
                run_id: RunId(SNAPSHOT_RUN_ID),
                node_id,
                port_id: stdout_port,
                port_label: "stdout".to_string(),
                text: OLDER_STDOUT_TEXT.to_string(),
            },
            RunOutputLine {
                run_id: RunId(SNAPSHOT_RUN_ID),
                node_id,
                port_id: exit_port,
                port_label: "exit".to_string(),
                text: "exit code: 0".to_string(),
            },
            RunOutputLine {
                run_id: RunId(SNAPSHOT_SECOND_RUN_ID),
                node_id,
                port_id: stdout_port,
                port_label: "stdout".to_string(),
                text: SNAPSHOT_STDOUT_TEXT.to_string(),
            },
            RunOutputLine {
                run_id: RunId(SNAPSHOT_SECOND_RUN_ID),
                node_id: other_node_id,
                port_id: other_stdout_port,
                port_label: "stdout".to_string(),
                text: OTHER_NODE_TEXT.to_string(),
            },
        ];

        let rendered = rendered_latest_node_output_lines(&records, &workflow, node_id);

        assert_eq!(rendered, vec![
            "  │ stdout: snapshot output".to_string(),
            "  │ exit: exit code: 0".to_string()
        ],);
    }

    #[test]
    fn completed_output_lines_omit_empty_streams_for_no_match() {
        let mut workflow = Workflow::new("display");
        let node_id = workflow.add_shell_node("grep", "grep world");
        let node = workflow.graph.node(node_id).expect("node exists");
        let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
        let stderr_port = node.output_ports[SHELL_STDERR_OUTPUT_INDEX].id;
        let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
        let mut outputs = HashMap::new();
        outputs.insert(stdout_port, Value::from_string(String::new()));
        outputs.insert(stderr_port, Value::from_string(String::new()));
        outputs.insert(exit_port, Value::from_exit_code(EXIT_CODE_GREP_NO_MATCH));

        assert_eq!(
            rendered_output_records(completed_output_records(RunId(SNAPSHOT_RUN_ID), &workflow, node_id, &outputs)),
            vec!["  │ exit: exit code: 1".to_string()],
        );
    }

    #[test]
    fn finish_workflow_run_logs_executed_workflow_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        let mut run_workflow = Workflow::new(SNAPSHOT_WORKFLOW_NAME);
        let node_id = run_workflow.add_shell_node(SNAPSHOT_NODE_LABEL, SNAPSHOT_COMMAND);
        let node = run_workflow.graph.node(node_id).expect("node exists");
        let stdout_port = node.output_ports[SHELL_STDOUT_OUTPUT_INDEX].id;
        let exit_port = node.output_ports[SHELL_EXIT_OUTPUT_INDEX].id;
        let mut outputs = HashMap::new();
        outputs.insert(stdout_port, Value::from_string(SNAPSHOT_STDOUT.to_string()));
        outputs.insert(exit_port, Value::from_exit_code(EXIT_CODE_SUCCESS));
        let step = StepResult {
            node_id,
            status: NodeStatus::Completed { outputs, cached: false },
            input_hash: ContentHash::from_bytes(SNAPSHOT_INPUT_HASH_BYTES),
        };

        app.workflow = run_workflow.clone();
        app.workflow.graph.node_mut(node_id).expect("current node exists").label = MUTATED_NODE_LABEL.to_string();
        app.finish_workflow_run(RunId(SNAPSHOT_RUN_ID), &run_workflow, vec![step]);

        let lines = output_text_lines(&app);
        assert!(lines.iter().any(|line| line.contains(SNAPSHOT_NODE_LABEL)), "lines: {lines:?}");
        assert!(!lines.iter().any(|line| line.contains(MUTATED_NODE_LABEL)), "lines: {lines:?}");
        assert!(lines.iter().any(|line| line == "  │ stdout: snapshot output"), "lines: {lines:?}");
        assert_eq!(app.run_output_lines.len(), SNAPSHOT_OUTPUT_RECORD_COUNT);
        assert!(
            app.run_output_lines.iter().any(|record| record.run_id == RunId(SNAPSHOT_RUN_ID)
                && record.node_id == node_id
                && record.text == SNAPSHOT_STDOUT_TEXT),
            "records: {:?}",
            app.run_output_lines
        );
    }

    #[tokio::test]
    async fn background_run_starts_without_blocking() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Long", "sleep 1");
        let node_id = app.graph_state.focused.expect("focused node");

        assert!(app.start_workflow_run());

        assert!(app.run_state.active);
        assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Pending),);
        assert!(app.cancel_active_run());
    }

    #[tokio::test]
    async fn cancel_active_run_marks_pending_nodes_skipped() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Long", "sleep 1");
        let node_id = app.graph_state.focused.expect("focused node");

        assert!(app.start_workflow_run());
        assert!(app.cancel_active_run());

        assert!(!app.run_state.active);
        assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Skipped),);
    }

    #[tokio::test]
    async fn failed_background_run_marks_node_failed() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        let node_id = app.workflow.add_shell_node("Bad cwd", "echo no");
        app.workflow.set_node_config(node_id, NodeConfig::Shell {
            command: "echo no".to_string(),
            cwd: Some(temp_dir.path().join("missing").display().to_string()),
            env: HashMap::new(),
            timeout_secs: DEFAULT_SHELL_TIMEOUT_SECS,
            pipe_stdin: true,
        });
        app.sync_graph_state();
        app.select_node(node_id);

        assert!(app.start_workflow_run());
        for _ in 0..RUN_EVENT_POLL_LIMIT {
            app.poll_run_events();
            if !app.run_state.active {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(RUN_EVENT_POLL_DELAY_MS)).await;
        }

        assert!(!app.run_state.active);
        assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Failed),);
    }

    // r[verify n8nclone.workflow_execution.image_input_nodes.production_io.tui]
    #[tokio::test]
    async fn tui_background_run_loads_local_image_with_default_executor() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let image_path = temp_dir.path().join(TUI_LOCAL_IMAGE_FILE_NAME);
        std::fs::write(&image_path, TUI_LOCAL_PNG_FIXTURE).expect("TUI image fixture written");
        let mut app = App::new(test_config(&temp_dir));
        app.add_image_input_node("Local image", &image_path.display().to_string());
        let node_id = app.graph_state.focused.expect("image node focused");

        assert!(app.start_workflow_run());
        drain_run(&mut app).await;

        assert!(!app.run_state.active);
        assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Completed),);
        assert!(output_text_lines(&app).iter().any(|line| line.contains("image/png image")));
    }

    #[tokio::test]
    async fn image_node_error_feedback_is_visible_after_run_failure() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut missing_app = App::new(test_config(&temp_dir));
        missing_app.set_executor_factory(image_executor_factory(HashMap::new()));
        missing_app.add_image_input_node("Missing image", TUI_MISSING_IMAGE_PATH);
        let missing_node_id = missing_app.graph_state.focused.expect("missing image node focused");

        assert!(missing_app.start_workflow_run());
        drain_run(&mut missing_app).await;

        assert_eq!(missing_app.run_state.node_statuses.get(&missing_node_id.0), Some(&NodeRunStatus::Failed),);
        assert!(output_text_lines(&missing_app).iter().any(|line| line.contains("missing image")));

        let mut invalid_app = App::new(test_config(&temp_dir));
        invalid_app.set_executor_factory(image_executor_factory(
            [(TUI_BAD_IMAGE_PATH.to_string(), TUI_BAD_IMAGE_BYTES.to_vec())].into(),
        ));
        invalid_app.add_image_input_node("Bad image", TUI_BAD_IMAGE_PATH);
        let invalid_node_id = invalid_app.graph_state.focused.expect("invalid image node focused");

        assert!(invalid_app.start_workflow_run());
        drain_run(&mut invalid_app).await;

        assert_eq!(invalid_app.run_state.node_statuses.get(&invalid_node_id.0), Some(&NodeRunStatus::Failed),);
        assert!(output_text_lines(&invalid_app).iter().any(|line| line.contains("unsupported image format")));
    }

    #[tokio::test]
    async fn agent_error_feedback_is_visible_after_run_failure() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.set_executor_factory(agent_error_executor_factory(AgentError::Adapter("runtime unavailable".to_string())));
        app.add_agent_node("Pi agent", AgentTarget::Pi, "summarize", AgentOutputMode::Text);
        let node_id = app.graph_state.focused.expect("focused node");

        assert!(app.start_workflow_run());
        drain_run(&mut app).await;

        assert!(!app.run_state.active);
        assert_eq!(app.run_state.node_statuses.get(&node_id.0), Some(&NodeRunStatus::Failed),);
        assert!(output_text_lines(&app).iter().any(|line| line.contains("runtime unavailable")));

        let mut preflight_app = App::new(test_config(&temp_dir));
        preflight_app.set_executor_factory(agent_error_executor_factory(AgentError::PreflightRejected(
            "model unavailable".to_string(),
        )));
        preflight_app.add_agent_node("Preflight agent", AgentTarget::Clankers, "summarize", AgentOutputMode::Text);
        let preflight_node_id = preflight_app.graph_state.focused.expect("preflight node focused");

        assert!(preflight_app.start_workflow_run());
        drain_run(&mut preflight_app).await;

        assert_eq!(preflight_app.run_state.node_statuses.get(&preflight_node_id.0), Some(&NodeRunStatus::Failed),);
        assert!(output_text_lines(&preflight_app).iter().any(|line| line.contains("preflight rejected")));
    }

    #[tokio::test]
    async fn completed_background_run_is_persisted_to_history() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Ok", "echo ok");

        assert!(app.start_workflow_run());
        for _ in 0..RUN_EVENT_POLL_LIMIT {
            app.poll_run_events();
            if !app.run_state.active {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(RUN_EVENT_POLL_DELAY_MS)).await;
        }
        app.open_run_history();

        assert!(app.is_run_history_open());
        assert_eq!(app.run_history.records.len(), 1);
        assert_eq!(app.run_history.records[0].node_count, 1);
        assert_eq!(app.run_history.records[0].failures, 0);
    }

    #[tokio::test]
    async fn canceled_background_run_is_persisted_to_history() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));
        app.add_shell_node("Long", "sleep 1");

        assert!(app.start_workflow_run());
        assert!(app.cancel_active_run());
        app.open_run_history();

        assert_eq!(app.run_history.records.len(), 1);
        assert_eq!(app.run_history.records[0].node_count, 1);
    }

    #[test]
    fn run_history_empty_and_inspect_states_are_visible() {
        let temp_dir = tempfile::tempdir().expect("temporary app directory");
        let mut app = App::new(test_config(&temp_dir));

        app.open_run_history();
        assert_eq!(app.run_history.message.as_deref(), Some("No run history available"),);

        let record = RunRecord {
            run_id: RunId(7),
            workflow_name: "wf".to_string(),
            started_epoch_ms: 0,
            finished_epoch_ms: Some(0),
            node_count: 2,
            cache_hits: 1,
            failures: 0,
        };
        app.run_history = RunHistoryState::open(vec![record], None);
        app.inspect_selected_run_history();

        assert_eq!(app.run_history.detail.as_deref(), Some("No run details available"));
        assert!(app.run_history.detail_record.is_none());
    }
}
