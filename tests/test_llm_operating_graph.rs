//! End-to-end checks for the Markdown-backed LLM operating-graph fixture.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use n8nclone::agent_node::AgentAdapter;
use n8nclone::agent_node::AgentError;
use n8nclone::agent_node::AgentOutput;
use n8nclone::agent_node::AgentOutputMode;
use n8nclone::agent_node::AgentRequest;
use n8nclone::agent_node::AgentResponse;
use n8nclone::agent_node::ExecutionContext as AgentExecutionContext;
use n8nclone::credentials::EmptyCredentialProvider;
use n8nclone::engine::executor::Executor;
use n8nclone::engine::executor::NodeStatus;
use n8nclone::file_node::MemoryFileSystem;
use n8nclone::http::NoopHttpClient;
use n8nclone::image_node::NoopImageIo;
use n8nclone::workflow_format::load_workflow_nickel_file;
use pretty_assertions::assert_eq;
use serde_json::json;

const FIXTURE_PATH: &str = "examples/workflows/llm-operating-graph.ncl";
const BRIEF_PATH: &str = "examples/workflows/state/llm-operating-graph-brief.md";
const REPORT_PATH: &str = "examples/workflows/state/llm-operating-graph-report.md";
const BRIEF_TEXT: &str = "Evidence-backed operating graph brief";
const RESEARCHER_ROLE: &str = "Researcher";
const BUILDER_ROLE: &str = "Builder";
const REVIEWER_ROLE: &str = "Reviewer";
const PUBLISHER_ROLE: &str = "Publisher";
const REPORT_EFFECT_LABEL: &str = "Write report artifact";
const RESEARCH_RESULT: &str = "primary evidence with explicit uncertainty";
const REVIEW_RESULT: &str = "reviewed recommendation with acceptance checks";
const PUBLISHED_REPORT: &str = "operator report preserving caveats and next actions";
const OPERATING_GRAPH_NODE_COUNT: usize = 6;
const OPERATING_GRAPH_ROLE_COUNT: usize = 4;
const FAILURE_ADAPTER_CALL_COUNT: usize = 2;
const BLOCKED_DOWNSTREAM_COUNT: usize = 3;
const RESEARCHER_REQUEST_INDEX: usize = 0;
const BUILDER_REQUEST_INDEX: usize = 1;
const REVIEWER_REQUEST_INDEX: usize = 2;
const PUBLISHER_REQUEST_INDEX: usize = 3;

#[derive(Debug, Clone)]
struct RecordingAgentAdapter {
    requests: Arc<Mutex<Vec<AgentRequest>>>,
    failing_role: Option<&'static str>,
}

impl RecordingAgentAdapter {
    fn succeeds(requests: Arc<Mutex<Vec<AgentRequest>>>) -> Self {
        Self {
            requests,
            failing_role: None,
        }
    }

    fn fails_for(requests: Arc<Mutex<Vec<AgentRequest>>>, role: &'static str) -> Self {
        Self {
            requests,
            failing_role: Some(role),
        }
    }
}

impl AgentAdapter for RecordingAgentAdapter {
    fn execute(&self, request: &AgentRequest, _context: &AgentExecutionContext) -> Result<AgentResponse, AgentError> {
        self.requests.lock().expect("request log lock succeeds").push(request.clone());
        let role = request_role(request);
        if self.failing_role == Some(role.as_str()) {
            return Err(AgentError::PreflightRejected(format!("{role} is unavailable")));
        }

        let output = match request.output_mode {
            AgentOutputMode::Text => AgentOutput::Text(text_output_for_role(&role)?),
            AgentOutputMode::Json => AgentOutput::Json(builder_output()),
            AgentOutputMode::Image | AgentOutputMode::Artifact => {
                return Err(AgentError::Adapter(format!("unexpected fixture output mode for {role}")));
            }
        };
        Ok(AgentResponse {
            output,
            metadata: json!({ "role": role }),
        })
    }
}

fn text_output_for_role(role: &str) -> Result<String, AgentError> {
    match role {
        RESEARCHER_ROLE => Ok(RESEARCH_RESULT.to_string()),
        REVIEWER_ROLE => Ok(REVIEW_RESULT.to_string()),
        PUBLISHER_ROLE => Ok(PUBLISHED_REPORT.to_string()),
        _ => Err(AgentError::Adapter(format!("unexpected text role: {role}"))),
    }
}

fn builder_output() -> serde_json::Value {
    json!({
        "deliverable": "structured proposal",
        "assumptions": [],
        "risks": ["external facts require validation"],
        "validation": ["reviewer approval"],
    })
}

fn request_role(request: &AgentRequest) -> String {
    request.prompt.lines().next().unwrap_or_default().trim_start_matches('#').trim().to_string()
}

fn fixture_file_system() -> MemoryFileSystem {
    MemoryFileSystem::new(HashMap::from([(BRIEF_PATH.to_string(), BRIEF_TEXT.to_string())]))
}

fn fixture_executor(file_system: MemoryFileSystem, adapter: RecordingAgentAdapter) -> Executor {
    Executor::new_with_runtime_clients(
        Box::<EmptyCredentialProvider>::default(),
        Box::<NoopHttpClient>::default(),
        Box::new(file_system),
        Box::<NoopImageIo>::default(),
        Box::new(adapter),
    )
}

fn request_log(requests: &Arc<Mutex<Vec<AgentRequest>>>) -> Vec<AgentRequest> {
    requests.lock().expect("request log lock succeeds").clone()
}

// r[verify n8nclone.workflow_execution.llm_operating_graph.scenario.fanout]
#[tokio::test]
async fn llm_operating_graph_executes_roles_fanout_aggregation_and_effects() {
    let workflow = load_workflow_nickel_file(Path::new(FIXTURE_PATH)).expect("operating graph fixture loads");
    let file_system = fixture_file_system();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let adapter = RecordingAgentAdapter::succeeds(Arc::clone(&requests));
    let mut executor = fixture_executor(file_system.clone(), adapter);

    let (_run_id, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert_eq!(steps.len(), OPERATING_GRAPH_NODE_COUNT);
    assert!(steps.iter().all(|step| matches!(step.status, NodeStatus::Completed { .. })));
    let requests = request_log(&requests);
    assert_eq!(requests.len(), OPERATING_GRAPH_ROLE_COUNT);
    assert_eq!(requests.iter().map(request_role).collect::<Vec<_>>(), vec![
        RESEARCHER_ROLE,
        BUILDER_ROLE,
        REVIEWER_ROLE,
        PUBLISHER_ROLE,
    ]);

    let researcher = &requests[RESEARCHER_REQUEST_INDEX];
    let builder = &requests[BUILDER_REQUEST_INDEX];
    let reviewer = &requests[REVIEWER_REQUEST_INDEX];
    let publisher = &requests[PUBLISHER_REQUEST_INDEX];
    assert_eq!(researcher.text_inputs, vec![BRIEF_TEXT.to_string()]);
    assert!(!builder.json_inputs.is_empty());
    assert_eq!(reviewer.text_inputs, vec![RESEARCH_RESULT.to_string()]);
    assert_eq!(reviewer.json_inputs, vec![builder_output()]);
    assert_eq!(publisher.text_inputs, vec![REVIEW_RESULT.to_string()]);
    assert_eq!(file_system.read_text(REPORT_PATH).as_deref(), Some(PUBLISHED_REPORT));
}

// r[verify n8nclone.workflow_execution.llm_operating_graph]
#[tokio::test]
async fn failed_role_blocks_aggregation_and_effect_nodes() {
    let workflow = load_workflow_nickel_file(Path::new(FIXTURE_PATH)).expect("operating graph fixture loads");
    let file_system = fixture_file_system();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let adapter = RecordingAgentAdapter::fails_for(Arc::clone(&requests), BUILDER_ROLE);
    let mut executor = fixture_executor(file_system.clone(), adapter);

    let (_run_id, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert_eq!(request_log(&requests).len(), FAILURE_ADAPTER_CALL_COUNT);
    assert!(steps.iter().any(|step| matches!(step.status, NodeStatus::Failed { .. })));
    let blocked_labels = steps
        .iter()
        .filter(|step| matches!(step.status, NodeStatus::Skipped))
        .map(|step| {
            workflow
                .graph
                .node(step.node_id)
                .map(|node| node.label.as_str())
                .expect("blocked node remains in graph")
        })
        .collect::<Vec<_>>();
    assert_eq!(blocked_labels.len(), BLOCKED_DOWNSTREAM_COUNT);
    assert_eq!(blocked_labels, vec![REVIEWER_ROLE, PUBLISHER_ROLE, REPORT_EFFECT_LABEL]);
    assert!(file_system.read_text(REPORT_PATH).is_none());
}
