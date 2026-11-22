/// The raw sources of one `lattice run` or `authority inspect` input set.
pub struct InputRequest<'a> {
    /// Each `--input NAME=VALUE` argument.
    pub cli: &'a [String],
    /// The `--input-file` path, if given.
    pub file: Option<&'a std::path::Path>,
    /// The `workflow-templates/` directory next to `config.ncl`.
    pub catalog_root: &'a std::path::Path,
}

impl InputRequest<'_> {
    /// Whether any input source was given. Without one, a run keeps the inputless path.
    pub fn is_present(&self) -> bool {
        !self.cli.is_empty() || self.file.is_some()
    }
}

/// Read an `--input-file` of at most [`super::inputs::MAX_INPUT_FILE_BYTES`] bytes.
fn read_input_file(path: &std::path::Path) -> Result<Vec<u8>, Error> {
    use std::io::Read as _;
    let file = std::fs::File::open(path).map_err(|_| Error::InputFile)?;
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

fn collect_raw(
    request: &InputRequest<'_>,
) -> Result<std::collections::BTreeMap<String, super::inputs::RawInput>, Error> {
    let file = request.file.map(read_input_file).transpose()?;
    super::inputs::collect(request.cli, file.as_deref()).map_err(Error::Inputs)
}

fn admit_loaded(
    workflow_name: &str,
    saved: Option<&crate::engine::Workflow>,
    binding: Option<&super::inputs::RetainedBinding>,
    request: &InputRequest<'_>,
    raw: &std::collections::BTreeMap<String, super::inputs::RawInput>,
) -> Result<super::inputs::Admitted, Error> {
    // Without a saved workflow or binding, refuse before reading the catalog.
    let catalog = match (saved, binding) {
        (Some(_), Some(_)) => load_catalog(request.catalog_root)?,
        (None, _) | (_, None) => super::model::Catalog {
            bundles: Vec::new(),
            diagnostics: Vec::new(),
        },
    };
    let facts = super::inputs::SavedFacts {
        workflow_name,
        saved,
        binding,
        catalog: &catalog.bundles,
    };
    super::inputs::admit(facts, raw).map_err(Error::Inputs)
}

// r[impl lattice.workflow_templates.run_inputs]
/// Admit run inputs for a saved workflow in a writable store.
///
/// This reads the input file, the saved workflow, its retained binding, and the template
/// catalog. It writes nothing; a caller reserves the run only after it succeeds.
pub fn admit_saved_inputs(
    store: &crate::store::Store,
    workflow_name: &str,
    request: &InputRequest<'_>,
) -> Result<super::inputs::Admitted, Error> {
    debug_assert!(request.is_present());
    let raw = collect_raw(request)?;
    let saved = store.load_workflow(workflow_name)?;
    let binding = store.load_workflow_template_binding(workflow_name)?;
    let admitted = admit_loaded(workflow_name, saved.as_ref(), binding.as_ref(), request, &raw)?;
    debug_assert_eq!(admitted.identity.workflow.workflow_name, workflow_name);
    Ok(admitted)
}

/// Admit run inputs against a store opened read-only, for `authority inspect`.
pub fn admit_saved_inputs_read_only(
    store_path: &std::path::Path,
    workflow_name: &str,
    request: &InputRequest<'_>,
) -> Result<super::inputs::Admitted, Error> {
    debug_assert!(request.is_present());
    let raw = collect_raw(request)?;
    let (saved, binding) = crate::store::Store::load_saved_template_facts_read_only(store_path, workflow_name)?;
    let admitted = admit_loaded(workflow_name, saved.as_ref(), binding.as_ref(), request, &raw)?;
    debug_assert_eq!(admitted.identity.workflow.workflow_name, workflow_name);
    Ok(admitted)
}
