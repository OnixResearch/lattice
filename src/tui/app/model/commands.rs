//! Command palette catalog and action mapping.

use super::*;

/// High-level actions produced by input handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    ConfirmQuit,
    CancelQuit,
    RunWorkflow,
    CancelRun,
    OpenRunHistory,
    OpenApprovals,
    OpenDiagnostics,
    SaveWorkflow,
    AddNode,
    OpenWorkflow,
    OpenWorkflowTemplates,
    AutoLayout,
    EditNodeLabel,
    EditNodeCommand,
    EditNodeTimeout,
    OpenAuthoring(&'static str),
}

pub struct PaletteData {
    pub items: Vec<FuzzyCandidate>,
}

impl PaletteData {
    pub(in crate::tui::app) fn new() -> Self {
        let entries = [
            ("add_node", "Add Node (all built-in kinds)", Some('+')),
            ("edit_command", "Edit Selected Node Command (e)", Some('✎')),
            ("edit_label", "Rename Selected Node (l)", Some('✎')),
            ("edit_timeout", "Edit Selected Node Timeout (t)", Some('✎')),
            ("run", "Run Workflow (Ctrl-R)", Some('▶')),
            ("cancel_run", "Cancel Run (Ctrl-X)", Some('■')),
            ("history", "Workflow History (Ctrl-H)", Some('◷')),
            ("approvals", "Workflow Approvals", Some('✓')),
            ("diagnostics", "Workflow Diagnostics (Ctrl-D)", Some('!')),
            ("save", "Save Workflow (Ctrl-S)", Some('💾')),
            ("open", "Open Workflow (Ctrl-O)", Some('📂')),
            ("workflow_templates", "Browse Workflow Templates", Some('▦')),
            ("layout", "Auto-Layout (Ctrl-L)", Some('⊞')),
            ("authoring_all", "Author Workflow / All Typed Fields (Ctrl-W)", Some('✎')),
            ("authoring_workflow", "Author Workflow Metadata / Timestamps", Some('✎')),
            ("authoring_config", "Author Selected Node Configuration / Maps / Mappings", Some('✎')),
            ("authoring_graph", "Author Node Position / Typed Ports / Edges", Some('✎')),
            ("authoring_credential", "Author Opaque Credential References", Some('✎')),
            ("authoring_runner", "Author Runtime Binding / Runner Policies", Some('✎')),
            ("authoring_trigger", "Author Manual / Schedule / Webhook Triggers", Some('✎')),
            ("authoring_operation", "Author Admitted Operation / Catalog / Typed Arguments", Some('✎')),
            ("quit", "Quit (Ctrl-Q)", Some('✕')),
        ];
        let items = entries
            .into_iter()
            .map(|(id, text, icon)| FuzzyCandidate {
                id: id.into(),
                text: text.into(),
                context: None,
                icon,
            })
            .collect();
        Self { items }
    }
}

impl FuzzySource for PaletteData {
    fn candidates(&self) -> &[FuzzyCandidate] {
        &self.items
    }
}

/// Resolve a palette selection to an AppAction.
pub fn palette_action(id: &str) -> Option<Action> {
    match id {
        "add_node" => Some(Action::AddNode),
        "run" => Some(Action::RunWorkflow),
        "cancel_run" => Some(Action::CancelRun),
        "history" => Some(Action::OpenRunHistory),
        "approvals" => Some(Action::OpenApprovals),
        "diagnostics" => Some(Action::OpenDiagnostics),
        "save" => Some(Action::SaveWorkflow),
        "open" => Some(Action::OpenWorkflow),
        "workflow_templates" => Some(Action::OpenWorkflowTemplates),
        "layout" => Some(Action::AutoLayout),
        "edit_label" => Some(Action::EditNodeLabel),
        "edit_command" => Some(Action::EditNodeCommand),
        "edit_timeout" => Some(Action::EditNodeTimeout),
        "authoring_all" => Some(Action::OpenAuthoring("all")),
        "authoring_workflow" => Some(Action::OpenAuthoring("workflow")),
        "authoring_config" => Some(Action::OpenAuthoring("config")),
        "authoring_graph" => Some(Action::OpenAuthoring("graph")),
        "authoring_credential" => Some(Action::OpenAuthoring("credential")),
        "authoring_runner" => Some(Action::OpenAuthoring("runner")),
        "authoring_trigger" => Some(Action::OpenAuthoring("trigger")),
        "authoring_operation" => Some(Action::OpenAuthoring("operation")),
        "quit" => Some(Action::Quit),
        _ => None,
    }
}
