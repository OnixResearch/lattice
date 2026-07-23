//! Tests for HTTP request nodes.

use std::collections::HashMap;

use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::executor::node_execution_cache_key;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::Workflow;
use lattice::http::HttpError;
use lattice::http::HttpResponse;
use lattice::http::StaticHttpClient;
use lattice::http::StaticHttpResult;
use lattice::workflow_validation::WorkflowDiagnosticCode;
use lattice::workflow_validation::validate_workflow;
use pretty_assertions::assert_eq;

const HTTP_NODE_LABEL: &str = "fetch";
const HTTP_METHOD_GET: &str = "GET";
const HTTP_METHOD_POST: &str = "POST";
const HTTP_METHOD_INVALID: &str = "FETCHISH";
const HTTP_URL: &str = "https://example.test/data";
const HTTP_OTHER_URL: &str = "https://example.test/other";
const HTTP_INVALID_URL: &str = "file:///tmp/nope";
const HTTP_BODY: &str = "response-body";
const HTTP_HEADER_KEY: &str = "content-type";
const HTTP_HEADER_VALUE: &str = "text/plain";
const STATUS_SUCCESS: u16 = 200;
const STATUS_OUTPUT_INDEX: usize = 0;
const HEADERS_OUTPUT_INDEX: usize = 1;
const BODY_OUTPUT_INDEX: usize = 2;

fn http_response() -> HttpResponse {
    HttpResponse {
        status: STATUS_SUCCESS,
        headers: [(HTTP_HEADER_KEY.to_string(), HTTP_HEADER_VALUE.to_string())].into(),
        body: HTTP_BODY.to_string(),
    }
}

fn workflow_with_http_node() -> (Workflow, rat_nodegraph::model::NodeId) {
    let mut workflow = Workflow::new("http-workflow");
    let node_id = workflow.add_http_node(HTTP_NODE_LABEL, HTTP_METHOD_GET, HTTP_URL);
    (workflow, node_id)
}

// r[verify n8nclone.workflow_execution.http_node.config]
#[test]
fn add_http_node_records_config_and_ports() {
    let (workflow, node_id) = workflow_with_http_node();
    let node = workflow.graph.node(node_id).expect("HTTP node exists");

    assert_eq!(node.input_ports.len(), 1);
    assert_eq!(node.output_ports.len(), BODY_OUTPUT_INDEX + 1);
    match workflow.node_config(node_id).expect("HTTP config exists") {
        NodeConfig::Http { method, url, .. } => {
            assert_eq!(method, HTTP_METHOD_GET);
            assert_eq!(url, HTTP_URL);
        }
        other => panic!("expected HTTP config, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.http_node.execute]
// r[verify n8nclone.workflow_execution.http_node.outputs]
#[tokio::test]
async fn http_node_executes_through_client_and_maps_outputs() {
    let (workflow, node_id) = workflow_with_http_node();
    let client = StaticHttpClient::new([(HTTP_URL.to_string(), StaticHttpResult::Response(http_response()))].into());
    let executor = Executor::new_with_http_client(Box::new(client));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Completed { outputs, .. } => {
            let node = workflow.graph.node(node_id).expect("HTTP node exists");
            let status = outputs
                .get(&node.output_ports[STATUS_OUTPUT_INDEX].id)
                .expect("status output exists")
                .as_display_string();
            let headers = outputs
                .get(&node.output_ports[HEADERS_OUTPUT_INDEX].id)
                .expect("headers output exists")
                .as_display_string();
            let body = outputs
                .get(&node.output_ports[BODY_OUTPUT_INDEX].id)
                .expect("body output exists")
                .as_display_string();
            assert_eq!(status, STATUS_SUCCESS.to_string());
            assert!(headers.contains(HTTP_HEADER_KEY));
            assert_eq!(body, HTTP_BODY);
        }
        other => panic!("expected completed HTTP node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.http_node.config]
// r[verify n8nclone.workflow_execution.http_node.errors]
#[test]
fn invalid_http_method_and_url_are_validation_diagnostics() {
    let mut workflow = Workflow::new("invalid-http");
    workflow.add_http_node(HTTP_NODE_LABEL, HTTP_METHOD_INVALID, HTTP_INVALID_URL);

    let diagnostics = validate_workflow(&workflow);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidHttpMethod));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidHttpUrl));
}

// r[verify n8nclone.workflow_execution.http_node.errors]
#[tokio::test]
async fn http_transport_failure_marks_node_failed() {
    let (workflow, node_id) = workflow_with_http_node();
    let client = StaticHttpClient::new(
        [(HTTP_URL.to_string(), StaticHttpResult::Error(HttpError::Transport("offline".to_string())))].into(),
    );
    let executor = Executor::new_with_http_client(Box::new(client));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("offline"), "error: {error}"),
        other => panic!("expected failed HTTP node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.http_node.errors]
#[tokio::test]
async fn http_timeout_marks_node_failed() {
    let (workflow, node_id) = workflow_with_http_node();
    let client = StaticHttpClient::new(
        [(HTTP_URL.to_string(), StaticHttpResult::Error(HttpError::Timeout("deadline".to_string())))].into(),
    );
    let executor = Executor::new_with_http_client(Box::new(client));

    let result = executor.execute_node(&workflow, node_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("timed out"), "error: {error}"),
        other => panic!("expected failed HTTP node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.http_node.cache_key]
#[test]
fn http_cache_key_changes_when_request_config_changes() {
    let (workflow, node_id) = workflow_with_http_node();
    let original_key =
        node_execution_cache_key(node_id, workflow.node_config(node_id).expect("config"), &HashMap::new());

    let mut changed_workflow = Workflow::new("changed-http");
    let changed_node_id = changed_workflow.add_http_node(HTTP_NODE_LABEL, HTTP_METHOD_POST, HTTP_OTHER_URL);
    let changed_key = node_execution_cache_key(
        node_id,
        changed_workflow.node_config(changed_node_id).expect("config"),
        &HashMap::new(),
    );

    assert_ne!(original_key, changed_key);
}
