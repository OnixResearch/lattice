use std::io::Read;
use std::io::Write;

/// Only the schema is read first so a legacy request names its readmission
/// path instead of failing on the fields it lacks.
#[derive(serde::Deserialize)]
struct SchemaProbe {
    schema: String,
}

pub(super) fn load_request(path: &std::path::Path) -> Result<(super::Request, Vec<u8>), String> {
    let bytes = read_bounded(path, super::MAX_REQUEST_BYTES)?;
    let probe: SchemaProbe = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if probe.schema == super::LEGACY_REQUEST_SCHEMA {
        return Err(format!(
            "{} requests are refused; readmit them explicitly as {}",
            super::LEGACY_REQUEST_SCHEMA,
            super::REQUEST_SCHEMA
        ));
    }
    let request = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    Ok((request, bytes))
}

fn create_private_directory(path: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|error| error.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(super) fn private_directory(path: &std::path::Path) -> Result<(), String> {
    if !path.is_absolute() {
        return Err("state directory must be absolute".into());
    }
    if !path.exists() {
        create_private_directory(path)?;
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || path.canonicalize().map_err(|error| error.to_string())? != path {
        return Err("state directory must be canonical and not a symlink".into());
    }
    #[cfg(unix)]
    validate_directory_permissions(&metadata)?;
    Ok(())
}

#[cfg(unix)]
fn validate_directory_permissions(metadata: &std::fs::Metadata) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err("state directory is not private".into());
    }
    Ok(())
}

fn sync_created_file(mut file: std::fs::File, path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|error| error.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::File::open(parent).and_then(|file| file.sync_all()).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn create_once(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

/// Create and fsync a new private file; `false` when it already exists.
pub(super) fn create_new(path: &std::path::Path, bytes: &[u8]) -> Result<bool, String> {
    match create_once(path) {
        Ok(file) => {
            sync_created_file(file, path, bytes)?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(error.to_string()),
    }
}

/// Create and fsync a new private file like [`write_once`], but a file this
/// call created is removed when its content or directory sync fails: a
/// readable terminal record that never became durable must not be read back as
/// evidence. An existing file is compared and never removed.
pub(super) fn write_once_durable(path: &std::path::Path, bytes: &[u8]) -> Result<bool, String> {
    debug_assert!(path.is_absolute(), "evidence is written under an absolute attempt directory");
    debug_assert!(path.file_name().is_some(), "a durable write names its file");
    match create_once(path) {
        Ok(file) => match sync_created_file(file, path, bytes) {
            Ok(()) => Ok(true),
            Err(error) => {
                let _removed = std::fs::remove_file(path);
                Err(error)
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_bounded(path, bytes.len())? != bytes {
                return Err("attempt state contains a different immutable binding".into());
            }
            Ok(false)
        }
        Err(error) => Err(error.to_string()),
    }
}

pub(super) fn write_once(path: &std::path::Path, bytes: &[u8]) -> Result<bool, String> {
    if create_new(path, bytes)? {
        return Ok(true);
    }
    if read_bounded(path, bytes.len())? != bytes {
        return Err("attempt state contains a different immutable binding".into());
    }
    Ok(false)
}

/// Read an optional retained artifact; only a missing file is `None`.
pub(super) fn read_optional(path: &std::path::Path, max_bytes: usize) -> Result<Option<Vec<u8>>, String> {
    match std::fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("retained artifact metadata is unavailable".into()),
        Ok(_) => read_bounded(path, max_bytes).map(Some),
    }
}

pub(super) fn directory(state: &std::path::Path, request: &super::Request) -> std::path::PathBuf {
    let key = state.join(format!("{}-{}", request.envelope.attempt_id, request.envelope.fence));
    state.join(blake3::hash(key.as_os_str().as_encoded_bytes()).to_hex().to_string())
}

fn open_artifact(path: &std::path::Path) -> Result<std::fs::File, String> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    options.open(path).map_err(|_| "retained artifact is unavailable".into())
}

pub(super) fn read_bounded(path: &std::path::Path, max_bytes: usize) -> Result<Vec<u8>, String> {
    let limit_bytes = u64::try_from(max_bytes).map_err(|_| "artifact bound is invalid")?;
    let probe_bytes = limit_bytes.checked_add(1).ok_or("artifact bound cannot include an overflow probe")?;
    let file = open_artifact(path)?;
    let metadata = file.metadata().map_err(|_| "retained artifact metadata is unavailable")?;
    if !metadata.is_file() || metadata.len() > limit_bytes {
        return Err("retained artifact is not a bounded ordinary file".into());
    }
    let capacity_bytes = usize::try_from(metadata.len()).map_err(|_| "artifact length is not addressable")?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    file.take(probe_bytes).read_to_end(&mut bytes).map_err(|_| "retained artifact read failed")?;
    if bytes.len() > max_bytes {
        return Err("retained artifact exceeds its admitted bound".into());
    }
    Ok(bytes)
}

pub(super) fn verify_events(
    directory: &std::path::Path,
    output_limit_bytes: usize,
    stdout_identity: &str,
) -> Result<(), String> {
    if output_limit_bytes == 0 || output_limit_bytes > super::MAX_OUTPUT_BYTES {
        return Err("retained output bound is invalid".into());
    }
    let raw = read_bounded(&directory.join("raw-events.jsonl"), output_limit_bytes)?;
    if blake3::hash(&raw).to_hex().as_str() != stdout_identity {
        return Err("retained raw OMP events differ from the owned process observation".into());
    }
    let normalized = read_bounded(&directory.join("omp-observation.json"), 16_384)?;
    let retained: Result<crate::candidate_worker::omp::Observation, String> =
        serde_json::from_slice(&normalized).map_err(|_| "retained OMP observation is malformed")?;
    let expected = crate::candidate_worker::omp::normalize_jsonl(&raw, output_limit_bytes, "/workspace");
    // A worker blocked before it produced any event retains its blocker as the
    // unavailable observation; no terminal can be invented from empty output.
    let is_blocked_before_output = raw.is_empty() && retained.is_err() && expected.is_err();
    if retained != expected && !is_blocked_before_output {
        return Err("retained OMP observation does not describe the exact raw event artifact".into());
    }
    Ok(())
}

fn normalized_request(request_path: &std::path::Path) -> Result<(super::Request, Vec<u8>), String> {
    let (mut request, _) = load_request(request_path)?;
    let baseline = crate::candidate_capture::workspace(&request.workspace_root).map_err(|error| error.to_string())?;
    request.omp.base_identity = crate::candidate::snapshot_identity(&baseline).map_err(|error| error.to_string())?;
    request.envelope.worker_profile = request.worker_profile_identity()?;
    request.admitted_envelope = request.envelope.identity();
    super::binding::normalize_primary_route(&mut request)?;
    request.validate(super::epoch_ms(&crate::operation::shell::SystemClock)?)?;
    let bytes = serde_json::to_vec_pretty(&request).map_err(|error| error.to_string())?;
    Ok((request, bytes))
}

/// `prepare`: write the normalized request once and print its identities,
/// without executing anything or granting authority. A request with a
/// repair policy also prints `repair_authority`, the standalone operator
/// profile requirements of its repair worker and checks, in the form
/// `authority inspect` prints.
// r[impl lattice.implementation_attempt.operator_repair.scenario.authority]
pub(super) fn prepare(request_path: &std::path::Path, output: &std::path::Path) -> Result<(), String> {
    let (request, bytes) = normalized_request(request_path)?;
    if !write_once(output, &bytes)? {
        return Err("prepared request output already exists".into());
    }
    let mut printed = serde_json::json!({"request":output,"identity":request.admitted_envelope,
        "worker_request":blake3::hash(&bytes).to_hex().to_string(),
        "worker_profile":request.envelope.worker_profile,"admitted":false});
    if request.repair.is_some()
        && let Some(fields) = printed.as_object_mut()
    {
        let requirements =
            crate::implementation_attempt::repair::step::operator::requirements(&request.envelope.worker_profile);
        let requirements = serde_json::to_value(requirements).map_err(|error| error.to_string())?;
        fields.insert("repair_authority".to_owned(), requirements);
    }
    println!("{printed}");
    Ok(())
}
