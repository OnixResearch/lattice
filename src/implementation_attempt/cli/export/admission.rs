use std::os::unix::fs::MetadataExt;

const MAX_COMPONENT_BYTES: usize = 255;

pub(super) fn load(
    request_path: &std::path::Path,
    state: &std::path::Path,
    output_parent: &std::path::Path,
) -> Result<super::Admitted, String> {
    let (request, bytes) = request(request_path)?;
    private_directory(state)?;
    let state = state.canonicalize().map_err(|error| error.to_string())?;
    let workspace = request.workspace_root.canonicalize().map_err(|error| error.to_string())?;
    let output = destination(&request, request_path, &state, &workspace, output_parent)?;
    let directory = super::super::artifacts::directory(&state, &request);
    private_directory(&directory)?;
    check_intent(&directory, &bytes)?;
    Ok(super::Admitted {
        request,
        workspace,
        output,
        directory,
    })
}

fn request(path: &std::path::Path) -> Result<(super::super::Request, Vec<u8>), String> {
    let (request, bytes) = super::super::artifacts::load_request(path)?;
    request.envelope.validate().map_err(|error| error.to_string())?;
    request.validate_binding()?;
    request.validate_paths().map_err(|_| "implementation request paths are invalid")?;
    if request.schema != super::super::REQUEST_SCHEMA || request.admitted_envelope != request.envelope.identity() {
        return Err("implementation request is crossed or unsupported".into());
    }
    Ok((request, bytes))
}

fn destination(
    request: &super::super::Request,
    request_path: &std::path::Path,
    state: &std::path::Path,
    workspace: &std::path::Path,
    output_parent: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    let parent = checked_parent(output_parent)?;
    let component = format!("{}-{}", request.envelope.attempt_id, request.envelope.fence);
    if invalid_component(&component) {
        return Err("attempt output name is not a bounded single component".into());
    }
    let output = parent.join(component);
    let request_path = request_path.canonicalize().map_err(|error| error.to_string())?;
    if overlaps(&output, workspace) || overlaps(&output, state) || output.starts_with(&request_path) {
        return Err("candidate output overlaps retained source material".into());
    }
    Ok(output)
}

fn invalid_component(component: &str) -> bool {
    component.len() > MAX_COMPONENT_BYTES
        || matches!(component, "." | "..")
        || component.bytes().any(|byte| matches!(byte, b'/' | b'\\' | 0))
}

fn overlaps(left: &std::path::Path, right: &std::path::Path) -> bool {
    left.starts_with(right) || right.starts_with(left)
}

fn private_directory(path: &std::path::Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("state directory must be absolute".into());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| "retained state directory is unavailable")?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || path.canonicalize().map_err(|error| error.to_string())? != path
    {
        return Err("retained state directory must be canonical and dispatcher-owned".into());
    }
    if metadata.mode() & 0o077 != 0 {
        return Err("retained state directory is not private".into());
    }
    Ok(())
}

fn checked_parent(path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    if !path.is_absolute() {
        return Err("output parent must be absolute".into());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || !is_owned(&metadata) || metadata.nlink() == 0 {
        return Err("output parent must be an owned dispatcher-group directory".into());
    }
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    if canonical != path {
        return Err("output parent must be canonical".into());
    }
    Ok(canonical)
}

fn check_intent(directory: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let identity = blake3::hash(bytes).to_hex().to_string();
    if super::super::artifacts::read_bounded(&directory.join("intent.json"), identity.len())? != identity.as_bytes() {
        return Err("request differs from persisted execution intent".into());
    }
    Ok(())
}

pub(super) fn retained(inputs: &super::Admitted) -> Result<super::Retained, String> {
    let store = store(&inputs.directory)?;
    let admitted = super::super::binding::load(&inputs.directory, &inputs.request)?
        .ok_or("retained attempt has no persisted execution binding")?;
    let observation =
        super::super::recovery::recover(&store, &inputs.request, &inputs.directory, &admitted.binding_identity)?;
    let result = complete_result(&store, &inputs.request)?;
    Ok(super::Retained {
        store,
        observation,
        result,
    })
}

fn store(directory: &std::path::Path) -> Result<crate::store::Store, String> {
    let path = directory.join("candidate.redb");
    let metadata = std::fs::symlink_metadata(&path).map_err(|_| "native attempt store is unavailable")?;
    if !metadata.is_file() || metadata.nlink() != 1 || !is_owned(&metadata) {
        return Err("native attempt store is not an owned ordinary file".into());
    }
    crate::store::Store::new(&path).map_err(|error| error.to_string())
}

fn complete_result(
    store: &crate::store::Store,
    request: &super::super::Request,
) -> Result<crate::candidate_worker::AttemptResult, String> {
    match crate::candidate_worker::reconcile_attempt(store, &request.envelope.attempt_id)
        .map_err(|error| error.to_string())?
    {
        crate::candidate_worker::Recovery::Published(result) => Ok(*result),
        crate::candidate_worker::Recovery::Missing | crate::candidate_worker::Recovery::Incomplete { .. } => {
            Err("native attempt has no complete retained result".into())
        }
    }
}

pub(super) fn is_owned(metadata: &std::fs::Metadata) -> bool {
    metadata.uid() == unsafe { libc::geteuid() } && metadata.gid() == unsafe { libc::getegid() }
}
