//! Admission of a component method through the existing operation planner.

use crate::operation_pack::model;
use crate::operation_pack::wasm;

pub(super) fn admit(
    request: &super::super::execution::Request<'_>,
    operation: &model::OperationBinding,
    runtime: &model::RuntimeArtifact,
) -> Result<crate::operation::cli::AdmittedPlan, crate::operation_pack::Error> {
    super::super::execution::validate_external_instances(request.instances_path)?;
    let descriptor = runtime.wasm.as_ref().ok_or(crate::operation_pack::Error::Invalid("component descriptor"))?;
    let root = crate::capability::open_root_path(request.pack_root)
        .map_err(|_| crate::operation_pack::Error::Io("component catalog capability root"))?;
    let catalog_bytes =
        super::verification::read_content(&root, &request.admitted.candidate, &operation.catalog_content_id)?;
    let catalog_source = std::str::from_utf8(&catalog_bytes)
        .map_err(|_| crate::operation_pack::Error::Invalid("component catalog UTF-8"))?;
    let value = crate::nickel::evaluate_source(catalog_source)
        .map_err(|error| crate::operation_pack::Error::Nickel(error.to_string()))?;
    let decoded = serde_json::from_value(value)
        .map_err(|_| crate::operation_pack::Error::Invalid("component operation catalog schema"))?;
    let catalog = crate::operation::catalog::validate(decoded)?;
    let mut instances = crate::operation::format::load_instances(request.instances_path, &catalog)?;
    let instance = instances
        .remove(request.instance_id)
        .ok_or(crate::operation_pack::Error::Invalid("component operation instance"))?;
    if instance.definition.type_id != operation.type_id || instance.definition.type_revision != operation.type_revision
    {
        return Err(crate::operation_pack::Error::Invalid("component operation binding"));
    }
    let content = request
        .admitted
        .candidate
        .manifest
        .contents
        .iter()
        .find(|content| content.id == runtime.content_id)
        .ok_or(crate::operation_pack::Error::Invalid("component runtime content"))?;
    let component_path = request.pack_root.join(&content.path);
    let executable_path = component_path
        .to_str()
        .ok_or(crate::operation_pack::Error::Invalid("component path encoding"))?
        .to_string();
    let maximum_bounds = crate::operation::Bounds {
        timeout_seconds: std::time::Duration::from_millis(descriptor.resources.deadline_millis).as_secs(),
        input_bytes: usize::try_from(descriptor.resources.maximum_input_bytes)
            .map_err(|_| crate::operation_pack::Error::Invalid("component input bound"))?,
        output_bytes: usize::try_from(descriptor.resources.maximum_output_bytes)
            .map_err(|_| crate::operation_pack::Error::Invalid("component output bound"))?,
        artifact_bytes: usize::try_from(descriptor.resources.maximum_component_bytes)
            .map_err(|_| crate::operation_pack::Error::Invalid("component artifact bound"))?,
    };
    let plan = crate::operation::planning::plan(crate::operation::PlanRequest {
        catalog: &catalog,
        instance: &instance,
        method: request.method,
        arguments: crate::operation::format::parse_arguments(request.arguments_json)?,
        expected_catalog_identity: &catalog.identity,
        expected_instance_identity: &instance.identity,
        runtime: crate::operation::RuntimeFact {
            kind: crate::operation::runtime::component::RUNTIME_KIND.to_string(),
            key: crate::operation::runtime::component::RUNTIME_KEY.to_string(),
            executor_identity: content.content_identity.clone(),
            executable_path,
            available: true,
            maximum_bounds,
        },
        policy: crate::operation::CapabilityPolicy {
            allowed: runtime.capabilities.clone(),
            allow_mutation: false,
        },
        preflight: crate::operation::PreflightFacts {
            executable_available: false,
            repository_root_admitted: false,
            credential_references_valid: instance.definition.credentials.is_empty(),
        },
    })?;
    crate::operation::planning::validate_plan_admission(&plan, &catalog, &instance)?;
    if plan.runtime.kind != wasm::RUNTIME_KIND
        || operation.launch_label != crate::operation::runtime::component::RUNTIME_KEY
        || plan.capabilities.iter().any(|capability| !runtime.capabilities.contains(capability))
        || plan.bounds.timeout_seconds > runtime.timeout_seconds
        || plan.bounds.input_bytes > runtime.input_bytes
        || plan.bounds.output_bytes > runtime.output_bytes
        || !plan.credentials.is_empty()
    {
        return Err(crate::operation_pack::Error::Invalid("component invocation scope"));
    }
    Ok(crate::operation::cli::AdmittedPlan {
        catalog,
        instance,
        plan,
    })
}
