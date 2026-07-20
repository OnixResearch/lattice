//! Regenerate the checked-in Markdown-backed LLM operating-graph fixture.

use std::fs;
use std::io;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::Workflow;
use lattice::file_node::FileOperationKind;
use lattice::workflow_format::workflow_to_nickel;
use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortId;

const WORKFLOW_NAME: &str = "llm-operating-graph";
const WORKFLOW_DESCRIPTION: &str = "Markdown-backed role graph with explicit shared-state reads, fan-out, aggregation, publishing, and output effects.";
const FIXTURE_TIMESTAMP_RFC3339: &str = "2026-07-18T00:00:00Z";
const WORKFLOW_OUTPUT_PATH: &str = "examples/workflows/llm-operating-graph.ncl";
const SHARED_STATE_ROOT: &str = "examples/workflows/state";
const BRIEF_PATH: &str = "examples/workflows/state/llm-operating-graph-brief.md";
const REPORT_PATH: &str = "examples/workflows/state/llm-operating-graph-report.md";
const RESEARCHER_PROMPT_PATH: &str = "roles/llm-operating-graph/researcher.md";
const BUILDER_PROMPT_PATH: &str = "roles/llm-operating-graph/builder.md";
const REVIEWER_PROMPT_PATH: &str = "roles/llm-operating-graph/reviewer.md";
const PUBLISHER_PROMPT_PATH: &str = "roles/llm-operating-graph/publisher.md";
const PROMPT_PLACEHOLDER: &str = "Loaded from the referenced Markdown role file.";
const FILE_CONTENT_PORT: &str = "content";
const FILE_METADATA_PORT: &str = "metadata";
const AGENT_PROMPT_PORT: &str = "prompt";
const AGENT_JSON_PORT: &str = "json";
const AGENT_TEXT_PORT: &str = "text";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow = build_fixture()?;
    fs::write(WORKFLOW_OUTPUT_PATH, workflow_to_nickel(&workflow))?;
    println!("wrote {WORKFLOW_OUTPUT_PATH}");
    Ok(())
}

fn build_fixture() -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    let timestamp = chrono::DateTime::parse_from_rfc3339(FIXTURE_TIMESTAMP_RFC3339)?.with_timezone(&chrono::Utc);
    workflow.meta.description = WORKFLOW_DESCRIPTION.to_string();
    workflow.meta.created = timestamp;
    workflow.meta.modified = timestamp;

    let read_brief = workflow.add_file_node("Read shared brief", FileOperationKind::Read, BRIEF_PATH);
    configure_file_effect(&mut workflow, read_brief, false)?;

    let researcher =
        add_role(&mut workflow, "Researcher", AgentTarget::Pi, AgentOutputMode::Text, RESEARCHER_PROMPT_PATH)?;
    let builder =
        add_role(&mut workflow, "Builder", AgentTarget::Clankers, AgentOutputMode::Json, BUILDER_PROMPT_PATH)?;
    let reviewer = add_role(&mut workflow, "Reviewer", AgentTarget::Pi, AgentOutputMode::Text, REVIEWER_PROMPT_PATH)?;
    let publisher =
        add_role(&mut workflow, "Publisher", AgentTarget::Pi, AgentOutputMode::Text, PUBLISHER_PROMPT_PATH)?;

    let write_report = workflow.add_file_node("Write report artifact", FileOperationKind::Write, REPORT_PATH);
    configure_file_effect(&mut workflow, write_report, true)?;

    connect(&mut workflow, read_brief, FILE_CONTENT_PORT, researcher, AGENT_PROMPT_PORT)?;
    connect(&mut workflow, read_brief, FILE_METADATA_PORT, builder, AGENT_JSON_PORT)?;
    connect(&mut workflow, researcher, AGENT_TEXT_PORT, reviewer, AGENT_PROMPT_PORT)?;
    connect(&mut workflow, builder, AGENT_JSON_PORT, reviewer, AGENT_JSON_PORT)?;
    connect(&mut workflow, reviewer, AGENT_TEXT_PORT, publisher, AGENT_PROMPT_PORT)?;
    connect(&mut workflow, publisher, AGENT_TEXT_PORT, write_report, FILE_CONTENT_PORT)?;

    Ok(workflow)
}

fn add_role(
    workflow: &mut Workflow,
    label: &str,
    target: AgentTarget,
    output_mode: AgentOutputMode,
    prompt_file: &str,
) -> Result<NodeId, Box<dyn std::error::Error>> {
    let node_id = workflow.add_agent_node(label, target, PROMPT_PLACEHOLDER, output_mode);
    let Some(NodeConfig::Agent(mut settings)) = workflow.node_config(node_id).cloned() else {
        return Err(io::Error::other(format!("agent config missing for role '{label}'")).into());
    };
    settings.prompt_file = Some(prompt_file.to_string());
    workflow.set_node_config(node_id, NodeConfig::Agent(settings));
    Ok(node_id)
}

fn configure_file_effect(
    workflow: &mut Workflow,
    node_id: NodeId,
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(NodeConfig::File {
        operation,
        path,
        content,
        allowed_roots: _,
        overwrite: _,
    }) = workflow.node_config(node_id).cloned()
    else {
        return Err(io::Error::other("file config missing for operating-graph effect").into());
    };
    workflow.set_node_config(node_id, NodeConfig::File {
        operation,
        path,
        content,
        overwrite,
        allowed_roots: vec![SHARED_STATE_ROOT.to_string()],
    });
    Ok(())
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
