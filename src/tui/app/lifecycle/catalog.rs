//! Workflow template preview and activation.

use super::*;

impl State {
    /// Replace the configured node templates.
    pub fn set_node_templates(&mut self, templates: Vec<NodeTemplate>) {
        self.node_templates = templates;
    }

    /// Replace the validated local workflow-template catalog.
    pub fn set_workflow_templates(&mut self, templates: Vec<crate::workflow_template::ValidatedBundle>) {
        self.workflow_templates = templates;
    }

    pub fn is_workflow_template_browser_open(&self) -> bool {
        self.workflow_template_browser.open
    }

    // r[impl lattice.workflow_templates.preview]
    pub fn open_workflow_template_browser(&mut self) {
        if self.is_workflow_template_browser_open() {
            self.close_workflow_template_browser();
        }
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.add_node_flow = AddNodeFlowState::closed();
        self.run_history = RunHistoryState::closed();
        self.approvals = ApprovalState::closed();
        self.diagnostics = DiagnosticsState::closed();
        self.workflow_template_browser = WorkflowTemplateBrowserState {
            open: true,
            selected: 0,
            step: WorkflowTemplateBrowseStep::Browse,
            message: if self.workflow_templates.is_empty() {
                Some("No valid workflow templates are available".to_string())
            } else {
                None
            },
            prior_workflow: None,
            prior_dirty: self.dirty,
        };
    }

    // r[impl lattice.workflow_templates.preview]
    pub fn close_workflow_template_browser(&mut self) {
        if self.workflow_template_browser.step == WorkflowTemplateBrowseStep::Preview {
            if let Some(previous) = self.workflow_template_browser.prior_workflow.take() {
                self.workflow = previous;
                self.dirty = self.workflow_template_browser.prior_dirty;
                self.sync_graph_state();
            }
            self.workflow_template_draft = None;
        }
        self.workflow_template_browser = WorkflowTemplateBrowserState::closed();
    }

    pub fn select_next_workflow_template(&mut self) {
        if self.workflow_templates.is_empty()
            || self.workflow_template_browser.step != WorkflowTemplateBrowseStep::Browse
        {
            return;
        }
        let last_index = self.workflow_templates.len().saturating_sub(1);
        self.workflow_template_browser.selected =
            self.workflow_template_browser.selected.saturating_add(1).min(last_index);
    }

    pub fn select_previous_workflow_template(&mut self) {
        if self.workflow_template_browser.step == WorkflowTemplateBrowseStep::Browse {
            self.workflow_template_browser.selected = self.workflow_template_browser.selected.saturating_sub(1);
        }
    }

    pub fn selected_workflow_template(&self) -> Option<&crate::workflow_template::ValidatedBundle> {
        self.workflow_templates.get(self.workflow_template_browser.selected)
    }

    // r[impl lattice.workflow_templates.preview]
    pub fn confirm_workflow_template_browser(&mut self) {
        match self.workflow_template_browser.step {
            WorkflowTemplateBrowseStep::Browse => self.preview_selected_workflow_template(),
            WorkflowTemplateBrowseStep::Preview => {
                self.workflow_template_browser = WorkflowTemplateBrowserState::closed();
                self.output.push_line("Workflow template retained as an unsaved draft");
            }
        }
    }

    pub(in crate::tui::app) fn preview_selected_workflow_template(&mut self) {
        if self.dirty {
            self.workflow_template_browser.message =
                Some("Save or discard current workflow changes before template preview".to_string());
            return;
        }
        let Some(bundle) = self.selected_workflow_template().cloned() else {
            self.workflow_template_browser.message = Some("Selected workflow template is unavailable".to_string());
            return;
        };
        self.workflow_template_browser.prior_workflow = Some(self.workflow.clone());
        self.workflow_template_browser.prior_dirty = self.dirty;
        self.workflow = bundle.workflow.clone();
        self.dirty = true;
        self.workflow_template_draft = Some(WorkflowTemplateDraft {
            bundle: bundle.identity.clone(),
            workflow: bundle.workflow_revision.clone(),
            orchestration: bundle.orchestration_revision.clone(),
        });
        self.workflow_template_browser.step = WorkflowTemplateBrowseStep::Preview;
        self.workflow_template_browser.message = Some(format!(
            "Preview only: {} │ orchestration={} │ no effects",
            bundle.manifest.display_name,
            if bundle.orchestration.is_some() {
                "present"
            } else {
                "none"
            },
        ));
        self.sync_graph_state();
        debug_assert!(self.dirty);
        debug_assert!(self.workflow_template_draft.is_some());
    }
}
