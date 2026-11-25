//! Durable workflow approval interaction.

use super::*;

impl State {
    pub fn is_approval_list_open(&self) -> bool {
        self.approvals.open
    }

    pub fn open_approvals(&mut self) {
        self.close_workflow_template_browser();
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.add_node_flow = AddNodeFlowState::closed();
        self.run_history = RunHistoryState::closed();
        self.diagnostics = DiagnosticsState::closed();
        let Some(store) = &self.store else {
            self.approvals = ApprovalState::open(Vec::new(), Some("No database available".to_string()));
            return;
        };
        match crate::workflow::gate::shell::list_safe(store, APPROVAL_LIST_LIMIT) {
            Ok(records) => {
                let message = records.is_empty().then(|| "No workflow approvals available".to_string());
                self.approvals = ApprovalState::open(records, message);
            }
            Err(_) => {
                self.approvals =
                    ApprovalState::open(Vec::new(), Some("Workflow approval inspection failed".to_string()));
            }
        }
    }

    pub fn close_approvals(&mut self) {
        self.approvals = ApprovalState::closed();
    }

    pub fn select_next_approval(&mut self) {
        if !self.approvals.records.is_empty() {
            self.approvals.selected =
                self.approvals.selected.saturating_add(1).min(self.approvals.records.len().saturating_sub(1));
            self.approvals.detail = None;
        }
    }

    pub fn select_previous_approval(&mut self) {
        self.approvals.selected = self.approvals.selected.saturating_sub(1);
        self.approvals.detail = None;
    }

    pub fn inspect_selected_approval(&mut self) {
        self.approvals.detail = self
            .approvals
            .records
            .get(self.approvals.selected)
            .map(crate::workflow::gate::shell::render_safe_text);
    }

    pub fn decide_selected_approval(&mut self, command: &crate::workflow::gate::DecisionCommand) -> bool {
        let Some(selected) = self.approvals.records.get(self.approvals.selected) else {
            self.approvals.message = Some("No workflow approval is selected".to_string());
            return false;
        };
        if selected.request_id != command.request_id.as_str() {
            self.approvals.message = Some("Workflow approval selection changed".to_string());
            return false;
        }
        let Some(store) = self.store.clone() else {
            self.approvals.message = Some("No database available".to_string());
            return false;
        };
        match crate::workflow::gate::shell::decide(&store, command) {
            Ok(_) => {
                self.output.push_line("Workflow approval decision recorded; execution remains suspended");
                self.open_approvals();
                true
            }
            Err(_) => {
                self.approvals.message = Some("Workflow approval decision failed or became stale".to_string());
                false
            }
        }
    }

    pub async fn resume_selected_approval(&mut self, command: &crate::workflow::gate::ResumeCommand) -> bool {
        let Some(selected) = self.approvals.records.get(self.approvals.selected) else {
            self.approvals.message = Some("No workflow approval is selected".to_string());
            return false;
        };
        if selected.request_id != command.request_id.as_str() {
            self.approvals.message = Some("Workflow approval selection changed".to_string());
            return false;
        }
        let Some(store) = self.store.clone() else {
            self.approvals.message = Some("No database available".to_string());
            return false;
        };
        let authority_provider = Arc::clone(&self.authority_provider);
        let result = crate::workflow::gate::shell::resume_with_app_config_and_authority_provider(
            &store,
            command,
            &self.config,
            authority_provider.as_ref(),
        )
        .await;
        match result {
            Ok(execution) => {
                self.output.push_line(&format!(
                    "Workflow resume completed with {:?}; downstream work ran only after the separate resume action",
                    execution.summary.status
                ));
                self.open_approvals();
                true
            }
            Err(_) => {
                self.approvals.message = Some("Workflow approval resume failed or became uncertain".to_string());
                false
            }
        }
    }
}
