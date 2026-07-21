//! Pure provider failure classification, exact-plan retry planning, and timer gating.

const BLAKE3_HEX_CHARS: usize = super::data::BLAKE3_HEX_CHARS;
const HASH_SEPARATOR: &str = super::data::HASH_SEPARATOR;
const MAX_ATTEMPT_COUNT: u32 = super::data::MAX_ATTEMPT_COUNT;
const MAX_DELAY_MS: u64 = super::data::MAX_DELAY_MS;
const MAX_MAPPING_COUNT: usize = super::data::MAX_MAPPING_COUNT;
const MAX_SAFE_LABEL_CHARS: usize = super::data::MAX_SAFE_LABEL_CHARS;
const MIN_ATTEMPT_COUNT: u32 = super::data::MIN_ATTEMPT_COUNT;
const MAPPING_HASH_PART_COUNT: usize = 5;
const PROVIDER_RETRY_DECISION_HASH_DOMAIN: &str = super::data::PROVIDER_RETRY_DECISION_HASH_DOMAIN;
const PROVIDER_RETRY_INVOCATION_HASH_DOMAIN: &str =
    super::data::PROVIDER_RETRY_INVOCATION_HASH_DOMAIN;
const PROVIDER_RETRY_POLICY_HASH_DOMAIN: &str = super::data::PROVIDER_RETRY_POLICY_HASH_DOMAIN;
const PROVIDER_RETRY_POLICY_SCHEMA: &str = super::data::PROVIDER_RETRY_POLICY_SCHEMA;

type ProviderRetryAttemptState = super::data::ProviderRetryAttemptState;
type ProviderRetryCancellation = super::data::ProviderRetryCancellation;
type ProviderRetryDecision = super::data::ProviderRetryDecision;
type ProviderRetryDecisionStatus = super::data::ProviderRetryDecisionStatus;
type ProviderRetryDispatchGate = super::data::ProviderRetryDispatchGate;
type ProviderRetryDispatchGateStatus = super::data::ProviderRetryDispatchGateStatus;
type ProviderRetryDispatchRequest = super::data::ProviderRetryDispatchRequest;
type ProviderRetryError = super::data::ProviderRetryError;
type ProviderRetryFailureClass = super::data::ProviderRetryFailureClass;
type ProviderRetryFailureObservation = super::data::ProviderRetryFailureObservation;
type ProviderRetryInvocation = super::data::ProviderRetryInvocation;
type ProviderRetryInvocationInput = super::data::ProviderRetryInvocationInput;
type ProviderRetryIssue = super::data::ProviderRetryIssue;
type ProviderRetryMapping = super::data::ProviderRetryMapping;
type ProviderRetryMappingInput = super::data::ProviderRetryMappingInput;
type ProviderRetryPolicy = super::data::ProviderRetryPolicy;
type ProviderRetryPolicyInput = super::data::ProviderRetryPolicyInput;
type ProviderRetryPlanInput<'a> = super::data::ProviderRetryPlanInput<'a>;
type ProviderRetryProgressEvent = super::data::ProviderRetryProgressEvent;
type ProviderRetryProgressStatus = super::data::ProviderRetryProgressStatus;
type ProviderRetrySemanticFrame = super::data::ProviderRetrySemanticFrame;
type ProviderRetryStreamObservation = super::data::ProviderRetryStreamObservation;
type ProviderRetryStreamState = super::data::ProviderRetryStreamState;
type ProviderRetryTimerEvaluation = super::data::ProviderRetryTimerEvaluation;
type ProviderRetryTimerObservation = super::data::ProviderRetryTimerObservation;
type ProviderRetryTimerOutcome = super::data::ProviderRetryTimerOutcome;
type ProviderRetryTimerRequest = super::data::ProviderRetryTimerRequest;
type ProviderRetryWaitPlan = super::data::ProviderRetryWaitPlan;
type ProviderRetryWaitState = super::data::ProviderRetryWaitState;
type ProviderRetryWaitStatus = super::data::ProviderRetryWaitStatus;

#[derive(Clone, Copy)]
pub(super) struct RetryField(pub(super) &'static str);

const FIELD_ATTEMPT: RetryField = RetryField("provider_retry_attempt");
const FIELD_CLASS: RetryField = RetryField("provider_retry_class");
const FIELD_DELAY: RetryField = RetryField("provider_retry_delay_ms");
const FIELD_EVENT: RetryField = RetryField("provider_retry_event");
const FIELD_IDENTITY: RetryField = RetryField("provider_retry_invocation");
const FIELD_MAPPING: RetryField = RetryField("provider_retry_mapping");
const FIELD_OBSERVATION: RetryField = RetryField("provider_retry_observation");
const FIELD_POLICY: RetryField = RetryField("provider_retry_policy");
const FIELD_TIMER: RetryField = RetryField("provider_retry_timer");

/// Revalidates a generated provider retry policy and canonical content identity.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] for stale schemas or hashes, malformed labels,
/// duplicate mappings/classes, or invalid attempt and delay bounds.
// r[impl agent_harness_runtime.tau_provider_retry.policy]
pub fn validate_provider_retry_policy(
    input: &ProviderRetryPolicyInput,
) -> Result<ProviderRetryPolicy, ProviderRetryError> {
    validate_policy_shape(input)?;
    let retryable_classes = validate_retryable_classes(input.retryable_classes.as_slice())?;
    let mappings = validate_mappings(input.mappings.as_slice())?;
    let policy = ProviderRetryPolicy {
        policy_id: input.policy_id.clone(),
        max_attempts: input.max_attempts,
        max_delay_ms: input.max_delay_ms,
        delay_schedule_ms: input.delay_schedule_ms.clone(),
        retryable_classes,
        mappings,
        policy_hash: input.policy_hash.clone(),
    };
    let expected_hash = provider_retry_policy_content_hash(&policy);
    if policy.policy_hash != expected_hash {
        return Err(error(ProviderRetryIssue::StalePolicy, FIELD_POLICY));
    }
    debug_assert_eq!(
        policy.delay_schedule_ms.len().saturating_add(1),
        policy.max_attempts as usize
    );
    debug_assert_eq!(policy.policy_hash.len(), BLAKE3_HEX_CHARS);
    Ok(policy)
}

/// Returns the canonical BLAKE3 content identity for a validated policy.
#[must_use]
pub fn provider_retry_policy_content_hash(policy: &ProviderRetryPolicy) -> String {
    let mut parts = Vec::with_capacity(policy_hash_part_capacity(policy));
    parts.extend([
        PROVIDER_RETRY_POLICY_SCHEMA.to_string(),
        policy.policy_id.clone(),
        policy.max_attempts.to_string(),
        policy.max_delay_ms.to_string(),
        "required".to_string(),
        "deny_after_semantic".to_string(),
    ]);
    parts.extend(
        policy
            .delay_schedule_ms
            .iter()
            .map(|delay_ms| format!("delay_ms:{delay_ms}")),
    );
    parts.extend(
        policy
            .retryable_classes
            .iter()
            .map(|class| format!("retryable:{}", class.as_str())),
    );
    for mapping in &policy.mappings {
        parts.extend([
            "mapping".to_string(),
            mapping.provider_family.clone(),
            mapping.mapping_version.clone(),
            mapping.native_class.clone(),
            mapping.canonical_class.as_str().to_string(),
        ]);
    }
    let hash = hash_owned_parts(PROVIDER_RETRY_POLICY_HASH_DOMAIN, parts.as_slice());
    debug_assert_eq!(parts.len(), policy_hash_part_capacity(policy));
    debug_assert_eq!(hash.len(), BLAKE3_HEX_CHARS);
    hash
}

/// Revalidates every exact invocation dimension and its canonical identity.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] when labels or hashes are malformed, policy
/// identity differs, or the exact plan hash is stale.
// r[impl agent_harness_runtime.tau_provider_retry.same_target]
pub fn validate_provider_retry_invocation(
    input: &ProviderRetryInvocationInput,
    policy: &ProviderRetryPolicy,
) -> Result<ProviderRetryInvocation, ProviderRetryError> {
    safe_label(input.provider_id.as_str(), FIELD_IDENTITY)?;
    safe_label(input.model_id.as_str(), FIELD_IDENTITY)?;
    for hash in invocation_hashes(input) {
        lowercase_blake3(hash, FIELD_IDENTITY)?;
    }
    if input.policy_hash != policy.policy_hash {
        return Err(error(ProviderRetryIssue::StaleInvocation, FIELD_IDENTITY));
    }
    let invocation = invocation_from_input(input);
    let expected_hash = provider_retry_invocation_content_hash(&invocation);
    if invocation.exact_plan_hash != expected_hash {
        return Err(error(ProviderRetryIssue::StaleInvocation, FIELD_IDENTITY));
    }
    debug_assert_eq!(invocation.policy_hash, policy.policy_hash);
    debug_assert_eq!(invocation.exact_plan_hash.len(), BLAKE3_HEX_CHARS);
    Ok(invocation)
}

/// Returns the canonical BLAKE3 identity for unvalidated invocation dimensions.
#[must_use]
pub fn provider_retry_invocation_input_content_hash(
    input: &ProviderRetryInvocationInput,
) -> String {
    let parts = [
        input.provider_id.clone(),
        input.model_id.clone(),
        input.request_hash.clone(),
        input.profile_plan_hash.clone(),
        input.prompt_hash.clone(),
        input.tool_catalog_hash.clone(),
        input.response_format_hash.clone(),
        input.reasoning_intent_hash.clone(),
        input.stream_intent_hash.clone(),
        input.credential_handle_hash.clone(),
        input.policy_hash.clone(),
    ];
    let hash = hash_owned_parts(PROVIDER_RETRY_INVOCATION_HASH_DOMAIN, &parts);
    debug_assert_eq!(parts.len(), invocation_hash_part_count());
    debug_assert_eq!(hash.len(), BLAKE3_HEX_CHARS);
    hash
}

/// Returns the canonical BLAKE3 identity for exact invocation dimensions.
#[must_use]
pub fn provider_retry_invocation_content_hash(invocation: &ProviderRetryInvocation) -> String {
    let parts = [
        invocation.provider_id.clone(),
        invocation.model_id.clone(),
        invocation.request_hash.clone(),
        invocation.profile_plan_hash.clone(),
        invocation.prompt_hash.clone(),
        invocation.tool_catalog_hash.clone(),
        invocation.response_format_hash.clone(),
        invocation.reasoning_intent_hash.clone(),
        invocation.stream_intent_hash.clone(),
        invocation.credential_handle_hash.clone(),
        invocation.policy_hash.clone(),
    ];
    let hash = hash_owned_parts(PROVIDER_RETRY_INVOCATION_HASH_DOMAIN, &parts);
    debug_assert_eq!(parts.len(), invocation_hash_part_count());
    debug_assert_eq!(hash.len(), BLAKE3_HEX_CHARS);
    hash
}

/// Maps a safe provider-native observation through the selected policy.
///
/// Unknown mappings return [`ProviderRetryFailureClass::Unknown`] and are
/// terminal by default.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] when observation labels are malformed.
// r[impl agent_harness_runtime.tau_provider_retry.classification]
pub fn classify_provider_retry_failure(
    policy: &ProviderRetryPolicy,
    observation: &ProviderRetryFailureObservation,
) -> Result<ProviderRetryFailureClass, ProviderRetryError> {
    safe_label(observation.provider_family.as_str(), FIELD_OBSERVATION)?;
    safe_label(observation.mapping_version.as_str(), FIELD_OBSERVATION)?;
    safe_label(observation.native_class.as_str(), FIELD_OBSERVATION)?;
    let class = policy
        .mappings
        .iter()
        .find(|mapping| mapping_matches(mapping, observation))
        .map_or(ProviderRetryFailureClass::Unknown, |mapping| {
            mapping.canonical_class
        });
    debug_assert!(ProviderRetryFailureClass::parse(class.as_str()).is_some());
    Ok(class)
}

/// Reduces one host stream observation into replay-safety state.
#[must_use]
// r[impl agent_harness_runtime.tau_provider_retry.stream_safety]
pub const fn observe_provider_retry_stream(
    state: ProviderRetryStreamState,
    observation: ProviderRetryStreamObservation,
) -> ProviderRetryStreamState {
    match (state, observation) {
        (ProviderRetryStreamState::Committed(frame), _other) => {
            ProviderRetryStreamState::Committed(frame)
        }
        (
            ProviderRetryStreamState::NotStarted,
            ProviderRetryStreamObservation::TransportStarted,
        ) => ProviderRetryStreamState::TransportStarted,
        (current, ProviderRetryStreamObservation::TransportStarted) => current,
        (_current, ProviderRetryStreamObservation::Semantic(frame)) => {
            ProviderRetryStreamState::Committed(frame)
        }
    }
}

/// Maps a canonical model-stream frame into retry replay-safety state.
#[must_use]
pub const fn observe_provider_retry_model_frame(
    state: ProviderRetryStreamState,
    frame: animus_adapters::ModelStreamFrameKind,
) -> ProviderRetryStreamState {
    let semantic = semantic_frame_from_model_frame(frame);
    match semantic {
        Some(frame) => ProviderRetryStreamState::Committed(frame),
        None => state,
    }
}

/// Plans retry, terminal stop, exhaustion, cancellation, or recomposition.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] when attempt state or safe observations are
/// malformed.
// r[impl agent_harness_runtime.tau_provider_retry.backoff]
// r[impl agent_harness_runtime.tau_provider_retry.same_target]
// r[impl agent_harness_runtime.tau_provider_retry.stream_safety]
pub fn plan_provider_retry(
    input: ProviderRetryPlanInput<'_>,
) -> Result<ProviderRetryDecision, ProviderRetryError> {
    validate_attempt_state(input.policy, input.current, input.state)?;
    let reason = classify_provider_retry_failure(input.policy, input.observation)?;
    let status = select_decision_status(&input, reason);
    let (next_attempt_count, delay_ms) = retry_schedule(input.policy, input.state, status)?;
    let hash_input = DecisionHashInput {
        status,
        reason,
        state: input.state,
        policy: input.policy,
        next_attempt_count,
        delay_ms,
    };
    let decision_hash = decision_hash(&hash_input);
    let decision = ProviderRetryDecision {
        status,
        reason,
        run_id: input.state.run_id.clone(),
        completed_attempt_count: input.state.completed_attempt_count,
        next_attempt_count,
        max_attempt_count: input.policy.max_attempts,
        delay_ms,
        exact_plan_hash: input.current.exact_plan_hash.clone(),
        policy_hash: input.policy.policy_hash.clone(),
        decision_hash,
    };
    debug_assert_eq!(
        decision.status == ProviderRetryDecisionStatus::Retry,
        delay_ms.is_some()
    );
    debug_assert_eq!(decision.next_attempt_count.is_some(), delay_ms.is_some());
    Ok(decision)
}

/// Projects an admitted retry decision into an injected timer request and
/// exact threaded wait state.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] when the decision does not admit retry.
// r[impl agent_harness_runtime.tau_provider_retry.backoff]
pub fn plan_provider_retry_wait(
    decision: &ProviderRetryDecision,
) -> Result<ProviderRetryWaitPlan, ProviderRetryError> {
    if decision.status != ProviderRetryDecisionStatus::Retry {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_TIMER));
    }
    let next_attempt_count = decision
        .next_attempt_count
        .ok_or_else(|| error(ProviderRetryIssue::InvalidInput, FIELD_TIMER))?;
    let delay_ms = decision
        .delay_ms
        .ok_or_else(|| error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY))?;
    let request = ProviderRetryTimerRequest {
        run_id: decision.run_id.clone(),
        next_attempt_count,
        delay_ms,
        exact_plan_hash: decision.exact_plan_hash.clone(),
        decision_hash: decision.decision_hash.clone(),
        cancellation_required: true,
    };
    let state = ProviderRetryWaitState {
        run_id: request.run_id.clone(),
        next_attempt_count: request.next_attempt_count,
        exact_plan_hash: request.exact_plan_hash.clone(),
        decision_hash: request.decision_hash.clone(),
        status: ProviderRetryWaitStatus::Waiting,
    };
    debug_assert!(request.delay_ms <= MAX_DELAY_MS);
    debug_assert!(request.cancellation_required);
    Ok(ProviderRetryWaitPlan { request, state })
}

/// Reduces one correlated timer observation through exact threaded wait state.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] for malformed timer evidence or stale wait
/// state. A closed state can never produce another ready dispatch gate.
// r[impl agent_harness_runtime.tau_provider_retry.backoff]
pub fn evaluate_provider_retry_timer(
    decision: &ProviderRetryDecision,
    state: &ProviderRetryWaitState,
    observation: &ProviderRetryTimerObservation,
) -> Result<ProviderRetryTimerEvaluation, ProviderRetryError> {
    safe_label(observation.run_id.as_str(), FIELD_TIMER)?;
    lowercase_blake3(observation.exact_plan_hash.as_str(), FIELD_TIMER)?;
    lowercase_blake3(observation.decision_hash.as_str(), FIELD_TIMER)?;
    validate_wait_state(decision, state)?;
    let is_correlated = timer_observation_matches(decision, state, observation);
    let (gate_status, wait_status) = reduce_wait_status(state, observation, is_correlated);
    let request = ready_dispatch_request(decision, observation, gate_status);
    let gate = ProviderRetryDispatchGate {
        status: gate_status,
        request,
    };
    let next_state = ProviderRetryWaitState {
        run_id: state.run_id.clone(),
        next_attempt_count: state.next_attempt_count,
        exact_plan_hash: state.exact_plan_hash.clone(),
        decision_hash: state.decision_hash.clone(),
        status: wait_status,
    };
    debug_assert_eq!(
        gate.status == ProviderRetryDispatchGateStatus::Ready,
        gate.request.is_some()
    );
    debug_assert_eq!(
        next_state.status == ProviderRetryWaitStatus::Waiting,
        !is_correlated && state.status == ProviderRetryWaitStatus::Waiting
    );
    Ok(ProviderRetryTimerEvaluation {
        gate,
        state: next_state,
    })
}

/// Projects a provider-neutral progress event from a retry decision.
#[must_use]
// r[impl agent_harness_runtime.tau_provider_retry.events]
pub fn project_provider_retry_decision_event(
    decision: &ProviderRetryDecision,
) -> ProviderRetryProgressEvent {
    ProviderRetryProgressEvent {
        run_id: decision.run_id.clone(),
        attempt_count: decision
            .next_attempt_count
            .unwrap_or(decision.completed_attempt_count),
        max_attempt_count: decision.max_attempt_count,
        delay_ms: decision.delay_ms,
        reason: decision.reason,
        status: progress_status_for_decision(decision.status),
        exact_plan_hash: decision.exact_plan_hash.clone(),
    }
}

/// Validates retry progress restored from a serialization boundary.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] for malformed identities or out-of-bounds
/// attempt and delay metadata.
pub fn validate_provider_retry_progress_event(
    event: ProviderRetryProgressEvent,
) -> Result<ProviderRetryProgressEvent, ProviderRetryError> {
    safe_label(event.run_id.as_str(), FIELD_EVENT)?;
    lowercase_blake3(event.exact_plan_hash.as_str(), FIELD_EVENT)?;
    let is_maximum_attempt_count_invalid =
        event.max_attempt_count < MIN_ATTEMPT_COUNT || event.max_attempt_count > MAX_ATTEMPT_COUNT;
    let is_attempt_count_invalid =
        event.attempt_count < MIN_ATTEMPT_COUNT || event.attempt_count > event.max_attempt_count;
    let is_backoff_above_bound = match event.delay_ms {
        Some(delay_ms) => delay_ms > MAX_DELAY_MS,
        None => false,
    };
    if is_maximum_attempt_count_invalid || is_attempt_count_invalid {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_EVENT));
    }
    if is_backoff_above_bound {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_EVENT));
    }
    debug_assert!(!event.run_id.is_empty());
    debug_assert!(event.attempt_count <= event.max_attempt_count);
    Ok(event)
}

/// Advances attempt state exactly once for an admitted dispatch request.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] when correlation is stale or the requested
/// attempt is not the immediate successor of the threaded state.
pub fn advance_provider_retry_attempt(
    state: &ProviderRetryAttemptState,
    request: &ProviderRetryDispatchRequest,
) -> Result<ProviderRetryAttemptState, ProviderRetryError> {
    safe_label(state.run_id.as_str(), FIELD_ATTEMPT)?;
    safe_label(request.run_id.as_str(), FIELD_ATTEMPT)?;
    lowercase_blake3(state.exact_plan_hash.as_str(), FIELD_ATTEMPT)?;
    lowercase_blake3(request.exact_plan_hash.as_str(), FIELD_ATTEMPT)?;
    lowercase_blake3(request.decision_hash.as_str(), FIELD_ATTEMPT)?;
    let expected_attempt_count = state
        .completed_attempt_count
        .checked_add(MIN_ATTEMPT_COUNT)
        .ok_or_else(|| error(ProviderRetryIssue::InvalidSchedule, FIELD_ATTEMPT))?;
    let is_correlated =
        request.run_id == state.run_id && request.exact_plan_hash == state.exact_plan_hash;
    if !is_correlated || request.attempt_count != expected_attempt_count {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_ATTEMPT));
    }
    let next = ProviderRetryAttemptState {
        run_id: state.run_id.clone(),
        completed_attempt_count: request.attempt_count,
        exact_plan_hash: state.exact_plan_hash.clone(),
        stream_state: ProviderRetryStreamState::NotStarted,
        cancellation: ProviderRetryCancellation::None,
    };
    debug_assert!(next.completed_attempt_count > state.completed_attempt_count);
    debug_assert_eq!(next.exact_plan_hash, state.exact_plan_hash);
    Ok(next)
}

/// Projects a provider-neutral progress event from a timer gate.
#[must_use]
pub fn project_provider_retry_timer_event(
    decision: &ProviderRetryDecision,
    gate: &ProviderRetryDispatchGate,
) -> ProviderRetryProgressEvent {
    ProviderRetryProgressEvent {
        run_id: decision.run_id.clone(),
        attempt_count: decision
            .next_attempt_count
            .unwrap_or(decision.completed_attempt_count),
        max_attempt_count: decision.max_attempt_count,
        delay_ms: decision.delay_ms,
        reason: decision.reason,
        status: progress_status_for_gate(gate.status),
        exact_plan_hash: decision.exact_plan_hash.clone(),
    }
}

struct DecisionHashInput<'a> {
    status: ProviderRetryDecisionStatus,
    reason: ProviderRetryFailureClass,
    state: &'a ProviderRetryAttemptState,
    policy: &'a ProviderRetryPolicy,
    next_attempt_count: Option<u32>,
    delay_ms: Option<u64>,
}

fn validate_policy_shape(input: &ProviderRetryPolicyInput) -> Result<(), ProviderRetryError> {
    if input.schema != PROVIDER_RETRY_POLICY_SCHEMA
        || input.cancellation_posture != "required"
        || input.stream_commit_policy != "deny_after_semantic"
    {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_POLICY));
    }
    safe_label(input.policy_id.as_str(), FIELD_POLICY)?;
    lowercase_blake3(input.policy_hash.as_str(), FIELD_POLICY)?;
    validate_schedule(input)?;
    debug_assert!(!input.policy_id.is_empty());
    debug_assert_eq!(input.schema, PROVIDER_RETRY_POLICY_SCHEMA);
    Ok(())
}

fn validate_schedule(input: &ProviderRetryPolicyInput) -> Result<(), ProviderRetryError> {
    if input.max_attempts < MIN_ATTEMPT_COUNT || input.max_attempts > MAX_ATTEMPT_COUNT {
        return Err(error(ProviderRetryIssue::InvalidSchedule, FIELD_ATTEMPT));
    }
    if input.max_delay_ms > MAX_DELAY_MS {
        return Err(error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY));
    }
    let retry_count = input
        .max_attempts
        .checked_sub(MIN_ATTEMPT_COUNT)
        .ok_or_else(|| error(ProviderRetryIssue::InvalidSchedule, FIELD_ATTEMPT))?;
    let expected_retry_count = usize::try_from(retry_count)
        .map_err(|_error| error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY))?;
    if input.delay_schedule_ms.len() != expected_retry_count {
        return Err(error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY));
    }
    if !input.delay_schedule_ms.is_sorted() {
        return Err(error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY));
    }
    if input
        .delay_schedule_ms
        .iter()
        .any(|delay_ms| *delay_ms > input.max_delay_ms)
    {
        return Err(error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY));
    }
    debug_assert_eq!(input.delay_schedule_ms.len(), expected_retry_count);
    debug_assert!(
        input
            .delay_schedule_ms
            .iter()
            .all(|delay_ms| *delay_ms <= MAX_DELAY_MS)
    );
    Ok(())
}

fn validate_retryable_classes(
    labels: &[String],
) -> Result<Vec<ProviderRetryFailureClass>, ProviderRetryError> {
    if labels.is_empty() || labels.len() > MAX_MAPPING_COUNT {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_CLASS));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut classes = Vec::with_capacity(labels.len());
    for label in labels {
        let class = ProviderRetryFailureClass::parse(label.as_str())
            .filter(|value| is_admissible_retryable_class(*value))
            .ok_or_else(|| error(ProviderRetryIssue::InvalidInput, FIELD_CLASS))?;
        if !seen.insert(class) {
            return Err(error(ProviderRetryIssue::InvalidInput, FIELD_CLASS));
        }
        classes.push(class);
    }
    debug_assert_eq!(seen.len(), classes.len());
    debug_assert!(!classes.is_empty());
    Ok(classes)
}

fn validate_mappings(
    inputs: &[ProviderRetryMappingInput],
) -> Result<Vec<ProviderRetryMapping>, ProviderRetryError> {
    if inputs.is_empty() || inputs.len() > MAX_MAPPING_COUNT {
        return Err(error(ProviderRetryIssue::InvalidMapping, FIELD_MAPPING));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut mappings = Vec::with_capacity(inputs.len());
    for input in inputs {
        safe_label(input.provider_family.as_str(), FIELD_MAPPING)?;
        safe_label(input.mapping_version.as_str(), FIELD_MAPPING)?;
        safe_label(input.native_class.as_str(), FIELD_MAPPING)?;
        let canonical_class = ProviderRetryFailureClass::parse(input.canonical_class.as_str())
            .ok_or_else(|| error(ProviderRetryIssue::InvalidMapping, FIELD_MAPPING))?;
        let key = format!(
            "{}:{}:{}",
            input.provider_family, input.mapping_version, input.native_class
        );
        if !seen.insert(key) {
            return Err(error(ProviderRetryIssue::InvalidMapping, FIELD_MAPPING));
        }
        mappings.push(ProviderRetryMapping {
            provider_family: input.provider_family.clone(),
            mapping_version: input.mapping_version.clone(),
            native_class: input.native_class.clone(),
            canonical_class,
        });
    }
    debug_assert_eq!(seen.len(), mappings.len());
    debug_assert!(mappings.len() <= MAX_MAPPING_COUNT);
    Ok(mappings)
}

const fn is_admissible_retryable_class(class: ProviderRetryFailureClass) -> bool {
    matches!(
        class,
        ProviderRetryFailureClass::RateLimited
            | ProviderRetryFailureClass::TransientTransport
            | ProviderRetryFailureClass::ServiceUnavailable
            | ProviderRetryFailureClass::RequestTimeout
            | ProviderRetryFailureClass::Conflict
    )
}

fn invocation_hashes(input: &ProviderRetryInvocationInput) -> [&str; 10] {
    [
        input.request_hash.as_str(),
        input.profile_plan_hash.as_str(),
        input.prompt_hash.as_str(),
        input.tool_catalog_hash.as_str(),
        input.response_format_hash.as_str(),
        input.reasoning_intent_hash.as_str(),
        input.stream_intent_hash.as_str(),
        input.credential_handle_hash.as_str(),
        input.policy_hash.as_str(),
        input.exact_plan_hash.as_str(),
    ]
}

fn invocation_from_input(input: &ProviderRetryInvocationInput) -> ProviderRetryInvocation {
    ProviderRetryInvocation {
        provider_id: input.provider_id.clone(),
        model_id: input.model_id.clone(),
        request_hash: input.request_hash.clone(),
        profile_plan_hash: input.profile_plan_hash.clone(),
        prompt_hash: input.prompt_hash.clone(),
        tool_catalog_hash: input.tool_catalog_hash.clone(),
        response_format_hash: input.response_format_hash.clone(),
        reasoning_intent_hash: input.reasoning_intent_hash.clone(),
        stream_intent_hash: input.stream_intent_hash.clone(),
        credential_handle_hash: input.credential_handle_hash.clone(),
        policy_hash: input.policy_hash.clone(),
        exact_plan_hash: input.exact_plan_hash.clone(),
    }
}

const fn invocation_hash_part_count() -> usize {
    11
}

fn policy_hash_part_capacity(policy: &ProviderRetryPolicy) -> usize {
    let fixed_part_count = 6_usize;
    fixed_part_count
        .saturating_add(policy.delay_schedule_ms.len())
        .saturating_add(policy.retryable_classes.len())
        .saturating_add(
            policy
                .mappings
                .len()
                .saturating_mul(MAPPING_HASH_PART_COUNT),
        )
}

fn mapping_matches(
    mapping: &ProviderRetryMapping,
    observation: &ProviderRetryFailureObservation,
) -> bool {
    mapping.provider_family == observation.provider_family
        && mapping.mapping_version == observation.mapping_version
        && mapping.native_class == observation.native_class
}

fn validate_attempt_state(
    policy: &ProviderRetryPolicy,
    current: &ProviderRetryInvocation,
    state: &ProviderRetryAttemptState,
) -> Result<(), ProviderRetryError> {
    safe_label(state.run_id.as_str(), FIELD_ATTEMPT)?;
    lowercase_blake3(state.exact_plan_hash.as_str(), FIELD_ATTEMPT)?;
    if state.completed_attempt_count < MIN_ATTEMPT_COUNT
        || state.completed_attempt_count > policy.max_attempts
        || state.exact_plan_hash != current.exact_plan_hash
    {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_ATTEMPT));
    }
    debug_assert!(state.completed_attempt_count >= MIN_ATTEMPT_COUNT);
    debug_assert!(state.completed_attempt_count <= policy.max_attempts);
    Ok(())
}

fn select_decision_status(
    input: &ProviderRetryPlanInput<'_>,
    reason: ProviderRetryFailureClass,
) -> ProviderRetryDecisionStatus {
    debug_assert_eq!(input.current.policy_hash, input.policy.policy_hash);
    debug_assert_eq!(input.state.exact_plan_hash, input.current.exact_plan_hash);
    if input.state.cancellation == ProviderRetryCancellation::Requested {
        return ProviderRetryDecisionStatus::Cancelled;
    }
    if !input.policy.retryable_classes.contains(&reason) {
        return ProviderRetryDecisionStatus::Terminal;
    }
    if input.current != input.next {
        return ProviderRetryDecisionStatus::RecompositionRequired;
    }
    if matches!(
        input.state.stream_state,
        ProviderRetryStreamState::Committed(_)
    ) {
        return ProviderRetryDecisionStatus::PostCommitTerminal;
    }
    if input.state.completed_attempt_count >= input.policy.max_attempts {
        return ProviderRetryDecisionStatus::Exhausted;
    }
    ProviderRetryDecisionStatus::Retry
}

fn retry_schedule(
    policy: &ProviderRetryPolicy,
    state: &ProviderRetryAttemptState,
    status: ProviderRetryDecisionStatus,
) -> Result<(Option<u32>, Option<u64>), ProviderRetryError> {
    if status != ProviderRetryDecisionStatus::Retry {
        return Ok((None, None));
    }
    let retry_cursor = state
        .completed_attempt_count
        .checked_sub(MIN_ATTEMPT_COUNT)
        .ok_or_else(|| error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY))?;
    let retry_index = usize::try_from(retry_cursor)
        .map_err(|_error| error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY))?;
    let delay_ms = policy
        .delay_schedule_ms
        .get(retry_index)
        .copied()
        .ok_or_else(|| error(ProviderRetryIssue::InvalidSchedule, FIELD_DELAY))?;
    let next_attempt_count = state
        .completed_attempt_count
        .checked_add(MIN_ATTEMPT_COUNT)
        .ok_or_else(|| error(ProviderRetryIssue::InvalidSchedule, FIELD_ATTEMPT))?;
    debug_assert!(next_attempt_count <= policy.max_attempts);
    debug_assert!(delay_ms <= policy.max_delay_ms);
    Ok((Some(next_attempt_count), Some(delay_ms)))
}

fn decision_hash(input: &DecisionHashInput<'_>) -> String {
    let parts = [
        input.status.as_str().to_string(),
        input.reason.as_str().to_string(),
        input.state.run_id.clone(),
        input.state.completed_attempt_count.to_string(),
        input
            .next_attempt_count
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        input
            .delay_ms
            .map_or_else(|| "none".to_string(), |value| value.to_string()),
        input.state.exact_plan_hash.clone(),
        input.policy.policy_hash.clone(),
    ];
    let hash = hash_owned_parts(PROVIDER_RETRY_DECISION_HASH_DOMAIN, &parts);
    debug_assert_eq!(parts.len(), decision_hash_part_count());
    debug_assert_eq!(hash.len(), BLAKE3_HEX_CHARS);
    hash
}

const fn decision_hash_part_count() -> usize {
    8
}

fn validate_wait_state(
    decision: &ProviderRetryDecision,
    state: &ProviderRetryWaitState,
) -> Result<(), ProviderRetryError> {
    safe_label(state.run_id.as_str(), FIELD_TIMER)?;
    lowercase_blake3(state.exact_plan_hash.as_str(), FIELD_TIMER)?;
    lowercase_blake3(state.decision_hash.as_str(), FIELD_TIMER)?;
    if decision.status != ProviderRetryDecisionStatus::Retry {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_TIMER));
    }
    if state.run_id != decision.run_id {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_TIMER));
    }
    if Some(state.next_attempt_count) != decision.next_attempt_count {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_TIMER));
    }
    if state.exact_plan_hash != decision.exact_plan_hash {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_TIMER));
    }
    if state.decision_hash != decision.decision_hash {
        return Err(error(ProviderRetryIssue::InvalidInput, FIELD_TIMER));
    }
    debug_assert_eq!(state.run_id, decision.run_id);
    debug_assert_eq!(state.decision_hash, decision.decision_hash);
    Ok(())
}

fn timer_observation_matches(
    decision: &ProviderRetryDecision,
    state: &ProviderRetryWaitState,
    observation: &ProviderRetryTimerObservation,
) -> bool {
    if state.status != ProviderRetryWaitStatus::Waiting {
        return false;
    }
    if decision.run_id != observation.run_id {
        return false;
    }
    if decision.next_attempt_count != Some(observation.next_attempt_count) {
        return false;
    }
    if decision.exact_plan_hash != observation.exact_plan_hash {
        return false;
    }
    decision.decision_hash == observation.decision_hash
}

fn reduce_wait_status(
    state: &ProviderRetryWaitState,
    observation: &ProviderRetryTimerObservation,
    is_correlated: bool,
) -> (ProviderRetryDispatchGateStatus, ProviderRetryWaitStatus) {
    match state.status {
        ProviderRetryWaitStatus::Completed => (
            ProviderRetryDispatchGateStatus::BlockedStale,
            ProviderRetryWaitStatus::Completed,
        ),
        ProviderRetryWaitStatus::Cancelled => (
            ProviderRetryDispatchGateStatus::Cancelled,
            ProviderRetryWaitStatus::Cancelled,
        ),
        ProviderRetryWaitStatus::Blocked => (
            ProviderRetryDispatchGateStatus::BlockedTimer,
            ProviderRetryWaitStatus::Blocked,
        ),
        ProviderRetryWaitStatus::Waiting => {
            reduce_waiting_status(observation.outcome, is_correlated)
        }
    }
}

fn reduce_waiting_status(
    outcome: ProviderRetryTimerOutcome,
    is_correlated: bool,
) -> (ProviderRetryDispatchGateStatus, ProviderRetryWaitStatus) {
    let reduced = if is_correlated {
        match outcome {
            ProviderRetryTimerOutcome::Completed => (
                ProviderRetryDispatchGateStatus::Ready,
                ProviderRetryWaitStatus::Completed,
            ),
            ProviderRetryTimerOutcome::Cancelled => (
                ProviderRetryDispatchGateStatus::Cancelled,
                ProviderRetryWaitStatus::Cancelled,
            ),
            ProviderRetryTimerOutcome::TimedOut
            | ProviderRetryTimerOutcome::Unavailable
            | ProviderRetryTimerOutcome::Failed => (
                ProviderRetryDispatchGateStatus::BlockedTimer,
                ProviderRetryWaitStatus::Blocked,
            ),
        }
    } else {
        (
            ProviderRetryDispatchGateStatus::BlockedStale,
            ProviderRetryWaitStatus::Waiting,
        )
    };
    debug_assert_eq!(
        reduced.0 == ProviderRetryDispatchGateStatus::Ready,
        reduced.1 == ProviderRetryWaitStatus::Completed
    );
    debug_assert_eq!(
        reduced.0 == ProviderRetryDispatchGateStatus::Cancelled,
        reduced.1 == ProviderRetryWaitStatus::Cancelled
    );
    reduced
}

fn ready_dispatch_request(
    decision: &ProviderRetryDecision,
    observation: &ProviderRetryTimerObservation,
    status: ProviderRetryDispatchGateStatus,
) -> Option<ProviderRetryDispatchRequest> {
    if status != ProviderRetryDispatchGateStatus::Ready {
        return None;
    }
    Some(ProviderRetryDispatchRequest {
        run_id: observation.run_id.clone(),
        attempt_count: observation.next_attempt_count,
        exact_plan_hash: observation.exact_plan_hash.clone(),
        decision_hash: decision.decision_hash.clone(),
    })
}

const fn semantic_frame_from_model_frame(
    frame: animus_adapters::ModelStreamFrameKind,
) -> Option<ProviderRetrySemanticFrame> {
    match frame {
        animus_adapters::ModelStreamFrameKind::ContentDelta => {
            Some(ProviderRetrySemanticFrame::Content)
        }
        animus_adapters::ModelStreamFrameKind::ThoughtDelta => {
            Some(ProviderRetrySemanticFrame::Reasoning)
        }
        animus_adapters::ModelStreamFrameKind::ToolCallDelta => {
            Some(ProviderRetrySemanticFrame::ToolCall)
        }
        animus_adapters::ModelStreamFrameKind::Usage => Some(ProviderRetrySemanticFrame::Other),
        animus_adapters::ModelStreamFrameKind::Finish
        | animus_adapters::ModelStreamFrameKind::Cancelled
        | animus_adapters::ModelStreamFrameKind::Failure => None,
    }
}

const fn progress_status_for_decision(
    status: ProviderRetryDecisionStatus,
) -> ProviderRetryProgressStatus {
    match status {
        ProviderRetryDecisionStatus::Retry => ProviderRetryProgressStatus::Scheduled,
        ProviderRetryDecisionStatus::Terminal => ProviderRetryProgressStatus::Terminal,
        ProviderRetryDecisionStatus::Exhausted => ProviderRetryProgressStatus::Exhausted,
        ProviderRetryDecisionStatus::RecompositionRequired => {
            ProviderRetryProgressStatus::RecompositionRequired
        }
        ProviderRetryDecisionStatus::PostCommitTerminal => ProviderRetryProgressStatus::PostCommit,
        ProviderRetryDecisionStatus::Cancelled => ProviderRetryProgressStatus::Cancelled,
    }
}

const fn progress_status_for_gate(
    status: ProviderRetryDispatchGateStatus,
) -> ProviderRetryProgressStatus {
    match status {
        ProviderRetryDispatchGateStatus::Ready => ProviderRetryProgressStatus::DispatchReady,
        ProviderRetryDispatchGateStatus::Cancelled => ProviderRetryProgressStatus::Cancelled,
        ProviderRetryDispatchGateStatus::BlockedTimer
        | ProviderRetryDispatchGateStatus::BlockedStale => {
            ProviderRetryProgressStatus::TimerBlocked
        }
    }
}

pub(super) fn hash_owned_parts(domain: &str, parts: &[String]) -> String {
    let mut material = String::from(domain);
    for part in parts {
        material.push_str(HASH_SEPARATOR);
        material.push_str(part.as_str());
    }
    blake3::hash(material.as_bytes()).to_hex().to_string()
}

pub(super) fn safe_label(value: &str, field: RetryField) -> Result<(), ProviderRetryError> {
    let has_valid_char_count = !value.is_empty() && value.chars().count() <= MAX_SAFE_LABEL_CHARS;
    let has_valid_characters = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'));
    if !has_valid_char_count || !has_valid_characters {
        return Err(error(ProviderRetryIssue::InvalidInput, field));
    }
    Ok(())
}

pub(super) fn lowercase_blake3(value: &str, field: RetryField) -> Result<(), ProviderRetryError> {
    let is_valid_hash = value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !is_valid_hash {
        return Err(error(ProviderRetryIssue::InvalidInput, field));
    }
    Ok(())
}

pub(super) const fn error(issue: ProviderRetryIssue, field: RetryField) -> ProviderRetryError {
    ProviderRetryError::new(issue, field.0)
}
