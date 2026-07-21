//! Regenerate the checked-in bounded Animus leaf-agent workflow.

use std::fs;
use std::path::Path;

use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentTarget;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::Workflow;
use lattice::workflow_format::workflow_to_nickel;

const WORKFLOW_NAME: &str = "animus-bounded-agent";
const WORKFLOW_DESCRIPTION: &str = "Run one model call through the fixed two-turn Animus leaf runtime";
const NODE_LABEL: &str = "bounded Animus answer";
const NODE_PROMPT: &str = "Explain the Lattice and Animus authority boundary in three concise sentences.";
const TOOL_POLICY: &str = "none";
const TIMESTAMP: &str = "2026-07-20T00:00:00Z";
const OUTPUT_PATH: &str = "examples/workflows/animus-bounded-agent.ncl";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut workflow = workflow_definition()?;
    let timestamp = chrono::DateTime::parse_from_rfc3339(TIMESTAMP)?.with_timezone(&chrono::Utc);
    workflow.meta.created = timestamp;
    workflow.meta.modified = timestamp;
    let output = workflow_to_nickel(&workflow);
    fs::write(Path::new(OUTPUT_PATH), output)?;
    println!("wrote {OUTPUT_PATH}");
    Ok(())
}

fn workflow_definition() -> Result<Workflow, Box<dyn std::error::Error>> {
    let mut workflow = Workflow::new(WORKFLOW_NAME);
    workflow.meta.description = WORKFLOW_DESCRIPTION.to_string();
    let node_id = workflow.add_agent_node(NODE_LABEL, AgentTarget::Animus, NODE_PROMPT, AgentOutputMode::Text);
    let Some(NodeConfig::Agent(mut settings)) = workflow.node_config(node_id).cloned() else {
        return Err("generated Animus node is missing typed settings".into());
    };
    settings.tool_policy = TOOL_POLICY.to_string();
    workflow.set_node_config(node_id, NodeConfig::Agent(settings));
    Ok(workflow)
}
