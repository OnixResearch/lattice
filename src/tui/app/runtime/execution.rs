//! Background workflow submission and cancellation.

use super::*;

impl State {
    /// Start the current workflow in the background.
    pub fn start_workflow_run(&mut self) -> bool {
        if self.run_state.active {
            self.output.push_line("Workflow already running");
            return false;
        }

        let diagnostics = crate::workflow_validation::validate_workflow(&self.workflow);
        self.diagnostics.diagnostics = diagnostics.clone();
        self.diagnostics.open = false;
        if crate::workflow_validation::has_fatal_diagnostics(&diagnostics) {
            self.diagnostics = DiagnosticsState::open(diagnostics.clone());
            self.output.push_line("Workflow validation failed; open diagnostics for details");
            for diagnostic in crate::workflow_validation::fatal_diagnostics(&diagnostics) {
                self.output.push_line(&crate::workflow_validation::format_diagnostic(&diagnostic));
            }
            return false;
        }
        for diagnostic in &diagnostics {
            self.output.push_line(&crate::workflow_validation::format_diagnostic(diagnostic));
        }

        let workflow = self.workflow.clone();
        self.start_workflow_snapshot(workflow, "▶ Running workflow…")
    }

    pub(in crate::tui::app) fn start_workflow_snapshot(&mut self, workflow: Workflow, message: &str) -> bool {
        if self.run_state.active {
            self.output.push_line("Workflow already running");
            return false;
        }

        let Some(next_run_id) = self.next_tui_run_id.checked_add(1) else {
            self.output.push_line("TUI run identity range exhausted");
            return false;
        };
        let (progress_sender, progress_receiver) = match crate::engine::executor::progress::channel(&workflow) {
            Ok(channel) => channel,
            Err(error) => {
                self.output.push_line(&error.to_string());
                return false;
            }
        };
        let run_id = RunId(self.next_tui_run_id);
        self.next_tui_run_id = next_run_id;
        let (sender, receiver) = mpsc::channel(RUN_EVENT_CAPACITY);
        let node_statuses =
            workflow.graph.nodes().map(|node| (node.id.0, NodeRunStatus::Pending)).collect::<HashMap<_, _>>();

        self.run_state = RunState {
            active: true,
            node_statuses,
            node_started: HashMap::new(),
            node_start_times: HashMap::new(),
            node_durations_millis: HashMap::new(),
            node_tokens: HashMap::new(),
            node_cost_microunits: HashMap::new(),
            node_attempts: HashMap::new(),
            active_replay_timeline: None,
        };
        self.run_output_lines.clear();
        self.output.push_line(message);

        let active_workflow = workflow.clone();
        let executor_factory = self.executor_factory.clone();
        let authority_provider = Arc::clone(&self.authority_provider);
        let authority_store = self.store.clone();
        let cancellation = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let executor_cancellation = Arc::clone(&cancellation);
        let handle = tokio::spawn(
            BackgroundRun {
                run_id,
                workflow,
                sender,
                progress_sender,
                progress_receiver,
                executor_factory,
                authority_provider,
                authority_store,
                executor_cancellation,
            }
            .execute(),
        );
        self.active_run = Some(crate::tui::app::model::live::ActiveRun {
            run_id,
            workflow: active_workflow,
            receiver,
            handle,
            cancellation,
        });
        true
    }

    /// Cancel the active background run, if any.
    pub fn cancel_active_run(&mut self) -> bool {
        let Some(active_run) = self.active_run.take() else {
            self.output.push_line("No active workflow run");
            return false;
        };

        active_run.cancellation.store(true, std::sync::atomic::Ordering::SeqCst);
        active_run.handle.abort();
        for (node_id, started) in self.run_state.node_started.drain() {
            match u64::try_from(started.elapsed().as_millis()) {
                Ok(duration_millis) => {
                    self.run_state.node_durations_millis.insert(node_id, duration_millis);
                }
                Err(error) => {
                    error!(node_id, %error, "canceled node duration cannot be represented in milliseconds");
                    self.output.push_line(&format!("Node {node_id} duration exceeds the supported millisecond range"));
                }
            }
        }
        for status in self.run_state.node_statuses.values_mut() {
            if matches!(*status, NodeRunStatus::Pending | NodeRunStatus::Running) {
                *status = NodeRunStatus::Skipped;
            }
        }
        self.run_state.active = false;
        let workflow_name = active_run.workflow.meta.name.clone();
        let replay_timestamp_epoch_ms =
            crate::operation::shell::Clock::observe_epoch_millis(&crate::operation::shell::SystemClock);
        let replay_events =
            crate::replay::cancelled_replay_events(active_run.run_id, &active_run.workflow, replay_timestamp_epoch_ms);
        self.run_state.active_replay_timeline =
            Some(crate::replay::build_replay_timeline(active_run.run_id, &replay_events, None));
        self.save_run_record(active_run.run_id, &workflow_name, model::receipts::RunCounts {
            cache_hits: 0,
            failures: 0,
            nodes: self.run_state.node_statuses.len(),
        });
        self.save_run_detail_record(crate::tui::app::model::receipts::canceled_run_detail_record(
            active_run.run_id,
            &active_run.workflow,
            &self.run_state.node_statuses,
        ));
        self.save_run_replay_events(active_run.run_id, replay_events);
        self.output.push_line("Workflow run canceled");
        true
    }
}

struct BackgroundRun {
    run_id: RunId,
    workflow: Workflow,
    sender: mpsc::Sender<crate::tui::app::model::live::RunEvent>,
    progress_sender: crate::engine::executor::progress::Sender,
    progress_receiver: mpsc::Receiver<crate::engine::executor::NodeProgress>,
    executor_factory: Arc<dyn Fn() -> Executor + Send + Sync>,
    authority_provider: Arc<dyn crate::engine::effect::execution::RunContextProvider>,
    authority_store: Option<Store>,
    executor_cancellation: Arc<std::sync::atomic::AtomicBool>,
}

impl BackgroundRun {
    async fn execute(self) {
        let Self {
            run_id,
            workflow,
            sender,
            progress_sender,
            mut progress_receiver,
            executor_factory,
            authority_provider,
            authority_store,
            executor_cancellation,
        } = self;
        if sender
            .send(crate::tui::app::model::live::RunEvent::ReplayStarted {
                run_id,
                events: crate::replay::active_events(run_id, &workflow, ACTIVE_REPLAY_TIMESTAMP_EPOCH_MS),
            })
            .await
            .is_err()
        {
            return;
        }
        let progress_observer = progress_sender.observer();
        let progress_events = sender.clone();
        let progress_task = tokio::spawn(async move {
            while let Some(progress) = progress_receiver.recv().await {
                if progress_events.send(crate::tui::app::model::live::RunEvent::NodeProgress(progress)).await.is_err() {
                    break;
                }
            }
        });
        let mut executor =
            executor_factory().with_progress_sender(progress_sender).with_cancellation(executor_cancellation);
        let cache = HashMap::new();
        let results = match authority_provider.load(run_id, authority_store) {
            Ok(authority) => executor.run_authorized(&workflow, &cache, &authority).await.1,
            Err(error) => workflow
                .graph
                .nodes()
                .map(|node| StepResult {
                    node_id: node.id,
                    status: NodeStatus::Failed {
                        error: format!("Choregraph product execution failed: {error}"),
                    },
                    input_hash: crate::engine::ContentHash::from_bytes(b""),
                })
                .collect(),
        };
        let observed_completion_order = if executor.observed_completion_order().is_empty() {
            results.iter().map(|step| step.node_id).collect()
        } else {
            executor.observed_completion_order().to_vec()
        };
        // Drain progress before the final event; otherwise a late Started
        // observation can overwrite the completed run's metadata.
        drop(executor);
        let has_observer_failed = match progress_task.await {
            Ok(()) => false,
            Err(error) => {
                error!(run_id = run_id.0, %error, "background progress observer failed; live metadata is incomplete");
                true
            }
        };
        let observation_fault = progress_observer.fault();
        if has_observer_failed || observation_fault.is_some() {
            error!(run_id = run_id.0, ?observation_fault, "live executor observations incomplete");
            if sender.send(crate::tui::app::model::live::RunEvent::ObservationIncomplete).await.is_err() {
                info!(run_id = run_id.0, "background run receiver dropped before observation diagnostic");
            }
        }
        if sender
            .send(crate::tui::app::model::live::RunEvent::Completed {
                run_id,
                workflow: Box::new(workflow),
                results,
                observed_completion_order,
            })
            .await
            .is_err()
        {
            info!(run_id = run_id.0, "background run receiver dropped before completion");
        }
    }
}
