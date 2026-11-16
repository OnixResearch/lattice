//! Run history inspection and replay selection.

use super::*;

impl State {
    /// Return whether the run-history modal is open.
    pub fn is_run_history_open(&self) -> bool {
        self.run_history.open
    }

    /// Open recent run history.
    pub fn open_run_history(&mut self) {
        self.close_workflow_template_browser();
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.diagnostics.open = false;
        self.add_node_flow = AddNodeFlowState::closed();
        self.approvals = ApprovalState::closed();
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
        let Some(last_index) = self.run_history.records.len().checked_sub(1) else {
            return;
        };
        if self.run_history.replay_timeline.is_some() {
            self.run_history.replay_cursor = crate::replay::move_replay_cursor(self.run_history.replay_cursor, 1);
            return;
        }
        if let Some(detail_record) = &self.run_history.detail_record {
            let last_node_index = detail_record.nodes.len().saturating_sub(1);
            self.run_history.selected_detail_node =
                self.run_history.selected_detail_node.saturating_add(1).min(last_node_index);
            return;
        }
        self.run_history.selected = self.run_history.selected.saturating_add(1).min(last_index);
        self.run_history.detail = None;
    }

    pub fn select_previous_run_history(&mut self) {
        if self.run_history.replay_timeline.is_some() {
            self.run_history.replay_cursor = crate::replay::move_replay_cursor(self.run_history.replay_cursor, -1);
            return;
        }
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
        let run_id = record.run_id;
        self.run_history.detail = Some(format_run_record(record));
        self.run_history.selected_detail_node = 0;

        let Some(store) = &self.store else {
            let message = "No database available".to_string();
            self.run_history.detail = Some(message.clone());
            self.run_history.message = Some(message);
            self.output.push_line("No database available");
            return;
        };
        let load = crate::tui::app::model::receipts::classify_run_detail_load(
            store.load_run_details(run_id).map_err(|error| error.to_string()),
        );
        self.apply_run_detail_load(load);
        self.load_selected_run_replay(run_id);
    }

    pub(in crate::tui::app) fn apply_run_detail_load(&mut self, load: crate::tui::app::model::receipts::RunDetailLoad) {
        match load {
            crate::tui::app::model::receipts::RunDetailLoad::Loaded(record) => {
                self.run_history.detail_record = Some(record);
                self.run_history.message = None;
            }
            crate::tui::app::model::receipts::RunDetailLoad::Missing => {
                let message = "No run details available".to_string();
                self.run_history.detail_record = None;
                self.run_history.detail = Some(message.clone());
                self.run_history.message = Some(message);
            }
            crate::tui::app::model::receipts::RunDetailLoad::Failed(error) => {
                let message = format!("Run details failed: {error}");
                self.output.push_line(&message);
                self.run_history.detail_record = None;
                self.run_history.detail = Some(message.clone());
                self.run_history.message = Some(message);
            }
        }
    }

    // r[impl n8nclone.tui_ux.replay_debugger.timeline]
    pub(in crate::tui::app) fn load_selected_run_replay(&mut self, run_id: RunId) {
        let Some(store) = &self.store else {
            return;
        };
        match store.load_run_replay_events(run_id) {
            Ok(Some(events)) => {
                let timeline = crate::replay::build_replay_timeline(run_id, &events, None);
                self.run_history.replay_cursor = crate::replay::replay_cursor(timeline.frames.len());
                self.run_history.replay_timeline = Some(timeline);
            }
            Ok(None) => {
                self.run_history.replay_timeline = None;
                self.run_history.replay_cursor = crate::replay::replay_cursor(0);
            }
            Err(error) => {
                let message = format!("Replay load failed: {error}");
                self.output.push_line(&message);
                self.run_history.replay_timeline = None;
                self.run_history.replay_cursor = crate::replay::replay_cursor(0);
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
}
