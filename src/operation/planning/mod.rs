#![allow(
    unknown_lints,
    non_trait_imports,
    reason = "invocation planning uses its local ordered argument and operation-contract vocabulary"
)]
#![allow(
    assertion_density,
    reason = "fallible plan admission uses typed diagnostics; positive and adversarial tests cover each invariant"
)]
#![allow(
    borrowed_argument_types,
    reason = "plan validators append diagnostics to one caller-owned growable accumulator"
)]
use std::collections::BTreeMap;

pub fn plan(mut request: super::model::PlanRequest<'_>) -> Result<super::model::InvocationPlan, super::Error> {
    let mut diagnostics = validate_request_identity(&request);
    let operation_type = match request.catalog.types.get(&request.instance.definition.type_id) {
        Some(value) => value,
        None => {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::MissingType,
                "plan.type_id",
                "operation type is not present in the admitted catalog",
            ));
            return Err(super::Error::Diagnostics(super::ordered(diagnostics)));
        }
    };
    let method = match operation_type.definition.methods.iter().find(|method| method.name == request.method) {
        Some(value) => value,
        None => {
            diagnostics.push(super::diagnostic(
                super::model::DiagnosticCode::MissingMethod,
                "plan.method",
                "operation method is not present in the admitted type",
            ));
            return Err(super::Error::Diagnostics(super::ordered(diagnostics)));
        }
    };

    let supplied_arguments = std::mem::take(&mut request.arguments);
    let arguments = merged_arguments(&request.instance.definition, method, supplied_arguments);
    diagnostics.extend(super::schema::validate_value(
        &method.arguments,
        &serde_json::Value::Object(arguments.clone().into_iter().collect()),
        "plan.arguments",
        super::model::DiagnosticCode::InvalidValue,
    ));
    validate_input_bound(&arguments, operation_type.definition.bounds, &mut diagnostics);
    validate_runtime(operation_type, &request.runtime, &mut diagnostics);
    validate_capabilities(method, &request.policy, &mut diagnostics);
    validate_preflight(method, &request, &mut diagnostics);
    if !diagnostics.is_empty() {
        return Err(super::Error::Diagnostics(super::ordered(diagnostics)));
    }

    let method_identity =
        operation_type.method_identities.get(request.method).cloned().ok_or(super::Error::Encoding)?;
    let output_contract_identity =
        operation_type.output_schema_identities.get(request.method).cloned().ok_or(super::Error::Encoding)?;
    let arguments_identity = super::identity::arguments(&arguments)?;
    let mut plan = super::model::InvocationPlan {
        schema: super::model::PLAN_SCHEMA.to_string(),
        identity: String::new(),
        catalog_identity: request.catalog.identity.clone(),
        type_id: operation_type.definition.id.clone(),
        type_revision: operation_type.definition.revision.clone(),
        instance_id: request.instance.definition.id.clone(),
        instance_identity: request.instance.identity.clone(),
        method: method.name.clone(),
        method_identity,
        arguments,
        arguments_identity,
        output_contract: method.output.clone(),
        output_contract_identity,
        runtime: request.runtime,
        capabilities: method.capabilities.clone(),
        target: request.instance.definition.target.clone(),
        credentials: request.instance.definition.credentials.clone(),
        bounds: operation_type.definition.bounds,
        required_non_claims: operation_type.definition.required_non_claims.clone(),
    };
    plan.identity = super::identity::invocation(&plan)?;
    debug_assert!(super::identity::validate(&plan.identity));
    Ok(plan)
}

pub fn validate_plan_identity(plan: &super::model::InvocationPlan) -> Result<(), super::Error> {
    if plan.schema != super::model::PLAN_SCHEMA || !super::identity::validate(&plan.identity) {
        return Err(super::Error::Diagnostics(vec![super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.identity",
            "operation invocation plan schema or identity is invalid",
        )]));
    }
    let mut diagnostics = super::schema::validate_contract(&plan.output_contract, "plan.output_contract");
    if super::identity::arguments(&plan.arguments)? != plan.arguments_identity {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.arguments_identity",
            "operation invocation arguments do not match their identity",
        ));
    }
    if super::identity::schema(&plan.output_contract)? != plan.output_contract_identity {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.output_contract_identity",
            "operation output contract does not match its identity",
        ));
    }
    let computed = super::identity::invocation(plan)?;
    if computed != plan.identity {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.identity",
            "operation invocation plan identity does not match its fields",
        ));
    }
    if !diagnostics.is_empty() {
        return Err(super::Error::Diagnostics(super::ordered(diagnostics)));
    }
    Ok(())
}

#[allow(
    unknown_lints,
    function_length,
    reason = "one exact comparator keeps all catalog and instance authority-bearing fields visible"
)]
pub fn validate_plan_admission(
    plan: &super::model::InvocationPlan,
    catalog: &super::model::ValidatedCatalog,
    instance: &super::model::ValidatedInstance,
) -> Result<(), super::Error> {
    validate_plan_identity(plan)?;
    let rebuilt_catalog = super::catalog::validate(super::model::Catalog {
        schema: super::model::CATALOG_SCHEMA.to_string(),
        types: catalog.types.values().map(|operation_type| operation_type.definition.clone()).collect(),
    })?;
    let rebuilt_instance = super::instance::validate(
        super::model::InstanceFile {
            schema: super::model::INSTANCE_SCHEMA.to_string(),
            instances: vec![instance.definition.clone()],
        },
        &rebuilt_catalog,
    )?
    .remove(&instance.definition.id)
    .ok_or(super::Error::Encoding)?;
    let mut diagnostics = Vec::new();
    if &rebuilt_catalog != catalog || &rebuilt_instance != instance {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.admission",
            "operation catalog or instance admission is not reproducible",
        ));
    }
    let Some(operation_type) = catalog.types.get(&plan.type_id) else {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::MissingType,
            "plan.type_id",
            "operation plan type is absent from the admitted catalog",
        ));
        return Err(super::Error::Diagnostics(super::ordered(diagnostics)));
    };
    let Some(method) = operation_type.definition.methods.iter().find(|method| method.name == plan.method) else {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::MissingMethod,
            "plan.method",
            "operation plan method is absent from the admitted catalog",
        ));
        return Err(super::Error::Diagnostics(super::ordered(diagnostics)));
    };
    let method_identity = operation_type.method_identities.get(&plan.method).map(String::as_str);
    let output_identity = operation_type.output_schema_identities.get(&plan.method).map(String::as_str);
    let is_catalog_field_set_exact = plan.catalog_identity == catalog.identity
        && plan.type_revision == operation_type.definition.revision
        && method_identity == Some(plan.method_identity.as_str())
        && output_identity == Some(plan.output_contract_identity.as_str())
        && plan.output_contract == method.output
        && plan.capabilities == method.capabilities
        && plan.bounds == operation_type.definition.bounds
        && plan.required_non_claims == operation_type.definition.required_non_claims;
    let is_instance_field_set_exact = plan.instance_id == instance.definition.id
        && plan.instance_identity == instance.identity
        && plan.type_id == instance.definition.type_id
        && plan.target == instance.definition.target
        && plan.credentials == instance.definition.credentials;
    if !is_catalog_field_set_exact || !is_instance_field_set_exact {
        diagnostics.push(super::diagnostic(
            super::model::DiagnosticCode::InvalidIdentity,
            "plan.admission",
            "operation plan differs from its admitted catalog or instance",
        ));
    }
    diagnostics.extend(super::schema::validate_value(
        &method.arguments,
        &serde_json::Value::Object(plan.arguments.clone().into_iter().collect()),
        "plan.arguments",
        super::model::DiagnosticCode::InvalidValue,
    ));
    validate_input_bound(&plan.arguments, plan.bounds, &mut diagnostics);
    validate_runtime(operation_type, &plan.runtime, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(super::Error::Diagnostics(super::ordered(diagnostics)))
    }
}

include!("validation.rs");
