const ATTEMPT_DOMAIN: &str = "lattice.operation-backend-attempt.v1";
const DIRECT_NODE_LABEL: &str = "direct-operation";

impl super::model::BackendAttemptRecord {
    pub fn admitted(plan: &crate::operation::model::InvocationPlan, run_id: crate::engine::executor::RunId) -> Self {
        let material = format!("{}\0{}\0{}", plan.identity, run_id.0, plan.backend_route.planner_plan_identity);
        let attempt_identity = crate::operation::identity::hash_bytes(ATTEMPT_DOMAIN, material.as_bytes());
        Self {
            schema: super::model::ATTEMPT_SCHEMA.to_string(),
            attempt_identity,
            run_id,
            workflow_revision_identity: plan.identity.clone(),
            node_identity: format!("{DIRECT_NODE_LABEL}:{}", plan.instance_id),
            operation_contract_identity: plan.method_identity.clone(),
            route: plan.backend_route.clone(),
            state: super::model::BackendAttemptState::Admitted,
        }
    }

    pub fn finish(&self, state: super::model::BackendAttemptState) -> Self {
        Self { state, ..self.clone() }
    }

    pub fn safe_view(&self) -> super::model::SafeBackendView {
        super::model::SafeBackendView {
            attempt_identity: self.attempt_identity.clone(),
            planner_cohort: self.route.planner_cohort.clone(),
            planner_plan_identity: self.route.planner_plan_identity.clone(),
            requirement_profile_identity: self.route.requirement_profile_identity.clone(),
            backend_class: self.route.backend_class,
            backend_identity: self.route.backend_identity.clone(),
            adapter_identity: self.route.adapter_identity.clone(),
            enforcement_profile_identity: self.route.enforcement_profile_identity.clone(),
            state: self.state,
        }
    }
}

// r[impl lattice.execution_backend_orchestration.dispatch]
pub fn admit_dispatch(
    plan: &crate::operation::model::InvocationPlan,
    attempt: &super::model::BackendAttemptRecord,
) -> Result<(), crate::operation::Error> {
    let context = super::route::RouteContext {
        requirements: &plan.execution_requirements,
        runtime: &plan.runtime,
        method_identity: &plan.method_identity,
        capabilities: &plan.capabilities,
        credentials: &plan.credentials,
        bounds: plan.bounds,
    };
    super::route::validate_route(&plan.backend_route, &context)?;
    let expected = super::model::BackendAttemptRecord::admitted(plan, attempt.run_id);
    if attempt != &expected {
        return Err(crate::operation::Error::Invalid(
            "operation backend attempt or route changed after persistence".to_string(),
        ));
    }
    Ok(())
}

// r[impl lattice.execution_backend_orchestration.dispatch]
pub fn admit_retry_route(
    previous: &super::model::BackendAttemptRecord,
    proposed: &super::model::BackendRoutePlan,
    unknown_outcome: bool,
) -> Result<(), crate::operation::Error> {
    if unknown_outcome {
        return Err(crate::operation::Error::Invalid(
            "unknown backend outcome requires explicit settlement before replanning".to_string(),
        ));
    }
    if &previous.route != proposed {
        return Err(crate::operation::Error::Invalid(
            "backend substitution requires a new explicit plan and attempt".to_string(),
        ));
    }
    Ok(())
}
