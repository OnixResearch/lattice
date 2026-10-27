//! Pure persistent-goal state, budget, ownership, and receipt contracts.

use blake3::Hasher;

const GOAL_SCHEMA: &str = "animus.persistent-goals.v1";
const GOAL_DOMAIN: &str = "animus/persistent-goal";
const GOAL_POLICY_DOMAIN: &str = "animus/persistent-goal-policy";
const REVIEWED_COHORT: &str = "prime-agent-persistent-goals";
const REVIEWED_REVISION: &str = "c5991bc853d27754aed345c13ff4d2e05c40f1f4";
const REQUIRED_NON_CLAIMS: [&str; 4] = [
    "not-objective-achievement",
    "not-work-quality",
    "not-automatic-inference",
    "not-production-or-release-readiness",
];
const REQUIRED_RECEIPT_FIELDS: [&str; 7] = [
    "policy_identity",
    "transition",
    "status",
    "budget_class",
    "token_counter_identity",
    "elapsed_counter_identity",
    "continuation_counter_identity",
];
const MAX_OBJECTIVE_IDENTITY_LENGTH: usize = 128;
const MIN_POLICY_LIMIT: u32 = 1;
const STATUS_COUNT: usize = 6;
const HASH_HEX_LENGTH: usize = 64;

/// Goal lifecycle status. Only [`Self::Completed`] is success.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalStatus {
    /// Goal accepts ordinary turns and completion requests.
    Active,
    /// Goal is retained but does not request continuation.
    Paused,
    /// Goal ended through an admitted explicit completion.
    Completed,
    /// Goal stopped because a configured budget was exhausted.
    BudgetLimited,
    /// Goal stopped because the host reported an error.
    Errored,
    /// Goal was explicitly removed by a user or host action.
    Cleared,
}

impl GoalStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::BudgetLimited => "budget_limited",
            Self::Errored => "errored",
            Self::Cleared => "cleared",
        }
    }

    /// Returns true only for an active goal.
    #[must_use]
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Active)
    }

    /// Returns true only for explicit successful completion.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Budget dimension that can stop continuation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalBudgetClass {
    /// Token counter budget.
    Tokens,
    /// Elapsed-time counter budget.
    Elapsed,
    /// Continuation-count budget.
    Continuations,
}

impl GoalBudgetClass {
    /// Returns the stable budget label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tokens => "tokens",
            Self::Elapsed => "elapsed",
            Self::Continuations => "continuations",
        }
    }
}

/// Origin that is allowed to create or control a goal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalProvenance {
    /// Explicit user action.
    User,
    /// Explicit host action.
    Host,
    /// Model action, which is never admitted for lifecycle control.
    Model,
    /// Inferred from an ordinary task, which is never admitted.
    Inferred,
}

impl GoalProvenance {
    /// Returns the stable provenance label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Host => "host",
            Self::Model => "model",
            Self::Inferred => "inferred",
        }
    }

    const fn is_explicit(self) -> bool {
        matches!(self, Self::User | Self::Host)
    }
}

/// Lifecycle action applied by an explicit user or host request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalAction {
    /// Pause an active goal.
    Pause,
    /// Resume a paused goal.
    Resume,
    /// Explicitly complete an active goal.
    Complete,
    /// Explicitly clear a goal.
    Clear,
    /// Stop an active goal due to a host-reported error.
    Error,
    /// Stop an active goal because a budget was reached.
    BudgetExhausted(GoalBudgetClass),
}

impl GoalAction {
    /// Returns the stable action label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Complete => "complete",
            Self::Clear => "clear",
            Self::Error => "error",
            Self::BudgetExhausted(_) => "budget_exhausted",
        }
    }
}

/// Optional explicit budget limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoalBudget {
    /// Maximum cumulative tokens.
    pub tokens: Option<u32>,
    /// Maximum cumulative elapsed units supplied by the host.
    pub elapsed: Option<u32>,
    /// Maximum continuation count.
    pub continuations: Option<u32>,
}

impl GoalBudget {
    /// Creates a budget and rejects zero limits.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError::InvalidBudget`] when a present limit is zero.
    pub fn new(
        token_limit: Option<u32>,
        elapsed_limit: Option<u32>,
        continuation_limit: Option<u32>,
    ) -> Result<Self, GoalError> {
        if token_limit == Some(0) || elapsed_limit == Some(0) || continuation_limit == Some(0) {
            return Err(GoalError::InvalidBudget);
        }
        Ok(Self {
            tokens: token_limit,
            elapsed: elapsed_limit,
            continuations: continuation_limit,
        })
    }
}

/// Host-injected accounting counters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoalCounters {
    /// Stable token counter identity.
    pub token_counter_identity: &'static str,
    /// Stable elapsed counter identity.
    pub elapsed_counter_identity: &'static str,
    /// Stable continuation counter identity.
    pub continuation_counter_identity: &'static str,
    /// Tokens consumed by the latest turn.
    pub token_delta: u32,
    /// Elapsed units consumed by the latest turn.
    pub elapsed_delta: u32,
}

impl GoalCounters {
    /// Validates host-provided counter identities and deltas.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError::InvalidCounters`] when an identity is empty or a
    /// delta would overflow the bounded accounting domain.
    pub const fn validate(self) -> Result<Self, GoalError> {
        if self.token_counter_identity.is_empty()
            || self.elapsed_counter_identity.is_empty()
            || self.continuation_counter_identity.is_empty()
        {
            return Err(GoalError::InvalidCounters);
        }
        Ok(self)
    }
}

/// Policy draft supplied by generated Nickel data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalPolicyDraft {
    /// Reviewed source cohort.
    pub cohort: String,
    /// Reviewed source revision.
    pub source_revision: String,
    /// Maximum objective identity length.
    pub max_objective_identity_length: usize,
    /// Minimum admitted budget limit.
    pub minimum_budget_limit: u32,
    /// Receipt metadata fields.
    pub receipt_fields: Vec<String>,
    /// Required non-claims.
    pub non_claims: Vec<String>,
}

/// Validated goal policy with deterministic identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalPolicy {
    cohort: String,
    source_revision: String,
    max_objective_identity_length: usize,
    minimum_budget_limit: u32,
    receipt_fields: Vec<String>,
    non_claims: Vec<String>,
    identity: String,
}

impl GoalPolicy {
    /// Validates a generated goal policy and derives its BLAKE3 identity.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError::PolicyRejected`] when the cohort, limits, fields,
    /// or required non-claims do not match the reviewed policy.
    pub fn new(draft: GoalPolicyDraft) -> Result<Self, GoalError> {
        if draft.cohort != REVIEWED_COHORT
            || draft.source_revision != REVIEWED_REVISION
            || draft.max_objective_identity_length == 0
            || draft.minimum_budget_limit < MIN_POLICY_LIMIT
            || draft.receipt_fields.len() < REQUIRED_RECEIPT_FIELDS.len()
            || REQUIRED_RECEIPT_FIELDS.iter().any(|field| {
                !draft
                    .receipt_fields
                    .iter()
                    .any(|candidate| candidate == field)
            })
            || REQUIRED_NON_CLAIMS
                .iter()
                .any(|claim| !draft.non_claims.iter().any(|candidate| candidate == claim))
        {
            return Err(GoalError::PolicyRejected);
        }
        let identity = policy_identity(&draft);
        Ok(Self {
            cohort: draft.cohort,
            source_revision: draft.source_revision,
            max_objective_identity_length: draft.max_objective_identity_length,
            minimum_budget_limit: draft.minimum_budget_limit,
            receipt_fields: draft.receipt_fields,
            non_claims: draft.non_claims,
            identity,
        })
    }

    /// Returns the policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Returns the reviewed cohort.
    #[must_use]
    pub fn cohort(&self) -> &str {
        &self.cohort
    }

    /// Returns the maximum objective identity length.
    #[must_use]
    pub const fn max_objective_identity_length(&self) -> usize {
        self.max_objective_identity_length
    }

    /// Returns the minimum accepted budget limit.
    #[must_use]
    pub const fn minimum_budget_limit(&self) -> u32 {
        self.minimum_budget_limit
    }

    /// Returns true when a receipt field is admitted.
    #[must_use]
    pub fn admits_receipt_field(&self, field: &str) -> bool {
        self.receipt_fields
            .iter()
            .any(|candidate| candidate == field)
    }

    /// Returns true when a required non-claim is present.
    #[must_use]
    pub fn admits_non_claim(&self, claim: &str) -> bool {
        self.non_claims.iter().any(|candidate| candidate == claim)
    }
}

/// Returns the checked-in reviewed goal policy.
///
/// # Panics
///
/// Panics only when the checked-in reviewed policy constants are inconsistent.
#[must_use]
pub fn reviewed_goal_policy() -> GoalPolicy {
    GoalPolicy::new(GoalPolicyDraft {
        cohort: REVIEWED_COHORT.into(),
        source_revision: REVIEWED_REVISION.into(),
        max_objective_identity_length: MAX_OBJECTIVE_IDENTITY_LENGTH,
        minimum_budget_limit: MIN_POLICY_LIMIT,
        receipt_fields: REQUIRED_RECEIPT_FIELDS
            .iter()
            .map(|field| (*field).into())
            .collect(),
        non_claims: REQUIRED_NON_CLAIMS
            .iter()
            .map(|claim| (*claim).into())
            .collect(),
    })
    .expect("reviewed persistent-goal policy is valid")
}

/// Goal state with metadata-only objective identity and accounting.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalState {
    objective_identity: String,
    status: GoalStatus,
    provenance: GoalProvenance,
    token_count: u32,
    elapsed_count: u32,
    continuation_count: u32,
    budget: GoalBudget,
    last_budget_class: Option<GoalBudgetClass>,
}

impl GoalState {
    /// Creates an active goal from an explicit user or host action.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError::OwnershipRejected`] for model or inferred
    /// provenance, or a validation error for unsafe identities and budgets.
    pub fn create(
        objective_identity: &str,
        provenance: GoalProvenance,
        budget: GoalBudget,
        policy: &GoalPolicy,
    ) -> Result<Self, GoalError> {
        if !provenance.is_explicit() {
            return Err(GoalError::OwnershipRejected);
        }
        if objective_identity.is_empty()
            || objective_identity.len() > policy.max_objective_identity_length
            || objective_identity.len() > MAX_OBJECTIVE_IDENTITY_LENGTH
        {
            return Err(GoalError::InvalidObjectiveIdentity);
        }
        if budget
            .tokens
            .is_some_and(|limit| limit < policy.minimum_budget_limit)
            || budget
                .elapsed
                .is_some_and(|limit| limit < policy.minimum_budget_limit)
            || budget
                .continuations
                .is_some_and(|limit| limit < policy.minimum_budget_limit)
        {
            return Err(GoalError::InvalidBudget);
        }
        Ok(Self {
            objective_identity: objective_identity.into(),
            status: GoalStatus::Active,
            provenance,
            token_count: 0,
            elapsed_count: 0,
            continuation_count: 0,
            budget,
            last_budget_class: None,
        })
    }

    /// Returns the metadata-only objective identity.
    #[must_use]
    pub fn objective_identity(&self) -> &str {
        &self.objective_identity
    }

    /// Returns the current lifecycle status.
    #[must_use]
    pub const fn status(&self) -> GoalStatus {
        self.status
    }

    /// Returns the creation provenance.
    #[must_use]
    pub const fn provenance(&self) -> GoalProvenance {
        self.provenance
    }

    /// Returns cumulative token accounting.
    #[must_use]
    pub const fn token_count(&self) -> u32 {
        self.token_count
    }

    /// Returns cumulative elapsed accounting.
    #[must_use]
    pub const fn elapsed_count(&self) -> u32 {
        self.elapsed_count
    }

    /// Returns cumulative continuation accounting.
    #[must_use]
    pub const fn continuation_count(&self) -> u32 {
        self.continuation_count
    }

    /// Returns the budget class that stopped the goal, if any.
    #[must_use]
    pub const fn last_budget_class(&self) -> Option<GoalBudgetClass> {
        self.last_budget_class
    }

    /// Applies a pure lifecycle transition with supplied accounting.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError::InvalidTransition`] for an invalid status change,
    /// [`GoalError::InvalidCounters`] for bad host counters, or a budget error
    /// when accounting exceeds the configured domain.
    pub fn transition(
        &self,
        action: GoalAction,
        counters: GoalCounters,
    ) -> Result<Self, GoalError> {
        counters.validate()?;
        let mut next = self.clone();
        if self.status == GoalStatus::Active {
            next.token_count = self
                .token_count
                .checked_add(counters.token_delta)
                .ok_or(GoalError::AccountingOverflow)?;
            next.elapsed_count = self
                .elapsed_count
                .checked_add(counters.elapsed_delta)
                .ok_or(GoalError::AccountingOverflow)?;
            next.continuation_count = self
                .continuation_count
                .checked_add(1)
                .ok_or(GoalError::AccountingOverflow)?;
        }
        if self.status == GoalStatus::Active && budget_exhausted(&next) {
            return Ok(next.with_budget_limit());
        }
        match action {
            GoalAction::Pause if self.status == GoalStatus::Active => {
                next.status = GoalStatus::Paused;
            }
            GoalAction::Resume if self.status == GoalStatus::Paused => {
                next.status = GoalStatus::Active;
            }
            GoalAction::Complete if self.status == GoalStatus::Active => {
                next.status = GoalStatus::Completed;
            }
            GoalAction::Clear if matches!(self.status, GoalStatus::Active | GoalStatus::Paused) => {
                next.status = GoalStatus::Cleared;
            }
            GoalAction::Error if self.status == GoalStatus::Active => {
                next.status = GoalStatus::Errored;
            }
            GoalAction::BudgetExhausted(class) if self.status == GoalStatus::Active => {
                next.status = GoalStatus::BudgetLimited;
                next.last_budget_class = Some(class);
            }
            _ => return Err(GoalError::InvalidTransition),
        }
        Ok(next)
    }

    fn with_budget_limit(mut self) -> Self {
        self.status = GoalStatus::BudgetLimited;
        self.last_budget_class = budget_class(&self);
        self
    }
}

/// Pure per-session one-active-goal container.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GoalSession {
    active: Option<GoalState>,
}

impl GoalSession {
    /// Creates an empty session.
    #[must_use]
    pub const fn new() -> Self {
        Self { active: None }
    }

    /// Admits one explicit goal and rejects a second active goal.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError::ActiveGoalExists`] when this session already has
    /// an active goal, or forwards goal validation errors.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &self,
        objective_identity: &str,
        provenance: GoalProvenance,
        budget: GoalBudget,
        policy: &GoalPolicy,
    ) -> Result<Self, GoalError> {
        if self
            .active
            .as_ref()
            .is_some_and(|goal| goal.status.is_active())
        {
            return Err(GoalError::ActiveGoalExists);
        }
        let goal = GoalState::create(objective_identity, provenance, budget, policy)?;
        Ok(Self { active: Some(goal) })
    }

    /// Returns the current goal, if one is retained.
    #[must_use]
    pub const fn goal(&self) -> Option<&GoalState> {
        self.active.as_ref()
    }

    /// Applies an action to the retained goal.
    ///
    /// # Errors
    ///
    /// Returns [`GoalError::NoActiveGoal`] when no goal exists, or a goal
    /// transition error.
    pub fn transition(
        &self,
        action: GoalAction,
        counters: GoalCounters,
    ) -> Result<Self, GoalError> {
        let goal = self.active.as_ref().ok_or(GoalError::NoActiveGoal)?;
        Ok(Self {
            active: Some(goal.transition(action, counters)?),
        })
    }
}

/// Goal-aware continuation result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalContinuation {
    /// Continue the active goal after an ordinary turn.
    Continue,
    /// Do not continue because no active goal exists.
    Idle,
    /// Do not continue because the goal stopped for a non-success reason.
    Stopped(GoalStatus),
}

/// Evaluates continuation without injecting prompts or accessing a clock.
#[must_use]
pub const fn evaluate_goal_continuation(goal: Option<&GoalState>) -> GoalContinuation {
    match goal {
        Some(goal) if goal.status().is_active() => GoalContinuation::Continue,
        Some(goal) => GoalContinuation::Stopped(goal.status()),
        None => GoalContinuation::Idle,
    }
}

/// Host attestation for an explicit completion or lifecycle event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoalAttestation {
    /// Stable host event identity.
    pub event_identity: &'static str,
    /// Provenance of the explicit action.
    pub provenance: GoalProvenance,
}

/// Metadata-only goal receipt draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoalReceiptDraft {
    /// Goal state to project.
    pub goal: GoalStatus,
    /// Transition label.
    pub transition: &'static str,
    /// Budget class, if the goal was budget-limited.
    pub budget_class: Option<GoalBudgetClass>,
    /// Host accounting identities.
    pub counters: GoalCounters,
    /// Explicit host or user attestation.
    pub attestation: GoalAttestation,
    /// Forbidden objective text or success overclaim marker.
    pub contains_objective_text: bool,
    /// Claim that the goal succeeded regardless of status.
    pub claims_success: bool,
}

/// Safe metadata-only goal receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoalReceipt {
    policy_identity: String,
    transition: String,
    status: GoalStatus,
    budget_class: Option<GoalBudgetClass>,
    token_counter_identity: String,
    elapsed_counter_identity: String,
    continuation_counter_identity: String,
    identity: String,
}

impl GoalReceipt {
    /// Returns the policy identity.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        &self.policy_identity
    }

    /// Returns the status.
    #[must_use]
    pub const fn status(&self) -> GoalStatus {
        self.status
    }

    /// Returns the receipt identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }
}

/// Projects a safe receipt without objective text or success overclaims.
///
/// # Errors
///
/// Returns [`GoalError::ReceiptRejected`] when the draft contains content,
/// lacks explicit provenance, or claims success for a non-completed goal.
pub fn project_goal_receipt(
    policy: &GoalPolicy,
    draft: &GoalReceiptDraft,
) -> Result<GoalReceipt, GoalError> {
    draft.counters.validate()?;
    if draft.contains_objective_text
        || draft.attestation.event_identity.is_empty()
        || !draft.attestation.provenance.is_explicit()
        || (draft.claims_success && !draft.goal.is_success())
        || (draft.goal == GoalStatus::BudgetLimited && draft.budget_class.is_none())
    {
        return Err(GoalError::ReceiptRejected);
    }
    let identity = receipt_identity(policy, draft);
    Ok(GoalReceipt {
        policy_identity: policy.identity.clone(),
        transition: draft.transition.into(),
        status: draft.goal,
        budget_class: draft.budget_class,
        token_counter_identity: draft.counters.token_counter_identity.into(),
        elapsed_counter_identity: draft.counters.elapsed_counter_identity.into(),
        continuation_counter_identity: draft.counters.continuation_counter_identity.into(),
        identity,
    })
}

/// Goal contract failures. No variant contains objective content.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoalError {
    /// Policy does not match the reviewed bounded profile.
    PolicyRejected,
    /// Goal creation or control came from an unadmitted provenance.
    OwnershipRejected,
    /// Objective identity is absent or too long.
    InvalidObjectiveIdentity,
    /// Budget is zero or below the policy minimum.
    InvalidBudget,
    /// Host counter identity or delta is invalid.
    InvalidCounters,
    /// Accounting would overflow.
    AccountingOverflow,
    /// Requested transition is not valid for the current status.
    InvalidTransition,
    /// A session already has an active goal.
    ActiveGoalExists,
    /// No goal is retained by the session.
    NoActiveGoal,
    /// Receipt content or claim boundary is unsafe.
    ReceiptRejected,
}

impl core::fmt::Display for GoalError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::PolicyRejected => "policy rejected",
            Self::OwnershipRejected => "goal ownership rejected",
            Self::InvalidObjectiveIdentity => "objective identity invalid",
            Self::InvalidBudget => "goal budget invalid",
            Self::InvalidCounters => "goal counters invalid",
            Self::AccountingOverflow => "goal accounting overflow",
            Self::InvalidTransition => "goal transition invalid",
            Self::ActiveGoalExists => "active goal exists",
            Self::NoActiveGoal => "no active goal",
            Self::ReceiptRejected => "goal receipt rejected",
        })
    }
}

fn budget_exhausted(goal: &GoalState) -> bool {
    goal.budget
        .tokens
        .is_some_and(|limit| goal.token_count >= limit)
        || goal
            .budget
            .elapsed
            .is_some_and(|limit| goal.elapsed_count >= limit)
        || goal
            .budget
            .continuations
            .is_some_and(|limit| goal.continuation_count >= limit)
}

fn budget_class(goal: &GoalState) -> Option<GoalBudgetClass> {
    if goal
        .budget
        .tokens
        .is_some_and(|limit| goal.token_count >= limit)
    {
        Some(GoalBudgetClass::Tokens)
    } else if goal
        .budget
        .elapsed
        .is_some_and(|limit| goal.elapsed_count >= limit)
    {
        Some(GoalBudgetClass::Elapsed)
    } else if goal
        .budget
        .continuations
        .is_some_and(|limit| goal.continuation_count >= limit)
    {
        Some(GoalBudgetClass::Continuations)
    } else {
        None
    }
}

fn policy_identity(draft: &GoalPolicyDraft) -> String {
    let material = format!(
        "{GOAL_SCHEMA}|{}|{}|{}|{}|{}|{}|{}",
        draft.cohort,
        draft.source_revision,
        draft.max_objective_identity_length,
        draft.minimum_budget_limit,
        draft.receipt_fields.join(","),
        draft.non_claims.join(","),
        STATUS_COUNT,
    );
    digest(GOAL_POLICY_DOMAIN, material.as_bytes())
}

fn receipt_identity(policy: &GoalPolicy, draft: &GoalReceiptDraft) -> String {
    let budget = draft.budget_class.map_or("none", GoalBudgetClass::as_str);
    let material = format!(
        "{GOAL_SCHEMA}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        policy.identity,
        draft.transition,
        draft.goal.as_str(),
        budget,
        draft.counters.token_counter_identity,
        draft.counters.elapsed_counter_identity,
        draft.counters.continuation_counter_identity,
        draft.counters.token_delta,
        draft.counters.elapsed_delta,
    );
    digest(GOAL_DOMAIN, material.as_bytes())
}

fn digest(domain: &str, material: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(material);
    let digest = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(digest.len(), HASH_HEX_LENGTH);
    digest
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN_COUNTER: &str = "tokens/session";
    const ELAPSED_COUNTER: &str = "elapsed/session";
    const CONTINUATION_COUNTER: &str = "continuations/session";
    const EVENT: &str = "host-event-1";

    fn policy() -> GoalPolicy {
        reviewed_goal_policy()
    }

    fn counters(token_delta: u32, elapsed_delta: u32) -> GoalCounters {
        GoalCounters {
            token_counter_identity: TOKEN_COUNTER,
            elapsed_counter_identity: ELAPSED_COUNTER,
            continuation_counter_identity: CONTINUATION_COUNTER,
            token_delta,
            elapsed_delta,
        }
    }

    fn budget() -> GoalBudget {
        GoalBudget::new(Some(10), Some(10), Some(3)).expect("budget")
    }

    #[test]
    fn lifecycle_accounting_and_explicit_completion_are_distinct() {
        let goal =
            GoalState::create("goal-id", GoalProvenance::Host, budget(), &policy()).expect("goal");
        let paused = goal
            .transition(GoalAction::Pause, counters(2, 1))
            .expect("pause");
        assert_eq!(paused.status(), GoalStatus::Paused);
        assert_eq!(paused.token_count(), 2);
        let active = paused
            .transition(GoalAction::Resume, counters(0, 0))
            .expect("resume");
        let complete = active
            .transition(GoalAction::Complete, counters(2, 1))
            .expect("complete");
        assert!(complete.status().is_success());
        assert_eq!(
            evaluate_goal_continuation(Some(&complete)),
            GoalContinuation::Stopped(GoalStatus::Completed)
        );
    }

    #[test]
    fn budget_exhaustion_is_not_success_and_requests_no_continuation() {
        let goal = GoalState::create(
            "goal-id",
            GoalProvenance::User,
            GoalBudget::new(Some(2), None, None).expect("budget"),
            &policy(),
        )
        .expect("goal");
        let limited = goal
            .transition(GoalAction::Pause, counters(2, 0))
            .expect("budget stop");
        assert_eq!(limited.status(), GoalStatus::BudgetLimited);
        assert!(!limited.status().is_success());
        assert_eq!(limited.last_budget_class(), Some(GoalBudgetClass::Tokens));
    }

    #[test]
    fn receipt_is_metadata_only_and_requires_explicit_attestation() {
        let goal =
            GoalState::create("goal-id", GoalProvenance::Host, budget(), &policy()).expect("goal");
        let receipt = project_goal_receipt(
            &policy(),
            &GoalReceiptDraft {
                goal: goal.status(),
                transition: "create",
                budget_class: None,
                counters: counters(0, 0),
                attestation: GoalAttestation {
                    event_identity: EVENT,
                    provenance: GoalProvenance::Host,
                },
                contains_objective_text: false,
                claims_success: false,
            },
        )
        .expect("receipt");
        assert_eq!(receipt.status(), GoalStatus::Active);
        assert_eq!(receipt.identity().len(), HASH_HEX_LENGTH);
    }

    #[test]
    fn invalid_ownership_transitions_and_success_overclaims_fail_closed() {
        assert_eq!(
            GoalState::create("goal-id", GoalProvenance::Model, budget(), &policy()),
            Err(GoalError::OwnershipRejected)
        );
        let session = GoalSession::new();
        let session = session
            .create("goal-id", GoalProvenance::User, budget(), &policy())
            .expect("first goal");
        assert_eq!(
            session.create("other", GoalProvenance::Host, budget(), &policy()),
            Err(GoalError::ActiveGoalExists)
        );
        let goal = session.goal().expect("goal");
        assert_eq!(
            project_goal_receipt(
                &policy(),
                &GoalReceiptDraft {
                    goal: GoalStatus::BudgetLimited,
                    transition: "stop",
                    budget_class: None,
                    counters: counters(0, 0),
                    attestation: GoalAttestation {
                        event_identity: EVENT,
                        provenance: GoalProvenance::Host,
                    },
                    contains_objective_text: true,
                    claims_success: true,
                },
            ),
            Err(GoalError::ReceiptRejected)
        );
        assert_eq!(goal.status(), GoalStatus::Active);
    }
}
