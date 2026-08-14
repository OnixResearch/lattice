//! Regenerate the checked-in guarded one-agent workflow-template assets.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::control_flow::ConditionBranch;
use lattice::control_flow::ConditionNodeConfig;
use lattice::control_flow::ConditionPredicate;
use lattice::control_flow::MergeNodeConfig;
use lattice::control_flow::MergePolicy;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::NodeId;
use lattice::engine::workflow::PortId;
use lattice::engine::workflow::Workflow;
use lattice::orchestration::AuthorityPolicy;
use lattice::orchestration::BudgetCharge;
use lattice::orchestration::BudgetDimensions;
use lattice::orchestration::BudgetLimits;
use lattice::orchestration::BudgetPolicy;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::OrchestrationDefinition;
use lattice::orchestration::RedactionPolicy;

const FIXTURE_TIMESTAMP_RFC3339: &str = "2026-08-14T00:00:00Z";
const CAIRN_BUNDLE_ROOT: &str = "builtins/workflow-templates/cairn-agent-drain";
const OCTET_BUNDLE_ROOT: &str = "builtins/workflow-templates/octet-agent-burndown";
const CAIRN_PROMPT_PATH: &str = "prompts/cairn-agent-drain.md";
const OCTET_PROMPT_PATH: &str = "prompts/octet-agent-burndown.md";
const CAIRN_WORKFLOW_NAME: &str = "cairn-agent-drain";
const OCTET_WORKFLOW_NAME: &str = "octet-agent-burndown";
const FIXED_ROOT: &str = ".";
const SHELL_STDIN_PORT: &str = "stdin";
const SHELL_STDOUT_PORT: &str = "stdout";
const SHELL_EXIT_PORT: &str = "exit";
const CONDITION_INPUT_PORT: &str = "value";
const CONDITION_WORK_PORT: &str = "work";
const AGENT_PROMPT_PORT: &str = "prompt";
const AGENT_TEXT_PORT: &str = "text";
const MERGE_OUTPUT_PORT: &str = "merged";
const MERGE_BEFORE_PORT: &str = "before";
const MERGE_VALIDATION_PORT: &str = "validation";
const MERGE_AFTER_PORT: &str = "after";
const CAIRN_DISCOVERY_COMMAND: &str = "lattice guarded cairn-discover --root .";
const CAIRN_VALIDATION_COMMAND: &str = "cairn validate --root . --strict";
const CAIRN_DECISION_COMMAND: &str = "lattice guarded decide --kind cairn --minimum-delta 1";
const OCTET_BEFORE_COMMAND: &str =
    "lattice guarded octet-probe --root . --artifact-dir target/lattice-guarded/octet-before --timeout-secs 600";
const OCTET_VALIDATION_COMMAND: &str = "scripts/check.rs";
const OCTET_AFTER_COMMAND: &str =
    "lattice guarded octet-probe --root . --artifact-dir target/lattice-guarded/octet-after --timeout-secs 600";
const OCTET_DECISION_COMMAND: &str = "lattice guarded decide --kind octet --minimum-delta 1";
const SHELL_TIMEOUT_SECS: u64 = 900;
const MAX_ROUNDS: u64 = 8;
const AGENT_INVOCATIONS_PER_ROUND: u64 = 1;
const MAX_AGENT_INVOCATIONS: u64 = MAX_ROUNDS * AGENT_INVOCATIONS_PER_ROUND;
const TOKENS_PER_ROUND: u64 = 32_768;
const MAX_TOKENS: u64 = MAX_ROUNDS * TOKENS_PER_ROUND;
const DURATION_PER_ROUND_MILLIS: u64 = 900_000;
const MAX_DURATION_MILLIS: u64 = MAX_ROUNDS * DURATION_PER_ROUND_MILLIS;
const ROUND_CHARGE: u64 = 1;
const HORIZONTAL_STEP: i32 = 220;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cairn = build_cairn_workflow()?;
    let octet = build_octet_workflow()?;
    write_bundle_assets(CAIRN_BUNDLE_ROOT, &cairn, "Complete one selected active Cairn change per approved round.")?;
    write_bundle_assets(
        OCTET_BUNDLE_ROOT,
        &octet,
        "Reduce no-disabled Octet findings through one approved candidate per round.",
    )?;
    Ok(())
}

fn build_cairn_workflow() -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut workflow = base_workflow(
        CAIRN_WORKFLOW_NAME,
        "Discover one active Cairn change, invoke one Pi agent, validate, remeasure, and gate strict progress.",
    )?;
    let discover = add_shell(&mut workflow, "Discover active changes", CAIRN_DISCOVERY_COMMAND)?;
    let has_work = workflow.add_condition_node("Select one sorted change", ConditionNodeConfig {
        branches: vec![ConditionBranch {
            label: CONDITION_WORK_PORT.to_owned(),
            predicate: ConditionPredicate::StringContains("\"has_work\":true".to_owned()),
        }],
    });
    let agent = add_agent(&mut workflow, "Drain selected change", CAIRN_BUNDLE_ROOT, CAIRN_PROMPT_PATH)?;
    let validate = add_shell(&mut workflow, "Validate repository", CAIRN_VALIDATION_COMMAND)?;
    let measure = add_shell(&mut workflow, "Measure active changes", CAIRN_DISCOVERY_COMMAND)?;
    let merge = add_progress_merge(&mut workflow);
    let decide = add_shell(&mut workflow, "Gate strict progress", CAIRN_DECISION_COMMAND)?;

    connect(&mut workflow, discover, SHELL_STDOUT_PORT, has_work, CONDITION_INPUT_PORT)?;
    connect(&mut workflow, has_work, CONDITION_WORK_PORT, agent, AGENT_PROMPT_PORT)?;
    connect(&mut workflow, agent, AGENT_TEXT_PORT, validate, SHELL_STDIN_PORT)?;
    connect(&mut workflow, validate, SHELL_STDOUT_PORT, measure, SHELL_STDIN_PORT)?;
    connect(&mut workflow, discover, SHELL_STDOUT_PORT, merge, MERGE_BEFORE_PORT)?;
    connect(&mut workflow, validate, SHELL_EXIT_PORT, merge, MERGE_VALIDATION_PORT)?;
    connect(&mut workflow, measure, SHELL_STDOUT_PORT, merge, MERGE_AFTER_PORT)?;
    connect(&mut workflow, merge, MERGE_OUTPUT_PORT, decide, SHELL_STDIN_PORT)?;
    position_nodes(&mut workflow);
    Ok(workflow)
}

fn build_octet_workflow() -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut workflow = base_workflow(
        OCTET_WORKFLOW_NAME,
        "Probe no-disabled Octet findings, invoke one Pi agent, validate, restore exact policy bytes, and gate strict progress.",
    )?;
    let baseline = add_shell(&mut workflow, "Probe baseline findings", OCTET_BEFORE_COMMAND)?;
    let agent = add_agent(&mut workflow, "Refactor one finding", OCTET_BUNDLE_ROOT, OCTET_PROMPT_PATH)?;
    let validate = add_shell(&mut workflow, "Validate repository", OCTET_VALIDATION_COMMAND)?;
    let measure = add_shell(&mut workflow, "Probe candidate findings", OCTET_AFTER_COMMAND)?;
    let merge = add_progress_merge(&mut workflow);
    let decide = add_shell(&mut workflow, "Gate strict progress", OCTET_DECISION_COMMAND)?;

    connect(&mut workflow, baseline, SHELL_STDOUT_PORT, agent, AGENT_PROMPT_PORT)?;
    connect(&mut workflow, agent, AGENT_TEXT_PORT, validate, SHELL_STDIN_PORT)?;
    connect(&mut workflow, validate, SHELL_STDOUT_PORT, measure, SHELL_STDIN_PORT)?;
    connect(&mut workflow, baseline, SHELL_STDOUT_PORT, merge, MERGE_BEFORE_PORT)?;
    connect(&mut workflow, validate, SHELL_EXIT_PORT, merge, MERGE_VALIDATION_PORT)?;
    connect(&mut workflow, measure, SHELL_STDOUT_PORT, merge, MERGE_AFTER_PORT)?;
    connect(&mut workflow, merge, MERGE_OUTPUT_PORT, decide, SHELL_STDIN_PORT)?;
    position_nodes(&mut workflow);
    Ok(workflow)
}

fn base_workflow(name: &str, description: &str) -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(name, lattice::engine::workflow::WorkflowTimestamp::epoch());
    let timestamp = chrono::DateTime::parse_from_rfc3339(FIXTURE_TIMESTAMP_RFC3339)?.with_timezone(&chrono::Utc);
    workflow.meta.description = description.to_owned();
    workflow.meta.created = timestamp;
    workflow.meta.modified = timestamp;
    Ok(workflow)
}

fn add_shell(workflow: &mut Workflow, label: &str, command: &str) -> Result<NodeId, io::Error> {
    let node = workflow.add_shell_node(label, command);
    let Some(NodeConfig::Shell {
        command,
        cwd: _,
        env,
        timeout_secs: _,
        pipe_stdin,
    }) = workflow.node_config(node).cloned()
    else {
        return Err(io::Error::other("generated shell node is missing its config"));
    };
    workflow.set_node_config(node, NodeConfig::Shell {
        command,
        cwd: Some(FIXED_ROOT.to_owned()),
        env,
        timeout_secs: SHELL_TIMEOUT_SECS,
        pipe_stdin,
    });
    Ok(node)
}

fn add_agent(
    workflow: &mut Workflow,
    label: &str,
    bundle_root: &str,
    prompt_path: &str,
) -> Result<NodeId, Box<dyn std::error::Error>> {
    let full_prompt_path = Path::new(bundle_root).join(prompt_path);
    let prompt = fs::read_to_string(full_prompt_path)?;
    let node = workflow.add_agent_node(label, AgentTarget::Pi, prompt, AgentOutputMode::Text);
    let Some(NodeConfig::Agent(mut settings)) = workflow.node_config(node).cloned() else {
        return Err(io::Error::other("generated agent node is missing its config").into());
    };
    settings.prompt_file = Some(prompt_path.to_owned());
    settings.tool_policy = lattice::guarded::authority::GUARDED_TOOL_POLICY.to_owned();
    settings.cwd = Some(FIXED_ROOT.to_owned());
    workflow.set_node_config(node, NodeConfig::Agent(settings));
    Ok(node)
}

fn add_progress_merge(workflow: &mut Workflow) -> NodeId {
    workflow.add_merge_node("Collect progress facts", MergeNodeConfig {
        policy: MergePolicy::Collect,
        required_inputs: vec![
            MERGE_BEFORE_PORT.to_owned(),
            MERGE_VALIDATION_PORT.to_owned(),
            MERGE_AFTER_PORT.to_owned(),
        ],
    })
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

fn position_nodes(workflow: &mut Workflow) {
    for (index, node_id) in workflow.graph.node_ids().into_iter().enumerate() {
        let Some(node) = workflow.graph.node_mut(node_id) else {
            continue;
        };
        let horizontal_index = i32::try_from(index).expect("guarded fixture node count fits in i32");
        node.x = horizontal_index.saturating_mul(HORIZONTAL_STEP);
    }
}

fn write_bundle_assets(
    bundle_root: &str,
    workflow: &Workflow,
    objective: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(bundle_root);
    fs::create_dir_all(root)?;
    fs::write(root.join("workflow.ncl"), lattice::workflow_format::workflow_to_nickel(workflow))?;
    let definition = OrchestrationDefinition {
        schema_version: lattice::orchestration::current_schema_version(),
        id: lattice::orchestration::DefinitionId::try_from_text(workflow.meta.name.clone())?,
        objective: objective.to_owned(),
        workflow: lattice::orchestration::workflow_revision(workflow)?,
        completion: CompletionPolicy::AwaitProposal,
        retry: None,
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
            approvers: BTreeSet::from(["operator:local".to_owned()]),
        },
        triggers: Vec::new(),
        redaction: RedactionPolicy {
            retain_safe_summaries: true,
        },
    };
    fs::write(root.join("orchestration.ncl"), lattice::orchestration::definition_to_nickel(&definition)?)?;
    println!("wrote guarded assets under {bundle_root}");
    Ok(())
}
