//! Workflow browser and validation modal state.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsState {
    pub open: bool,
    pub diagnostics: Vec<WorkflowDiagnostic>,
}

impl DiagnosticsState {
    pub(in crate::tui::app) fn closed() -> Self {
        Self {
            open: false,
            diagnostics: Vec::new(),
        }
    }

    pub(in crate::tui::app) fn open(diagnostics: Vec<WorkflowDiagnostic>) -> Self {
        Self {
            open: true,
            diagnostics,
        }
    }
}

pub struct WorkflowBrowserState {
    pub open: bool,
    pub names: Vec<String>,
    pub selected: usize,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowTemplateBrowseStep {
    Browse,
    Preview,
}

#[derive(Debug, Clone)]
pub struct WorkflowTemplateBrowserState {
    pub open: bool,
    pub selected: usize,
    pub step: WorkflowTemplateBrowseStep,
    pub message: Option<String>,
    pub(in crate::tui::app) prior_workflow: Option<Workflow>,
    pub(in crate::tui::app) prior_dirty: bool,
}

impl WorkflowTemplateBrowserState {
    pub(in crate::tui::app) fn closed() -> Self {
        Self {
            open: false,
            selected: 0,
            step: WorkflowTemplateBrowseStep::Browse,
            message: None,
            prior_workflow: None,
            prior_dirty: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowTemplateDraft {
    pub bundle: crate::workflow_template::ContentIdentity,
    pub workflow: crate::orchestration::WorkflowRevisionRef,
    pub orchestration: Option<crate::orchestration::DefinitionRevision>,
}

impl WorkflowBrowserState {
    pub(in crate::tui::app) fn closed() -> Self {
        Self {
            open: false,
            names: Vec::new(),
            selected: 0,
            message: None,
        }
    }

    pub fn open(names: Vec<String>, message: Option<String>) -> Self {
        Self {
            open: true,
            names,
            selected: 0,
            message,
        }
    }
}
