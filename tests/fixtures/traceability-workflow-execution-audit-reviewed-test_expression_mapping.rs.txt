//! Tests for workflow expression data mapping.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use lattice::engine::executor::Executor;
use lattice::engine::executor::NodeStatus;
use lattice::engine::executor::node_execution_cache_key;
use lattice::engine::value::ContentHash;
use lattice::engine::value::Value;
use lattice::engine::workflow::NodeConfig;
use lattice::engine::workflow::Workflow;
use lattice::expressions::ExpressionContext;
use lattice::expressions::ExpressionSegment;
use lattice::expressions::ExpressionTransform;
use lattice::expressions::evaluate_expression_template;
use lattice::expressions::parse_expression_template;
use lattice::http::HttpClient;
use lattice::http::HttpError;
use lattice::http::HttpRequestPlan;
use lattice::http::HttpResponse;
use lattice::workflow_validation::WorkflowDiagnosticCode;
use lattice::workflow_validation::validate_workflow;
use pretty_assertions::assert_eq;
use rat_nodegraph::model::PortId;

const SOURCE_LABEL: &str = "source";
const TARGET_LABEL: &str = "target";
const HTTP_LABEL: &str = "http";
const STDOUT_OUTPUT_INDEX: usize = 0;
const HTTP_BODY_INPUT_INDEX: usize = 0;
const SECRET_VALUE: &str = "s3cr3t";
const REDACTED_SECRET: &str = "<redacted>";
const TRIMMED_VALUE: &str = "Hello";
const JSON_FIELD_VALUE: &str = "field-value";
const HTTP_URL: &str = "https://example.test/expression";
const HTTP_STATUS_OK: u16 = 200;
const HTTP_TIMEOUT_SECS: u64 = 60;

#[derive(Clone, Default)]
struct RecordingHttpClient {
    request: Arc<Mutex<Option<HttpRequestPlan>>>,
}

impl RecordingHttpClient {
    fn recorded(&self) -> Option<HttpRequestPlan> {
        self.request.lock().expect("request lock").clone()
    }
}

impl HttpClient for RecordingHttpClient {
    fn execute(&self, request: &HttpRequestPlan) -> Result<HttpResponse, HttpError> {
        *self.request.lock().expect("request lock") = Some(request.clone());
        Ok(HttpResponse {
            status: HTTP_STATUS_OK,
            headers: HashMap::new(),
            body: String::new(),
        })
    }
}

fn stdout_port(workflow: &Workflow, node_id: rat_nodegraph::model::NodeId) -> PortId {
    workflow.graph.node(node_id).expect("node exists").output_ports[STDOUT_OUTPUT_INDEX].id
}

fn expression_for(node_id: rat_nodegraph::model::NodeId, port_id: PortId) -> String {
    format!("{{{{node:{}:port:{}}}}}", node_id.0, port_id.0)
}

fn cache_for_node(
    workflow: &Workflow,
    node_id: rat_nodegraph::model::NodeId,
    outputs: HashMap<PortId, Value>,
) -> HashMap<ContentHash, HashMap<PortId, Value>> {
    let key = node_execution_cache_key(node_id, workflow.node_config(node_id).expect("node config"), &HashMap::new());
    [(key, outputs)].into()
}

// r[verify n8nclone.workflow_execution.expression_mapping.syntax]
// r[verify n8nclone.workflow_execution.expression_mapping.syntax.reference]
#[test]
fn parses_output_reference_expression() {
    let template = parse_expression_template("prefix {{node:1:port:2|trim|upper}} suffix").expect("parse expression");

    assert_eq!(template.segments.len(), 3);
    match &template.segments[1] {
        ExpressionSegment::Reference(reference) => {
            assert_eq!(reference.node_id, 1);
            assert_eq!(reference.port_id, 2);
            assert_eq!(reference.transforms, vec![ExpressionTransform::Trim, ExpressionTransform::Upper]);
        }
        other => panic!("expected reference segment, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.expression_mapping.syntax.invalid]
#[test]
fn invalid_expression_syntax_is_validation_diagnostic() {
    let mut workflow = Workflow::new("invalid-expression");
    workflow.add_shell_node(TARGET_LABEL, "echo {{node:bad}}");

    let diagnostics = validate_workflow(&workflow);

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == WorkflowDiagnosticCode::InvalidExpressionSyntax)
    );
}

// r[verify n8nclone.workflow_execution.expression_mapping.evaluate]
// r[verify n8nclone.workflow_execution.expression_mapping.evaluate.transform]
#[test]
fn evaluates_bounded_transforms_without_side_effects() {
    let mut context = ExpressionContext::default();
    context.insert(1, 2, Value::from_string("  hello  ".to_string()));
    let template = parse_expression_template("{{node:1:port:2|trim|upper}}").expect("parse expression");

    let value = evaluate_expression_template(&template, &context).expect("evaluate expression");

    assert_eq!(value.text, "HELLO");
}

// r[verify n8nclone.workflow_execution.expression_mapping.evaluate.stdout]
// r[verify n8nclone.workflow_execution.expression_mapping.config.command]
#[tokio::test]
async fn shell_command_resolves_upstream_stdout_expression() {
    let mut workflow = Workflow::new("command-expression");
    let source_id = workflow.add_shell_node(SOURCE_LABEL, "unused");
    let source_stdout = stdout_port(&workflow, source_id);
    let target_id = workflow.add_shell_node(TARGET_LABEL, "placeholder");
    let target_stdin = workflow.graph.node(target_id).expect("target node").input_ports[HTTP_BODY_INPUT_INDEX].id;
    workflow.graph.add_edge(source_stdout, target_stdin).expect("connect nodes");
    workflow.set_node_config(
        target_id,
        NodeConfig::shell(format!("printf '{}'", expression_for(source_id, source_stdout))),
    );
    let cache =
        cache_for_node(&workflow, source_id, [(source_stdout, Value::from_string(TRIMMED_VALUE.to_string()))].into());
    let mut executor = Executor::new();

    let (_run_id, results) = executor.run(&workflow, &cache).await;

    let target = results.iter().find(|result| result.node_id == target_id).expect("target result");
    match &target.status {
        NodeStatus::Completed { outputs, .. } => {
            let target_stdout = stdout_port(&workflow, target_id);
            assert_eq!(outputs.get(&target_stdout).expect("target stdout").as_display_string(), TRIMMED_VALUE);
        }
        other => panic!("expected completed target, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.expression_mapping.config.http_body]
#[tokio::test]
async fn http_body_resolves_upstream_expression() {
    let mut workflow = Workflow::new("http-body-expression");
    let source_id = workflow.add_shell_node(SOURCE_LABEL, "unused");
    let source_stdout = stdout_port(&workflow, source_id);
    let http_id = workflow.add_http_node(HTTP_LABEL, "POST", HTTP_URL);
    let http_body_input = workflow.graph.node(http_id).expect("http node").input_ports[HTTP_BODY_INPUT_INDEX].id;
    workflow.graph.add_edge(source_stdout, http_body_input).expect("connect nodes");
    workflow.set_node_config(http_id, NodeConfig::Http {
        method: "POST".to_string(),
        url: HTTP_URL.to_string(),
        headers: HashMap::new(),
        body: Some(expression_for(source_id, source_stdout)),
        timeout_secs: HTTP_TIMEOUT_SECS,
    });
    let cache = cache_for_node(
        &workflow,
        source_id,
        [(source_stdout, Value::from_string(JSON_FIELD_VALUE.to_string()))].into(),
    );
    let client = RecordingHttpClient::default();
    let recorded = client.clone();
    let mut executor = Executor::new_with_http_client(Box::new(client));

    let (_run_id, _results) = executor.run(&workflow, &cache).await;

    let request = recorded.recorded().expect("recorded HTTP request");
    assert_eq!(request.body.as_deref(), Some(JSON_FIELD_VALUE));
}

// r[verify n8nclone.workflow_execution.expression_mapping.errors]
// r[verify n8nclone.workflow_execution.expression_mapping.errors.missing]
#[tokio::test]
async fn missing_expression_reference_fails_before_adapter_execution() {
    let mut workflow = Workflow::new("missing-expression");
    let target_id = workflow.add_shell_node(TARGET_LABEL, "printf '{{node:999:port:999}}'");
    let executor = Executor::new();

    let result = executor.execute_node(&workflow, target_id, &HashMap::new()).await;

    match result.status {
        NodeStatus::Failed { error } => assert!(error.contains("unavailable"), "error: {error}"),
        other => panic!("expected failed node, got {other:?}"),
    }
}

// r[verify n8nclone.workflow_execution.expression_mapping.errors.type]
#[test]
fn json_transform_on_string_reports_type_mismatch() {
    let mut context = ExpressionContext::default();
    context.insert(1, 2, Value::from_string("not-json".to_string()));
    let template = parse_expression_template("{{node:1:port:2|json:name}}").expect("parse expression");

    let error = evaluate_expression_template(&template, &context).expect_err("type mismatch");

    assert!(error.to_string().contains("cannot be applied to string"));
}

// r[verify n8nclone.workflow_execution.expression_mapping.redaction]
// r[verify n8nclone.workflow_execution.expression_mapping.redaction.secret]
#[tokio::test]
async fn secret_derived_expression_output_is_redacted() {
    let mut workflow = Workflow::new("redacted-expression");
    let source_id = workflow.add_shell_node(SOURCE_LABEL, "unused");
    let source_stdout = stdout_port(&workflow, source_id);
    let target_id = workflow.add_shell_node(TARGET_LABEL, "placeholder");
    let target_stdin = workflow.graph.node(target_id).expect("target node").input_ports[HTTP_BODY_INPUT_INDEX].id;
    workflow.graph.add_edge(source_stdout, target_stdin).expect("connect nodes");
    workflow.set_node_config(
        target_id,
        NodeConfig::shell(format!("printf 'token={}'", expression_for(source_id, source_stdout))),
    );
    let cache = cache_for_node(
        &workflow,
        source_id,
        [(source_stdout, Value::from_sensitive_string(SECRET_VALUE.to_string()))].into(),
    );
    let mut executor = Executor::new();

    let (_run_id, results) = executor.run(&workflow, &cache).await;

    let target = results.iter().find(|result| result.node_id == target_id).expect("target result");
    match &target.status {
        NodeStatus::Completed { outputs, .. } => {
            let target_stdout = stdout_port(&workflow, target_id);
            let output = outputs.get(&target_stdout).expect("target stdout").as_display_string();
            assert_eq!(output, format!("token={REDACTED_SECRET}"));
        }
        other => panic!("expected completed target, got {other:?}"),
    }
}
