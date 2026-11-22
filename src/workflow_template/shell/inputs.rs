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
fn load_bound_bundle(
    catalog_root: &std::path::Path,
    binding: &super::inputs::RetainedBinding,
) -> Result<super::model::ValidatedBundle, Error> {
    let root = crate::capability::open_root_path(catalog_root)
        .map_err(|_| Error::Inputs(super::inputs::InputError::TemplateUnavailable))?;
    let directory = std::ffi::OsString::from(&binding.bundle);
    let label = safe_directory_label(&directory);
    load_one_bundle(&root, &directory, &label).map_err(|diagnostic| {
        Error::Inputs(super::inputs::InputError::TemplateLoad(diagnostic))
    })
}

fn admit_loaded(
    workflow_name: &str,
    saved: Option<&crate::engine::Workflow>,
    binding: Option<&super::inputs::RetainedBinding>,
    catalog_root: &std::path::Path,
    raw: &std::collections::BTreeMap<String, super::inputs::RawInput>,
) -> Result<super::inputs::Admitted, Error> {
    // Without a saved workflow or binding, refuse before reading the catalog.
    let template = match (saved, binding) {
        (Some(_), Some(binding)) => Some(load_bound_bundle(catalog_root, binding)?),
        (None, _) | (_, None) => None,
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
