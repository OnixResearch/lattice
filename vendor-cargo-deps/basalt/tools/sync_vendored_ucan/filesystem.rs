use crate::model::{
    EXECUTABLE_FILE_MODE, EXECUTABLE_MODE_MASK, MAXIMUM_SNAPSHOT_BYTES,
    MAXIMUM_SNAPSHOT_ENTRIES, REGULAR_FILE_MODE, SnapshotEntry,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};

pub(crate) fn collect_snapshot_entries(
    snapshot_root: &Path,
) -> Result<Vec<SnapshotEntry>, String> {
    if !snapshot_root.is_dir() {
        return Err(format!(
            "snapshot directory does not exist: {}",
            snapshot_root.display()
        ));
    }
    let mut directories = vec![snapshot_root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = directories.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("cannot enumerate {}: {error}", directory.display()))?;
        children.sort_by_key(std::fs::DirEntry::file_name);
        for child in children {
            let file_type = child
                .file_type()
                .map_err(|error| format!("cannot inspect {}: {error}", child.path().display()))?;
            if file_type.is_symlink() {
                return Err(format!(
                    "snapshot symlink is not admitted: {}",
                    child.path().display()
                ));
            }
            if file_type.is_dir() {
                directories.push(child.path());
            } else if file_type.is_file() {
                files.push(child.path());
            } else {
                return Err(format!(
                    "snapshot entry has unsupported type: {}",
                    child.path().display()
                ));
            }
            if files.len() + directories.len() > MAXIMUM_SNAPSHOT_ENTRIES {
                return Err(format!(
                    "snapshot exceeds entry limit {MAXIMUM_SNAPSHOT_ENTRIES}"
                ));
            }
        }
    }
    files.sort();
    build_snapshot_entries(snapshot_root, &files)
}

fn build_snapshot_entries(
    snapshot_root: &Path,
    files: &[PathBuf],
) -> Result<Vec<SnapshotEntry>, String> {
    let mut total_bytes = 0_u64;
    let mut entries = Vec::with_capacity(files.len());
    for file in files {
        let metadata = fs::metadata(file)
            .map_err(|error| format!("cannot read metadata for {}: {error}", file.display()))?;
        total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| String::from("snapshot byte total overflowed"))?;
        if total_bytes > MAXIMUM_SNAPSHOT_BYTES {
            return Err(format!(
                "snapshot exceeds byte limit {MAXIMUM_SNAPSHOT_BYTES}"
            ));
        }
        let relative = safe_relative_path(snapshot_root, file)?;
        let mode = if metadata.permissions().mode() & EXECUTABLE_MODE_MASK == 0 {
            REGULAR_FILE_MODE
        } else {
            EXECUTABLE_FILE_MODE
        };
        entries.push(SnapshotEntry {
            mode,
            digest: crate::process::blake3_file(file)?,
            path: relative,
            byte_len: metadata.len(),
        });
    }
    assert_eq!(entries.len(), files.len());
    Ok(entries)
}

fn safe_relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| format!("{} escaped {}: {error}", path.display(), root.display()))?;
    let mut components = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => {
                let text = value
                    .to_str()
                    .ok_or_else(|| format!("snapshot path is not UTF-8: {}", path.display()))?;
                if text.chars().any(char::is_control) || text.contains('\t') {
                    return Err(format!(
                        "snapshot path contains a control character: {}",
                        path.display()
                    ));
                }
                components.push(text);
            }
            _ => {
                return Err(format!(
                    "snapshot path is not normalized: {}",
                    path.display()
                ));
            }
        }
    }
    if components.is_empty() {
        return Err(format!(
            "snapshot file has an empty relative path: {}",
            path.display()
        ));
    }
    Ok(components.join("/"))
}

pub(crate) fn compare_text_file(path: &Path, expected: &str) -> Result<(), String> {
    let actual = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if actual != expected {
        return Err(format!(
            "{} is stale; run `tools/sync-vendored-ucan.rs --import SOURCE_REPOSITORY`",
            path.display()
        ));
    }
    Ok(())
}

pub(crate) fn remove_directory_if_present(path: &Path) -> Result<(), String> {
    if path.exists() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("cannot remove {}: {error}", path.display()))?;
    }
    Ok(())
}
