use super::{
    CatchUpRule, DurableClaimProof, HASH_HEX_LENGTH, REQUIRED_NON_CLAIMS, REQUIRED_RECEIPT_FIELDS,
    REVIEWED_COHORT, REVIEWED_REVISION, ScheduleDecisionClass, ScheduleError, ScheduleJob,
    ScheduleMutationAction, ScheduleMutationRequest, ScheduleOwnerClass, SchedulePolicy,
    SchedulePolicyDraft, ScheduleReceiptDraft, ScheduleState, ScheduleSurface,
    admit_delivery_after_claim, admit_schedule_mutation, evaluate_schedule_tick,
    project_schedule_receipt, validate_schedule_receipt_text,
};

const SESSION_ID: &str = "session:alpha";
const OTHER_SESSION_ID: &str = "session:beta";
const USER_ID: &str = "user:owner";
const AGENT_ID: &str = "agent:session";
const HOST_ID: &str = "host:scheduler";
const USER_JOB_ID: &str = "job:user-heartbeat";
const AGENT_JOB_ID: &str = "job:agent-recurring";
const HOST_JOB_ID: &str = "job:scheduled";
const CLOCK_ID: &str = "clock:fixture";
const MINIMUM_INTERVAL_TICKS: u64 = 5;
const MAXIMUM_INTERVAL_TICKS: u64 = 3_600;
const MAXIMUM_JOBS: u32 = 8;
const MAXIMUM_MISSED_COUNT: u64 = 100;
const DUE_TICK: u64 = 10;
const ON_TIME_TICK: u64 = 10;
const LATE_TICK: u64 = 30;
const INTERVAL_TICKS: u64 = 5;
const EXPECTED_MISSED_COUNT: u64 = 4;

fn policy_draft() -> SchedulePolicyDraft {
    SchedulePolicyDraft {
        cohort: REVIEWED_COHORT.to_string(),
        source_revision: REVIEWED_REVISION.to_string(),
        minimum_interval_ticks: MINIMUM_INTERVAL_TICKS,
        maximum_interval_ticks: MAXIMUM_INTERVAL_TICKS,
        maximum_jobs_per_session: MAXIMUM_JOBS,
        maximum_missed_count: MAXIMUM_MISSED_COUNT,
        user_heartbeat_catch_up: CatchUpRule::FireOnce,
        agent_recurring_catch_up: CatchUpRule::Skip,
        scheduled_catch_up: CatchUpRule::Pause,
        receipt_fields: REQUIRED_RECEIPT_FIELDS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        non_claims: required_non_claims(),
    }
}

fn policy() -> SchedulePolicy {
    SchedulePolicy::validate(&policy_draft()).expect("fixture policy must validate")
}

fn required_non_claims() -> Vec<String> {
    REQUIRED_NON_CLAIMS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn user_job() -> ScheduleJob {
    ScheduleJob {
        session_id: SESSION_ID.to_string(),
        job_id: USER_JOB_ID.to_string(),
        owner_id: USER_ID.to_string(),
        owner_class: ScheduleOwnerClass::User,
        surface: ScheduleSurface::UserHeartbeat,
        interval_ticks: INTERVAL_TICKS,
        next_due_tick: DUE_TICK,
        claimed_tick: None,
        is_paused: false,
    }
}

fn agent_job() -> ScheduleJob {
    ScheduleJob {
        session_id: SESSION_ID.to_string(),
        job_id: AGENT_JOB_ID.to_string(),
        owner_id: AGENT_ID.to_string(),
        owner_class: ScheduleOwnerClass::Agent,
        surface: ScheduleSurface::AgentRecurring,
        interval_ticks: INTERVAL_TICKS,
        next_due_tick: DUE_TICK,
        claimed_tick: None,
        is_paused: false,
    }
}

fn host_job() -> ScheduleJob {
    ScheduleJob {
        session_id: SESSION_ID.to_string(),
        job_id: HOST_JOB_ID.to_string(),
        owner_id: HOST_ID.to_string(),
        owner_class: ScheduleOwnerClass::Host,
        surface: ScheduleSurface::Scheduled,
        interval_ticks: INTERVAL_TICKS,
        next_due_tick: DUE_TICK,
        claimed_tick: None,
        is_paused: false,
    }
}

fn state(jobs: Vec<ScheduleJob>) -> ScheduleState {
    ScheduleState::recover(&policy(), SESSION_ID, jobs).expect("fixture state must recover")
}

#[test]
fn on_time_tick_requires_durable_claim_before_delivery() {
    // r[verify agent_harness_runtime.schedule_reliability.claim_before_delivery.ordered]
    let schedule_state = state(vec![user_job()]);
    let plan = evaluate_schedule_tick(&policy(), &schedule_state, USER_JOB_ID, ON_TIME_TICK)
        .expect("on-time tick must plan");
    let claim = plan.claim.as_ref().expect("claim must exist");

    assert_eq!(plan.class, ScheduleDecisionClass::ClaimRequired);
    assert_eq!(plan.missed_count, 0);
    assert_eq!(
        admit_delivery_after_claim(
            &plan,
            &DurableClaimProof {
                claim_identity: claim.claim_identity.clone(),
                durable_state_identity:
                    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
            }
        )
        .expect("durable claim must admit delivery")
        .job_id,
        USER_JOB_ID
    );
}

#[test]
fn crash_recovery_does_not_redeliver_claimed_tick() {
    // r[verify agent_harness_runtime.schedule_reliability.claim_before_delivery.crash]
    let mut claimed = user_job();
    claimed.claimed_tick = Some(DUE_TICK);
    let schedule_state = state(vec![claimed]);
    let plan = evaluate_schedule_tick(&policy(), &schedule_state, USER_JOB_ID, ON_TIME_TICK)
        .expect("recovery must classify the durable claim");

    assert_eq!(plan.class, ScheduleDecisionClass::ClaimedUndelivered);
    assert!(plan.claim.is_none());
    assert_eq!(
        admit_delivery_after_claim(
            &plan,
            &DurableClaimProof {
                claim_identity: String::new(),
                durable_state_identity: schedule_state.identity().to_string(),
            }
        ),
        Err(ScheduleError::ClaimNotDurable)
    );
}

#[test]
fn downtime_coalesces_to_one_surface_policy_decision() {
    // r[verify agent_harness_runtime.schedule_reliability.coalesce.catch_up]
    let schedule_state = state(vec![user_job(), agent_job(), host_job()]);
    let user_plan = evaluate_schedule_tick(&policy(), &schedule_state, USER_JOB_ID, LATE_TICK)
        .expect("user catch-up must plan");
    let agent_plan = evaluate_schedule_tick(&policy(), &schedule_state, AGENT_JOB_ID, LATE_TICK)
        .expect("agent catch-up must plan");
    let host_plan = evaluate_schedule_tick(&policy(), &schedule_state, HOST_JOB_ID, LATE_TICK)
        .expect("host catch-up must plan");

    assert_eq!(user_plan.class, ScheduleDecisionClass::ClaimRequired);
    assert_eq!(agent_plan.class, ScheduleDecisionClass::CoalescedSkip);
    assert_eq!(host_plan.class, ScheduleDecisionClass::CoalescedPause);
    assert_eq!(user_plan.missed_count, EXPECTED_MISSED_COUNT);
    assert_eq!(agent_plan.missed_count, EXPECTED_MISSED_COUNT);
    assert_eq!(host_plan.missed_count, EXPECTED_MISSED_COUNT);
}

#[test]
fn recovery_identity_is_stable_for_job_order() {
    // r[verify agent_harness_runtime.schedule_reliability.persistence.recovery]
    let first = state(vec![user_job(), agent_job(), host_job()]);
    let second = state(vec![host_job(), user_job(), agent_job()]);

    assert_eq!(first.identity(), second.identity());
    assert_eq!(first.jobs().len(), second.jobs().len());
}

#[test]
fn surface_ownership_rejects_agent_mutation_and_second_user_heartbeat() {
    // r[verify agent_harness_runtime.schedule_reliability.surfaces.cross_surface]
    let schedule_state = state(vec![user_job()]);
    let agent_mutation = ScheduleMutationRequest {
        session_id: SESSION_ID.to_string(),
        requester_id: AGENT_ID.to_string(),
        requester_class: ScheduleOwnerClass::Agent,
        job_id: USER_JOB_ID.to_string(),
        surface: ScheduleSurface::UserHeartbeat,
        action: ScheduleMutationAction::Clear,
    };
    let second_user_heartbeat = ScheduleMutationRequest {
        session_id: SESSION_ID.to_string(),
        requester_id: USER_ID.to_string(),
        requester_class: ScheduleOwnerClass::User,
        job_id: "job:second-heartbeat".to_string(),
        surface: ScheduleSurface::UserHeartbeat,
        action: ScheduleMutationAction::Create,
    };

    assert_eq!(
        admit_schedule_mutation(&schedule_state, &agent_mutation),
        Err(ScheduleError::OwnerSurfaceMismatch)
    );
    assert_eq!(
        admit_schedule_mutation(&schedule_state, &second_user_heartbeat),
        Err(ScheduleError::DuplicateUserHeartbeat)
    );
}

#[test]
fn cross_session_and_backlog_overflow_fail_closed() {
    // r[verify agent_harness_runtime.schedule_reliability.persistence.session_scoped]
    // r[verify agent_harness_runtime.schedule_reliability.coalesce.no_backlog]
    let mut cross_session = user_job();
    cross_session.session_id = OTHER_SESSION_ID.to_string();
    let mut narrow_policy = policy_draft();
    narrow_policy.maximum_missed_count = 1;
    let narrow_policy = SchedulePolicy::validate(&narrow_policy).expect("narrow policy");
    let schedule_state = state(vec![user_job()]);

    assert_eq!(
        ScheduleState::recover(&policy(), SESSION_ID, vec![cross_session]),
        Err(ScheduleError::CrossSession)
    );
    assert_eq!(
        evaluate_schedule_tick(&narrow_policy, &schedule_state, USER_JOB_ID, LATE_TICK),
        Err(ScheduleError::MissedCountExceeded)
    );
}

#[test]
fn receipt_projection_is_metadata_only_and_rejects_overclaims() {
    // r[verify agent_harness_runtime.schedule_reliability.evidence.safe]
    // r[verify agent_harness_runtime.schedule_reliability.evidence.rejects]
    let policy = policy();
    let schedule_state = state(vec![user_job()]);
    let receipt = project_schedule_receipt(
        &policy,
        ScheduleReceiptDraft {
            policy_identity: policy.identity().to_string(),
            state_identity: schedule_state.identity().to_string(),
            job_identity: USER_JOB_ID.to_string(),
            surface: ScheduleSurface::UserHeartbeat,
            decision_class: ScheduleDecisionClass::ClaimRequired,
            clock_identity: CLOCK_ID.to_string(),
            missed_count: 0,
            non_claims: required_non_claims(),
        },
    )
    .expect("safe receipt must project");

    assert_eq!(receipt.identity().len(), HASH_HEX_LENGTH);
    assert_eq!(receipt.missed_count(), 0);
    assert_eq!(
        validate_schedule_receipt_text("claim counts and identities"),
        Ok(())
    );
    assert_eq!(
        validate_schedule_receipt_text("prompt: run deployment"),
        Err(ScheduleError::UnsafeReceipt)
    );
    assert_eq!(
        validate_schedule_receipt_text("exactly-once delivery"),
        Err(ScheduleError::UnsafeReceipt)
    );
}

#[test]
fn stale_policy_missing_bounds_and_unsafe_receipt_fields_fail_closed() {
    // r[verify agent_harness_runtime.schedule_reliability.profile.invalid]
    let mut stale = policy_draft();
    stale.source_revision = "moving-tag".to_string();
    let mut missing_bound = policy_draft();
    missing_bound.minimum_interval_ticks = 0;
    let mut unsafe_fields = policy_draft();
    unsafe_fields.receipt_fields.push("prompt_body".to_string());

    assert_eq!(
        SchedulePolicy::validate(&stale),
        Err(ScheduleError::PolicyRejected)
    );
    assert_eq!(
        SchedulePolicy::validate(&missing_bound),
        Err(ScheduleError::PolicyRejected)
    );
    assert_eq!(
        SchedulePolicy::validate(&unsafe_fields),
        Err(ScheduleError::PolicyRejected)
    );
}
