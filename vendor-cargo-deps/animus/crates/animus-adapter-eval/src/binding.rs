//! Exact evaluation request binding and observation validation.

use alloc::{format, string::String};

use animus_eval_core::{hash_identity, validate_identity};

use crate::{ServiceAdmission, ServiceKind};

/// Every identity that can materially change an evaluation service call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationBinding<'a> {
    /// Evaluation identity.
    pub evaluation_identity: &'a str,
    /// Native evaluation plan identity.
    pub plan_identity: &'a str,
    /// Task identity.
    pub task_identity: &'a str,
    /// Episode identity.
    pub episode_identity: &'a str,
    /// Run identity.
    pub run_identity: &'a str,
    /// Semantic role identity.
    pub role_identity: &'a str,
    /// Effective model identity.
    pub model_identity: &'a str,
    /// Harness identity.
    pub harness_identity: &'a str,
    /// Target environment identity for this request.
    pub environment_identity: &'a str,
    /// Solver environment identity used to check verifier isolation.
    pub solver_environment_identity: &'a str,
    /// Tool-catalog identity.
    pub tool_identity: &'a str,
    /// Verifier identity.
    pub verifier_identity: &'a str,
    /// Rubric identity.
    pub rubric_identity: &'a str,
    /// Artifact-manifest identity.
    pub artifact_identity: &'a str,
    /// Evaluation policy identity.
    pub policy_identity: &'a str,
    /// Cancellation-scope identity.
    pub cancellation_identity: &'a str,
    /// Deadline-policy identity.
    pub deadline_identity: &'a str,
    /// Positive attempt number.
    pub attempt: u32,
    /// Exact attempt-lineage identity.
    pub attempt_lineage_identity: &'a str,
    /// Per-call correlation identity.
    pub correlation_identity: &'a str,
}

/// One generic service request embedded by each typed port request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceRequest<'a> {
    /// Requested service.
    pub(crate) service: ServiceKind,
    /// Exact admission identity.
    pub(crate) admission_identity: String,
    /// Exact selected implementation identity.
    pub(crate) implementation_identity: String,
    /// Full evaluation binding.
    pub(crate) binding: EvaluationBinding<'a>,
    /// Requested item count.
    pub(crate) requested_items: u32,
    /// Requested byte capacity.
    pub(crate) requested_bytes: u64,
}

impl ServiceRequest<'_> {
    /// Return the requested service kind.
    #[must_use]
    pub const fn service(&self) -> ServiceKind {
        self.service
    }

    /// Return the exact admission identity.
    #[must_use]
    pub fn admission_identity(&self) -> &str {
        &self.admission_identity
    }

    /// Return the exact implementation identity.
    #[must_use]
    pub fn implementation_identity(&self) -> &str {
        &self.implementation_identity
    }

    /// Return the full evaluation binding.
    #[must_use]
    pub const fn binding(&self) -> &EvaluationBinding<'_> {
        &self.binding
    }

    /// Return the requested item capacity.
    #[must_use]
    pub const fn requested_items(&self) -> u32 {
        self.requested_items
    }

    /// Return the requested byte capacity.
    #[must_use]
    pub const fn requested_bytes(&self) -> u64 {
        self.requested_bytes
    }

    /// Return the canonical full request identity.
    ///
    /// # Errors
    /// Returns an identity or bound class for malformed request material.
    pub fn identity(&self) -> Result<String, BindingError> {
        let binding = binding_identity(&self.binding)?;
        validate_identity(&self.admission_identity).map_err(|_| BindingError::Identity)?;
        validate_identity(&self.implementation_identity).map_err(|_| BindingError::Identity)?;
        if self.requested_items == 0 || self.requested_bytes == 0 {
            return Err(BindingError::Bound);
        }
        let items = format!("{}", self.requested_items);
        let bytes = format!("{}", self.requested_bytes);
        Ok(hash_identity(
            "animus.native-evaluation.service-request.v1",
            &[
                self.service.label(),
                &self.admission_identity,
                &self.implementation_identity,
                &binding,
                &items,
                &bytes,
            ],
        ))
    }
}

/// Build a request after checking its exact admission.
///
/// # Errors
/// Returns mismatch when service, implementation, policy, or bounds differ.
pub fn build_service_request<'a>(
    service: ServiceKind,
    admission_identity: &str,
    implementation_identity: &str,
    binding: &EvaluationBinding<'a>,
    requested_items: u32,
    requested_bytes: u64,
    admission: &ServiceAdmission,
) -> Result<ServiceRequest<'a>, BindingError> {
    if service != admission.kind
        || admission_identity != admission.identity
        || implementation_identity != admission.implementation_identity
        || binding.policy_identity != admission.policy_identity
    {
        return Err(BindingError::Mismatch);
    }
    if service == ServiceKind::VerifierExecution
        && binding.environment_identity == binding.solver_environment_identity
    {
        return Err(BindingError::Isolation);
    }
    if service == ServiceKind::AgentExecution
        && binding.environment_identity != binding.solver_environment_identity
    {
        return Err(BindingError::Mismatch);
    }
    if requested_items == 0
        || requested_bytes == 0
        || requested_items > admission.bounds.max_items
        || requested_bytes > admission.bounds.max_bytes
    {
        return Err(BindingError::Bound);
    }
    let request = ServiceRequest {
        service,
        admission_identity: admission_identity.into(),
        implementation_identity: implementation_identity.into(),
        binding: *binding,
        requested_items,
        requested_bytes,
    };
    request.identity()?;
    Ok(request)
}

/// Return the canonical identity of all material bindings.
///
/// # Errors
/// Returns identity or bound for malformed metadata or a zero attempt.
pub fn binding_identity(binding: &EvaluationBinding<'_>) -> Result<String, BindingError> {
    for identity in [
        binding.evaluation_identity,
        binding.plan_identity,
        binding.task_identity,
        binding.episode_identity,
        binding.run_identity,
        binding.role_identity,
        binding.model_identity,
        binding.harness_identity,
        binding.environment_identity,
        binding.solver_environment_identity,
        binding.tool_identity,
        binding.verifier_identity,
        binding.rubric_identity,
        binding.artifact_identity,
        binding.policy_identity,
        binding.cancellation_identity,
        binding.deadline_identity,
        binding.attempt_lineage_identity,
        binding.correlation_identity,
    ] {
        validate_identity(identity).map_err(|_| BindingError::Identity)?;
    }
    if binding.attempt == 0 {
        return Err(BindingError::Bound);
    }
    let attempt = format!("{}", binding.attempt);
    Ok(hash_identity(
        "animus.native-evaluation.binding.v1",
        &[
            binding.evaluation_identity,
            binding.plan_identity,
            binding.task_identity,
            binding.episode_identity,
            binding.run_identity,
            binding.role_identity,
            binding.model_identity,
            binding.harness_identity,
            binding.environment_identity,
            binding.solver_environment_identity,
            binding.tool_identity,
            binding.verifier_identity,
            binding.rubric_identity,
            binding.artifact_identity,
            binding.policy_identity,
            binding.cancellation_identity,
            binding.deadline_identity,
            &attempt,
            binding.attempt_lineage_identity,
            binding.correlation_identity,
        ],
    ))
}

/// Bounded service outcome class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationClass {
    /// Service completed with bounded result evidence.
    Success,
    /// Policy denied the call.
    Denied,
    /// Service was unavailable.
    Unavailable,
    /// Service does not support the request.
    Unsupported,
    /// Cancellation was observed.
    Cancelled,
    /// Deadline was observed.
    TimedOut,
    /// Observation was malformed.
    Malformed,
    /// Observation was stale.
    Stale,
    /// Observation identities did not match.
    Mismatch,
    /// Bounded output overflowed.
    Overflow,
    /// Host infrastructure failed.
    InfrastructureFailure,
}

impl ObservationClass {
    /// Return a safe stable class label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
            Self::Unsupported => "unsupported",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Malformed => "malformed",
            Self::Stale => "stale",
            Self::Mismatch => "mismatch",
            Self::Overflow => "overflow",
            Self::InfrastructureFailure => "infrastructure_failure",
        }
    }
}

/// Bounded host-supplied observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceObservation {
    /// Exact service kind.
    pub service: ServiceKind,
    /// Canonical request identity.
    pub request_identity: String,
    /// Canonical full-binding identity.
    pub binding_identity: String,
    /// Exact correlation identity echoed by the service.
    pub correlation_identity: String,
    /// Exact implementation identity echoed by the service.
    pub implementation_identity: String,
    /// Exact attempt echoed by the service.
    pub attempt: u32,
    /// Outcome class.
    pub outcome: ObservationClass,
    /// Optional bounded result identity. Success requires it.
    pub result_identity: Option<String>,
    /// Observed item count.
    pub item_count: u32,
    /// Observed byte count.
    pub byte_count: u64,
}

/// Validated observation eligible for pure reduction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedObservation {
    /// Canonical observation identity.
    pub identity: String,
    /// Service kind.
    pub service: ServiceKind,
    /// Request identity.
    pub request_identity: String,
    /// Exact implementation identity.
    pub implementation_identity: String,
    /// Outcome class.
    pub outcome: ObservationClass,
    /// Optional bounded result identity.
    pub result_identity: Option<String>,
    /// Observed item count.
    pub item_count: u32,
    /// Observed byte count.
    pub byte_count: u64,
}

/// Pure state-reduction disposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservationDisposition {
    /// Success can enter native evaluation reduction.
    Eligible,
    /// Policy, availability, cancellation, deadline, stale, mismatch, or overflow blocks reduction.
    Blocked,
    /// Malformed or infrastructure failure ends this attempt as failed.
    Failed,
}

/// Binding or observation validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BindingError {
    /// An identity is malformed.
    Identity,
    /// A count or attempt bound is invalid.
    Bound,
    /// Service, plan, task, attempt, policy, correlation, or implementation differs.
    Mismatch,
    /// Required verifier isolation is absent.
    Isolation,
    /// A successful observation lacks result evidence.
    Incomplete,
    /// A failed observation improperly carries success evidence.
    Contradiction,
}

/// Validate one observation against the exact request before state reduction.
///
/// # Errors
/// Returns mismatch, bound, incomplete, or contradiction before any state mutation.
pub fn validate_service_observation(
    request: &ServiceRequest<'_>,
    observation: &ServiceObservation,
) -> Result<ValidatedObservation, BindingError> {
    let request_identity = request.identity()?;
    let exact_binding = binding_identity(&request.binding)?;
    for identity in [
        observation.request_identity.as_str(),
        observation.binding_identity.as_str(),
        observation.correlation_identity.as_str(),
        observation.implementation_identity.as_str(),
    ] {
        validate_identity(identity).map_err(|_| BindingError::Identity)?;
    }
    if observation.service != request.service
        || observation.request_identity != request_identity
        || observation.binding_identity != exact_binding
        || observation.correlation_identity != request.binding.correlation_identity
        || observation.implementation_identity != request.implementation_identity
        || observation.attempt != request.binding.attempt
    {
        return Err(BindingError::Mismatch);
    }
    if observation.item_count > request.requested_items
        || observation.byte_count > request.requested_bytes
    {
        return Err(BindingError::Bound);
    }
    match (observation.outcome, observation.result_identity.as_deref()) {
        (ObservationClass::Success, None) => return Err(BindingError::Incomplete),
        (ObservationClass::Success, Some(identity)) => {
            validate_identity(identity).map_err(|_| BindingError::Identity)?;
        }
        (_, Some(_)) => return Err(BindingError::Contradiction),
        (_, None) => {}
    }
    let items = format!("{}", observation.item_count);
    let bytes = format!("{}", observation.byte_count);
    let identity = hash_identity(
        "animus.native-evaluation.service-observation.v1",
        &[
            observation.service.label(),
            &observation.request_identity,
            &observation.binding_identity,
            &observation.correlation_identity,
            &observation.implementation_identity,
            observation.outcome.label(),
            observation.result_identity.as_deref().unwrap_or("none"),
            &items,
            &bytes,
        ],
    );
    Ok(ValidatedObservation {
        identity,
        service: observation.service,
        request_identity,
        implementation_identity: observation.implementation_identity.clone(),
        outcome: observation.outcome,
        result_identity: observation.result_identity.clone(),
        item_count: observation.item_count,
        byte_count: observation.byte_count,
    })
}

/// Reduce an already-validated observation to its explicit disposition.
#[must_use]
pub const fn reduce_observation(observation: &ValidatedObservation) -> ObservationDisposition {
    match observation.outcome {
        ObservationClass::Success => ObservationDisposition::Eligible,
        ObservationClass::Malformed | ObservationClass::InfrastructureFailure => {
            ObservationDisposition::Failed
        }
        ObservationClass::Denied
        | ObservationClass::Unavailable
        | ObservationClass::Unsupported
        | ObservationClass::Cancelled
        | ObservationClass::TimedOut
        | ObservationClass::Stale
        | ObservationClass::Mismatch
        | ObservationClass::Overflow => ObservationDisposition::Blocked,
    }
}
