/// The raw sources of one `lattice run` or `authority inspect` input set.
pub struct InputRequest<'a> {
    /// Each `--input NAME=VALUE` argument.
    pub cli: &'a [String],
    /// The `--input-file` path, if given.
    pub file: Option<&'a std::path::Path>,
}

impl InputRequest<'_> {
    /// Whether any input source was given. Without one, a run keeps the inputless path.
    pub fn is_present(&self) -> bool {
        !self.cli.is_empty() || self.file.is_some()
    }

    /// Read and collect the raw inputs. A caller reads them before it opens the store, so a
    /// refused input file never holds the store.
    pub fn read(&self) -> Result<std::collections::BTreeMap<String, super::inputs::RawInput>, Error> {
        let file = self.file.map(read_input_file).transpose()?;
        super::inputs::collect(self.cli, file.as_deref()).map_err(Error::Inputs)
    }
}

/// Open an `--input-file` without blocking on a FIFO or device: non-blocking on Unix, then
/// refused unless the opened handle is a regular file.
fn open_input_file(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_NONBLOCK);
    }
    options.open(path)
}

/// Read an `--input-file` of at most [`super::inputs::MAX_INPUT_FILE_BYTES`] bytes.
fn read_input_file(path: &std::path::Path) -> Result<Vec<u8>, Error> {
    use std::io::Read as _;
    let file = open_input_file(path).map_err(|_| Error::InputFile)?;
    let metadata = file.metadata().map_err(|_| Error::InputFile)?;
    if !metadata.is_file() || metadata.len() > super::inputs::MAX_INPUT_FILE_BYTES {
        return Err(Error::InputFile);
    }
    let mut bytes = Vec::new();
    let limit_bytes = super::inputs::MAX_INPUT_FILE_BYTES.saturating_add(1);
    file.take(limit_bytes).read_to_end(&mut bytes).map_err(|_| Error::InputFile)?;
    match u64::try_from(bytes.len()) {
        Ok(length_bytes) if length_bytes <= super::inputs::MAX_INPUT_FILE_BYTES => Ok(bytes),
        Ok(_) | Err(_) => Err(Error::InputFile),
    }
}

/// Load only the bundle directory the binding names, surfacing its load diagnostic.
///
/// Like the catalog loader, a symlinked bundle directory is refused rather than followed. The
/// directory is opened with `O_NOFOLLOW`, so the check and the open are one step; the opened
/// handle is then compared with the entry, which also covers the non-`openat2` fallback.
fn load_bound_bundle(
    catalog_root: &std::path::Path,
    binding: &super::inputs::RetainedBinding,
) -> Result<super::model::ValidatedBundle, Error> {
    let root = crate::capability::open_root_path(catalog_root)
        .map_err(|_| Error::Inputs(super::inputs::InputError::TemplateUnavailable))?;
    let directory = std::ffi::OsString::from(&binding.bundle);
    let label = safe_directory_label(&directory);
    let refuse = |code, message| {
        Error::Inputs(super::inputs::InputError::TemplateLoad(super::core::diagnostic(&label, code, message)))
    };
    let symlink = || refuse(super::model::DiagnosticCode::SymlinkRejected, "bundle directory is a symlink");
    let bundle_dir = open_dir_nofollow(&root, &directory).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Error::Inputs(super::inputs::InputError::TemplateUnavailable)
        } else if is_symlink_entry(&root, &directory) {
            symlink()
        } else {
            refuse(super::model::DiagnosticCode::FileAccess, "bundle directory could not be opened")
        }
    })?;
    if !is_same_entry(&root, &directory, &bundle_dir) {
        return Err(symlink());
    }
    debug_assert!(bundle_dir.dir_metadata().is_ok_and(|metadata| metadata.is_dir()));
    let bundle = load_bundle_dir(&bundle_dir, &label)
        .map_err(|diagnostic| Error::Inputs(super::inputs::InputError::TemplateLoad(diagnostic)))?;
    debug_assert_eq!(bundle.manifest().id, binding.bundle);
    Ok(bundle)
}

/// Open a directory entry of `root` without following a symlink in its final component.
fn open_dir_nofollow(root: &cap_std::fs::Dir, name: &std::ffi::OsString) -> std::io::Result<cap_std::fs::Dir> {
    let mut options = cap_std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt as _;
        options.custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW);
    }
    let file = root.open_with(name, &options)?;
    Ok(cap_std::fs::Dir::from_std_file(file.into_std()))
}

/// Whether the entry itself is a symlink. `O_NOFOLLOW` and `O_DIRECTORY` refuse a symlink with
/// `ELOOP` or `ENOTDIR`, but `ENOTDIR` also means a plain file, so the entry decides the message.
fn is_symlink_entry(root: &cap_std::fs::Dir, name: &std::ffi::OsString) -> bool {
    root.symlink_metadata(name).is_ok_and(|metadata| metadata.file_type().is_symlink())
}

/// Whether the opened handle is still the entry itself: not a symlink, same device and inode.
fn is_same_entry(root: &cap_std::fs::Dir, name: &std::ffi::OsString, opened: &cap_std::fs::Dir) -> bool {
    let (Ok(entry), Ok(handle)) = (root.symlink_metadata(name), opened.dir_metadata()) else {
        return false;
    };
    #[cfg(unix)]
    let is_same_inode = {
        use cap_std::fs::MetadataExt as _;
        entry.dev() == handle.dev() && entry.ino() == handle.ino()
    };
    #[cfg(not(unix))]
    let is_same_inode = true;
    !entry.file_type().is_symlink() && handle.is_dir() && is_same_inode
}

fn admit_loaded(
    workflow_name: &str,
    saved: Option<&crate::engine::Workflow>,
    binding: Option<&super::inputs::RetainedBinding>,
    catalog_root: &std::path::Path,
    raw: &std::collections::BTreeMap<String, super::inputs::RawInput>,
) -> Result<super::inputs::Admitted, Error> {
    // Load the bundle only for a saved workflow that still matches its binding, so a missing
    // binding or saved-workflow drift is reported as such, never hidden by a load failure.
    let template = match (saved, binding) {
        (Some(saved), Some(binding)) if super::inputs::is_current(saved, binding) => {
            Some(load_bound_bundle(catalog_root, binding)?)
        }
        (Some(_) | None, Some(_) | None) => None,
    };
    let facts = super::inputs::SavedFacts {
        workflow_name,
        saved,
        binding,
        template: template.as_ref(),
    };
    super::inputs::admit(facts, raw).map_err(Error::Inputs)
}

// r[impl lattice.workflow_templates.run_inputs]
/// Admit read run inputs for a saved workflow in a writable store.
///
/// This reads the saved workflow, its retained binding, and the one bundle directory the
/// binding names. It writes nothing; a caller reserves the run only after it succeeds.
pub fn admit_saved_inputs(
    store: &crate::store::Store,
    workflow_name: &str,
    catalog_root: &std::path::Path,
    raw: &std::collections::BTreeMap<String, super::inputs::RawInput>,
) -> Result<super::inputs::Admitted, Error> {
    let saved = store.load_workflow(workflow_name)?;
    let binding = store.load_workflow_template_binding(workflow_name)?;
    let admitted = admit_loaded(workflow_name, saved.as_ref(), binding.as_ref(), catalog_root, raw)?;
    debug_assert_eq!(admitted.identity().workflow.workflow_name, workflow_name);
    Ok(admitted)
}

/// Admit read run inputs against a store opened read-only, for `authority inspect`.
pub fn admit_saved_inputs_read_only(
    store_path: &std::path::Path,
    workflow_name: &str,
    catalog_root: &std::path::Path,
    raw: &std::collections::BTreeMap<String, super::inputs::RawInput>,
) -> Result<super::inputs::Admitted, Error> {
    let (saved, binding) = crate::store::Store::load_saved_template_facts_read_only(store_path, workflow_name)?;
    let admitted = admit_loaded(workflow_name, saved.as_ref(), binding.as_ref(), catalog_root, raw)?;
    debug_assert_eq!(admitted.identity().workflow.workflow_name, workflow_name);
    Ok(admitted)
}
