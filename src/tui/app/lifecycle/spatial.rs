//! Synchronized graph selection and node placement.

use super::*;

impl State {
    /// Cycle focus to the next panel.
    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Canvas => FocusPanel::Properties,
            FocusPanel::Properties => FocusPanel::Output,
            FocusPanel::Output => FocusPanel::Canvas,
        };
    }

    // r[impl lattice.tui_ux.rows.selection]
    pub fn toggle_canvas_view(&mut self) {
        self.graph_state.mode = rat_nodegraph::view::InteractionMode::Normal;
        self.graph_state.selected_edge = None;
        self.last_drag_canvas = None;
        self.rows_hit_area = None;
        if self.graph_state.focused.is_none() {
            self.graph_state.focused = self.graph_state.selected.iter().copied().min_by_key(|id| id.0);
        }
        self.canvas_view = match self.canvas_view {
            crate::tui::rows::CanvasView::Graph => crate::tui::rows::CanvasView::Rows,
            crate::tui::rows::CanvasView::Rows => crate::tui::rows::CanvasView::Graph,
        };
    }

    pub(crate) fn select_row(&mut self, delta: isize) {
        let Some(layout) = self.row_selection_layout() else {
            return;
        };
        if layout.rows.is_empty() {
            return;
        }
        let Some(current) =
            self.graph_state.focused.and_then(|id| layout.rows.iter().position(|row| row.node_id.0 == id.0))
        else {
            self.focus_row(&layout, 0);
            return;
        };
        let next = if delta < 0 {
            current.saturating_sub(delta.unsigned_abs())
        } else {
            current.saturating_add(delta.unsigned_abs()).min(layout.rows.len().saturating_sub(1))
        };
        self.focus_row(&layout, next);
    }

    /// Select the first or last row in the current topological layout.
    pub fn select_row_boundary(&mut self, last: bool) {
        let Some(layout) = self.row_selection_layout() else {
            return;
        };
        if layout.rows.is_empty() {
            return;
        }
        let index = if last { layout.rows.len().saturating_sub(1) } else { 0 };
        self.focus_row(&layout, index);
    }

    /// Select a specific visible row index.
    pub(crate) fn select_row_index(&mut self, index: usize) {
        let Some(layout) = self.row_selection_layout() else {
            return;
        };
        self.focus_row(&layout, index);
    }

    fn row_selection_layout(&mut self) -> Option<crate::tui::rows::Layout> {
        match crate::tui::rows::layout(&self.workflow, crate::tui::rows::DEFAULT_MAX_LANES) {
            Ok(layout) => Some(layout),
            Err(error) => {
                self.output.push_line(&format!("Rows layout failed: {error}"));
                None
            }
        }
    }

    pub(in crate::tui::app) fn focus_row(&mut self, layout: &crate::tui::rows::Layout, index: usize) {
        let Some(row) = layout.rows.get(index) else {
            return;
        };
        let graph_id = crate::tui::graph::subwayrat_node_id(row.node_id);
        self.graph_state.selected.clear();
        self.graph_state.selected.insert(graph_id);
        self.graph_state.focused = Some(graph_id);
        self.focus = FocusPanel::Canvas;
    }

    // r[impl n8nclone.tui_ux.triggers.manage]
    pub fn trigger_status_lines(&self) -> Vec<String> {
        let records = self
            .store
            .as_ref()
            .and_then(|store| store.list_trigger_runs(TRIGGER_STATUS_HISTORY_LIMIT).ok())
            .unwrap_or_default();
        trigger_status_lines_from_history(&self.workflow.triggers, &records)
    }

    /// Add a new shell node to the workflow.
    pub fn add_shell_node(&mut self, label: impl AsRef<str>, command: impl Into<String>) {
        let id = self.workflow.add_shell_node(label.as_ref(), command.into());
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    pub fn add_http_node(&mut self, label: impl Into<String>, method: impl Into<String>, url: impl Into<String>) {
        let id = self.workflow.add_http_node(label, method, url);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    pub fn add_file_node(
        &mut self,
        label: impl AsRef<str>,
        operation: crate::file_node::FileOperationKind,
        path: impl Into<String>,
    ) {
        let id = self.workflow.add_file_node(label.as_ref(), operation, path.into());
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    pub fn add_image_input_node(&mut self, label: impl AsRef<str>, path: impl Into<String>) {
        let path = path.into();
        let id = self.workflow.add_image_input_node(label.as_ref(), &path);
        self.workflow.set_node_config(id, crate::tui::app::editing::parsing::image_input_config(path));
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    pub fn add_agent_node(
        &mut self,
        label: impl AsRef<str>,
        target: crate::agent_node::Target,
        prompt: impl Into<String>,
        output_mode: crate::agent_node::OutputMode,
    ) {
        let id = self.workflow.add_agent_node(label.as_ref(), target, prompt.into(), output_mode);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    // r[impl n8nclone.tui_ux.control_flow.add_flow]
    pub fn add_condition_node(&mut self, label: &str, config: ConditionNodeConfig) {
        let id = self.workflow.add_condition_node(label, config);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    // r[impl n8nclone.tui_ux.control_flow.add_flow]
    pub fn add_merge_node(&mut self, label: &str, config: MergeNodeConfig) {
        let id = self.workflow.add_merge_node(label, config);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    // r[impl n8nclone.tui_ux.control_flow.add_flow]
    pub fn add_subworkflow_node(&mut self, label: &str, config: SubworkflowNodeConfig) {
        let id = self.workflow.add_subworkflow_node(label, config);
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
    }

    pub(in crate::tui::app) fn place_new_node(&mut self, id: crate::engine::workflow::NodeId) {
        self.sync_graph_state();
        let id = crate::tui::graph::subwayrat_node_id(id);
        let mut max_right: i64 = 0;
        let mut stack_y: i32 = 0;
        for node in self.graph_state.graph.nodes() {
            if node.id == id {
                continue;
            }
            if let Some(bounds) = node_bounds(&self.graph_state.graph, node.id) {
                let right = i64::from(bounds.x.get()) + i64::from(bounds.width);
                if right > max_right {
                    max_right = right;
                    stack_y = bounds.y.get();
                }
            }
        }
        let has_others = self.graph_state.graph.node_count() > 1;
        if let Some(node) = self.graph_state.graph.node_mut(id)
            && has_others
        {
            match i32::try_from(max_right + i64::from(NEW_NODE_HORIZONTAL_GAP)) {
                Ok(x) => {
                    node.x = x;
                    node.y = stack_y;
                }
                Err(_) => {
                    self.output.push_line("Canvas placement exceeds supported coordinates; node position unchanged")
                }
            }
        }
        self.sync_workflow_graph_from_state();
    }

    pub(in crate::tui::app) fn select_node(&mut self, id: impl WorkflowNodeIdentity) {
        let id = crate::tui::graph::subwayrat_node_id(id.workflow_node_id());
        self.graph_state.selected.clear();
        self.graph_state.selected.insert(id);
        self.graph_state.focused = Some(id);
    }

    /// Run auto-layout on the graph.
    pub fn auto_layout(&mut self) {
        let layout = LayoutConfig {
            direction: rat_nodegraph::layout::LayoutDirection::LeftToRight,
            h_spacing: AUTO_LAYOUT_HORIZONTAL_SPACING_COLUMNS,
            v_spacing: AUTO_LAYOUT_VERTICAL_SPACING_ROWS,
            grid_size: 0,
            node_width: AUTO_LAYOUT_NODE_WIDTH_COLUMNS,
            node_height: AUTO_LAYOUT_NODE_HEIGHT_ROWS,
            partial: false,
        };
        auto_layout(&mut self.graph_state.graph, &layout);
        self.sync_workflow_graph_from_state();
        self.mark_workflow_dirty();
        self.output.push_line("Auto-layout applied");
    }

    /// Rebuild the Subwayrat state from the Lattice-owned workflow graph.
    pub(in crate::tui::app) fn sync_graph_state(&mut self) {
        let old_viewport = self.graph_state.viewport.clone();
        let Ok(graph) = crate::tui::graph::to_subwayrat(&self.workflow.graph) else {
            self.output.push_line("Canvas graph conversion failed; the current canvas was preserved");
            return;
        };
        self.graph_state = NodeGraphState::new(graph, old_viewport.width, old_viewport.height);
        self.graph_state.viewport = old_viewport;
    }

    /// Admit current Subwayrat edit state into the Lattice-owned workflow graph.
    pub(crate) fn sync_workflow_graph_from_state(&mut self) {
        match crate::tui::graph::from_subwayrat(&self.graph_state.graph) {
            Ok(graph) => self.workflow.graph = graph,
            Err(_) => self.output.push_line("Canvas graph conversion was rejected; workflow state was preserved"),
        }
    }
}
