/// Cleanup owner for execution-only process environment copies.
pub(crate) struct ExecutionProcessRequest(pub(crate) ProcessRequest);

impl Drop for ExecutionProcessRequest {
    fn drop(&mut self) {
        for value in self.0.env.values_mut() {
            zeroize::Zeroize::zeroize(value);
        }
    }
}

/// Execution-only secrets for an agent adapter call.
#[derive(Default)]
pub struct ExecutionContext {
    pub(super) secret_env: std::collections::BTreeMap<String, String>,
    pub(in crate::agent_node) secret_values: Vec<String>,
}

impl ExecutionContext {
    pub fn new(secret_env: std::collections::HashMap<String, String>) -> Self {
        let mut secret_values = secret_env.values().filter(|value| !value.is_empty()).cloned().collect::<Vec<_>>();
        secret_values.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| left.cmp(right)));
        Self {
            secret_env: secret_env.into_iter().collect(),
            secret_values,
        }
    }

    pub fn secret(&self, binding: &str) -> Option<&str> {
        self.secret_env.get(binding).map(String::as_str)
    }

    pub fn redact(&self, value: &str) -> String {
        crate::credentials::redact_text(value, &self.secret_values)
    }
}

impl Drop for ExecutionContext {
    fn drop(&mut self) {
        for value in self.secret_env.values_mut() {
            zeroize::Zeroize::zeroize(value);
        }
        zeroize::Zeroize::zeroize(&mut self.secret_values);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessRequest {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: std::collections::BTreeMap<String, String>,
    pub stdin: Vec<u8>,
    pub timeout_secs: u64,
    pub output_limit_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Phase known from the pinned runner's control flow, without OS diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessFailurePhase {
    Execution,
    Capture,
    Teardown,
}

/// Observation, not a statement that an unobserved child is still running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessTermination {
    Observed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessError {
    InvalidRequest(&'static str),
    Spawn(String),
    Execution {
        phase: ProcessFailurePhase,
        termination: ProcessTermination,
    },
    Timeout,
    Cancelled,
    OutputLimitExceeded {
        limit_bytes: usize,
        actual_bytes: usize,
    },
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "process request rejected: {message}"),
            Self::Spawn(message) => write!(f, "process spawn failed: {message}"),
            Self::Execution { phase, termination } => {
                write!(f, "process post-start failure ({phase:?}; termination {termination:?})")
            }
            Self::Timeout => write!(f, "process timed out"),
            Self::Cancelled => write!(f, "process cancelled"),
            Self::OutputLimitExceeded {
                limit_bytes: limit,
                actual_bytes: actual,
            } => {
                write!(f, "process output exceeded limit {limit} bytes with {actual} bytes")
            }
        }
    }
}

impl std::error::Error for ProcessError {}

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
pub trait ProcessRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError>;
}

#[derive(Debug, Clone, Default)]
pub struct ProductionProcessRunner {
    config: crate::process::core::RuntimeConfig,
}

impl ProductionProcessRunner {
    pub fn new(config: crate::process::core::RuntimeConfig) -> Self {
        Self { config }
    }

    pub fn run_with_cancellation(
        &self,
        request: &ProcessRequest,
        cancellation: &std::sync::atomic::AtomicBool,
    ) -> Result<ProcessOutput, ProcessError> {
        crate::process::shell::run_bounded(request, &self.config, Some(cancellation))
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
impl ProcessRunner for ProductionProcessRunner {
    fn run(&self, request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        crate::process::shell::run_bounded(request, &self.config, None)
    }
}

#[derive(Debug, Clone)]
pub struct FakeProcessRunner {
    result: Result<ProcessOutput, ProcessError>,
}

impl FakeProcessRunner {
    pub fn new(result: Result<ProcessOutput, ProcessError>) -> Self {
        Self { result }
    }
}

// r[impl n8nclone.workflow_execution.trait_boundaries.process_runner]
impl ProcessRunner for FakeProcessRunner {
    fn run(&self, _request: &ProcessRequest) -> Result<ProcessOutput, ProcessError> {
        self.result.clone()
    }
}
