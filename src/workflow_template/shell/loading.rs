// r[impl lattice.workflow_templates.loading]
pub fn load_catalog(root_path: &std::path::Path) -> Result<super::model::Catalog, Error> {
    if !root_path.exists() {
        return Ok(super::model::Catalog {
            bundles: Vec::new(),
            diagnostics: Vec::new(),
        });
    }
    let root = crate::capability::open_root_path(root_path).map_err(|_| Error::RootUnavailable)?;
    let mut directory_names = immediate_directory_names(&root)?;
    directory_names.sort();
    if directory_names.len() > super::model::MAX_CATALOG_BUNDLES {
        return Ok(super::model::Catalog {
            bundles: Vec::new(),
            diagnostics: vec![super::core::diagnostic(
                "catalog",
                super::model::DiagnosticCode::LimitExceeded,
                "workflow-template catalog count exceeds its limit",
            )],
        });
    }

    let mut catalog = super::model::Catalog {
        bundles: Vec::with_capacity(directory_names.len()),
        diagnostics: Vec::with_capacity(directory_names.len()),
    };
    for name in directory_names {
        let safe_name = safe_directory_label(&name);
        match load_one_bundle(&root, &name, &safe_name) {
            Ok(bundle) => catalog.bundles.push(bundle),
            Err(bundle_diagnostic) => catalog.diagnostics.push(bundle_diagnostic),
        }
    }
    catalog.bundles.sort_by(|left, right| left.manifest.id.cmp(&right.manifest.id));
    debug_assert!(catalog.bundles.len() <= super::model::MAX_CATALOG_BUNDLES);
    debug_assert!(catalog.diagnostics.len() <= super::model::MAX_CATALOG_BUNDLES);
    Ok(catalog)
}

fn immediate_directory_names(root: &cap_std::fs::Dir) -> Result<Vec<std::ffi::OsString>, Error> {
    let entries = root.entries().map_err(|_| Error::RootUnavailable)?;
    let mut names = Vec::with_capacity(super::model::MAX_CATALOG_BUNDLES);
    for entry in entries {
        let entry = entry.map_err(|_| Error::RootUnavailable)?;
        let file_type = entry.file_type().map_err(|_| Error::RootUnavailable)?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if names.len() == super::model::MAX_CATALOG_BUNDLES {
                names.push(entry.file_name());
                break;
            }
            names.push(entry.file_name());
        }
    }
    Ok(names)
}

fn load_one_bundle(
    root: &cap_std::fs::Dir,
    directory_name: &std::ffi::OsString,
    safe_name: &str,
) -> Result<super::model::ValidatedBundle, super::model::Diagnostic> {
    let bundle_dir = root.open_dir(directory_name).map_err(|_| {
        super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::FileAccess,
            "bundle directory could not be opened",
        )
    })?;
    let captured = capture_files(&bundle_dir, safe_name)?;
    let manifest_bytes = captured
        .get(MANIFEST_FILE_NAME)
        .ok_or_else(|| {
            super::core::diagnostic(safe_name, super::model::DiagnosticCode::MissingFile, "bundle manifest is missing")
        })?
        .clone();
    let manifest = parse_manifest(&manifest_bytes, safe_name)?;
    if manifest.id != safe_name {
        return Err(super::core::diagnostic(
            safe_name,
            super::model::DiagnosticCode::InvalidIdentifier,
            "bundle directory and manifest identity differ",
        ));
    }
    let files = captured
        .iter()
        .filter(|(path, _)| path.as_str() != MANIFEST_FILE_NAME)
        .map(|(path, bytes)| super::model::BundleFile {
            path: path.clone(),
            bytes: bytes.clone(),
        })
        .collect::<Vec<_>>();
    let assets = manifest
        .assets
        .iter()
        .filter_map(|asset| captured.get(&asset.path).map(|bytes| (asset.path.clone(), bytes.clone())))
        .collect::<std::collections::BTreeMap<_, _>>();
    let workflow = parse_graph(&manifest, &captured, &assets);
    let orchestration = parse_orchestration(&manifest, &captured);
    debug_assert!(captured.contains_key(MANIFEST_FILE_NAME));
    debug_assert!(files.len() <= super::model::MAX_BUNDLE_FILES);
    super::core::validate_bundle(super::model::BundleValidationInput {
        manifest_bytes,
        manifest,
        files,
        workflow,
        orchestration,
    })
}
