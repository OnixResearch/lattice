//! Tests for generic agent operation nodes.

use std::collections::HashMap;
use std::sync::Arc;

use n8nclone::agent_node::AgentAdapter;
use n8nclone::agent_node::AgentError;
use n8nclone::agent_node::AgentOutput;
use n8nclone::agent_node::AgentOutputMode;
use n8nclone::agent_node::AgentRequest;
use n8nclone::agent_node::AgentResponse;
use n8nclone::agent_node::AgentTarget;
use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::engine::executor::node_execution_cache_key;
use n8nclone::engine::value::ImageMetadata;
use n8nclone::engine::value::ImageValue;
use n8nclone::engine::value::TypeTag;
use n8nclone::engine::value::Value;
use n8nclone::engine::workflow::NodeConfig;
use n8nclone::engine::workflow::Workflow;
use n8nclone::workflow_validation::WorkflowDiagnosticCode;
use n8nclone::workflow_validation::validate_workflow;
use parking_lot::Mutex;
use pretty_assertions::assert_eq;

const AGENT_NODE_LABEL: &str = "agent";
const PROMPT_TEXT: &str = "Convert this selfie into fine art";
const PROMPT_TEXT_EDITED: &str = "Convert this selfie into ink art";
const TEXT_OUTPUT: &str = "done";
const IMAGE_MEDIA_TYPE: &str = "image/png";
const IMAGE_BYTES_A: &[u8] = b"\x89PNG\r\n\x1a\nagent-a";
const IMAGE_BYTES_B: &[u8] = b"\x89PNG\r\n\x1a\nagent-b";
const PROMPT_INPUT_INDEX: usize = 0;
const IMAGE_INPUT_INDEX: usize = 1;
const JSON_INPUT_INDEX: usize = 2;
const TEXT_OUTPUT_INDEX: usize = 0;
const IMAGE_OUTPUT_INDEX: usize = 2;
const METADATA_OUTPUT_INDEX: usize = 4;

#[derive(Clone)]
struct StaticAgentAdapter {
    response: AgentResponse,
    requests: Arc<Mutex<Vec<AgentRequest>>>,
}

impl StaticAgentAdapter {
    fn new(response: AgentResponse) -> Self {
        Self {
            response,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn requests(&self) -> Vec<AgentRequest> {
        self.requests.lock().clone()
    }
}

impl AgentAdapter for StaticAgentAdapter {
    fn execute(&self, request: &AgentRequest) -> Result<AgentResponse, AgentError> {
        self.requests.lock().push(request.clone());
        Ok(self.response.clone())
    }
}

fn image_value(bytes: &[u8]) -> Value {
    Value::from_image(ImageValue::new(bytes.to_vec(), IMAGE_MEDIA_TYPE.to_string(), ImageMetadata::default()))
}

// r[verify n8nclone.workflow_execution.agent_nodes.config]
#[test]
fn add_agent_node_records_config_and_ports() {
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Image);
    let node = workflow.graph.node(node_id).expect("agent node exists");

    assert_eq!(node.input_ports.len(), JSON_INPUT_INDEX + 1);
    assert_eq!(node.output_ports.len(), METADATA_OUTPUT_INDEX + 1);
    assert_eq!(node.input_ports[IMAGE_INPUT_INDEX].type_tag, TypeTag::Image.as_str());
    assert_eq!(node.output_ports[IMAGE_OUTPUT_INDEX].type_tag, TypeTag::Image.as_str());
    match workflow.node_config(node_id).expect("agent config exists") {
        NodeConfig::Agent(settings) => {
            assert_eq!(settings.target, AgentTarget::Pi);
            assert_eq!(settings.prompt, PROMPT_TEXT);
            assert_eq!(settings.output_mode, AgentOutputMode::Image);
        }
        other => panic!("expected agent config, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.pi_adapter]
// r[verify n8nclone.workflow_execution.agent_nodes.execute]
#[tokio::test]
async fn pi_agent_node_executes_text_request_through_adapter() {
    let adapter = StaticAgentAdapter::new(AgentResponse {
        output: AgentOutput::Text(TEXT_OUTPUT.to_string()),
        metadata: serde_json::json!({"adapter": "pi"}),
    });
    let observed_adapter = adapter.clone();
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Text);
    let node = workflow.graph.node(node_id).expect("agent node exists");
    let prompt_input = node.input_ports[PROMPT_INPUT_INDEX].id;
    let inputs = [(prompt_input, Value::from_string("extra prompt".to_string()))].into();
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &inputs).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let text = outputs.get(&node.output_ports[TEXT_OUTPUT_INDEX].id).expect("text output");
            let metadata = outputs.get(&node.output_ports[METADATA_OUTPUT_INDEX].id).expect("metadata output");
            assert_eq!(text.as_display_string(), TEXT_OUTPUT);
            assert!(metadata.as_display_string().contains("pi"));
        }
        other => panic!("expected completed agent node, got {other:?}"),
    }
    let requests = observed_adapter.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].target, AgentTarget::Pi);
    assert_eq!(requests[0].prompt, PROMPT_TEXT);
    assert_eq!(requests[0].text_inputs, vec!["extra prompt".to_string()]);
}

// r[verify n8nclone.workflow_execution.agent_nodes.clankers_adapter]
// r[verify n8nclone.workflow_execution.agent_nodes.execute]
#[tokio::test]
async fn clankers_agent_node_executes_image_request_through_adapter() {
    let generated_image =
        ImageValue::new(IMAGE_BYTES_B.to_vec(), IMAGE_MEDIA_TYPE.to_string(), ImageMetadata::default());
    let adapter = StaticAgentAdapter::new(AgentResponse {
        output: AgentOutput::Image(generated_image),
        metadata: serde_json::json!({"adapter": "clankers"}),
    });
    let observed_adapter = adapter.clone();
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Clankers, PROMPT_TEXT, AgentOutputMode::Image);
    let node = workflow.graph.node(node_id).expect("agent node exists");
    let image_input = node.input_ports[IMAGE_INPUT_INDEX].id;
    let inputs = [(image_input, image_value(IMAGE_BYTES_A))].into();
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &inputs).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let image = outputs.get(&node.output_ports[IMAGE_OUTPUT_INDEX].id).expect("image output");
            assert_eq!(image.type_tag(), TypeTag::Image);
            assert!(image.as_display_string().contains(IMAGE_MEDIA_TYPE));
        }
        other => panic!("expected completed agent node, got {other:?}"),
    }
    let requests = observed_adapter.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].target, AgentTarget::Clankers);
    assert_eq!(requests[0].image_inputs.len(), 1);
}

// r[verify n8nclone.workflow_execution.agent_nodes.execute]
#[tokio::test]
async fn output_mode_mismatch_marks_node_failed() {
    let adapter = StaticAgentAdapter::new(AgentResponse {
        output: AgentOutput::Text(TEXT_OUTPUT.to_string()),
        metadata: serde_json::json!({}),
    });
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Image);
    let executor = Executor::new_with_agent_adapter(Box::new(adapter));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("output type mismatch"), "error: {error}"),
        other => panic!("expected failed agent node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.agent_nodes.cache_key]
#[test]
fn agent_cache_key_changes_for_prompt_target_and_image_edits() {
    let mut workflow = Workflow::new("agent-workflow");
    let node_id = workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, PROMPT_TEXT, AgentOutputMode::Image);
    let node = workflow.graph.node(node_id).expect("agent node exists");
    let image_input = node.input_ports[IMAGE_INPUT_INDEX].id;
    let inputs_a = [(image_input, image_value(IMAGE_BYTES_A))].into();
    let inputs_b = [(image_input, image_value(IMAGE_BYTES_B))].into();
    let original_config = workflow.node_config(node_id).expect("agent config");
    let original_key = node_execution_cache_key(node_id, original_config, &inputs_a);

    let prompt_config = NodeConfig::agent(AgentTarget::Pi, PROMPT_TEXT_EDITED, AgentOutputMode::Image);
    let target_config = NodeConfig::agent(AgentTarget::Clankers, PROMPT_TEXT, AgentOutputMode::Image);

    assert_ne!(original_key, node_execution_cache_key(node_id, &prompt_config, &inputs_a));
    assert_ne!(original_key, node_execution_cache_key(node_id, &target_config, &inputs_a));
    assert_ne!(original_key, node_execution_cache_key(node_id, original_config, &inputs_b));
}

// r[verify n8nclone.workflow_execution.agent_nodes.config]
#[test]
fn empty_agent_prompt_is_validation_diagnostic() {
    let mut workflow = Workflow::new("agent-workflow");
    workflow.add_agent_node(AGENT_NODE_LABEL, AgentTarget::Pi, "", AgentOutputMode::Text);

    let diagnostics = validate_workflow(&workflow);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::EmptyAgentPrompt));
}
