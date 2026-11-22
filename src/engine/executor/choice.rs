//! Imperative shell of the optional typed policy operation on declared Condition nodes.
//!
//! The pure core in `crate::workflow::decision` admits, projects, validates, resolves and records;
//! this shell enforces the wall-clock deadline and cancellation around the one adapter call and
//! routes the resolved label through the Condition node's existing branch ports.

const CANCELLATION_POLL_MILLIS: u64 = 5;
const TERMINAL_PREFIX: &str = "typed policy terminal outcome";

/// One node about to execute.
pub(super) struct Step<'a> {
    pub(super) workflow: &'a crate::engine::workflow::Workflow,
    pub(super) node: &'a crate::engine::workflow::Node,
    pub(super) config: &'a crate::engine::workflow::NodeConfig,
    pub(super) inputs: &'a std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
    pub(super) input_hash: crate::engine::value::ContentHash,
}

impl super::Executor {
    /// The enabled profile selecting `node` of `workflow`, if any.
    pub(super) fn bound_profile(
        &self,
        workflow: &crate::engine::workflow::Workflow,
        node: &crate::engine::workflow::Node,
    ) -> Option<&crate::workflow::decision::admission::Admitted> {
        self.typed_policy.as_ref()?.profile_for(workflow, node)
    }

    /// Folds the bound profile identity into a node cache key; unbound nodes keep their key.
    // r[impl lattice.typed_policy.replay]
    pub(crate) fn policy_cache_key(
        &self,
        workflow: &crate::engine::workflow::Workflow,
        node_id: crate::engine::workflow::NodeId,
        key: crate::engine::value::ContentHash,
    ) -> crate::engine::value::ContentHash {
        let bound = workflow.graph.node(node_id).and_then(|node| self.bound_profile(workflow, node));
        bound.map_or(key, |admitted| {
            crate::engine::value::ContentHash::combine(&[
                key,
                crate::engine::value::ContentHash::from_bytes(admitted.identity.as_bytes()),
            ])
        })
    }

    /// Decides a bound node through its profile; `None` leaves unbound nodes to ordinary execution.
    // r[impl lattice.typed_policy.route]
    pub(super) async fn decide_bound_node(&self, step: Step<'_>) -> Option<super::StepResult> {
        let admitted = self.bound_profile(step.workflow, step.node)?;
        let node_id = step.node.id;
        let condition = match crate::workflow::decision::binding::check_node(&admitted.profile, step.config) {
            Ok(condition) => condition,
            Err(refusal) => return Some(failed(node_id, step.input_hash, refusal.to_string())),
        };
        let value = super::first_input_value(step.inputs);
        let predicate_labels = crate::control_flow::evaluate_condition(condition, value);
        let route = self.decide(admitted, node_id, value, &predicate_labels).await;
        debug_assert!(admitted.profile.is_enabled);
        Some(match route {
            crate::workflow::decision::validation::Route::Branch { label, .. } => super::completed_step_result(
                node_id,
                step.input_hash,
                step.node,
                super::condition_outputs_by_real_port(step.node, std::slice::from_ref(&label), value),
                false,
            ),
            crate::workflow::decision::validation::Route::Terminal { reason } => {
                failed(node_id, step.input_hash, format!("{TERMINAL_PREFIX}: {reason}"))
            }
        })
    }

    async fn decide(
        &self,
        admitted: &crate::workflow::decision::admission::Admitted,
        node_id: crate::engine::workflow::NodeId,
        value: Option<&crate::engine::value::Value>,
        predicate_labels: &[String],
    ) -> crate::workflow::decision::validation::Route {
        let profile = &admitted.profile;
        let (invocation, checked) = match crate::workflow::decision::binding::project(&profile.input, value) {
            Ok(features) => {
                let invocation = crate::workflow::decision::binding::invocation(admitted, node_id, features);
                let checked = self.invoke_bounded(profile, &invocation).await;
                (Some(invocation), checked)
            }
            Err(fault) => (None, crate::workflow::decision::validation::unprojected(fault)),
        };
        let route =
            crate::workflow::decision::validation::resolve(&profile.fallback, &checked.outcome, predicate_labels);
        let record = crate::workflow::decision::ledger::build(crate::workflow::decision::ledger::Facts {
            admitted,
            invocation: invocation.as_ref(),
            checked: &checked,
            route: &route,
        });
        debug_assert_eq!(record.profile, admitted.identity);
        debug_assert!(invocation.as_ref().is_none_or(|invocation| invocation.node == node_id));
        self.policy_decisions
            .lock()
            .push(crate::workflow::decision::ledger::NodeRecord { node: node_id, record });
        route
    }

    // r[impl lattice.typed_policy.invocation]
    async fn invoke_bounded(
        &self,
        profile: &crate::workflow::decision::admission::Profile,
        invocation: &crate::workflow::decision::binding::Invocation,
    ) -> crate::workflow::decision::validation::Checked {
        let Some(runtime) = self.typed_policy.as_ref() else {
            return crate::workflow::decision::validation::validate(
                profile,
                invocation,
                &crate::workflow::decision::Response::Unavailable,
            );
        };
        if self.cancellation_requested() {
            return crate::workflow::decision::validation::cancelled();
        }
        let adapter = std::sync::Arc::clone(&runtime.adapter);
        let owned = invocation.clone();
        let task = tokio::task::spawn_blocking(move || adapter.invoke(&owned));
        let deadline = std::time::Duration::from_millis(profile.limits.deadline_millis);
        tokio::select! {
            joined = task => {
                let response = joined.unwrap_or(crate::workflow::decision::Response::Unavailable);
                crate::workflow::decision::validation::validate(profile, invocation, &response)
            }
            () = tokio::time::sleep(deadline) => crate::workflow::decision::validation::timed_out(),
            () = cancellation(std::sync::Arc::clone(&self.cancellation), profile.limits.deadline_millis) => {
                crate::workflow::decision::validation::cancelled()
            }
        }
    }
}

async fn cancellation(flag: std::sync::Arc<std::sync::atomic::AtomicBool>, deadline_millis: u64) {
    let poll_count = deadline_millis.div_ceil(CANCELLATION_POLL_MILLIS);
    for _ in 0..poll_count {
        if flag.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(CANCELLATION_POLL_MILLIS)).await;
    }
    std::future::pending::<()>().await;
}

fn failed(
    node_id: crate::engine::workflow::NodeId,
    input_hash: crate::engine::value::ContentHash,
    error: String,
) -> super::StepResult {
    super::StepResult {
        node_id,
        status: super::NodeStatus::Failed { error },
        input_hash,
    }
}

impl super::Executor {
    /// Continue a product run from terminal checkpoint steps, stopping at approval gates.
    ///
    /// Checkpoint nodes restore through `Session::new_with_checkpoint` and are never re-run.
    /// An empty checkpoint is exactly [`Self::run_authorized_until_approval`].
    pub async fn run_authorized_from_checkpoint(
        &mut self,
        workflow: &crate::engine::workflow::Workflow,
        cache: &std::collections::HashMap<
            crate::engine::value::ContentHash,
            std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
        >,
        context: &crate::engine::effect::execution::RunContext,
        checkpoint: &[super::StepResult],
    ) -> super::ApprovalRunOutcome {
        let control = super::ExecutionControl {
            checkpoint,
            stop_at_approval: true,
            resumed_gate: None,
            resumed_gate_outputs: None,
        };
        let bound_inputs = std::collections::HashMap::new();
        self.run_with_bound_inputs_control(workflow, cache, &bound_inputs, Some(context), control).await
    }
}
