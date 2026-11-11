//! End-to-end checks for the bounded planner/generator/evaluator build loop.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use lattice::agent_node::AgentAdapter;
use lattice::agent_node::AgentError;
use lattice::agent_node::AgentOutput;
use lattice::agent_node::AgentOutputMode;
use lattice::agent_node::AgentRequest;
use lattice::agent_node::AgentResponse;
use lattice::agent_node::ExecutionContext as AgentExecutionContext;
use lattice::credentials::EmptyCredentialProvider;
use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::file_node::MemoryFileSystem;
use lattice::http::NoopHttpClient;
use lattice::image_node::NoopImageIo;
use lattice::orchestration::CompletionPolicy;
use lattice::orchestration::load_definition_nickel_file;
use lattice::orchestration::validate_definition;
use lattice::orchestration::workflow_revision;
use lattice::workflow_format::load_workflow_nickel_file;
use pretty_assertions::assert_eq;
use serde_json::json;

const WORKFLOW_PATH: &str = "examples/workflows/bounded-build-loop.ncl";
const ORCHESTRATION_PATH: &str = "examples/orchestrations/bounded-build-loop.ncl";
const PLANNER_ROLE: &str = "Planner";
const GENERATOR_ROLE: &str = "Generator";
const PLAN: &str = "Print one line with a stable greeting.";
const GENERATED_SOURCE: &str = "fn main() {\n    println!(\"bounded build loop\");\n}\n";
const EXPECTED_NODE_COUNT: usize = 3;
const EXPECTED_AGENT_CALL_COUNT: usize = 2;
const EVALUATOR_NODE_ID: u64 = 2;
const EVALUATOR_STDOUT: &str = "bounded build loop";

#[derive(Debug, Clone)]
struct BoundedBuildAdapter {
    requests: Arc<Mutex<Vec<AgentRequest>>>,
}

impl AgentAdapter for BoundedBuildAdapter {
    fn execution_identity(&self, _request: &AgentRequest) -> Result<String, AgentError> {
        Ok("test/bounded-build-loop/v1".to_owned())
    }

    fn execute(&self, request: &AgentRequest, _context: &AgentExecutionContext) -> Result<AgentResponse, AgentError> {
        self.requests.lock().expect("request log lock succeeds").push(request.clone());
        let role = request.prompt.lines().next().unwrap_or_default().trim_start_matches('#').trim();
        let text = match role {
            PLANNER_ROLE => PLAN,
            GENERATOR_ROLE => GENERATED_SOURCE,
            other => return Err(AgentError::Adapter(format!("unexpected role: {other}"))),
        };
        Ok(AgentResponse {
            output: AgentOutput::Text(text.to_string()),
            metadata: json!({ "role": role }),
        })
    }
}

fn request_log(requests: &Arc<Mutex<Vec<AgentRequest>>>) -> Vec<AgentRequest> {
    requests.lock().expect("request log lock succeeds").clone()
}

// r[verify n8nclone.workflow_execution.llm_operating_graph.scenario.fanout]
#[tokio::test]
async fn bounded_build_loop_compiles_and_runs_generated_rust() {
    let workflow = load_workflow_nickel_file(Path::new(WORKFLOW_PATH)).expect("build-loop fixture loads");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let adapter = BoundedBuildAdapter {
        requests: Arc::clone(&requests),
    };
    let mut executor = Executor::new_with_runtime_clients(
        Box::<EmptyCredentialProvider>::default(),
        Box::<NoopHttpClient>::default(),
        Box::new(MemoryFileSystem::new(HashMap::new())),
        Box::<NoopImageIo>::default(),
        Box::new(adapter),
    );

    let (_run_id, steps) = executor.run(&workflow, &HashMap::new()).await;

    assert_eq!(steps.len(), EXPECTED_NODE_COUNT);
    assert!(
        steps.iter().all(|step| matches!(step.status, NodeStatus::Completed { .. })),
        "build loop must complete every node: {steps:?}"
    );

    let requests = request_log(&requests);
    assert_eq!(requests.len(), EXPECTED_AGENT_CALL_COUNT);
    assert_eq!(requests[0].output_mode, AgentOutputMode::Text);
    assert_eq!(requests[1].output_mode, AgentOutputMode::Text);
    assert_eq!(requests[1].text_inputs, vec![PLAN.to_string()]);

    let evaluator = steps.iter().find(|step| step.node_id.0 == EVALUATOR_NODE_ID).expect("evaluator step is present");
    let NodeStatus::Completed { outputs, .. } = &evaluator.status else {
        panic!("expected completed evaluator, got {:?}", evaluator.status);
    };
    let stdout = workflow
        .graph
        .node(evaluator.node_id)
        .and_then(|node| node.output_ports.iter().find(|port| port.label == "stdout"))
        .and_then(|port| outputs.get(&port.id))
        .expect("evaluator stdout is present");
    assert!(stdout.as_display_string().contains(EVALUATOR_STDOUT));
}

// r[verify lattice.agent_orchestration.rounds]
// r[verify lattice.agent_orchestration.budgets]
#[test]
fn bounded_build_orchestration_caps_rounds_and_retries() {
    let workflow = load_workflow_nickel_file(Path::new(WORKFLOW_PATH)).expect("build-loop fixture loads");
    let definition = load_definition_nickel_file(Path::new(ORCHESTRATION_PATH)).expect("orchestration loads");
    let revision = workflow_revision(&workflow).expect("workflow revision hashes");

    assert!(validate_definition(&definition, Some(&revision)).is_empty());
    assert_eq!(definition.completion, CompletionPolicy::AwaitProposal);
    assert_eq!(definition.budgets.limits.rounds, 2);
    assert_eq!(definition.budgets.limits.agent_invocations, 4);
    assert_eq!(definition.budgets.per_round_upper_bound.agent_invocations, 2);
    assert_eq!(definition.retry.expect("retry policy is present").max_observed_failure_retries, 1);
}
