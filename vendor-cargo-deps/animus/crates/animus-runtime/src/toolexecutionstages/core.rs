use super::data::{
    TOOL_EXECUTION_STAGE_CONTRACT_IDENTITY, TOOL_EXECUTION_STAGE_PHASES,
    ToolExecutionStageCandidate, ToolExecutionStageCandidateDraft,
    ToolExecutionStageCommitObservation, ToolExecutionStageDisposition, ToolExecutionStageError,
    ToolExecutionStageFinalObservation, ToolExecutionStageHandler, ToolExecutionStageHandlerDraft,
    ToolExecutionStageIssue, ToolExecutionStageOutcomeClass, ToolExecutionStageOutputObservation,
    ToolExecutionStagePhase, ToolExecutionStagePlan, ToolExecutionStagePlanDraft,
    ToolExecutionStageRawObservation, ToolExecutionStageState, ToolExecutionStageStatus,
};
use crate::tool_executor_kit::{ToolExecutionPlan, ToolMutationPosture};

const PLAN_IDENTITY_DOMAIN: &str = "animus.tool-execution-stages.plan.v1";
const HANDLER_SET_IDENTITY_DOMAIN: &str = "animus.tool-execution-stages.handlers.v1";
const COMMIT_IDENTITY_DOMAIN: &str = "animus.tool-execution-stages.commit.v1";
const CODING_SESSION_COMMIT_OWNER_DOMAIN: &str =
    "animus.tool-execution-stages.coding-session-commit-owner.v1";
const MAX_STAGE_HANDLERS: usize = 32;
const BLAKE3_HEX_CHARS: usize = 64;
const MUTATION_RANK_NONE: u8 = 0;
const MUTATION_RANK_READ_ONLY: u8 = 1;
const MUTATION_RANK_DECLARED_TARGETS: u8 = 2;

/// Validates one host-selected stage handler descriptor.
///
/// # Errors
///
/// Returns a typed issue for a fixed owner phase or malformed identity.
pub fn validate_tool_execution_stage_handler(
    draft: ToolExecutionStageHandlerDraft<'_>,
) -> Result<ToolExecutionStageHandler, ToolExecutionStageError> {
    if !draft.phase.allows_handlers() {
        return Err(error(ToolExecutionStageIssue::InvalidHandlerPhase));
    }
    validate_identity(draft.handler_identity)?;
    validate_identity(draft.cohort_identity)?;
    Ok(ToolExecutionStageHandler {
        phase: draft.phase,
        handler_identity: String::from(draft.handler_identity),
        cohort_identity: String::from(draft.cohort_identity),
    })
}

// r[impl agent_harness_runtime.tool_execution_stages.plan]
/// Builds one immutable staged tool-execution plan.
///
/// # Errors
///
/// Returns a typed issue for reordered phases, stale request binding, malformed
/// identities, duplicate handlers, or an excessive selected handler set.
pub fn plan_tool_execution_stages(
    draft: ToolExecutionStagePlanDraft<'_>,
) -> Result<ToolExecutionStagePlan, ToolExecutionStageError> {
    if draft.phase_order != TOOL_EXECUTION_STAGE_PHASES {
        return Err(error(ToolExecutionStageIssue::InvalidPhaseOrder));
    }
    if draft.handlers.len() > MAX_STAGE_HANDLERS {
        return Err(error(ToolExecutionStageIssue::HandlerBoundExceeded));
    }
    validate_identity(draft.authority_seal_identity)?;
    validate_identity(draft.commit_owner_identity)?;
    validate_handler_set(draft.handlers)?;
    let request_identity = crate::tool_executor_kit::tool_request_binding_identity(draft.request);
    if request_identity != draft.tool_plan.request_identity()
        || draft.request.name() != draft.tool_plan.tool_name()
        || draft.request.schema_identity() != draft.tool_plan.schema_identity()
    {
        return Err(error(ToolExecutionStageIssue::PlanBindingMismatch));
    }
    let mut plan = ToolExecutionStagePlan {
        plan_identity: String::new(),
        handlers: draft.handlers.to_vec(),
        tool_name: String::from(draft.tool_plan.tool_name()),
        schema_identity: String::from(draft.tool_plan.schema_identity()),
        request_identity,
        executor_binding: draft.tool_plan.executor_binding().map(String::from),
        authority_seal_identity: String::from(draft.authority_seal_identity),
        capability_labels: draft.tool_plan.capability_labels().to_vec(),
        mutation_targets: draft.tool_plan.mutation_targets().to_vec(),
        mutation_posture: draft.tool_plan.mutation_posture(),
        output_policy: draft.tool_plan.output_policy(),
        cancellation: draft.tool_plan.cancellation(),
        deadline: draft.tool_plan.deadline(),
        commit_owner_identity: String::from(draft.commit_owner_identity),
    };
    plan.plan_identity = stage_plan_identity(&plan);
    debug_assert!(is_blake3_identity(plan.plan_identity()));
    Ok(plan)
}

/// Returns the exact ordered handler-set identity for one phase.
#[must_use]
pub fn tool_execution_stage_handler_set_identity(
    plan: &ToolExecutionStagePlan,
    phase: ToolExecutionStagePhase,
) -> String {
    let mut fields = vec![phase.as_str().as_bytes()];
    for handler in plan
        .handlers
        .iter()
        .filter(|handler| handler.phase == phase)
    {
        fields.push(handler.handler_identity.as_bytes());
        fields.push(handler.cohort_identity.as_bytes());
    }
    hash_fields(HANDLER_SET_IDENTITY_DOMAIN, fields.as_slice())
}

/// Returns the deterministic coding-session durable commit-owner identity.
#[must_use]
pub fn coding_session_tool_commit_owner_identity() -> String {
    hash_fields(CODING_SESSION_COMMIT_OWNER_DOMAIN, &[])
}

/// Builds one owned phase candidate before immutable-plan comparison.
///
/// # Errors
///
/// Returns a typed issue when a candidate identity is malformed.
pub fn build_tool_execution_stage_candidate(
    draft: ToolExecutionStageCandidateDraft<'_>,
) -> Result<ToolExecutionStageCandidate, ToolExecutionStageError> {
    validate_identity(draft.stage_plan_identity)?;
    validate_identity(draft.handler_set_identity)?;
    validate_identity(draft.request_identity)?;
    validate_identity(draft.authority_seal_identity)?;
    Ok(ToolExecutionStageCandidate {
        phase: draft.phase,
        stage_plan_identity: String::from(draft.stage_plan_identity),
        handler_set_identity: String::from(draft.handler_set_identity),
        request_identity: String::from(draft.request_identity),
        schema_identity: String::from(draft.schema_identity),
        executor_binding: draft.executor_binding.map(String::from),
        authority_seal_identity: String::from(draft.authority_seal_identity),
        capability_labels: draft.capability_labels.to_vec(),
        mutation_targets: draft.mutation_targets.to_vec(),
        mutation_posture: draft.mutation_posture,
        max_visible_chars: draft.max_visible_chars,
        max_output_bytes: draft.max_output_bytes,
        required_redaction: draft.required_redaction,
        disposition: draft.disposition,
        outcome_class: draft.outcome_class,
    })
}

/// Builds an exact admitted or denied candidate for one phase.
#[must_use]
pub fn exact_tool_execution_stage_candidate(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    phase: ToolExecutionStagePhase,
    disposition: ToolExecutionStageDisposition,
) -> ToolExecutionStageCandidate {
    ToolExecutionStageCandidate {
        phase,
        stage_plan_identity: plan.plan_identity.clone(),
        handler_set_identity: tool_execution_stage_handler_set_identity(plan, phase),
        request_identity: plan.request_identity.clone(),
        schema_identity: plan.schema_identity.clone(),
        executor_binding: plan.executor_binding.clone(),
        authority_seal_identity: plan.authority_seal_identity.clone(),
        capability_labels: plan.capability_labels.clone(),
        mutation_targets: plan.mutation_targets.clone(),
        mutation_posture: plan.mutation_posture,
        max_visible_chars: state.max_visible_chars,
        max_output_bytes: state.max_output_bytes,
        required_redaction: state.required_redaction,
        disposition,
        outcome_class: state.outcome_class,
    }
}

/// Starts one pure staged execution state.
#[must_use]
pub fn start_tool_execution_stages(plan: &ToolExecutionStagePlan) -> ToolExecutionStageState {
    let (max_visible_chars, max_output_bytes) = plan.output_policy.map_or((None, None), |policy| {
        (
            Some(policy.max_visible_chars()),
            Some(policy.max_output_bytes()),
        )
    });
    ToolExecutionStageState {
        stage_plan_identity: plan.plan_identity.clone(),
        next_phase: Some(ToolExecutionStagePhase::PreAdmission),
        status: ToolExecutionStageStatus::Active,
        outcome_class: ToolExecutionStageOutcomeClass::Pending,
        max_visible_chars,
        max_output_bytes,
        required_redaction: false,
        truncated: false,
        raw_output_identity: None,
        raw_output_byte_count: None,
        raw_output_char_count: None,
        narrowed_output_identity: None,
        result_identity: None,
        executor_receipt_identity: None,
        durable_head_identity: None,
        durable_commit_identity: None,
    }
}

// r[impl agent_harness_runtime.tool_execution_stages.monotonic]
/// Applies one exact pre-admission candidate.
///
/// # Errors
///
/// Returns a typed issue for substitution, widening, reordering, or denial reversal.
pub fn apply_tool_pre_admission_candidate(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    candidate: &ToolExecutionStageCandidate,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    apply_optional_candidate(
        plan,
        state,
        candidate,
        ToolExecutionStagePhase::PreAdmission,
    )
}

/// Applies the existing executor-kit plan as the authoritative final admission.
///
/// A denied kit plan produces sticky denial and never advances to execution.
///
/// # Errors
///
/// Returns a typed issue for reordered or substituted final admission.
pub fn apply_tool_final_admission(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    tool_plan: &ToolExecutionPlan,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    require_active_phase(state, ToolExecutionStagePhase::FinalAdmission)?;
    validate_tool_plan_binding(plan, tool_plan)?;
    if !tool_plan.is_admitted() {
        return Ok(denied_state(state));
    }
    let mut next = state.clone();
    next.next_phase = Some(ToolExecutionStagePhase::PreExecution);
    Ok(next)
}

/// Applies one exact pre-execution candidate.
///
/// # Errors
///
/// Returns a typed issue for substitution, widening, reordering, or denial reversal.
pub fn apply_tool_pre_execution_candidate(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    candidate: &ToolExecutionStageCandidate,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    apply_optional_candidate(
        plan,
        state,
        candidate,
        ToolExecutionStagePhase::PreExecution,
    )
}

/// Records one exact transient executor outcome.
///
/// # Errors
///
/// Returns a typed issue for reordered execution, stale binding, or malformed output identity.
pub fn observe_tool_stage_execution(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    observation: &ToolExecutionStageRawObservation,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    require_active_phase(state, ToolExecutionStagePhase::Execution)?;
    if observation.stage_plan_identity != plan.plan_identity
        || observation.request_identity != plan.request_identity
    {
        return Err(error(ToolExecutionStageIssue::RequestSubstitution));
    }
    validate_identity(observation.output_identity.as_str())?;
    if observation.outcome_class == ToolExecutionStageOutcomeClass::Pending {
        return Err(error(ToolExecutionStageIssue::OutcomeWidening));
    }
    let mut next = state.clone();
    next.outcome_class = observation.outcome_class;
    next.required_redaction = observation.required_redaction;
    next.raw_output_identity = Some(observation.output_identity.clone());
    next.raw_output_byte_count = Some(observation.output_byte_count);
    next.raw_output_char_count = Some(observation.output_char_count);
    next.next_phase = Some(ToolExecutionStagePhase::OutputNarrowing);
    Ok(next)
}

// r[impl agent_harness_runtime.tool_execution_stages.output]
/// Applies one bounded output-narrowing observation.
///
/// # Errors
///
/// Returns a typed issue for output substitution, class widening, bound increase,
/// redaction removal, or phase reordering.
pub fn apply_tool_output_narrowing(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    observation: &ToolExecutionStageOutputObservation,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    validate_candidate(
        plan,
        state,
        &observation.candidate,
        ToolExecutionStagePhase::OutputNarrowing,
    )?;
    let input_identity = state
        .raw_output_identity
        .as_deref()
        .ok_or_else(|| error(ToolExecutionStageIssue::OutputSubstitution))?;
    if observation.input_output_identity != input_identity {
        return Err(error(ToolExecutionStageIssue::OutputSubstitution));
    }
    validate_identity(observation.narrowed_output_identity.as_str())?;
    if observation.candidate.outcome_class != state.outcome_class {
        return Err(error(ToolExecutionStageIssue::OutcomeWidening));
    }
    if state.truncated && !observation.truncated {
        return Err(error(ToolExecutionStageIssue::OutcomeWidening));
    }
    validate_output_counts(state, observation)?;
    if observation.candidate.disposition == ToolExecutionStageDisposition::Denied {
        return Ok(denied_state(state));
    }
    let mut next = state.clone();
    next.max_visible_chars = observation.candidate.max_visible_chars;
    next.max_output_bytes = observation.candidate.max_output_bytes;
    next.required_redaction = observation.candidate.required_redaction;
    next.truncated |= observation.truncated;
    next.narrowed_output_identity = Some(observation.narrowed_output_identity.clone());
    next.next_phase = Some(ToolExecutionStagePhase::FinalValidation);
    Ok(next)
}

/// Records the existing executor kit's final output and receipt validation.
///
/// # Errors
///
/// Returns a typed issue for stale output, class widening, malformed evidence,
/// or final-validation reordering.
pub fn observe_tool_final_validation(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    observation: &ToolExecutionStageFinalObservation,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    require_active_phase(state, ToolExecutionStagePhase::FinalValidation)?;
    let narrowed_identity = state
        .narrowed_output_identity
        .as_deref()
        .ok_or_else(|| error(ToolExecutionStageIssue::OutputSubstitution))?;
    if observation.stage_plan_identity != plan.plan_identity
        || observation.narrowed_output_identity != narrowed_identity
    {
        return Err(error(ToolExecutionStageIssue::OutputSubstitution));
    }
    if observation.outcome_class != state.outcome_class
        || (state.truncated && !observation.truncated)
    {
        return Err(error(ToolExecutionStageIssue::OutcomeWidening));
    }
    validate_identity(observation.result_identity.as_str())?;
    validate_identity(observation.executor_receipt_identity.as_str())?;
    let mut next = state.clone();
    next.result_identity = Some(observation.result_identity.clone());
    next.executor_receipt_identity = Some(observation.executor_receipt_identity.clone());
    next.truncated |= observation.truncated;
    next.status = ToolExecutionStageStatus::AwaitingCommit;
    next.next_phase = Some(ToolExecutionStagePhase::DurableCommit);
    Ok(next)
}

// r[impl agent_harness_runtime.tool_execution_stages.commit]
/// Applies one exact coding-session-owned durable tool-result commit.
///
/// # Errors
///
/// Returns a typed issue for a missing, stale, partial, unsynchronized, or
/// mismatched commit.
pub fn observe_tool_stage_durable_commit(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    observation: &ToolExecutionStageCommitObservation,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    require_phase(state, ToolExecutionStagePhase::DurableCommit)?;
    if state.status != ToolExecutionStageStatus::AwaitingCommit {
        return Err(error(ToolExecutionStageIssue::CommitRequired));
    }
    let result_identity = state
        .result_identity
        .as_deref()
        .ok_or_else(|| error(ToolExecutionStageIssue::CommitRequired))?;
    if observation.stage_plan_identity != plan.plan_identity
        || observation.result_identity != result_identity
        || observation.commit_owner_identity != plan.commit_owner_identity
        || !observation.complete
        || !observation.synchronized
    {
        return Err(error(ToolExecutionStageIssue::CommitMismatch));
    }
    validate_identity(observation.durable_head_identity.as_str())?;
    let commit_identity = hash_fields(
        COMMIT_IDENTITY_DOMAIN,
        &[
            plan.plan_identity.as_bytes(),
            result_identity.as_bytes(),
            plan.commit_owner_identity.as_bytes(),
            observation.durable_head_identity.as_bytes(),
        ],
    );
    let mut next = state.clone();
    next.durable_head_identity = Some(observation.durable_head_identity.clone());
    next.durable_commit_identity = Some(commit_identity);
    next.status = ToolExecutionStageStatus::Committed;
    next.next_phase = Some(ToolExecutionStagePhase::PostCommitObservation);
    Ok(next)
}

/// Applies one metadata-only post-commit candidate.
///
/// # Errors
///
/// Returns a typed issue when observation precedes commit or changes any bound fact.
pub fn apply_tool_post_commit_candidate(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    candidate: &ToolExecutionStageCandidate,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    if state.status != ToolExecutionStageStatus::Committed {
        return Err(error(ToolExecutionStageIssue::CommitRequired));
    }
    let next = apply_optional_candidate(
        plan,
        state,
        candidate,
        ToolExecutionStagePhase::PostCommitObservation,
    )?;
    if next.status == ToolExecutionStageStatus::Denied {
        return Ok(next);
    }
    let mut observed = next;
    observed.status = ToolExecutionStageStatus::Observed;
    observed.next_phase = None;
    Ok(observed)
}

fn apply_optional_candidate(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    candidate: &ToolExecutionStageCandidate,
    phase: ToolExecutionStagePhase,
) -> Result<ToolExecutionStageState, ToolExecutionStageError> {
    validate_candidate(plan, state, candidate, phase)?;
    if candidate.disposition == ToolExecutionStageDisposition::Denied {
        return Ok(denied_state(state));
    }
    let mut next = state.clone();
    next.max_visible_chars = candidate.max_visible_chars;
    next.max_output_bytes = candidate.max_output_bytes;
    next.required_redaction = candidate.required_redaction;
    next.next_phase = Some(optional_next_phase(phase)?);
    Ok(next)
}

const fn optional_next_phase(
    phase: ToolExecutionStagePhase,
) -> Result<ToolExecutionStagePhase, ToolExecutionStageError> {
    match phase {
        ToolExecutionStagePhase::PreAdmission => Ok(ToolExecutionStagePhase::FinalAdmission),
        ToolExecutionStagePhase::PreExecution => Ok(ToolExecutionStagePhase::Execution),
        ToolExecutionStagePhase::PostCommitObservation => {
            Ok(ToolExecutionStagePhase::PostCommitObservation)
        }
        ToolExecutionStagePhase::OutputNarrowing
        | ToolExecutionStagePhase::FinalAdmission
        | ToolExecutionStagePhase::Execution
        | ToolExecutionStagePhase::FinalValidation
        | ToolExecutionStagePhase::DurableCommit => {
            Err(error(ToolExecutionStageIssue::PhaseOrderViolation))
        }
    }
}

fn validate_candidate(
    plan: &ToolExecutionStagePlan,
    state: &ToolExecutionStageState,
    candidate: &ToolExecutionStageCandidate,
    phase: ToolExecutionStagePhase,
) -> Result<(), ToolExecutionStageError> {
    require_active_phase(state, phase)?;
    if candidate.stage_plan_identity != plan.plan_identity
        || candidate.phase != phase
        || candidate.handler_set_identity != tool_execution_stage_handler_set_identity(plan, phase)
    {
        return Err(error(ToolExecutionStageIssue::HandlerSubstitution));
    }
    if candidate.request_identity != plan.request_identity
        || candidate.schema_identity != plan.schema_identity
        || candidate.executor_binding != plan.executor_binding
        || candidate.mutation_targets != plan.mutation_targets
    {
        return Err(error(ToolExecutionStageIssue::RequestSubstitution));
    }
    if candidate.authority_seal_identity != plan.authority_seal_identity {
        return Err(error(ToolExecutionStageIssue::AuthorityWidening));
    }
    validate_capabilities(plan, candidate)?;
    validate_mutation(plan, candidate)?;
    validate_bounds(state, candidate)?;
    if state.required_redaction && !candidate.required_redaction {
        return Err(error(ToolExecutionStageIssue::RedactionRemoved));
    }
    if candidate.outcome_class != state.outcome_class {
        return Err(error(ToolExecutionStageIssue::OutcomeWidening));
    }
    Ok(())
}

fn validate_capabilities(
    plan: &ToolExecutionStagePlan,
    candidate: &ToolExecutionStageCandidate,
) -> Result<(), ToolExecutionStageError> {
    if candidate
        .capability_labels
        .iter()
        .any(|label| !plan.capability_labels.contains(label))
    {
        return Err(error(ToolExecutionStageIssue::CapabilityWidening));
    }
    if candidate.disposition == ToolExecutionStageDisposition::Admitted
        && candidate.capability_labels != plan.capability_labels
    {
        return Err(error(ToolExecutionStageIssue::RequestSubstitution));
    }
    Ok(())
}

fn validate_mutation(
    plan: &ToolExecutionStagePlan,
    candidate: &ToolExecutionStageCandidate,
) -> Result<(), ToolExecutionStageError> {
    if mutation_rank(candidate.mutation_posture) > mutation_rank(plan.mutation_posture) {
        return Err(error(ToolExecutionStageIssue::MutationWidening));
    }
    if candidate.disposition == ToolExecutionStageDisposition::Admitted
        && candidate.mutation_posture != plan.mutation_posture
    {
        return Err(error(ToolExecutionStageIssue::RequestSubstitution));
    }
    Ok(())
}

fn validate_bounds(
    state: &ToolExecutionStageState,
    candidate: &ToolExecutionStageCandidate,
) -> Result<(), ToolExecutionStageError> {
    if optional_bound_widened(candidate.max_visible_chars, state.max_visible_chars)
        || optional_bound_widened(candidate.max_output_bytes, state.max_output_bytes)
    {
        return Err(error(ToolExecutionStageIssue::BoundWidening));
    }
    Ok(())
}

fn validate_output_counts(
    state: &ToolExecutionStageState,
    observation: &ToolExecutionStageOutputObservation,
) -> Result<(), ToolExecutionStageError> {
    let visible_bound = observation.candidate.max_visible_chars.map(u64::from);
    let exceeds_input = state
        .raw_output_byte_count
        .is_none_or(|count| observation.output_byte_count > count)
        || state
            .raw_output_char_count
            .is_none_or(|count| observation.output_char_count > count);
    if exceeds_input
        || visible_bound.is_some_and(|bound| observation.output_char_count > bound)
        || observation
            .candidate
            .max_output_bytes
            .is_some_and(|bound| observation.output_byte_count > bound)
    {
        return Err(error(ToolExecutionStageIssue::BoundWidening));
    }
    if state.max_visible_chars.is_none() != observation.candidate.max_visible_chars.is_none()
        || state.max_output_bytes.is_none() != observation.candidate.max_output_bytes.is_none()
    {
        return Err(error(ToolExecutionStageIssue::BoundWidening));
    }
    Ok(())
}

fn validate_tool_plan_binding(
    plan: &ToolExecutionStagePlan,
    tool_plan: &ToolExecutionPlan,
) -> Result<(), ToolExecutionStageError> {
    if tool_plan.tool_name() != plan.tool_name
        || tool_plan.schema_identity() != plan.schema_identity
        || tool_plan.request_identity() != plan.request_identity
        || tool_plan.executor_binding() != plan.executor_binding.as_deref()
        || tool_plan.capability_labels() != plan.capability_labels
        || tool_plan.mutation_targets() != plan.mutation_targets
        || tool_plan.mutation_posture() != plan.mutation_posture
        || tool_plan.output_policy() != plan.output_policy
        || tool_plan.cancellation() != plan.cancellation
        || tool_plan.deadline() != plan.deadline
    {
        return Err(error(ToolExecutionStageIssue::PlanBindingMismatch));
    }
    Ok(())
}

fn validate_handler_set(
    handlers: &[ToolExecutionStageHandler],
) -> Result<(), ToolExecutionStageError> {
    for (index, handler) in handlers.iter().enumerate() {
        if !handler.phase.allows_handlers() {
            return Err(error(ToolExecutionStageIssue::InvalidHandlerPhase));
        }
        validate_identity(handler.handler_identity.as_str())?;
        validate_identity(handler.cohort_identity.as_str())?;
        if handlers[..index]
            .iter()
            .any(|seen| seen.handler_identity == handler.handler_identity)
        {
            return Err(error(ToolExecutionStageIssue::DuplicateHandler));
        }
    }
    Ok(())
}

fn require_active_phase(
    state: &ToolExecutionStageState,
    phase: ToolExecutionStagePhase,
) -> Result<(), ToolExecutionStageError> {
    if state.status == ToolExecutionStageStatus::Denied {
        return Err(error(ToolExecutionStageIssue::StickyDenial));
    }
    if state.status != ToolExecutionStageStatus::Active
        && !(state.status == ToolExecutionStageStatus::Committed
            && phase == ToolExecutionStagePhase::PostCommitObservation)
    {
        return Err(error(ToolExecutionStageIssue::PhaseOrderViolation));
    }
    require_phase(state, phase)
}

fn require_phase(
    state: &ToolExecutionStageState,
    phase: ToolExecutionStagePhase,
) -> Result<(), ToolExecutionStageError> {
    if state.next_phase != Some(phase) {
        return Err(error(ToolExecutionStageIssue::PhaseOrderViolation));
    }
    Ok(())
}

fn denied_state(state: &ToolExecutionStageState) -> ToolExecutionStageState {
    let mut denied = state.clone();
    denied.status = ToolExecutionStageStatus::Denied;
    denied.next_phase = None;
    denied
}

const fn mutation_rank(posture: Option<ToolMutationPosture>) -> u8 {
    match posture {
        None => MUTATION_RANK_NONE,
        Some(ToolMutationPosture::ReadOnly) => MUTATION_RANK_READ_ONLY,
        Some(ToolMutationPosture::DeclaredTargets) => MUTATION_RANK_DECLARED_TARGETS,
    }
}

fn optional_bound_widened<T: Ord>(candidate: Option<T>, current: Option<T>) -> bool {
    match (candidate, current) {
        (Some(candidate), Some(current)) => candidate > current,
        (Some(_), None) | (None, Some(_)) => true,
        (None, None) => false,
    }
}

fn stage_plan_identity(plan: &ToolExecutionStagePlan) -> String {
    let mutation = match plan.mutation_posture {
        None => "none",
        Some(ToolMutationPosture::ReadOnly) => "read_only",
        Some(ToolMutationPosture::DeclaredTargets) => "declared_targets",
    };
    let cancellation = match plan.cancellation {
        crate::tool_executor_kit::ToolExecutorCancellation::Continue => "continue",
        crate::tool_executor_kit::ToolExecutorCancellation::Cancelled => "cancelled",
    };
    let deadline = match plan.deadline {
        crate::tool_executor_kit::ToolExecutorDeadline::WithinDeadline => "within_deadline",
        crate::tool_executor_kit::ToolExecutorDeadline::TimedOut => "timed_out",
    };
    let visible_chars = plan.output_policy.map_or_else(
        || String::from("none"),
        |policy| policy.max_visible_chars().to_string(),
    );
    let output_bytes = plan.output_policy.map_or_else(
        || String::from("none"),
        |policy| policy.max_output_bytes().to_string(),
    );
    let mut fields = vec![
        TOOL_EXECUTION_STAGE_CONTRACT_IDENTITY.as_bytes(),
        plan.tool_name.as_bytes(),
        plan.schema_identity.as_bytes(),
        plan.request_identity.as_bytes(),
        plan.executor_binding
            .as_deref()
            .unwrap_or("none")
            .as_bytes(),
        plan.authority_seal_identity.as_bytes(),
        mutation.as_bytes(),
        visible_chars.as_bytes(),
        output_bytes.as_bytes(),
        cancellation.as_bytes(),
        deadline.as_bytes(),
        plan.commit_owner_identity.as_bytes(),
    ];
    for phase in TOOL_EXECUTION_STAGE_PHASES {
        fields.push(phase.as_str().as_bytes());
    }
    for capability in &plan.capability_labels {
        fields.push(capability.as_bytes());
    }
    for target in &plan.mutation_targets {
        fields.push(target.as_bytes());
    }
    for handler in &plan.handlers {
        fields.push(handler.phase.as_str().as_bytes());
        fields.push(handler.handler_identity.as_bytes());
        fields.push(handler.cohort_identity.as_bytes());
    }
    hash_fields(PLAN_IDENTITY_DOMAIN, fields.as_slice())
}

pub(super) fn hash_fields(domain: &str, fields: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    hash_field(&mut hasher, domain.as_bytes());
    for field in fields {
        hash_field(&mut hasher, field);
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_field(hasher: &mut blake3::Hasher, field: &[u8]) {
    let canonical_length = field.len() as u64;
    hasher.update(&canonical_length.to_le_bytes());
    hasher.update(field);
}

pub(super) fn is_blake3_identity(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_identity(value: &str) -> Result<(), ToolExecutionStageError> {
    if is_blake3_identity(value) {
        Ok(())
    } else {
        Err(error(ToolExecutionStageIssue::InvalidIdentity))
    }
}

pub(super) const fn error(issue: ToolExecutionStageIssue) -> ToolExecutionStageError {
    ToolExecutionStageError { issue }
}
