/// The runtimes a resume executes the remaining nodes with.
struct ResumeRuntime {
    agent: crate::agent_node::RuntimeOptions,
    typed_policy: Option<crate::workflow::decision::Runtime>,
}

impl ResumeRuntime {
    fn executor(
        self,
        subworkflow_shell: Box<dyn crate::engine::SubworkflowRunnerShell>,
    ) -> crate::engine::executor::Executor {
        let executor = crate::engine::executor::Executor::new_with_agent_runtime_config(self.agent)
            .with_subworkflow_runner_shell(subworkflow_shell);
        match self.typed_policy {
            Some(runtime) => executor.with_typed_policy(runtime),
            None => executor,
        }
    }
}

/// Resumes an approved gate with typed policy profiles bound to their declared Condition nodes
/// (Lattice #23). Decisions persisted before the suspension are revalidated against `runtime`
/// before the resume is claimed and carried forward without adapter work; decisions made after
/// the resume persist in the run detail record.
///
/// # Errors
///
/// Returns [`Error::PolicyDecisionStale`] when a persisted decision no longer binds to the
/// current profile, artifact or model; nothing is re-decided and the stored record is unchanged.
// r[impl lattice.typed_policy.record]
// r[impl lattice.typed_policy.replay]
pub async fn resume_with_typed_policy(
    store: &crate::store::Store,
    command: &super::ResumeCommand,
    agent_runtime: crate::agent_node::RuntimeOptions,
    authority_provider: &dyn crate::engine::effect::execution::RunContextProvider,
    runtime: crate::workflow::decision::Runtime,
) -> Result<ResumeExecution, Error> {
    let runtime = ResumeRuntime {
        agent: agent_runtime,
        typed_policy: Some(runtime),
    };
    resume(store, command, runtime, authority_provider).await
}

/// Loads the typed policy decisions persisted before the suspension and revalidates each against
/// the current runtime's profile without adapter work. A decision whose node no longer binds an
/// enabled profile, or whose profile, artifact, model or receipt changed, refuses the resume
/// before it is claimed; nothing is re-decided and the stored record is unchanged.
// r[impl lattice.typed_policy.replay]
fn carried_policy_decisions(
    store: &crate::store::Store,
    current: &super::ApprovalRequest,
    workflow: &crate::engine::workflow::Workflow,
    typed_policy: Option<&crate::workflow::decision::Runtime>,
) -> Result<Vec<crate::workflow::decision::ledger::NodeRecord>, Error> {
    let Some(details) = store.load_run_details(current.run_id)? else {
        return Ok(Vec::new());
    };
    let carried = details
        .nodes
        .into_iter()
        .filter_map(|node| node.policy_decision.map(|record| (node.node_id, record)))
        .map(|(node_id, record)| {
            let admitted =
                workflow.graph.node(node_id).and_then(|node| typed_policy?.profile_for(workflow, node)).ok_or(
                    Error::PolicyDecisionStale(crate::workflow::decision::ledger::ReplayRefusal::ProfileChanged),
                )?;
            crate::workflow::decision::ledger::revalidate(&record, admitted).map_err(Error::PolicyDecisionStale)?;
            Ok(crate::workflow::decision::ledger::NodeRecord { node: node_id, record })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    debug_assert!(carried.len() <= workflow.graph.node_count());
    debug_assert!(carried.iter().all(|decision| decision.record.workflow_outcome.is_none()));
    Ok(carried)
}
