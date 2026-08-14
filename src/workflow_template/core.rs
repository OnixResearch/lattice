const SUPPORTED_FEATURE_IDS: &[&str] = &["lattice.workflow-template-bundle.v1"];
const BUNDLE_IDENTITY_DOMAIN: &str = "lattice.workflow-template-bundle.v1";
const MANIFEST_FILE_NAME: &str = "manifest.ncl";

// r[impl lattice.workflow_templates.core]
// r[impl lattice.workflow_templates.catalog]
pub fn validate_bundle(
    input: super::model::BundleValidationInput,
) -> Result<super::model::ValidatedBundle, super::model::Diagnostic> {
    let bundle = safe_bundle_label(&input.manifest.id);
    validate_manifest_shape(&input, &bundle)?;
    let declarations = declared_files(&input, &bundle)?;
    let (file_identities, total_bytes) = validate_files(&input.files, &declarations, &bundle)?;
    if total_bytes > super::model::MAX_BUNDLE_BYTES {
        return Err(diagnostic(&bundle, super::model::DiagnosticCode::BundleTooLarge, "bundle byte limit exceeded"));
    }

    let workflow = input.workflow.ok_or_else(|| {
        diagnostic(&bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow content is invalid")
    })?;
    if !crate::workflow_validation::validate_workflow(&workflow).is_empty() {
        return Err(diagnostic(&bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow validation failed"));
    }
    validate_declared_compatibility(&input.manifest.compatibility, &workflow, &bundle)?;
    let workflow_revision = crate::orchestration::workflow_revision(&workflow)
        .map_err(|_| diagnostic(&bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow identity failed"))?;

    let (orchestration, orchestration_revision) = validate_orchestration(
        input.manifest.orchestration.is_some(),
        input.orchestration,
        &workflow_revision,
        &bundle,
    )?;
    let manifest_identity = super::model::ContentIdentity::from_bytes(&input.manifest_bytes);
    let identity = bundle_identity(&manifest_identity, &file_identities);
    debug_assert!(total_bytes <= super::model::MAX_BUNDLE_BYTES);
    debug_assert!(file_identities.len() <= super::model::MAX_BUNDLE_FILES);

    Ok(super::model::ValidatedBundle {
        identity,
        manifest_identity,
        manifest: input.manifest,
        workflow_revision,
        workflow,
        orchestration_revision,
        orchestration,
        file_identities,
    })
}

fn validate_manifest_shape(
    input: &super::model::BundleValidationInput,
    bundle: &str,
) -> Result<(), super::model::Diagnostic> {
    let manifest = &input.manifest;
    if manifest.schema_version != super::model::CURRENT_BUNDLE_SCHEMA_VERSION {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::UnsupportedSchema,
            "bundle schema is unsupported",
        ));
    }
    if !valid_identifier(&manifest.id) || !valid_identifier(&manifest.category) {
        return Err(diagnostic(bundle, super::model::DiagnosticCode::InvalidIdentifier, "bundle identity is invalid"));
    }
    if manifest.display_name.is_empty()
        || manifest.display_name.len() > super::model::MAX_DISPLAY_TEXT_BYTES
        || manifest.description.len() > super::model::MAX_DISPLAY_TEXT_BYTES
    {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::LimitExceeded,
            "bundle display text exceeds its limit",
        ));
    }
    let has_too_many_assets = manifest.assets.len() > super::model::MAX_BUNDLE_FILES;
    let has_too_many_compatibility_items = [
        manifest.compatibility.node_kinds.len(),
        manifest.compatibility.runtime_kinds.len(),
        manifest.compatibility.feature_ids.len(),
    ]
    .into_iter()
    .any(|count| count > super::model::MAX_COMPATIBILITY_ITEMS);
    let has_too_many_notes =
        manifest.operator_notes.len() > super::model::MAX_NOTES || manifest.non_claims.len() > super::model::MAX_NOTES;
    if has_too_many_assets || has_too_many_compatibility_items || has_too_many_notes {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::LimitExceeded,
            "bundle manifest collection exceeds its limit",
        ));
    }
    if input.manifest_bytes.len() > super::model::MAX_FILE_BYTES {
        return Err(diagnostic(bundle, super::model::DiagnosticCode::FileTooLarge, "manifest byte limit exceeded"));
    }
    if manifest
        .operator_notes
        .iter()
        .chain(manifest.non_claims.iter())
        .any(|text| text.len() > super::model::MAX_DISPLAY_TEXT_BYTES)
    {
        return Err(diagnostic(bundle, super::model::DiagnosticCode::LimitExceeded, "bundle note exceeds its limit"));
    }
    debug_assert_eq!(manifest.schema_version, super::model::CURRENT_BUNDLE_SCHEMA_VERSION);
    debug_assert!(input.manifest_bytes.len() <= super::model::MAX_FILE_BYTES);
    Ok(())
}

fn declared_files(
    input: &super::model::BundleValidationInput,
    bundle: &str,
) -> Result<std::collections::BTreeMap<String, (super::model::ContentIdentity, bool)>, super::model::Diagnostic> {
    let manifest = &input.manifest;
    let expected_count = manifest
        .assets
        .len()
        .saturating_add(usize::from(manifest.orchestration.is_some()))
        .saturating_add(1);
    if expected_count > super::model::MAX_BUNDLE_FILES {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::TooManyFiles,
            "bundle file count exceeds its limit",
        ));
    }
    let mut declarations = std::collections::BTreeMap::new();
    insert_declaration(
        &mut declarations,
        std::path::Path::new(&manifest.workflow.path),
        &manifest.workflow.blake3,
        true,
        bundle,
    )?;
    if let Some(binding) = &manifest.orchestration {
        insert_declaration(&mut declarations, std::path::Path::new(&binding.path), &binding.blake3, true, bundle)?;
    }
    for asset in &manifest.assets {
        if !valid_identifier(&asset.media_kind.replace('/', "-")) {
            return Err(diagnostic(
                bundle,
                super::model::DiagnosticCode::InvalidIdentifier,
                "asset media kind is invalid",
            ));
        }
        insert_declaration(
            &mut declarations,
            std::path::Path::new(&asset.path),
            &asset.blake3,
            asset.required,
            bundle,
        )?;
    }
    debug_assert_eq!(declarations.len(), expected_count);
    Ok(declarations)
}

fn insert_declaration(
    declarations: &mut std::collections::BTreeMap<String, (super::model::ContentIdentity, bool)>,
    path: &std::path::Path,
    identity: &super::model::ContentIdentity,
    required: bool,
    bundle: &str,
) -> Result<(), super::model::Diagnostic> {
    let path_text = path.to_string_lossy();
    if !safe_relative_path(&path_text) || path_text == MANIFEST_FILE_NAME {
        return Err(diagnostic(bundle, super::model::DiagnosticCode::UnsafePath, "bundle path is unsafe"));
    }
    if declarations.insert(path_text.into_owned(), (identity.clone(), required)).is_some() {
        return Err(diagnostic(bundle, super::model::DiagnosticCode::DuplicatePath, "bundle path is duplicated"));
    }
    Ok(())
}

fn validate_files(
    files: &[super::model::BundleFile],
    declarations: &std::collections::BTreeMap<String, (super::model::ContentIdentity, bool)>,
    bundle: &str,
) -> Result<(Vec<(String, super::model::ContentIdentity)>, usize), super::model::Diagnostic> {
    if files.len() > super::model::MAX_BUNDLE_FILES {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::TooManyFiles,
            "supplied file count exceeds its limit",
        ));
    }
    let mut supplied = std::collections::BTreeSet::new();
    let mut file_identities = Vec::with_capacity(files.len());
    let mut total_bytes = 0usize;
    for file in files {
        if !safe_relative_path(&file.path) || file.path == MANIFEST_FILE_NAME {
            return Err(diagnostic(bundle, super::model::DiagnosticCode::UnsafePath, "supplied file path is unsafe"));
        }
        if !supplied.insert(file.path.clone()) {
            return Err(diagnostic(
                bundle,
                super::model::DiagnosticCode::DuplicatePath,
                "supplied file path is duplicated",
            ));
        }
        let Some((expected, _required)) = declarations.get(&file.path) else {
            return Err(diagnostic(
                bundle,
                super::model::DiagnosticCode::UndeclaredFile,
                "bundle contains an undeclared file",
            ));
        };
        if file.bytes.len() > super::model::MAX_FILE_BYTES {
            return Err(diagnostic(
                bundle,
                super::model::DiagnosticCode::FileTooLarge,
                "bundle file byte limit exceeded",
            ));
        }
        total_bytes = total_bytes.checked_add(file.bytes.len()).ok_or_else(|| {
            diagnostic(bundle, super::model::DiagnosticCode::BundleTooLarge, "bundle byte count overflowed")
        })?;
        let actual = super::model::ContentIdentity::from_bytes(&file.bytes);
        if &actual != expected {
            return Err(diagnostic(
                bundle,
                super::model::DiagnosticCode::IdentityMismatch,
                "bundle file identity differs",
            ));
        }
        file_identities.push((file.path.clone(), actual));
    }
    if declarations.iter().any(|(path, (_, required))| *required && !supplied.contains(path)) {
        return Err(diagnostic(bundle, super::model::DiagnosticCode::MissingFile, "bundle is missing a required file"));
    }
    file_identities.sort_by(|left, right| left.0.cmp(&right.0));
    debug_assert!(file_identities.len() <= files.len());
    debug_assert!(total_bytes <= super::model::MAX_BUNDLE_BYTES);
    Ok((file_identities, total_bytes))
}

include!("core/compatibility.rs");
include!("core/planning.rs");
include!("core/path.rs");
