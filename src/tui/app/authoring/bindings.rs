//! Node template binding and default labels.

use super::*;

impl State {
    pub(in crate::tui::app) fn confirm_template_node(&mut self) {
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

    pub(in crate::tui::app) fn start_template_parameter_flow(&mut self, template: &NodeTemplate) {
        self.add_node_flow.step = AddNodeStep::TemplateParameter;
        self.add_node_flow.parameter_index = 0;
        self.add_node_flow.parameter_values = template
            .parameters
            .iter()
            .map(|parameter| parameter.default.as_ref().map_or_else(String::new, |default| default.as_raw_string()))
            .collect();
    }

    pub(in crate::tui::app) fn confirm_template_parameter(&mut self) {
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

    pub(in crate::tui::app) fn add_template_node(&mut self, template: &NodeTemplate) -> bool {
        if template.kind != "shell" {
            self.output.push_line(&format!("Unsupported template kind: {}", template.kind));
            return false;
        }

        let inputs = template
            .input_ports
            .iter()
            .map(|port| crate::engine::workflow::PortSpec::new(port.label.clone(), port.type_tag.clone()))
            .collect::<Vec<_>>();
        let outputs = template
            .output_ports
            .iter()
            .map(|port| crate::engine::workflow::PortSpec::new(port.label.clone(), port.type_tag.clone()))
            .collect::<Vec<_>>();
        let parameter_values = template_parameter_values(template, &self.add_node_flow.parameter_values);
        let command = match render_template_command(template, &parameter_values) {
            Ok(command) => command,
            Err(error) => {
                self.output.push_line(&format!("Template parameter error: {error}"));
                return false;
            }
        };
        let config = NodeConfig::shell(command);
        let issues = node_kind_provider::provider_for_config(&config).validate_config(&template.name, &config);
        if !issues.is_empty() {
            for issue in issues {
                self.output.push_line(&issue.message);
            }
            return false;
        }
        let mut candidate = self.workflow.clone();
        let id = match candidate.graph.try_add_node(template.name.clone(), &inputs, &outputs) {
            Ok(id) => id,
            Err(error) => {
                self.output.push_line(&format!("Template creation rejected: {error}"));
                return false;
            }
        };
        candidate.set_node_config(id, config);
        if let Err(error) = candidate.validate_aggregate_ownership() {
            self.output.push_line(&format!("Template creation rejected: {error:?}"));
            return false;
        }
        self.workflow = candidate;
        self.place_new_node(id);
        self.sync_graph_state();
        self.select_node(id);
        self.mark_workflow_dirty();
        true
    }

    pub(in crate::tui::app) fn next_shell_node_label(&self) -> String {
        format!("Shell {}", self.workflow.graph.node_count().saturating_add(1))
    }

    pub(in crate::tui::app) fn next_http_node_label(&self) -> String {
        format!("HTTP {}", self.workflow.graph.node_count().saturating_add(1))
    }

    pub(in crate::tui::app) fn next_file_node_label(&self) -> String {
        format!("File {}", self.workflow.graph.node_count().saturating_add(1))
    }

    pub(in crate::tui::app) fn next_image_node_label(&self) -> String {
        format!("Image {}", self.workflow.graph.node_count().saturating_add(1))
    }

    pub(in crate::tui::app) fn next_agent_node_label(&self) -> String {
        format!("Agent {}", self.workflow.graph.node_count().saturating_add(1))
    }

    pub(in crate::tui::app) fn next_condition_node_label(&self) -> String {
        format!("Condition {}", self.workflow.graph.node_count().saturating_add(1))
    }

    pub(in crate::tui::app) fn next_merge_node_label(&self) -> String {
        format!("Merge {}", self.workflow.graph.node_count().saturating_add(1))
    }

    pub(in crate::tui::app) fn next_subworkflow_node_label(&self) -> String {
        format!("Subworkflow {}", self.workflow.graph.node_count().saturating_add(1))
    }
}
