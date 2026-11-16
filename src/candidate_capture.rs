//! Host-side bounded workspace capture for candidate receipts.
//!
//! This is an imperative shell helper. It reads a selected root, rejects
//! symlinks, special files and non-UTF-8 paths, excludes only root VCS metadata,
//! and returns a sorted file map suitable for host-derived candidate receipts.

mod traversal;

use thiserror::Error;

use crate::candidate::MAX_SNAPSHOT_BYTES;

// r[impl lattice.candidate_worker.file_capture]
/// Capture a bounded deterministic map of relative UTF-8 paths to file bytes.
pub fn workspace(root: &std::path::Path) -> Result<std::collections::BTreeMap<String, Vec<u8>>, Error> {
    let metadata = std::fs::symlink_metadata(root).map_err(Error::Io)?;
    if !metadata.is_dir() {
        return Err(Error::RootNotDirectory);
    }
    traversal::snapshot(root)
}

fn read_file(path: &std::path::Path, metadata: &std::fs::Metadata, total_bytes: usize) -> Result<Vec<u8>, Error> {
    use std::io::Read;
    let remaining = MAX_SNAPSHOT_BYTES.checked_sub(total_bytes).ok_or(Error::TooLarge)?;
    let remaining = u64::try_from(remaining).map_err(|_| Error::TooLarge)?;
    if metadata.len() > remaining {
        return Err(Error::TooLarge);
    }
    let read_limit_bytes = remaining.checked_add(1).ok_or(Error::TooLarge)?;
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(Error::Io)?
        .take(read_limit_bytes)
        .read_to_end(&mut bytes)
        .map_err(Error::Io)?;
    Ok(bytes)
}

/// Byte receipts cannot encode Unix mode changes; check and reject those separately.
pub(crate) fn snapshot_modes(
    root: &std::path::Path,
    snapshot: &std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<std::collections::BTreeMap<String, u32>, Error> {
    snapshot
        .keys()
        .map(|path| {
            let metadata = std::fs::symlink_metadata(root.join(path)).map_err(Error::Io)?;
            #[cfg(unix)]
            let mode = {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o7777
            };
            #[cfg(not(unix))]
            let mode = u32::from(metadata.permissions().readonly());
            Ok((path.clone(), mode))
        })
        .collect()
}

/// Admitted workspace boundary for one candidate-producing attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBoundary {
    pub root: std::path::PathBuf,
    pub read_paths: Vec<String>,
    pub write_paths: Vec<String>,
    pub network: bool,
    pub credentials: bool,
}

/// Admit relative read/write paths under an existing root without symlink escape.
pub fn admit_workspace_boundary(
    root: &std::path::Path,
    read_paths: &[String],
    write_paths: &[String],
    network: bool,
    credentials: bool,
) -> Result<WorkspaceBoundary, Error> {
    let root = std::fs::canonicalize(root).map_err(Error::Io)?;
    let metadata = std::fs::symlink_metadata(&root).map_err(Error::Io)?;
    if !metadata.is_dir() {
        return Err(Error::RootNotDirectory);
    }
    let read_paths = canonical_relative_paths(&root, read_paths)?;
    let write_paths = canonical_relative_paths(&root, write_paths)?;
    for write in &write_paths {
        if !read_paths.iter().any(|read| write == read || write.starts_with(&format!("{read}/"))) {
            return Err(Error::WriteOutsideRead);
        }
    }
    Ok(WorkspaceBoundary {
        root,
        read_paths,
        write_paths,
        network,
        credentials,
    })
}

fn canonical_relative_paths(root: &std::path::Path, paths: &[String]) -> Result<Vec<String>, Error> {
    let mut admitted = Vec::with_capacity(paths.len());
    for path in paths {
        if path.is_empty() || path.starts_with('/') || path.starts_with('\\') {
            return Err(Error::InvalidPath);
        }
        let mut parts = Vec::new();
        for component in std::path::Path::new(path).components() {
            let std::path::Component::Normal(part) = component else {
                return Err(Error::InvalidPath);
            };
            parts.push(part.to_str().ok_or(Error::NonUtf8Path)?);
        }
        if parts.is_empty() {
            return Err(Error::InvalidPath);
        }
        if parts.iter().any(|part| matches!(*part, ".git" | ".jj")) {
            return Err(Error::VcsMetadataRejected);
        }
        let candidate = root.join(parts.iter().collect::<std::path::PathBuf>());
        let canonical = match std::fs::canonicalize(&candidate) {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let parent = candidate.parent().ok_or(Error::InvalidPath)?;
                let parent = std::fs::canonicalize(parent).map_err(Error::Io)?;
                parent.join(candidate.file_name().ok_or(Error::InvalidPath)?)
            }
            Err(error) => return Err(Error::Io(error)),
        };
        if !canonical.starts_with(root) {
            return Err(Error::BoundaryEscape);
        }
        admitted.push(parts.join("/"));
    }
    admitted.sort();
    if admitted.windows(2).any(|window| window[0] == window[1]) {
        return Err(Error::DuplicateBoundaryPath);
    }
    Ok(admitted)
}

fn path_key(path: &std::path::Path) -> Result<String, Error> {
    let mut parts = Vec::new();
    for component in path.components() {
        let std::path::Component::Normal(part) = component else {
            return Err(Error::InvalidPath);
        };
        parts.push(part.to_str().ok_or(Error::NonUtf8Path)?);
    }
    if parts.is_empty() {
        return Err(Error::InvalidPath);
    }
    let key = parts.join("/");
    crate::candidate::validate_candidate_path(&key).map_err(|_| Error::InvalidPath)?;
    Ok(key)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("workspace root is not a directory")]
    RootNotDirectory,
    #[error("workspace path is not valid UTF-8")]
    NonUtf8Path,
    #[error("workspace contains a symlink")]
    SymlinkRejected,
    #[error("workspace contains an unsupported special file")]
    SpecialFileRejected,
    #[error("workspace contains a multiply linked file")]
    HardLinkRejected,
    #[error("workspace path is not a normal relative component")]
    InvalidPath,
    #[error("workspace snapshot exceeds the admitted file or byte bound")]
    TooLarge,
    #[error("workspace boundary path escapes the admitted root")]
    BoundaryEscape,
    #[error("workspace write path is not covered by a read path")]
    WriteOutsideRead,
    #[error("workspace boundary path is duplicated")]
    DuplicateBoundaryPath,
    #[error("workspace boundary does not admit VCS metadata")]
    VcsMetadataRejected,
    #[error("workspace read failed: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests;

#[cfg(all(test, unix))]
mod links;

#[cfg(test)]
mod bounds;
