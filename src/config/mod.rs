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
const SERIAL_MAX_PARALLELISM: u32 = 1;
const DEFAULT_SHELL: &str = "sh";

/// Application-level configuration loaded from Nickel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Directory for redb database and other state.
    pub data_dir: PathBuf,
    /// Absolute path to an operator-supplied effect authority profile. Omission denies effects.
    #[serde(default = "no_effect_authority")]
    pub effect_authority: Option<PathBuf>,
    /// Default shell command timeout in seconds.
    pub default_timeout_secs: u64,
    /// Shell to use for commands (default: "sh").
    pub shell: String,
    /// Maximum concurrent node executions. Omission preserves serial execution.
    #[serde(default = "serial_max_parallelism")]
    pub max_parallelism: u32,
    /// Named resource limits. An empty map is valid only for serial omission.
    #[serde(default = "empty_resource_lanes")]
    pub resource_lanes: std::collections::BTreeMap<String, u32>,
    /// Production command and OpenAI-compatible agent runtime configuration.
    #[serde(default)]
    pub agent_runtime: AgentRuntimeConfig,
    /// Shared stdin, capture, command-agent deadline, polling, and teardown bounds.
    #[serde(default)]
    pub process_limits: crate::process::core::LimitConfig,
    /// Public reviewed provider declarations; no secret values or deployment paths.
    #[serde(default)]
    pub credential_providers: Vec<crate::credentials::config::SecretSpecProviderConfig>,
    /// Explicit trusted host coordinates, kept separate from provider declarations.
    #[serde(default)]
    pub credential_hosts: Vec<crate::credentials::config::SecretSpecHostConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let data_dir = default_data_dir().unwrap_or_else(|| PathBuf::from(identity::CANONICAL_DATA_DIR_NAME));
        Self {
            data_dir,
            effect_authority: None,
            default_timeout_secs: DEFAULT_SHELL_TIMEOUT_SECS,
            shell: DEFAULT_SHELL.into(),
            max_parallelism: SERIAL_MAX_PARALLELISM,
            resource_lanes: std::collections::BTreeMap::new(),
            agent_runtime: AgentRuntimeConfig::default(),
            process_limits: crate::process::core::LimitConfig::default(),
            credential_providers: Vec::new(),
            credential_hosts: Vec::new(),
        }
    }
}

const fn no_effect_authority() -> Option<PathBuf> {
    None
}

impl AppConfig {
    /// Select standalone authority without falling back from an invalid selected profile.
    pub fn effect_authority_provider(
        &self,
    ) -> std::sync::Arc<dyn crate::engine::effect::execution::RunContextProvider> {
        match &self.effect_authority {
            Some(path) => {
                std::sync::Arc::new(crate::engine::effect::standalone::OperatorRunContextSource::new(path.clone()))
            }
            None => std::sync::Arc::new(crate::engine::effect::execution::UnavailableRunContextSource),
        }
    }

    /// Derive the explicit process runtime consumed by every executor entry point.
    pub fn effective_agent_runtime(&self) -> AgentRuntimeConfig {
        let mut runtime = self.agent_runtime.clone();
        runtime.local_process = crate::process::core::RuntimeConfig {
            shell_program: self.shell.clone(),
            limits: self.process_limits.clone(),
        };
        runtime
    }

    fn validated_credential_hosts(
        &self,
    ) -> Result<
        std::collections::BTreeMap<&str, &crate::credentials::config::SecretSpecHostConfig>,
        crate::credentials::config::SecretSpecDiagnostic,
    > {
        use crate::credentials::config::SecretSpecDiagnostic;

        let hosts: std::collections::BTreeMap<_, _> =
            self.credential_hosts.iter().map(|host| (host.provider_id.as_str(), host)).collect();
        let mut provider_ids = std::collections::BTreeSet::new();
        if hosts.len() != self.credential_hosts.len() || hosts.len() != self.credential_providers.len() {
            return Err(SecretSpecDiagnostic::InvalidConfiguration);
        }
        for provider in &self.credential_providers {
            provider.validate()?;
            if provider.id == "env" || !provider_ids.insert(provider.id.as_str()) {
                return Err(SecretSpecDiagnostic::InvalidConfiguration);
            }
            hosts.get(provider.id.as_str()).ok_or(SecretSpecDiagnostic::InvalidConfiguration)?.validate()?;
        }
        if !cfg!(feature = "animus-secretspec") && !provider_ids.is_empty() {
            return Err(SecretSpecDiagnostic::Unavailable);
        }
        Ok(hosts)
    }

    /// Compose explicit providers without treating their public metadata as authority.
    pub fn credential_registry(
        &self,
        additional_environment_keys: &[String],
        cancellation: std::sync::Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<crate::credentials::CredentialProviderRegistry, crate::credentials::config::SecretSpecDiagnostic> {
        use crate::credentials::config::SecretSpecDiagnostic;

        let hosts = self.validated_credential_hosts()?;
        let mut registry = crate::credentials::CredentialProviderRegistry::new();
        #[cfg(feature = "animus-secretspec")]
        for provider in &self.credential_providers {
            let host = hosts.get(provider.id.as_str()).ok_or(SecretSpecDiagnostic::InvalidConfiguration)?;
            let selected =
                crate::credentials::secretspec::build_provider(provider.clone(), host, cancellation.clone())?;
            registry
                .insert_provider(
                    crate::credentials::CredentialProviderMetadata {
                        id: provider.id.clone(),
                        display_name: provider.id.clone(),
                        capabilities: crate::credentials::CredentialProviderCapabilities {
                            supports_fields: true,
                            supports_listing: false,
                            supports_test: false,
                            supports_write: false,
                        },
                    },
                    Box::new(selected),
                )
                .map_err(|_| SecretSpecDiagnostic::InvalidConfiguration)?;
        }
        #[cfg(not(feature = "animus-secretspec"))]
        let _ = (cancellation, hosts);
        let environment_keys: Vec<_> = self
            .agent_runtime
            .openai_compatible
            .values()
            .filter_map(|runtime| runtime.api_key_env.as_ref())
            .chain(additional_environment_keys.iter())
            .cloned()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        registry
            .insert_provider(
                crate::credentials::CredentialProviderMetadata {
                    id: "env".to_owned(),
                    display_name: "Environment".to_owned(),
                    capabilities: crate::credentials::CredentialProviderCapabilities::read_only_without_fields(),
                },
                Box::new(crate::credentials::EnvironmentCredentialProvider::from_current_env(&environment_keys)),
            )
            .map_err(|_| SecretSpecDiagnostic::InvalidConfiguration)?;
        Ok(registry)
    }

    pub fn frontier_capacity_policy(
        &self,
    ) -> Result<lattice_core::scheduling::CapacityPolicy, lattice_core::scheduling::PolicyError> {
        crate::engine::frontier::capacity_policy(self.max_parallelism, &self.resource_lanes)
    }
}

const fn serial_max_parallelism() -> u32 {
    SERIAL_MAX_PARALLELISM
}

fn empty_resource_lanes() -> std::collections::BTreeMap<String, u32> {
    std::collections::BTreeMap::new()
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
    #[serde(default = "parameter_is_optional")]
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

const fn parameter_is_optional() -> bool {
    false
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

/// Load an existing selected config without silently discarding invalid settings.
/// A missing default config preserves the existing unconfigured behavior.
pub fn load_app_config(path: &Path) -> crate::error::Result<AppConfig> {
    let config_error = |message: &str| crate::error::Error::Config {
        message: message.to_owned(),
    };
    if !path.try_exists().map_err(|_| config_error("app_config_unavailable"))? {
        return Ok(AppConfig::default());
    }

    let json = load_nickel_as_json(path).map_err(|_| config_error("app_config_evaluation_failed"))?;
    let config: AppConfig = serde_json::from_value(json).map_err(|_| config_error("app_config_invalid_shape"))?;
    config.validated_credential_hosts().map_err(|error| config_error(error.code()))?;
    Ok(config)
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
    crate::nickel::evaluate_file(path).map_err(|error| error.to_string())
}

/// Generate a default config.ncl file.
#[allow(unknown_lints)]
#[allow(
    path_segment_repetition,
    reason = "the public API names the exact generated config.ncl artifact"
)]
pub fn default_config_ncl() -> &'static str {
    include_str!("default.ncl")
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
