struct PendingDirectory {
    directory: cap_std::fs::Dir,
    relative: std::path::PathBuf,
}

struct CaptureState {
    pending: Vec<PendingDirectory>,
    directory_count: usize,
    files: std::collections::BTreeMap<String, Vec<u8>>,
}

fn capture_files(
    bundle_dir: &cap_std::fs::Dir,
    safe_name: &str,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>, super::model::Diagnostic> {
    let root = bundle_dir.try_clone().map_err(|_| {
        super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::FileAccess,
            "bundle directory could not be captured",
        )
    })?;
    let mut state = CaptureState {
        pending: Vec::with_capacity(super::model::MAX_BUNDLE_DIRECTORIES),
        directory_count: 1,
        files: std::collections::BTreeMap::new(),
    };
    state.pending.push(PendingDirectory {
        directory: root,
        relative: std::path::PathBuf::new(),
    });
    while let Some(current) = state.pending.pop() {
        capture_directory_entries(bundle_dir, current, &mut state, safe_name)?;
    }
    let total_bytes =
        state
            .files
            .values()
            .try_fold(0usize, |total, bytes| total.checked_add(bytes.len()))
            .ok_or_else(|| {
                super::core::diagnostic(
                    safe_name,
                    super::model::DiagnosticCode::BundleTooLarge,
                    "bundle byte count overflowed",
                )
            })?;
    if total_bytes > super::model::MAX_BUNDLE_BYTES {
        return Err(super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::BundleTooLarge,
            "bundle byte limit exceeded",
        ));
    }
    debug_assert!(state.directory_count <= super::model::MAX_BUNDLE_DIRECTORIES);
    debug_assert!(state.files.len() <= super::model::MAX_BUNDLE_FILES.saturating_add(1));
    Ok(state.files)
}

fn capture_directory_entries(
    bundle_dir: &cap_std::fs::Dir,
    current: PendingDirectory,
    state: &mut CaptureState,
    safe_name: &str,
) -> Result<(), super::model::Diagnostic> {
    let entries = current.directory.entries().map_err(|_| {
        super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::FileAccess,
            "bundle directory could not be enumerated",
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|_| {
            super::core::diagnostic(
                safe_name,
                super::model::DiagnosticCode::FileAccess,
                "bundle entry could not be inspected",
            )
        })?;
        let file_type = entry.file_type().map_err(|_| {
            super::core::diagnostic(
                safe_name,
                super::model::DiagnosticCode::FileAccess,
                "bundle entry type could not be inspected",
            )
        })?;
        if file_type.is_symlink() {
            return Err(super::core::diagnostic(
                safe_name,
                super::model::DiagnosticCode::SymlinkRejected,
                "bundle symlink is not admitted",
            ));
        }
        let relative_path = current.relative.join(entry.file_name());
        if !super::core::safe_relative_path(&relative_path.to_string_lossy()) {
            return Err(super::core::diagnostic(
                safe_name,
                super::model::DiagnosticCode::UnsafePath,
                "bundle path is unsafe",
            ));
        }
        if file_type.is_dir() {
            capture_subdirectory(bundle_dir, relative_path, state, safe_name)?;
            continue;
        }
        capture_regular_file(bundle_dir, &relative_path, file_type.is_file(), &mut state.files, safe_name)?;
    }
    debug_assert!(state.directory_count <= super::model::MAX_BUNDLE_DIRECTORIES);
    debug_assert!(state.files.len() <= super::model::MAX_BUNDLE_FILES.saturating_add(1));
    Ok(())
}

fn capture_subdirectory(
    bundle_dir: &cap_std::fs::Dir,
    relative_path: std::path::PathBuf,
    state: &mut CaptureState,
    safe_name: &str,
) -> Result<(), super::model::Diagnostic> {
    state.directory_count = state.directory_count.saturating_add(1);
    if state.directory_count > super::model::MAX_BUNDLE_DIRECTORIES {
        return Err(super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::TooManyFiles,
            "bundle directory count exceeds its limit",
        ));
    }
    let directory = bundle_dir.open_dir(&relative_path).map_err(|_| {
        super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::FileAccess,
            "bundle subdirectory could not be opened",
        )
    })?;
    state.pending.push(PendingDirectory {
        directory,
        relative: relative_path,
    });
    debug_assert!(state.directory_count <= super::model::MAX_BUNDLE_DIRECTORIES);
    debug_assert!(state.pending.len() <= super::model::MAX_BUNDLE_DIRECTORIES);
    Ok(())
}

fn capture_regular_file(
    bundle_dir: &cap_std::fs::Dir,
    relative_path: &std::path::Path,
    is_regular_file: bool,
    files: &mut std::collections::BTreeMap<String, Vec<u8>>,
    safe_name: &str,
) -> Result<(), super::model::Diagnostic> {
    if !is_regular_file {
        return Err(super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::NonRegularFile,
            "bundle entry is not a regular file",
        ));
    }
    if files.len() > super::model::MAX_BUNDLE_FILES {
        return Err(super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::TooManyFiles,
            "bundle file count exceeds its limit",
        ));
    }
    let file = crate::capability::open_relative_read(bundle_dir, relative_path).map_err(|_| {
        super::core::diagnostic(safe_name, super::model::DiagnosticCode::FileAccess, "bundle file could not be opened")
    })?;
    let bytes = crate::capability::read_bounded(file, super::model::MAX_FILE_BYTES).map_err(|_| {
        super::core::diagnostic(safe_name, super::model::DiagnosticCode::FileAccess, "bundle file could not be read")
    })?;
    if bytes.len() > super::model::MAX_FILE_BYTES {
        return Err(super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::FileTooLarge,
            "bundle file byte limit exceeded",
        ));
    }
    let path = relative_path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
    if files.insert(path, bytes).is_some() {
        return Err(super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::DuplicatePath,
            "bundle path is duplicated",
        ));
    }
    debug_assert!(files.len() <= super::model::MAX_BUNDLE_FILES.saturating_add(1));
    debug_assert!(files.values().all(|value| value.len() <= super::model::MAX_FILE_BYTES));
    Ok(())
}

fn parse_manifest(bytes: &[u8], safe_name: &str) -> Result<super::model::BundleManifest, super::model::Diagnostic> {
    let source = std::str::from_utf8(bytes).map_err(|_| {
        super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::InvalidManifest,
            "bundle manifest is not UTF-8",
        )
    })?;
    let value = crate::nickel::evaluate_source(source).map_err(|_| {
        super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::NickelEvaluation,
            "bundle manifest evaluation failed",
        )
    })?;
    let manifest = serde_json::from_value(value).map_err(|_| {
        super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::InvalidManifest,
            "bundle manifest schema is invalid",
        )
    })?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(source.len() <= super::model::MAX_FILE_BYTES);
    Ok(manifest)
}

fn parse_graph(
    manifest: &super::model::BundleManifest,
    captured: &std::collections::BTreeMap<String, Vec<u8>>,
    assets: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Option<crate::engine::Workflow> {
    let bytes = captured.get(&manifest.workflow.path)?;
    let source = std::str::from_utf8(bytes).ok()?;
    let value = crate::nickel::evaluate_source(source).ok()?;
    crate::workflow_format::from_nickel_value(value, assets).ok()
}

fn parse_orchestration(
    manifest: &super::model::BundleManifest,
    captured: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Option<crate::orchestration::OrchestrationDefinition> {
    let binding = manifest.orchestration.as_ref()?;
    let bytes = captured.get(&binding.path)?;
    let source = std::str::from_utf8(bytes).ok()?;
    let value = crate::nickel::evaluate_source(source).ok()?;
    crate::orchestration::definition_from_nickel_value(value).ok()
}

fn safe_directory_label(name: &std::ffi::OsString) -> String {
    let value = name.to_string_lossy();
    if super::core::safe_relative_path(&value)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.'))
    {
        value.into_owned()
    } else {
        "invalid-bundle".to_string()
    }
}
