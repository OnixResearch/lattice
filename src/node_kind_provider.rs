//! Built-in node-kind provider metadata and validation.
//!
//! The persisted `NodeConfig` enum remains the data model; this module centralizes
//! node-kind behavior needed by validation, format handlers, and TUI metadata.

use crate::agent_node;
use crate::control_flow;
use crate::engine::workflow::NodeConfig;
use crate::file_node;
use crate::http;
use crate::image_node;

const KIND_SHELL: &str = "shell";
const KIND_HTTP: &str = "http";
const KIND_FILE: &str = "file";
const KIND_IMAGE_INPUT: &str = "image_input";
const KIND_AGENT: &str = "agent";
const KIND_CONDITION: &str = "condition";
const KIND_MERGE: &str = "merge";
const KIND_SUBWORKFLOW: &str = "subworkflow";
const KIND_APPROVAL: &str = "approval";
const DISPLAY_SHELL: &str = "Shell";
const DISPLAY_HTTP: &str = "HTTP";
const DISPLAY_FILE: &str = "File";
const DISPLAY_IMAGE_INPUT: &str = "Image Input";
const DISPLAY_AGENT: &str = "Agent";
const DISPLAY_CONDITION: &str = "Condition";
const DISPLAY_MERGE: &str = "Merge";
const DISPLAY_SUBWORKFLOW: &str = "Subworkflow";
const DISPLAY_APPROVAL: &str = "Approval";
const MIN_IMAGE_BYTE_BUDGET: usize = 1;
const PRODUCT_FILE_ROOT_COUNT: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Shell,
    Http,
    File,
    ImageInput,
    Agent,
    Condition,
    Merge,
    Subworkflow,
    Approval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKindValidationCode {
    EmptyShellCommand,
    InvalidHttpMethod,
    InvalidHttpUrl,
    UnsafeFilePath,
    UnsafeImagePath,
    UnsupportedImageFormat,
    InvalidImageByteBudget,
    EmptyAgentPrompt,
    InvalidCondition,
    InvalidMerge,
    InvalidSubworkflow,
    InvalidApproval,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeKindValidationIssue {
    pub code: NodeKindValidationCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePropertyLine {
    pub label: &'static str,
    pub value: String,
}

pub trait NodeKindProvider {
    fn kind(&self) -> NodeKind;
    fn portable_kind(&self) -> &'static str;
    fn display_label(&self) -> &'static str;
    fn property_lines(&self, config: &NodeConfig) -> Vec<NodePropertyLine>;
    fn validate_config(&self, node_label: &str, config: &NodeConfig) -> Vec<NodeKindValidationIssue>;
}

#[derive(Debug, Clone, Copy)]
struct BuiltinNodeKindProvider {
    kind: NodeKind,
}

static SHELL_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider { kind: NodeKind::Shell };
static HTTP_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider { kind: NodeKind::Http };
static FILE_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider { kind: NodeKind::File };
static IMAGE_INPUT_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider {
    kind: NodeKind::ImageInput,
};
static AGENT_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider { kind: NodeKind::Agent };
static CONDITION_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider {
    kind: NodeKind::Condition,
};
static MERGE_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider { kind: NodeKind::Merge };
static SUBWORKFLOW_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider {
    kind: NodeKind::Subworkflow,
};
static APPROVAL_PROVIDER: BuiltinNodeKindProvider = BuiltinNodeKindProvider {
    kind: NodeKind::Approval,
};

impl NodeKindProvider for BuiltinNodeKindProvider {
    fn kind(&self) -> NodeKind {
        self.kind
    }

    fn portable_kind(&self) -> &'static str {
        portable_kind_for_node_kind(self.kind)
    }

    fn display_label(&self) -> &'static str {
        display_label_for_node_kind(self.kind)
    }

    fn property_lines(&self, config: &NodeConfig) -> Vec<NodePropertyLine> {
        property_lines_for_config(config)
    }

    fn validate_config(&self, node_label: &str, config: &NodeConfig) -> Vec<NodeKindValidationIssue> {
        validate_config_for_kind(node_label, config)
    }
}

pub fn provider_for_config(config: &NodeConfig) -> &'static dyn NodeKindProvider {
    provider_for_kind(node_kind_for_config(config))
}

pub fn provider_for_portable_kind(kind: &str) -> Option<&'static dyn NodeKindProvider> {
    node_kind_for_portable_kind(kind).map(provider_for_kind)
}

pub fn node_kind_for_config(config: &NodeConfig) -> NodeKind {
    match config {
        NodeConfig::Shell { .. } => NodeKind::Shell,
        NodeConfig::Http { .. } => NodeKind::Http,
        NodeConfig::File { .. } => NodeKind::File,
        NodeConfig::ImageInput { .. } => NodeKind::ImageInput,
        NodeConfig::Agent(_) => NodeKind::Agent,
        NodeConfig::Condition(_) => NodeKind::Condition,
        NodeConfig::Merge(_) => NodeKind::Merge,
        NodeConfig::Subworkflow(_) => NodeKind::Subworkflow,
        NodeConfig::Approval(_) => NodeKind::Approval,
    }
}

pub fn portable_kind_for_config(config: &NodeConfig) -> &'static str {
    provider_for_config(config).portable_kind()
}

pub fn display_label_for_config(config: &NodeConfig) -> &'static str {
    provider_for_config(config).display_label()
}

// r[impl n8nclone.tui_ux.node_kind_provider.property_metadata]
pub fn property_lines_for_config(config: &NodeConfig) -> Vec<NodePropertyLine> {
    match config {
        NodeConfig::Shell {
            command,
            cwd,
            timeout_secs,
            pipe_stdin,
            ..
        } => shell_property_lines(command, cwd.as_deref(), *timeout_secs, *pipe_stdin),
        NodeConfig::Http {
            method,
            url,
            timeout_secs,
            ..
        } => vec![
            property_line("Type", DISPLAY_HTTP),
            property_line("Method", method),
            property_line("URL", url),
            property_line("Timeout", format!("{timeout_secs}s")),
        ],
        NodeConfig::File { operation, path, .. } => vec![
            property_line("Type", DISPLAY_FILE),
            property_line("Op", format!("{operation:?}")),
            property_line("Path", path),
        ],
        NodeConfig::ImageInput { path, max_bytes, .. } => vec![
            property_line("Type", DISPLAY_IMAGE_INPUT),
            property_line("Path", path),
            property_line("Max bytes", max_bytes.to_string()),
        ],
        NodeConfig::Agent(settings) => {
            let mut lines = vec![
                property_line("Type", DISPLAY_AGENT),
                property_line("Target", agent_node::agent_target_label(settings.target)),
                property_line("Output", settings.output_mode.as_str()),
                property_line("Prompt", &settings.prompt),
            ];
            if let Some(session) = &settings.session {
                lines.push(property_line("Session", session));
            }
            lines
        }
        NodeConfig::Condition(config) => vec![
            property_line("Type", DISPLAY_CONDITION),
            property_line(
                "Branches",
                config.branches.iter().map(|branch| branch.label.clone()).collect::<Vec<_>>().join(", "),
            ),
        ],
        NodeConfig::Merge(config) => vec![
            property_line("Type", DISPLAY_MERGE),
            property_line("Policy", format!("{:?}", config.policy).to_lowercase()),
            property_line("Inputs", config.required_inputs.join(", ")),
        ],
        NodeConfig::Subworkflow(config) => vec![
            property_line("Type", DISPLAY_SUBWORKFLOW),
            property_line("Workflow", &config.workflow_name),
        ],
        NodeConfig::Approval(config) => vec![
            property_line("Type", DISPLAY_APPROVAL),
            property_line("Gate", config.gate_id.as_str()),
            property_line("Summary", &config.safe_summary),
        ],
    }
}

fn shell_property_lines(
    command: &str,
    cwd: Option<&str>,
    timeout_secs: u64,
    pipe_stdin: bool,
) -> Vec<NodePropertyLine> {
    let mut lines = vec![property_line("Type", DISPLAY_SHELL), property_line("Cmd", command)];
    if let Some(cwd) = cwd {
        lines.push(property_line("CWD", cwd));
    }
    lines.push(property_line("Timeout", format!("{timeout_secs}s")));
    lines.push(property_line("Pipe stdin", if pipe_stdin { "yes" } else { "no" }));
    lines
}

fn valid_product_file_target(path: &str, allowed_roots: &[String]) -> bool {
    let [root_id] = allowed_roots else {
        return false;
    };
    debug_assert_eq!(allowed_roots.len(), PRODUCT_FILE_ROOT_COUNT);
    !root_id.trim().is_empty()
        && !root_id.contains('/')
        && !root_id.contains('\\')
        && cap_root_core::RootRelativePath::new(path, cap_root_core::AdmissionMode::NonEmptyRelative).is_ok()
}

fn property_line(label: &'static str, value: impl Into<String>) -> NodePropertyLine {
    NodePropertyLine {
        label,
        value: value.into(),
    }
}

// r[impl n8nclone.workflow_validation.node_kind_provider.config_validation]
// r[impl n8nclone.workflow_validation.node_kind_provider.fallback]
pub fn validate_config_for_kind(node_label: &str, config: &NodeConfig) -> Vec<NodeKindValidationIssue> {
    let mut issues = Vec::new();
    match config {
        NodeConfig::Shell { command, .. } => {
            if command.trim().is_empty() {
                issues.push(issue(
                    NodeKindValidationCode::EmptyShellCommand,
                    format!("shell node '{node_label}' has an empty command"),
                ));
            }
        }
        NodeConfig::Http { method, url, .. } => {
            if !http::validate_http_method(method) {
                issues.push(issue(
                    NodeKindValidationCode::InvalidHttpMethod,
                    format!("HTTP node '{node_label}' has an invalid method"),
                ));
            }
            if !http::validate_http_url(url) {
                issues.push(issue(
                    NodeKindValidationCode::InvalidHttpUrl,
                    format!("HTTP node '{node_label}' has an invalid URL"),
                ));
            }
        }
        NodeConfig::File {
            path, allowed_roots, ..
        } => {
            if !valid_product_file_target(path, allowed_roots)
                && let Err(error) = file_node::validate_path_allowed(path, allowed_roots)
            {
                issues.push(issue(NodeKindValidationCode::UnsafeFilePath, error.to_string()));
            }
        }
        NodeConfig::ImageInput {
            path,
            allowed_roots,
            max_bytes,
        } => {
            if let Err(error) = image_node::validate_image_path_allowed(path, allowed_roots) {
                issues.push(issue(NodeKindValidationCode::UnsafeImagePath, error.to_string()));
            }
            if let Err(error) = image_node::validate_declared_format(path) {
                issues.push(issue(NodeKindValidationCode::UnsupportedImageFormat, error.to_string()));
            }
            if *max_bytes < MIN_IMAGE_BYTE_BUDGET {
                issues.push(issue(
                    NodeKindValidationCode::InvalidImageByteBudget,
                    format!("image input node '{node_label}' has a zero byte budget"),
                ));
            }
        }
        NodeConfig::Agent(settings) => {
            if settings.prompt.trim().is_empty() {
                issues.push(issue(
                    NodeKindValidationCode::EmptyAgentPrompt,
                    format!("generic agent node '{node_label}' has an empty prompt"),
                ));
            }
        }
        NodeConfig::Condition(config) => {
            if let Err(error) = control_flow::validate_condition_config(config) {
                issues.push(issue(NodeKindValidationCode::InvalidCondition, error.to_string()));
            }
        }
        NodeConfig::Merge(config) => {
            if let Err(error) = control_flow::validate_merge_config(config) {
                issues.push(issue(NodeKindValidationCode::InvalidMerge, error.to_string()));
            }
        }
        NodeConfig::Subworkflow(config) => {
            if let Err(error) = control_flow::validate_subworkflow_config(config) {
                issues.push(issue(NodeKindValidationCode::InvalidSubworkflow, error.to_string()));
            }
        }
        NodeConfig::Approval(config) => {
            issues.extend(
                crate::workflow::gate::core::validate_node_config(config)
                    .into_iter()
                    .map(|validation| issue(NodeKindValidationCode::InvalidApproval, validation.message)),
            );
        }
    }
    issues
}

fn issue(code: NodeKindValidationCode, message: String) -> NodeKindValidationIssue {
    NodeKindValidationIssue { code, message }
}

fn node_kind_for_portable_kind(kind: &str) -> Option<NodeKind> {
    match kind {
        KIND_SHELL => Some(NodeKind::Shell),
        KIND_HTTP => Some(NodeKind::Http),
        KIND_FILE => Some(NodeKind::File),
        KIND_IMAGE_INPUT => Some(NodeKind::ImageInput),
        KIND_AGENT => Some(NodeKind::Agent),
        KIND_CONDITION => Some(NodeKind::Condition),
        KIND_MERGE => Some(NodeKind::Merge),
        KIND_SUBWORKFLOW => Some(NodeKind::Subworkflow),
        KIND_APPROVAL => Some(NodeKind::Approval),
        _ => None,
    }
}

fn provider_for_kind(kind: NodeKind) -> &'static dyn NodeKindProvider {
    match kind {
        NodeKind::Shell => &SHELL_PROVIDER,
        NodeKind::Http => &HTTP_PROVIDER,
        NodeKind::File => &FILE_PROVIDER,
        NodeKind::ImageInput => &IMAGE_INPUT_PROVIDER,
        NodeKind::Agent => &AGENT_PROVIDER,
        NodeKind::Condition => &CONDITION_PROVIDER,
        NodeKind::Merge => &MERGE_PROVIDER,
        NodeKind::Subworkflow => &SUBWORKFLOW_PROVIDER,
        NodeKind::Approval => &APPROVAL_PROVIDER,
    }
}

fn portable_kind_for_node_kind(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Shell => KIND_SHELL,
        NodeKind::Http => KIND_HTTP,
        NodeKind::File => KIND_FILE,
        NodeKind::ImageInput => KIND_IMAGE_INPUT,
        NodeKind::Agent => KIND_AGENT,
        NodeKind::Condition => KIND_CONDITION,
        NodeKind::Merge => KIND_MERGE,
        NodeKind::Subworkflow => KIND_SUBWORKFLOW,
        NodeKind::Approval => KIND_APPROVAL,
    }
}

fn display_label_for_node_kind(kind: NodeKind) -> &'static str {
    match kind {
        NodeKind::Shell => DISPLAY_SHELL,
        NodeKind::Http => DISPLAY_HTTP,
        NodeKind::File => DISPLAY_FILE,
        NodeKind::ImageInput => DISPLAY_IMAGE_INPUT,
        NodeKind::Agent => DISPLAY_AGENT,
        NodeKind::Condition => DISPLAY_CONDITION,
        NodeKind::Merge => DISPLAY_MERGE,
        NodeKind::Subworkflow => DISPLAY_SUBWORKFLOW,
        NodeKind::Approval => DISPLAY_APPROVAL,
    }
}
