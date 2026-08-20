fn validate_type_shape(
    definition: &super::model::OperationType,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if !super::identity::valid_identifier(&definition.id) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            format!("{path}.id"),
            "operation type id is not canonical",
        ));
    }
    if definition.schema_version != super::model::TYPE_SCHEMA_VERSION {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Schema,
            format!("{path}.schema_version"),
            "operation type schema version is unsupported",
        ));
    }
    if definition.description.is_empty() || definition.description.len() > super::model::MAXIMUM_DESCRIPTION_BYTES {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.description"),
            "operation type description is outside the admitted bounds",
        ));
    }
    if !super::identity::validate(&definition.revision) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            format!("{path}.revision"),
            "operation type revision is not a canonical BLAKE3 identity",
        ));
    }
    validate_ordered_identifiers(&definition.tags, super::model::MAXIMUM_TAGS, &format!("{path}.tags"), diagnostics);
    validate_methods(definition, path, diagnostics);
    validate_runtime(&definition.runtime, path, diagnostics);
    validate_execution_requirements(definition, path, diagnostics);
    validate_capabilities(&definition.capabilities, &format!("{path}.capabilities"), diagnostics);
    validate_credentials(&definition.credential_bindings, path, diagnostics);
    validate_cairn_profile(definition, path, diagnostics);
    validate_bounds(definition.bounds, path, diagnostics);
    if definition.required_non_claims
        != super::model::REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect::<Vec<_>>()
    {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            format!("{path}.required_non_claims"),
            "operation type does not contain the exact required non-claims",
        ));
    }
}

fn validate_methods(
    definition: &super::model::OperationType,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if definition.methods.is_empty() || definition.methods.len() > super::model::MAXIMUM_TYPE_METHODS {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.methods"),
            "operation method count is outside the admitted bounds",
        ));
    }
    let mut names = BTreeSet::new();
    let mut previous = None;
    for method in definition.methods.iter().take(super::model::MAXIMUM_TYPE_METHODS) {
        let method_path = format!("{path}.methods.{}", method.name);
        if !super::identity::valid_identifier(&method.name) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                &method_path,
                "operation method name is not canonical",
            ));
        }
        if !names.insert(method.name.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::Duplicate,
                &method_path,
                "operation method name is duplicated",
            ));
        }
        if previous.is_some_and(|value: &str| value >= method.name.as_str()) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::InvalidValue,
                format!("{path}.methods"),
                "operation methods must be ordered by canonical name",
            ));
        }
        previous = Some(method.name.as_str());
        if method.description.is_empty() || method.description.len() > super::model::MAXIMUM_DESCRIPTION_BYTES {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::Limit,
                format!("{method_path}.description"),
                "operation method description is outside the admitted bounds",
            ));
        }
        diagnostics.extend(super::schema::validate_contract(&method.arguments, &format!("{method_path}.arguments")));
        diagnostics.extend(super::schema::validate_contract(&method.output, &format!("{method_path}.output")));
        validate_sorted_unique(
            &method.preflight,
            super::model::MAXIMUM_PREFLIGHT_CHECKS,
            &format!("{method_path}.preflight"),
            diagnostics,
        );
        validate_capabilities(&method.capabilities, &format!("{method_path}.capabilities"), diagnostics);
        if method.capabilities.iter().any(|capability| !definition.capabilities.contains(capability)) {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::CapabilityDenied,
                format!("{method_path}.capabilities"),
                "method capability is not declared by its operation type",
            ));
        }
    }
}

fn validate_runtime(
    runtime: &super::model::RuntimeRequirement,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if !super::identity::valid_identifier(&runtime.kind) || !super::identity::valid_identifier(&runtime.key) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidValue,
            format!("{path}.runtime"),
            "runtime kind or key is not canonical",
        ));
    }
    if runtime.kind != super::runtime::CAIRN_RUNTIME_KIND || runtime.key != super::runtime::CAIRN_RUNTIME_KEY {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::RuntimeUnavailable,
            format!("{path}.runtime"),
            "operation runtime binding is not present in the closed runtime registry",
        ));
    }
    if !super::identity::validate(&runtime.executor_identity) {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            format!("{path}.runtime.executor_identity"),
            "runtime executor identity is not canonical",
        ));
    }
}

fn validate_execution_requirements(
    definition: &super::model::OperationType,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if definition.execution_requirements.validate().is_err() {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::BackendDenied,
            format!("{path}.execution_requirements"),
            "operation execution requirements are incomplete or unbounded",
        ));
    }
}

fn validate_cairn_profile(
    definition: &super::model::OperationType,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    let method_names = definition
        .methods
        .iter()
        .take(super::model::MAXIMUM_TYPE_METHODS)
        .map(|method| method.name.as_str())
        .collect::<Vec<_>>();
    let is_method_set_exact = method_names == CAIRN_PILOT_METHODS;
    let is_method_contract_set_exact =
        definition.methods.iter().take(super::model::MAXIMUM_TYPE_METHODS).all(|method| {
            method.arguments.fields.is_empty()
                && method.arguments.deny_unknown_fields
                && method.preflight == CAIRN_PILOT_PREFLIGHT
                && method.capabilities == CAIRN_PILOT_CAPABILITIES
                && method.side_effect == super::model::SideEffectClass::ReadOnly
        });
    let is_type_contract_exact = definition.id == super::readiness::CAIRN_PILOT_TYPE_ID
        && definition.capabilities == CAIRN_PILOT_CAPABILITIES
        && definition.credential_bindings.is_empty()
        && definition.execution_requirements == super::backend::ExecutionRequirements::cairn_read_only();
    if !is_method_set_exact || !is_method_contract_set_exact || !is_type_contract_exact {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::RuntimeUnavailable,
            format!("{path}.runtime"),
            "operation type does not match the closed Cairn read-only adapter profile",
        ));
    }
}

fn validate_credentials(
    bindings: &[super::model::CredentialBinding],
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    if bindings.len() > super::model::MAXIMUM_CREDENTIAL_BINDINGS {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.credential_bindings"),
            "credential binding count exceeds the admitted limit",
        ));
    }
    let names = bindings
        .iter()
        .take(super::model::MAXIMUM_CREDENTIAL_BINDINGS)
        .map(|binding| binding.name.clone())
        .collect::<Vec<_>>();
    validate_ordered_identifiers(
        &names,
        super::model::MAXIMUM_CREDENTIAL_BINDINGS,
        &format!("{path}.credential_bindings"),
        diagnostics,
    );
}

pub(crate) fn validate_bounds(
    bounds: super::model::Bounds,
    path: &str,
    diagnostics: &mut Vec<super::model::Diagnostic>,
) {
    let is_timeout_valid = bounds.timeout_seconds >= super::model::MINIMUM_POSITIVE_BOUND
        && bounds.timeout_seconds <= super::model::MAXIMUM_TIMEOUT_SECONDS;
    let is_byte_bound_set_valid = [bounds.input_bytes, bounds.output_bytes, bounds.artifact_bytes]
        .into_iter()
        .all(|bound| bound > 0 && bound <= super::model::MAXIMUM_OPERATION_BYTES);
    if !is_timeout_valid || !is_byte_bound_set_valid {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::Limit,
            format!("{path}.bounds"),
            "operation bounds are not positive and bounded",
        ));
    }
}
