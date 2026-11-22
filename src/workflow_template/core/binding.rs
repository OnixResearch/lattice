struct AdmittedContent {
    workflow: crate::engine::Workflow,
    workflow_revision: crate::orchestration::WorkflowRevisionRef,
    orchestration: Option<crate::orchestration::Definition>,
    orchestration_revision: Option<crate::orchestration::DefinitionRevision>,
}

/// Schema 2 requires `mode` and `params`. Schema 1 is the documented compatibility path:
/// it may carry neither field and loads as `mutating` with no parameters.
fn validate_schema_fields(
    manifest: &super::model::BundleManifest,
    bundle: &str,
) -> Result<(), super::model::Diagnostic> {
    let has_mode = manifest.mode.is_some();
    let has_params = manifest.params.is_some();
    match manifest.schema_version {
        super::model::CURRENT_BUNDLE_SCHEMA_VERSION if has_mode && has_params => Ok(()),
        super::model::CURRENT_BUNDLE_SCHEMA_VERSION => Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::InvalidManifest,
            "bundle schema 2 requires mode and params",
        )),
        super::model::LEGACY_BUNDLE_SCHEMA_VERSION if !has_mode && !has_params => Ok(()),
        super::model::LEGACY_BUNDLE_SCHEMA_VERSION => Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::UnsupportedSchema,
            "bundle schema 1 cannot declare mode or params; use schema 2",
        )),
        _ => Err(diagnostic(bundle, super::model::DiagnosticCode::UnsupportedSchema, "bundle schema is unsupported")),
    }
}

/// Admit one workflow and optional orchestration under a manifest: workflow validation,
/// compatibility, declared mode, exact orchestration binding, and guarded authority.
fn admit_content(
    manifest: &super::model::BundleManifest,
    workflow: crate::engine::Workflow,
    orchestration: Option<crate::orchestration::Definition>,
    bundle: &str,
) -> Result<AdmittedContent, super::model::Diagnostic> {
    if !crate::workflow_validation::validate_workflow(&workflow).is_empty() {
        return Err(diagnostic(bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow validation failed"));
    }
    validate_declared_compatibility(&manifest.compatibility, &workflow, bundle)?;
    if !super::classification::admits(manifest.mode(), &workflow) {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::ModeViolation,
            "read_only bundle declares a mutating effect",
        ));
    }
    let workflow_revision = crate::orchestration::workflow_revision(&workflow)
        .map_err(|_| diagnostic(bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow identity failed"))?;
    let (orchestration, orchestration_revision) =
        validate_orchestration(manifest.orchestration.is_some(), orchestration, &workflow_revision, bundle)?;
    if manifest.compatibility.feature_ids.iter().any(|feature| feature == crate::guarded::authority::GUARDED_FEATURE_ID)
    {
        crate::guarded::authority::validate_bundle(&manifest.id, &workflow, orchestration.as_ref()).map_err(|_| {
            diagnostic(
                bundle,
                super::model::DiagnosticCode::IncompatibleContent,
                "guarded workflow authority validation failed",
            )
        })?;
    }
    debug_assert!(super::classification::admits(manifest.mode(), &workflow));
    Ok(AdmittedContent {
        workflow,
        workflow_revision,
        orchestration,
        orchestration_revision,
    })
}

// r[impl lattice.workflow_templates.params]
/// Bind typed values into a catalog template and readmit the result.
///
/// Values reach the workflow only as declared shell environment values. The bound workflow,
/// its revision, and an optional orchestration rebound to that exact revision pass the same
/// admission as the template. The bound identity commits to the template identity and the
/// resolved-value identity.
pub fn bind(
    template: &super::model::ValidatedBundle,
    supplied: &std::collections::BTreeMap<String, super::params::ParamValue>,
) -> Result<super::model::ValidatedBundle, super::model::Diagnostic> {
    let bundle = template.manifest.id.as_str();
    if template.params.is_some() {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::InvalidParamDeclaration,
            "bundle parameters are already bound",
        ));
    }
    let declared = template.manifest.params();
    let resolved = super::params::resolve(declared, supplied).map_err(|error| param_diagnostic(bundle, &error))?;
    let workflow = super::params::apply(declared, &resolved, &template.workflow)
        .map_err(|error| param_diagnostic(bundle, &error))?;
    let revision = crate::orchestration::workflow_revision(&workflow)
        .map_err(|_| diagnostic(bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow identity failed"))?;
    let orchestration = template.orchestration.clone().map(|mut definition| {
        definition.workflow = revision;
        definition
    });
    let content = admit_content(&template.manifest, workflow, orchestration, bundle)?;
    let identity = super::params::bound_identity(&template.identity, &resolved);
    debug_assert_eq!(content.workflow.configs.len(), template.workflow.configs.len());
    Ok(super::model::ValidatedBundle {
        identity,
        manifest_identity: template.manifest_identity.clone(),
        manifest: template.manifest.clone(),
        workflow_revision: content.workflow_revision,
        workflow: content.workflow,
        orchestration_revision: content.orchestration_revision,
        orchestration: content.orchestration,
        file_identities: template.file_identities.clone(),
        params: Some(super::params::Bound {
            template: template.identity.clone(),
            resolved,
        }),
    })
}

fn param_diagnostic(bundle: &str, error: &super::params::ParamError) -> super::model::Diagnostic {
    let code = match error.code {
        super::params::ParamErrorCode::InvalidDeclaration => super::model::DiagnosticCode::InvalidParamDeclaration,
        super::params::ParamErrorCode::UnknownParam | super::params::ParamErrorCode::TooManyValues => {
            super::model::DiagnosticCode::UnknownParam
        }
        super::params::ParamErrorCode::WrongType => super::model::DiagnosticCode::ParamTypeMismatch,
        super::params::ParamErrorCode::OutOfBounds => super::model::DiagnosticCode::ParamOutOfBounds,
        super::params::ParamErrorCode::MissingRequired => super::model::DiagnosticCode::MissingParam,
    };
    diagnostic(bundle, code, error.message())
}
