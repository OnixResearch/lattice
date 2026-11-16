//! A reference host over Lattice's durable orchestration and production executor.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use lattice::agent_node::CommandOptions;
use lattice::agent_node::RuntimeOptions;
use lattice::engine::Workflow;
use lattice::engine::effect::standalone::core::Profile;
use lattice::engine::effect::standalone::{self};
use lattice::engine::runner;
use lattice::headless::HeadlessRunStatus;
use lattice::headless::SnapshotRunRequest;
use lattice::headless::{self};
use lattice::orchestration::*;
use lattice::store::RunOwner;
use lattice::store::Store;
use lattice::workflow_format::load_workflow_nickel_file;
use lattice::workflow_format::workflow_to_nickel;
use serde::Deserialize;
use serde::Serialize;

mod graph;
pub use graph::ACTOR;
pub use graph::ROUND_MILLIS;
pub use graph::Result;
pub use graph::TOTAL_MILLIS;
pub use graph::charge;
pub use graph::definition;
pub use graph::workflow;
const HANDOFFS: [&str; 3] = ["spec.txt", "build.rs", "verdict.txt"];
const SCRIPTS: [(&str, &str); 3] = [
    ("planner.sh", include_str!("planner.sh")),
    ("generator.sh", include_str!("generator.sh")),
    ("evaluator.sh", include_str!("evaluator.sh")),
];

fn plan() -> BudgetChargePlan {
    BudgetChargePlan {
        rounds: Some(1),
        agent_invocations: Some(2),
        duration_millis: Some(ROUND_MILLIS),
        tokens: Some(0),
        monetary_microunits: Some(0),
    }
}

pub fn runtime(root: &Path) -> RuntimeOptions {
    RuntimeOptions {
        runtimes: [("pi", "planner.sh"), ("clankers", "generator.sh")]
            .into_iter()
            .map(|(key, script)| {
                (key.into(), CommandOptions {
                    command: "sh".into(),
                    args: vec![
                        root.join(script).to_string_lossy().into_owned(),
                        "active".into(),
                        "{prompt}".into(),
                    ],
                })
            })
            .collect(),
        max_output_bytes: 16_384,
        ..RuntimeOptions::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenInputs {
    pub workflow_revision: WorkflowRevisionRef,
    pub definition_revision: DefinitionRevision,
    pub workflow_source: String,
    pub scripts: BTreeMap<String, String>,
    pub runtime: RuntimeOptions,
    pub mode: String,
}

/// Preparation creates only inert files. `run` requires an explicit local grant.
pub fn prepare(root: &Path, mode: &str) -> Result<()> {
    if !matches!(mode, "repair" | "pass" | "fail" | "cancel") {
        return Err("unknown producer mode".into());
    }
    fs::create_dir(root)?;
    let root = fs::canonicalize(root)?;
    let workflow = workflow(&root)?;
    let definition = definition(&workflow)?;
    let frozen = FrozenInputs {
        workflow_revision: workflow_revision(&workflow)?,
        definition_revision: definition_revision(&definition)?,
        workflow_source: workflow_to_nickel(&workflow),
        scripts: SCRIPTS.iter().map(|(name, source)| ((*name).into(), digest(source.as_bytes()))).collect(),
        runtime: runtime(&root),
        mode: mode.into(),
    };
    for (name, source) in SCRIPTS {
        fs::write(root.join(name), source)?;
    }
    fs::write(root.join("workflow.ncl"), &frozen.workflow_source)?;
    fs::write(root.join("definition.ncl"), definition_to_nickel(&definition)?)?;
    write_json(&root.join("frozen.json"), &frozen)?;
    let inspection = standalone::inspect_workflow(&workflow)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let profile = Profile {
        schema: inspection.schema,
        workflow_revision: inspection.workflow_revision,
        machine_identity: inspection.machine_identity,
        policy_ref: inspection.policy_ref,
        expires_at_unix_ms: u64::try_from(now)?.checked_add(3_600_000).ok_or("clock overflow")?,
        grants: inspection.grants,
        roots: BTreeMap::new(),
    };
    write_json(&root.join("authority.json"), &profile)?;
    fs::write(root.join("authority.ncl"), "import \"authority.json\"\n")?;
    Ok(())
}

pub fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn admit(instance: &Instance, workflow: &Workflow) -> Result<()> {
    let diagnostics = lattice::workflow_validation::validate_workflow(workflow);
    if lattice::workflow_validation::has_fatal_diagnostics(&diagnostics) {
        return Err(format!("graph admission rejected: {diagnostics:?}").into());
    }
    let build = runner::build_runner_plan(workflow);
    let plan = build.plan.ok_or("graph has no runner plan")?;
    if matches!(runner::admit_runner_plan(&plan), runner::RunnerAdmission::Rejected { .. }) {
        return Err("runner admission rejected".into());
    }
    if workflow_revision(workflow)? != instance.workflow_revision {
        return Err("stale or unapproved candidate revision; fixed profile cannot change topology".into());
    }
    if !validate_definition(&instance.definition, Some(&instance.workflow_revision)).is_empty() {
        return Err("definition admission rejected".into());
    }
    Ok(())
}

pub fn commit(store: &Store, instance: &mut Instance, action: Action) -> Result<()> {
    let fact = Fact {
        id: fact_identity(&instance.id, instance.version, &action)?,
        action,
    };
    let decision = transition(instance, fact)?;
    if let Some(event) = &decision.event {
        store.commit_orchestration_transition(instance.version, &decision.instance, event)?;
    }
    *instance = decision.instance;
    Ok(())
}

pub fn finish(
    store: &Store,
    instance: &mut Instance,
    intent: DispatchIntent,
    outcome: &headless::SnapshotRunOutcome,
    retryable_observed_failure: bool,
    safe_summary: String,
) -> Result<()> {
    commit(store, instance, Action::RunFinishedObserved {
        dispatch_id: intent.id,
        run_id: intent.run_id,
        succeeded: outcome.summary.status == HeadlessRunStatus::Succeeded,
        cancelled: outcome.cancelled,
        retryable_observed_failure,
        observed_usage: charge(),
        usage_basis: BudgetUsageBasis::ReservedFallback,
        safe_summary,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoundReceipt {
    pub round: u64,
    pub run_id: u64,
    pub revision: WorkflowRevisionRef,
    pub frozen_inputs: String,
    pub previous_verdict: Option<String>,
    pub handoffs: BTreeMap<String, String>,
    pub outcome: String,
    pub cancellation_requested: bool,
    pub owned_termination_observed: bool,
    pub deadline_reached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub instance_id: String,
    pub outcome: String,
    pub rounds: Vec<RoundReceipt>,
    pub total_charge: BudgetCharge,
}

/// All retries are durable RetryRound transitions. There is no second retry ledger.
// r[impl lattice.agent_orchestration.bounded_build_loop]
pub async fn run(root: &Path, cancellation: Arc<AtomicBool>) -> Result<Report> {
    let started = Instant::now();
    let root = fs::canonicalize(root)?;
    let frozen_bytes = fs::read(root.join("frozen.json"))?;
    let frozen: FrozenInputs = serde_json::from_slice(&frozen_bytes)?;
    let workflow = load_workflow_nickel_file(&root.join("workflow.ncl"))?;
    let definition = load_definition_nickel_file(&root.join("definition.ncl"))?;
    if workflow_revision(&workflow)? != frozen.workflow_revision
        || definition_revision(&definition)? != frozen.definition_revision
    {
        return Err("prepared snapshot identity mismatch".into());
    }
    for (name, hash) in &frozen.scripts {
        if digest(&fs::read(root.join(name))?) != *hash {
            return Err("prepared producer source changed".into());
        }
    }
    if frozen.runtime != runtime(&root) {
        return Err("prepared runtime recipe changed".into());
    }
    let store = Store::new(root.join("state.redb"))?;
    if !store.list_orchestrations()?.is_empty() {
        return Err("existing execution: inspect/replay; do not blindly retry".into());
    }
    let mut instance = new_instance(definition, store.next_orchestration_sequence()?, None)?;
    admit(&instance, &workflow)?;
    store.save_workflow(&workflow)?;
    store.save_orchestration_definition(&instance.definition)?;
    store.create_orchestration(&instance, &[])?;
    let authority = standalone::OperatorRunContextSource::new(root.join("authority.ncl"));
    let mut receipts = Vec::new();
    let mut previous_verdict = None;
    loop {
        // Re-read editable inputs only for drift detection, never to mutate the pinned snapshot.
        let candidate = match load_workflow_nickel_file(&root.join("workflow.ncl")) {
            Ok(candidate) => candidate,
            Err(error) => {
                fs::write(root.join("admission-rejection.txt"), error.to_string())?;
                return Err(error.into());
            }
        };
        if let Err(error) = admit(&instance, &candidate) {
            fs::write(root.join("admission-rejection.txt"), error.to_string())?;
            return Err(error);
        }
        if cancellation.load(Ordering::Acquire) {
            commit(&store, &mut instance, Action::CancelRequested { actor: ACTOR.into() })?;
            break;
        }
        let run_id = store.next_run_id()?;
        let intent = DispatchIntent {
            id: dispatch_identity(&instance, run_id)?,
            run_id,
            workflow_revision: instance.workflow_revision.clone(),
            status: DispatchStatus::Pending,
        };
        if instance.status == Status::Failed {
            if let Err(error) = preflight_retry(&instance, ACTOR, plan()) {
                if !matches!(error.code, DiagnosticCode::BudgetExceeded | DiagnosticCode::RetryLimitExceeded) {
                    return Err(error.into());
                }
                commit(&store, &mut instance, Action::BudgetExhausted {
                    safe_reason: error.message,
                })?;
                break;
            }
            commit(&store, &mut instance, Action::RetryRound {
                actor: ACTOR.into(),
                intent: intent.clone(),
                planned_charge: plan(),
            })?;
        } else {
            commit(&store, &mut instance, Action::StartRound {
                intent: intent.clone(),
                planned_charge: plan(),
            })?;
        }
        fs::create_dir(root.join("active"))?;
        fs::write(root.join("active/mode"), &frozen.mode)?;
        if let Some(text) = &previous_verdict {
            fs::write(root.join("active/previous-verdict.txt"), text)?;
        }
        write_json(
            &root.join("active/input.json"),
            &serde_json::json!({
                "round": instance.round, "revision": instance.workflow_revision,
                "frozen_inputs": digest(&frozen_bytes), "previous_verdict": previous_verdict,
                "construction": "fixed admitted DAG, no constructor inlining or generated topology",
                "registry": frozen.scripts, "runtime": frozen.runtime,
            }),
        )?;
        commit(&store, &mut instance, Action::DispatchAcknowledged {
            dispatch_id: intent.id.clone(),
        })?;
        let round_cancel = Arc::new(AtomicBool::new(false));
        let request = SnapshotRunRequest {
            workflow: workflow.clone(),
            run_id,
            owner: RunOwner::Orchestration {
                instance_id: instance.id.clone(),
                dispatch_id: intent.id.clone(),
            },
            agent_runtime: frozen.runtime.clone(),
            cancellation: Arc::clone(&round_cancel),
        };
        let remaining = Duration::from_millis(TOTAL_MILLIS).saturating_sub(started.elapsed());
        let deadline = tokio::time::sleep(remaining.min(Duration::from_millis(ROUND_MILLIS)));
        tokio::pin!(deadline);
        let execution =
            headless::run_workflow_snapshot_with_authority_provider_and_cancellation(&store, request, &authority);
        tokio::pin!(execution);
        let mut requested = false;
        let mut timed_out = false;
        let result = loop {
            tokio::select! {
                result = &mut execution => break result,
                _ = &mut deadline, if !requested => { timed_out = true; requested = true; },
                _ = tokio::time::sleep(Duration::from_millis(10)), if !requested => {
                    if cancellation.load(Ordering::Acquire) { requested = true; }
                }
            }
            if requested {
                commit(&store, &mut instance, Action::CancelRequested { actor: ACTOR.into() })?;
                round_cancel.store(true, Ordering::Release);
                break execution.await;
            }
        };
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) => {
                // A post-start persistence/observation error is not a retryable failure.
                fs::write(root.join("uncertain.txt"), error.to_string())?;
                return Err(error.into());
            }
        };
        if requested {
            write_json(
                &root.join("active/cancellation-observation.json"),
                &serde_json::json!({
                    "run_id": run_id.0, "dispatch_id": intent.id,
                    "requested": true, "owned_termination_observed": outcome.cancelled,
                    "deadline_reached": timed_out,
                }),
            )?;
        }
        let passed = outcome.summary.status == HeadlessRunStatus::Succeeded;
        let verdict_path = root.join("active/verdict.txt");
        if !verdict_path.exists() && requested {
            fs::write(
                &verdict_path,
                if outcome.cancelled {
                    "cancelled: owned process termination observed\n"
                } else {
                    "cancel requested: termination not observed\n"
                },
            )?;
        }
        let observed_failure = !passed && !requested && fs::read_to_string(&verdict_path)
            .is_ok_and(|verdict| verdict.starts_with("fail")) && store.load_run(run_id)?.is_some_and(|record|
            record.finished_epoch_ms.is_some() && record.failures > 0 && matches!(record.owner,
                RunOwner::Orchestration { instance_id, dispatch_id } if instance_id == instance.id && dispatch_id == intent.id));
        finish(
            &store,
            &mut instance,
            intent,
            &outcome,
            observed_failure,
            if timed_out {
                "owned round deadline reached".into()
            } else {
                format!("round verdict: {:?}", outcome.summary.status)
            },
        )?;
        let handoffs = HANDOFFS
            .into_iter()
            .filter_map(|name| fs::read(root.join("active").join(name)).ok().map(|bytes| (name.into(), digest(&bytes))))
            .collect();
        let receipt = RoundReceipt {
            round: instance.round,
            run_id: run_id.0,
            revision: instance.workflow_revision.clone(),
            frozen_inputs: digest(&frozen_bytes),
            previous_verdict: previous_verdict.as_ref().map(|text: &String| digest(text.as_bytes())),
            handoffs,
            outcome: if outcome.cancelled {
                "cancelled"
            } else if passed {
                "pass"
            } else {
                "fail"
            }
            .into(),
            cancellation_requested: requested,
            owned_termination_observed: outcome.cancelled,
            deadline_reached: timed_out,
        };
        previous_verdict = fs::read_to_string(&verdict_path).ok();
        write_json(&root.join("active/receipt.json"), &receipt)?;
        fs::rename(root.join("active"), root.join(format!("round-{}", instance.round)))?;
        receipts.push(receipt);
        if instance.status != Status::Failed || !observed_failure {
            break;
        }
    }
    let report = Report {
        instance_id: instance.id.to_string(),
        outcome: match instance.status {
            Status::Completed => "pass",
            Status::Exhausted => "exhausted",
            Status::Cancelled => "cancelled",
            _ => "fail",
        }
        .into(),
        rounds: receipts,
        total_charge: instance.budgets.consumed,
    };
    write_json(&root.join("report.json"), &report)?;
    Ok(report)
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
