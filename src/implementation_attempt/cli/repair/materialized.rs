//! One check's immutable copy of a captured candidate revision.
//!
//! Each check runs against its own copy, written from the host-captured
//! snapshot with every write permission removed, so no check sees the worker
//! workspace or another check's changes. After the check the copy is
//! captured again and compared with the candidate's identity; any difference
//! is drift. The copy is removed afterwards.

// r[impl lattice.implementation_attempt.repair.scenario.bypass]

/// The captured tree of one candidate revision.
pub(super) struct Tree {
    pub(super) files: std::collections::BTreeMap<String, Vec<u8>>,
    /// Unix modes of `files` at capture; only their read and execute bits
    /// are copied.
    pub(super) modes: std::collections::BTreeMap<String, u32>,
    /// `crate::candidate::snapshot_identity` of `files`.
    pub(super) identity: String,
}

/// One check's copy of a candidate tree.
pub(super) struct Copy {
    root: std::path::PathBuf,
}

impl Copy {
    /// Write `tree` under the new directory `root`, then seal it read-only.
    pub(super) fn create(root: &std::path::Path, tree: &Tree) -> Result<Self, String> {
        std::fs::create_dir(root).map_err(|error| format!("candidate copy cannot be created: {error}"))?;
        let copy = Self {
            root: root.to_path_buf(),
        };
        let mut directories = std::collections::BTreeSet::new();
        for (path, bytes) in &tree.files {
            debug_assert!(std::path::Path::new(path).is_relative(), "captured paths are relative");
            let target = root.join(path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
            }
            let mut ancestor = std::path::Path::new(path).parent();
            while let Some(directory) = ancestor.filter(|directory| !directory.as_os_str().is_empty()) {
                directories.insert(directory.to_path_buf());
                ancestor = directory.parent();
            }
            write_new(&target, bytes)?;
            seal(&target, tree.modes.get(path).copied().unwrap_or(0o444))?;
        }
        // Deepest first, so every directory is still writable while its
        // children are sealed.
        let mut ordered: Vec<_> = directories.into_iter().collect();
        ordered.sort_by_key(|directory| std::cmp::Reverse(directory.components().count()));
        for directory in ordered {
            seal(&root.join(directory), 0o555)?;
        }
        seal(root, 0o555)?;
        Ok(copy)
    }

    pub(super) fn path(&self) -> &std::path::Path {
        &self.root
    }

    /// Whether the copy still captures to `identity`. A copy that can no
    /// longer be captured has drifted.
    pub(super) fn is_intact(&self, identity: &str) -> bool {
        crate::candidate_capture::workspace(&self.root)
            .ok()
            .and_then(|snapshot| crate::candidate::snapshot_identity(&snapshot).ok())
            .is_some_and(|captured| captured == identity)
    }

    /// Make every directory writable again and remove the copy.
    pub(super) fn remove(self) -> Result<(), String> {
        unseal(&self.root)?;
        std::fs::remove_dir_all(&self.root).map_err(|error| format!("candidate copy cannot be removed: {error}"))
    }
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())
}

/// Keep only the read and execute bits of `mode`.
fn seal(path: &std::path::Path, mode: u32) -> Result<(), String> {
    set_mode(path, mode & 0o555)
}

fn set_mode(path: &std::path::Path, mode: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(|error| error.to_string())?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

/// Make `root` and every directory below it, never following a symlink,
/// owner-writable so the copy can be removed. Each directory is unsealed
/// before it is listed.
fn unseal(root: &std::path::Path) -> Result<(), String> {
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        set_mode(&directory, 0o700)?;
        for entry in std::fs::read_dir(&directory).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();
            if std::fs::symlink_metadata(&path).map_err(|error| error.to_string())?.is_dir() {
                pending.push(path);
            }
        }
    }
    Ok(())
}
