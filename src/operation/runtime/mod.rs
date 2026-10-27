#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "the operation runtime uses its local process observation and serialization vocabulary"
)]
#![allow(
    assertion_density,
    reason = "output classification uses typed observations and adversarial malformed, secret, and overclaim tests"
)]

use serde::Deserialize;
use serde::Serialize;

pub mod component;

pub const CAIRN_RUNTIME_KIND: &str = "process";
pub const CAIRN_RUNTIME_KEY: &str = "cairn-read-only-v1";
pub const CAIRN_CHANGE_LIST_METHOD: &str = "change_list";
pub const CAIRN_VALIDATE_METHOD: &str = "validate";
pub const REPOSITORY_ROOT_TARGET: &str = "repository_root";
const SUCCESS_EXIT_CODE: i32 = 0;
pub(crate) const OUTPUT_IDENTITY_DOMAIN: &str = "lattice.operation-output.v1";
const PROHIBITED_CLAIM_FIELDS: &[&str] = &[
    "lifecycle_complete",
    "production_ready",
    "release_eligible",
    "release_ready",
    "source_correct",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationClass {
    Completed,
    DeterministicFailure,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub class: ObservationClass,
    pub output: Option<serde_json::Value>,
    pub output_identity: Option<String>,
    pub output_bytes: usize,
    pub stderr_bytes: usize,
    pub diagnostic: String,
    pub redacted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedProcessRequest {
    pub process: crate::agent_node::ProcessRequest,
    pub secret_values: Vec<String>,
}

pub fn prepare_process_request(
    plan: &super::model::InvocationPlan,
    credentials: &dyn crate::credentials::CredentialProvider,
) -> Result<AuthorizedProcessRequest, Observation> {
    let process = process_request_plan(plan)?;
    let observed_executor_identity =
        super::identity::executor_file(std::path::Path::new(&plan.runtime.executable_path))
            .map_err(|_| unknown("operation executor identity is unavailable"))?;
    if observed_executor_identity != plan.runtime.executor_identity {
        return Err(deterministic("operation executor identity changed after planning"));
    }
    validate_repository_root_observation(process.cwd.as_deref())?;
    let secret_values = resolve_credentials(plan, credentials)?;
    Ok(AuthorizedProcessRequest { process, secret_values })
}

pub fn process_request_plan(
    plan: &super::model::InvocationPlan,
) -> Result<crate::agent_node::ProcessRequest, Observation> {
    super::planning::validate_plan_identity(plan).map_err(|_| deterministic("invocation plan identity is invalid"))?;
    if plan.runtime.kind != CAIRN_RUNTIME_KIND || plan.runtime.key != CAIRN_RUNTIME_KEY {
        return Err(deterministic("operation runtime binding is unsupported"));
    }
    let repository_root = repository_root(plan)?;
    let args = cairn_arguments(&plan.method, &repository_root)?;
    Ok(crate::agent_node::ProcessRequest {
        program: plan.runtime.executable_path.clone(),
        args,
        cwd: Some(repository_root),
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: plan.bounds.timeout_seconds,
        output_limit_bytes: plan.bounds.output_bytes,
    })
}

fn resolve_credentials(
    plan: &super::model::InvocationPlan,
    provider: &dyn crate::credentials::CredentialProvider,
) -> Result<Vec<String>, Observation> {
    let mut values = Vec::with_capacity(plan.credentials.len());
    for reference in plan.credentials.values() {
        let value = provider.resolve(reference).map_err(|_| deterministic("operation credential resolution failed"))?;
        values.push(value.expose_for_execution().to_string());
    }
    Ok(values)
}

fn repository_root(plan: &super::model::InvocationPlan) -> Result<String, Observation> {
    let Some(root) = plan.target.get(REPOSITORY_ROOT_TARGET).and_then(serde_json::Value::as_str) else {
        return Err(deterministic("operation repository root target is missing"));
    };
    if !std::path::Path::new(root).is_absolute() {
        return Err(deterministic("operation repository root must be absolute"));
    }
    Ok(root.to_string())
}

fn validate_repository_root_observation(root: Option<&str>) -> Result<(), Observation> {
    let root = root.ok_or_else(|| deterministic("operation repository root target is missing"))?;
    let metadata =
        std::fs::symlink_metadata(root).map_err(|_| deterministic("operation repository root is unavailable"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(deterministic("operation repository root is unsafe"));
    }
    Ok(())
}

#[allow(
    unknown_lints,
    ambiguous_params,
    reason = "the closed Cairn adapter receives one method identifier and one absolute repository root"
)]
fn cairn_arguments(method: &str, root: &str) -> Result<Vec<String>, Observation> {
    match method {
        CAIRN_CHANGE_LIST_METHOD => Ok(vec![
            "change".to_string(),
            "list".to_string(),
            "--root".to_string(),
            root.to_string(),
        ]),
        CAIRN_VALIDATE_METHOD => Ok(vec!["validate".to_string(), "--root".to_string(), root.to_string()]),
        _ => Err(deterministic("operation Cairn method is unsupported")),
    }
}

pub fn classify_process_output(
    plan: &super::model::InvocationPlan,
    output: crate::agent_node::ProcessOutput,
    secret_values: &[String],
) -> Result<Observation, Observation> {
    let output_bytes = output.stdout.len();
    let stderr_bytes = output.stderr.len();
    if contains_secret(&output.stdout, secret_values) || contains_secret(&output.stderr, secret_values) {
        return Err(deterministic_with_sizes(
            "operation output contained credential material",
            output_bytes,
            stderr_bytes,
        ));
    }
    if output.status_code != Some(SUCCESS_EXIT_CODE) {
        return Ok(Observation {
            class: ObservationClass::DeterministicFailure,
            output: None,
            output_identity: None,
            output_bytes,
            stderr_bytes,
            diagnostic: "operation process returned a nonzero status".to_string(),
            redacted: true,
        });
    }
    let value = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .map_err(|_| deterministic_with_sizes("operation output is malformed JSON", output_bytes, stderr_bytes))?;
    if output_contains_prohibited_field(&value) {
        return Err(deterministic_with_sizes(
            "operation output contains prohibited secret or overclaim fields",
            output_bytes,
            stderr_bytes,
        ));
    }
    let diagnostics = super::schema::validate_value(
        &plan.output_contract,
        &value,
        "output",
        super::model::DiagnosticCode::OutputRejected,
    );
    if !diagnostics.is_empty() {
        return Err(deterministic_with_sizes(
            "operation output violates its declared contract",
            output_bytes,
            stderr_bytes,
        ));
    }
    let identity = super::identity::hash_value(OUTPUT_IDENTITY_DOMAIN, &value)
        .map_err(|_| deterministic_with_sizes("operation output identity failed", output_bytes, stderr_bytes))?;
    Ok(Observation {
        class: ObservationClass::Completed,
        output: Some(value),
        output_identity: Some(identity),
        output_bytes,
        stderr_bytes,
        diagnostic: "operation output passed its declared contract".to_string(),
        redacted: true,
    })
}

pub(crate) fn output_contains_prohibited_field(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Array(values) => values.iter().any(output_contains_prohibited_field),
        serde_json::Value::Object(values) => values.iter().any(|(key, value)| {
            super::schema::key_is_secret_bearing(key)
                || PROHIBITED_CLAIM_FIELDS.contains(&key.as_str())
                || output_contains_prohibited_field(value)
        }),
        serde_json::Value::Null
        | serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::String(_) => false,
    }
}

fn contains_secret(bytes: &[u8], secret_values: &[String]) -> bool {
    secret_values.iter().any(|secret| {
        let secret_bytes = secret.as_bytes();
        !secret_bytes.is_empty() && bytes.windows(secret_bytes.len()).any(|candidate| candidate == secret_bytes)
    })
}

pub fn process_failure(error: crate::agent_node::ProcessError) -> Observation {
    let (class, diagnostic) = match error {
        crate::agent_node::ProcessError::Timeout => (ObservationClass::Unknown, "operation process timed out"),
        crate::agent_node::ProcessError::Cancelled => (ObservationClass::Unknown, "operation process was cancelled"),
        crate::agent_node::ProcessError::OutputLimitExceeded { .. } => {
            (ObservationClass::Unknown, "operation process output was truncated")
        }
        crate::agent_node::ProcessError::InvalidRequest(_) => {
            (ObservationClass::DeterministicFailure, "operation process request was rejected")
        }
        crate::agent_node::ProcessError::Spawn(_) => {
            (ObservationClass::Unknown, "operation process outcome is unavailable")
        }
    };
    Observation {
        class,
        output: None,
        output_identity: None,
        output_bytes: 0,
        stderr_bytes: 0,
        diagnostic: diagnostic.to_string(),
        redacted: true,
    }
}

pub fn backend_denied(message: &str) -> Observation {
    deterministic(message)
}

pub fn unknown(message: &str) -> Observation {
    Observation {
        class: ObservationClass::Unknown,
        output: None,
        output_identity: None,
        output_bytes: 0,
        stderr_bytes: 0,
        diagnostic: message.to_string(),
        redacted: true,
    }
}

fn deterministic(message: &str) -> Observation {
    deterministic_with_sizes(message, 0, 0)
}

#[allow(
    unknown_lints,
    ambiguous_params,
    reason = "the metadata-only observation records distinctly named stdout and stderr byte counts"
)]
fn deterministic_with_sizes(message: &str, output_bytes: usize, stderr_bytes: usize) -> Observation {
    Observation {
        class: ObservationClass::DeterministicFailure,
        output: None,
        output_identity: None,
        output_bytes,
        stderr_bytes,
        diagnostic: message.to_string(),
        redacted: true,
    }
}
