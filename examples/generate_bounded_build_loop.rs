//! Regenerate the bounded planner/generator/evaluator build-loop fixture.

use std::collections::BTreeSet;
use std::fs;
use std::io;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::Workflow;
use lattice::engine::workflow::WorkflowTimestamp;
use lattice::orchestration::AuthorityPolicy;
use lattice::orchestration::BudgetCharge;
use lattice::orchestration::BudgetDimensions;
use lattice::orchestration::BudgetLimits;
use lattice::orchestration::BudgetPolicy;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::DefinitionId;
use lattice::orchestration::OrchestrationDefinition;
use lattice::orchestration::OrchestrationTriggerBinding;
use lattice::orchestration::OrchestrationTriggerKind;
use lattice::orchestration::RedactionPolicy;
use lattice::orchestration::RetryPolicy;
use lattice::orchestration::TriggerId;
use lattice::orchestration::current_schema_version;
use lattice::orchestration::definition_to_nickel;
use lattice::orchestration::workflow_revision;
use lattice::workflow_format::workflow_to_nickel;

const WORKFLOW_NAME: &str = "bounded-build-loop";
const WORKFLOW_DESCRIPTION: &str =
    "Plan, generate, and compile a minimal Rust program through two bounded agent roles and a real build check.";
const WORKFLOW_OUTPUT_PATH: &str = "examples/workflows/bounded-build-loop.ncl";
const ORCHESTRATION_OUTPUT_PATH: &str = "examples/orchestrations/bounded-build-loop.ncl";
const DEFINITION_ID: &str = "bounded-build-loop";
const OBJECTIVE: &str =
    "Generate and compile a minimal Rust program, retrying at most once when the build check fails.";
const OPERATOR_AUTHORITY: &str = "operator:local";
const SCHEDULE_TRIGGER_ID: &str = "bounded-build-loop-schedule";
const MAX_ROUNDS: u64 = 2;
const AGENT_INVOCATIONS_PER_ROUND: u64 = 2;
const MAX_AGENT_INVOCATIONS: u64 = MAX_ROUNDS * AGENT_INVOCATIONS_PER_ROUND;
const TOKENS_PER_ROUND: u64 = 4_096;
const MAX_TOKENS: u64 = MAX_ROUNDS * TOKENS_PER_ROUND;
const DURATION_PER_ROUND_MILLIS: u64 = 120_000;
const MAX_DURATION_MILLIS: u64 = MAX_ROUNDS * DURATION_PER_ROUND_MILLIS;
const MAX_OBSERVED_FAILURE_RETRIES: u64 = 1;
const PLANNER_PROMPT: &str = "# Planner\nReturn a short implementation plan for a Rust program that prints one line.";
const GENERATOR_PROMPT: &str = "# Generator\nReturn only Rust source for the planned program.";
const EVALUATOR_COMMAND: &str = "sh -c 'out=/tmp/lattice-bounded-build-loop-$$; rustc --edition 2024 -o \"$out\" - && \"$out\"; status=$?; rm -f \"$out\"; exit $status'";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = build_workflow()?;
    fs::write(WORKFLOW_OUTPUT_PATH, workflow_to_nickel(&workflow))?;
    println!("wrote {WORKFLOW_OUTPUT_PATH}");

    let definition = OrchestrationDefinition {
        schema_version: current_schema_version(),
        id: DefinitionId::try_from_text(DEFINITION_ID)?,
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
                rounds: 1,
                agent_invocations: AGENT_INVOCATIONS_PER_ROUND,
                tokens: TOKENS_PER_ROUND,
                monetary_microunits: 0,
                duration_millis: DURATION_PER_ROUND_MILLIS,
            },
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([OPERATOR_AUTHORITY.to_string()]),
        },
        triggers: vec![OrchestrationTriggerBinding {
            trigger_id: TriggerId::try_from_text(SCHEDULE_TRIGGER_ID)?,
            kind: OrchestrationTriggerKind::Schedule,
        }],
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    fs::write(ORCHESTRATION_OUTPUT_PATH, definition_to_nickel(&definition)?)?;
    println!("wrote {ORCHESTRATION_OUTPUT_PATH}");
    Ok(())
}

fn build_workflow() -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(WORKFLOW_NAME, WorkflowTimestamp::epoch());
    let timestamp = chrono::DateTime::parse_from_rfc3339("2026-11-11T00:00:00Z")?.with_timezone(&chrono::Utc);
    workflow.meta.description = WORKFLOW_DESCRIPTION.to_string();
    workflow.meta.created = timestamp;
    workflow.meta.modified = timestamp;

    let planner = workflow.add_agent_node("Planner", AgentTarget::Pi, PLANNER_PROMPT, AgentOutputMode::Text);
    let generator =
        workflow.add_agent_node("Generator", AgentTarget::Clankers, GENERATOR_PROMPT, AgentOutputMode::Text);
    let evaluator = workflow.add_shell_node("Build and run generated program", EVALUATOR_COMMAND);

    connect(&mut workflow, planner, "text", generator, "prompt")?;
    connect(&mut workflow, generator, "text", evaluator, "stdin")?;
    Ok(workflow)
}

fn connect(
    workflow: &mut Workflow,
    source_node: NodeId,
    source_label: &str,
    target_node: NodeId,
    target_label: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = output_port(workflow, source_node, source_label)?;
    let target = input_port(workflow, target_node, target_label)?;
    workflow.graph.add_edge(source, target)?;
    Ok(())
}

fn output_port(workflow: &Workflow, node_id: NodeId, label: &str) -> Result<PortId, io::Error> {
    workflow
        .graph
        .node(node_id)
        .and_then(|node| node.output_ports.iter().find(|port| port.label == label))
        .map(|port| port.id)
        .ok_or_else(|| io::Error::other(format!("output port '{label}' is missing")))
}

fn input_port(workflow: &Workflow, node_id: NodeId, label: &str) -> Result<PortId, io::Error> {
    workflow
        .graph
        .node(node_id)
        .and_then(|node| node.input_ports.iter().find(|port| port.label == label))
        .map(|port| port.id)
        .ok_or_else(|| io::Error::other(format!("input port '{label}' is missing")))
}
