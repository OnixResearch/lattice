//! Guided node entry state and template parameters.

use super::*;

pub fn normalized_node_label(input: impl AsRef<str>, default_label: impl AsRef<str>) -> String {
    let trimmed = input.as_ref().trim();
    if trimmed.is_empty() {
        default_label.as_ref().to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn template_parameter_values(template: &NodeTemplate, values: &[String]) -> HashMap<String, String> {
    template
        .parameters
        .iter()
        .zip(values.iter())
        .map(|(parameter, value)| (parameter.name.clone(), value.clone()))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddNodeStep {
    Kind,
    TemplateSelect,
    TemplateParameter,
    Label,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddNodeKind {
    Shell,
    Http,
    File,
    ImageInput,
    PiAgent,
    ClankersAgent,
    Condition,
    Merge,
    Subworkflow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddNodeFlowState {
    pub open: bool,
    pub step: AddNodeStep,
    pub kind: AddNodeKind,
    pub label: String,
    pub command: String,
    pub selected_template: usize,
    pub parameter_values: Vec<String>,
    pub parameter_index: usize,
}

impl AddNodeFlowState {
    pub(in crate::tui::app) fn closed() -> Self {
        Self {
            open: false,
            step: AddNodeStep::Label,
            kind: AddNodeKind::Shell,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    pub(in crate::tui::app) fn kind() -> Self {
        Self {
            open: true,
            step: AddNodeStep::Kind,
            kind: AddNodeKind::Shell,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    pub(in crate::tui::app) fn shell(default_label: String) -> Self {
        Self {
            open: true,
            step: AddNodeStep::Label,
            kind: AddNodeKind::Shell,
            label: default_label,
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    pub(in crate::tui::app) fn typed(kind: AddNodeKind, default_label: String) -> Self {
        Self {
            open: true,
            step: AddNodeStep::Label,
            kind,
            label: default_label,
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }

    pub(in crate::tui::app) fn templates() -> Self {
        Self {
            open: true,
            step: AddNodeStep::TemplateSelect,
            kind: AddNodeKind::Shell,
            label: String::new(),
            command: String::new(),
            selected_template: 0,
            parameter_values: Vec::new(),
            parameter_index: 0,
        }
    }
}
