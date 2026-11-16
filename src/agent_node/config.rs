const DEFAULT_PI_RUNTIME_KEY: &str = "pi";
const DEFAULT_CLANKERS_RUNTIME_KEY: &str = "clankers";
const DEFAULT_PI_COMMAND: &str = "pi";
const DEFAULT_CLANKERS_COMMAND: &str = "clankers";
const PRINT_MODE_FLAG: &str = "-p";
const NO_SESSION_FLAG: &str = "--no-session";
const PROMPT_PLACEHOLDER: &str = "{prompt}";

const DEFAULT_AGENT_OUTPUT_MAX_BYTES: usize = 67_108_864;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub target: super::Target,
    pub prompt: String,
    #[serde(default = "absent_prompt_file")]
    pub prompt_file: Option<String>,
    pub output_mode: super::OutputMode,
    pub model_hint: Option<String>,
    pub tool_policy: String,
    pub context_refs: Vec<String>,
    pub cwd: Option<String>,
    pub session: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RuntimeOptions {
    #[serde(default = "default_runtimes")]
    pub runtimes: std::collections::BTreeMap<String, CommandOptions>,
    #[serde(default = "empty_openai_runtimes")]
    pub openai_compatible: std::collections::BTreeMap<String, crate::openai::RuntimeConfig>,
    // r[impl lattice.agent_runtime.structured_input_projection]
    #[serde(default)]
    pub structured_input_projection: StructuredInputProjectionPolicy,
    #[serde(default = "default_output_max_bytes")]
    pub max_output_bytes: usize,
    #[serde(skip, default)]
    pub local_process: crate::process::core::RuntimeConfig,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuredInputProjectionPolicy {
    #[serde(default = "absent_projection")]
    pub pi: Option<crate::agent_prompt::StructuredInputProjection>,
    #[serde(default = "absent_projection")]
    pub clankers: Option<crate::agent_prompt::StructuredInputProjection>,
    #[serde(default = "absent_projection")]
    pub animus: Option<crate::agent_prompt::StructuredInputProjection>,
}

fn absent_projection() -> Option<crate::agent_prompt::StructuredInputProjection> {
    None
}

impl StructuredInputProjectionPolicy {
    #[must_use]
    pub fn for_target(&self, target: super::Target) -> crate::agent_prompt::StructuredInputProjection {
        match target {
            super::Target::Pi => self.pi,
            super::Target::Clankers => self.clankers,
            super::Target::Animus => self.animus,
        }
        .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CommandOptions {
    pub command: String,
    #[serde(default = "empty_args")]
    pub args: Vec<String>,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self {
            runtimes: default_runtimes(),
            openai_compatible: std::collections::BTreeMap::new(),
            structured_input_projection: StructuredInputProjectionPolicy::default(),
            max_output_bytes: default_output_max_bytes(),
            local_process: crate::process::core::RuntimeConfig::default(),
        }
    }
}

fn absent_prompt_file() -> Option<String> {
    None
}

fn default_runtimes() -> std::collections::BTreeMap<String, CommandOptions> {
    std::collections::BTreeMap::from([
        (DEFAULT_PI_RUNTIME_KEY.to_string(), CommandOptions {
            command: DEFAULT_PI_COMMAND.to_string(),
            args: vec![
                NO_SESSION_FLAG.to_string(),
                PRINT_MODE_FLAG.to_string(),
                PROMPT_PLACEHOLDER.to_string(),
            ],
        }),
        (DEFAULT_CLANKERS_RUNTIME_KEY.to_string(), CommandOptions {
            command: DEFAULT_CLANKERS_COMMAND.to_string(),
            args: vec![PRINT_MODE_FLAG.to_string(), PROMPT_PLACEHOLDER.to_string()],
        }),
    ])
}

fn empty_openai_runtimes() -> std::collections::BTreeMap<String, crate::openai::RuntimeConfig> {
    std::collections::BTreeMap::new()
}

fn default_output_max_bytes() -> usize {
    DEFAULT_AGENT_OUTPUT_MAX_BYTES
}

impl Settings {
    pub fn pi(prompt: impl Into<String>, output_mode: super::OutputMode) -> Self {
        Self::new(super::Target::Pi, prompt, output_mode)
    }

    pub fn clankers(prompt: impl Into<String>, output_mode: super::OutputMode) -> Self {
        Self::new(super::Target::Clankers, prompt, output_mode)
    }

    pub fn animus(prompt: impl Into<String>, output_mode: super::OutputMode) -> Self {
        Self::new(super::Target::Animus, prompt, output_mode)
    }

    pub fn new(target: super::Target, prompt: impl Into<String>, output_mode: super::OutputMode) -> Self {
        Self {
            target,
            prompt: prompt.into(),
            prompt_file: None,
            output_mode,
            model_hint: None,
            tool_policy: String::new(),
            context_refs: Vec::new(),
            cwd: None,
            session: None,
        }
    }
}

fn empty_args() -> Vec<String> {
    Vec::new()
}
