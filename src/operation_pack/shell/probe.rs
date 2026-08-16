#![allow(
    unknown_lints,
    assertion_density,
    reason = "probe admission and process failures return typed outcomes instead of terminating the process"
)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Succeeded,
    Failed,
    TimedOut,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub pack_identity: String,
    pub runtime_identity: String,
    pub status: Status,
    pub output_bytes: usize,
    pub output_content_omitted: bool,
    pub production_credentials_used: bool,
    pub mutation_authority_granted: bool,
    pub network_authority_granted: bool,
}

pub fn run(
    pack_root: &std::path::Path,
    admitted: &super::super::model::AdmittedPack,
    process: &crate::process::core::RuntimeConfig,
    cancellation: Option<&std::sync::atomic::AtomicBool>,
) -> Result<Outcome, super::super::Error> {
    super::super::admission::validate_lock(&admitted.candidate, &admitted.lock)?;
    let probe = admitted
        .candidate
        .manifest
        .probe
        .as_ref()
        .ok_or(super::super::Error::Invalid("doctor probe missing"))?;
    if probe.effectful || probe.uses_production_credentials {
        return Err(super::super::Error::Invalid("doctor probe authority"));
    }
    let runtime = admitted
        .candidate
        .manifest
        .runtimes
        .iter()
        .find(|runtime| runtime.id == probe.runtime_id)
        .ok_or(super::super::Error::Invalid("doctor probe runtime"))?;
    if !runtime.launch_labels.contains(&probe.launch_label)
        || runtime
            .capabilities
            .iter()
            .any(|capability| *capability != crate::operation::CapabilityClass::ProcessRead)
    {
        return Err(super::super::Error::Invalid("doctor probe capability"));
    }
    let executable = super::content_path(pack_root, &admitted.candidate, &runtime.content_id)?;
    let request = crate::agent_node::ProcessRequest {
        program: executable.to_string_lossy().into_owned(),
        args: vec![probe.launch_label.clone()],
        cwd: Some(pack_root.to_string_lossy().into_owned()),
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs: runtime.timeout_seconds,
        output_limit_bytes: runtime.output_bytes,
    };
    let (status, output_bytes) =
        match crate::process::shell::run_bounded_clear_environment_with_cancellation(&request, process, cancellation) {
            Ok(output) => {
                let bytes = output.stdout.len().saturating_add(output.stderr.len());
                let status = if output.status_code == Some(0) {
                    Status::Succeeded
                } else {
                    Status::Failed
                };
                (status, bytes)
            }
            Err(crate::agent_node::ProcessError::Timeout | crate::agent_node::ProcessError::Cancelled) => {
                (Status::TimedOut, 0)
            }
            Err(crate::agent_node::ProcessError::OutputLimitExceeded { .. }) => (Status::Truncated, 0),
            Err(crate::agent_node::ProcessError::InvalidRequest(_) | crate::agent_node::ProcessError::Spawn(_)) => {
                (Status::Failed, 0)
            }
        };
    Ok(Outcome {
        pack_identity: admitted.candidate.pack_identity.clone(),
        runtime_identity: super::super::identity::runtime_identity(runtime)?,
        status,
        output_bytes,
        output_content_omitted: true,
        production_credentials_used: false,
        mutation_authority_granted: false,
        network_authority_granted: false,
    })
}
