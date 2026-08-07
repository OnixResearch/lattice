//! Nickel-based configuration layer.
//!
//! Three Nickel roles:
//!  1. **App config** — global settings (data dir, default timeouts, etc.)
//!  2. **Workflow definitions** — import/export workflows as .ncl files.
//!  3. **Node templates** — reusable node type definitions in Nickel.

use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::agent_node::AgentRuntimeConfig;

pub mod identity;

const DEFAULT_SHELL_TIMEOUT_SECS: u64 = 60;
const DEFAULT_MAX_PARALLELISM: usize = 4;
const DEFAULT_SHELL: &str = "sh";

/// Application-level configuration loaded from Nickel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Directory for redb database and other state.
    pub data_dir: PathBuf,
    /// Default shell command timeout in seconds.
    pub default_timeout_secs: u64,
    /// Shell to use for commands (default: "sh").
    pub shell: String,
    /// Max parallel node executions.
    pub max_parallelism: usize,
    /// Production command and OpenAI-compatible agent runtime configuration.
    #[serde(default)]
    pub agent_runtime: AgentRuntimeConfig,
    /// Shared stdin, capture, command-agent deadline, polling, and teardown bounds.
    #[serde(default)]
    pub process_limits: crate::process::core::LimitConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = default_data_dir().unwrap_or_else(|| PathBuf::from(identity::CANONICAL_DATA_DIR_NAME));
        Self {
            data_dir,
            default_timeout_secs: DEFAULT_SHELL_TIMEOUT_SECS,
            shell: DEFAULT_SHELL.into(),
            max_parallelism: DEFAULT_MAX_PARALLELISM,
            agent_runtime: AgentRuntimeConfig::default(),
            process_limits: crate::process::core::LimitConfig::default(),
        }
    }
}

impl AppConfig {
    /// Derive the explicit process runtime consumed by every executor entry point.
    pub fn effective_agent_runtime(&self) -> AgentRuntimeConfig {
        let mut runtime = self.agent_runtime.clone();
        runtime.local_process = crate::process::core::RuntimeConfig {
            shell_program: self.shell.clone(),
            limits: self.process_limits.clone(),
        };
        runtime
    }
}

/// Imperative shell: collect environment and filesystem facts for the default data directory.
fn default_data_dir() -> Option<PathBuf> {
    let home_dir = home::home_dir();
    let has_legacy_default = home_dir.as_ref().is_some_and(|home| home.join(identity::LEGACY_DATA_DIR_NAME).exists());
    identity::select_data_dir(identity::DataDirFacts {
        canonical_override: read_data_dir_env(identity::CANONICAL_DATA_DIR_ENV),
        legacy_override: read_data_dir_env(identity::LEGACY_DATA_DIR_ENV),
        home_dir,
        has_legacy_default,
    })
}

fn read_data_dir_env(name: &str) -> Option<PathBuf> {
    std::env::var(name).ok().map(PathBuf::from)
}

/// Imperative shell: collect filesystem facts for the configured store directory.
pub fn store_path(data_dir: &Path) -> PathBuf {
    let canonical_path = data_dir.join(identity::CANONICAL_STORE_FILE_NAME);
    let legacy_path = data_dir.join(identity::LEGACY_STORE_FILE_NAME);
    identity::select_store_path(identity::StorePathFacts {
        has_canonical_store: canonical_path.exists(),
        has_legacy_store: legacy_path.exists(),
        canonical_path,
        legacy_path,
    })
}

/// A node template defined in Nickel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeTemplate {
    pub name: String,
    pub description: String,
    pub kind: String,
    /// Default command (for shell nodes).
    pub command: Option<String>,
    #[serde(default)]
    pub parameters: Vec<TemplateParameter>,
    pub input_ports: Vec<PortTemplate>,
    pub output_ports: Vec<PortTemplate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub type_tag: TemplateParameterType,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub default: Option<TemplateParameterDefault>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateParameterType {
    String,
    Number,
    Bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TemplateParameterDefault {
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateParameterValue {
    String(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateRenderError {
    DuplicateParameter(String),
    EmptyParameterName,
    InvalidDefaultType(String),
    InvalidNumberRange(String),
    MissingRequired(String),
    InvalidNumber { name: String, value: String },
    InvalidBool { name: String, value: String },
    NumberBelowMinimum { name: String, min: String },
    NumberAboveMaximum { name: String, max: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateDiagnostic {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeTemplateLoadResult {
    pub templates: Vec<NodeTemplate>,
    pub diagnostics: Vec<TemplateDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortTemplate {
    pub label: String,
    pub type_tag: String,
}

impl fmt::Display for TemplateRenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateRenderError::DuplicateParameter(name) => write!(f, "duplicate template parameter '{name}'"),
            TemplateRenderError::EmptyParameterName => write!(f, "template parameter name cannot be empty"),
            TemplateRenderError::InvalidDefaultType(name) => {
                write!(f, "default value type does not match parameter '{name}'")
            }
            TemplateRenderError::InvalidNumberRange(name) => {
                write!(f, "minimum exceeds maximum for parameter '{name}'")
            }
            TemplateRenderError::MissingRequired(name) => write!(f, "missing required template parameter '{name}'"),
            TemplateRenderError::InvalidNumber { name, value } => {
                write!(f, "parameter '{name}' expects a number, got '{value}'")
            }
            TemplateRenderError::InvalidBool { name, value } => {
                write!(f, "parameter '{name}' expects true or false, got '{value}'")
            }
            TemplateRenderError::NumberBelowMinimum { name, min } => {
                write!(f, "parameter '{name}' must be at least {min}")
            }
            TemplateRenderError::NumberAboveMaximum { name, max } => {
                write!(f, "parameter '{name}' must be at most {max}")
            }
        }
    }
}

impl TemplateParameterDefault {
    pub fn as_raw_string(&self) -> String {
        match self {
            TemplateParameterDefault::String(value) => value.clone(),
            TemplateParameterDefault::Number(value) => value.to_string(),
            TemplateParameterDefault::Bool(value) => value.to_string(),
        }
    }
}

impl TemplateParameterValue {
    fn as_command_string(&self) -> String {
        match self {
            TemplateParameterValue::String(value) => value.clone(),
            TemplateParameterValue::Number(value) => value.to_string(),
            TemplateParameterValue::Bool(value) => value.to_string(),
        }
    }
}

/// Validate a template's parameter schema without touching I/O.
pub fn validate_template_parameters(template: &NodeTemplate) -> Result<(), TemplateRenderError> {
    let mut seen = std::collections::HashSet::new();
    for parameter in &template.parameters {
        if parameter.name.trim().is_empty() {
            return Err(TemplateRenderError::EmptyParameterName);
        }
        if !seen.insert(parameter.name.clone()) {
            return Err(TemplateRenderError::DuplicateParameter(parameter.name.clone()));
        }
        if !default_matches_type(parameter) {
            return Err(TemplateRenderError::InvalidDefaultType(parameter.name.clone()));
        }
        if let (Some(min), Some(max)) = (parameter.min, parameter.max)
            && min > max
        {
            return Err(TemplateRenderError::InvalidNumberRange(parameter.name.clone()));
        }
        if let Some(default) = &parameter.default
            && let TemplateParameterDefault::Number(value) = default
        {
            validate_number_constraints(parameter, *value)?;
        }
    }
    Ok(())
}

/// Render a shell command from raw parameter values after schema and value validation.
pub fn render_template_command(
    template: &NodeTemplate,
    raw_values: &HashMap<String, String>,
) -> Result<String, TemplateRenderError> {
    validate_template_parameters(template)?;
    let values = validated_template_values(template, raw_values)?;
    let mut command = template.command.clone().unwrap_or_default();
    for parameter in &template.parameters {
        let placeholder = format!("${{{}}}", parameter.name);
        let value = values
            .get(&parameter.name)
            .expect("validated values must contain every declared parameter")
            .as_command_string();
        command = command.replace(&placeholder, &value);
    }
    Ok(command)
}

/// Parse, type-check, and default raw template parameter values.
pub fn validated_template_values(
    template: &NodeTemplate,
    raw_values: &HashMap<String, String>,
) -> Result<HashMap<String, TemplateParameterValue>, TemplateRenderError> {
    let mut parsed = HashMap::new();
    for parameter in &template.parameters {
        let raw = raw_values.get(&parameter.name).map(|value| value.trim()).filter(|value| !value.is_empty());
        let value = match raw {
            Some(value) => parse_parameter_value(parameter, value)?,
            None => default_parameter_value(parameter)?,
        };
        parsed.insert(parameter.name.clone(), value);
    }
    Ok(parsed)
}

fn default_matches_type(parameter: &TemplateParameter) -> bool {
    matches!(
        (&parameter.type_tag, &parameter.default),
        (_, None)
            | (TemplateParameterType::String, Some(TemplateParameterDefault::String(_)))
            | (TemplateParameterType::Number, Some(TemplateParameterDefault::Number(_)))
            | (TemplateParameterType::Bool, Some(TemplateParameterDefault::Bool(_)))
    )
}

fn parse_parameter_value(
    parameter: &TemplateParameter,
    raw: &str,
) -> Result<TemplateParameterValue, TemplateRenderError> {
    match parameter.type_tag {
        TemplateParameterType::String => Ok(TemplateParameterValue::String(raw.to_string())),
        TemplateParameterType::Number => {
            let value = raw.parse::<f64>().map_err(|_| TemplateRenderError::InvalidNumber {
                name: parameter.name.clone(),
                value: raw.to_string(),
            })?;
            validate_number_constraints(parameter, value)?;
            Ok(TemplateParameterValue::Number(value))
        }
        TemplateParameterType::Bool => match raw {
            "true" => Ok(TemplateParameterValue::Bool(true)),
            "false" => Ok(TemplateParameterValue::Bool(false)),
            _ => Err(TemplateRenderError::InvalidBool {
                name: parameter.name.clone(),
                value: raw.to_string(),
            }),
        },
    }
}

fn default_parameter_value(parameter: &TemplateParameter) -> Result<TemplateParameterValue, TemplateRenderError> {
    match &parameter.default {
        Some(TemplateParameterDefault::String(value)) => Ok(TemplateParameterValue::String(value.clone())),
        Some(TemplateParameterDefault::Number(value)) => {
            validate_number_constraints(parameter, *value)?;
            Ok(TemplateParameterValue::Number(*value))
        }
        Some(TemplateParameterDefault::Bool(value)) => Ok(TemplateParameterValue::Bool(*value)),
        None if parameter.required => Err(TemplateRenderError::MissingRequired(parameter.name.clone())),
        None => Ok(TemplateParameterValue::String(String::new())),
    }
}

fn validate_number_constraints(parameter: &TemplateParameter, value: f64) -> Result<(), TemplateRenderError> {
    if let Some(min) = parameter.min
        && value < min
    {
        return Err(TemplateRenderError::NumberBelowMinimum {
            name: parameter.name.clone(),
            min: min.to_string(),
        });
    }
    if let Some(max) = parameter.max
        && value > max
    {
        return Err(TemplateRenderError::NumberAboveMaximum {
            name: parameter.name.clone(),
            max: max.to_string(),
        });
    }
    Ok(())
}

/// Load app config from a Nickel file. Falls back to defaults on any error.
pub fn load_app_config(path: &Path) -> AppConfig {
    if !path.exists() {
        return AppConfig::default();
    }

    match load_nickel_as_json(path) {
        Ok(json) => match serde_json::from_value(json) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "invalid config, using defaults");
                AppConfig::default()
            }
        },
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to load config, using defaults");
            AppConfig::default()
        }
    }
}

/// Load node templates from a directory of .ncl files.
pub fn load_node_templates(dir: &Path) -> Vec<NodeTemplate> {
    load_node_templates_with_diagnostics(dir).templates
}

/// Load node templates and collect diagnostics for invalid template files.
pub fn load_node_templates_with_diagnostics(dir: &Path) -> NodeTemplateLoadResult {
    let mut templates = Vec::new();
    let mut diagnostics = Vec::new();

    if !dir.exists() {
        return NodeTemplateLoadResult { templates, diagnostics };
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            diagnostics.push(TemplateDiagnostic {
                path: dir.to_path_buf(),
                message: format!("failed to read template directory: {error}"),
            });
            return NodeTemplateLoadResult { templates, diagnostics };
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "ncl") {
            match load_nickel_as_json(&path) {
                Ok(json) => match serde_json::from_value::<NodeTemplate>(json) {
                    Ok(template) => match validate_template_parameters(&template) {
                        Ok(()) => templates.push(template),
                        Err(error) => {
                            tracing::warn!(path = %path.display(), error = %error, "invalid template parameters");
                            diagnostics.push(TemplateDiagnostic {
                                path: path.clone(),
                                message: error.to_string(),
                            });
                        }
                    },
                    Err(error) => {
                        tracing::warn!(path = %path.display(), error = %error, "invalid template");
                        diagnostics.push(TemplateDiagnostic {
                            path: path.clone(),
                            message: format!("invalid template: {error}"),
                        });
                    }
                },
                Err(error) => {
                    tracing::warn!(path = %path.display(), error = %error, "failed to eval template");
                    diagnostics.push(TemplateDiagnostic {
                        path: path.clone(),
                        message: format!("failed to eval template: {error}"),
                    });
                }
            }
        }
    }

    NodeTemplateLoadResult { templates, diagnostics }
}

/// Evaluate a Nickel file and return the result as serde_json::Value.
pub(crate) fn load_nickel_as_json(path: &Path) -> Result<serde_json::Value, String> {
    crate::nickel_eval_adapter::evaluate_file(path).map_err(|error| error.to_string())
}

/// Generate a default config.ncl file.
pub fn default_config_ncl() -> &'static str {
    r#"let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let ProcessLimits = {
  stdin_max_bytes | PositiveNumber,
  stdout_max_bytes | PositiveNumber,
  stderr_max_bytes | PositiveNumber,
  command_agent_timeout_secs | PositiveNumber,
  poll_interval_millis | PositiveNumber,
  teardown_grace_millis | PositiveNumber,
} in
let stream_limit_bytes = 64 * 1024 * 1024 in
let command_agent_timeout_secs = 600 in
let process_poll_interval_millis = 10 in
let process_teardown_grace_millis = 1000 in
let process_limits = {
  stdin_max_bytes = stream_limit_bytes,
  stdout_max_bytes = stream_limit_bytes,
  stderr_max_bytes = stream_limit_bytes,
  command_agent_timeout_secs = command_agent_timeout_secs,
  poll_interval_millis = process_poll_interval_millis,
  teardown_grace_millis = process_teardown_grace_millis,
} | ProcessLimits in
{
  data_dir = "~/.lattice",
  default_timeout_secs = 60,
  shell = "sh",
  max_parallelism = 4,
  process_limits = process_limits,
  agent_runtime = {
    max_output_bytes = 64 * 1024 * 1024,
    runtimes = {
      pi = {
        command = "pi",
        args = ["--no-session", "-p", "{prompt}"],
      },
      clankers = {
        command = "clankers",
        args = ["-p", "{prompt}"],
      },
    },
    openai_compatible = {},
  },
}
"#
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultNodeTemplateFile {
    pub file_name: &'static str,
    pub contents: &'static str,
}

const COMPLEX_CUT_FIELD_TEMPLATE_FILE_NAME: &str = "complex-cut-field.ncl";
const COMPLEX_SORT_LINES_TEMPLATE_FILE_NAME: &str = "complex-sort-lines.ncl";
const COMPLEX_UNIQUE_LINES_TEMPLATE_FILE_NAME: &str = "complex-unique-lines.ncl";
const GREP_TEMPLATE_FILE_NAME: &str = "grep.ncl";
const HEAD_TEMPLATE_FILE_NAME: &str = "head.ncl";
const REPLACE_TEMPLATE_FILE_NAME: &str = "replace.ncl";
const WORD_COUNT_TEMPLATE_FILE_NAME: &str = "word-count.ncl";

const COMPLEX_CUT_FIELD_TEMPLATE_NCL: &str = include_str!("../../examples/templates/complex-cut-field.ncl");
const COMPLEX_SORT_LINES_TEMPLATE_NCL: &str = include_str!("../../examples/templates/complex-sort-lines.ncl");
const COMPLEX_UNIQUE_LINES_TEMPLATE_NCL: &str = include_str!("../../examples/templates/complex-unique-lines.ncl");

const GREP_TEMPLATE_NCL: &str = r#"{
  name = "grep",
  description = "Filter lines matching a pattern",
  kind = "shell",
  command = "grep -E '${pattern}'",
  parameters = [
    {
      name = "pattern",
      type = "string",
      description = "Extended regular expression to search for",
      required = true,
    },
  ],
  input_ports = [
    { label = "stdin", type_tag = "string" },
  ],
  output_ports = [
    { label = "stdout", type_tag = "string" },
    { label = "stderr", type_tag = "string" },
    { label = "exit",   type_tag = "exitcode" },
  ],
}
"#;

const HEAD_TEMPLATE_NCL: &str = r#"let default_line_count = 10 in
let minimum_line_count = 1 in
let maximum_line_count = 1000 in
{
  name = "head",
  description = "Take the first lines from stdin",
  kind = "shell",
  command = "head -n ${lines}",
  parameters = [
    {
      name = "lines",
      type = "number",
      description = "Number of leading lines to keep",
      default = default_line_count,
      required = false,
      min = minimum_line_count,
      max = maximum_line_count,
    },
  ],
  input_ports = [
    { label = "stdin", type_tag = "string" },
  ],
  output_ports = [
    { label = "stdout", type_tag = "string" },
    { label = "stderr", type_tag = "string" },
    { label = "exit",   type_tag = "exitcode" },
  ],
}
"#;

const REPLACE_TEMPLATE_NCL: &str = r#"{
  name = "replace",
  description = "Replace matching text in stdin with sed",
  kind = "shell",
  command = "sed 's/${pattern}/${replacement}/g'",
  parameters = [
    {
      name = "pattern",
      type = "string",
      description = "Text or regex pattern to replace",
      required = true,
    },
    {
      name = "replacement",
      type = "string",
      description = "Replacement text",
      required = true,
    },
  ],
  input_ports = [
    { label = "stdin", type_tag = "string" },
  ],
  output_ports = [
    { label = "stdout", type_tag = "string" },
    { label = "stderr", type_tag = "string" },
    { label = "exit",   type_tag = "exitcode" },
  ],
}
"#;

const WORD_COUNT_TEMPLATE_NCL: &str = r#"{
  name = "word-count",
  description = "Count lines, words, bytes, or characters from stdin",
  kind = "shell",
  command = "wc -${mode}",
  parameters = [
    {
      name = "mode",
      type = "string",
      description = "wc mode flag without the dash: l, w, c, or m",
      default = "l",
      required = false,
    },
  ],
  input_ports = [
    { label = "stdin", type_tag = "string" },
  ],
  output_ports = [
    { label = "stdout", type_tag = "string" },
    { label = "stderr", type_tag = "string" },
    { label = "exit",   type_tag = "exitcode" },
  ],
}
"#;

pub const DEFAULT_NODE_TEMPLATE_FILES: &[DefaultNodeTemplateFile] = &[
    DefaultNodeTemplateFile {
        file_name: COMPLEX_CUT_FIELD_TEMPLATE_FILE_NAME,
        contents: COMPLEX_CUT_FIELD_TEMPLATE_NCL,
    },
    DefaultNodeTemplateFile {
        file_name: COMPLEX_SORT_LINES_TEMPLATE_FILE_NAME,
        contents: COMPLEX_SORT_LINES_TEMPLATE_NCL,
    },
    DefaultNodeTemplateFile {
        file_name: COMPLEX_UNIQUE_LINES_TEMPLATE_FILE_NAME,
        contents: COMPLEX_UNIQUE_LINES_TEMPLATE_NCL,
    },
    DefaultNodeTemplateFile {
        file_name: GREP_TEMPLATE_FILE_NAME,
        contents: GREP_TEMPLATE_NCL,
    },
    DefaultNodeTemplateFile {
        file_name: HEAD_TEMPLATE_FILE_NAME,
        contents: HEAD_TEMPLATE_NCL,
    },
    DefaultNodeTemplateFile {
        file_name: REPLACE_TEMPLATE_FILE_NAME,
        contents: REPLACE_TEMPLATE_NCL,
    },
    DefaultNodeTemplateFile {
        file_name: WORD_COUNT_TEMPLATE_FILE_NAME,
        contents: WORD_COUNT_TEMPLATE_NCL,
    },
];

/// Generate an example node template.
pub fn example_template_ncl() -> &'static str {
    GREP_TEMPLATE_NCL
}
