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
use rat_nodegraph::view::NodeGraphState;
use rat_nodegraph::view::node_bounds;
use rat_streaming::StreamingOutput;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tracing::error;
use tracing::info;

use crate::config::AppConfig;
use crate::engine::executor::Executor;
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

        self.sync_graph_state();

        // Select the new node.
        self.graph_state.selected.clear();
        self.graph_state.selected.insert(id);
        self.graph_state.focused = Some(id);
        self.mark_workflow_dirty();
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
                            let n = self.workflow.graph.node_count();
                            self.add_shell_node(&format!("Shell {}", n + 1), "echo 'hello'");
                        }
                        AppAction::OpenWorkflow => {
                            self.output.push_line("Workflow browser not yet implemented");
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
}
