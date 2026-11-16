//! Shared immutable graph and orchestration policy for the example.
use std::collections::BTreeSet;
use std::path::Path;

use lattice::agent_node::OutputMode;
use lattice::agent_node::Target;
use lattice::engine::NodeConfig;
use lattice::engine::Workflow;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::WorkflowTimestamp;
use lattice::orchestration::*;

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;
pub const ACTOR: &str = "operator:local";
pub const ROUND_MILLIS: u64 = 120_000;
pub const TOTAL_MILLIS: u64 = 240_000;
pub fn workflow(root: &Path) -> Result<Workflow> {
    let mut workflow = Workflow::new("bounded-build-loop", WorkflowTimestamp::epoch());
    workflow.meta.description =
        "Fixed planner/generator/evaluator round; runtime repetition belongs to the bounded host.".into();
    let planner = workflow.add_agent_node(
        "Planner",
        Target::Pi,
        "# Planner\nWrite only the fixed local task specification; no tools or authority changes.",
        OutputMode::Text,
    );
    let generator = workflow.add_agent_node(
        "Generator",
        Target::Clankers,
        "# Generator\nRead the exact specification and prior verdict; produce Rust source only.",
        OutputMode::Text,
    );
    let evaluator = workflow.add_shell_node("Evaluator: real Rust build and test", "sh evaluator.sh");
    for config in workflow.configs.values_mut() {
        match config {
            NodeConfig::Agent(settings) => {
                settings.cwd = Some(root.to_string_lossy().into_owned());
                settings.tool_policy = "none".into();
            }
            NodeConfig::Shell { cwd, timeout_secs, .. } => {
                *cwd = Some(root.to_string_lossy().into_owned());
                *timeout_secs = 60;
            }
            _ => unreachable!("fixed role profile"),
        }
    }
    connect(&mut workflow, planner, "text", generator, "prompt")?;
    connect(&mut workflow, generator, "text", evaluator, "stdin")?;
    Ok(workflow)
}

fn connect(workflow: &mut Workflow, source: NodeId, output: &str, target: NodeId, input: &str) -> Result<()> {
    let source = workflow
        .graph
        .node(source)
        .ok_or("missing source")?
        .output_ports
        .iter()
        .find(|port| port.label == output)
        .ok_or("missing output")?
        .id;
    let target = workflow
        .graph
        .node(target)
        .ok_or("missing target")?
        .input_ports
        .iter()
        .find(|port| port.label == input)
        .ok_or("missing input")?
        .id;
    workflow.graph.add_edge(source, target)?;
    Ok(())
}

pub fn definition(workflow: &Workflow) -> Result<Definition> {
    Ok(Definition {
        schema_version: current_schema_version(),
        id: DefinitionId::try_from_text("bounded-build-loop")?,
        objective: "Build and test the fixed greeting task; stop on pass or at two total rounds.".into(),
        workflow: workflow_revision(workflow)?,
        completion: CompletionPolicy::RunSucceeded,
        retry: Some(RetryPolicy {
            max_observed_failure_retries: 1,
        }),
        budgets: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                agent_invocations: true,
                duration_millis: true,
                tokens: false,
                monetary_microunits: false,
            },
            limits: BudgetLimits {
                rounds: 2,
                agent_invocations: 4,
                duration_millis: TOTAL_MILLIS,
                tokens: 0,
                monetary_microunits: 0,
            },
            per_round_upper_bound: charge(),
        },
        authority: AuthorityPolicy {
            approvers: BTreeSet::from([ACTOR.into()]),
        },
        triggers: vec![],
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    })
}

pub fn charge() -> BudgetCharge {
    BudgetCharge {
        rounds: 1,
        agent_invocations: 2,
        duration_millis: ROUND_MILLIS,
        tokens: 0,
        monetary_microunits: 0,
    }
}
