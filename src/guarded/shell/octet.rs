// r[impl lattice.guarded_agent_templates.octet]
// r[impl lattice.guarded_agent_templates.octet.scenario.restoration]
pub fn probe_octet(
    root: &std::path::Path,
    artifact_relative: &std::path::Path,
    timeout_secs: u64,
) -> Result<super::core::OctetSnapshot, Error> {
    probe_octet_with(root, artifact_relative, timeout_secs, &BoundedProbeProcess)
}

pub fn probe_octet_with(
    root: &std::path::Path,
    artifact_relative: &std::path::Path,
    timeout_secs: u64,
    process: &dyn ProbeProcess,
) -> Result<super::core::OctetSnapshot, Error> {
    if !(MINIMUM_TIMEOUT_SECS..=MAXIMUM_TIMEOUT_SECS).contains(&timeout_secs) {
        return Err(Error::LimitExceeded);
    }
    let canonical_root = root.canonicalize().map_err(|_| Error::RootUnavailable)?;
    debug_assert!(canonical_root.is_absolute());
    let rooted_artifact =
        crate::capability::rooted_relative_path(&canonical_root, artifact_relative).ok_or(Error::UnsafePath)?;
    debug_assert!(!rooted_artifact.relative().is_absolute());
    let root_dir = crate::capability::open_root(&rooted_artifact).map_err(|_| Error::RootUnavailable)?;
    let artifact_prefix_dir = ensure_directory_path(&root_dir, rooted_artifact.relative())?;
    if path_exists(&root_dir, OCTET_TEMPORARY_POLICY_FILE)? {
        return Err(Error::RecoveryBlocked);
    }
    let (artifact_relative, artifact_dir) = reserve_probe_slot(&artifact_prefix_dir, rooted_artifact.relative())?;

    let policy_bytes = read_regular_bounded(&root_dir, std::path::Path::new(OCTET_POLICY_FILE), MAX_POLICY_BYTES)?;
    let policy_identity = blake3::hash(&policy_bytes).to_hex().to_string();
    write_new(&artifact_dir, std::path::Path::new(OCTET_BACKUP_FILE), &policy_bytes)?;

    let temporary_result = replace_policy(&root_dir, NO_DISABLED_OCTET_POLICY)
        .and_then(|()| run_octet_process(&canonical_root, &artifact_relative, timeout_secs, process));
    if !temporary_policy_is_exact(&root_dir)? {
        return Err(Error::RecoveryBlocked);
    }
    let restoration_result = restore_policy(&root_dir, &policy_bytes, &policy_identity);
    if restoration_result.is_err() {
        return Err(Error::RestorationFailed);
    }
    artifact_dir.remove_file(OCTET_BACKUP_FILE).map_err(|_| Error::RestorationFailed)?;
    let process_output = temporary_result?;

    write_new(&artifact_dir, std::path::Path::new(OCTET_STDOUT_FILE), &process_output.stdout)?;
    write_new(&artifact_dir, std::path::Path::new(OCTET_STDERR_FILE), &process_output.stderr)?;
    let summary = read_regular_bounded(&artifact_dir, std::path::Path::new(OCTET_SUMMARY_FILE), MAX_SUMMARY_BYTES)?;
    let findings = parse_findings(&summary)?;
    debug_assert!(!findings.is_empty());

    Ok(super::core::OctetSnapshot {
        schema: "lattice.guarded.octet-snapshot.v1".to_owned(),
        findings,
        restoration_confirmed: true,
        policy_identity_before: policy_identity.clone(),
        policy_identity_after: policy_identity,
    })
}

fn ensure_directory_path(root: &cap_std::fs::Dir, relative: &std::path::Path) -> Result<cap_std::fs::Dir, Error> {
    debug_assert!(!relative.is_absolute());
    let mut current = root.try_clone().map_err(|_| Error::RootUnavailable)?;
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            return Err(Error::UnsafePath);
        };
        match current.symlink_metadata(name) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(Error::SymlinkRejected);
                }
                if !metadata.is_dir() {
                    return Err(Error::UnsafePath);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                current.create_dir(name).map_err(|_| Error::RootUnavailable)?;
            }
            Err(_) => return Err(Error::RootUnavailable),
        }
        current = current.open_dir(name).map_err(|_| Error::RootUnavailable)?;
    }
    debug_assert!(!relative.is_absolute());
    Ok(current)
}

fn reserve_probe_slot(
    artifact_prefix: &cap_std::fs::Dir,
    prefix_relative: &std::path::Path,
) -> Result<(std::path::PathBuf, cap_std::fs::Dir), Error> {
    debug_assert!(!prefix_relative.is_absolute());
    for index in 0..MAX_PROBE_SLOTS {
        debug_assert!(index < MAX_PROBE_SLOTS);
        let slot_name = format!("probe-{index:0width$}", width = PROBE_SLOT_DIGITS);
        match artifact_prefix.symlink_metadata(&slot_name) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() {
                    return Err(Error::SymlinkRejected);
                }
                if !metadata.is_dir() {
                    return Err(Error::UnsafePath);
                }
                let slot = artifact_prefix.open_dir(&slot_name).map_err(|_| Error::RootUnavailable)?;
                if path_exists(&slot, OCTET_BACKUP_FILE)? {
                    return Err(Error::RecoveryBlocked);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                artifact_prefix.create_dir(&slot_name).map_err(|_| Error::RootUnavailable)?;
                let slot = artifact_prefix.open_dir(&slot_name).map_err(|_| Error::RootUnavailable)?;
                return Ok((prefix_relative.join(slot_name), slot));
            }
            Err(_) => return Err(Error::RootUnavailable),
        }
    }
    Err(Error::LimitExceeded)
}

fn path_exists(directory: &cap_std::fs::Dir, path: &str) -> Result<bool, Error> {
    match directory.symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(Error::RootUnavailable),
    }
}

fn read_regular_bounded(
    directory: &cap_std::fs::Dir,
    relative: &std::path::Path,
    max_bytes: usize,
) -> Result<Vec<u8>, Error> {
    debug_assert!(max_bytes > 0);
    let metadata = directory.symlink_metadata(relative).map_err(|_| Error::RootUnavailable)?;
    if metadata.file_type().is_symlink() {
        return Err(Error::SymlinkRejected);
    }
    if !metadata.is_file() {
        return Err(Error::UnsafePath);
    }
    let file = directory.open(relative).map_err(|_| Error::RootUnavailable)?;
    let bytes = crate::capability::read_bounded(file, max_bytes).map_err(|_| Error::RootUnavailable)?;
    if bytes.len() > max_bytes {
        return Err(Error::LimitExceeded);
    }
    debug_assert!(bytes.len() <= max_bytes);
    Ok(bytes)
}

fn write_new(directory: &cap_std::fs::Dir, relative: &std::path::Path, bytes: &[u8]) -> Result<(), Error> {
    let mut options = cap_std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    let mut file = directory.open_with(relative, &options).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            Error::ArtifactConflict
        } else {
            Error::RootUnavailable
        }
    })?;
    file.write_all(bytes).map_err(|_| Error::RootUnavailable)?;
    file.sync_all().map_err(|_| Error::RootUnavailable)
}

fn replace_policy(root: &cap_std::fs::Dir, bytes: &[u8]) -> Result<(), Error> {
    write_new(root, std::path::Path::new(OCTET_TEMPORARY_POLICY_FILE), bytes)?;
    root.rename(OCTET_TEMPORARY_POLICY_FILE, root, OCTET_POLICY_FILE)
        .map_err(|_| Error::RestorationFailed)
}

fn temporary_policy_is_exact(root: &cap_std::fs::Dir) -> Result<bool, Error> {
    let current = read_regular_bounded(root, std::path::Path::new(OCTET_POLICY_FILE), MAX_POLICY_BYTES)?;
    Ok(current == NO_DISABLED_OCTET_POLICY)
}

fn restore_policy(root: &cap_std::fs::Dir, bytes: &[u8], expected_identity: &str) -> Result<(), Error> {
    if path_exists(root, OCTET_TEMPORARY_POLICY_FILE)? {
        root.remove_file(OCTET_TEMPORARY_POLICY_FILE).map_err(|_| Error::RestorationFailed)?;
    }
    replace_policy(root, bytes)?;
    let restored = read_regular_bounded(root, std::path::Path::new(OCTET_POLICY_FILE), MAX_POLICY_BYTES)?;
    let restored_identity = blake3::hash(&restored).to_hex().to_string();
    if restored != bytes || restored_identity != expected_identity {
        return Err(Error::RestorationFailed);
    }
    Ok(())
}

fn run_octet_process(
    root: &std::path::Path,
    artifact_relative: &std::path::Path,
    timeout_secs: u64,
    process: &dyn ProbeProcess,
) -> Result<crate::agent_node::ProcessOutput, Error> {
    let artifact = artifact_relative.to_str().ok_or(Error::UnsafePath)?;
    let request = crate::agent_node::ProcessRequest {
        program: "cargo".to_owned(),
        args: vec![
            "octet".to_owned(),
            "check".to_owned(),
            "--artifact-dir".to_owned(),
            artifact.to_owned(),
        ],
        cwd: Some(root.to_string_lossy().into_owned()),
        env: std::collections::BTreeMap::new(),
        stdin: Vec::new(),
        timeout_secs,
        output_limit_bytes: MAX_PROBE_OUTPUT_BYTES,
    };
    process.run(&request)
}

fn parse_findings(summary: &[u8]) -> Result<String, Error> {
    let text = std::str::from_utf8(summary).map_err(|_| Error::MalformedObservation)?;
    let mut findings = None;
    for line in text.lines() {
        let Some(raw) = line.strip_prefix(OCTET_FINDINGS_PREFIX) else {
            continue;
        };
        if findings.is_some() {
            return Err(Error::MalformedObservation);
        }
        let value = raw.trim();
        if !valid_findings_value(value) {
            return Err(Error::MalformedObservation);
        }
        debug_assert!(value.len() <= super::core::MAX_METRIC_DIGITS);
        findings = Some(value.to_owned());
    }
    let value = findings.ok_or(Error::MalformedObservation)?;
    debug_assert!(!value.is_empty());
    Ok(value)
}

fn valid_findings_value(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= super::core::MAX_METRIC_DIGITS
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u64>().is_ok()
}
