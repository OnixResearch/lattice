//! Applying live node observations and completed results.

use super::*;

impl State {
    /// Poll background run events and apply completed results.
    pub fn poll_run_events(&mut self) {
        for _ in 0..RUN_EVENT_CAPACITY {
            let Some(event) = self.active_run.as_mut().and_then(|run| run.receiver.try_recv().ok()) else {
                break;
            };
            match event {
                crate::tui::app::model::live::RunEvent::ReplayStarted { run_id, events } => {
                    if self.run_state.active {
                        let timeline = crate::replay::build_replay_timeline(run_id, &events, None);
                        debug_assert!(timeline.diagnostics.iter().all(|diagnostic| !diagnostic.fatal));
                        self.run_state.active_replay_timeline = Some(timeline);
                    }
                }
                crate::tui::app::model::live::RunEvent::NodeProgress(progress) => match progress {
                    crate::engine::executor::NodeProgress::Started { node_id } => {
                        self.observe_node_started(node_id);
                    }
                    crate::engine::executor::NodeProgress::Finished {
                        node_id,
                        duration_millis,
                        result,
                    } => {
                        self.run_state.node_started.remove(&node_id.0);
                        if let Some(duration_millis) = duration_millis {
                            self.run_state.node_durations_millis.insert(node_id.0, duration_millis);
                        }
                        debug_assert_eq!(node_id, result.node_id);
                        if let Some(active_run) = self.active_run.take() {
                            self.observe_node_result(active_run.run_id, &active_run.workflow, &result);
                            self.active_run = Some(active_run);
                        }
                    }
                },
                crate::tui::app::model::live::RunEvent::ObservationIncomplete => {
                    self.run_state.node_started.clear();
                    self.output.push_line(
                        "Live observations incomplete; missing start/timing metadata remains unknown; final execution results retained",
                    );
                }
                crate::tui::app::model::live::RunEvent::Completed {
                    run_id,
                    workflow,
                    results,
                    observed_completion_order,
                } => {
                    self.finish_workflow_run(run_id, &workflow, results, observed_completion_order);
                    self.active_run = None;
                }
            }
        }
    }

    fn observe_node_started(&mut self, node_id: crate::engine::workflow::NodeId) {
        self.run_state
            .node_started
            .insert(node_id.0, crate::operation::shell::SystemClock.observe_monotonic_instant());
        self.observe_node_start_time(node_id);
        self.run_state.node_durations_millis.remove(&node_id.0);
        let attempts = self.run_state.node_attempts.entry(node_id.0).or_insert(0);
        *attempts = attempts.saturating_add(1);
        if matches!(self.run_state.node_statuses.get(&node_id.0), None | Some(NodeRunStatus::Pending)) {
            self.run_state.node_statuses.insert(node_id.0, NodeRunStatus::Running);
        }
    }

    fn observe_node_start_time(&mut self, node_id: crate::engine::workflow::NodeId) {
        let epoch_ms = crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
        let observed = chrono::DateTime::from_timestamp_millis(epoch_ms)
            .map(|timestamp| timestamp.with_timezone(&chrono::Local).format("%H:%M:%S").to_string());
        if let Some(observed) = observed {
            self.run_state.node_start_times.insert(node_id.0, observed);
        } else {
            self.run_state.node_start_times.remove(&node_id.0);
            self.output.push_line("Node start timestamp is outside the supported calendar range");
        }
    }

    // r[impl lattice.tui_ux.rows.metadata]
    pub(in crate::tui::app) fn observe_node_result(
        &mut self,
        run_id: RunId,
        run_workflow: &Workflow,
        step: &StepResult,
    ) {
        let node_label = run_workflow.graph.node(step.node_id).map(|node| node.label.as_str()).unwrap_or("?");
        match &step.status {
            NodeStatus::Completed { outputs, cached } => {
                self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Completed);
                if let Some(tokens) =
                    crate::tui::app::model::values::node_usage_tokens(run_workflow, step.node_id, outputs)
                {
                    self.run_state.node_tokens.insert(step.node_id.0, tokens);
                }
                if let Some(cost) = crate::tui::app::model::values::node_usage_cost(run_workflow, step.node_id, outputs)
                {
                    self.run_state.node_cost_microunits.insert(step.node_id.0, cost);
                }
                let cache_tag = if *cached { " (cached)" } else { "" };
                self.output.push_line(&format!("✓ {node_label}{cache_tag} [{}]", step.input_hash));
                for record in crate::tui::app::model::values::completed_output_records(
                    run_id,
                    run_workflow,
                    step.node_id,
                    outputs,
                ) {
                    self.output.push_line(&record.rendered());
                    self.run_output_lines.push(record);
                }
            }
            NodeStatus::Failed { error } => {
                self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Failed);
                self.output.push_line(&format!("✗ {node_label}: {error}"));
            }
            NodeStatus::Skipped => {
                self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Skipped);
            }
            NodeStatus::Pending => {
                self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Pending);
            }
            NodeStatus::Running => {
                self.run_state.node_statuses.insert(step.node_id.0, NodeRunStatus::Running);
            }
        }
    }

    pub(in crate::tui::app) fn finish_workflow_run(
        &mut self,
        run_id: RunId,
        run_workflow: &Workflow,
        results: Vec<StepResult>,
        observed_completion_order: Vec<crate::engine::workflow::NodeId>,
    ) {
        for step in &results {
            if !matches!(
                self.run_state.node_statuses.get(&step.node_id.0),
                Some(NodeRunStatus::Completed | NodeRunStatus::Failed | NodeRunStatus::Skipped)
            ) {
                self.observe_node_result(run_id, run_workflow, step);
            }
            if let NodeStatus::Completed { outputs, .. } = &step.status
                && let Some(ref store) = self.store
                && store.put_cache(&step.input_hash, outputs).is_err()
            {
                self.output.push_line("Cache update failed");
            }
        }

        self.run_state.active = false;
        let failures = results.iter().filter(|step| matches!(step.status, NodeStatus::Failed { .. })).count();
        let cache_hits = results
            .iter()
            .filter(|step| matches!(step.status, NodeStatus::Completed { cached: true, .. }))
            .count();
        let replay_finished_epoch_ms =
            crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
        let replay_events = crate::replay::replay_events_from_results(
            run_id,
            run_workflow,
            &results,
            replay_finished_epoch_ms,
            Some(replay_finished_epoch_ms),
        );
        self.run_state.active_replay_timeline =
            Some(crate::replay::build_replay_timeline(run_id, &replay_events, None));
        self.save_run_record(run_id, &run_workflow.meta.name, model::receipts::RunCounts {
            cache_hits,
            failures,
            nodes: results.len(),
        });
        self.save_run_detail_record(crate::run::history::detail_record_from_results_with_completion_order(
            run_id,
            run_workflow,
            &results,
            observed_completion_order,
        ));
        self.save_run_replay_events(run_id, replay_events);
        self.output.push_line(&format!("── run {} complete ({} steps) ──", run_id, results.len()));
    }
}
