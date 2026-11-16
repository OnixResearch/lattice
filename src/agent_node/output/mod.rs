const ERROR_OUTPUT_LIMIT_CHARS: usize = 2048;

const HANDOFF_PATH_FIELD: &str = "path";
const HANDOFF_MEDIA_TYPE_FIELD: &str = "media_type";
const HANDOFF_METADATA_FIELD: &str = "metadata";
const CURRENT_DIRECTORY: &str = ".";

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct FileEnvelope {
    pub path: String,
    #[serde(default = "absent_media_type")]
    pub media_type: Option<String>,
    #[serde(default = "empty_metadata")]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HandoffFile {
    path_label: String,
    bytes: Vec<u8>,
}

// r[impl n8nclone.workflow_execution.agent_nodes.production_adapters.command]
// r[impl n8nclone.workflow_execution.runtime_registry.command_runtime]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.contract]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.image]
// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.artifact]
pub fn decode_command(
    plan: &super::CommandPlan,
    output: &super::CommandOutput,
) -> Result<super::Response, super::Error> {
    if output.status_code != Some(0) {
        return Err(super::Error::Adapter(format!(
            "{} agent command exited with status {}; stderr: {}",
            plan.adapter_label,
            status_label(output.status_code),
            bounded_error_text(&output.stderr)
        )));
    }

    match plan.output_mode {
        super::OutputMode::Text => Ok(super::Response {
            output: super::Output::Text(output.stdout.trim_end().to_string()),
            metadata: command_metadata(plan, output, None),
        }),
        super::OutputMode::Json => decode_json(plan, output),
        super::OutputMode::Image => decode_image_file_handoff(plan, output),
        super::OutputMode::Artifact => decode_artifact_file_handoff(plan, output),
    }
}

fn decode_image_file_handoff(
    plan: &super::CommandPlan,
    output: &super::CommandOutput,
) -> Result<super::Response, super::Error> {
    let envelope = parse_file_envelope(plan, &output.stdout)?;
    let output_file = load_handoff_file(plan, &envelope)?;
    let image = crate::image_node::image_value_from_loaded_bytes(
        &output_file.path_label,
        output_file.bytes,
        plan.max_output_bytes,
    )
    .map_err(|error| super::Error::Adapter(error.to_string()))?;
    validate_declared_media_type(plan, envelope.media_type.as_deref(), &image.media_type)?;
    let metadata =
        command_metadata(plan, output, Some(file_metadata(&envelope, &output_file.path_label, image.bytes.len())));
    Ok(super::Response {
        output: super::Output::Image(image),
        metadata,
    })
}

fn decode_artifact_file_handoff(
    plan: &super::CommandPlan,
    output: &super::CommandOutput,
) -> Result<super::Response, super::Error> {
    let envelope = parse_file_envelope(plan, &output.stdout)?;
    let output_file = load_handoff_file(plan, &envelope)?;
    let metadata = command_metadata(
        plan,
        output,
        Some(file_metadata(&envelope, &output_file.path_label, output_file.bytes.len())),
    );
    Ok(super::Response {
        output: super::Output::Artifact(output_file.bytes),
        metadata,
    })
}

pub fn parse_file_envelope(plan: &super::CommandPlan, stdout: &str) -> Result<FileEnvelope, super::Error> {
    let envelope = serde_json::from_str::<FileEnvelope>(stdout.trim()).map_err(|error| {
        super::Error::Adapter(format!(
            "{} agent {} handoff envelope is invalid JSON: {error}",
            plan.adapter_label,
            plan.output_mode.as_str()
        ))
    })?;
    if envelope.path.trim().is_empty() {
        return Err(super::Error::Adapter(format!(
            "{} agent {} handoff envelope must include a non-empty {HANDOFF_PATH_FIELD}",
            plan.adapter_label,
            plan.output_mode.as_str()
        )));
    }
    Ok(envelope)
}

// r[impl n8nclone.workflow_execution.agent_nodes.output_file_handoff.capability_read]
fn load_handoff_file(plan: &super::CommandPlan, envelope: &FileEnvelope) -> Result<HandoffFile, super::Error> {
    debug_assert!(!envelope.path.trim().is_empty());
    let resolved = resolve_handoff_path(plan, &envelope.path)?;
    let base = plan.cwd.as_deref().unwrap_or(CURRENT_DIRECTORY);
    let root = crate::capability::open_root(&resolved).map_err(|error| {
        super::Error::Adapter(format!("{} agent working directory is unavailable: {base}: {error}", plan.adapter_label))
    })?;
    let file = crate::capability::open_relative_read(&root, resolved.relative())
        .map_err(|error| file_io_error(plan, &envelope.path, error))?;
    let metadata = file.metadata().map_err(|error| file_io_error(plan, &envelope.path, error))?;
    let metadata_bytes = usize::try_from(metadata.len()).map_err(|_| {
        super::Error::Adapter(format!(
            "{} agent output file is too large to address on this platform: {}",
            plan.adapter_label, envelope.path
        ))
    })?;
    validate_file_size(plan, &envelope.path, metadata_bytes)?;
    let bytes = crate::capability::read_bounded(file, plan.max_output_bytes)
        .map_err(|error| file_io_error(plan, &envelope.path, error))?;
    validate_file_size(plan, &envelope.path, bytes.len())?;
    debug_assert!(bytes.len() <= plan.max_output_bytes);
    Ok(HandoffFile {
        path_label: envelope.path.clone(),
        bytes,
    })
}

fn resolve_handoff_path(plan: &super::CommandPlan, path: &str) -> Result<crate::capability::RootedPath, super::Error> {
    let base = plan.cwd.as_deref().unwrap_or(CURRENT_DIRECTORY);
    crate::capability::rooted_relative_path(std::path::Path::new(base), std::path::Path::new(path)).ok_or_else(|| {
        super::Error::Adapter(format!("{} agent output file is outside working directory: {path}", plan.adapter_label))
    })
}

fn validate_file_size(plan: &super::CommandPlan, path: &str, bytes: usize) -> Result<(), super::Error> {
    if bytes <= plan.max_output_bytes {
        return Ok(());
    }
    Err(super::Error::Adapter(format!(
        "{} agent output file is too large: {path} has {bytes} bytes, max {}",
        plan.adapter_label, plan.max_output_bytes
    )))
}

fn file_io_error(plan: &super::CommandPlan, path: &str, error: std::io::Error) -> super::Error {
    match error.kind() {
        std::io::ErrorKind::NotFound => {
            super::Error::Adapter(format!("{} agent output file is missing: {path}", plan.adapter_label))
        }
        std::io::ErrorKind::PermissionDenied => {
            super::Error::Adapter(format!("{} agent output file permission denied: {path}", plan.adapter_label))
        }
        _ => super::Error::Adapter(format!("{} agent output file I/O error: {path}: {error}", plan.adapter_label)),
    }
}

fn validate_declared_media_type(
    plan: &super::CommandPlan,
    declared: Option<&str>,
    sniffed: &str,
) -> Result<(), super::Error> {
    let Some(declared) = declared else {
        return Ok(());
    };
    if declared == sniffed {
        return Ok(());
    }
    Err(super::Error::Adapter(format!(
        "{} agent image media type mismatch: declared {declared}, sniffed {sniffed}",
        plan.adapter_label
    )))
}

fn command_metadata(
    plan: &super::CommandPlan,
    output: &super::CommandOutput,
    file_metadata: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut metadata = serde_json::json!({
        "adapter": plan.adapter_label,
        "target": super::target_label(plan.target),
        "stdout_bytes": output.stdout.len(),
    });
    if let Some(file_metadata) = file_metadata {
        metadata["file_handoff"] = file_metadata;
    }
    metadata
}

fn file_metadata(envelope: &FileEnvelope, path_label: &str, bytes: usize) -> serde_json::Value {
    serde_json::json!({
        HANDOFF_PATH_FIELD: path_label,
        "bytes": bytes,
        HANDOFF_MEDIA_TYPE_FIELD: envelope.media_type.clone(),
        HANDOFF_METADATA_FIELD: envelope.metadata.clone(),
    })
}

fn status_label(status_code: Option<i32>) -> String {
    match status_code {
        Some(code) => code.to_string(),
        None => "terminated by signal".to_string(),
    }
}

fn bounded_error_text(text: &str) -> String {
    text.chars().take(ERROR_OUTPUT_LIMIT_CHARS).collect()
}

fn absent_media_type() -> Option<String> {
    None
}

fn empty_metadata() -> serde_json::Value {
    serde_json::Value::Null
}

fn decode_json(plan: &super::CommandPlan, output: &super::CommandOutput) -> Result<super::Response, super::Error> {
    let json = serde_json::from_str(output.stdout.trim()).map_err(|error| {
        super::Error::Adapter(format!("{} agent command returned invalid JSON: {error}", plan.adapter_label))
    })?;
    Ok(super::Response {
        output: super::Output::Json(json),
        metadata: command_metadata(plan, output, None),
    })
}
