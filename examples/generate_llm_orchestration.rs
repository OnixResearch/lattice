//! Regenerate the checked-in durable LLM orchestration definition.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

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
use lattice::orchestration::RetryPolicy;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::definition_to_nickel;
use lattice::orchestration::workflow_revision;
use lattice::workflow_format::load_workflow_nickel_file;

const DEFINITION_ID: &str = "llm-review-loop";
const OBJECTIVE: &str = "Iteratively research, build, review, and publish an operator-approved report.";
const WORKFLOW_PATH: &str = "examples/workflows/llm-operating-graph.ncl";
const OUTPUT_PATH: &str = "examples/orchestrations/llm-review-loop.ncl";
const OPERATOR_AUTHORITY: &str = "operator:local";
const SCHEDULE_TRIGGER_ID: &str = "llm-review-schedule";
const WEBHOOK_TRIGGER_ID: &str = "llm-review-webhook";
const MAX_ROUNDS: u64 = 3;
const AGENT_INVOCATIONS_PER_ROUND: u64 = 4;
const MAX_AGENT_INVOCATIONS: u64 = MAX_ROUNDS * AGENT_INVOCATIONS_PER_ROUND;
const TOKENS_PER_ROUND: u64 = 8_192;
const MAX_TOKENS: u64 = MAX_ROUNDS * TOKENS_PER_ROUND;
const DURATION_PER_ROUND_MILLIS: u64 = 600_000;
const MAX_DURATION_MILLIS: u64 = MAX_ROUNDS * DURATION_PER_ROUND_MILLIS;
const ROUND_CHARGE: u64 = 1;
const MAX_OBSERVED_FAILURE_RETRIES: u64 = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = load_workflow_nickel_file(Path::new(WORKFLOW_PATH))?;
    let definition = OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: lattice::orchestration::DefinitionId::try_from_text(DEFINITION_ID)
            .expect("example definition identity must be valid"),
        objective: OBJECTIVE.to_string(),
        workflow: workflow_revision(&workflow)?,
        completion: CompletionPolicy::AwaitProposal,
        retry: Some(RetryPolicy {
            max_observed_failure_retries: MAX_OBSERVED_FAILURE_RETRIES,
        }),
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                agent_invocations: true,
                tokens: true,
                monetary_microunits: false,
                duration_millis: true,
            },
            limits: BudgetLimits {
                rounds: MAX_ROUNDS,
                agent_invocations: MAX_AGENT_INVOCATIONS,
                tokens: MAX_TOKENS,
                monetary_microunits: 0,
                duration_millis: MAX_DURATION_MILLIS,
            },
            per_round_upper_bound: BudgetCharge {
                rounds: ROUND_CHARGE,
                agent_invocations: AGENT_INVOCATIONS_PER_ROUND,
                tokens: TOKENS_PER_ROUND,
                monetary_microunits: 0,
                duration_millis: DURATION_PER_ROUND_MILLIS,
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([OPERATOR_AUTHORITY.to_string()]),
        },
        triggers: vec![
            OrchestrationTriggerBinding {
                trigger_id: lattice::orchestration::TriggerId::try_from_text(SCHEDULE_TRIGGER_ID)
                    .expect("example schedule trigger identity must be valid"),
                kind: OrchestrationTriggerKind::Schedule,
            },
            OrchestrationTriggerBinding {
                trigger_id: lattice::orchestration::TriggerId::try_from_text(WEBHOOK_TRIGGER_ID)
                    .expect("example webhook trigger identity must be valid"),
                kind: OrchestrationTriggerKind::Webhook,
            },
        ],
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    let output = definition_to_nickel(&definition)?;
    if let Some(parent) = Path::new(OUTPUT_PATH).parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(OUTPUT_PATH, output)?;
    println!("wrote {OUTPUT_PATH}");
    Ok(())
}
