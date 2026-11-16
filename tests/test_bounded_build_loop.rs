//! Real local runtime repetition; deterministic producer is not model qualification.
#[path = "../examples/bounded-build-loop/mod.rs"]
mod bounded_build_loop;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use bounded_build_loop::FrozenInputs;
use bounded_build_loop::Report;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::NodeRunnerSettings;
use lattice::orchestration::Status;
use lattice::orchestration::{self};
use lattice::store::Store;
use lattice::workflow_format::load_workflow_nickel_file;
use lattice::workflow_format::workflow_to_nickel;

fn prepared(mode: &str) -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("loop");
    bounded_build_loop::prepare(&root, mode).unwrap();
    (directory, root)
}

fn not_cancelled() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

async fn started(root: &Path) {
    tokio::time::timeout(Duration::from_secs(20), async {
        while !root.join("active/evaluator-started").exists() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("real evaluator must start");
}

fn retained(root: &Path, report: &Report) {
    for receipt in &report.rounds {
        let round = root.join(format!("round-{}", receipt.round));
        for (name, identity) in &receipt.handoffs {
            assert_eq!(&bounded_build_loop::digest(&fs::read(round.join(name)).unwrap()), identity);
        }
        assert_eq!(receipt.handoffs.len(), 3);
        assert!(round.join("build.log").exists());
    }
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.pass]
#[tokio::test]
async fn evaluator_pass_stops_without_a_second_round() {
    let (_directory, root) = prepared("pass");
    let report = bounded_build_loop::run(&root, not_cancelled()).await.unwrap();
    assert_eq!(report.outcome, "pass");
    assert_eq!(report.rounds.len(), 1);
    assert_eq!(report.total_charge.rounds, 1);
    retained(&root, &report);
    assert!(fs::read_to_string(root.join("round-1/build.log")).unwrap().contains("test result: ok"));
    assert!(!root.join("round-2").exists());
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.retry]
#[tokio::test]
async fn failed_build_artifact_and_verdict_survive_runtime_retry() {
    let (_directory, root) = prepared("repair");
    let report = bounded_build_loop::run(&root, not_cancelled()).await.unwrap();
    assert_eq!(report.outcome, "pass");
    assert_eq!(report.rounds.len(), 2);
    assert_eq!(report.total_charge.rounds, 2);
    assert_eq!(report.total_charge.agent_invocations, 4);
    assert_eq!(report.total_charge.duration_millis, bounded_build_loop::TOTAL_MILLIS);
    retained(&root, &report);
    assert_eq!(report.rounds[0].outcome, "fail");
    assert_eq!(report.rounds[1].outcome, "pass");
    assert_eq!(report.rounds[0].revision, report.rounds[1].revision);
    assert_eq!(report.rounds[1].previous_verdict.as_ref(), report.rounds[0].handoffs.get("verdict.txt"));
    assert_ne!(report.rounds[0].handoffs["build.rs"], report.rounds[1].handoffs["build.rs"]);
    let store = Store::new(root.join("state.redb")).unwrap();
    let instance = store.list_orchestrations().unwrap().pop().unwrap();
    assert_eq!(instance.status, Status::Completed);
    assert_eq!(instance.retry.admitted, 1);
    assert_eq!(instance.budgets.consumed, report.total_charge);
    let replay = orchestration::replay(&store, instance.id.as_str()).unwrap();
    assert!(replay.valid);
    assert_eq!(replay.instance, instance);
    drop(store);
    assert!(bounded_build_loop::run(&root, not_cancelled()).await.is_err(), "no blind restart");
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.exhausted]
#[tokio::test]
async fn repeated_test_failures_exhaust_one_total_ledger() {
    let (_directory, root) = prepared("fail");
    let report = bounded_build_loop::run(&root, not_cancelled()).await.unwrap();
    assert_eq!(report.outcome, "exhausted");
    assert_eq!(report.rounds.len(), 2);
    assert!(report.rounds.iter().all(|round| round.outcome == "fail"));
    assert_eq!(report.total_charge.agent_invocations, 4);
    retained(&root, &report);
    let store = Store::new(root.join("state.redb")).unwrap();
    assert_eq!(store.list_orchestrations().unwrap()[0].status, Status::Exhausted);
    assert!(!root.join("round-3").exists());
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.resource]
#[tokio::test]
async fn a_resource_cap_blocks_retry_without_resetting_the_budget() {
    let (_directory, root) = prepared("repair");
    let mut definition = orchestration::load_definition_nickel_file(&root.join("definition.ncl")).unwrap();
    definition.budgets.limits.agent_invocations = 2;
    fs::write(root.join("definition.ncl"), orchestration::definition_to_nickel(&definition).unwrap()).unwrap();
    let mut frozen: FrozenInputs = serde_json::from_slice(&fs::read(root.join("frozen.json")).unwrap()).unwrap();
    frozen.definition_revision = orchestration::definition_revision(&definition).unwrap();
    fs::write(root.join("frozen.json"), serde_json::to_vec(&frozen).unwrap()).unwrap();
    let report = bounded_build_loop::run(&root, not_cancelled()).await.unwrap();
    assert_eq!(report.outcome, "exhausted");
    assert_eq!(report.rounds.len(), 1);
    assert_eq!(report.total_charge.agent_invocations, 2);
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.authority]
#[tokio::test]
async fn removing_a_role_grant_denies_before_its_adapter_dispatch() {
    for denied_role in [0, 1, 2] {
        let (_directory, root) = prepared("pass");
        let mut profile: lattice::engine::effect::standalone::core::Profile =
            serde_json::from_slice(&fs::read(root.join("authority.json")).unwrap()).unwrap();
        profile.grants.retain(|grant| grant.node_id != denied_role);
        fs::write(root.join("authority.json"), serde_json::to_vec(&profile).unwrap()).unwrap();
        let report = bounded_build_loop::run(&root, not_cancelled()).await.unwrap();
        assert_eq!(report.outcome, "fail");
        assert_eq!(report.rounds.len(), 1);
        assert_eq!(root.join("round-1/spec.txt").exists(), denied_role != 0);
        assert_eq!(root.join("round-1/build.rs").exists(), denied_role == 2);
        assert!(!root.join("round-1/evaluator-started").exists());
    }
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.cancel]
#[tokio::test]
async fn active_cancellation_is_forwarded_and_prevents_later_rounds() {
    let (_directory, root) = prepared("cancel");
    let cancellation = not_cancelled();
    let signal = Arc::clone(&cancellation);
    let (report, ()) = tokio::join!(bounded_build_loop::run(&root, cancellation), async {
        started(&root).await;
        signal.store(true, Ordering::Release);
    });
    let report = report.unwrap();
    assert_eq!(report.outcome, "cancelled");
    assert_eq!(report.rounds.len(), 1);
    assert!(report.rounds[0].cancellation_requested);
    assert!(report.rounds[0].owned_termination_observed);
    let verdict = fs::read_to_string(root.join("round-1/verdict.txt")).unwrap();
    assert!(verdict.contains("owned process termination observed"), "{verdict}");
    assert!(!root.join("round-2").exists());
    let store = Store::new(root.join("state.redb")).unwrap();
    let instance = store.list_orchestrations().unwrap().pop().unwrap();
    assert_eq!(instance.status, Status::Cancelled);
    assert_eq!(instance.budgets.consumed, bounded_build_loop::charge());
    assert_eq!(instance.budgets.consumed, report.total_charge);
    assert!(instance.budgets.reservations.is_empty());
    let replay = orchestration::replay(&store, instance.id.as_str()).unwrap();
    assert!(replay.valid);
    assert_eq!(replay.instance, instance);
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.admission]
#[tokio::test]
async fn invalid_next_graph_dispatches_no_work_and_keeps_original_results() {
    for invalid_runtime in [true, false] {
        let (_directory, root) = prepared("repair");
        let original = load_workflow_nickel_file(&root.join("workflow.ncl")).unwrap();
        let original_revision = orchestration::workflow_revision(&original).unwrap();
        let (result, ()) = tokio::join!(bounded_build_loop::run(&root, not_cancelled()), async {
            started(&root).await;
            let mut candidate = original.clone();
            if invalid_runtime {
                candidate.set_node_runner_settings(NodeId(2), NodeRunnerSettings::new("undeclared", "runtime"));
            } else {
                // Regeneration/parameter edits cannot alter the running snapshot or replay as its revision.
                candidate.meta.name = "unapproved-next-round".into();
                let Some(lattice::engine::NodeConfig::Shell { command, .. }) = candidate.configs.get_mut(&2) else {
                    panic!("fixed evaluator configuration");
                };
                *command = format!("touch '{}'", root.join("rejected-dispatch").display());
            }
            fs::write(root.join("workflow.ncl"), workflow_to_nickel(&candidate)).unwrap();
        });
        assert!(result.is_err());
        assert!(root.join("admission-rejection.txt").exists());
        assert!(!root.join("round-2").exists());
        assert!(!root.join("active").exists());
        assert!(!root.join("rejected-dispatch").exists());
        let receipt: bounded_build_loop::RoundReceipt =
            serde_json::from_slice(&fs::read(root.join("round-1/receipt.json")).unwrap()).unwrap();
        assert_eq!(receipt.revision, original_revision);
        assert_eq!(receipt.outcome, "fail");
        assert!(root.join("round-1/verdict.txt").exists());
        let store = Store::new(root.join("state.redb")).unwrap();
        assert_eq!(store.list_orchestrations().unwrap()[0].round, 1);
        assert_eq!(store.list_orchestrations().unwrap().len(), 1);
        assert_eq!(store.list_orchestrations().unwrap()[0].budgets.consumed.agent_invocations, 2);
        assert_eq!(
            orchestration::workflow_revision(&store.load_workflow("bounded-build-loop").unwrap().unwrap()).unwrap(),
            original_revision
        );
    }
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.authority]
#[tokio::test]
async fn saved_child_reference_does_not_inherit_parent_effect_authority() {
    use lattice::control_flow::SubworkflowNodeConfig;
    use lattice::control_flow::SubworkflowOutputMapping;
    use lattice::engine::RunId;
    use lattice::engine::Workflow;
    use lattice::engine::workflow::WorkflowTimestamp;
    let directory = tempfile::tempdir().unwrap();
    let store = Store::new(directory.path().join("child.redb")).unwrap();
    let marker = directory.path().join("unintended-child-effect");
    let mut child = Workflow::new("bounded-denied-child", WorkflowTimestamp::epoch());
    child.add_shell_node("Denied child effect", format!("touch '{}'", marker.display()));
    store.save_workflow(&child).unwrap();
    let mut parent = Workflow::new("bounded-parent", WorkflowTimestamp::epoch());
    parent.add_subworkflow_node("Saved child", SubworkflowNodeConfig {
        workflow_name: child.meta.name,
        input_mappings: vec![],
        output_mappings: vec![SubworkflowOutputMapping {
            child_output: "stdout".into(),
            parent_output: "result".into(),
        }],
    });
    store.save_workflow(&parent).unwrap();
    let result = lattice::headless::run_saved_workflow(&store, &parent.meta.name, RunId(77)).await.unwrap();
    assert_eq!(result.status, lattice::headless::HeadlessRunStatus::Failed);
    assert!(!marker.exists());
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.authority]
#[test]
fn generated_output_cannot_approve_or_expand_role_authority() {
    let workflow = bounded_build_loop::workflow(Path::new("examples/bounded-build-loop")).unwrap();
    let definition = bounded_build_loop::definition(&workflow).unwrap();
    for role in ["agent:planner", "agent:generator", "agent:evaluator"] {
        assert!(orchestration::authorize_definition_actor(&definition, role).is_err());
    }
    let instance = orchestration::new_instance(definition, 1, None).unwrap();
    let mut candidate = workflow.clone();
    candidate.add_shell_node("Widen effect", "touch not-authorized");
    assert!(bounded_build_loop::admit(&instance, &candidate).is_err());
}

// r[verify lattice.agent_orchestration.bounded_build_loop.scenario.cancel-winner]
#[tokio::test]
async fn observed_completion_wins_a_late_cancellation_before_durable_settlement() {
    use lattice::engine::effect::standalone::OperatorRunContextSource;
    use lattice::headless::SnapshotRunRequest;
    use lattice::headless::{self};
    use lattice::orchestration::Action;
    use lattice::orchestration::BudgetChargePlan;
    use lattice::orchestration::DispatchIntent;
    use lattice::orchestration::DispatchStatus;
    use lattice::store::RunOwner;

    for (mode, expected) in [("pass", Status::Completed), ("fail", Status::Failed)] {
        let (_directory, root) = prepared(mode);
        let workflow = load_workflow_nickel_file(&root.join("workflow.ncl")).unwrap();
        let definition = orchestration::load_definition_nickel_file(&root.join("definition.ncl")).unwrap();
        let store = Store::new(root.join("state.redb")).unwrap();
        let mut instance =
            orchestration::new_instance(definition, store.next_orchestration_sequence().unwrap(), None).unwrap();
        bounded_build_loop::admit(&instance, &workflow).unwrap();
        store.save_workflow(&workflow).unwrap();
        store.save_orchestration_definition(&instance.definition).unwrap();
        store.create_orchestration(&instance, &[]).unwrap();
        let run_id = store.next_run_id().unwrap();
        let intent = DispatchIntent {
            id: orchestration::dispatch_identity(&instance, run_id).unwrap(),
            run_id,
            workflow_revision: instance.workflow_revision.clone(),
            status: DispatchStatus::Pending,
        };
        let charge = bounded_build_loop::charge();
        bounded_build_loop::commit(&store, &mut instance, Action::StartRound {
            intent: intent.clone(),
            planned_charge: BudgetChargePlan {
                rounds: Some(charge.rounds),
                agent_invocations: Some(charge.agent_invocations),
                tokens: Some(charge.tokens),
                monetary_microunits: Some(charge.monetary_microunits),
                duration_millis: Some(charge.duration_millis),
            },
        })
        .unwrap();
        bounded_build_loop::commit(&store, &mut instance, Action::DispatchAcknowledged {
            dispatch_id: intent.id.clone(),
        })
        .unwrap();
        fs::create_dir(root.join("active")).unwrap();
        fs::write(root.join("active/mode"), mode).unwrap();
        let outcome = headless::run_workflow_snapshot_with_authority_provider_and_cancellation(
            &store,
            SnapshotRunRequest {
                workflow,
                run_id,
                owner: RunOwner::Orchestration {
                    instance_id: instance.id.clone(),
                    dispatch_id: intent.id.clone(),
                },
                agent_runtime: bounded_build_loop::runtime(&root),
                cancellation: not_cancelled(),
            },
            &OperatorRunContextSource::new(root.join("authority.ncl")),
        )
        .await
        .unwrap();
        assert!(!outcome.cancelled, "the actual command completed before cancellation intent");
        assert_eq!(outcome.summary.status == headless::HeadlessRunStatus::Succeeded, mode == "pass");
        // Deterministic race: record operator intent after observed product completion,
        // but before its durable orchestration settlement. No outcome is mocked.
        bounded_build_loop::commit(&store, &mut instance, Action::CancelRequested {
            actor: bounded_build_loop::ACTOR.into(),
        })
        .unwrap();
        bounded_build_loop::finish(
            &store,
            &mut instance,
            intent.clone(),
            &outcome,
            mode == "fail",
            "real completion preceded the cancellation request".into(),
        )
        .unwrap();
        assert_eq!(instance.status, expected);
        assert_eq!(instance.budgets.consumed, charge);
        assert!(instance.budgets.reservations.is_empty());
        assert!(
            instance.retry.retryable_failure.is_none(),
            "late cancellation must bar retries after a real failure"
        );
        assert!(
            bounded_build_loop::finish(
                &store,
                &mut instance,
                intent,
                &outcome,
                false,
                "duplicate completion cannot charge again".into()
            )
            .is_err()
        );
        assert_eq!(instance.budgets.consumed, charge);
        let replay = orchestration::replay(&store, instance.id.as_str()).unwrap();
        assert!(replay.valid);
        assert_eq!(replay.instance, instance);
    }
}
