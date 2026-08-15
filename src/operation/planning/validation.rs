fn validate_request_identity(request: &super::model::PlanRequest<'_>) -> Vec<super::model::Diagnostic> {
    let mut diagnostics = Vec::new();
    if request.expected_catalog_identity != request.catalog.identity {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.catalog_identity",
            "operation catalog identity is stale",
        ));
    }
    if request.expected_instance_identity != request.instance.identity {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.instance_identity",
            "operation instance identity is stale",
        ));
    }
    if request.instance.definition.type_revision
        != request
            .catalog
            .types
            .get(&request.instance.definition.type_id)
            .map_or("", |operation_type| operation_type.definition.revision.as_str())
    {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::StaleType,
            "plan.type_revision",
            "operation type revision is stale",
        ));
    }
    diagnostics
}

fn merged_arguments(
    instance: &super::model::OperationInstance,
    method: &super::model::Method,
    arguments: BTreeMap<String, serde_json::Value>,
) -> BTreeMap<String, serde_json::Value> {
    let mut merged = instance.method_defaults.get(&method.name).cloned().unwrap_or_default();
    merged.extend(arguments);
    merged
}

fn validate_input_bound(
    arguments: &BTreeMap<String, serde_json::Value>,
    bounds: super::model::Bounds,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    match serde_json::to_vec(arguments) {
        Ok(bytes) if bytes.len() <= bounds.input_bytes => {}
        Ok(_) => diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            "plan.arguments",
            "operation arguments exceed the declared input bound",
        )),
        Err(_) => diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            "plan.arguments",
            "operation arguments cannot be encoded",
        )),
    }
}

fn validate_runtime(
    operation_type: &super::model::ValidatedType,
    runtime: &super::model::RuntimeFact,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    let required = &operation_type.definition.runtime;
    let is_binding_exact = runtime.kind == required.kind
        && runtime.key == required.key
        && runtime.executor_identity == required.executor_identity;
    let is_executable_absolute = std::path::Path::new(&runtime.executable_path).is_absolute();
    let is_bound_set_sufficient = bounds_cover(runtime.maximum_bounds, operation_type.definition.bounds);
    let is_available_and_exact = runtime.available && is_binding_exact;
    let is_available_executable_safe = is_available_and_exact && is_executable_absolute;
    let is_runtime_admitted = is_available_executable_safe && is_bound_set_sufficient;
    if !is_runtime_admitted {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::RuntimeUnavailable,
            "plan.runtime",
            "operation runtime is unavailable, mismatched, relative, or under-bounded",
        ));
    }
}

fn bounds_cover(available: super::model::Bounds, required: super::model::Bounds) -> bool {
    available.timeout_seconds >= required.timeout_seconds
        && available.input_bytes >= required.input_bytes
        && available.output_bytes >= required.output_bytes
        && available.artifact_bytes >= required.artifact_bytes
}

fn validate_capabilities(
    method: &super::model::Method,
    policy: &super::model::CapabilityPolicy,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if method.capabilities.iter().any(|capability| !policy.allowed.contains(capability))
        || (method.side_effect == super::model::SideEffectClass::Mutation && !policy.allow_mutation)
    {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::CapabilityDenied,
            "plan.capabilities",
            "operation capability policy denies the selected method",
        ));
    }
}

fn validate_preflight(
    method: &super::model::Method,
    request: &super::model::PlanRequest<'_>,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    for check in &method.preflight {
        let is_satisfied = match check {
            super::model::PreflightCheck::RuntimeAvailable => request.runtime.available,
            super::model::PreflightCheck::ExecutableAvailable => request.preflight.executable_available,
            super::model::PreflightCheck::RepositoryRootAdmitted => request.preflight.repository_root_admitted,
            super::model::PreflightCheck::CredentialReferencesValid => request.preflight.credential_references_valid,
        };
        if !is_satisfied {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::PreflightRejected,
                format!("plan.preflight.{check:?}"),
                "operation preflight fact is missing or contradictory",
            ));
        }
    }
}
