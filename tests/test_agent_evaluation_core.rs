use std::collections::BTreeSet;
use std::path::Path;

use lattice::agent_evaluation::Error;
use lattice::agent_evaluation::artifacts::ArtifactCandidate;
use lattice::agent_evaluation::artifacts::admit;
use lattice::agent_evaluation::catalog::validate;
use lattice::agent_evaluation::format::load_catalog;
use lattice::agent_evaluation::identity::hash_bytes;
use lattice::agent_evaluation::identity::hash_value;
use lattice::agent_evaluation::live::Observation;
use lattice::agent_evaluation::live::ObservationClass;
use lattice::agent_evaluation::live::admit as admit_live;
use lattice::agent_evaluation::live::reduce as reduce_live_observation;
use lattice::agent_evaluation::model::AdmittedCatalog;
use lattice::agent_evaluation::model::ArtifactRole;
use lattice::agent_evaluation::model::AssetFact;
use lattice::agent_evaluation::model::Catalog;
use lattice::agent_evaluation::model::GateStatus;
use lattice::agent_evaluation::model::MediaClass;
use lattice::agent_evaluation::model::ServiceKind;
use lattice::agent_evaluation::model::TrialStatus;
use lattice::agent_evaluation::oracles::GateEvidence;
use lattice::agent_evaluation::oracles::JudgeEvidence;
use lattice::agent_evaluation::oracles::reduce as reduce_oracles;
use lattice::agent_evaluation::planning::BudgetAvailability;
use lattice::agent_evaluation::planning::LiveAuthorityFact;
use lattice::agent_evaluation::planning::PlanRequest;
use lattice::agent_evaluation::planning::PriorTrial;
use lattice::agent_evaluation::planning::ResumeDecision;
use lattice::agent_evaluation::planning::SubjectObservation;
use lattice::agent_evaluation::planning::plan_matrix;

const BUILTIN_ROOT: &str = "builtins/agent-evaluations";
const BUILTIN_PLAN_REQUEST: &str = "builtins/agent-evaluations/requests/unit-local.ncl";
const ALTERED_ASSET: &str = "altered\n";
const BYTE_COUNT: u64 = 1;
const TOO_MANY_TRIALS: u32 = 17;
const ADVISORY_SCORE: u16 = 9_000;

fn builtins() -> AdmittedCatalog {
    load_catalog(&Path::new(env!("CARGO_MANIFEST_DIR")).join(BUILTIN_ROOT)).expect("built-in catalog")
}

fn identity(label: &str) -> String {
    hash_bytes("lattice.agent-evaluation.test.v1", label.as_bytes())
}

fn source_catalog(catalog: &AdmittedCatalog) -> Catalog {
    Catalog {
        schema: lattice::agent_evaluation::model::CATALOG_SCHEMA.to_string(),
        cases: catalog.cases.iter().map(|case| case.definition.clone()).collect(),
    }
}

fn asset_facts(catalog: &Catalog) -> Vec<AssetFact> {
    catalog
        .cases
        .iter()
        .map(|case| AssetFact {
            relative_path: case.asset_path.clone(),
            content_identity: case.asset_identity.clone(),
            byte_count: BYTE_COUNT,
            regular_file: true,
            symlink: false,
        })
        .collect()
}

fn all_services() -> BTreeSet<ServiceKind> {
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

fn subject_observation(catalog: &AdmittedCatalog, case_id: &str) -> SubjectObservation {
    let case = catalog.cases.iter().find(|case| case.definition.id == case_id).expect("selected case");
    SubjectObservation {
        case_id: case_id.to_string(),
        subject_identity: hash_value("lattice.agent-evaluation.subject.v1", &case.definition.subject)
            .expect("subject identity"),
    }
}

fn request(catalog: &AdmittedCatalog, case_id: &str) -> PlanRequest {
    let case = catalog.cases.iter().find(|case| case.definition.id == case_id).expect("budget case");
    let budget = case.definition.budget;
    PlanRequest {
        selected_case_ids: vec![case_id.to_string()],
        subject_observations: vec![subject_observation(catalog, case_id)],
        available_services: all_services(),
        budget_availability: vec![BudgetAvailability {
            case_id: case_id.to_string(),
            trials: budget.trials,
            concurrency: budget.concurrency,
            tokens: budget.max_tokens,
            duration_ms: budget.max_duration_ms,
            deployments: budget.max_deployments,
        }],
        live_authority: Vec::new(),
        prior_trials: Vec::new(),
    }
}

// r[verify lattice.agent_evaluation.catalog.valid]
// r[verify lattice.agent_evaluation.case_contract.local]
// r[verify lattice.agent_evaluation.case_contract.hard_live]
#[test]
fn built_in_catalog_loads_with_all_classes_and_stable_identity() {
    let first = builtins();
    let second = builtins();
    let classes = first.cases.iter().map(|case| case.definition.class).collect::<BTreeSet<_>>();

    assert_eq!(first.cases.len(), 4);
    assert_eq!(classes.len(), 4);
    assert_eq!(first.identity, second.identity);
}

// r[verify lattice.agent_evaluation.catalog.drift]
// r[verify lattice.agent_evaluation.case_contract.drift]
#[test]
fn catalog_rejects_uppercase_identity_budget_overflow_and_stale_reference() {
    let admitted = builtins();

    let mut uppercase = source_catalog(&admitted);
    uppercase.cases[0].agent_profile_identity = uppercase.cases[0].agent_profile_identity.to_uppercase();
    let uppercase_assets = asset_facts(&uppercase);
    assert_eq!(validate(uppercase, &uppercase_assets), Err(Error::InvalidIdentity));

    let mut overflow = source_catalog(&admitted);
    overflow.cases[0].budget.trials = TOO_MANY_TRIALS;
    let overflow_assets = asset_facts(&overflow);
    assert_eq!(validate(overflow, &overflow_assets), Err(Error::InvalidBudget));

    let mut stale = source_catalog(&admitted);
    stale.cases[0].references[0].freshness_required = true;
    stale.cases[0].references[0].fresh = false;
    let stale_assets = asset_facts(&stale);
    assert_eq!(validate(stale, &stale_assets), Err(Error::InvalidOracle));
}

// r[verify lattice.agent_evaluation.catalog.drift]
#[test]
fn production_loader_rejects_asset_drift_and_symlink() {
    let fixture = tempfile::tempdir().expect("fixture root");
    let root = fixture.path();
    std::fs::create_dir(root.join("cases")).expect("case directory");
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(BUILTIN_ROOT);
    std::fs::copy(source.join("catalog.ncl"), root.join("catalog.ncl")).expect("catalog copy");
    for name in [
        "unit-local.txt",
        "skill-local.txt",
        "hard-local.txt",
        "live-sandbox.txt",
    ] {
        std::fs::copy(source.join("cases").join(name), root.join("cases").join(name)).expect("asset copy");
    }
    std::fs::write(root.join("cases/unit-local.txt"), ALTERED_ASSET).expect("alter asset");
    assert_eq!(load_catalog(root), Err(Error::AssetParity));

    #[cfg(unix)]
    {
        std::fs::remove_file(root.join("cases/unit-local.txt")).expect("remove altered asset");
        std::os::unix::fs::symlink(source.join("cases/unit-local.txt"), root.join("cases/unit-local.txt"))
            .expect("symlink asset");
        assert_eq!(load_catalog(root), Err(Error::FileAccess));
    }
}

// r[verify lattice.agent_evaluation.planning.valid]
// r[verify lattice.agent_evaluation.planning.blocked]
#[test]
fn matrix_planning_is_deterministic_and_fails_closed_without_services_or_authority() {
    let catalog = builtins();
    let local = request(&catalog, "unit-local");
    let first = plan_matrix(&catalog, &local).expect("first plan");
    let second = plan_matrix(&catalog, &local).expect("second plan");
    assert_eq!(first, second);
    assert_eq!(first.trials.len(), 2);

    let mut missing = request(&catalog, "unit-local");
    missing.available_services.remove(&ServiceKind::Agent);
    assert_eq!(plan_matrix(&catalog, &missing), Err(Error::ServiceUnavailable));

    let mut exhausted = request(&catalog, "unit-local");
    exhausted.budget_availability[0].tokens = 0;
    assert_eq!(plan_matrix(&catalog, &exhausted), Err(Error::InvalidBudget));

    let live = request(&catalog, "live-sandbox");
    assert_eq!(plan_matrix(&catalog, &live), Err(Error::AuthorityDenied));
}

// r[verify lattice.agent_evaluation.live.admitted]
// r[verify lattice.agent_evaluation.live.denied]
#[test]
fn live_planning_accepts_exact_sandbox_authority_and_rejects_production() {
    let catalog = builtins();
    let mut allowed = request(&catalog, "live-sandbox");
    allowed.live_authority.push(LiveAuthorityFact {
        case_id: "live-sandbox".to_string(),
        actor: "operator-reviewed".to_string(),
        target_namespace: "eval-sandbox".to_string(),
        credential_handles: vec!["secret-ref-eval-sandbox".to_string()],
        max_deployments: 1,
    });
    let plan = plan_matrix(&catalog, &allowed).expect("live plan");
    assert!(plan.trials[0].live);

    allowed.live_authority[0].target_namespace = "production".to_string();
    assert_eq!(plan_matrix(&catalog, &allowed), Err(Error::AuthorityDenied));
}

// r[verify lattice.agent_evaluation.planning.resume]
#[test]
fn exact_receipt_skips_but_drift_reruns_and_unknown_blocks() {
    let catalog = builtins();
    let base = request(&catalog, "unit-local");
    let plan = plan_matrix(&catalog, &base).expect("initial plan");
    let case = catalog.cases.iter().find(|case| case.definition.id == "unit-local").expect("case");
    let trial = &plan.trials[0];
    let gate_set_identity = hash_value("lattice.agent-evaluation.gate-set.v1", &case.definition.gates).expect("gates");
    let reference_set_identity =
        hash_value("lattice.agent-evaluation.reference-set.v1", &case.definition.references).expect("references");
    let receipt_identity = identity("receipt");
    let prior = PriorTrial {
        trial_identity: trial.trial_identity.clone(),
        case_revision_identity: case.revision_identity.clone(),
        subject_identity: trial.subject_identity.clone(),
        agent_profile_identity: trial.agent_profile_identity.clone(),
        environment_identity: trial.environment_identity.clone(),
        policy_identity: trial.policy_identity.clone(),
        gate_set_identity,
        rubric_identity: None,
        reference_set_identity,
        artifact_policy_identity: case.definition.artifacts.identity.clone(),
        status: TrialStatus::Passed,
        receipt_identity: Some(receipt_identity.clone()),
        expected_receipt_identity: Some(receipt_identity),
    };
    let mut resumed = request(&catalog, "unit-local");
    resumed.prior_trials.push(prior.clone());
    let resumed_plan = plan_matrix(&catalog, &resumed).expect("resumed plan");
    assert_eq!(resumed_plan.trials[0].resume, ResumeDecision::SkipExact);

    resumed.prior_trials[0].artifact_policy_identity = identity("drift");
    let drifted = plan_matrix(&catalog, &resumed).expect("drifted plan");
    assert_eq!(drifted.trials[0].resume, ResumeDecision::Run);

    resumed.prior_trials[0] = PriorTrial {
        status: TrialStatus::Unknown,
        ..prior
    };
    let blocked = plan_matrix(&catalog, &resumed).expect("blocked plan");
    assert_eq!(blocked.trials[0].resume, ResumeDecision::BlockUnknown);
}

// r[verify lattice.agent_evaluation.oracles.hard_failure]
// r[verify lattice.agent_evaluation.oracles.ground_truth]
#[test]
fn objective_gates_control_status_and_advisory_judge_cannot_override_failure() {
    let catalog = builtins();
    let case = catalog.cases.iter().find(|case| case.definition.id == "skill-local").expect("case");
    let gate = &case.definition.gates[0];
    let failed = GateEvidence {
        gate_id: gate.id.clone(),
        implementation_identity: gate.implementation_identity.clone(),
        input_identity: gate.input_identity.clone(),
        evidence_schema: gate.evidence_schema.clone(),
        status: GateStatus::Failed,
        agent_controlled: false,
    };
    let rubric = case.definition.rubric.as_ref().expect("rubric");
    let judge = JudgeEvidence {
        rubric_identity: rubric.identity.clone(),
        judge_profile_identity: rubric.judge_profile_identity.clone(),
        score_basis_points: ADVISORY_SCORE,
        advisory: true,
        contains_private_content: false,
    };
    let decision = reduce_oracles(case, std::slice::from_ref(&failed), Some(&judge)).expect("oracle decision");
    assert_eq!(decision.status, TrialStatus::Failed);
    assert_eq!(decision.advisory_score_basis_points, Some(ADVISORY_SCORE));
    let agent_controlled = GateEvidence {
        status: GateStatus::Passed,
        agent_controlled: true,
        ..failed
    };
    assert_eq!(reduce_oracles(case, &[agent_controlled], Some(&judge)), Err(Error::InvalidOracleEvidence));
    assert_eq!(reduce_oracles(case, &[], Some(&judge)), Err(Error::InvalidOracleEvidence));
}

// r[verify lattice.agent_evaluation.evidence.artifact_rejected]
#[test]
fn artifact_admission_rejects_escape_secret_and_oversize_candidates() {
    let catalog = builtins();
    let case = catalog.cases.iter().find(|case| case.definition.id == "unit-local").expect("case");
    let candidate = ArtifactCandidate {
        role: ArtifactRole::Result,
        media: MediaClass::Json,
        relative_path: "results/result.json".to_string(),
        content_identity: identity("artifact"),
        byte_count: BYTE_COUNT,
        symlink: false,
        remote: false,
        secret_bearing: false,
        hidden_material_bearing: false,
        redacted: true,
    };
    assert_eq!(admit(case, std::slice::from_ref(&candidate)).expect("artifact").entries.len(), 1);

    let escaped = ArtifactCandidate {
        relative_path: "../escape".to_string(),
        ..candidate.clone()
    };
    assert_eq!(admit(case, &[escaped]), Err(Error::InvalidArtifact));
    let secret = ArtifactCandidate {
        secret_bearing: true,
        ..candidate.clone()
    };
    assert_eq!(admit(case, &[secret]), Err(Error::InvalidArtifact));
    let oversized = ArtifactCandidate {
        byte_count: case.definition.artifacts.max_bytes + 1,
        ..candidate
    };
    assert_eq!(admit(case, &[oversized]), Err(Error::InvalidArtifact));
}

// r[verify lattice.agent_evaluation.live.unknown]
#[test]
fn live_observations_preserve_unknown_and_require_cleanup_for_success() {
    let catalog = builtins();
    let case = catalog.cases.iter().find(|case| case.definition.id == "live-sandbox").expect("case");
    let fact = LiveAuthorityFact {
        case_id: case.definition.id.clone(),
        actor: "operator-reviewed".to_string(),
        target_namespace: "eval-sandbox".to_string(),
        credential_handles: vec!["secret-ref-eval-sandbox".to_string()],
        max_deployments: 1,
    };
    let admission = admit_live(case, &fact).expect("live admission");
    let unknown = Observation {
        admission_identity: admission.identity.clone(),
        class: ObservationClass::Unknown,
        deployment_identity: None,
        cleanup_confirmed: false,
    };
    assert_eq!(reduce_live_observation(&admission, &unknown), Ok(TrialStatus::Unknown));
    let incomplete = Observation {
        admission_identity: admission.identity.clone(),
        class: ObservationClass::Validated,
        deployment_identity: Some(identity("deployment")),
        cleanup_confirmed: false,
    };
    assert_eq!(reduce_live_observation(&admission, &incomplete), Ok(TrialStatus::CleanupRequired));
}

// r[verify lattice.agent_evaluation.verification.positive]
#[test]
fn catalog_cli_uses_the_production_loader_and_emits_metadata_only_json() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(BUILTIN_ROOT);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
        .args(["evaluation", "catalog", "--root"])
        .arg(&root)
        .args(["--output", "json"])
        .output()
        .expect("catalog command");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("catalog JSON");
    assert_eq!(value["schema"], lattice::agent_evaluation::model::CATALOG_SCHEMA);
    assert_eq!(value["cases"].as_array().expect("case array").len(), 4);
    let public = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(!public.contains("credential"));
    assert!(!public.contains("hidden_material"));
}

// r[verify lattice.agent_evaluation.planning.valid]
// r[verify lattice.agent_evaluation.verification.positive]
#[test]
fn plan_cli_loads_the_typed_request_and_emits_stable_trial_order() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lattice"))
        .args(["evaluation", "plan", "--root"])
        .arg(repository.join(BUILTIN_ROOT))
        .arg("--request")
        .arg(repository.join(BUILTIN_PLAN_REQUEST))
        .args(["--output", "json"])
        .output()
        .expect("plan command");

    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("plan JSON");
    let trials = value["trials"].as_array().expect("trial array");
    assert_eq!(trials.len(), 2);
    assert_eq!(trials[0]["order"], 0);
    assert_eq!(trials[1]["order"], 1);
    assert_eq!(trials[0]["case_id"], "unit-local");
    assert_ne!(trials[0]["trial_identity"], trials[1]["trial_identity"]);
}

// r[verify lattice.agent_evaluation.verification.positive]
#[test]
fn reviewed_catalog_workflow_bundle_loads_without_bundle_diagnostics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("builtins/workflow-templates");
    let catalog = lattice::workflow_template::load_catalog(&root).expect("workflow-template catalog");
    assert!(
        catalog.bundles.iter().any(|bundle| bundle.manifest.id == "agent-evaluation-catalog-review"),
        "diagnostics: {:?}",
        catalog.diagnostics
    );
    assert!(!catalog.diagnostics.iter().any(|diagnostic| diagnostic.bundle == "agent-evaluation-catalog-review"));
}
