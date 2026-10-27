fn contains_import_token(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut is_inside_string = false;
    let mut is_inside_comment = false;
    let mut is_escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if is_inside_comment {
            is_inside_comment = byte != b'\n';
            index = index.saturating_add(1);
            continue;
        }
        if is_inside_string {
            match (is_escaped, byte) {
                (true, _) => is_escaped = false,
                (false, b'\\') => is_escaped = true,
                (false, b'"') => is_inside_string = false,
                (false, _) => {}
            }
            index = index.saturating_add(1);
            continue;
        }
        if byte == b'#' {
            is_inside_comment = true;
            index = index.saturating_add(1);
            continue;
        }
        if byte == b'"' {
            is_inside_string = true;
            index = index.saturating_add(1);
            continue;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' {
            let start = index;
            index = index.saturating_add(1);
            while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_') {
                index = index.saturating_add(1);
            }
            if &bytes[start..index] == IMPORT_TOKEN {
                return true;
            }
            continue;
        }
        index = index.saturating_add(1);
    }
    false
}

fn validate_runtime_files(root: &std::path::Path, manifest: &super::model::Manifest) -> Result<(), super::Error> {
    for runtime in &manifest.runtimes {
        let content = manifest
            .contents
            .iter()
            .find(|content| content.id == runtime.content_id)
            .ok_or(super::Error::Invalid("runtime content"))?;
        let metadata =
            std::fs::symlink_metadata(root.join(&content.path)).map_err(|_| super::Error::Io("runtime observation"))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(super::Error::Invalid("runtime file type"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            if runtime.kind == "bounded_process" && metadata.permissions().mode() & EXECUTABLE_MODE_MASK == 0 {
                return Err(super::Error::Invalid("runtime executable mode"));
            }
        }
    }
    Ok(())
}

fn validate_root(root: &std::path::Path) -> Result<(), super::Error> {
    if !root.is_absolute() {
        return Err(super::Error::Invalid("pack root must be absolute"));
    }
    let metadata = std::fs::symlink_metadata(root).map_err(|_| super::Error::Io("pack root observation"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(super::Error::Invalid("pack root file type"));
    }
    Ok(())
}

fn read_bounded_regular(path: &std::path::Path, maximum_bytes: u64) -> Result<Vec<u8>, super::Error> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| super::Error::Io("pack content observation"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > maximum_bytes {
        return Err(super::Error::Invalid("pack content file"));
    }
    let file = std::fs::File::open(path).map_err(|_| super::Error::Io("pack content open"))?;
    let initial = usize::try_from(metadata.len()).unwrap_or(READ_BUFFER_BYTES).min(READ_BUFFER_BYTES);
    let mut bytes = Vec::with_capacity(initial);
    file.take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| super::Error::Io("pack content read"))?;
    if u64::try_from(bytes.len()).map_err(|_| super::Error::Invalid("pack content length"))? > maximum_bytes {
        return Err(super::Error::Invalid("pack content bound"));
    }
    Ok(bytes)
}

fn collect_files(root: &std::path::Path) -> Result<std::collections::BTreeSet<String>, super::Error> {
    let mut files = std::collections::BTreeSet::new();
    let mut pending = vec![std::path::PathBuf::new()];
    let mut observed_entries = 0usize;
    while let Some(relative) = pending.pop() {
        let directory = root.join(&relative);
        for entry in std::fs::read_dir(directory).map_err(|_| super::Error::Io("pack directory read"))? {
            observed_entries = observed_entries.saturating_add(1);
            if observed_entries > MAXIMUM_TREE_ENTRIES {
                return Err(super::Error::Invalid("pack tree bound"));
            }
            let entry = entry.map_err(|_| super::Error::Io("pack directory entry"))?;
            let name = entry.file_name();
            let next = relative.join(name);
            let next_text = next.to_str().ok_or(super::Error::Invalid("pack path encoding"))?.replace('\\', "/");
            super::validation::validate_safe_relative_path(&next_text)?;
            let metadata =
                std::fs::symlink_metadata(entry.path()).map_err(|_| super::Error::Io("pack entry observation"))?;
            if metadata.file_type().is_symlink() {
                return Err(super::Error::Invalid("pack symlink"));
            }
            if metadata.is_dir() {
                pending.push(next);
            } else if metadata.is_file() {
                files.insert(next_text);
            } else {
                return Err(super::Error::Invalid("pack file kind"));
            }
        }
    }
    Ok(files)
}
