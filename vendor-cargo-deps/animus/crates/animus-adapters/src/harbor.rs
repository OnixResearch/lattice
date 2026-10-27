//! Implementation-free ports for the optional Harbor-compatible evaluation lane.

/// Exact reviewed Harbor release.
pub const HARBOR_RELEASE: &str = "v0.18.0";
/// Exact reviewed Harbor commit.
pub const HARBOR_REVISION: &str = "527d50deb63a5d279e8c20593c18a2cbc7f61f9e";
/// Exact reviewed task schema.
pub const HARBOR_TASK_SCHEMA: &str = "1.3";
/// Exact reviewed trajectory schema.
pub const ATIF_SCHEMA: &str = "ATIF-v1.7";

/// One admitted evaluation trial request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrialRequest {
    /// Stable trial identity.
    pub trial_identity: alloc::string::String,
    /// Exact task identity.
    pub task_identity: alloc::string::String,
    /// Exact agent identity.
    pub agent_identity: alloc::string::String,
    /// Exact model-route identity.
    pub model_route_identity: alloc::string::String,
    /// Exact environment identity.
    pub environment_identity: alloc::string::String,
    /// Exact verifier identity.
    pub verifier_identity: alloc::string::String,
    /// Bounded artifact identities.
    pub artifacts: alloc::vec::Vec<alloc::string::String>,
}

/// Bounded observed outcome class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutcomeClass {
    /// Service completed and supplied typed observations.
    Completed,
    /// Service was unavailable.
    Unavailable,
    /// Policy denied execution.
    Denied,
    /// Cancellation was observed.
    Cancelled,
    /// Deadline was observed.
    TimedOut,
    /// Output was malformed.
    Malformed,
    /// Evidence was stale or mismatched.
    Stale,
}

/// Normalized trial observation without raw payloads.
#[derive(Clone, Debug, PartialEq)]
pub struct TrialObservation {
    /// Stable trial identity.
    pub trial_identity: alloc::string::String,
    /// Outcome class.
    pub outcome: OutcomeClass,
    /// ATIF projection identity.
    pub trajectory_identity: Option<alloc::string::String>,
    /// Finite attributed rewards.
    pub rewards: alloc::vec::Vec<(alloc::string::String, f64)>,
    /// Bounded artifact identities.
    pub artifacts: alloc::vec::Vec<alloc::string::String>,
}

/// Environment preparation and trial execution port.
pub trait EvaluationEnvironmentService {
    /// Execute one admitted trial without ambient fallback.
    ///
    /// # Errors
    ///
    /// Returns an explicit outcome class for every unavailable or failed path.
    fn execute(&mut self, request: &TrialRequest) -> Result<TrialObservation, OutcomeClass>;
}

/// Independently controlled verifier port.
pub trait VerifierService {
    /// Verify one admitted subject through an explicit artifact manifest.
    ///
    /// # Errors
    ///
    /// Returns an explicit outcome class without replacing the selected verifier.
    fn verify(&mut self, request: &TrialRequest) -> Result<TrialObservation, OutcomeClass>;
}

/// Native request produced from one exact Harbor port call.
#[cfg(feature = "evaluation")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HarborNativeRequest<'a> {
    /// Harbor environment call translated to native agent execution.
    Agent(animus_adapter_eval::AgentExecutionRequest<'a>),
    /// Harbor verifier call translated to native verifier execution.
    Verifier(animus_adapter_eval::VerifierExecutionRequest<'a>),
}

/// Fail-closed Harbor-to-native compatibility failure.
#[cfg(feature = "evaluation")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarborCompatibilityError {
    /// A required task, run, role, model, environment, verifier, or artifact identity was lost.
    IdentityLoss,
    /// The selected native service is not a Harbor environment or verifier port.
    UnsupportedService,
    /// Native request binding failed.
    Binding(animus_adapter_eval::BindingError),
    /// A Harbor observation is malformed or exceeds native request bounds.
    Observation,
}

/// Return the exact ordered Harbor artifact-manifest identity.
#[cfg(feature = "evaluation")]
#[must_use]
pub fn harbor_artifact_manifest_identity(
    artifacts: &[alloc::string::String],
) -> alloc::string::String {
    let mut hasher = blake3::Hasher::new();
    let domain = b"animus.harbor-artifact-manifest.v1";
    frame_harbor_identity(&mut hasher, domain);
    for artifact in artifacts {
        frame_harbor_identity(&mut hasher, artifact.as_bytes());
    }
    alloc::format!("b3:{}", hasher.finalize().to_hex())
}

/// Exact Harbor request and native binding presented for compatibility conversion.
#[cfg(feature = "evaluation")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HarborBridgeRequest<'a> {
    /// Harbor-owned trial request.
    pub trial: &'a TrialRequest,
    /// Selected native service.
    pub service: animus_adapter_eval::ServiceKind,
    /// Full native evaluation binding.
    pub binding: &'a animus_adapter_eval::EvaluationBinding<'a>,
    /// Requested item capacity.
    pub requested_items: u32,
    /// Requested byte capacity.
    pub requested_bytes: u64,
}

/// Translate an exact Harbor environment or verifier request into a native service request.
///
/// # Errors
/// Returns identity loss, unsupported service, or native binding failure before dispatch.
#[cfg(feature = "evaluation")]
pub fn bridge_harbor_request<'a>(
    bridge: &HarborBridgeRequest<'a>,
    admission: &animus_adapter_eval::ServiceAdmission,
) -> Result<HarborNativeRequest<'a>, HarborCompatibilityError> {
    if bridge.trial.trial_identity != bridge.binding.run_identity
        || bridge.trial.task_identity != bridge.binding.task_identity
        || bridge.trial.agent_identity != bridge.binding.role_identity
        || bridge.trial.model_route_identity != bridge.binding.model_identity
        || bridge.trial.environment_identity != bridge.binding.solver_environment_identity
        || bridge.trial.verifier_identity != bridge.binding.verifier_identity
        || harbor_artifact_manifest_identity(&bridge.trial.artifacts)
            != bridge.binding.artifact_identity
    {
        return Err(HarborCompatibilityError::IdentityLoss);
    }
    let request = animus_adapter_eval::build_service_request(
        bridge.service,
        admission.identity(),
        admission.implementation_identity(),
        bridge.binding,
        bridge.requested_items,
        bridge.requested_bytes,
        admission,
    )
    .map_err(HarborCompatibilityError::Binding)?;
    match bridge.service {
        animus_adapter_eval::ServiceKind::AgentExecution => {
            animus_adapter_eval::AgentExecutionRequest::new(request)
                .map(HarborNativeRequest::Agent)
                .map_err(HarborCompatibilityError::Binding)
        }
        animus_adapter_eval::ServiceKind::VerifierExecution => {
            animus_adapter_eval::VerifierExecutionRequest::new(request)
                .map(HarborNativeRequest::Verifier)
                .map_err(HarborCompatibilityError::Binding)
        }
        _ => Err(HarborCompatibilityError::UnsupportedService),
    }
}

/// Translate one Harbor observation to bounded native observation evidence.
///
/// # Errors
/// Returns observation when trial identity, terminal evidence, or bounds are invalid.
#[cfg(feature = "evaluation")]
pub fn bridge_harbor_observation(
    request: &animus_adapter_eval::ServiceRequest<'_>,
    observation: &TrialObservation,
    observed_bytes: u64,
) -> Result<animus_adapter_eval::ServiceObservation, HarborCompatibilityError> {
    if observation.trial_identity != request.binding().run_identity {
        return Err(HarborCompatibilityError::Observation);
    }
    let outcome = match observation.outcome {
        OutcomeClass::Completed => animus_adapter_eval::ObservationClass::Success,
        OutcomeClass::Unavailable => animus_adapter_eval::ObservationClass::Unavailable,
        OutcomeClass::Denied => animus_adapter_eval::ObservationClass::Denied,
        OutcomeClass::Cancelled => animus_adapter_eval::ObservationClass::Cancelled,
        OutcomeClass::TimedOut => animus_adapter_eval::ObservationClass::TimedOut,
        OutcomeClass::Malformed => animus_adapter_eval::ObservationClass::Malformed,
        OutcomeClass::Stale => animus_adapter_eval::ObservationClass::Stale,
    };
    let result_identity = match (outcome, observation.trajectory_identity.as_ref()) {
        (animus_adapter_eval::ObservationClass::Success, Some(identity)) => Some(identity.clone()),
        (animus_adapter_eval::ObservationClass::Success, None) => {
            return Err(HarborCompatibilityError::Observation);
        }
        (_, None) => None,
        (_, Some(_)) => return Err(HarborCompatibilityError::Observation),
    };
    let item_count = observation
        .artifacts
        .len()
        .checked_add(observation.rewards.len())
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(HarborCompatibilityError::Observation)?;
    if item_count > request.requested_items() || observed_bytes > request.requested_bytes() {
        return Err(HarborCompatibilityError::Observation);
    }
    Ok(animus_adapter_eval::ServiceObservation {
        service: request.service(),
        request_identity: request
            .identity()
            .map_err(HarborCompatibilityError::Binding)?,
        binding_identity: animus_adapter_eval::binding_identity(request.binding())
            .map_err(HarborCompatibilityError::Binding)?,
        correlation_identity: request.binding().correlation_identity.into(),
        implementation_identity: request.implementation_identity().into(),
        attempt: request.binding().attempt,
        outcome,
        result_identity,
        item_count,
        byte_count: observed_bytes,
    })
}

#[cfg(feature = "evaluation")]
fn frame_harbor_identity(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

#[cfg(all(test, feature = "evaluation"))]
mod tests {
    use super::*;
    use animus_adapter_eval::{
        EvaluationBinding, ServiceAvailability, ServiceCapability, ServiceCatalog,
        ServiceDescriptor, ServiceKind, ServiceRequirement, ServiceResourceBounds, admit_service,
        validate_service_observation,
    };

    const IMPLEMENTATION: &str = "implementation.harbor.v1";
    const POLICY: &str = "policy.harbor.v1";
    const REQUEST_ITEMS: u32 = 4;
    const REQUEST_BYTES: u64 = 1_024;
    const OBSERVED_BYTES: u64 = 128;

    fn trial() -> TrialRequest {
        TrialRequest {
            trial_identity: "run.harbor".into(),
            task_identity: "task.harbor".into(),
            agent_identity: "role.solver".into(),
            model_route_identity: "model.harbor".into(),
            environment_identity: "environment.solver".into(),
            verifier_identity: "verifier.harbor".into(),
            artifacts: alloc::vec!["artifact.output".into()],
        }
    }

    fn binding(artifact_identity: &str) -> EvaluationBinding<'_> {
        EvaluationBinding {
            evaluation_identity: "evaluation.harbor",
            plan_identity: "plan.harbor",
            task_identity: "task.harbor",
            episode_identity: "episode.harbor",
            run_identity: "run.harbor",
            role_identity: "role.solver",
            model_identity: "model.harbor",
            harness_identity: "harness.harbor",
            environment_identity: "environment.solver",
            solver_environment_identity: "environment.solver",
            tool_identity: "tools.harbor",
            verifier_identity: "verifier.harbor",
            rubric_identity: "rubric.harbor",
            artifact_identity,
            policy_identity: POLICY,
            cancellation_identity: "cancel.harbor",
            deadline_identity: "deadline.harbor",
            attempt: 1,
            attempt_lineage_identity: "attempt.harbor",
            correlation_identity: "correlation.harbor",
        }
    }

    fn admission(kind: ServiceKind) -> animus_adapter_eval::ServiceAdmission {
        let capability = match kind {
            ServiceKind::AgentExecution => ServiceCapability::AgentExecution,
            ServiceKind::VerifierExecution => ServiceCapability::VerifierExecution,
            _ => panic!("test only admits Harbor execution ports"),
        };
        let descriptors = [ServiceDescriptor {
            kind,
            implementation_identity: IMPLEMENTATION,
            capabilities: &[capability],
            availability: ServiceAvailability::Available,
            bounds: ServiceResourceBounds {
                max_items: REQUEST_ITEMS,
                max_bytes: REQUEST_BYTES,
            },
            policy_identity: POLICY,
        }];
        admit_service(
            &ServiceCatalog {
                catalog_identity: "catalog.harbor",
                services: &descriptors,
            },
            &ServiceRequirement {
                kind,
                implementation_identity: Some(IMPLEMENTATION),
                required_items: REQUEST_ITEMS,
                required_bytes: REQUEST_BYTES,
                policy_identity: POLICY,
            },
        )
        .expect("Harbor test service must be admitted")
    }

    #[test]
    fn exact_harbor_environment_request_and_observation_bridge() {
        let trial = trial();
        let artifact_identity = harbor_artifact_manifest_identity(&trial.artifacts);
        let binding = binding(&artifact_identity);
        let admission = admission(ServiceKind::AgentExecution);
        let native = bridge_harbor_request(
            &HarborBridgeRequest {
                trial: &trial,
                service: ServiceKind::AgentExecution,
                binding: &binding,
                requested_items: REQUEST_ITEMS,
                requested_bytes: REQUEST_BYTES,
            },
            &admission,
        )
        .expect("exact Harbor request must bridge");
        let HarborNativeRequest::Agent(agent) = native else {
            panic!("environment conversion must produce agent execution");
        };
        let observed = bridge_harbor_observation(
            agent.service(),
            &TrialObservation {
                trial_identity: trial.trial_identity.clone(),
                outcome: OutcomeClass::Completed,
                trajectory_identity: Some("trajectory.harbor".into()),
                rewards: alloc::vec![],
                artifacts: alloc::vec![],
            },
            OBSERVED_BYTES,
        )
        .expect("bounded Harbor observation must bridge");
        assert!(validate_service_observation(agent.service(), &observed).is_ok());
    }

    #[test]
    fn harbor_bridge_rejects_identity_loss_and_shared_verifier_environment() {
        let mut tampered_trial = trial();
        let artifact_identity = harbor_artifact_manifest_identity(&tampered_trial.artifacts);
        let binding = binding(&artifact_identity);
        let agent_admission = admission(ServiceKind::AgentExecution);
        tampered_trial.task_identity = "task.other".into();
        assert_eq!(
            bridge_harbor_request(
                &HarborBridgeRequest {
                    trial: &tampered_trial,
                    service: ServiceKind::AgentExecution,
                    binding: &binding,
                    requested_items: REQUEST_ITEMS,
                    requested_bytes: REQUEST_BYTES,
                },
                &agent_admission,
            ),
            Err(HarborCompatibilityError::IdentityLoss)
        );

        let trial = trial();
        let verifier_admission = admission(ServiceKind::VerifierExecution);
        assert_eq!(
            bridge_harbor_request(
                &HarborBridgeRequest {
                    trial: &trial,
                    service: ServiceKind::VerifierExecution,
                    binding: &binding,
                    requested_items: REQUEST_ITEMS,
                    requested_bytes: REQUEST_BYTES,
                },
                &verifier_admission,
            ),
            Err(HarborCompatibilityError::Binding(
                animus_adapter_eval::BindingError::Isolation
            ))
        );
    }
}
