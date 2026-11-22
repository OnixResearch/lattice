struct AdmittedContent {
    workflow: crate::engine::Workflow,
    workflow_revision: crate::orchestration::WorkflowRevisionRef,
    orchestration: Option<crate::orchestration::Definition>,
    orchestration_revision: Option<crate::orchestration::DefinitionRevision>,
}

/// Field presence in the evaluated manifest, before deserialization turns `null` into absence.
///
/// A schema 1 manifest that has a `mode` or `params` key, even with a `null` value, is refused.
pub(crate) fn check_legacy_fields(
    evaluated: &serde_json::Value,
    bundle: &str,
) -> Result<(), super::model::Diagnostic> {
    let is_legacy = evaluated.get("schema_version").and_then(serde_json::Value::as_u64)
        == Some(u64::from(super::model::LEGACY_BUNDLE_SCHEMA_VERSION));
    let has_new_field = evaluated.get("mode").is_some() || evaluated.get("params").is_some();
    if is_legacy && has_new_field {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::UnsupportedSchema,
            "bundle schema 1 cannot declare mode or params; use schema 2",
        ));
    }
    Ok(())
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
    let mut workflow = super::params::apply(declared, &resolved, &template.workflow)
        .map_err(|error| param_diagnostic(bundle, &error))?;
    // Carry exactly what the store persists: a `MaterializedFork` association when the values
    // change the expanded graph, and the unchanged association otherwise.
    crate::authoring::reconcile_materialized(&mut workflow).map_err(|_| {
        diagnostic(bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow source association failed")
    })?;
    let revision = crate::orchestration::workflow_revision(&workflow)
        .map_err(|_| diagnostic(bundle, super::model::DiagnosticCode::InvalidWorkflow, "workflow identity failed"))?;
    let orchestration = template.orchestration.clone().map(|mut definition| {
        definition.workflow = revision;
        definition
    });
    let content = admit_content(&template.manifest, workflow, orchestration, bundle)?;
    super::params::validate_bound(declared, &resolved, &content.workflow)
        .map_err(|error| param_diagnostic(bundle, &error))?;
    let identity = super::params::bound_identity(&template.identity, &resolved, &content.workflow_revision);
    debug_assert_eq!(content.workflow.configs.len(), template.workflow.configs.len());
    Ok(super::model::ValidatedBundle {
        identity,
        manifest_identity: template.manifest_identity.clone(),
        manifest: template.manifest.clone(),
        workflow_revision: content.workflow_revision.clone(),
        workflow: content.workflow,
        orchestration_revision: content.orchestration_revision,
        orchestration: content.orchestration,
        file_identities: template.file_identities.clone(),
        params: Some(super::params::Bound {
            template: template.identity.clone(),
            resolved,
            workflow_revision: content.workflow_revision,
        }),
    })
}

/// Re-verify a bound bundle at a save or start boundary.
///
/// It compares, in order: the recorded values re-resolve to the same `Resolved`; the workflow
/// still holds them and passes the target checks; reconciling its source association leaves it
/// unchanged, as `bind` and the store reconcile it; the content passes full admission again; the
/// recomputed workflow revision equals the recorded and bound revisions; the recomputed
/// orchestration revision equals the recorded one; the template identity recomputed from the
/// manifest and file identities equals the bound template; and the bound identity recomputed
/// from template, values, and revision equals the bundle identity.
pub fn reverify(bundle: &super::model::ValidatedBundle) -> Result<(), super::model::Diagnostic> {
    let label = bundle.manifest.id.as_str();
    let drift = |message: &'static str| diagnostic(label, super::model::DiagnosticCode::BoundContentDrift, message);
    let bound = bundle.params.as_ref().ok_or_else(|| {
        diagnostic(label, super::model::DiagnosticCode::ParamsUnbound, "bundle parameters are not bound")
    })?;
    let declared = bundle.manifest.params();
    let recorded = bound
        .resolved
        .values()
        .iter()
        .map(|param| (param.name.clone(), param.value.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let resolved =
        super::params::resolve(declared, &recorded).map_err(|_| drift("bound parameter values no longer resolve"))?;
    if resolved != bound.resolved {
        return Err(drift("bound parameter values differ"));
    }
    super::params::validate_bound(declared, &resolved, &bundle.workflow)
        .map_err(|_| drift("bound workflow does not hold the resolved values"))?;
    // Reconciliation changes the association exactly when the workflow diverges from its recorded
    // expansion and the association is not already a fork; anything else is a fixed point.
    let diverges = crate::authoring::diverges_from_expansion(&bundle.workflow)
        .map_err(|_| drift("bound workflow source association is invalid"))?;
    let forked = bundle
        .workflow
        .authoring
        .as_ref()
        .is_some_and(|association| association.state == crate::authoring::AssociationState::MaterializedFork);
    if diverges == Some(true) && !forked {
        return Err(drift("bound workflow source association differs"));
    }
    // A mode failure keeps `mode_violation`; every other admission failure is drift.
    let content = admit_content(&bundle.manifest, bundle.workflow.clone(), bundle.orchestration.clone(), label)
        .map_err(|error| match error.code {
            super::model::DiagnosticCode::ModeViolation => error,
            _ => drift("bound content no longer passes admission"),
        })?;
    if content.workflow_revision != bundle.workflow_revision || content.workflow_revision != bound.workflow_revision {
        return Err(drift("bound workflow revision differs"));
    }
    if content.orchestration_revision != bundle.orchestration_revision {
        return Err(drift("bound orchestration revision differs"));
    }
    if bundle_identity(&bundle.manifest_identity, &bundle.file_identities) != bound.template {
        return Err(drift("bound template identity differs"));
    }
    if super::params::bound_identity(&bound.template, &resolved, &content.workflow_revision) != bundle.identity {
        return Err(drift("bound identity differs"));
    }
    debug_assert_eq!(bundle.workflow_revision, bound.workflow_revision);
    debug_assert!(bundle.manifest.params().len() == resolved.values().len());
    Ok(())
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
        super::params::ParamErrorCode::TargetExpression => super::model::DiagnosticCode::ParamTargetExpression,
        super::params::ParamErrorCode::TargetEnv => super::model::DiagnosticCode::ParamTargetEnv,
    };
    diagnostic(bundle, code, error.message())
}
