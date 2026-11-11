//! Host-side bounded workspace capture for candidate receipts.
//!
//! This is an imperative shell helper. It reads a selected root, rejects
//! symlinks and non-UTF-8 paths, skips VCS/build metadata, and returns a sorted
//! file map suitable for `candidate::produced_receipt_from_snapshots`.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use thiserror::Error;

use crate::candidate::MAX_SNAPSHOT_BYTES;
use crate::candidate::MAX_SNAPSHOT_FILES;

const IGNORED_DIRECTORIES: &[&str] = &[".git", ".omp", ".pi", "target"];
const IGNORED_FILES: &[&str] = &[".DS_Store", "Thumbs.db"];

/// Capture a bounded deterministic map of relative UTF-8 paths to file bytes.
pub fn capture_workspace(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, CandidateCaptureError> {
    let metadata = std::fs::symlink_metadata(root).map_err(CandidateCaptureError::Io)?;
    if !metadata.is_dir() {
        return Err(CandidateCaptureError::RootNotDirectory);
    }
    let mut snapshot = BTreeMap::new();
    let mut total_bytes = 0usize;
    let mut stack = vec![(root.to_path_buf(), PathBuf::new())];
    while let Some((directory, relative)) = stack.pop() {
        let entries = std::fs::read_dir(&directory).map_err(CandidateCaptureError::Io)?;
        for entry in entries {
            let entry = entry.map_err(CandidateCaptureError::Io)?;
            let file_name = entry.file_name();
            let name = file_name.to_str().ok_or(CandidateCaptureError::NonUtf8Path)?;
            let metadata = std::fs::symlink_metadata(entry.path()).map_err(CandidateCaptureError::Io)?;
            if metadata.file_type().is_symlink() {
                return Err(CandidateCaptureError::SymlinkRejected);
            }
            let relative_path = relative.join(name);
            if metadata.is_dir() {
                if !IGNORED_DIRECTORIES.contains(&name) {
                    stack.push((entry.path(), relative_path));
                }
                continue;
            }
            if !metadata.is_file() || IGNORED_FILES.contains(&name) {
                continue;
            }
            if snapshot.len() >= MAX_SNAPSHOT_FILES {
                return Err(CandidateCaptureError::TooLarge);
            }
            let bytes = std::fs::read(entry.path()).map_err(CandidateCaptureError::Io)?;
            total_bytes = total_bytes.checked_add(bytes.len()).ok_or(CandidateCaptureError::TooLarge)?;
            if total_bytes > MAX_SNAPSHOT_BYTES {
                return Err(CandidateCaptureError::TooLarge);
            }
            let key = path_key(&relative_path)?;
            snapshot.insert(key, bytes);
        }
    }
    Ok(snapshot)
}

/// Admitted workspace boundary for one candidate-producing attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceBoundary {
    pub root: PathBuf,
    pub read_paths: Vec<String>,
    pub write_paths: Vec<String>,
    pub network: bool,
    pub credentials: bool,
}

/// Admit relative read/write paths under an existing root without symlink escape.
pub fn admit_workspace_boundary(
    root: &Path,
    read_paths: &[String],
    write_paths: &[String],
    network: bool,
    credentials: bool,
) -> Result<WorkspaceBoundary, CandidateCaptureError> {
    let root = std::fs::canonicalize(root).map_err(CandidateCaptureError::Io)?;
    let metadata = std::fs::symlink_metadata(&root).map_err(CandidateCaptureError::Io)?;
    if !metadata.is_dir() {
        return Err(CandidateCaptureError::RootNotDirectory);
    }
    let read_paths = canonical_relative_paths(&root, read_paths)?;
    let write_paths = canonical_relative_paths(&root, write_paths)?;
    for write in &write_paths {
        if !read_paths.iter().any(|read| write == read || write.starts_with(&format!("{read}/"))) {
            return Err(CandidateCaptureError::WriteOutsideRead);
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

fn canonical_relative_paths(root: &Path, paths: &[String]) -> Result<Vec<String>, CandidateCaptureError> {
    let mut admitted = Vec::with_capacity(paths.len());
    for path in paths {
        if path.is_empty() || path.starts_with('/') || path.starts_with('\\') {
            return Err(CandidateCaptureError::InvalidPath);
        }
        let mut parts = Vec::new();
        for component in Path::new(path).components() {
            let std::path::Component::Normal(part) = component else {
                return Err(CandidateCaptureError::InvalidPath);
            };
            parts.push(part.to_str().ok_or(CandidateCaptureError::NonUtf8Path)?);
        }
        if parts.is_empty() {
            return Err(CandidateCaptureError::InvalidPath);
        }
        if parts.iter().any(|part| matches!(*part, ".git" | ".jj")) {
            return Err(CandidateCaptureError::VcsMetadataRejected);
        }
        let candidate = root.join(parts.iter().collect::<PathBuf>());
        let canonical = match std::fs::canonicalize(&candidate) {
            Ok(path) => path,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let parent = candidate.parent().ok_or(CandidateCaptureError::InvalidPath)?;
                let parent = std::fs::canonicalize(parent).map_err(CandidateCaptureError::Io)?;
                parent.join(candidate.file_name().ok_or(CandidateCaptureError::InvalidPath)?)
            }
            Err(error) => return Err(CandidateCaptureError::Io(error)),
        };
        if !canonical.starts_with(root) {
            return Err(CandidateCaptureError::BoundaryEscape);
        }
        admitted.push(parts.join("/"));
    }
    admitted.sort();
    if admitted.windows(2).any(|window| window[0] == window[1]) {
        return Err(CandidateCaptureError::DuplicateBoundaryPath);
    }
    Ok(admitted)
}

fn path_key(path: &Path) -> Result<String, CandidateCaptureError> {
    let mut parts = Vec::new();
    for component in path.components() {
        let std::path::Component::Normal(part) = component else {
            return Err(CandidateCaptureError::InvalidPath);
        };
        parts.push(part.to_str().ok_or(CandidateCaptureError::NonUtf8Path)?);
    }
    if parts.is_empty() {
        return Err(CandidateCaptureError::InvalidPath);
    }
    Ok(parts.join("/"))
}

#[derive(Debug, Error)]
pub enum CandidateCaptureError {
    #[error("workspace root is not a directory")]
    RootNotDirectory,
    #[error("workspace path is not valid UTF-8")]
    NonUtf8Path,
    #[error("workspace contains a symlink")]
    SymlinkRejected,
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
mod tests {
    use super::*;

    #[test]
    fn captures_sorted_files_and_skips_vcs_and_build_metadata() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src/nested")).expect("source directories");
        std::fs::create_dir_all(root.join(".git")).expect("git directory");
        std::fs::create_dir_all(root.join("target/debug")).expect("target directory");
        std::fs::write(root.join("src/main.rs"), b"main").expect("main source");
        std::fs::write(root.join("src/nested/lib.rs"), b"lib").expect("nested source");
        std::fs::write(root.join("README.md"), b"readme").expect("readme");
        std::fs::write(root.join(".git/config"), b"git").expect("git metadata");
        std::fs::write(root.join("target/debug/out"), b"build").expect("build artifact");
        std::fs::write(root.join(".DS_Store"), b"host").expect("host metadata");

        let snapshot = capture_workspace(root).expect("workspace captures");

        assert_eq!(snapshot.keys().collect::<Vec<_>>(), vec!["README.md", "src/main.rs", "src/nested/lib.rs"]);
        assert_eq!(snapshot["src/main.rs"], b"main");
    }

    #[test]
    fn skips_agent_runtime_metadata() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join(".omp/agent")).expect("OMP metadata directory");
        std::fs::write(root.join(".omp/agent/models.yml"), b"models").expect("OMP model metadata");
        std::fs::write(root.join("src.rs"), b"source").expect("source file");

        let snapshot = capture_workspace(root).expect("workspace captures");

        assert_eq!(snapshot.keys().collect::<Vec<_>>(), vec!["src.rs"]);
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::write(root.join("real"), b"real").expect("real file");
        std::os::unix::fs::symlink(root.join("real"), root.join("link")).expect("symlink");

        assert!(matches!(capture_workspace(root), Err(CandidateCaptureError::SymlinkRejected)));
    }

    #[test]
    fn admits_read_and_write_paths_within_the_root() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src/nested")).expect("source directory");
        std::fs::write(root.join("src/main.rs"), b"main").expect("source file");

        let boundary = admit_workspace_boundary(
            root,
            &["src".to_string(), "src/nested".to_string()],
            &["src".to_string(), "src/new.rs".to_string()],
            false,
            false,
        )
        .expect("boundary admits");
        assert_eq!(boundary.read_paths, vec!["src", "src/nested"]);
        assert_eq!(boundary.write_paths, vec!["src", "src/new.rs"]);
        assert!(!boundary.network);
        assert!(!boundary.credentials);
    }

    #[test]
    fn rejects_vcs_metadata_boundaries() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join(".git")).expect("git metadata");
        std::fs::create_dir_all(root.join("src")).expect("source directory");

        assert!(matches!(
            admit_workspace_boundary(root, &[".git".to_string()], &[], false, false),
            Err(CandidateCaptureError::VcsMetadataRejected)
        ));
        assert!(matches!(
            admit_workspace_boundary(root, &["src".to_string()], &["src/.git/config".to_string()], false, false),
            Err(CandidateCaptureError::VcsMetadataRejected)
        ));
    }

    #[test]
    fn rejects_escaping_outside_and_uncovered_write_paths() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        std::fs::create_dir_all(root.join("src")).expect("source directory");

        assert!(matches!(
            admit_workspace_boundary(root, &["../outside".to_string()], &[], false, false),
            Err(CandidateCaptureError::InvalidPath)
        ));
        assert!(matches!(
            admit_workspace_boundary(root, &["src".to_string()], &["docs".to_string()], false, false),
            Err(CandidateCaptureError::WriteOutsideRead)
        ));
        assert!(matches!(
            admit_workspace_boundary(root, &["src".to_string(), "src".to_string()], &[], false, false),
            Err(CandidateCaptureError::DuplicateBoundaryPath)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape_from_the_boundary() {
        let directory = tempfile::tempdir().expect("temporary workspace");
        let root = directory.path();
        let outside = tempfile::tempdir().expect("outside directory");
        std::os::unix::fs::symlink(outside.path(), root.join("escape")).expect("symlink");

        assert!(matches!(
            admit_workspace_boundary(root, &["escape".to_string()], &[], false, false),
            Err(CandidateCaptureError::BoundaryEscape)
        ));
    }
}
