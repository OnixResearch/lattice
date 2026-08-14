//! Pure n8n JSON import/export mapping.
//!
//! The core in this module consumes JSON values and produces typed workflows,
//! mapping reports, and deterministic diagnostics without filesystem or store I/O.

use std::collections::BTreeMap;
use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::credentials::NodeCredentialRefs;
use crate::credentials::SecretRef;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::PortId;
use crate::engine::workflow::Workflow;

const N8N_EXECUTE_COMMAND_TYPE: &str = "n8n-nodes-base.executeCommand";
const N8N_HTTP_REQUEST_TYPE: &str = "n8n-nodes-base.httpRequest";
const N8N_CREDENTIAL_PROVIDER: &str = "n8n";
const CREDENTIAL_ENV_PREFIX: &str = "N8N_CREDENTIAL";
const CONNECTION_MAIN: &str = "main";
const HTTP_BODY_OUTPUT_INDEX: usize = 2;
const DEFAULT_HTTP_METHOD: &str = "GET";
const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 60;
const DEFAULT_POSITION: i32 = 0;
const POSITION_X_INDEX: usize = 0;
const POSITION_Y_INDEX: usize = 1;
const REQUIRED_CONNECTION_INDEX: usize = 0;
const N8N_NODE_ID_PREFIX: &str = "node-";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct N8nMappingReport {
    pub workflow_name: String,
    pub entries: Vec<N8nMappingEntry>,
    pub diagnostics: Vec<N8nDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct N8nMappingEntry {
    pub source_id: String,
    pub source_name: String,
    pub target_node_id: Option<u64>,
    pub status: N8nMappingStatus,
    pub diagnostics: Vec<N8nDiagnostic>,
    pub unsupported_fields: Vec<String>,
    pub credential_placeholders: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum N8nMappingStatus {
    Mapped,
    Unsupported,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct N8nDiagnostic {
    pub code: N8nDiagnosticCode,
    pub source_id: Option<String>,
    pub source_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum N8nDiagnosticCode {
    MalformedJson,
    MissingWorkflowField,
    UnsupportedNodeKind,
    UnsupportedExpression,
    MalformedConnection,
    PlaintextCredentialMaterial,
    UnsupportedExportNode,
}

#[derive(Debug, Clone)]
pub struct N8nImportResult {
    pub workflow: Option<Workflow>,
    pub report: N8nMappingReport,
}

#[derive(Debug, Clone)]
pub struct N8nExportResult {
    pub json: Option<JsonValue>,
    pub report: N8nMappingReport,
}

#[derive(Debug, Clone, Deserialize)]
struct N8nWorkflowJson {
    name: Option<String>,
    nodes: Option<Vec<N8nNodeJson>>,
    #[serde(default)]
    connections: BTreeMap<String, N8nConnectionGroups>,
    #[serde(default)]
    credentials: Option<JsonValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct N8nNodeJson {
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    position: Option<Vec<i32>>,
    #[serde(default)]
    parameters: JsonValue,
    #[serde(default)]
    credentials: Option<JsonValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct N8nConnectionGroups {
    #[serde(default)]
    main: Vec<Vec<N8nConnectionJson>>,
}

#[derive(Debug, Clone, Deserialize)]
struct N8nConnectionJson {
    node: String,
}

// r[impl n8nclone.workflow_format.n8n_json.import]
// r[impl n8nclone.workflow_validation.n8n_json.unsupported]
// r[impl n8nclone.credentials.n8n_json.placeholders]
pub fn import_n8n_json_str(input: &str, timestamp: crate::engine::workflow::WorkflowTimestamp) -> N8nImportResult {
    let parsed = match serde_json::from_str::<JsonValue>(input) {
        Ok(value) => value,
        Err(error) => return failed_import_report("unknown", malformed_json_diagnostic(error.to_string())),
    };
    import_n8n_json_value(parsed, timestamp)
}

// r[impl n8nclone.workflow_format.n8n_json.import]
// r[impl n8nclone.workflow_validation.n8n_json.unsupported]
pub fn import_n8n_json_value(
    value: JsonValue,
    timestamp: crate::engine::workflow::WorkflowTimestamp,
) -> N8nImportResult {
    let decoded = match serde_json::from_value::<N8nWorkflowJson>(value) {
        Ok(decoded) => decoded,
        Err(error) => return failed_import_report("unknown", malformed_json_diagnostic(error.to_string())),
    };
    let workflow_name = decoded.name.clone().unwrap_or_default();
    let Some(nodes) = decoded.nodes else {
        return failed_import_report(&workflow_name, missing_field_diagnostic("nodes"));
    };
    if workflow_name.trim().is_empty() {
        return failed_import_report(&workflow_name, missing_field_diagnostic("name"));
    }
    if decoded.credentials.is_some() {
        return failed_import_report(&workflow_name, plaintext_workflow_credentials_diagnostic());
    }

    let mut workflow = Workflow::new(workflow_name.clone(), timestamp);
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    let mut node_ids_by_name = HashMap::new();
    let mut configs_by_node_id = HashMap::new();

    for node in &nodes {
        let mapping = import_node(&mut workflow, node);
        if let Some(node_id) = mapping.node_id {
            node_ids_by_name.insert(node.name.clone(), node_id);
            if let Some(config) = workflow.node_config(node_id).cloned() {
                configs_by_node_id.insert(node_id, config);
            }
        }
        diagnostics.extend(mapping.entry.diagnostics.clone());
        entries.push(mapping.entry);
    }

    diagnostics.extend(import_connections(&mut workflow, &decoded.connections, &node_ids_by_name, &configs_by_node_id));
    debug_assert!(node_ids_by_name.len() <= nodes.len());
    debug_assert!(configs_by_node_id.len() <= node_ids_by_name.len());

    let report = N8nMappingReport {
        workflow_name,
        entries,
        diagnostics,
    };
    let workflow = if report.diagnostics.is_empty() {
        Some(workflow)
    } else {
        None
    };
    N8nImportResult { workflow, report }
}

struct ImportedNode {
    node_id: Option<crate::engine::workflow::NodeId>,
    entry: N8nMappingEntry,
}

fn import_node(workflow: &mut Workflow, node: &N8nNodeJson) -> ImportedNode {
    let source_id = source_id(node);
    let mut entry = N8nMappingEntry {
        source_id: source_id.clone(),
        source_name: node.name.clone(),
        target_node_id: None,
        status: N8nMappingStatus::Mapped,
        diagnostics: Vec::new(),
        unsupported_fields: Vec::new(),
        credential_placeholders: Vec::new(),
    };

    let expression_diagnostics = expression_diagnostics_for_node(node);
    entry.diagnostics.extend(expression_diagnostics);
    let credential_placeholders = match credential_refs_for_node(node) {
        Ok(placeholders) => placeholders,
        Err(diagnostic) => {
            entry.diagnostics.push(diagnostic);
            CredentialPlaceholders::default()
        }
    };

    let node_id = match node.kind.as_str() {
        N8N_EXECUTE_COMMAND_TYPE => import_execute_command_node(workflow, node, &mut entry),
        N8N_HTTP_REQUEST_TYPE => import_http_request_node(workflow, node, &mut entry),
        _ => {
            entry.status = N8nMappingStatus::Unsupported;
            entry.diagnostics.push(N8nDiagnostic {
                code: N8nDiagnosticCode::UnsupportedNodeKind,
                source_id: Some(source_id),
                source_name: Some(node.name.clone()),
                message: format!("unsupported n8n node kind '{}'", node.kind),
            });
            None
        }
    };

    if let Some(node_id) = node_id {
        entry.target_node_id = Some(node_id.0);
        apply_position(workflow, node_id, node.position.as_deref());
        if !credential_placeholders.refs.is_empty() {
            workflow.set_node_credentials(node_id, credential_placeholders.refs);
            entry.credential_placeholders = credential_placeholders.labels;
        }
    }
    if !entry.diagnostics.is_empty() && entry.status == N8nMappingStatus::Mapped {
        entry.status = N8nMappingStatus::Failed;
    }

    ImportedNode { node_id, entry }
}

fn import_execute_command_node(
    workflow: &mut Workflow,
    node: &N8nNodeJson,
    entry: &mut N8nMappingEntry,
) -> Option<crate::engine::workflow::NodeId> {
    let Some(command) = string_parameter(&node.parameters, "command") else {
        entry.diagnostics.push(N8nDiagnostic {
            code: N8nDiagnosticCode::MissingWorkflowField,
            source_id: Some(source_id(node)),
            source_name: Some(node.name.clone()),
            message: "execute command node is missing parameters.command".to_string(),
        });
        return None;
    };
    Some(workflow.add_shell_node(&node.name, command))
}

fn import_http_request_node(
    workflow: &mut Workflow,
    node: &N8nNodeJson,
    entry: &mut N8nMappingEntry,
) -> Option<crate::engine::workflow::NodeId> {
    let Some(url) = string_parameter(&node.parameters, "url") else {
        entry.diagnostics.push(N8nDiagnostic {
            code: N8nDiagnosticCode::MissingWorkflowField,
            source_id: Some(source_id(node)),
            source_name: Some(node.name.clone()),
            message: "HTTP request node is missing parameters.url".to_string(),
        });
        return None;
    };
    let method = string_parameter(&node.parameters, "method").unwrap_or(DEFAULT_HTTP_METHOD);
    let node_id = workflow.add_http_node(&node.name, method, url);
    if let Some(body) = string_parameter(&node.parameters, "body") {
        workflow.set_node_config(node_id, NodeConfig::Http {
            method: method.to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: Some(body.to_string()),
            timeout_secs: DEFAULT_HTTP_TIMEOUT_SECS,
        });
    }
    Some(node_id)
}

fn import_connections(
    workflow: &mut Workflow,
    connections: &BTreeMap<String, N8nConnectionGroups>,
    node_ids_by_name: &HashMap<String, crate::engine::workflow::NodeId>,
    configs_by_node_id: &HashMap<crate::engine::workflow::NodeId, NodeConfig>,
) -> Vec<N8nDiagnostic> {
    let mut diagnostics = Vec::new();
    for (source_name, groups) in connections {
        let Some(source_node_id) = node_ids_by_name.get(source_name).copied() else {
            diagnostics.push(connection_diagnostic(source_name, "connection source node is missing"));
            continue;
        };
        let Some(source_port) = main_output_port(workflow, source_node_id, configs_by_node_id) else {
            diagnostics.push(connection_diagnostic(source_name, "connection source has no output port"));
            continue;
        };
        for group in &groups.main {
            for target in group {
                let Some(target_node_id) = node_ids_by_name.get(&target.node).copied() else {
                    diagnostics.push(connection_diagnostic(source_name, "connection target node is missing"));
                    continue;
                };
                let Some(target_port) = workflow
                    .graph
                    .node(target_node_id)
                    .and_then(|node| node.input_ports.get(REQUIRED_CONNECTION_INDEX))
                    .map(|port| port.id)
                else {
                    diagnostics.push(connection_diagnostic(source_name, "connection target has no input port"));
                    continue;
                };
                if let Err(error) = workflow.graph.add_edge(source_port, target_port) {
                    diagnostics.push(connection_diagnostic(source_name, &error.to_string()));
                }
            }
        }
    }
    diagnostics
}

fn main_output_port(
    workflow: &Workflow,
    node_id: crate::engine::workflow::NodeId,
    configs_by_node_id: &HashMap<crate::engine::workflow::NodeId, NodeConfig>,
) -> Option<PortId> {
    let node = workflow.graph.node(node_id)?;
    match configs_by_node_id.get(&node_id) {
        Some(NodeConfig::Http { .. }) => node.output_ports.get(HTTP_BODY_OUTPUT_INDEX).map(|port| port.id),
        _ => node.output_ports.get(REQUIRED_CONNECTION_INDEX).map(|port| port.id),
    }
}

fn apply_position(workflow: &mut Workflow, node_id: crate::engine::workflow::NodeId, position: Option<&[i32]>) {
    let Some(node) = workflow.graph.node_mut(node_id) else {
        return;
    };
    node.x = position.and_then(|position| position.get(POSITION_X_INDEX).copied()).unwrap_or(DEFAULT_POSITION);
    node.y = position.and_then(|position| position.get(POSITION_Y_INDEX).copied()).unwrap_or(DEFAULT_POSITION);
}

fn expression_diagnostics_for_node(node: &N8nNodeJson) -> Vec<N8nDiagnostic> {
    expression_paths(&node.parameters)
        .into_iter()
        .map(|path| N8nDiagnostic {
            code: N8nDiagnosticCode::UnsupportedExpression,
            source_id: Some(source_id(node)),
            source_name: Some(node.name.clone()),
            message: format!("unsupported n8n expression at parameters.{path}"),
        })
        .collect()
}

fn expression_paths(value: &JsonValue) -> Vec<String> {
    let mut paths = Vec::new();
    collect_expression_paths(value, String::new(), &mut paths);
    paths
}

fn collect_expression_paths(value: &JsonValue, path: String, paths: &mut Vec<String>) {
    match value {
        JsonValue::String(value) if value.contains("{{") => paths.push(path),
        JsonValue::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                collect_expression_paths(value, format!("{path}[{index}]"), paths);
            }
        }
        JsonValue::Object(map) => {
            for (key, value) in map {
                let next_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                collect_expression_paths(value, next_path, paths);
            }
        }
        _ => {}
    }
}

#[derive(Debug, Clone, Default)]
struct CredentialPlaceholders {
    refs: NodeCredentialRefs,
    labels: Vec<String>,
}

fn credential_refs_for_node(node: &N8nNodeJson) -> Result<CredentialPlaceholders, N8nDiagnostic> {
    let Some(credentials) = &node.credentials else {
        return Ok(CredentialPlaceholders::default());
    };
    let Some(credentials) = credentials.as_object() else {
        return Err(plaintext_node_credentials_diagnostic(node));
    };
    let mut refs = NodeCredentialRefs::default();
    let mut labels = Vec::new();
    for (credential_type, identity) in credentials {
        let Some(identity) = identity.as_object() else {
            return Err(plaintext_node_credentials_diagnostic(node));
        };
        if identity.keys().any(|key| key != "id" && key != "name") {
            return Err(plaintext_node_credentials_diagnostic(node));
        }
        let key = identity
            .get("id")
            .and_then(JsonValue::as_str)
            .or_else(|| identity.get("name").and_then(JsonValue::as_str))
            .unwrap_or(credential_type);
        let env_name = credential_env_name(credential_type);
        refs.env.insert(env_name.clone(), SecretRef::new(N8N_CREDENTIAL_PROVIDER, key, None));
        labels.push(format!("{credential_type}:{key}"));
    }
    Ok(CredentialPlaceholders { refs, labels })
}

fn credential_env_name(credential_type: &str) -> String {
    let sanitized = credential_type
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("{CREDENTIAL_ENV_PREFIX}_{sanitized}")
}

// r[impl n8nclone.workflow_format.n8n_json.export]
// r[impl n8nclone.workflow_validation.n8n_json.export_limits]
// r[impl n8nclone.credentials.n8n_json.placeholders]
pub fn export_workflow_to_n8n_json(workflow: &Workflow) -> N8nExportResult {
    let mut diagnostics = Vec::new();
    let mut entries = Vec::new();
    let mut nodes = Vec::new();
    let mut names_by_node_id = HashMap::new();

    for node in workflow.graph.nodes() {
        let source_id = node.id.0.to_string();
        let Some(config) = workflow.node_config(node.id) else {
            diagnostics.push(export_unsupported_node_diagnostic(node.id, &node.label, "missing node config"));
            continue;
        };
        match export_node_to_n8n_json(workflow, node.id, &node.label, node.x, node.y, config) {
            Ok(value) => {
                nodes.push(value);
                names_by_node_id.insert(node.id, node.label.clone());
                entries.push(N8nMappingEntry {
                    source_id,
                    source_name: node.label.clone(),
                    target_node_id: Some(node.id.0),
                    status: N8nMappingStatus::Mapped,
                    diagnostics: Vec::new(),
                    unsupported_fields: Vec::new(),
                    credential_placeholders: Vec::new(),
                });
            }
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    let connections = export_connections(workflow, &names_by_node_id);
    let report = N8nMappingReport {
        workflow_name: workflow.meta.name.clone(),
        entries,
        diagnostics,
    };
    if !report.diagnostics.is_empty() {
        return N8nExportResult { json: None, report };
    }

    let json = serde_json::json!({
        "name": &workflow.meta.name,
        "nodes": nodes,
        "connections": connections,
    });
    N8nExportResult {
        json: Some(json),
        report,
    }
}

fn export_node_to_n8n_json(
    workflow: &Workflow,
    node_id: crate::engine::workflow::NodeId,
    label: &str,
    x: i32,
    y: i32,
    config: &NodeConfig,
) -> Result<JsonValue, N8nDiagnostic> {
    let (kind, parameters) = match config {
        NodeConfig::Shell { command, .. } => (N8N_EXECUTE_COMMAND_TYPE, serde_json::json!({ "command": command })),
        NodeConfig::Http { method, url, body, .. } => {
            let mut parameters = serde_json::json!({ "method": method, "url": url });
            if let Some(body) = body {
                parameters["body"] = JsonValue::String(body.clone());
            }
            (N8N_HTTP_REQUEST_TYPE, parameters)
        }
        NodeConfig::File { .. }
        | NodeConfig::ImageInput { .. }
        | NodeConfig::Agent(_)
        | NodeConfig::Condition(_)
        | NodeConfig::Merge(_)
        | NodeConfig::Subworkflow(_)
        | NodeConfig::Approval(_) => {
            return Err(export_unsupported_node_diagnostic(node_id, label, "unsupported node kind"));
        }
    };
    let credentials = export_credentials(workflow.node_credentials(node_id));
    Ok(serde_json::json!({
        "id": format!("{N8N_NODE_ID_PREFIX}{}", node_id.0),
        "name": label,
        "type": kind,
        "position": [x, y],
        "parameters": parameters,
        "credentials": credentials,
    }))
}

fn export_credentials(credentials: Option<&NodeCredentialRefs>) -> JsonValue {
    let Some(credentials) = credentials else {
        return serde_json::json!({});
    };
    let mut exported = serde_json::Map::new();
    for (env_name, secret_ref) in &credentials.env {
        exported.insert(
            env_name.clone(),
            serde_json::json!({
                "id": secret_ref.identity(),
                "name": secret_ref.identity(),
            }),
        );
    }
    JsonValue::Object(exported)
}

fn export_connections(
    workflow: &Workflow,
    names_by_node_id: &HashMap<crate::engine::workflow::NodeId, String>,
) -> JsonValue {
    let mut connections = serde_json::Map::new();
    for edge in workflow.graph.edges() {
        let Some(source_node_id) = workflow.graph.port_owner(edge.source) else {
            continue;
        };
        let Some(target_node_id) = workflow.graph.port_owner(edge.target) else {
            continue;
        };
        let Some(source_name) = names_by_node_id.get(&source_node_id) else {
            continue;
        };
        let Some(target_name) = names_by_node_id.get(&target_node_id) else {
            continue;
        };
        let entry = connections
            .entry(source_name.clone())
            .or_insert_with(|| serde_json::json!({ CONNECTION_MAIN: [[]] }));
        if let Some(group) = entry
            .get_mut(CONNECTION_MAIN)
            .and_then(JsonValue::as_array_mut)
            .and_then(|groups| groups.get_mut(REQUIRED_CONNECTION_INDEX))
            .and_then(JsonValue::as_array_mut)
        {
            group.push(
                serde_json::json!({ "node": target_name, "type": CONNECTION_MAIN, "index": REQUIRED_CONNECTION_INDEX }),
            );
        }
    }
    JsonValue::Object(connections)
}

pub fn report_to_pretty_json(report: &N8nMappingReport) -> String {
    serde_json::to_string_pretty(report).expect("n8n mapping report serializes")
}

pub fn n8n_json_to_pretty_string(value: &JsonValue) -> String {
    serde_json::to_string_pretty(value).expect("n8n workflow JSON serializes")
}

fn string_parameter<'a>(parameters: &'a JsonValue, key: &str) -> Option<&'a str> {
    parameters.get(key).and_then(JsonValue::as_str)
}

fn source_id(node: &N8nNodeJson) -> String {
    node.id.clone().unwrap_or_else(|| node.name.clone())
}

fn failed_import_report(workflow_name: &str, diagnostic: N8nDiagnostic) -> N8nImportResult {
    N8nImportResult {
        workflow: None,
        report: N8nMappingReport {
            workflow_name: workflow_name.to_string(),
            entries: Vec::new(),
            diagnostics: vec![diagnostic],
        },
    }
}

fn malformed_json_diagnostic(message: String) -> N8nDiagnostic {
    N8nDiagnostic {
        code: N8nDiagnosticCode::MalformedJson,
        source_id: None,
        source_name: None,
        message,
    }
}

fn missing_field_diagnostic(field: &str) -> N8nDiagnostic {
    N8nDiagnostic {
        code: N8nDiagnosticCode::MissingWorkflowField,
        source_id: None,
        source_name: None,
        message: format!("n8n workflow is missing required field '{field}'"),
    }
}

fn plaintext_workflow_credentials_diagnostic() -> N8nDiagnostic {
    N8nDiagnostic {
        code: N8nDiagnosticCode::PlaintextCredentialMaterial,
        source_id: None,
        source_name: None,
        message: "n8n workflow-level credential material is not imported".to_string(),
    }
}

fn plaintext_node_credentials_diagnostic(node: &N8nNodeJson) -> N8nDiagnostic {
    N8nDiagnostic {
        code: N8nDiagnosticCode::PlaintextCredentialMaterial,
        source_id: Some(source_id(node)),
        source_name: Some(node.name.clone()),
        message: "n8n credential material must be an opaque id/name reference".to_string(),
    }
}

fn connection_diagnostic(source_name: &str, message: &str) -> N8nDiagnostic {
    N8nDiagnostic {
        code: N8nDiagnosticCode::MalformedConnection,
        source_id: None,
        source_name: Some(source_name.to_string()),
        message: message.to_string(),
    }
}

fn export_unsupported_node_diagnostic(
    node_id: crate::engine::workflow::NodeId,
    node_label: &str,
    reason: &str,
) -> N8nDiagnostic {
    N8nDiagnostic {
        code: N8nDiagnosticCode::UnsupportedExportNode,
        source_id: Some(node_id.0.to_string()),
        source_name: Some(node_label.to_string()),
        message: reason.to_string(),
    }
}
