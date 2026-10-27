//! Typed authoring descriptors and transient, effect-free editor drafts.
//!
//! The terminal shell never evaluates Nickel, resolves a secret, or dispatches a
//! runtime while browsing or applying these fields. Owner modules parse values
//! and validate a candidate before replacing the workflow.

pub mod catalog;
pub mod metadata;
pub mod node;
pub mod operation;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
pub mod workflow;

use crate::engine::Workflow;
use crate::workflow_format::PortableWorkflow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub id: String,
    pub label: String,
    pub help: String,
    pub value: String,
    pub spec: metadata::Spec,
}

pub fn field(id: &str, label: &str, help: &str, value: impl Into<String>) -> Field {
    Field {
        id: id.into(),
        label: label.into(),
        help: help.into(),
        value: value.into(),
        spec: metadata::spec(id),
    }
}

#[derive(Debug, Clone)]
pub struct Draft {
    pub field: Field,
    pub text: String,
    pub original: PortableWorkflow,
}

#[derive(Debug, Clone)]
pub struct State {
    pub node_id: Option<u64>,
    pub family: &'static str,
    pub fields: Vec<Field>,
    pub selected: usize,
    pub draft: Option<Draft>,
    pub error: Option<String>,
    pub operation: operation::Builder,
}

impl State {
    pub fn new(workflow: &Workflow, node_id: Option<u64>, family: &'static str) -> Self {
        let mut state = Self {
            node_id,
            family,
            fields: Vec::new(),
            selected: 0,
            draft: None,
            error: None,
            operation: operation::Builder::default(),
        };
        state.refresh(workflow);
        state
    }

    pub fn refresh(&mut self, workflow: &Workflow) {
        self.fields = fields(workflow, self.node_id, self.family);
        if self.family == "all" || self.family == "operation" {
            let replace = self.node_id.is_some_and(|id| {
                matches!(workflow.configs.get(&id), Some(crate::engine::workflow::NodeConfig::Operation(_)))
            });
            self.fields.extend(self.operation.fields(replace));
        }
        self.selected = self.selected.min(self.fields.len().saturating_sub(1));
    }

    pub fn begin(&mut self, workflow: &Workflow) {
        if let Some(field) = self.fields.get(self.selected).cloned() {
            self.draft = Some(Draft {
                text: field.value.clone(),
                field,
                original: crate::workflow_format::workflow_to_portable(workflow),
            });
            self.error = None;
        }
    }
}

pub fn fields(workflow: &Workflow, node_id: Option<u64>, family: &str) -> Vec<Field> {
    let mut fields = workflow::fields(workflow, node_id);
    if let Some(node_id) = node_id {
        fields.extend(node::fields(workflow, node_id));
    }
    fields.push(field(
        "config.approval.add",
        "Add approval gate",
        "label|gate_id|local_actor; finite defaults are editable after creation",
        "",
    ));
    if family != "all" {
        let prefix = format!("{family}.");
        fields.retain(|field| field.id.starts_with(&prefix));
    }
    fields
}

pub fn apply(workflow: &mut Workflow, node_id: Option<u64>, id: &str, value: &str) -> Result<(), String> {
    if id == "config.approval.add" {
        let parts = value.split('|').map(str::trim).collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err("Use label|gate_id|local_actor".into());
        }
        return node::create_approval(workflow, parts[0], parts[1], parts[2]).map(|_| ());
    }
    if id.starts_with("config.") {
        node::apply(workflow, node_id.ok_or("Select exactly one node")?, id, value)
    } else {
        workflow::apply(workflow, node_id, id, value)
    }
}
