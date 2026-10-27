use super::{
    CatchUpRule, DurableClaimProof, FORBIDDEN_RECEIPT_MARKERS, HASH_HEX_LENGTH,
    MAX_POLICY_JOB_COUNT, MIN_BOUND, POLICY_DOMAIN, RECEIPT_DOMAIN, REQUIRED_NON_CLAIMS,
    REQUIRED_RECEIPT_FIELDS, REVIEWED_COHORT, REVIEWED_REVISION, SCHEDULE_DOMAIN, SCHEDULE_SCHEMA,
    STATE_DOMAIN, ScheduleClaimTransition, ScheduleDecisionClass, ScheduleDeliveryInstruction,
    ScheduleError, ScheduleJob, ScheduleMutationAction, ScheduleMutationPlan,
    ScheduleMutationRequest, ScheduleOwnerClass, SchedulePolicy, SchedulePolicyDraft,
    ScheduleReceipt, ScheduleReceiptDraft, ScheduleState, ScheduleSurface, ScheduleTickPlan,
    has_required_values, hash_parts, is_safe_identity,
};

/// Admits one schedule mutation without performing durable storage effects.
///
/// # Errors
///
/// Returns a typed denial for cross-session, cross-owner, or protected-surface
/// mutation.
pub fn admit_schedule_mutation(
    state: &ScheduleState,
    request: &ScheduleMutationRequest,
) -> Result<ScheduleMutationPlan, ScheduleError> {
    validate_mutation_request(state, request)?;
    let existing = state.jobs.iter().find(|job| job.job_id == request.job_id);
    validate_mutation_target(request, existing)?;
    if request.action == ScheduleMutationAction::Create {
        validate_create_request(state, request)?;
    }
    Ok(ScheduleMutationPlan {
        job_id: request.job_id.clone(),
        action: request.action,
        surface: request.surface,
    })
}

/// Evaluates one job against an injected logical clock.
///
/// # Errors
///
/// Returns a typed error when job state or missed-count metadata exceeds policy.
pub fn evaluate_schedule_tick(
    policy: &SchedulePolicy,
    state: &ScheduleState,
    job_id: &str,
    current_tick: u64,
) -> Result<ScheduleTickPlan, ScheduleError> {
    let job = state
        .jobs
        .iter()
        .find(|candidate| candidate.job_id == job_id)
        .ok_or(ScheduleError::InvalidIdentity)?;
    validate_job(policy, state.session_id.as_str(), job)?;
    if job.is_paused || current_tick < job.next_due_tick {
        return Ok(non_claim_plan(ScheduleDecisionClass::NotDue, 0));
    }
    if job.claimed_tick == Some(job.next_due_tick) {
        return Ok(non_claim_plan(ScheduleDecisionClass::ClaimedUndelivered, 0));
    }
    let elapsed_ticks = current_tick
        .checked_sub(job.next_due_tick)
        .ok_or(ScheduleError::InvalidInterval)?;
    let due_count = elapsed_ticks
        .checked_div(job.interval_ticks)
        .and_then(|count| count.checked_add(1))
        .ok_or(ScheduleError::InvalidInterval)?;
    let missed_count = due_count
        .checked_sub(1)
        .ok_or(ScheduleError::InvalidInterval)?;
    if missed_count > policy.maximum_missed_count {
        return Err(ScheduleError::MissedCountExceeded);
    }
    if missed_count == 0 {
        return Ok(claim_plan(state, job, missed_count));
    }
    Ok(coalesce_plan(policy, state, job, missed_count))
}

/// Admits delivery only after the host supplies an exact durable claim proof.
///
/// # Errors
///
/// Returns [`ScheduleError::ClaimNotDurable`] when the plan has no claim or the
/// proof does not bind that claim and a new durable state identity.
pub fn admit_delivery_after_claim(
    plan: &ScheduleTickPlan,
    proof: &DurableClaimProof,
) -> Result<ScheduleDeliveryInstruction, ScheduleError> {
    let claim = plan.claim.as_ref().ok_or(ScheduleError::ClaimNotDurable)?;
    if proof.claim_identity != claim.claim_identity {
        return Err(ScheduleError::ClaimNotDurable);
    }
    if proof.durable_state_identity.is_empty() {
        return Err(ScheduleError::ClaimNotDurable);
    }
    if proof.durable_state_identity == claim.prior_state_identity {
        return Err(ScheduleError::ClaimNotDurable);
    }
    debug_assert_eq!(plan.class, ScheduleDecisionClass::ClaimRequired);
    Ok(ScheduleDeliveryInstruction {
        job_id: claim.job_id.clone(),
        due_tick: claim.due_tick,
        claim_identity: claim.claim_identity.clone(),
    })
}

/// Projects one metadata-only schedule receipt.
///
/// # Errors
///
/// Returns [`ScheduleError::UnsafeReceipt`] when identities, counts, policy,
/// or non-claims do not match the admitted schedule boundary.
pub fn project_schedule_receipt(
    policy: &SchedulePolicy,
    draft: ScheduleReceiptDraft,
) -> Result<ScheduleReceipt, ScheduleError> {
    validate_receipt_draft(policy, &draft)?;
    let missed_count = draft.missed_count.to_string();
    let identity = hash_parts(
        RECEIPT_DOMAIN,
        &[
            SCHEDULE_SCHEMA,
            draft.policy_identity.as_str(),
            draft.state_identity.as_str(),
            draft.job_identity.as_str(),
            draft.surface.as_str(),
            draft.decision_class.as_str(),
            draft.clock_identity.as_str(),
            missed_count.as_str(),
        ],
    );
    debug_assert_eq!(identity.len(), HASH_HEX_LENGTH);
    Ok(ScheduleReceipt { draft, identity })
}

/// Rejects text that cannot appear in metadata-only schedule evidence.
///
/// # Errors
///
/// Returns [`ScheduleError::UnsafeReceipt`] for prompt bodies, host output,
/// physical paths, or unsupported exactly-once claims.
pub fn validate_schedule_receipt_text(value: &str) -> Result<(), ScheduleError> {
    let normalized = value.to_ascii_lowercase();
    if FORBIDDEN_RECEIPT_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
    {
        return Err(ScheduleError::UnsafeReceipt);
    }
    Ok(())
}

pub(super) fn validate_policy_draft(draft: &SchedulePolicyDraft) -> Result<(), ScheduleError> {
    if draft.cohort != REVIEWED_COHORT || draft.source_revision != REVIEWED_REVISION {
        return Err(ScheduleError::PolicyRejected);
    }
    if draft.minimum_interval_ticks == 0 {
        return Err(ScheduleError::PolicyRejected);
    }
    if draft.maximum_interval_ticks < draft.minimum_interval_ticks {
        return Err(ScheduleError::PolicyRejected);
    }
    if draft.maximum_jobs_per_session < MIN_BOUND {
        return Err(ScheduleError::PolicyRejected);
    }
    if draft.maximum_jobs_per_session > MAX_POLICY_JOB_COUNT {
        return Err(ScheduleError::PolicyRejected);
    }
    if draft.maximum_missed_count == 0 {
        return Err(ScheduleError::PolicyRejected);
    }
    if !has_required_values(&draft.receipt_fields, &REQUIRED_RECEIPT_FIELDS) {
        return Err(ScheduleError::PolicyRejected);
    }
    if !has_required_values(&draft.non_claims, &REQUIRED_NON_CLAIMS) {
        return Err(ScheduleError::PolicyRejected);
    }
    Ok(())
}

pub(super) fn policy_identity(draft: &SchedulePolicyDraft) -> String {
    let minimum_interval = draft.minimum_interval_ticks.to_string();
    let maximum_interval = draft.maximum_interval_ticks.to_string();
    let maximum_jobs = draft.maximum_jobs_per_session.to_string();
    let maximum_missed = draft.maximum_missed_count.to_string();
    let mut parts = vec![
        SCHEDULE_SCHEMA,
        draft.cohort.as_str(),
        draft.source_revision.as_str(),
        minimum_interval.as_str(),
        maximum_interval.as_str(),
        maximum_jobs.as_str(),
        maximum_missed.as_str(),
        draft.user_heartbeat_catch_up.as_str(),
        draft.agent_recurring_catch_up.as_str(),
        draft.scheduled_catch_up.as_str(),
    ];
    parts.extend(draft.receipt_fields.iter().map(String::as_str));
    parts.extend(draft.non_claims.iter().map(String::as_str));
    hash_parts(POLICY_DOMAIN, parts.as_slice())
}

pub(super) fn validate_recovered_jobs(
    policy: &SchedulePolicy,
    session_id: &str,
    jobs: &[ScheduleJob],
) -> Result<(), ScheduleError> {
    if !is_safe_identity(session_id) {
        return Err(ScheduleError::InvalidIdentity);
    }
    let job_count = u32::try_from(jobs.len()).map_err(|_| ScheduleError::JobLimitExceeded)?;
    if job_count > policy.maximum_jobs_per_session {
        return Err(ScheduleError::JobLimitExceeded);
    }
    let mut user_heartbeat_count = 0_u32;
    for (job_index, job) in jobs.iter().enumerate() {
        validate_job(policy, session_id, job)?;
        if job.surface == ScheduleSurface::UserHeartbeat {
            user_heartbeat_count = user_heartbeat_count
                .checked_add(1)
                .ok_or(ScheduleError::DuplicateUserHeartbeat)?;
        }
        if jobs[..job_index]
            .iter()
            .any(|previous| previous.job_id == job.job_id)
        {
            return Err(ScheduleError::DuplicateJob);
        }
    }
    if user_heartbeat_count > MIN_BOUND {
        return Err(ScheduleError::DuplicateUserHeartbeat);
    }
    Ok(())
}

pub(super) fn state_identity(session_id: &str, jobs: &[ScheduleJob]) -> String {
    let mut sorted_jobs = jobs.to_vec();
    sorted_jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
    let mut owned_parts = Vec::with_capacity(sorted_jobs.len());
    for job in &sorted_jobs {
        owned_parts.push(job_identity_part(job));
    }
    let mut parts = vec![SCHEDULE_SCHEMA, session_id];
    parts.extend(owned_parts.iter().map(String::as_str));
    hash_parts(STATE_DOMAIN, parts.as_slice())
}

fn validate_job(
    policy: &SchedulePolicy,
    session_id: &str,
    job: &ScheduleJob,
) -> Result<(), ScheduleError> {
    if job.session_id != session_id {
        return Err(ScheduleError::CrossSession);
    }
    if !is_safe_identity(job.job_id.as_str()) || !is_safe_identity(job.owner_id.as_str()) {
        return Err(ScheduleError::InvalidIdentity);
    }
    if job.interval_ticks < policy.minimum_interval_ticks {
        return Err(ScheduleError::InvalidInterval);
    }
    if job.interval_ticks > policy.maximum_interval_ticks {
        return Err(ScheduleError::InvalidInterval);
    }
    validate_surface_owner(job.surface, job.owner_class)
}

fn validate_surface_owner(
    surface: ScheduleSurface,
    owner_class: ScheduleOwnerClass,
) -> Result<(), ScheduleError> {
    let is_valid = match surface {
        ScheduleSurface::UserHeartbeat => owner_class == ScheduleOwnerClass::User,
        ScheduleSurface::AgentRecurring => owner_class == ScheduleOwnerClass::Agent,
        ScheduleSurface::Scheduled => {
            matches!(
                owner_class,
                ScheduleOwnerClass::User | ScheduleOwnerClass::Host
            )
        }
    };
    if !is_valid {
        return Err(ScheduleError::OwnerSurfaceMismatch);
    }
    Ok(())
}

fn validate_mutation_request(
    state: &ScheduleState,
    request: &ScheduleMutationRequest,
) -> Result<(), ScheduleError> {
    if request.session_id != state.session_id {
        return Err(ScheduleError::CrossSession);
    }
    if !is_safe_identity(request.requester_id.as_str())
        || !is_safe_identity(request.job_id.as_str())
    {
        return Err(ScheduleError::InvalidIdentity);
    }
    validate_surface_owner(request.surface, request.requester_class)
}

fn validate_mutation_target(
    request: &ScheduleMutationRequest,
    existing: Option<&ScheduleJob>,
) -> Result<(), ScheduleError> {
    if request.action == ScheduleMutationAction::Create {
        if existing.is_some() {
            return Err(ScheduleError::DuplicateJob);
        }
        return Ok(());
    }
    let job = existing.ok_or(ScheduleError::MutationDenied)?;
    if job.surface != request.surface {
        return Err(ScheduleError::MutationDenied);
    }
    if job.owner_id != request.requester_id {
        return Err(ScheduleError::MutationDenied);
    }
    if matches!(
        (request.requester_class, job.surface),
        (ScheduleOwnerClass::Agent, ScheduleSurface::UserHeartbeat)
    ) {
        return Err(ScheduleError::MutationDenied);
    }
    Ok(())
}

fn validate_create_request(
    state: &ScheduleState,
    request: &ScheduleMutationRequest,
) -> Result<(), ScheduleError> {
    let has_user_heartbeat = state
        .jobs
        .iter()
        .any(|job| job.surface == ScheduleSurface::UserHeartbeat);
    if matches!(
        (request.surface, has_user_heartbeat),
        (ScheduleSurface::UserHeartbeat, true)
    ) {
        return Err(ScheduleError::DuplicateUserHeartbeat);
    }
    Ok(())
}

fn non_claim_plan(class: ScheduleDecisionClass, missed_count: u64) -> ScheduleTickPlan {
    ScheduleTickPlan {
        class,
        missed_count,
        claim: None,
    }
}

fn claim_plan(state: &ScheduleState, job: &ScheduleJob, missed_count: u64) -> ScheduleTickPlan {
    let due_tick = job.next_due_tick.to_string();
    let missed = missed_count.to_string();
    let claim_identity = hash_parts(
        SCHEDULE_DOMAIN,
        &[
            state.identity.as_str(),
            job.job_id.as_str(),
            due_tick.as_str(),
            missed.as_str(),
        ],
    );
    ScheduleTickPlan {
        class: ScheduleDecisionClass::ClaimRequired,
        missed_count,
        claim: Some(ScheduleClaimTransition {
            job_id: job.job_id.clone(),
            due_tick: job.next_due_tick,
            prior_state_identity: state.identity.clone(),
            claim_identity,
        }),
    }
}

fn coalesce_plan(
    policy: &SchedulePolicy,
    state: &ScheduleState,
    job: &ScheduleJob,
    missed_count: u64,
) -> ScheduleTickPlan {
    match policy.catch_up_rule(job.surface) {
        CatchUpRule::FireOnce => claim_plan(state, job, missed_count),
        CatchUpRule::Skip => non_claim_plan(ScheduleDecisionClass::CoalescedSkip, missed_count),
        CatchUpRule::Pause => non_claim_plan(ScheduleDecisionClass::CoalescedPause, missed_count),
    }
}

fn validate_receipt_draft(
    policy: &SchedulePolicy,
    draft: &ScheduleReceiptDraft,
) -> Result<(), ScheduleError> {
    if draft.policy_identity != policy.identity {
        return Err(ScheduleError::UnsafeReceipt);
    }
    if !is_safe_identity(draft.job_identity.as_str()) {
        return Err(ScheduleError::UnsafeReceipt);
    }
    if !is_safe_identity(draft.clock_identity.as_str()) {
        return Err(ScheduleError::UnsafeReceipt);
    }
    if draft.state_identity.len() != HASH_HEX_LENGTH {
        return Err(ScheduleError::UnsafeReceipt);
    }
    if draft.missed_count > policy.maximum_missed_count {
        return Err(ScheduleError::UnsafeReceipt);
    }
    if !has_required_values(&draft.non_claims, &REQUIRED_NON_CLAIMS) {
        return Err(ScheduleError::UnsafeReceipt);
    }
    if !has_required_values(&policy.receipt_fields, &REQUIRED_RECEIPT_FIELDS) {
        return Err(ScheduleError::UnsafeReceipt);
    }
    Ok(())
}

fn job_identity_part(job: &ScheduleJob) -> String {
    let interval = job.interval_ticks.to_string();
    let next_due = job.next_due_tick.to_string();
    let claimed = job
        .claimed_tick
        .map_or_else(String::new, |tick| tick.to_string());
    let is_paused = if job.is_paused { "paused" } else { "active" };
    hash_parts(
        SCHEDULE_DOMAIN,
        &[
            job.session_id.as_str(),
            job.job_id.as_str(),
            job.owner_id.as_str(),
            job.owner_class.as_str(),
            job.surface.as_str(),
            interval.as_str(),
            next_due.as_str(),
            claimed.as_str(),
            is_paused,
        ],
    )
}
