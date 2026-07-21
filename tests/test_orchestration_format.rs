use std::collections::BTreeSet;

use lattice::engine::Workflow;
use lattice::orchestration::AuthorityPolicy;
use lattice::orchestration::BudgetCharge;
use lattice::orchestration::BudgetDimensions;
use lattice::orchestration::BudgetLimits;
use lattice::orchestration::BudgetPolicy;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::OrchestrationDefinition;
use lattice::orchestration::OrchestrationTriggerBinding;
use lattice::orchestration::OrchestrationTriggerKind;
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::definition_to_nickel;
use lattice::orchestration::load_definition_nickel_file;
use lattice::orchestration::load_proposal_nickel_file;
use lattice::orchestration::make_proposal;
use lattice::orchestration::new_instance;
use lattice::orchestration::proposal_to_nickel;
use lattice::orchestration::reject_default_n8n_orchestration_export;
use lattice::orchestration::workflow_revision;
use tempfile::tempdir;

const DEFINITION_ID: &str = "format-orchestration";
const WORKFLOW_NAME: &str = "format-workflow";
const SUCCESSOR_NAME: &str = "format-successor";
const OBJECTIVE: &str = "Preserve typed orchestration meaning";
const ACTOR: &str = "operator:format";
const TRIGGER_ID: &str = "format-schedule";
const FIRST_SEQUENCE: u64 = 1;
const ROUND_LIMIT: u64 = 3;

fn definition() -> OrchestrationDefinition {
    let workflow = Workflow::new(WORKFLOW_NAME);
    OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DEFINITION_ID.to_string(),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(&workflow).expect("workflow must hash"),
        completion: CompletionPolicy::AwaitProposal,
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: ROUND_LIMIT,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge {
                rounds: FIRST_SEQUENCE,
                ..BudgetCharge::default()
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.to_string()]),
        },
        triggers: vec![OrchestrationTriggerBinding {
            trigger_id: TRIGGER_ID.to_string(),
            kind: OrchestrationTriggerKind::Schedule,
        }],
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    }
}

// r[verify lattice.agent_orchestration.format]
#[test]
fn typed_nickel_definition_round_trip_preserves_meaning() {
    let directory = tempdir().expect("temporary directory must exist");
    let path = directory.path().join("orchestration.ncl");
    let expected = definition();
    std::fs::write(&path, definition_to_nickel(&expected).expect("definition must render"))
        .expect("definition must write");

    let loaded = load_definition_nickel_file(&path).expect("rendered Nickel must load");

    assert_eq!(loaded, expected);
}

// r[verify lattice.agent_orchestration.format]
#[test]
fn proposal_nickel_round_trip_preserves_hash_bound_identity() {
    let directory = tempdir().expect("temporary directory must exist");
    let path = directory.path().join("proposal.ncl");
    let instance = new_instance(definition(), FIRST_SEQUENCE, None).expect("instance must hash");
    let successor = workflow_revision(&Workflow::new(SUCCESSOR_NAME)).expect("successor must hash");
    let expected = make_proposal(&instance, successor, "bounded successor").expect("proposal must hash");
    std::fs::write(&path, proposal_to_nickel(&expected).expect("proposal must render")).expect("proposal must write");

    let loaded = load_proposal_nickel_file(&path).expect("proposal must load");

    assert_eq!(loaded, expected);
}

// r[verify lattice.agent_orchestration.format]
#[test]
fn plaintext_credential_field_is_rejected() {
    let directory = tempdir().expect("temporary directory must exist");
    let path = directory.path().join("unsafe.ncl");
    let mut rendered = definition_to_nickel(&definition()).expect("definition must render");
    rendered = rendered.replace("objective =", "api_key = \"plaintext-secret\",\n  objective =");
    std::fs::write(&path, rendered).expect("unsafe fixture must write");

    let error = load_definition_nickel_file(&path).expect_err("unknown plaintext field must fail");

    assert!(!error.to_string().contains("plaintext-secret"));
}

// r[verify lattice.agent_orchestration.format]
#[test]
fn default_n8n_export_rejects_unmapped_orchestration_semantics() {
    let error = reject_default_n8n_orchestration_export().expect_err("unmapped orchestration must not export");

    assert!(error.to_string().contains("no exact mapping"));
}

#[test]
fn malformed_nickel_definition_is_rejected() {
    let directory = tempdir().expect("temporary directory must exist");
    let path = directory.path().join("malformed.ncl");
    std::fs::write(&path, "{ objective = }").expect("fixture must write");

    assert!(load_definition_nickel_file(&path).is_err());
}
