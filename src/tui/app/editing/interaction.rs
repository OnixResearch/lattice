//! Transactional selected-node property editing.

use super::*;

impl State {
    /// Start editing a selected node property.
    // r[impl n8nclone.tui_ux.node_kind_provider.unsupported_edit]
    pub fn start_property_edit(&mut self, field: PropertyEditField) -> bool {
        let Some(node_id) = self.selected_node_id() else {
            self.output.push_line("Select one node to edit properties");
            return false;
        };

        let Some(draft) = self.property_value(node_id, field) else {
            self.output.push_line("Selected node property is unavailable");
            return false;
        };

        self.property_edit = Some(PropertyEditState {
            node_id,
            field,
            original: draft.clone(),
            workflow_snapshot: crate::workflow_format::workflow_to_portable(&self.workflow),
            draft,
        });
        true
    }

    /// Append one character to the active property draft.
    pub fn push_property_edit_char(&mut self, ch: char) {
        if let Some(edit) = &mut self.property_edit {
            edit.draft.push(ch);
        }
    }

    /// Remove one character from the active property draft.
    pub fn pop_property_edit_char(&mut self) {
        if let Some(edit) = &mut self.property_edit {
            edit.draft.pop();
        }
    }

    /// Cancel active property editing without mutating workflow state.
    pub fn cancel_property_edit(&mut self) {
        self.property_edit = None;
    }

    /// Save active property editing, preserving previous state on validation failure.
    pub fn save_property_edit(&mut self) -> bool {
        let Some(edit) = self.property_edit.clone() else {
            return false;
        };

        match self.apply_property_edit(&edit) {
            Ok(()) => {
                self.property_edit = None;
                self.mark_workflow_dirty();
                self.output.push_line(&format!("Updated node {}", edit.field.label()));
                true
            }
            Err(PropertyEditError::InvalidTimeout) => {
                self.output.push_line("Invalid timeout; enter whole seconds");
                false
            }
            Err(PropertyEditError::InvalidAgentTarget) => {
                self.output.push_line("Invalid agent target; enter pi, clankers, or animus");
                false
            }
            Err(PropertyEditError::InvalidAgentOutputMode) => {
                self.output.push_line("Invalid agent output mode; enter text, json, image, or artifact");
                false
            }
            Err(PropertyEditError::InvalidAgentCredentials) => {
                self.output.push_line("Invalid credentials; use ENV=provider:key[, ENV=provider:key.field]");
                false
            }
            Err(PropertyEditError::InvalidConditionBranches) => {
                self.output.push_line("Invalid condition branches; use label=predicate entries");
                false
            }
            Err(PropertyEditError::InvalidMergePolicy) => {
                self.output.push_line("Invalid merge policy; use all|first|last|collect");
                false
            }
            Err(PropertyEditError::Typed(message)) => {
                self.output.push_line(&message);
                false
            }
            Err(_) => {
                self.output.push_line("Property edit failed");
                false
            }
        }
    }

    pub(in crate::tui::app) fn selected_node_id(&self) -> Option<NodeId> {
        if self.graph_state.selected.len() != 1 {
            return None;
        }
        self.graph_state.selected.iter().next().copied()
    }

    pub(in crate::tui::app) fn property_value(&self, node_id: NodeId, field: PropertyEditField) -> Option<String> {
        match field {
            PropertyEditField::Label => self.workflow.graph.node(node_id).map(|node| node.label.clone()),
            PropertyEditField::Command => command_text(self.workflow.node_config(node_id)?),
            PropertyEditField::TimeoutSecs => timeout_text(self.workflow.node_config(node_id)?),
            PropertyEditField::ImagePath => match self.workflow.node_config(node_id)? {
                NodeConfig::ImageInput { path, .. } => Some(path.clone()),
                _ => None,
            },
            PropertyEditField::AgentPrompt => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.prompt.clone()),
                _ => None,
            },
            PropertyEditField::AgentTarget => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(agent_node::target_label(settings.target).to_string()),
                _ => None,
            },
            PropertyEditField::AgentOutputMode => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.output_mode.as_str().to_string()),
                _ => None,
            },
            PropertyEditField::AgentModelHint => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.model_hint.clone().unwrap_or_default()),
                _ => None,
            },
            PropertyEditField::AgentToolPolicy => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.tool_policy.clone()),
                _ => None,
            },
            PropertyEditField::AgentContextRefs => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.context_refs.join(", ")),
                _ => None,
            },
            PropertyEditField::AgentCwd => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.cwd.clone().unwrap_or_default()),
                _ => None,
            },
            PropertyEditField::AgentSession => match self.workflow.node_config(node_id)? {
                NodeConfig::Agent(settings) => Some(settings.session.clone().unwrap_or_default()),
                _ => None,
            },
            PropertyEditField::Credentials => {
                self.workflow.node_config(node_id)?;
                Some(crate::tui::app::editing::parsing::credentials_to_draft(self.workflow.node_credentials(node_id)))
            }
            PropertyEditField::ConditionBranches => match self.workflow.node_config(node_id)? {
                NodeConfig::Condition(config) => {
                    Some(crate::tui::app::editing::parsing::condition_branches_to_draft(config))
                }
                _ => None,
            },
        }
    }
}

fn command_text(config: &NodeConfig) -> Option<String> {
    match config {
        NodeConfig::Shell { command, .. } => Some(command.clone()),
        NodeConfig::Http { .. }
        | NodeConfig::File { .. }
        | NodeConfig::ImageInput { .. }
        | NodeConfig::Agent(_)
        | NodeConfig::Operation(_)
        | NodeConfig::Condition(_)
        | NodeConfig::Merge(_)
        | NodeConfig::Subworkflow(_)
        | NodeConfig::Approval(_) => None,
    }
}

fn timeout_text(config: &NodeConfig) -> Option<String> {
    match config {
        NodeConfig::Shell { timeout_secs, .. } | NodeConfig::Http { timeout_secs, .. } => {
            Some(timeout_secs.to_string())
        }
        NodeConfig::File { .. }
        | NodeConfig::ImageInput { .. }
        | NodeConfig::Agent(_)
        | NodeConfig::Operation(_)
        | NodeConfig::Condition(_)
        | NodeConfig::Merge(_)
        | NodeConfig::Subworkflow(_)
        | NodeConfig::Approval(_) => None,
    }
}
