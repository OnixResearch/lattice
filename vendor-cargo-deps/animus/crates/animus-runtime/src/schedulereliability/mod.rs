//! Pure schedule ownership, claim, coalescing, recovery, and receipt contracts.

use blake3::Hasher;

mod core;

#[cfg(test)]
mod tests;

const SCHEDULE_SCHEMA: &str = "animus.schedule-reliability.v1";
const SCHEDULE_DOMAIN: &str = "animus/schedule-reliability";
const POLICY_DOMAIN: &str = "animus/schedule-reliability-policy";
const STATE_DOMAIN: &str = "animus/schedule-reliability-state";
const RECEIPT_DOMAIN: &str = "animus/schedule-reliability-receipt";
const REVIEWED_COHORT: &str = "prime-agent-schedule-reliability";
const REVIEWED_REVISION: &str = "c5991bc853d27754aed345c13ff4d2e05c40f1f4";
const MIN_BOUND: u32 = 1;
const MAX_POLICY_JOB_COUNT: u32 = 4_096;
const MAX_IDENTITY_LENGTH: usize = 128;
const HASH_HEX_LENGTH: usize = 64;
const REQUIRED_RECEIPT_FIELDS: [&str; 8] = [
    "policy_identity",
    "state_identity",
    "job_identity",
    "surface",
    "decision_class",
    "clock_identity",
    "missed_count",
    "non_claims",
];
const REQUIRED_NON_CLAIMS: [&str; 5] = [
    "not-exactly-once-execution",
    "not-real-time",
    "not-timer-accuracy",
    "not-outcome-correctness",
    "not-production-or-release-readiness",
];
const FORBIDDEN_RECEIPT_MARKERS: [&str; 8] = [
    "prompt:",
    "message:",
    "tool output",
    "stdout",
    "stderr",
    "environment=",
    "/home/",
    "exactly-once",
];

/// Schedule surface with an explicit ownership class.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ScheduleSurface {
    /// One visible heartbeat controlled by the user.
    UserHeartbeat,
    /// Recurring prompt controlled by the agent for its current session.
    AgentRecurring,
    /// One-time or recurring job controlled by a user or host.
    Scheduled,
}

impl ScheduleSurface {
    /// Returns the stable surface label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserHeartbeat => "user_heartbeat",
            Self::AgentRecurring => "agent_recurring",
            Self::Scheduled => "scheduled",
        }
    }
}

/// Owner class used for schedule mutation admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleOwnerClass {
    /// Explicit user owner.
    User,
    /// Session-local agent owner.
    Agent,
    /// Host or automation owner.
    Host,
}

impl ScheduleOwnerClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Agent => "agent",
            Self::Host => "host",
        }
    }
}

/// Policy for a missed-tick catch-up decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatchUpRule {
    /// Claim one catch-up delivery and discard the remaining backlog.
    FireOnce,
    /// Record one skipped catch-up decision.
    Skip,
    /// Pause the job after one catch-up decision.
    Pause,
}

impl CatchUpRule {
    const fn as_str(self) -> &'static str {
        match self {
            Self::FireOnce => "fire_once",
            Self::Skip => "skip",
            Self::Pause => "pause",
        }
    }
}

/// Generated Nickel policy input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulePolicyDraft {
    /// Reviewed source cohort.
    pub cohort: String,
    /// Reviewed immutable revision.
    pub source_revision: String,
    /// Minimum recurring interval in logical ticks.
    pub minimum_interval_ticks: u64,
    /// Maximum recurring interval in logical ticks.
    pub maximum_interval_ticks: u64,
    /// Maximum jobs in one session.
    pub maximum_jobs_per_session: u32,
    /// Maximum missed count retained in metadata.
    pub maximum_missed_count: u64,
    /// User-heartbeat catch-up rule.
    pub user_heartbeat_catch_up: CatchUpRule,
    /// Agent-recurring catch-up rule.
    pub agent_recurring_catch_up: CatchUpRule,
    /// General-schedule catch-up rule.
    pub scheduled_catch_up: CatchUpRule,
    /// Safe receipt fields.
    pub receipt_fields: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
}

/// Revalidated schedule policy with a deterministic identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchedulePolicy {
    minimum_interval_ticks: u64,
    maximum_interval_ticks: u64,
    maximum_jobs_per_session: u32,
    maximum_missed_count: u64,
    catch_up_rules: [CatchUpRule; 3],
    receipt_fields: Vec<String>,
    non_claims: Vec<String>,
    identity: String,
}

impl SchedulePolicy {
    /// Revalidates generated policy and derives its BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns [`ScheduleError::PolicyRejected`] for stale or unsafe policy.
    pub fn validate(draft: &SchedulePolicyDraft) -> Result<Self, ScheduleError> {
        core::validate_policy_draft(draft)?;
        let identity = core::policy_identity(draft);
        debug_assert_eq!(identity.len(), HASH_HEX_LENGTH);
        Ok(Self {
            minimum_interval_ticks: draft.minimum_interval_ticks,
            maximum_interval_ticks: draft.maximum_interval_ticks,
            maximum_jobs_per_session: draft.maximum_jobs_per_session,
            maximum_missed_count: draft.maximum_missed_count,
            catch_up_rules: [
                draft.user_heartbeat_catch_up,
                draft.agent_recurring_catch_up,
                draft.scheduled_catch_up,
            ],
            receipt_fields: draft.receipt_fields.clone(),
            non_claims: draft.non_claims.clone(),
            identity,
        })
    }

    /// Returns the policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    fn catch_up_rule(&self, surface: ScheduleSurface) -> CatchUpRule {
        match surface {
            ScheduleSurface::UserHeartbeat => self.catch_up_rules[0],
            ScheduleSurface::AgentRecurring => self.catch_up_rules[1],
            ScheduleSurface::Scheduled => self.catch_up_rules[2],
        }
    }
}

/// Validated session-local schedule job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleJob {
    /// Session identity.
    pub session_id: String,
    /// Job identity.
    pub job_id: String,
    /// Owner identity.
    pub owner_id: String,
    /// Owner class.
    pub owner_class: ScheduleOwnerClass,
    /// Schedule surface.
    pub surface: ScheduleSurface,
    /// Recurrence interval in logical ticks.
    pub interval_ticks: u64,
    /// Next due logical tick.
    pub next_due_tick: u64,
    /// Due tick already claimed durably.
    pub claimed_tick: Option<u64>,
    /// Paused state.
    pub is_paused: bool,
}

/// Deterministically recovered session schedule state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleState {
    session_id: String,
    jobs: Vec<ScheduleJob>,
    identity: String,
}

impl ScheduleState {
    /// Recovers and validates durable jobs for one session.
    ///
    /// # Errors
    ///
    /// Returns a typed error for malformed, duplicated, cross-session, or
    /// over-bound state.
    pub fn recover(
        policy: &SchedulePolicy,
        session_id: &str,
        jobs: Vec<ScheduleJob>,
    ) -> Result<Self, ScheduleError> {
        core::validate_recovered_jobs(policy, session_id, jobs.as_slice())?;
        let identity = core::state_identity(session_id, jobs.as_slice());
        debug_assert_eq!(identity.len(), HASH_HEX_LENGTH);
        Ok(Self {
            session_id: session_id.to_string(),
            jobs,
            identity,
        })
    }

    /// Returns the session identity.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    /// Returns the durable jobs.
    #[must_use]
    pub fn jobs(&self) -> &[ScheduleJob] {
        self.jobs.as_slice()
    }

    /// Returns the deterministic durable-state identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }
}

/// Mutation action admitted by the pure ownership core.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleMutationAction {
    /// Create a job.
    Create,
    /// Update a job.
    Update,
    /// Pause a job.
    Pause,
    /// Clear a job.
    Clear,
}

/// Mutation request over caller-supplied durable state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleMutationRequest {
    /// Requesting session.
    pub session_id: String,
    /// Requesting owner identity.
    pub requester_id: String,
    /// Requesting owner class.
    pub requester_class: ScheduleOwnerClass,
    /// Target job identity.
    pub job_id: String,
    /// Target surface.
    pub surface: ScheduleSurface,
    /// Mutation action.
    pub action: ScheduleMutationAction,
}

/// Admitted mutation description for a host-owned durable store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleMutationPlan {
    /// Target job identity.
    pub job_id: String,
    /// Admitted action.
    pub action: ScheduleMutationAction,
    /// Target surface.
    pub surface: ScheduleSurface,
}

/// Due-tick decision class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleDecisionClass {
    /// The job is not due.
    NotDue,
    /// Recovery found a durable claim without a completed delivery.
    ClaimedUndelivered,
    /// The host must persist a claim before delivery admission.
    ClaimRequired,
    /// Missed ticks coalesced into one skipped decision.
    CoalescedSkip,
    /// Missed ticks coalesced into one pause decision.
    CoalescedPause,
}

impl ScheduleDecisionClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NotDue => "not_due",
            Self::ClaimedUndelivered => "claimed_undelivered",
            Self::ClaimRequired => "claim_required",
            Self::CoalescedSkip => "coalesced_skip",
            Self::CoalescedPause => "coalesced_pause",
        }
    }
}

/// Durable claim transition that must occur before prompt delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleClaimTransition {
    /// Job identity.
    pub job_id: String,
    /// Due tick being claimed.
    pub due_tick: u64,
    /// State identity that the host must replace atomically.
    pub prior_state_identity: String,
    /// Deterministic claim identity.
    pub claim_identity: String,
}

/// Pure due-tick plan. It never performs persistence or delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleTickPlan {
    /// Decision class.
    pub class: ScheduleDecisionClass,
    /// Number of older due ticks coalesced into this decision.
    pub missed_count: u64,
    /// Durable claim required before delivery, when applicable.
    pub claim: Option<ScheduleClaimTransition>,
}

/// Proof from the host that the exact claim became durable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableClaimProof {
    /// Claim identity written to durable state.
    pub claim_identity: String,
    /// New durable state identity.
    pub durable_state_identity: String,
}

/// Delivery instruction admitted only after a matching durable claim proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleDeliveryInstruction {
    /// Job identity.
    pub job_id: String,
    /// Claimed due tick.
    pub due_tick: u64,
    /// Durable claim identity.
    pub claim_identity: String,
}

/// Safe schedule receipt input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleReceiptDraft {
    /// Policy identity.
    pub policy_identity: String,
    /// Durable state identity.
    pub state_identity: String,
    /// Job identity.
    pub job_identity: String,
    /// Schedule surface.
    pub surface: ScheduleSurface,
    /// Decision class.
    pub decision_class: ScheduleDecisionClass,
    /// Host-injected clock identity.
    pub clock_identity: String,
    /// Missed-tick count.
    pub missed_count: u64,
    /// Required non-claims.
    pub non_claims: Vec<String>,
}

/// Metadata-only schedule receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScheduleReceipt {
    draft: ScheduleReceiptDraft,
    identity: String,
}

impl ScheduleReceipt {
    /// Returns the receipt identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns the recorded missed count.
    #[must_use]
    pub const fn missed_count(&self) -> u64 {
        self.draft.missed_count
    }
}

/// Schedule contract error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleError {
    /// Generated policy is stale, incomplete, or unsafe.
    PolicyRejected,
    /// A stable identity is empty or malformed.
    InvalidIdentity,
    /// A job interval is outside policy bounds.
    InvalidInterval,
    /// A session contains too many jobs.
    JobLimitExceeded,
    /// Two jobs use the same identity.
    DuplicateJob,
    /// More than one visible user heartbeat exists.
    DuplicateUserHeartbeat,
    /// A job targets another session.
    CrossSession,
    /// The owner class does not match the selected surface.
    OwnerSurfaceMismatch,
    /// A caller tried to mutate another owner or protected surface.
    MutationDenied,
    /// The missed count exceeds the bounded policy field.
    MissedCountExceeded,
    /// Delivery lacks an exact durable claim proof.
    ClaimNotDurable,
    /// Receipt metadata contains unsafe or unsupported content.
    UnsafeReceipt,
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    debug_assert!(!domain.is_empty());
    debug_assert!(!parts.is_empty());
    let mut hasher = Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&[0]);
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn is_safe_identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_IDENTITY_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

fn has_required_values(actual: &[String], required: &[&str]) -> bool {
    actual.len() == required.len()
        && required
            .iter()
            .all(|required_value| actual.iter().any(|value| value == required_value))
}

pub use core::{
    admit_delivery_after_claim, admit_schedule_mutation, evaluate_schedule_tick,
    project_schedule_receipt, validate_schedule_receipt_text,
};
