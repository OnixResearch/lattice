//! Typed authoring commits and node deletion.

use super::*;

impl State {
    /// Browse typed fields without changing the workflow or activating any effects.
    pub fn open_authoring(&mut self, family: &'static str) {
        self.close_workflow_template_browser();
        self.show_palette = false;
        self.property_edit = None;
        self.add_node_flow = AddNodeFlowState::closed();
        self.workflow_browser = WorkflowBrowserState::closed();
        self.run_history = RunHistoryState::closed();
        self.approvals = ApprovalState::closed();
        self.diagnostics.open = false;
        let mut authoring =
            crate::tui::authoring::State::new(&self.workflow, self.selected_node_id().map(|id| id.0), family);
        self.refresh_source_fields(&mut authoring);
        self.authoring = Some(authoring);
    }

    /// Apply exactly one fully parsed field, retaining invalid/stale drafts for repair.
    pub fn confirm_authoring(&mut self) -> bool {
        let Some(mut state) = self.authoring.take() else {
            return false;
        };
        let Some(draft) = state.draft.as_ref() else {
            state.begin(&self.workflow);
            self.authoring = Some(state);
            return false;
        };
        let current = crate::workflow_format::workflow_to_portable(&self.workflow);
        if current != draft.original
            || crate::authoring::identity("lattice.tui-authoring-snapshot.v1", &self.workflow).ok()
                != draft.authoring_snapshot
        {
            state.error = Some("Workflow changed while editing; Esc cancels this stale draft".into());
            self.authoring = Some(state);
            return false;
        }
        let id = draft.field.id.clone();
        let is_source_edit = id.starts_with("source.");
        let result = if is_source_edit {
            self.apply_source_edit(draft)
        } else if id.starts_with("operation.") {
            let runtime = crate::process::core::RuntimeConfig {
                shell_program: self.config.shell.clone(),
                limits: self.config.process_limits.clone(),
            };
            state.operation.apply(&mut self.workflow, state.node_id, &id, &draft.text, &runtime)
        } else {
            crate::tui::authoring::apply(&mut self.workflow, state.node_id, &id, &draft.text)
        };
        match result {
            Ok(()) => {
                if !is_source_edit && let Err(error) = crate::authoring::reconcile_materialized(&mut self.workflow) {
                    state.error = Some(error.into());
                    self.authoring = Some(state);
                    return false;
                }
                let has_changed = current != crate::workflow_format::workflow_to_portable(&self.workflow);
                if has_changed && !is_source_edit {
                    let selection = self.graph_state.selected.clone();
                    self.sync_graph_state();
                    self.graph_state.selected =
                        selection.into_iter().filter(|id| self.graph_state.graph.node(*id).is_some()).collect();
                    // Explicit metadata timestamps are authorable; do not overwrite
                    // them with a second clock observation after an exact field edit.
                    self.dirty = true;
                }
                state.draft = None;
                state.error = None;
                state.refresh(&self.workflow);
                self.refresh_source_fields(&mut state);
                if let Some(index) = state.fields.iter().position(|field| field.id == id) {
                    state.selected = index;
                }
                self.authoring = Some(state);
                has_changed
            }
            Err(error) => {
                state.error = Some(error);
                self.authoring = Some(state);
                false
            }
        }
    }

    pub fn cancel_authoring(&mut self) {
        if let Some(state) = &mut self.authoring
            && state.draft.is_some()
        {
            state.draft = None;
            state.error = None;
            state.refresh(&self.workflow);
            return;
        }
        self.authoring = None;
    }

    /// Delete selected nodes through the aggregate workflow lifecycle operation.
    pub(crate) fn delete_selected_nodes(&mut self) -> usize {
        let mut node_ids = self.graph_state.selected.iter().copied().collect::<Vec<_>>();
        node_ids.sort_unstable();
        if node_ids.is_empty() {
            return 0;
        }
        if node_ids.iter().any(|node_id| self.workflow.plan_node_removal(*node_id).is_err()) {
            self.output.push_line("Node deletion failed; selection is stale");
            return 0;
        }

        let mut candidate = self.workflow.clone();
        for node_id in &node_ids {
            if let Err(error) = candidate.remove_node(*node_id) {
                self.output.push_line(&format!("Node deletion rejected: {error:?}"));
                return 0;
            }
        }
        let removed = node_ids.len();
        self.workflow = candidate;
        debug_assert_eq!(removed, node_ids.len());
        self.property_edit = self.property_edit.take().filter(|edit| !node_ids.contains(&edit.node_id));
        if self
            .authoring
            .as_ref()
            .and_then(|state| state.node_id)
            .is_some_and(|id| node_ids.iter().any(|node| node.0 == id))
        {
            self.authoring = None;
        }
        self.sync_graph_state();
        self.mark_workflow_dirty();
        self.output.push_line(&format!("Deleted {removed} node(s)"));
        removed
    }
}
