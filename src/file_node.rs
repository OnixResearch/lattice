//! File node operation planning and filesystem interface.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::Mutex;
use serde::Deserialize;
use serde::Serialize;

mod local;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperationKind {
    Read,
    Write,
    Append,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileError {
    UnsafePath(String),
    MissingFile(String),
    AlreadyExists(String),
    PermissionDenied(String),
    Io(String),
}

impl std::fmt::Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsafePath(path) => write!(f, "unsafe file path: {path}"),
            Self::MissingFile(path) => write!(f, "missing file: {path}"),
            Self::AlreadyExists(path) => write!(f, "file already exists: {path}"),
            Self::PermissionDenied(path) => write!(f, "permission denied: {path}"),
            Self::Io(message) => write!(f, "file I/O error: {message}"),
        }
    }
}

impl std::error::Error for FileError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileOperationPlan {
    pub kind: FileOperationKind,
    pub path: String,
    pub content: Option<String>,
    pub overwrite: bool,
    pub allowed_roots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileOperationResult {
    Read { content: String, bytes: usize },
    Written { bytes: usize },
}

pub trait FileSystem: Send + Sync {
    fn execute(&self, plan: &FileOperationPlan) -> Result<FileOperationResult, FileError>;
}

#[derive(Debug, Default)]
pub struct NoopFileSystem;

impl FileSystem for NoopFileSystem {
    fn execute(&self, _plan: &FileOperationPlan) -> Result<FileOperationResult, FileError> {
        Err(FileError::Io("no filesystem configured".to_string()))
    }
}

// r[impl n8nclone.workflow_execution.llm_operating_graph.scenario.production-file-effects]
/// Production filesystem shell for explicit file-node operations.
#[derive(Debug, Default)]
pub struct LocalFs;

impl FileSystem for LocalFs {
    fn execute(&self, plan: &FileOperationPlan) -> Result<FileOperationResult, FileError> {
        local::execute(plan)
    }
}

fn io_error(path: &str, error: std::io::Error) -> FileError {
    match error.kind() {
        std::io::ErrorKind::NotFound => FileError::MissingFile(path.to_string()),
        std::io::ErrorKind::AlreadyExists => FileError::AlreadyExists(path.to_string()),
        std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied(path.to_string()),
        _ => FileError::Io(format!("{path}: {error}")),
    }
}

#[derive(Debug, Clone, Default)]
pub struct MemoryFileSystem {
    files: Arc<Mutex<HashMap<String, String>>>,
    denied_paths: Arc<Mutex<Vec<String>>>,
}

impl MemoryFileSystem {
    pub fn new(files: HashMap<String, String>) -> Self {
        Self {
            files: Arc::new(Mutex::new(files)),
            denied_paths: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn deny_path(&self, path: impl Into<String>) {
        self.denied_paths.lock().push(path.into());
    }

    pub fn read_text(&self, path: &str) -> Option<String> {
        self.files.lock().get(path).cloned()
    }
}

impl FileSystem for MemoryFileSystem {
    fn execute(&self, plan: &FileOperationPlan) -> Result<FileOperationResult, FileError> {
        validate_path_allowed(&plan.path, &plan.allowed_roots)?;
        if self.denied_paths.lock().contains(&plan.path) {
            return Err(FileError::PermissionDenied(plan.path.clone()));
        }

        match plan.kind {
            FileOperationKind::Read => {
                let files = self.files.lock();
                let content =
                    files.get(&plan.path).cloned().ok_or_else(|| FileError::MissingFile(plan.path.clone()))?;
                let bytes = content.len();
                Ok(FileOperationResult::Read { content, bytes })
            }
            FileOperationKind::Write => {
                let mut files = self.files.lock();
                if !plan.overwrite && files.contains_key(&plan.path) {
                    return Err(FileError::AlreadyExists(plan.path.clone()));
                }
                let content = plan.content.clone().unwrap_or_default();
                let bytes = content.len();
                files.insert(plan.path.clone(), content);
                Ok(FileOperationResult::Written { bytes })
            }
            FileOperationKind::Append => {
                let mut files = self.files.lock();
                let content = plan.content.clone().unwrap_or_default();
                let bytes = content.len();
                files.entry(plan.path.clone()).or_default().push_str(&content);
                Ok(FileOperationResult::Written { bytes })
            }
        }
    }
}

// r[impl n8nclone.workflow_execution.file_nodes.safety]
pub fn validate_path_allowed(path: &str, allowed_roots: &[String]) -> Result<(), FileError> {
    if allowed_roots.is_empty() {
        return Ok(());
    }
    let candidate = std::path::Path::new(path);
    if allowed_roots.iter().any(|root| candidate.starts_with(std::path::Path::new(root))) {
        Ok(())
    } else {
        Err(FileError::UnsafePath(path.to_string()))
    }
}
