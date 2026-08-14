fn validate_declared_compatibility(
    compatibility: &super::model::CompatibilityRequirements,
    workflow: &crate::engine::Workflow,
    bundle: &str,
) -> Result<(), super::model::Diagnostic> {
    if compatibility.workflow_schema != super::model::CURRENT_WORKFLOW_SCHEMA_VERSION
        || compatibility
            .orchestration_schema
            .is_some_and(|version| version != super::model::CURRENT_ORCHESTRATION_SCHEMA_VERSION)
    {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::IncompatibleSchema,
            "bundle schema compatibility is unsupported",
        ));
    }
    if compatibility.feature_ids.iter().any(|feature| !SUPPORTED_FEATURE_IDS.contains(&feature.as_str())) {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::IncompatibleContent,
            "bundle feature compatibility is unsupported",
        ));
    }
    let declared_node_kinds = compatibility
        .node_kinds
        .iter()
        .map(|kind| kind.to_ascii_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    for config in workflow.configs.values() {
        if !declared_node_kinds.contains(&config.kind_label().to_ascii_lowercase()) {
            return Err(diagnostic(
                bundle,
                super::model::DiagnosticCode::IncompatibleContent,
                "workflow node kind is undeclared",
            ));
        }
    }
    let declared_runtime_kinds = compatibility.runtime_kinds.iter().collect::<std::collections::BTreeSet<_>>();
    for node in workflow.graph.nodes() {
        if let Some(settings) = workflow.node_runner_settings(node.id)
            && !settings.runtime_kind.is_empty()
            && !declared_runtime_kinds.contains(&settings.runtime_kind)
        {
            return Err(diagnostic(
                bundle,
                super::model::DiagnosticCode::IncompatibleContent,
                "workflow runtime kind is undeclared",
            ));
        }
    }
    debug_assert_eq!(compatibility.workflow_schema, super::model::CURRENT_WORKFLOW_SCHEMA_VERSION);
    debug_assert!(compatibility.feature_ids.iter().all(|feature| SUPPORTED_FEATURE_IDS.contains(&feature.as_str())));
    Ok(())
}

fn validate_orchestration(
    expected: bool,
    orchestration: Option<crate::orchestration::OrchestrationDefinition>,
    workflow_revision: &crate::orchestration::WorkflowRevisionRef,
    bundle: &str,
) -> Result<
    (
        Option<crate::orchestration::OrchestrationDefinition>,
        Option<crate::orchestration::DefinitionRevision>,
    ),
    super::model::Diagnostic,
> {
    if expected && orchestration.is_none() {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::InvalidOrchestration,
            "orchestration content is invalid",
        ));
    }
    if !expected && orchestration.is_some() {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::UndeclaredFile,
            "orchestration content is undeclared",
        ));
    }
    let Some(orchestration) = orchestration else {
        return Ok((None, None));
    };
    let diagnostics = crate::orchestration::validate_definition(&orchestration, Some(workflow_revision));
    if !diagnostics.is_empty() {
        return Err(diagnostic(
            bundle,
            super::model::DiagnosticCode::WorkflowRevisionMismatch,
            "orchestration does not bind the exact workflow revision",
        ));
    }
    let revision = crate::orchestration::definition_revision(&orchestration).map_err(|_| {
        diagnostic(bundle, super::model::DiagnosticCode::InvalidOrchestration, "orchestration identity failed")
    })?;
    debug_assert!(expected);
    debug_assert_eq!(orchestration.workflow, *workflow_revision);
    Ok((Some(orchestration), Some(revision)))
}

fn bundle_identity(
    manifest_identity: &super::model::ContentIdentity,
    files: &[(String, super::model::ContentIdentity)],
) -> super::model::ContentIdentity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(BUNDLE_IDENTITY_DOMAIN.as_bytes());
    hasher.update(manifest_identity.as_str().as_bytes());
    for (path, identity) in files {
        hasher.update(path.as_bytes());
        hasher.update(identity.as_str().as_bytes());
    }
    super::model::ContentIdentity::from_bytes(hasher.finalize().as_bytes())
}
