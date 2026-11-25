//! Persisting workflow revisions and execution receipts.

use super::*;

impl State {
    pub(in crate::tui::app) fn save_run_record(
        &mut self,
        run_id: RunId,
        workflow_name: &str,
        counts: model::receipts::RunCounts,
    ) {
        let Some(store) = &self.store else {
            return;
        };
        let now = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
        let record = RunRecord {
            run_id,
            owner: crate::store::RunOwner::Unscoped,
            workflow_name: workflow_name.to_string(),
            started_epoch_ms: now,
            finished_epoch_ms: Some(now),
            node_count: counts.nodes,
            cache_hits: counts.cache_hits,
            failures: counts.failures,
        };
        if let Err(e) = store.save_run(&record) {
            self.output.push_line(&format!("Run history save failed: {e}"));
        }
    }

    pub(in crate::tui::app) fn save_run_detail_record(&mut self, record: RunDetailRecord) {
        let Some(store) = &self.store else {
            return;
        };
        if let Err(error) = store.save_run_details(&record) {
            self.output.push_line(&format!("Run details save failed: {error}"));
        }
    }

    // r[impl n8nclone.workflow_execution.replay.event_store]
    // r[impl n8nclone.tui_ux.replay_debugger.active_run]
    pub(in crate::tui::app) fn save_run_replay_events(
        &mut self,
        run_id: RunId,
        events: Vec<crate::replay::ReplayEventRecord>,
    ) {
        let Some(store) = &self.store else {
            return;
        };
        if let Err(error) = store.save_run_replay_events(run_id, &events) {
            self.output.push_line(&format!("Replay save failed: {error}"));
        }
    }

    /// Save the current workflow to the store.
    // r[impl lattice.workflow_templates.activation]
    pub fn save_workflow(&mut self) {
        let Some(store) = self.store.clone() else {
            self.output.push_line("No database available");
            return;
        };
        match self.save_current_workflow(&store) {
            Ok(()) => {
                self.clear_workflow_dirty();
                self.workflow_template_draft = None;
                self.output.push_line(&format!("Saved '{}'", self.workflow.meta.name));
            }
            Err(error) => self.output.push_line(&format!("Save failed: {error}")),
        }
    }

    pub(in crate::tui::app) fn save_current_workflow(&mut self, store: &Store) -> Result<(), String> {
        let diagnostics = self.validate_workflow_diagnostics();
        if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
            return Err("workflow validation failed; inspect diagnostics before saving".into());
        }
        let Some(draft) = &self.workflow_template_draft else {
            self.workflow = store
                .save_workflow_cas(&self.workflow, self.workflow.catalog_revision.as_ref())
                .map_err(|error| error.to_string())?;
            return Ok(());
        };
        let current_revision = crate::orchestration::workflow_revision(&self.workflow)
            .map_err(|_| "workflow-template draft identity is invalid".to_string())?;
        let bundle = &draft.bundle;
        if current_revision != bundle.workflow_revision {
            return Err("workflow-template draft differs from the previewed revision".to_string());
        }
        let is_source_available = bundle
            .params
            .as_ref()
            .is_some_and(|bound| self.workflow_templates.iter().any(|template| template.identity == bound.template));
        if !is_source_available {
            return Err("workflow-template draft source is unavailable".to_string());
        }
        crate::workflow_template::save_draft(store, bundle).map_err(|error| error.to_string())?;
        self.workflow = store
            .load_workflow(&self.workflow.meta.name)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "saved workflow is unavailable".to_string())?;
        Ok(())
    }
}
