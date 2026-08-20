#![allow(
    unknown_lints,
    path_segment_repetition,
    reason = "backend route fields retain exact product identity names at the Animus compatibility boundary"
)]

const PLANNER_POLICY_DOMAIN: &str = "lattice.operation-backend-planner-policy.v1";
const PLANNER_PROJECTION_DOMAIN: &str = "lattice.operation-backend-planner-projection.v1";
const WORKLOAD_DOMAIN: &str = "lattice.operation-backend-workload.v1";
const BACKEND_DOMAIN: &str = "lattice.operation-backend.v1";
const ADAPTER_DOMAIN: &str = "lattice.operation-backend-adapter.v1";
const HOST_BINDING_DOMAIN: &str = "lattice.operation-backend-host-binding.v1";
const AUTHORITY_DOMAIN: &str = "lattice.operation-backend-authority.v1";
const CREDENTIAL_DOMAIN: &str = "lattice.operation-backend-credentials.v1";
const EFFECT_DOMAIN: &str = "lattice.operation-backend-effect.v1";
const BUDGET_DOMAIN: &str = "lattice.operation-backend-budget.v1";
const RESOURCE_DOMAIN: &str = "lattice.operation-backend-resource.v1";
const ENFORCEMENT_DOMAIN: &str = "lattice.operation-backend-enforcement.v1";
const PERSISTENCE_DOMAIN: &str = "lattice.operation-backend-persistence.v1";
const RECOVERY_DOMAIN: &str = "lattice.operation-backend-recovery.v1";

pub struct RouteContext<'a> {
    pub requirements: &'a super::model::ExecutionRequirements,
    pub runtime: &'a crate::operation::model::RuntimeFact,
    pub method_identity: &'a str,
    pub capabilities: &'a [crate::operation::model::CapabilityClass],
    pub credentials: &'a std::collections::BTreeMap<String, crate::credentials::SecretRef>,
    pub bounds: crate::operation::model::Bounds,
}

struct RouteIdentities {
    requirement_profile: String,
    backend: String,
    adapter: String,
    host_binding: String,
    authority: String,
    credentials: String,
    effect: String,
    budget: String,
    resource: String,
    enforcement: String,
    persistence: String,
    recovery: String,
    policy: String,
}

#[derive(serde::Serialize)]
struct ProjectionMaterial<'a> {
    cohort: &'a str,
    schema: &'a str,
    workload: &'a str,
    class: &'a str,
    requirement_profile: &'a str,
    backend: &'a str,
    adapter: &'a str,
    host_binding: &'a str,
    authority: &'a str,
    effect: &'a str,
    resource: &'a str,
    enforcement: &'a str,
    policy: &'a str,
}

// r[impl lattice.execution_backend_orchestration.planner_mapping]
// r[impl lattice.execution_backend_orchestration.admission]
pub fn plan_route(context: &RouteContext<'_>) -> Result<super::model::BackendRoutePlan, crate::operation::Error> {
    context.requirements.validate()?;
    if !context.runtime.available {
        return Err(crate::operation::Error::Invalid("operation backend is unavailable during planning".to_string()));
    }
    let class = selected_class(context.requirements)?;
    let identities = route_identities(context, class)?;
    let workload = crate::operation::identity::hash_value(
        WORKLOAD_DOMAIN,
        &(context.method_identity, identities.requirement_profile.as_str()),
    )?;
    let planner_plan_identity = planner_projection_identity(&identities, &workload, class)?;
    Ok(super::model::BackendRoutePlan {
        schema: super::model::ROUTE_SCHEMA.to_string(),
        planner_cohort: super::model::PLANNER_COHORT.to_string(),
        planner_schema: super::model::PLANNER_SCHEMA.to_string(),
        planner_plan_identity,
        requirement_profile_identity: identities.requirement_profile,
        backend_class: class,
        backend_identity: identities.backend,
        adapter_identity: identities.adapter,
        application_host_binding_identity: identities.host_binding,
        runtime_authority_identity: identities.authority,
        credential_reference_identity: identities.credentials,
        effect_permission_identity: identities.effect,
        budget_identity: identities.budget,
        resource_profile_identity: identities.resource,
        enforcement_profile_identity: identities.enforcement,
        persistence_policy_identity: identities.persistence,
        recovery_policy_identity: identities.recovery,
        policy_identity: identities.policy,
        resources: context.requirements.resources,
    })
}

pub fn validate_route(
    route: &super::model::BackendRoutePlan,
    context: &RouteContext<'_>,
) -> Result<(), crate::operation::Error> {
    let expected = plan_route(context)?;
    if route != &expected {
        return Err(crate::operation::Error::Invalid(
            "operation backend route differs from current admitted facts".to_string(),
        ));
    }
    Ok(())
}

pub fn compatibility_projection(
    class: super::model::BackendClass,
    requirements: &super::model::ExecutionRequirements,
) -> Result<super::model::BackendClass, crate::operation::Error> {
    requirements.validate()?;
    parse_class(class.label())
}

fn selected_class(
    requirements: &super::model::ExecutionRequirements,
) -> Result<super::model::BackendClass, crate::operation::Error> {
    match (requirements.process, requirements.isolation) {
        (super::model::ProcessModel::InProcess, super::model::IsolationClaim::MediatedIsolation) => {
            Ok(super::model::BackendClass::EmbeddedComponent)
        }
        (
            super::model::ProcessModel::SingleProcess | super::model::ProcessModel::ProcessTree,
            super::model::IsolationClaim::GuardedProcessIsolation,
        ) => Ok(super::model::BackendClass::GuardedLocalProcess),
        (_, super::model::IsolationClaim::HostIsolation) => Ok(super::model::BackendClass::IsolatedHost),
        _ => Err(crate::operation::Error::Invalid(
            "execution requirements do not map to a reviewed backend class".to_string(),
        )),
    }
}

fn parse_class(value: &str) -> Result<super::model::BackendClass, crate::operation::Error> {
    match value {
        "embedded_component" => Ok(super::model::BackendClass::EmbeddedComponent),
        "guarded_local_process" => Ok(super::model::BackendClass::GuardedLocalProcess),
        "isolated_host" => Ok(super::model::BackendClass::IsolatedHost),
        "external_machine" => Ok(super::model::BackendClass::ExternalMachine),
        _ => Err(crate::operation::Error::Invalid("Animus planner class is outside the frozen cohort".to_string())),
    }
}

fn route_identities(
    context: &RouteContext<'_>,
    class: super::model::BackendClass,
) -> Result<RouteIdentities, crate::operation::Error> {
    let runtime = context.runtime;
    Ok(RouteIdentities {
        requirement_profile: crate::operation::identity::hash_value(REQUIREMENT_DOMAIN, context.requirements)?,
        backend: crate::operation::identity::hash_value(
            BACKEND_DOMAIN,
            &(runtime.kind.as_str(), runtime.key.as_str(), class.label()),
        )?,
        adapter: crate::operation::identity::hash_value(
            ADAPTER_DOMAIN,
            &(runtime.kind.as_str(), runtime.key.as_str(), runtime.executor_identity.as_str()),
        )?,
        host_binding: crate::operation::identity::hash_value(
            HOST_BINDING_DOMAIN,
            &(runtime.kind.as_str(), runtime.key.as_str(), runtime.executable_path.as_str()),
        )?,
        authority: crate::operation::identity::hash_value(
            AUTHORITY_DOMAIN,
            &(context.method_identity, context.capabilities),
        )?,
        credentials: crate::operation::identity::hash_value(CREDENTIAL_DOMAIN, context.credentials)?,
        effect: crate::operation::identity::hash_value(
            EFFECT_DOMAIN,
            &(context.method_identity, runtime.kind.as_str()),
        )?,
        budget: crate::operation::identity::hash_value(
            BUDGET_DOMAIN,
            &(context.bounds, context.requirements.resources),
        )?,
        resource: crate::operation::identity::hash_value(RESOURCE_DOMAIN, &context.requirements.resources)?,
        enforcement: crate::operation::identity::hash_value(
            ENFORCEMENT_DOMAIN,
            &(runtime.executor_identity.as_str(), runtime.maximum_bounds),
        )?,
        persistence: crate::operation::identity::hash_value(
            PERSISTENCE_DOMAIN,
            &(crate::operation::run_model::RUN_SCHEMA, "persist-before-dispatch"),
        )?,
        recovery: crate::operation::identity::hash_value(
            RECOVERY_DOMAIN,
            &("no-silent-substitution", "explicit-unknown-settlement"),
        )?,
        policy: crate::operation::identity::hash_value(
            PLANNER_POLICY_DOMAIN,
            &(super::model::PLANNER_COHORT, super::model::PLANNER_SCHEMA, class.label()),
        )?,
    })
}

fn planner_projection_identity(
    identities: &RouteIdentities,
    workload: &str,
    class: super::model::BackendClass,
) -> Result<String, crate::operation::Error> {
    crate::operation::identity::hash_value(PLANNER_PROJECTION_DOMAIN, &ProjectionMaterial {
        cohort: super::model::PLANNER_COHORT,
        schema: super::model::PLANNER_SCHEMA,
        workload,
        class: class.label(),
        requirement_profile: &identities.requirement_profile,
        backend: &identities.backend,
        adapter: &identities.adapter,
        host_binding: &identities.host_binding,
        authority: &identities.authority,
        effect: &identities.effect,
        resource: &identities.resource,
        enforcement: &identities.enforcement,
        policy: &identities.policy,
    })
}

const REQUIREMENT_DOMAIN: &str = "lattice.operation-backend-requirements.v1";
