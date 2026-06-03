//! Core application state and event loop.

use std::collections::HashMap;
use std::io;

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
use rat_nodegraph::model::PortSpec;
use rat_nodegraph::view::NodeGraphState;
use rat_nodegraph::view::node_bounds;
use rat_streaming::StreamingOutput;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tracing::error;
use tracing::info;

use crate::config::AppConfig;
use crate::config::NodeTemplate;
use crate::engine::executor::Executor;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;
use crate::store::Store;
use crate::tui::input;
use crate::tui::panels;

const INITIAL_VIEWPORT_WIDTH: u16 = 80;
const INITIAL_VIEWPORT_HEIGHT: u16 = 24;
const NEW_NODE_HORIZONTAL_GAP: i32 = 4;

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
    SaveWorkflow,
    AddNode,
    OpenWorkflow,
    AutoLayout,
}

/// Command palette data.
pub struct PaletteData {
    pub items: Vec<FuzzyCandidate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyEditField {
    Label,
    Command,
    TimeoutSecs,
}

impl PropertyEditField {
    fn label(self) -> &'static str {
        match self {
            Self::Label => "label",
            Self::Command => "command",
            Self::TimeoutSecs => "timeout",
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
}

pub fn validate_timeout_secs(input: &str) -> Result<u64, PropertyEditError> {
    input.trim().parse::<u64>().map_err(|_| PropertyEditError::InvalidTimeout)
}

pub fn normalized_node_label(input: &str, default_label: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        default_label.to_string()
    } else {
        trimmed.to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddNodeStep {
    TemplateSelect,
    Label,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddNodeFlowState {
    pub open: bool,
    pub step: AddNodeStep,
    pub label: String,
    pub command: String,
    pub selected_template: usize,
}

impl AddNodeFlowState {
    fn closed() -> Self {
        Self {
            open: false,
            step: AddNodeStep::Label,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
        }
    }

    fn shell(default_label: String) -> Self {
        Self {
            open: true,
            step: AddNodeStep::Label,
            label: default_label,
            command: String::new(),
            selected_template: 0,
        }
    }

    fn templates() -> Self {
        Self {
            open: true,
            step: AddNodeStep::TemplateSelect,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
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
                id: "run".into(),
                text: "Run Workflow (Ctrl-R)".into(),
                context: None,
                icon: Some('▶'),
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
        "save" => Some(AppAction::SaveWorkflow),
        "open" => Some(AppAction::OpenWorkflow),
        "layout" => Some(AppAction::AutoLayout),
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
    pub workflow_browser: WorkflowBrowserState,
    pub property_edit: Option<PropertyEditState>,
    pub add_node_flow: AddNodeFlowState,
    pub node_templates: Vec<NodeTemplate>,
    pub store: Option<Store>,
    pub config: AppConfig,
    pub last_drag_canvas: Option<(i32, i32)>,
    dirty: bool,
    confirm_quit: bool,
    executor: Executor,
}

impl App {
    /// Create a new app with an empty workflow.
    pub fn new(config: AppConfig) -> Self {
        let workflow = Workflow::new("untitled");
        let graph_state = NodeGraphState::new(workflow.graph.clone(), INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT);

        let store = match Store::new(&config.data_dir.join("n8nclone.redb")) {
            Ok(s) => Some(s),
            Err(e) => {
                error!(error = %e, "failed to open database, running without persistence");
                None
            }
        };

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
            workflow_browser: WorkflowBrowserState::closed(),
            property_edit: None,
            add_node_flow: AddNodeFlowState::closed(),
            node_templates: Vec::new(),
            store,
            config,
            last_drag_canvas: None,
            dirty: false,
            confirm_quit: false,
            executor: Executor::new(),
        }
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

    /// Replace the configured node templates.
    pub fn set_node_templates(&mut self, templates: Vec<NodeTemplate>) {
        self.node_templates = templates;
    }

    /// Start the guided add-node flow.
    pub fn open_add_node_flow(&mut self) {
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.property_edit = None;
        self.add_node_flow = if self.node_templates.is_empty() {
            AddNodeFlowState::shell(self.next_shell_node_label())
        } else {
            AddNodeFlowState::templates()
        };
    }

    /// Cancel guided add-node state without mutating the workflow.
    pub fn cancel_add_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::closed();
    }

    /// Switch from template selection to manual shell-node entry.
    pub fn start_manual_shell_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::shell(self.next_shell_node_label());
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
            AddNodeStep::Label => self.add_node_flow.label.push(ch),
            AddNodeStep::Command => self.add_node_flow.command.push(ch),
            AddNodeStep::TemplateSelect => {}
        }
    }

    /// Remove one character from the active add-node draft field.
    pub fn pop_add_node_char(&mut self) {
        match self.add_node_flow.step {
            AddNodeStep::Label => {
                self.add_node_flow.label.pop();
            }
            AddNodeStep::Command => {
                self.add_node_flow.command.pop();
            }
            AddNodeStep::TemplateSelect => {}
        }
    }

    /// Advance or finish the guided add-node flow.
    pub fn confirm_add_node_flow(&mut self) {
        match self.add_node_flow.step {
            AddNodeStep::TemplateSelect => self.confirm_template_node(),
            AddNodeStep::Label => self.add_node_flow.step = AddNodeStep::Command,
            AddNodeStep::Command => self.confirm_shell_node_from_flow(),
        }
    }

    fn confirm_shell_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, &self.next_shell_node_label());
        let command = self.add_node_flow.command.clone();
        self.add_shell_node(&label, &command);
        self.cancel_add_node_flow();
    }

    fn confirm_template_node(&mut self) {
        let Some(template) = self.node_templates.get(self.add_node_flow.selected_template).cloned() else {
            return;
        };
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
        let id = self.workflow.graph.add_node(template.name.clone(), &inputs, &outputs);
        self.workflow.set_node_config(id, NodeConfig::shell(template.command.clone().unwrap_or_default()));
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
        true
    }

    fn next_shell_node_label(&self) -> String {
        format!("Shell {}", self.workflow.graph.node_count() + 1)
    }

    /// Return whether the workflow browser modal is open.
    pub fn is_workflow_browser_open(&self) -> bool {
        self.workflow_browser.open
    }

    /// Open the saved-workflow browser.
    pub fn open_workflow_browser(&mut self) {
        self.show_palette = false;
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
            },
            PropertyEditField::TimeoutSecs => match self.workflow.node_config(node_id)? {
                NodeConfig::Shell { timeout_secs, .. } => Some(timeout_secs.to_string()),
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
                }
            }
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
        if let Some(node) = self.workflow.graph.node_mut(id) {
            if has_others {
                node.x = max_right + NEW_NODE_HORIZONTAL_GAP;
                node.y = stack_y;
            }
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

    /// Run the current workflow.
    pub async fn run_workflow(&mut self) {
        self.output.push_line("▶ Running workflow…");

        let cache = HashMap::new();
        let (run_id, results) = self.executor.run(&self.workflow, &cache).await;

        for step in &results {
            let node_label = self.workflow.graph.node(step.node_id).map(|n| n.label.as_str()).unwrap_or("?");

            match &step.status {
                crate::engine::executor::NodeStatus::Completed { outputs, cached, .. } => {
                    let cache_tag = if *cached { " (cached)" } else { "" };
                    self.output.push_line(&format!("✓ {node_label}{cache_tag} [{}]", step.input_hash));

                    // Show stdout if present.
                    for val in outputs.values() {
                        let display = val.as_display_string();
                        if !display.is_empty() {
                            for line in display.lines() {
                                self.output.push_line(&format!("  │ {line}"));
                            }
                        }
                    }

                    // Cache the result.
                    if let Some(ref store) = self.store {
                        let _ = store.put_cache(&step.input_hash, outputs);
                    }
                }
                crate::engine::executor::NodeStatus::Failed { error } => {
                    self.output.push_line(&format!("✗ {node_label}: {error}"));
                }
                _ => {}
            }
        }

        self.output.push_line(&format!("── run {} complete ({} steps) ──", run_id, results.len()));
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
                            self.run_workflow().await;
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
    use tempfile::TempDir;

    use super::*;
    use crate::config::PortTemplate;

    const DEFAULT_SHELL_TIMEOUT_SECS: u64 = 60;

    fn test_config(temp_dir: &TempDir) -> AppConfig {
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir).expect("create app data directory");
        AppConfig {
            data_dir,
            ..AppConfig::default()
        }
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
        app.add_node_flow.label = "Custom".to_string();
        app.confirm_add_node_flow();
        app.add_node_flow.command = "echo custom".to_string();
        app.confirm_add_node_flow();

        assert_eq!(app.workflow.graph.node_count(), 1);
        let node_id = app.graph_state.focused.expect("new node focused");
        assert_eq!(app.workflow.graph.node(node_id).expect("node").label, "Custom");
        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "echo custom"),
        }
        assert!(!app.is_add_node_flow_open());
        assert!(app.has_unsaved_changes());
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
        app.confirm_add_node_flow();

        assert_eq!(app.workflow.graph.node_count(), 1);
        let node_id = app.graph_state.focused.expect("new node focused");
        let node = app.workflow.graph.node(node_id).expect("node");
        assert_eq!(node.label, "grep");
        assert_eq!(node.input_ports.len(), 1);
        assert_eq!(node.output_ports.len(), 1);
        match app.workflow.node_config(node_id).expect("node config") {
            NodeConfig::Shell { command, .. } => assert_eq!(command, "grep needle"),
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
    fn empty_add_node_label_uses_default() {
        assert_eq!(normalized_node_label("", "Shell 1"), "Shell 1");
        assert_eq!(normalized_node_label("  Custom  ", "Shell 1"), "Custom");
    }
}
