//! One check's immutable copy of a captured candidate revision.
//!
//! Each check runs against its own copy, written from the host-captured
//! snapshot with every write permission removed, so no check sees the worker
//! workspace or another check's changes. The copy records the type and mode
//! it sealed for every file and directory. After the check the copy is
//! captured again and compared with the candidate's identity, and every
//! sealed type and mode is read again; any difference is drift. The copy is
//! removed afterwards.

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

/// The type and permission bits one path of a copy was sealed with.
struct Sealed {
    path: std::path::PathBuf,
    mode: u32,
    is_directory: bool,
}

impl Sealed {
    /// Whether `path` is still, without following a symlink, the sealed
    /// type with exactly the sealed permission bits.
    fn is_unchanged(&self) -> bool {
        let Ok(metadata) = std::fs::symlink_metadata(&self.path) else {
            return false;
        };
        let file_type = metadata.file_type();
        let is_same_type = if self.is_directory {
            file_type.is_dir()
        } else {
            file_type.is_file()
        };
        #[cfg(unix)]
        let is_same_mode = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o7777 == self.mode
        };
        #[cfg(not(unix))]
        let is_same_mode = metadata.permissions().readonly();
        is_same_type && is_same_mode
    }
}

/// One check's copy of a candidate tree.
pub(super) struct Copy {
    root: std::path::PathBuf,
    /// Every file, every directory and the root, as sealed.
    sealed: Vec<Sealed>,
}

impl Copy {
    /// Write `tree` under the new directory `root`, then seal it read-only.
    pub(super) fn create(root: &std::path::Path, tree: &Tree) -> Result<Self, String> {
        std::fs::create_dir(root).map_err(|error| format!("candidate copy cannot be created: {error}"))?;
        let mut copy = Self {
            root: root.to_path_buf(),
            sealed: Vec::with_capacity(tree.files.len().saturating_add(1)),
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
            copy.seal(target, tree.modes.get(path).copied().unwrap_or(0o444), false)?;
        }
        // Deepest first, so every directory is still writable while its
        // children are sealed.
        let mut ordered: Vec<_> = directories.into_iter().collect();
        ordered.sort_by_key(|directory| std::cmp::Reverse(directory.components().count()));
        for directory in ordered {
            copy.seal(root.join(directory), 0o555, true)?;
        }
        copy.seal(root.to_path_buf(), 0o555, true)?;
        Ok(copy)
    }

    /// Keep only the read and execute bits of `mode` on `path` and record
    /// them with its type.
    fn seal(&mut self, path: std::path::PathBuf, mode: u32, is_directory: bool) -> Result<(), String> {
        let mode = mode & 0o555;
        set_mode(&path, mode)?;
        self.sealed.push(Sealed {
            path,
            mode,
            is_directory,
        });
        Ok(())
    }

    pub(super) fn path(&self) -> &std::path::Path {
        &self.root
    }

    /// Whether the copy still captures to `identity` and every sealed path
    /// keeps its type and mode. A copy that can no longer be captured, or a
    /// permission-only change such as a newly executable script, has
    /// drifted.
    pub(super) fn is_intact(&self, identity: &str) -> bool {
        let is_same_bytes = crate::candidate_capture::workspace(&self.root)
            .ok()
            .and_then(|snapshot| crate::candidate::snapshot_identity(&snapshot).ok())
            .is_some_and(|captured| captured == identity);
        is_same_bytes && self.sealed.iter().all(Sealed::is_unchanged)
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
