//! Transactional typed node-field admission against a workflow snapshot.

use super::*;

impl State {
    pub(in crate::tui::app) fn apply_property_edit(
        &mut self,
        edit: &PropertyEditState,
    ) -> Result<(), PropertyEditError> {
        if crate::workflow_format::workflow_to_portable(&self.workflow) != edit.workflow_snapshot
            || crate::authoring::identity("lattice.tui-authoring-snapshot.v1", &self.workflow).ok()
                != edit.authoring_snapshot
        {
            return Err(PropertyEditError::Typed("Workflow changed; cancel the stale draft and reopen".into()));
        }
        let mut candidate = self.workflow.clone();
        let node_id = edit.node_id.0;
        let field_id = match edit.field {
            PropertyEditField::Credentials => {
                replace_credentials(&mut candidate, edit)?;
                None
            }
            PropertyEditField::ConditionBranches => {
                candidate
                    .replace_condition_branches(
                        edit.node_id,
                        crate::tui::app::editing::parsing::condition_branches_from_draft(&edit.draft)?,
                    )
                    .map_err(|error| PropertyEditError::Typed(format!("{error:?}")))?;
                None
            }
            PropertyEditField::AgentContextRefs => {
                replace_context(&mut candidate, edit)?;
                None
            }
            PropertyEditField::Label => Some("graph.label"),
            PropertyEditField::Command => Some("config.shell.command"),
            PropertyEditField::TimeoutSecs => {
                validate_timeout_secs(&edit.draft)?;
                Some(match candidate.configs.get(&node_id) {
                    Some(NodeConfig::Shell { .. }) => "config.shell.timeout_secs",
                    Some(NodeConfig::Http { .. }) => "config.http.timeout_secs",
                    _ => return Err(PropertyEditError::MissingConfig),
                })
            }
            PropertyEditField::ImagePath => Some("config.image_input.path"),
            PropertyEditField::AgentPrompt => Some("config.agent.prompt"),
            PropertyEditField::AgentTarget => {
                validate_agent_target(&edit.draft)?;
                Some("config.agent.target")
            }
            PropertyEditField::AgentOutputMode => {
                validate_agent_output_mode(&edit.draft)?;
                Some("config.agent.output_mode")
            }
            PropertyEditField::AgentModelHint => Some("config.agent.model_hint"),
            PropertyEditField::AgentToolPolicy => Some("config.agent.tool_policy"),
            PropertyEditField::AgentCwd => Some("config.agent.cwd"),
            PropertyEditField::AgentSession => Some("config.agent.session"),
        };
        if let Some(field_id) = field_id {
            crate::tui::authoring::apply(&mut candidate, Some(node_id), field_id, &edit.draft)
                .map_err(PropertyEditError::Typed)?;
        }
        crate::authoring::reconcile_materialized(&mut candidate)
            .map_err(|error| PropertyEditError::Typed(error.into()))?;
        self.workflow = candidate;
        self.sync_graph_state();
        self.graph_state.selected.insert(edit.node_id);
        self.graph_state.focused = Some(edit.node_id);
        Ok(())
    }
}

fn replace_credentials(candidate: &mut Workflow, edit: &PropertyEditState) -> Result<(), PropertyEditError> {
    let node_id = edit.node_id.0;
    let refs = crate::tui::app::editing::parsing::credentials_from_draft(&edit.draft)?;
    if candidate.graph.node(crate::engine::workflow::NodeId(node_id)).is_none() {
        return Err(PropertyEditError::NodeNotFound);
    }
    let names = candidate
        .node_credentials(edit.node_id)
        .map(|refs| refs.env.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    for name in names {
        crate::tui::authoring::workflow::apply(candidate, Some(node_id), &format!("credential.env.row.{name}"), "")
            .map_err(PropertyEditError::Typed)?;
    }
    for (name, reference) in refs.env {
        add_credential(candidate, node_id, &name, &reference)?;
    }
    Ok(())
}

fn add_credential(
    candidate: &mut Workflow,
    node_id: u64,
    name: &str,
    reference: &SecretRef,
) -> Result<(), PropertyEditError> {
    let row = crate::tui::authoring::workflow::row(&[
        name,
        &reference.provider,
        &reference.key,
        reference.field.as_deref().unwrap_or(""),
    ]);
    crate::tui::authoring::workflow::apply(candidate, Some(node_id), "credential.env.add", &row)
        .map_err(PropertyEditError::Typed)
}

fn replace_context(candidate: &mut Workflow, edit: &PropertyEditState) -> Result<(), PropertyEditError> {
    let node_id = edit.node_id.0;
    let Some(NodeConfig::Agent(settings)) = candidate.configs.get(&node_id) else {
        return Err(PropertyEditError::MissingConfig);
    };
    let context_ref_count = settings.context_refs.len();
    for _ in 0..context_ref_count {
        crate::tui::authoring::node::apply(candidate, node_id, "config.agent.context_refs.0", "")
            .map_err(PropertyEditError::Typed)?;
    }
    for value in crate::tui::app::editing::parsing::context_refs_from_draft(&edit.draft) {
        crate::tui::authoring::node::apply(candidate, node_id, "config.agent.context_refs.add", &value)
            .map_err(PropertyEditError::Typed)?;
    }
    Ok(())
}
