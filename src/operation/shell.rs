#![allow(
    unknown_lints,
    assertion_density,
    reason = "the imperative operation shell uses explicit fail-closed branches and direct authority tests"
)]

const LOCAL_AUTHORITY_RUN_DOMAIN: &[u8] = b"lattice.operation-local-run.v1";
const LOCAL_AUTHORITY_DOMAIN: &[u8] = b"lattice.operation-local-read-only-authority.v1";
const LOCAL_GRANT_DOMAIN: &[u8] = b"lattice.operation-local-read-only-grant.v1";
const LOCAL_RECEIPT_DOMAIN: &[u8] = b"lattice.operation-local-read-only-receipt.v1";
const LOCAL_AUTHORITY_CONTRACT: &str = "lattice-process";
const TYPED_OPERATION_EFFECT: &str = "lattice/operation";
const PROCESS_EXECUTE_ABILITY: &str = "process/execute";
const PROCESS_RESOURCE_PREFIX: &str = "lattice://process/";
const DIRECT_OPERATION_NODE_INDEX: u64 = 0;

pub trait Clock {
    fn observe_epoch_millis(&self) -> i64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    #[allow(
        unknown_lints,
        ambient_clock,
        reason = "SystemClock is the sole shell clock adapter; domain operations depend on the injected Clock port"
    )]
    fn observe_epoch_millis(&self) -> i64 {
        chrono::Utc::now().timestamp_millis()
    }
}

#[derive(Debug, Clone)]
pub struct LocalReadOnlyAuthorityProvider {
    plan_identity: String,
    expected_payload: String,
    expected_resource: String,
}

impl LocalReadOnlyAuthorityProvider {
    pub fn new(
        catalog: &super::model::ValidatedCatalog,
        instance: &super::model::ValidatedInstance,
        plan: &super::model::InvocationPlan,
    ) -> Result<Self, super::Error> {
        super::planning::validate_plan_admission(plan, catalog, instance)?;
        let process = super::runtime::process_request_plan(plan)
            .map_err(|observation| super::Error::Invalid(observation.diagnostic))?;
        Ok(Self {
            plan_identity: plan.identity.clone(),
            expected_payload: crate::engine::effect::runtime::process_plan_identity(&process),
            expected_resource: format!("{PROCESS_RESOURCE_PREFIX}{DIRECT_OPERATION_NODE_INDEX}"),
        })
    }
}

impl crate::engine::effect::execution::RunContextProvider for LocalReadOnlyAuthorityProvider {
    fn load(
        &self,
        run_id: crate::engine::executor::RunId,
        store: Option<crate::store::Store>,
    ) -> Result<crate::engine::effect::execution::RunContext, crate::engine::effect::execution::Error> {
        let attempts: Box<dyn crate::engine::effect::dispatch::EffectAttemptStore> = match store {
            Some(store) => Box::new(crate::engine::effect::execution::StoreAttempts::new(store)),
            None => Box::new(crate::engine::effect::execution::UnavailableAttempts),
        };
        let run_number = run_id.0.to_string();
        let run_reference = crate::engine::effect::authority::blake3_reference(LOCAL_AUTHORITY_RUN_DOMAIN, &[
            run_number.as_str(),
            self.plan_identity.as_str(),
        ]);
        let authority_reference =
            crate::engine::effect::authority::blake3_reference(LOCAL_AUTHORITY_DOMAIN, &[self.plan_identity.as_str()]);
        let dispatch = crate::engine::effect::dispatch::Shell::new(
            Box::new(LocalAuthorityFacts {
                plan_identity: self.plan_identity.clone(),
                expected_payload: self.expected_payload.clone(),
                expected_resource: self.expected_resource.clone(),
            }),
            Box::new(NoScopeSnapshot),
            attempts,
            Box::new(crate::engine::effect::runtime::ConfiguredRoots::new(std::collections::BTreeMap::new())),
        );
        Ok(crate::engine::effect::execution::RunContext::new(
            crate::engine::effect::authority::RunIdentity::new(run_reference)?,
            crate::engine::effect::authority::AuthorityIdentity::new(authority_reference)?,
            crate::engine::effect::authority::CollaborationScopeBinding::Unscoped,
            std::sync::Arc::new(dispatch),
        ))
    }
}

struct LocalAuthorityFacts {
    plan_identity: String,
    expected_payload: String,
    expected_resource: String,
}

impl crate::engine::effect::dispatch::AuthorityFactsProvider for LocalAuthorityFacts {
    fn load(
        &self,
        binding: &crate::engine::effect::authority::IntentBinding,
    ) -> Result<crate::engine::effect::authority::AuthorizationFacts, crate::engine::effect::dispatch::Error> {
        let is_operation_exact = binding.operation.as_str() == TYPED_OPERATION_EFFECT;
        let is_ability_exact = binding.ability.as_str() == PROCESS_EXECUTE_ABILITY;
        let is_resource_exact = binding.resource.as_str() == self.expected_resource;
        let is_payload_exact = binding.payload.as_str() == self.expected_payload;
        let is_operation_and_ability_exact = is_operation_exact && is_ability_exact;
        let is_resource_and_payload_exact = is_resource_exact && is_payload_exact;
        let is_binding_exact = is_operation_and_ability_exact && is_resource_and_payload_exact;
        if !is_binding_exact {
            return Err(crate::engine::effect::dispatch::Error::AuthorityDenied);
        }
        let resource = binding.resource.as_str().to_string();
        let ability = binding.ability.as_str().to_string();
        let request_ref = binding.request.as_str().to_string();
        let grant_ref = crate::engine::effect::authority::blake3_reference(LOCAL_GRANT_DOMAIN, &[
            self.plan_identity.as_str(),
            request_ref.as_str(),
        ]);
        let receipt_ref = crate::engine::effect::authority::blake3_reference(LOCAL_RECEIPT_DOMAIN, &[
            grant_ref.as_str(),
            request_ref.as_str(),
        ]);
        Ok(crate::engine::effect::authority::AuthorizationFacts {
            binding: binding.clone(),
            basalt_input: basalt::authority_core::AuthorityInput {
                schema: basalt::authority_core::AUTHORITY_INPUT_SCHEMA.to_string(),
                request: basalt::authority_core::AuthorityRequest {
                    contract_id: LOCAL_AUTHORITY_CONTRACT.to_string(),
                    resource: resource.clone(),
                    ability: ability.clone(),
                    request_ref: request_ref.clone(),
                },
                verified_grants: vec![basalt::authority_core::VerifiedGrantRef {
                    grant_ref: grant_ref.clone(),
                    verification_receipt_ref: receipt_ref.clone(),
                    resource,
                    ability,
                    kind: basalt::authority_core::VerifiedGrantKind::Direct,
                }],
                verification_receipts: vec![basalt::authority_core::UcanVerificationReceiptRef {
                    receipt_ref,
                    request_ref,
                    verified_grant_refs: vec![grant_ref],
                    disposition: basalt::authority_core::VerificationRefDisposition::Current,
                }],
                policy_refs: vec![crate::engine::effect::dispatch::trusted_policy_ref(
                    LOCAL_AUTHORITY_CONTRACT,
                )],
                caveat: basalt::authority_core::AuthorityCaveatDisposition::Satisfied,
                replay: basalt::authority_core::AuthorityReplayDisposition::Admitted,
                claims: vec![basalt::authority_core::AuthorityClaim::ExactRequest],
                non_claims: basalt::authority_core::REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_string).to_vec(),
            },
            trusted_policy_ref: crate::engine::effect::dispatch::trusted_policy_ref(LOCAL_AUTHORITY_CONTRACT),
            scope_requirement: crate::engine::effect::authority::ScopeRequirement::UnscopedPermitted,
            scope_snapshot: None,
            revocation: crate::engine::effect::authority::RevocationDisposition::Current,
        })
    }
}

struct NoScopeSnapshot;

impl crate::engine::effect::dispatch::ScopeSnapshotProvider for NoScopeSnapshot {
    fn load_current(
        &self,
        _binding: &crate::engine::effect::authority::IntentBinding,
    ) -> Result<Option<crate::engine::effect::authority::ScopeSnapshotFacts>, crate::engine::effect::dispatch::Error>
    {
        Ok(None)
    }
}

pub struct DirectRunRequest<'a> {
    pub store: &'a crate::store::Store,
    pub plan: &'a super::model::InvocationPlan,
    pub authority_provider: &'a dyn crate::engine::effect::execution::RunContextProvider,
    pub process: &'a crate::process::core::RuntimeConfig,
    pub credentials: &'a dyn crate::credentials::CredentialProvider,
    pub clock: &'a dyn Clock,
    pub cancellation: Option<&'a std::sync::atomic::AtomicBool>,
}

pub fn run_direct(request: DirectRunRequest<'_>) -> Result<super::run_model::DirectRunResult, super::Error> {
    super::planning::validate_plan_identity(request.plan)?;
    let started = request
        .store
        .begin_operation_run(request.plan, request.clock.observe_epoch_millis())
        .map_err(|error| super::Error::Store(error.to_string()))?;
    debug_assert_eq!(started.state, super::run_model::RunState::Started);
    let observation = execute_authorized(&request, started.run_id);
    let terminal = started.finished(&observation, request.clock.observe_epoch_millis());
    request
        .store
        .finish_operation_run(&terminal)
        .map_err(|error| super::Error::Store(error.to_string()))?;
    Ok(super::run_model::DirectRunResult {
        record: terminal,
        output: observation.output,
    })
}

fn execute_authorized(
    request: &DirectRunRequest<'_>,
    run_id: crate::engine::executor::RunId,
) -> super::runtime::Observation {
    let Ok((workflow, node_id)) = super::promotion::workflow_for_plan(request.plan, "direct-operation") else {
        return super::runtime::unknown("operation workflow admission failed");
    };
    let Ok(context) = request.authority_provider.load(run_id, Some(request.store.clone())) else {
        return super::runtime::unknown("operation authority context is unavailable");
    };
    let Ok(mut session) = crate::engine::effect::execution::Session::new(&workflow, &context) else {
        return super::runtime::unknown("operation authority machine admission failed");
    };
    if session.mark_ready(node_id).is_err() {
        return super::runtime::unknown("operation authority readiness transition failed");
    }
    let process_request = match super::runtime::prepare_process_request(request.plan, request.credentials) {
        Ok(process_request) => process_request,
        Err(observation) => return observation,
    };
    let payload = crate::engine::effect::runtime::process_plan_identity(&process_request.process);
    let binding = match session.bind_effect(node_id, &payload) {
        Ok(binding) => binding,
        Err(error) => {
            return super::runtime::unknown(&format!("operation authority denied the effect transition: {error}"));
        }
    };
    let dispatch = session.dispatch();
    let http = crate::http::NoopHttpClient;
    let runtime = match request.cancellation {
        Some(cancellation) => crate::engine::effect::runtime::Adapters::new_with_cancellation(
            dispatch,
            &http,
            request.process,
            cancellation,
        ),
        None => crate::engine::effect::runtime::Adapters::new(dispatch, &http, request.process),
    };
    let observation = match dispatch.dispatch(&runtime, binding, &process_request) {
        Ok(Ok(output)) => super::runtime::classify_process_output(request.plan, output, &process_request.secret_values)
            .unwrap_or_else(std::convert::identity),
        Ok(Err(error)) => super::runtime::process_failure(error),
        Err(error) => {
            super::runtime::unknown(&format!("operation authorized process outcome is unavailable: {error:?}"))
        }
    };
    let status = match observation.class {
        super::runtime::ObservationClass::Completed => crate::engine::executor::NodeStatus::Completed {
            outputs: std::collections::HashMap::new(),
            cached: false,
        },
        super::runtime::ObservationClass::DeterministicFailure | super::runtime::ObservationClass::Unknown => {
            crate::engine::executor::NodeStatus::Failed {
                error: observation.diagnostic.clone(),
            }
        }
    };
    if session.finish(node_id, &status).is_err() {
        return super::runtime::unknown("operation authority terminal transition failed");
    }
    observation
}

pub fn environment_provider_for_plan(
    plan: &super::model::InvocationPlan,
) -> crate::credentials::EnvironmentCredentialProvider {
    let keys = plan
        .credentials
        .values()
        .filter(|reference| reference.provider == "env")
        .map(|reference| reference.key.clone())
        .collect::<Vec<_>>();
    crate::credentials::EnvironmentCredentialProvider::from_current_env(&keys)
}
