//! Guided node entry and validation.

use super::*;

impl State {
    /// Start the guided add-node flow.
    pub fn open_add_node_flow(&mut self) {
        self.close_workflow_template_browser();
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

    pub fn start_http_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::Http, self.next_http_node_label());
        self.add_node_flow.command = DEFAULT_HTTP_DRAFT.to_string();
    }

    pub fn start_file_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::File, self.next_file_node_label());
        self.add_node_flow.command = DEFAULT_FILE_DRAFT.to_string();
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

    // r[impl n8nclone.tui_ux.control_flow.add_flow]
    pub fn start_condition_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::Condition, self.next_condition_node_label());
        self.add_node_flow.command = format!("{DEFAULT_CONDITION_BRANCH_LABEL}={DEFAULT_CONDITION_PREDICATE}");
    }

    // r[impl n8nclone.tui_ux.control_flow.add_flow]
    pub fn start_merge_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::Merge, self.next_merge_node_label());
        self.add_node_flow.command = format!("all:{DEFAULT_MERGE_INPUT_LEFT},{DEFAULT_MERGE_INPUT_RIGHT}");
    }

    // r[impl n8nclone.tui_ux.control_flow.add_flow]
    pub fn start_subworkflow_node_flow(&mut self) {
        self.add_node_flow = AddNodeFlowState::typed(AddNodeKind::Subworkflow, self.next_subworkflow_node_label());
    }

    /// Move add-node template selection down.
    pub fn select_next_node_template(&mut self) {
        let Some(last_index) = self.node_templates.len().checked_sub(1) else {
            return;
        };
        self.add_node_flow.selected_template = self.add_node_flow.selected_template.saturating_add(1).min(last_index);
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

    pub(in crate::tui::app) fn confirm_typed_node_from_flow(&mut self) {
        match self.add_node_flow.kind {
            AddNodeKind::Shell => self.confirm_shell_node_from_flow(),
            AddNodeKind::Http => self.confirm_http_node_from_flow(),
            AddNodeKind::File => self.confirm_file_node_from_flow(),
            AddNodeKind::ImageInput => self.confirm_image_input_node_from_flow(),
            AddNodeKind::PiAgent => self.confirm_agent_node_from_flow(crate::agent_node::Target::Pi),
            AddNodeKind::ClankersAgent => self.confirm_agent_node_from_flow(crate::agent_node::Target::Clankers),
            AddNodeKind::Condition => self.confirm_condition_node_from_flow(),
            AddNodeKind::Merge => self.confirm_merge_node_from_flow(),
            AddNodeKind::Subworkflow => self.confirm_subworkflow_node_from_flow(),
        }
    }

    pub(in crate::tui::app) fn commit_guided_node(&mut self, label: &str, config: NodeConfig) {
        let issues = node_kind_provider::provider_for_config(&config).validate_config(label, &config);
        if !issues.is_empty() {
            for issue in issues {
                self.output.push_line(&issue.message);
            }
            return;
        }
        match self.workflow.try_add_configured_node(label, config) {
            Ok(id) => {
                self.place_new_node(id);
                self.sync_graph_state();
                self.select_node(id);
                self.mark_workflow_dirty();
                self.cancel_add_node_flow();
            }
            Err(error) => self.output.push_line(&format!("Node creation rejected: {error:?}")),
        }
    }
}
