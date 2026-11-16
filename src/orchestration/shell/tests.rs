mod charging;
mod provider;
mod recovery;

fn known_status(succeeded: bool) -> crate::orchestration::usage::RunOutcomeObservation {
    crate::orchestration::usage::RunOutcomeObservation {
        succeeded,
        cancelled: Some(false),
    }
}

fn pending_usage_fixture(
    finished_epoch_ms: Option<i64>,
) -> (tempfile::TempDir, crate::store::Store, crate::orchestration::Instance) {
    let workflow = crate::engine::Workflow::new("usage-run", crate::engine::workflow::WorkflowTimestamp::epoch());
    pending_usage_fixture_for_workflow(finished_epoch_ms, workflow)
}

fn pending_usage_fixture_for_workflow(
    finished_epoch_ms: Option<i64>,
    workflow: crate::engine::Workflow,
) -> (tempfile::TempDir, crate::store::Store, crate::orchestration::Instance) {
    use crate::orchestration::*;
    let directory = tempfile::tempdir().expect("temporary usage store");
    let store = crate::store::Store::new(directory.path().join("usage.redb")).expect("store opens");
    store.save_workflow(&workflow).expect("workflow saves");
    let charge = BudgetCharge {
        rounds: 1,
        tokens: 100,
        monetary_microunits: 50,
        duration_millis: 500,
        ..BudgetCharge::default()
    };
    let definition = Definition {
        schema_version: current_schema_version(),
        id: DefinitionId::try_from_text("usage-test").expect("definition identity"),
        objective: "retain unknown usage".to_string(),
        workflow: workflow_revision(&workflow).expect("workflow revision"),
        completion: CompletionPolicy::RunSucceeded,
        retry: None,
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                tokens: true,
                monetary_microunits: true,
                duration_millis: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: 2,
                tokens: 200,
                monetary_microunits: 100,
                duration_millis: 1000,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: charge,
        },
        authority: AuthorityPolicy {
            approvers: std::collections::BTreeSet::from(["operator".to_string()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy::default(),
    };
    let base = new_instance(definition, 1, None).expect("instance");
    let run_id = crate::engine::RunId(7);
    let dispatch_id = dispatch_identity(&base, run_id).expect("dispatch identity");
    let action = Action::StartRound {
        intent: DispatchIntent {
            id: dispatch_id.clone(),
            run_id,
            workflow_revision: base.workflow_revision.clone(),
            status: DispatchStatus::Pending,
        },
        planned_charge: BudgetChargePlan {
            rounds: Some(1),
            agent_invocations: Some(0),
            tokens: Some(100),
            monetary_microunits: Some(50),
            duration_millis: Some(500),
        },
    };
    let started = transition(&base, Fact {
        id: fact_identity(&base.id, base.version, &action).expect("fact"),
        action,
    })
    .expect("round admitted");
    store
        .create_orchestration(&started.instance, &[started.event.expect("event")])
        .expect("pending instance persists");
    store
        .save_run(&crate::store::RunRecord {
            run_id,
            owner: crate::store::RunOwner::Orchestration {
                instance_id: started.instance.id.clone(),
                dispatch_id,
            },
            workflow_name: workflow.meta.name,
            started_epoch_ms: 10,
            finished_epoch_ms,
            node_count: 0,
            cache_hits: 0,
            failures: 0,
        })
        .expect("owned run persists");
    (directory, store, started.instance)
}
