//! Stored workflow browsing and loading.

use super::*;

impl State {
    /// Return whether the workflow browser modal is open.
    pub fn is_workflow_browser_open(&self) -> bool {
        self.workflow_browser.open
    }

    /// Open the saved-workflow browser.
    pub fn open_workflow_browser(&mut self) {
        self.close_workflow_template_browser();
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
        let Some(last_index) = self.workflow_browser.names.len().checked_sub(1) else {
            return;
        };
        self.workflow_browser.selected = self.workflow_browser.selected.saturating_add(1).min(last_index);
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
                self.authoring = None;
                self.property_edit = None;
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
}
