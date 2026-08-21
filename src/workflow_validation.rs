//! Pure workflow validation diagnostics.

use serde::Deserialize;
use serde::Serialize;

use crate::credentials;
use crate::engine::runner;
use crate::engine::workflow::Node;
use crate::engine::workflow::NodeConfig;
use crate::engine::workflow::PortDirection;
use crate::engine::workflow::PortId;
use crate::engine::workflow::Workflow;
use crate::node_kind_provider;
use crate::node_kind_provider::NodeKindValidationCode;
use crate::triggers;

const MAX_EXPRESSION_DIAGNOSTICS_PER_CONFIG: usize = 18;

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
    UnknownCredentialProvider,
    UnsupportedCredentialField,
    InvalidHttpMethod,
    InvalidHttpUrl,
    UnsafeFilePath,
    UnsafeImagePath,
    UnsupportedImageFormat,
    InvalidImageByteBudget,
    EmptyAgentPrompt,
    InvalidOperationConfig,
    InvalidConditionConfig,
    InvalidMergeConfig,
    InvalidSubworkflowConfig,
    InvalidApprovalConfig,
    DuplicateApprovalGate,
    InvalidTriggerConfig,
    InvalidExpressionSyntax,
    ExpressionProducerMissing,
    ExpressionOutputPortMissing,
    ExpressionSelfReference,
    UnsupportedExpressionField,
    RunnerCycleDetected,
    RunnerMissingRuntimeBinding,
    RunnerUnsupportedRuntimePolicy,
    RunnerRuntimeOutputPortMissing,
    RunnerRuntimeOutputTypeMismatch,
    RunnerRuntimeFailure,
    RunnerTerminalStateMutation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: WorkflowDiagnosticCode,
    pub node_id: Option<crate::engine::workflow::NodeId>,
    pub port_id: Option<PortId>,
    pub message: String,
}

// r[impl n8nclone.workflow_validation.diagnostics.core]
// r[impl n8nclone.workflow_validation.diagnostics.negative]
pub fn validate_workflow(workflow: &Workflow) -> Vec<WorkflowDiagnostic> {
    let mut diagnostics = base_workflow_diagnostics(workflow);
    validate_credentials(workflow, &mut diagnostics);
    diagnostics
}

// r[impl n8nclone.workflow_validation.credentials.providers]
pub fn validate_workflow_with_credential_registry(
    workflow: &Workflow,
    registry: &credentials::CredentialProviderRegistry,
) -> Vec<WorkflowDiagnostic> {
    let mut diagnostics = base_workflow_diagnostics(workflow);
    validate_credentials_with_registry(workflow, registry, &mut diagnostics);
    diagnostics
}

fn base_workflow_diagnostics(workflow: &Workflow) -> Vec<WorkflowDiagnostic> {
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
    diagnostics.extend(expression_dependency_diagnostics(workflow));
    diagnostics.extend(validate_approval_topology(workflow));
    validate_edges(workflow, &mut diagnostics);
    validate_control_flow_topology(workflow, &mut diagnostics);
    validate_runner_admission(workflow, &mut diagnostics);
    validate_triggers(workflow, &mut diagnostics);

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
            Some(config) => diagnostics.extend(provider_diagnostics(node.id, &node.label, config)),
            None => diagnostics.push(fatal(
                WorkflowDiagnosticCode::MissingNodeConfig,
                Some(node.id),
                None,
                format!("node '{}' has no execution config", node.label),
            )),
        }
    }

    for node_id in workflow.configs.keys() {
        let node_id = crate::engine::workflow::NodeId(*node_id);
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

// r[impl n8nclone.scheduling.triggers.model]
fn validate_triggers(workflow: &Workflow, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    let workflow_names = std::collections::HashSet::from([workflow.meta.name.clone()]);
    for trigger in &workflow.triggers {
        diagnostics.extend(triggers::validate_trigger_definition(trigger, &workflow_names).into_iter().map(
            |trigger_diagnostic| {
                fatal(
                    WorkflowDiagnosticCode::InvalidTriggerConfig,
                    None,
                    None,
                    format!("trigger {}: {}", trigger_diagnostic.trigger_id, trigger_diagnostic.message),
                )
            },
        ));
    }
}

fn provider_diagnostics(
    node_id: crate::engine::workflow::NodeId,
    node_label: &str,
    config: &NodeConfig,
) -> Vec<WorkflowDiagnostic> {
    node_kind_provider::provider_for_config(config)
        .validate_config(node_label, config)
        .into_iter()
        .map(|issue| fatal(workflow_code_from_provider_code(issue.code), Some(node_id), None, issue.message))
        .collect()
}

fn workflow_code_from_provider_code(code: NodeKindValidationCode) -> WorkflowDiagnosticCode {
    match code {
        NodeKindValidationCode::EmptyShellCommand => WorkflowDiagnosticCode::EmptyShellCommand,
        NodeKindValidationCode::InvalidHttpMethod => WorkflowDiagnosticCode::InvalidHttpMethod,
        NodeKindValidationCode::InvalidHttpUrl => WorkflowDiagnosticCode::InvalidHttpUrl,
        NodeKindValidationCode::UnsafeFilePath => WorkflowDiagnosticCode::UnsafeFilePath,
        NodeKindValidationCode::UnsafeImagePath => WorkflowDiagnosticCode::UnsafeImagePath,
        NodeKindValidationCode::UnsupportedImageFormat => WorkflowDiagnosticCode::UnsupportedImageFormat,
        NodeKindValidationCode::InvalidImageByteBudget => WorkflowDiagnosticCode::InvalidImageByteBudget,
        NodeKindValidationCode::EmptyAgentPrompt => WorkflowDiagnosticCode::EmptyAgentPrompt,
        NodeKindValidationCode::InvalidOperation => WorkflowDiagnosticCode::InvalidOperationConfig,
        NodeKindValidationCode::InvalidCondition => WorkflowDiagnosticCode::InvalidConditionConfig,
        NodeKindValidationCode::InvalidMerge => WorkflowDiagnosticCode::InvalidMergeConfig,
        NodeKindValidationCode::InvalidSubworkflow => WorkflowDiagnosticCode::InvalidSubworkflowConfig,
        NodeKindValidationCode::InvalidApproval => WorkflowDiagnosticCode::InvalidApprovalConfig,
    }
}

// r[impl lattice.workflow_validation.explicit_dependencies.admission]
fn expression_dependency_diagnostics(workflow: &Workflow) -> Vec<WorkflowDiagnostic> {
    let Err(issues) = crate::expression_dependencies::workflow_projection(workflow) else {
        return Vec::new();
    };
    let diagnostics = issues
        .into_iter()
        .map(|issue| {
            let code = match issue.code {
                crate::expression_dependencies::ExpressionDependencyIssueCode::InvalidSyntax => {
                    WorkflowDiagnosticCode::InvalidExpressionSyntax
                }
                crate::expression_dependencies::ExpressionDependencyIssueCode::MissingProducerNode => {
                    WorkflowDiagnosticCode::ExpressionProducerMissing
                }
                crate::expression_dependencies::ExpressionDependencyIssueCode::MissingProducerOutputPort => {
                    WorkflowDiagnosticCode::ExpressionOutputPortMissing
                }
                crate::expression_dependencies::ExpressionDependencyIssueCode::SelfReference => {
                    WorkflowDiagnosticCode::ExpressionSelfReference
                }
                crate::expression_dependencies::ExpressionDependencyIssueCode::UnsupportedField => {
                    WorkflowDiagnosticCode::UnsupportedExpressionField
                }
            };
            fatal(
                code,
                Some(crate::engine::workflow::NodeId(issue.consumer_node)),
                issue.producer_output_port.map(|port| PortId(port.0)),
                format!("{}: {}", issue.config_field_path, issue.message),
            )
        })
        .collect::<Vec<_>>();
    let maximum_diagnostics = workflow.configs.len().saturating_mul(MAX_EXPRESSION_DIAGNOSTICS_PER_CONFIG);
    debug_assert!(diagnostics.len() <= maximum_diagnostics);
    debug_assert!(diagnostics.iter().all(|diagnostic| diagnostic.severity == DiagnosticSeverity::Fatal));
    diagnostics
}

fn validate_approval_topology(workflow: &Workflow) -> Vec<WorkflowDiagnostic> {
    let mut diagnostics = Vec::with_capacity(workflow.graph.node_count());
    let mut gate_ids = std::collections::BTreeSet::new();
    for node in workflow.graph.nodes() {
        let Some(NodeConfig::Approval(config)) = workflow.node_config(node.id) else {
            continue;
        };
        if !gate_ids.insert(config.gate_id.as_str()) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::DuplicateApprovalGate,
                Some(node.id),
                None,
                "approval gate identity must be unique within the workflow".to_string(),
            ));
        }
        let input_labels =
            node.input_ports.iter().map(|port| port.label.as_str()).collect::<std::collections::BTreeSet<_>>();
        if config.selected_input_ports.iter().any(|label| !input_labels.contains(label.as_str())) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::InvalidApprovalConfig,
                Some(node.id),
                None,
                "approval selected inputs must name declared node input ports".to_string(),
            ));
        }
    }
    debug_assert!(gate_ids.len() <= workflow.graph.node_count());
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.message.is_empty()));
    diagnostics
}

fn validate_credentials(workflow: &Workflow, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    for (node_id, refs) in &workflow.credentials {
        let node_id = crate::engine::workflow::NodeId(*node_id);
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

// r[impl n8nclone.workflow_validation.credentials.providers]
fn validate_credentials_with_registry(
    workflow: &Workflow,
    registry: &credentials::CredentialProviderRegistry,
    diagnostics: &mut Vec<WorkflowDiagnostic>,
) {
    validate_credentials(workflow, diagnostics);
    for (node_id, refs) in &workflow.credentials {
        let node_id = crate::engine::workflow::NodeId(*node_id);
        if workflow.graph.node(node_id).is_none() {
            continue;
        }
        for secret_ref in refs.env.values() {
            if let Err(error) = registry.validate_secret_ref(secret_ref) {
                let code = match error {
                    credentials::CredentialError::UnknownProvider { .. } => {
                        WorkflowDiagnosticCode::UnknownCredentialProvider
                    }
                    credentials::CredentialError::UnsupportedFieldAccess { .. } => {
                        WorkflowDiagnosticCode::UnsupportedCredentialField
                    }
                    _ => WorkflowDiagnosticCode::MalformedSecretRef,
                };
                diagnostics.push(fatal(code, Some(node_id), None, error.to_string()));
            }
        }
    }
}

// r[impl n8nclone.workflow_validation.control_flow]
// r[impl n8nclone.workflow_validation.subworkflows]
fn validate_control_flow_topology(workflow: &Workflow, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    for node in workflow.graph.nodes() {
        match workflow.node_config(node.id) {
            Some(NodeConfig::Condition(config)) => validate_condition_topology(node, config, diagnostics),
            Some(NodeConfig::Merge(_)) => validate_merge_topology(workflow, node, diagnostics),
            Some(NodeConfig::Subworkflow(config)) => validate_subworkflow_topology(workflow, node, config, diagnostics),
            Some(
                NodeConfig::Shell { .. }
                | NodeConfig::Http { .. }
                | NodeConfig::File { .. }
                | NodeConfig::ImageInput { .. }
                | NodeConfig::Agent(_)
                | NodeConfig::Operation(_)
                | NodeConfig::Approval(_),
            )
            | None => {}
        }
    }
}

fn validate_condition_topology(
    node: &Node,
    config: &crate::control_flow::ConditionNodeConfig,
    diagnostics: &mut Vec<WorkflowDiagnostic>,
) {
    let branch_labels = config
        .branches
        .iter()
        .map(|branch| branch.label.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for branch in &config.branches {
        if !node.output_ports.iter().any(|port| port.label == branch.label) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::InvalidConditionConfig,
                Some(node.id),
                None,
                format!("condition branch '{}' has no matching output port", branch.label),
            ));
        }
    }
    for port in &node.output_ports {
        if !branch_labels.contains(port.label.as_str()) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::InvalidConditionConfig,
                Some(node.id),
                Some(port.id),
                format!("condition output port '{}' has no predicate branch", port.label),
            ));
        }
    }
}

fn validate_merge_topology(workflow: &Workflow, node: &Node, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    let Some(output_port) = node.output_ports.first() else {
        return;
    };
    debug_assert_eq!(output_port.direction, PortDirection::Output);
    debug_assert!(node.input_ports.iter().all(|port| port.direction == PortDirection::Input));
    if output_port.type_tag == "any" {
        return;
    }
    let input_port_ids = node.input_ports.iter().map(|port| port.id).collect::<Vec<_>>();
    for edge in workflow.graph.edges().iter().filter(|edge| input_port_ids.contains(&edge.target)) {
        let Some(source_port) = workflow.graph.port(edge.source) else {
            continue;
        };
        if !port_types_compatible(&source_port.type_tag, &output_port.type_tag) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::InvalidMergeConfig,
                Some(node.id),
                Some(edge.target),
                format!(
                    "merge output type '{}' is incompatible with upstream input type '{}'",
                    output_port.type_tag, source_port.type_tag,
                ),
            ));
        }
    }
}

fn validate_subworkflow_topology(
    workflow: &Workflow,
    node: &Node,
    config: &crate::control_flow::SubworkflowNodeConfig,
    diagnostics: &mut Vec<WorkflowDiagnostic>,
) {
    if config.workflow_name == workflow.meta.name {
        diagnostics.push(fatal(
            WorkflowDiagnosticCode::InvalidSubworkflowConfig,
            Some(node.id),
            None,
            format!("subworkflow '{}' recursively references the parent workflow", config.workflow_name),
        ));
    }
    for mapping in &config.input_mappings {
        if !node.input_ports.iter().any(|port| port.label == mapping.parent_input) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::InvalidSubworkflowConfig,
                Some(node.id),
                None,
                format!("subworkflow input mapping references unknown parent input '{}'", mapping.parent_input),
            ));
        }
    }
    for mapping in &config.output_mappings {
        if !node.output_ports.iter().any(|port| port.label == mapping.parent_output) {
            diagnostics.push(fatal(
                WorkflowDiagnosticCode::InvalidSubworkflowConfig,
                Some(node.id),
                None,
                format!("subworkflow output mapping references unknown parent output '{}'", mapping.parent_output),
            ));
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

// r[impl n8nclone.workflow_validation.generic_runner_admission]
// r[impl n8nclone.workflow_validation.node_kind_provider.runtime_binding]
fn validate_runner_admission(workflow: &Workflow, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    debug_assert!(diagnostics.iter().all(|diagnostic| !diagnostic.message.is_empty()));
    let build = runner::build_runner_plan(workflow);
    let Some(plan) = build.plan else {
        return;
    };
    match runner::admit_runner_plan(&plan) {
        runner::RunnerAdmission::Accepted { .. } => {}
        runner::RunnerAdmission::Rejected {
            diagnostics: runner_diagnostics,
        } => {
            diagnostics.extend(runner_diagnostics.into_iter().filter_map(workflow_diagnostic_from_runner));
        }
    }
    diagnostics.extend(
        runner::validate_runtime_bindings(&plan, &runner::builtin_runtime_registry())
            .into_iter()
            .filter_map(workflow_diagnostic_from_runner),
    );
    diagnostics
        .extend(runner::validate_runtime_policies(&plan).into_iter().filter_map(workflow_diagnostic_from_runner));
}

fn workflow_diagnostic_from_runner(diagnostic: runner::RunnerDiagnostic) -> Option<WorkflowDiagnostic> {
    debug_assert!(!diagnostic.message.is_empty());
    let code = match diagnostic.code {
        runner::RunnerDiagnosticCode::CycleDetected => WorkflowDiagnosticCode::RunnerCycleDetected,
        runner::RunnerDiagnosticCode::MissingRuntimeBinding => WorkflowDiagnosticCode::RunnerMissingRuntimeBinding,
        runner::RunnerDiagnosticCode::UnsupportedRuntimePolicy => {
            WorkflowDiagnosticCode::RunnerUnsupportedRuntimePolicy
        }
        runner::RunnerDiagnosticCode::RuntimeOutputPortMissing => {
            WorkflowDiagnosticCode::RunnerRuntimeOutputPortMissing
        }
        runner::RunnerDiagnosticCode::RuntimeOutputTypeMismatch => {
            WorkflowDiagnosticCode::RunnerRuntimeOutputTypeMismatch
        }
        runner::RunnerDiagnosticCode::RuntimeFailure => WorkflowDiagnosticCode::RunnerRuntimeFailure,
        runner::RunnerDiagnosticCode::TerminalStateMutation => WorkflowDiagnosticCode::RunnerTerminalStateMutation,
        runner::RunnerDiagnosticCode::MissingNodeConfig
        | runner::RunnerDiagnosticCode::ConfigWithoutNode
        | runner::RunnerDiagnosticCode::EdgePortMissing
        | runner::RunnerDiagnosticCode::EdgeWrongDirection
        | runner::RunnerDiagnosticCode::EdgeSelfLoop
        | runner::RunnerDiagnosticCode::EdgeIncompatibleType
        | runner::RunnerDiagnosticCode::InvalidExpressionDependency => return None,
    };
    Some(fatal(code, diagnostic.node_id, diagnostic.port_id, diagnostic.message))
}

fn port_types_compatible(source: &str, target: &str) -> bool {
    source == target || source == "any" || target == "any"
}

fn fatal(
    code: WorkflowDiagnosticCode,
    node_id: Option<crate::engine::workflow::NodeId>,
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
    node_id: Option<crate::engine::workflow::NodeId>,
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
