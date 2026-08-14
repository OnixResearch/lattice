use std::cell::Cell;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::Path;
use std::rc::Rc;

use lattice::agent_evaluation::Error;
use lattice::agent_evaluation::artifacts::ArtifactCandidate;
use lattice::agent_evaluation::artifacts::ArtifactManifest;
use lattice::agent_evaluation::evidence::ANIMUS_FACT_SCHEMA;
use lattice::agent_evaluation::evidence::AnimusFact;
use lattice::agent_evaluation::evidence::Receipt;
use lattice::agent_evaluation::format::load_catalog;
use lattice::agent_evaluation::host::Driver;
use lattice::agent_evaluation::host::DriverConfig;
use lattice::agent_evaluation::host::DriverProcessPort;
use lattice::agent_evaluation::identity::hash_bytes;
use lattice::agent_evaluation::identity::hash_value;
use lattice::agent_evaluation::live::Admission;
use lattice::agent_evaluation::live::Observation;
use lattice::agent_evaluation::live::ObservationClass;
use lattice::agent_evaluation::model::AdmittedCase;
use lattice::agent_evaluation::model::ArtifactRole;
use lattice::agent_evaluation::model::GateStatus;
use lattice::agent_evaluation::model::MediaClass;
use lattice::agent_evaluation::model::ServiceKind;
use lattice::agent_evaluation::model::TrialStatus;
use lattice::agent_evaluation::oracles::GateEvidence;
use lattice::agent_evaluation::oracles::JudgeEvidence;
use lattice::agent_evaluation::planning::BudgetAvailability;
use lattice::agent_evaluation::planning::LiveAuthorityFact;
use lattice::agent_evaluation::planning::PlanRequest;
use lattice::agent_evaluation::planning::SubjectObservation;
use lattice::agent_evaluation::planning::TrialPlan;
use lattice::agent_evaluation::planning::plan_matrix;
use lattice::agent_evaluation::shell::AgentPort;
use lattice::agent_evaluation::shell::ArtifactPort;
use lattice::agent_evaluation::shell::CleanupPort;
use lattice::agent_evaluation::shell::ClockPort;
use lattice::agent_evaluation::shell::DeploymentPort;
use lattice::agent_evaluation::shell::ExecutionFact;
use lattice::agent_evaluation::shell::JudgePort;
use lattice::agent_evaluation::shell::RunRequest;
use lattice::agent_evaluation::shell::Services;
use lattice::agent_evaluation::shell::TrialStorePort;
use lattice::agent_evaluation::shell::VerifierPort;
use lattice::agent_evaluation::shell::WorkflowPort;
use lattice::agent_evaluation::shell::WorkspacePort;
use lattice::agent_evaluation::shell::run_trial;

const BUILTIN_ROOT: &str = "builtins/agent-evaluations";
const OBSERVED_EPOCH_MS: u64 = 1_800_000_000_000;
const ARTIFACT_BYTES: u64 = 16;
const JUDGE_SCORE: u16 = 7_500;
const COMPLETED_EPOCH_MS: u64 = 1_800_000_000_001;
const DRIVER_TIMEOUT_SECS: u64 = 30;
const DRIVER_OUTPUT_MAX_BYTES: u64 = 1_048_576;
const SUCCESS_EXIT_CODE: i32 = 0;

fn identity(label: &str) -> String {
    hash_bytes("lattice.agent-evaluation.shell-test.v1", label.as_bytes())
}

fn services() -> BTreeSet<ServiceKind> {
    [
        ServiceKind::Workspace,
        ServiceKind::Agent,
        ServiceKind::Workflow,
        ServiceKind::Verifier,
        ServiceKind::Judge,
        ServiceKind::Deployment,
        ServiceKind::Artifact,
        ServiceKind::Cleanup,
        ServiceKind::Clock,
        ServiceKind::Store,
    ]
    .into_iter()
    .collect()
}

fn plan(
    case_id: &str,
    live: bool,
) -> (lattice::agent_evaluation::model::AdmittedCatalog, TrialPlan, Option<LiveAuthorityFact>) {
    let catalog = load_catalog(&Path::new(env!("CARGO_MANIFEST_DIR")).join(BUILTIN_ROOT)).expect("catalog");
    let case = catalog.cases.iter().find(|case| case.definition.id == case_id).expect("case");
    let subject_identity =
        hash_value("lattice.agent-evaluation.subject.v1", &case.definition.subject).expect("subject");
    let authority = live.then(|| LiveAuthorityFact {
        case_id: case_id.to_string(),
        actor: "operator-reviewed".to_string(),
        target_namespace: "eval-sandbox".to_string(),
        credential_handles: vec!["secret-ref-eval-sandbox".to_string()],
        max_deployments: 1,
    });
    let budget = case.definition.budget;
    let request = PlanRequest {
        selected_case_ids: vec![case_id.to_string()],
        subject_observations: vec![SubjectObservation {
            case_id: case_id.to_string(),
            subject_identity: subject_identity.clone(),
        }],
        available_services: services(),
        budget_availability: vec![BudgetAvailability {
            case_id: case_id.to_string(),
            trials: budget.trials,
            concurrency: budget.concurrency,
            tokens: budget.max_tokens,
            duration_ms: budget.max_duration_ms,
            deployments: budget.max_deployments,
        }],
        live_authority: authority.clone().into_iter().collect(),
        prior_trials: Vec::new(),
    };
    let trial = plan_matrix(&catalog, &request).expect("plan").trials.remove(0);
    (catalog, trial, authority)
}

struct MockHost {
    subject_identity: String,
    events: RefCell<Vec<&'static str>>,
    fail_execution: Cell<bool>,
    private_animus_fact: Cell<bool>,
    deployment_class: Cell<ObservationClass>,
}

impl MockHost {
    fn new(subject_identity: String) -> Self {
        Self {
            subject_identity,
            events: RefCell::new(Vec::new()),
            fail_execution: Cell::new(false),
            private_animus_fact: Cell::new(false),
            deployment_class: Cell::new(ObservationClass::Validated),
        }
    }

    fn push(&self, event: &'static str) {
        self.events.borrow_mut().push(event);
    }

    fn execute(&self, route: &'static str) -> Result<ExecutionFact, Error> {
        self.push(route);
        if self.fail_execution.get() {
            return Err(Error::ServiceFailure);
        }
        Ok(ExecutionFact {
            output_identity: identity("output"),
            animus_fact: Some(AnimusFact {
                schema: ANIMUS_FACT_SCHEMA.to_string(),
                task_identity: identity("animus-task"),
                run_identity: identity("animus-run"),
                artifact_manifest_identity: identity("animus-artifacts"),
                outcome_class: "completed".to_string(),
                item_count: 1,
                byte_count: ARTIFACT_BYTES,
                contains_private_content: self.private_animus_fact.get(),
            }),
        })
    }
}

struct ProtocolProcess {
    events: Rc<RefCell<Vec<String>>>,
    started: Rc<Cell<bool>>,
    drift_driver_identity: bool,
}

impl DriverProcessPort for ProtocolProcess {
    fn run(
        &self,
        request: &lattice::agent_node::ProcessRequest,
    ) -> Result<lattice::agent_node::ProcessOutput, lattice::agent_node::ProcessError> {
        let value: serde_json::Value = serde_json::from_slice(&request.stdin)
            .map_err(|_| lattice::agent_node::ProcessError::Spawn("invalid protocol request".to_string()))?;
        let operation = value["operation"]
            .as_str()
            .ok_or_else(|| lattice::agent_node::ProcessError::Spawn("missing protocol operation".to_string()))?;
        if operation == "agent_execute" && !self.started.get() {
            return Err(lattice::agent_node::ProcessError::Spawn("trial start was not durable".to_string()));
        }
        self.events.borrow_mut().push(operation.to_string());
        let result = protocol_result(operation, &value)?;
        let driver_identity = if self.drift_driver_identity {
            identity("wrong-driver")
        } else {
            value["driver_identity"].as_str().unwrap_or_default().to_string()
        };
        let response = serde_json::json!({
            "schema": "lattice.agent-evaluation-driver-response.v1",
            "driver_identity": driver_identity,
            "trial_identity": value["trial"]["trial_identity"],
            "result": result,
        });
        Ok(lattice::agent_node::ProcessOutput {
            status_code: Some(SUCCESS_EXIT_CODE),
            stdout: serde_json::to_vec(&response)
                .map_err(|_| lattice::agent_node::ProcessError::Spawn("response encoding failed".to_string()))?,
            stderr: Vec::new(),
        })
    }
}

fn protocol_result(
    operation: &str,
    request: &serde_json::Value,
) -> Result<serde_json::Value, lattice::agent_node::ProcessError> {
    match operation {
        "workspace_prepare" => Ok(serde_json::json!({
            "kind": "identity",
            "identity": request["trial"]["subject_identity"],
        })),
        "agent_execute" => Ok(serde_json::json!({
            "kind": "execution",
            "output_identity": identity("driver-output"),
            "animus_fact": null,
        })),
        "verify" => {
            let gates = request["case_definition"]["gates"]
                .as_array()
                .ok_or_else(|| lattice::agent_node::ProcessError::Spawn("missing gates".to_string()))?;
            let evidence = gates
                .iter()
                .map(|gate| {
                    serde_json::json!({
                        "gate_id": gate["id"],
                        "implementation_identity": gate["implementation_identity"],
                        "input_identity": gate["input_identity"],
                        "evidence_schema": gate["evidence_schema"],
                        "status": "passed",
                        "agent_controlled": false,
                    })
                })
                .collect::<Vec<_>>();
            Ok(serde_json::json!({ "kind": "gates", "evidence": evidence }))
        }
        "collect_artifacts" => Ok(serde_json::json!({
            "kind": "artifacts",
            "candidates": [{
                "role": "result",
                "media": "json",
                "relative_path": "results/driver-summary.json",
                "content_identity": identity("driver-artifact"),
                "byte_count": ARTIFACT_BYTES,
                "symlink": false,
                "remote": false,
                "secret_bearing": false,
                "hidden_material_bearing": false,
                "redacted": true,
            }],
        })),
        "persist_artifacts" => Ok(serde_json::json!({ "kind": "persisted" })),
        "capture_post_state" => Ok(serde_json::json!({
            "kind": "identity",
            "identity": identity("driver-post-state"),
        })),
        _ => Err(lattice::agent_node::ProcessError::Spawn("unsupported protocol operation".to_string())),
    }
}

struct RecordingStore<'a> {
    store: &'a lattice::store::Store,
    started: Rc<Cell<bool>>,
}

impl TrialStorePort for RecordingStore<'_> {
    fn record_start(&self, trial: &TrialPlan, observed_epoch_ms: u64) -> Result<(), Error> {
        self.store.record_evaluation_start(trial, observed_epoch_ms).map_err(|_| Error::ServiceFailure)?;
        self.started.set(true);
        Ok(())
    }

    fn cancellation_requested(&self, trial: &TrialPlan) -> Result<bool, Error> {
        TrialStorePort::cancellation_requested(self.store, trial)
    }

    fn record_cancelled(&self, trial: &TrialPlan, observed_epoch_ms: u64) -> Result<(), Error> {
        TrialStorePort::record_cancelled(self.store, trial, observed_epoch_ms)
    }

    fn record_unknown(&self, trial: &TrialPlan, observed_epoch_ms: u64) -> Result<(), Error> {
        TrialStorePort::record_unknown(self.store, trial, observed_epoch_ms)
    }

    fn record_receipt(&self, receipt: &Receipt, observed_epoch_ms: u64) -> Result<(), Error> {
        TrialStorePort::record_receipt(self.store, receipt, observed_epoch_ms)
    }
}

impl WorkspacePort for MockHost {
    fn prepare(&self, _case: &AdmittedCase, _trial: &TrialPlan) -> Result<String, Error> {
        self.push("prepare");
        Ok(self.subject_identity.clone())
    }

    fn capture_post_state(&self, _case: &AdmittedCase, _trial: &TrialPlan) -> Result<String, Error> {
        self.push("capture");
        Ok(identity("post-state"))
    }
}

impl AgentPort for MockHost {
    fn execute(&self, _case: &AdmittedCase, _trial: &TrialPlan) -> Result<ExecutionFact, Error> {
        self.execute("agent")
    }
}

impl WorkflowPort for MockHost {
    fn execute(&self, _case: &AdmittedCase, _trial: &TrialPlan) -> Result<ExecutionFact, Error> {
        self.execute("workflow")
    }
}

impl VerifierPort for MockHost {
    fn verify(
        &self,
        case: &AdmittedCase,
        _trial: &TrialPlan,
        _execution: &ExecutionFact,
    ) -> Result<Vec<GateEvidence>, Error> {
        self.push("verify");
        Ok(case
            .definition
            .gates
            .iter()
            .map(|gate| GateEvidence {
                gate_id: gate.id.clone(),
                implementation_identity: gate.implementation_identity.clone(),
                input_identity: gate.input_identity.clone(),
                evidence_schema: gate.evidence_schema.clone(),
                status: GateStatus::Passed,
                agent_controlled: false,
            })
            .collect())
    }
}

impl JudgePort for MockHost {
    fn judge(
        &self,
        case: &AdmittedCase,
        _trial: &TrialPlan,
        _execution: &ExecutionFact,
    ) -> Result<JudgeEvidence, Error> {
        self.push("judge");
        let rubric = case.definition.rubric.as_ref().ok_or(Error::InvalidJudgeEvidence)?;
        Ok(JudgeEvidence {
            rubric_identity: rubric.identity.clone(),
            judge_profile_identity: rubric.judge_profile_identity.clone(),
            score_basis_points: JUDGE_SCORE,
            advisory: true,
            contains_private_content: false,
        })
    }
}

impl DeploymentPort for MockHost {
    fn deploy(
        &self,
        _case: &AdmittedCase,
        admission: &Admission,
        _trial: &TrialPlan,
        _execution: &ExecutionFact,
    ) -> Result<Observation, Error> {
        self.push("deploy");
        let class = self.deployment_class.get();
        Ok(Observation {
            admission_identity: admission.identity.clone(),
            class,
            deployment_identity: (class != ObservationClass::Unknown).then(|| identity("deployment")),
            cleanup_confirmed: false,
        })
    }
}

impl CleanupPort for MockHost {
    fn cleanup(
        &self,
        _case: &AdmittedCase,
        _trial: &TrialPlan,
        _admission: &Admission,
        _observation: &Observation,
    ) -> Result<bool, Error> {
        self.push("cleanup");
        Ok(true)
    }
}

impl ArtifactPort for MockHost {
    fn collect(
        &self,
        _case: &AdmittedCase,
        _trial: &TrialPlan,
        _execution: &ExecutionFact,
    ) -> Result<Vec<ArtifactCandidate>, Error> {
        self.push("collect");
        Ok(vec![ArtifactCandidate {
            role: ArtifactRole::Result,
            media: MediaClass::Json,
            relative_path: "results/summary.json".to_string(),
            content_identity: identity("artifact"),
            byte_count: ARTIFACT_BYTES,
            symlink: false,
            remote: false,
            secret_bearing: false,
            hidden_material_bearing: false,
            redacted: true,
        }])
    }

    fn persist(&self, _case: &AdmittedCase, _trial: &TrialPlan, _manifest: &ArtifactManifest) -> Result<(), Error> {
        self.push("persist-artifact");
        Ok(())
    }
}

impl ClockPort for MockHost {
    fn observed_epoch_ms(&self) -> Result<u64, Error> {
        Ok(OBSERVED_EPOCH_MS)
    }
}

impl TrialStorePort for MockHost {
    fn record_start(&self, _trial: &TrialPlan, _observed_epoch_ms: u64) -> Result<(), Error> {
        self.push("start");
        Ok(())
    }

    fn cancellation_requested(&self, _trial: &TrialPlan) -> Result<bool, Error> {
        Ok(false)
    }

    fn record_cancelled(&self, _trial: &TrialPlan, _observed_epoch_ms: u64) -> Result<(), Error> {
        self.push("cancelled");
        Ok(())
    }

    fn record_unknown(&self, _trial: &TrialPlan, _observed_epoch_ms: u64) -> Result<(), Error> {
        self.push("unknown");
        Ok(())
    }

    fn record_receipt(&self, _receipt: &Receipt, _observed_epoch_ms: u64) -> Result<(), Error> {
        self.push("receipt");
        Ok(())
    }
}

fn host_services(host: &MockHost) -> Services<'_> {
    Services {
        workspace: host,
        agent: host,
        workflow: host,
        verifier: host,
        judge: host,
        deployment: host,
        cleanup: host,
        artifact: host,
        clock: host,
        store: host,
    }
}

fn command_host_config(trial: &TrialPlan) -> DriverConfig {
    DriverConfig {
        driver_identity: identity("command-driver"),
        agent_profile_identity: trial.agent_profile_identity.clone(),
        environment_identity: trial.environment_identity.clone(),
        policy_identity: trial.policy_identity.clone(),
        program: "/bin/true".to_string(),
        args: Vec::new(),
        current_directory: env!("CARGO_MANIFEST_DIR").to_string(),
        timeout_secs: DRIVER_TIMEOUT_SECS,
        output_max_bytes: DRIVER_OUTPUT_MAX_BYTES,
    }
}

// r[verify lattice.agent_evaluation.execution.local]
// r[verify lattice.agent_evaluation.oracles.pass]
// r[verify lattice.agent_evaluation.evidence.safe]
// r[verify lattice.agent_evaluation.evidence.receipt_rejected]
// r[verify lattice.agent_evaluation.verification.positive]
#[test]
fn local_runner_persists_start_before_agent_and_emits_metadata_only_receipt() {
    let (catalog, trial, authority) = plan("unit-local", false);
    assert!(authority.is_none());
    let host = MockHost::new(trial.subject_identity.clone());
    let receipt = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: None,
        },
        &host_services(&host),
    )
    .expect("local receipt");

    let events = host.events.borrow();
    let start = events.iter().position(|event| *event == "start").expect("start event");
    let agent = events.iter().position(|event| *event == "agent").expect("agent event");
    assert!(start < agent);
    assert_eq!(events.last(), Some(&"receipt"));
    assert_eq!(receipt.status, TrialStatus::Passed);
    assert!(receipt.animus_fact_identity.is_some());
    let public = serde_json::to_string(&receipt).expect("receipt JSON");
    assert!(!public.contains("prompt"));
    assert!(!public.contains("transcript"));
    assert!(!public.contains("secret-ref"));
}

// r[verify lattice.agent_evaluation.execution.local]
// r[verify lattice.agent_evaluation.verification.positive]
#[test]
fn bounded_command_host_runs_the_production_protocol_after_durable_start() {
    let (catalog, trial, _) = plan("unit-local", false);
    let directory = tempfile::tempdir().expect("store directory");
    let app_config = lattice::config::AppConfig {
        data_dir: directory.path().to_path_buf(),
        ..lattice::config::AppConfig::default()
    };
    let store = lattice::store::Store::new(lattice::config::store_path(&app_config.data_dir)).expect("store");
    let events = Rc::new(RefCell::new(Vec::new()));
    let started = Rc::new(Cell::new(false));
    let host = Driver::new_with_process(
        command_host_config(&trial),
        [OBSERVED_EPOCH_MS, COMPLETED_EPOCH_MS],
        ProtocolProcess {
            events: Rc::clone(&events),
            started: Rc::clone(&started),
            drift_driver_identity: false,
        },
    )
    .expect("command host");
    let recording_store = RecordingStore {
        store: &store,
        started: Rc::clone(&started),
    };
    let receipt = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: None,
        },
        &Services {
            workspace: &host,
            agent: &host,
            workflow: &host,
            verifier: &host,
            judge: &host,
            deployment: &host,
            cleanup: &host,
            artifact: &host,
            clock: &host,
            store: &recording_store,
        },
    )
    .expect("command-host receipt");

    assert!(started.get());
    assert_eq!(receipt.status, TrialStatus::Passed);
    assert_eq!(events.borrow().as_slice(), [
        "workspace_prepare",
        "agent_execute",
        "verify",
        "collect_artifacts",
        "persist_artifacts",
        "capture_post_state",
    ]);
    let durable = store.load_evaluation_receipt(&receipt.identity).expect("receipt lookup").expect("durable receipt");
    lattice::agent_evaluation::evidence::validate_receipt(&durable).expect("valid durable receipt");
    drop(recording_store);
    drop(host);
    drop(store);
    let artifact_exit =
        lattice::agent_evaluation::cli::run(&app_config, &lattice::agent_evaluation::cli::CommandArgs {
            command: lattice::agent_evaluation::cli::Command::Artifacts {
                trial: trial.trial_identity.clone(),
                output: lattice::agent_evaluation::cli::OutputFormat::Json,
            },
        });
    assert_eq!(artifact_exit, std::process::ExitCode::SUCCESS);
    let resume_exit = lattice::agent_evaluation::cli::run(&app_config, &lattice::agent_evaluation::cli::CommandArgs {
        command: lattice::agent_evaluation::cli::Command::Resume(lattice::agent_evaluation::cli::PlanArgs {
            root: Path::new(env!("CARGO_MANIFEST_DIR")).join(BUILTIN_ROOT),
            request: Path::new(env!("CARGO_MANIFEST_DIR")).join("builtins/agent-evaluations/requests/unit-local.ncl"),
            output: lattice::agent_evaluation::cli::OutputFormat::Json,
        }),
    });
    assert_eq!(resume_exit, std::process::ExitCode::SUCCESS);
}

// r[verify lattice.agent_evaluation.verification.negative]
#[test]
fn bounded_command_host_rejects_driver_identity_drift_before_trial_start() {
    let (catalog, trial, _) = plan("unit-local", false);
    let directory = tempfile::tempdir().expect("store directory");
    let store = lattice::store::Store::new(directory.path().join("driver-drift.redb")).expect("store");
    let started = Rc::new(Cell::new(false));
    let host = Driver::new_with_process(
        command_host_config(&trial),
        [OBSERVED_EPOCH_MS, COMPLETED_EPOCH_MS],
        ProtocolProcess {
            events: Rc::new(RefCell::new(Vec::new())),
            started: Rc::clone(&started),
            drift_driver_identity: true,
        },
    )
    .expect("command host");
    let recording_store = RecordingStore {
        store: &store,
        started: Rc::clone(&started),
    };
    let result = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: None,
        },
        &Services {
            workspace: &host,
            agent: &host,
            workflow: &host,
            verifier: &host,
            judge: &host,
            deployment: &host,
            cleanup: &host,
            artifact: &host,
            clock: &host,
            store: &recording_store,
        },
    );

    assert_eq!(result, Err(Error::ServiceFailure));
    assert!(!started.get());
    assert!(store.load_evaluation_trial(&trial.trial_identity).expect("trial lookup").is_none());
}

// r[verify lattice.agent_evaluation.execution.subject_changed]
#[test]
fn subject_drift_blocks_before_durable_start_or_agent_effect() {
    let (catalog, trial, _) = plan("unit-local", false);
    let host = MockHost::new(identity("changed-subject"));
    let result = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: None,
        },
        &host_services(&host),
    );

    assert_eq!(result, Err(Error::SubjectDrift));
    assert_eq!(host.events.borrow().as_slice(), ["prepare"]);
}

// r[verify lattice.agent_evaluation.execution.ambiguous]
// r[verify lattice.agent_evaluation.verification.negative]
#[test]
fn execution_failure_records_unknown_after_durable_start() {
    let (catalog, trial, _) = plan("unit-local", false);
    let host = MockHost::new(trial.subject_identity.clone());
    host.fail_execution.set(true);
    let result = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: None,
        },
        &host_services(&host),
    );

    assert_eq!(result, Err(Error::ServiceFailure));
    assert_eq!(host.events.borrow().as_slice(), ["prepare", "start", "agent", "unknown"]);
}

// r[verify lattice.agent_evaluation.evidence.receipt_rejected]
// r[verify lattice.agent_evaluation.verification.negative]
#[test]
fn private_animus_fact_is_rejected_without_receipt_acceptance() {
    let (catalog, trial, _) = plan("unit-local", false);
    let host = MockHost::new(trial.subject_identity.clone());
    host.private_animus_fact.set(true);
    let result = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: None,
        },
        &host_services(&host),
    );

    assert_eq!(result, Err(Error::InvalidAnimusFact));
    assert!(!host.events.borrow().contains(&"receipt"));
    assert!(host.events.borrow().contains(&"unknown"));
}

// r[verify lattice.agent_evaluation.live.admitted]
// r[verify lattice.agent_evaluation.verification.positive]
#[test]
fn live_runner_uses_workflow_then_deployment_and_cleanup_under_exact_authority() {
    let (catalog, trial, authority) = plan("live-sandbox", true);
    let host = MockHost::new(trial.subject_identity.clone());
    let receipt = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: authority.as_ref(),
        },
        &host_services(&host),
    )
    .expect("live receipt");

    let events = host.events.borrow();
    let start = events.iter().position(|event| *event == "start").expect("start");
    let workflow = events.iter().position(|event| *event == "workflow").expect("workflow");
    let deploy = events.iter().position(|event| *event == "deploy").expect("deploy");
    let cleanup = events.iter().position(|event| *event == "cleanup").expect("cleanup");
    assert!(start < workflow && workflow < deploy && deploy < cleanup);
    assert_eq!(receipt.status, TrialStatus::Passed);
    assert!(receipt.deployment_identity.is_some());
}

// r[verify lattice.agent_evaluation.live.unknown]
// r[verify lattice.agent_evaluation.execution.ambiguous]
#[test]
fn live_unknown_outcome_is_retained_and_not_reported_as_success() {
    let (catalog, trial, authority) = plan("live-sandbox", true);
    let host = MockHost::new(trial.subject_identity.clone());
    host.deployment_class.set(ObservationClass::Unknown);
    let receipt = run_trial(
        &RunRequest {
            catalog: &catalog,
            trial: &trial,
            policy_identity: &trial.policy_identity,
            live_authority: authority.as_ref(),
        },
        &host_services(&host),
    )
    .expect("unknown receipt");

    assert_eq!(receipt.status, TrialStatus::Unknown);
    assert!(host.events.borrow().contains(&"cleanup"));
}

// r[verify lattice.agent_evaluation.execution.ambiguous]
// r[verify lattice.agent_evaluation.verification.closeout]
#[test]
fn redb_store_preserves_started_trial_and_cancel_request_across_restart() {
    let (_catalog, trial, _) = plan("unit-local", false);
    let directory = tempfile::tempdir().expect("store directory");
    let path = directory.path().join("evaluation.redb");
    let store = lattice::store::Store::new(&path).expect("store");
    store.record_evaluation_start(&trial, OBSERVED_EPOCH_MS).expect("durable start");
    store.request_evaluation_cancel(&trial.trial_identity, OBSERVED_EPOCH_MS).expect("cancel request");
    assert!(store.record_evaluation_start(&trial, OBSERVED_EPOCH_MS).is_err());
    drop(store);

    let reopened = lattice::store::Store::new(&path).expect("reopen store");
    let recovered = reopened.load_evaluation_trial(&trial.trial_identity).expect("load trial").expect("stored trial");
    assert_eq!(recovered.status, TrialStatus::Started);
    assert!(recovered.cancel_requested);
    TrialStorePort::record_cancelled(&reopened, &trial, OBSERVED_EPOCH_MS).expect("record cancelled");
    let cancelled = reopened
        .load_evaluation_trial(&trial.trial_identity)
        .expect("load cancelled trial")
        .expect("cancelled trial");
    assert_eq!(cancelled.status, TrialStatus::Cancelled);
    assert!(cancelled.receipt_identity.is_none());
}
