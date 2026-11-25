//! Application construction, dirty state, and diagnostics.

use super::*;

impl State {
    /// Create a new app with an empty workflow.
    pub fn new(config: AppConfig) -> Self {
        let timestamp = crate::engine::workflow::WorkflowTimestamp::from_datetime(
            crate::operation::shell::SystemClock.observe_utc_datetime(),
        );
        let workflow = Workflow::new("untitled", timestamp);
        let graph = crate::tui::graph::empty_subwayrat();
        let graph_state = NodeGraphState::new(graph, INITIAL_VIEWPORT_WIDTH, INITIAL_VIEWPORT_HEIGHT);
        let agent_runtime = config.effective_agent_runtime();
        let authority_provider = config.effect_authority_provider();
        let frontier_policy = config.frontier_capacity_policy().unwrap_or_else(|error| {
            error!(?error, "invalid frontier policy, preserving serial execution");
            crate::engine::frontier::serial_capacity_policy()
        });
        debug_assert!(frontier_policy.total_slots > 0);
        debug_assert!(!frontier_policy.lane_slots.is_empty());

        let store = open_store(&config);

        let next_tui_run_id = store
            .as_ref()
            .and_then(|store| store.next_run_id().ok())
            .map(|run_id| run_id.0)
            .unwrap_or(INITIAL_TUI_RUN_ID);
        let palette_state = FuzzyState::new();
        let palette_data = PaletteData::new();

        let executor_config = config.clone();
        Self {
            workflow,
            graph_state,
            canvas_view: crate::tui::rows::CanvasView::Graph,
            focus: FocusPanel::Canvas,
            show_palette: false,
            palette_state,
            palette_data,
            output: StreamingOutput::new(),
            run_output_lines: Vec::new(),
            workflow_browser: WorkflowBrowserState::closed(),
            workflow_template_browser: WorkflowTemplateBrowserState::closed(),
            workflow_template_draft: None,
            property_edit: None,
            authoring: None,
            add_node_flow: AddNodeFlowState::closed(),
            node_templates: Vec::new(),
            workflow_templates: Vec::new(),
            run_state: RunState::idle(),
            run_history: RunHistoryState::closed(),
            approvals: ApprovalState::closed(),
            diagnostics: DiagnosticsState::closed(),
            store,
            active_run: None,
            next_tui_run_id,
            config,
            last_drag_canvas: None,
            rows_hit_area: None,
            rows_scroll: 0,
            dirty: false,
            confirm_quit: false,
            executor_factory: Arc::new(move || {
                let fallback_runtime = agent_runtime.clone();
                let mut executor = match Executor::new_with_agent_runtime_config(agent_runtime.clone())
                    .with_frontier_policy(frontier_policy.clone())
                {
                    Ok(executor) => executor,
                    Err(error) => {
                        error!(?error, "validated frontier policy was rejected, preserving serial execution");
                        Executor::new_with_agent_runtime_config(fallback_runtime)
                    }
                };
                if let Ok(registry) =
                    executor_config.credential_registry(&[], Arc::new(std::sync::atomic::AtomicBool::new(false)))
                {
                    executor = executor.with_credential_provider(Box::new(registry));
                }
                executor
            }),
            #[cfg(test)]
            custom_executor_factory: false,
            authority_provider,
        }
    }

    #[cfg(test)]
    pub(in crate::tui::app) fn set_executor_factory(&mut self, factory: Arc<dyn Fn() -> Executor + Send + Sync>) {
        self.executor_factory = factory;
        self.custom_executor_factory = true;
    }

    pub fn with_authority_provider(
        mut self,
        provider: Arc<dyn crate::engine::effect::execution::RunContextProvider>,
    ) -> Self {
        self.authority_provider = provider;
        self
    }

    /// Return whether the workflow has unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        self.dirty
    }

    /// Return whether quit confirmation is currently active.
    pub fn is_confirming_quit(&self) -> bool {
        self.confirm_quit
    }

    // r[impl lattice.workflow_validation.domain_boundary.validation.shell_time]
    /// Mark the current workflow as dirty after a user-visible mutation.
    pub fn mark_workflow_dirty(&mut self) {
        let timestamp = crate::engine::workflow::WorkflowTimestamp::from_datetime(
            crate::operation::shell::SystemClock.observe_utc_datetime(),
        );
        self.workflow.touch(timestamp);
        if let Err(error) = crate::authoring::reconcile_materialized(&mut self.workflow) {
            self.output.push_line(error);
        }
        self.dirty = true;
    }

    /// Clear dirty state after a successful save or clean workflow load.
    pub fn clear_workflow_dirty(&mut self) {
        self.dirty = false;
        self.confirm_quit = false;
    }

    /// Request quit. Returns true when the event loop may exit immediately.
    pub fn request_quit(&mut self) -> bool {
        if self.has_unsaved_changes() {
            self.confirm_quit = true;
            self.output.push_line("Unsaved changes. Press y/Enter to quit or n/Esc to stay.");
            false
        } else {
            true
        }
    }

    /// Cancel a pending quit confirmation.
    pub fn cancel_quit_confirmation(&mut self) {
        self.confirm_quit = false;
        self.output.push_line("Quit canceled");
    }

    /// Return whether the add-node flow is open.
    pub fn is_add_node_flow_open(&self) -> bool {
        self.add_node_flow.open
    }

    /// Return whether the diagnostics modal is open.
    pub fn is_diagnostics_open(&self) -> bool {
        self.diagnostics.open
    }

    /// Open workflow diagnostics for the current workflow snapshot.
    pub fn open_diagnostics(&mut self) {
        self.close_workflow_template_browser();
        self.show_palette = false;
        self.workflow_browser = WorkflowBrowserState::closed();
        self.add_node_flow = AddNodeFlowState::closed();
        self.run_history = RunHistoryState::closed();
        self.approvals = ApprovalState::closed();
        self.diagnostics = DiagnosticsState::open(self.validate_workflow_diagnostics());
    }

    // r[impl lattice.credentials.animus_secret_providers.tui]
    /// Return configured credential provider identifiers (`"env"` plus configured SecretSpec ids).
    pub fn configured_credential_providers(&self) -> Vec<String> {
        let mut providers = vec!["env".to_string()];
        for provider in &self.config.credential_providers {
            providers.push(provider.id().to_string());
        }
        providers
    }

    pub(in crate::tui) fn credential_registry(
        &self,
        workflow: &Workflow,
        cancellation: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<crate::credentials::CredentialProviderRegistry, crate::credentials::config::SecretSpecDiagnostic> {
        self.config.workflow_credential_registry(Some(workflow), cancellation)
    }

    pub(in crate::tui) fn validate_workflow_diagnostics(&self) -> Vec<WorkflowDiagnostic> {
        if self.config.credential_providers.is_empty() {
            return crate::workflow_validation::validate_workflow(&self.workflow);
        }
        match self.credential_registry(&self.workflow, Arc::new(std::sync::atomic::AtomicBool::new(false))) {
            Ok(registry) => {
                crate::workflow_validation::validate_workflow_with_credential_registry(&self.workflow, &registry)
            }
            Err(error) => {
                let mut diagnostics = crate::workflow_validation::validate_workflow(&self.workflow);
                diagnostics.push(WorkflowDiagnostic {
                    severity: crate::workflow_validation::DiagnosticSeverity::Fatal,
                    code: crate::workflow_validation::WorkflowDiagnosticCode::UnknownCredentialProvider,
                    node_id: None,
                    port_id: None,
                    message: error.code().to_string(),
                });
                diagnostics
            }
        }
    }

    // r[impl lattice.credentials.animus_secret_providers.tui]
    /// Safely probe a credential reference without loading SecretSpec material.
    pub fn probe_credential_reference(&self, reference: &SecretRef) -> crate::credentials::CredentialProbeResult {
        if !self.configured_credential_providers().iter().any(|id| id == &reference.provider) {
            return crate::credentials::CredentialProbeResult {
                identity: reference.identity(),
                reachable: false,
                message: "unknown_credential_provider".to_string(),
            };
        }
        match self.credential_registry(&self.workflow, Arc::new(std::sync::atomic::AtomicBool::new(false))) {
            Ok(registry) => crate::credentials::test_provider_reference(&registry, reference),
            Err(error) => crate::credentials::CredentialProbeResult {
                identity: reference.identity(),
                reachable: false,
                message: error.code().to_string(),
            },
        }
    }

    // r[impl lattice.credentials.animus_secret_providers.tui]
    /// Probe all credential references on the selected node and log fixed safe statuses.
    pub fn probe_selected_node_credentials(&mut self) -> Vec<crate::credentials::CredentialProbeResult> {
        let Some(node_id) = self.selected_node_id() else {
            self.output.push_line("Select one node to probe credentials");
            return Vec::new();
        };
        let Some(refs) = self.workflow.node_credentials(node_id).cloned() else {
            self.output.push_line("Selected node has no configured credentials");
            return Vec::new();
        };
        let mut entries = refs.env.into_iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        let mut results = Vec::with_capacity(entries.len());
        for (binding, reference) in entries {
            let result = self.probe_credential_reference(&reference);
            self.output.push_line(&format!(
                "Credential {binding} ({}): {} (reachable={})",
                result.identity, result.message, result.reachable
            ));
            results.push(result);
        }
        results
    }

    /// Close the diagnostics modal without mutating workflow state.
    pub fn close_diagnostics(&mut self) {
        self.diagnostics.open = false;
    }
}

fn open_store(config: &AppConfig) -> Option<Store> {
    if let Err(error) = std::fs::create_dir_all(&config.data_dir) {
        error!(error = %error, data_dir = ?config.data_dir, "failed to create data directory");
    }
    match Store::new(crate::config::store_path(&config.data_dir)) {
        Ok(s) => Some(s),
        Err(e) => {
            error!(error = %e, "failed to open database, running without persistence");
            None
        }
    }
}
