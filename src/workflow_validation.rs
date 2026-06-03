//! Pure workflow validation diagnostics.

use rat_nodegraph::model::NodeId;
use rat_nodegraph::model::PortDirection;
use rat_nodegraph::model::PortId;
use serde::Deserialize;
use serde::Serialize;

use crate::credentials;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::Workflow;
use crate::http;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Warning,
    Fatal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowDiagnosticCode {
    EmptyWorkflow,
    EmptyShellCommand,
    MissingNodeConfig,
    ConfigWithoutNode,
    EdgePortMissing,
    EdgeWrongDirection,
    EdgeSelfLoop,
    EdgeIncompatibleType,
    MalformedSecretRef,
    CredentialRefWithoutNode,
    InvalidHttpMethod,
    InvalidHttpUrl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: WorkflowDiagnosticCode,
    pub node_id: Option<NodeId>,
    pub port_id: Option<PortId>,
    pub message: String,
}

// r[impl n8nclone.workflow_validation.diagnostics.core]
// r[impl n8nclone.workflow_validation.diagnostics.negative]
pub fn validate_workflow(workflow: &Workflow) -> Vec<WorkflowDiagnostic> {
    let mut diagnostics = Vec::new();

    if workflow.graph.node_count() == 0 {
        diagnostics.push(warning(
            WorkflowDiagnosticCode::EmptyWorkflow,
            None,
            None,
            "workflow has no nodes".to_string(),
        ));
    }

    validate_node_configs(workflow, &mut diagnostics);
    validate_credentials(workflow, &mut diagnostics);
    validate_edges(workflow, &mut diagnostics);

    diagnostics
}

// r[impl n8nclone.workflow_validation.run_gate]
pub fn has_fatal_diagnostics(diagnostics: &[WorkflowDiagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Fatal)
}

pub fn fatal_diagnostics(diagnostics: &[WorkflowDiagnostic]) -> Vec<WorkflowDiagnostic> {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Fatal)
        .cloned()
        .collect()
}

pub fn format_diagnostic(diagnostic: &WorkflowDiagnostic) -> String {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Fatal => "fatal",
    };
    let location = match (diagnostic.node_id, diagnostic.port_id) {
        (Some(node_id), Some(port_id)) => format!("{node_id}/{port_id}"),
        (Some(node_id), None) => node_id.to_string(),
        (None, Some(port_id)) => port_id.to_string(),
        (None, None) => "workflow".to_string(),
    };
    format!("{severity} {location}: {}", diagnostic.message)
}

fn validate_node_configs(workflow: &Workflow, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    for node in workflow.graph.nodes() {
        match workflow.node_config(node.id) {
            Some(NodeConfig::Shell { command, .. }) if command.trim().is_empty() => {
                diagnostics.push(fatal(
                    WorkflowDiagnosticCode::EmptyShellCommand,
                    Some(node.id),
                    None,
                    format!("shell node '{}' has an empty command", node.label),
                ));
            }
            Some(NodeConfig::Shell { .. }) => {}
            Some(NodeConfig::Http { method, url, .. }) => {
                if !http::validate_http_method(method) {
                    diagnostics.push(fatal(
                        WorkflowDiagnosticCode::InvalidHttpMethod,
                        Some(node.id),
                        None,
                        format!("HTTP node '{}' has an invalid method", node.label),
                    ));
                }
                if !http::validate_http_url(url) {
                    diagnostics.push(fatal(
                        WorkflowDiagnosticCode::InvalidHttpUrl,
                        Some(node.id),
                        None,
                        format!("HTTP node '{}' has an invalid URL", node.label),
                    ));
                }
            }
            None => diagnostics.push(fatal(
                WorkflowDiagnosticCode::MissingNodeConfig,
                Some(node.id),
                None,
                format!("node '{}' has no execution config", node.label),
            )),
        }
    }

    for node_id in workflow.configs.keys() {
        let node_id = NodeId(*node_id);
        if workflow.graph.node(node_id).is_none() {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::ConfigWithoutNode,
                Some(node_id),
                None,
                "node config references a missing graph node".to_string(),
            ));
        }
    }
}

fn validate_credentials(workflow: &Workflow, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    for (node_id, refs) in &workflow.credentials {
        let node_id = NodeId(*node_id);
        if workflow.graph.node(node_id).is_none() {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::CredentialRefWithoutNode,
                Some(node_id),
                None,
                "credential reference belongs to a missing graph node".to_string(),
            ));
            continue;
        }

        for (env_name, secret_ref) in &refs.env {
            if env_name.trim().is_empty() {
                diagnostics.push(fatal(
                    WorkflowDiagnosticCode::MalformedSecretRef,
                    Some(node_id),
                    None,
                    "credential environment variable name is empty".to_string(),
                ));
            }
            if let Err(error) = credentials::validate_secret_ref(secret_ref) {
                diagnostics.push(fatal(
                    WorkflowDiagnosticCode::MalformedSecretRef,
                    Some(node_id),
                    None,
                    error.to_string(),
                ));
            }
        }
    }
}

fn validate_edges(workflow: &Workflow, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    for edge in workflow.graph.edges() {
        let Some(source_port) = workflow.graph.port(edge.source) else {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::EdgePortMissing,
                None,
                Some(edge.source),
                "edge source port is missing".to_string(),
            ));
            continue;
        };
        let Some(target_port) = workflow.graph.port(edge.target) else {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::EdgePortMissing,
                None,
                Some(edge.target),
                "edge target port is missing".to_string(),
            ));
            continue;
        };

        if source_port.direction != PortDirection::Output || target_port.direction != PortDirection::Input {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::EdgeWrongDirection,
                None,
                Some(edge.source),
                "edge must connect an output port to an input port".to_string(),
            ));
            continue;
        }

        let source_node = workflow.graph.port_owner(edge.source);
        let target_node = workflow.graph.port_owner(edge.target);
        if source_node == target_node {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::EdgeSelfLoop,
                source_node,
                None,
                "edge connects ports on the same node".to_string(),
            ));
            continue;
        }

        if !port_types_compatible(&source_port.type_tag, &target_port.type_tag) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::EdgeIncompatibleType,
                target_node,
                Some(edge.target),
                format!("edge connects incompatible types '{}' to '{}'", source_port.type_tag, target_port.type_tag),
            ));
        }
    }
}

fn port_types_compatible(source: &str, target: &str) -> bool {
    source == target || source == "any" || target == "any"
}

fn fatal(
    code: WorkflowDiagnosticCode,
    node_id: Option<NodeId>,
    port_id: Option<PortId>,
    message: String,
) -> WorkflowDiagnostic {
    WorkflowDiagnostic {
        severity: DiagnosticSeverity::Fatal,
        code,
        node_id,
        port_id,
        message,
    }
}

fn warning(
    code: WorkflowDiagnosticCode,
    node_id: Option<NodeId>,
    port_id: Option<PortId>,
    message: String,
) -> WorkflowDiagnostic {
    WorkflowDiagnostic {
        severity: DiagnosticSeverity::Warning,
        code,
        node_id,
        port_id,
        message,
    }
}
