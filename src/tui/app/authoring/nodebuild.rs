//! Typed draft admission into configured workflow nodes.

use super::*;

impl State {
    pub(in crate::tui::app) fn confirm_shell_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_shell_node_label());
        let command = self.add_node_flow.command.clone();
        self.commit_guided_node(&label, NodeConfig::shell(command));
    }

    // r[impl lattice.tui_ux.portable_authoring_parity.node_kinds]
    pub(in crate::tui::app) fn confirm_http_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_http_node_label());
        let (method, url) = match crate::tui::app::editing::parsing::http_config_from_draft(&self.add_node_flow.command)
        {
            Ok(config) => config,
            Err(_) => {
                self.output.push_line("Invalid HTTP config; use METHOD URL");
                return;
            }
        };
        self.commit_guided_node(&label, NodeConfig::http(method, url));
    }

    pub(in crate::tui::app) fn confirm_file_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_file_node_label());
        let (operation, path) =
            match crate::tui::app::editing::parsing::file_config_from_draft(&self.add_node_flow.command) {
                Ok(config) => config,
                Err(_) => {
                    self.output.push_line("Invalid file config; use read|write|append PATH");
                    return;
                }
            };
        if let Err(error) = crate::tui::authoring::node::path_text(&path) {
            self.output.push_line(&error);
            return;
        }
        self.commit_guided_node(&label, NodeConfig::file(operation, path));
    }

    pub(in crate::tui::app) fn confirm_image_input_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_image_node_label());
        let path = self.add_node_flow.command.clone();
        if let Err(error) = crate::tui::authoring::node::path_text(&path) {
            self.output.push_line(&error);
            return;
        }
        self.commit_guided_node(&label, crate::tui::app::editing::parsing::image_input_config(path));
    }

    pub(in crate::tui::app) fn confirm_agent_node_from_flow(&mut self, target: crate::agent_node::Target) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_agent_node_label());
        let prompt = self.add_node_flow.command.clone();
        self.commit_guided_node(&label, NodeConfig::agent(target, prompt, crate::agent_node::OutputMode::Image));
    }

    pub(in crate::tui::app) fn confirm_condition_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_condition_node_label());
        let branches =
            match crate::tui::app::editing::parsing::condition_branches_from_draft(&self.add_node_flow.command) {
                Ok(branches) => branches,
                Err(_) => {
                    self.output.push_line("Invalid condition branches; use label=predicate entries");
                    return;
                }
            };
        self.commit_guided_node(&label, NodeConfig::condition(ConditionNodeConfig { branches }));
    }

    pub(in crate::tui::app) fn confirm_merge_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_merge_node_label());
        let config = match crate::tui::app::editing::parsing::merge_config_from_draft(&self.add_node_flow.command) {
            Ok(config) => config,
            Err(_) => {
                self.output.push_line("Invalid merge policy; use all|first|last|collect:input_a,input_b");
                return;
            }
        };
        self.commit_guided_node(&label, NodeConfig::merge(config));
    }

    pub(in crate::tui::app) fn confirm_subworkflow_node_from_flow(&mut self) {
        let label = normalized_node_label(&self.add_node_flow.label, self.next_subworkflow_node_label());
        let workflow_name = self.add_node_flow.command.trim();
        if workflow_name.is_empty() {
            self.output.push_line("Subworkflow name is required");
            return;
        }
        let config = SubworkflowNodeConfig {
            workflow_name: workflow_name.to_string(),
            input_mappings: Vec::new(),
            output_mappings: Vec::new(),
        };
        self.commit_guided_node(&label, NodeConfig::subworkflow(config));
    }
}
