//! Pure multidimensional orchestration budget reservation and settlement ledger.
//!
//! This crate owns the deterministic, side-effect-free decisions behind a bounded
//! orchestration budget: named enforced dimensions, limits, planned charges,
//! reservations, consumed totals, breach state, checked arithmetic, reserve,
//! settle, and remaining-balance projection.
//!
//! The crate is intentionally free of store, filesystem, environment, clock,
//! process, network, provider, CLI, event-hashing, and product-authority
//! concerns. Callers supply every policy value, planned charge, reservation
//! identity, and observed charge. No decision is inferred from ambient state.
//!
//! The ledger classifies supplied integer accounting facts only. It does not
//! prove provider usage, billing, elapsed time, resource availability, fairness,
//! work completion, budget-policy correctness, or release readiness.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerError {
    /// An enforced dimension was admitted without an upper bound.
    MissingBudgetBound { label: &'static str },
    /// A committed or requested total exceeds an enforced limit.
    BudgetExceeded { label: &'static str },
    /// Checked addition overflowed a dimension.
    BudgetOverflow { label: &'static str },
    /// A reservation identity was already admitted.
    DuplicateReservation,
    /// Settlement does not match a pending reservation.
    MissingReservation,
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerError::MissingBudgetBound { label } => {
                write!(f, "enforced {label} budget has no admitted upper bound")
            }
            LedgerError::BudgetExceeded { label } => write!(f, "{label} budget is exhausted"),
            LedgerError::BudgetOverflow { label } => write!(f, "{label} budget accounting overflowed"),
            LedgerError::DuplicateReservation => write!(f, "budget reservation identity was already admitted"),
            LedgerError::MissingReservation => write!(f, "budget settlement does not match a pending reservation"),
        }
    }
}

impl std::error::Error for LedgerError {}

/// Which named dimensions the policy enforces.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetDimensions {
    pub rounds: bool,
    pub agent_invocations: bool,
    pub tokens: bool,
    pub monetary_microunits: bool,
    pub duration_millis: bool,
}

/// A fully materialized charge across the named dimensions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetCharge {
    pub rounds: u64,
    pub agent_invocations: u64,
    pub tokens: u64,
    pub monetary_microunits: u64,
    pub duration_millis: u64,
}

/// An optional planned charge prior to admission.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetChargePlan {
    pub rounds: Option<u64>,
    pub agent_invocations: Option<u64>,
    pub tokens: Option<u64>,
    pub monetary_microunits: Option<u64>,
    pub duration_millis: Option<u64>,
}

/// Hard per-dimension limits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetLimits {
    pub rounds: u64,
    pub agent_invocations: u64,
    pub tokens: u64,
    pub monetary_microunits: u64,
    pub duration_millis: u64,
}

/// The reviewed budget policy: enforced dimensions, hard limits, and the
/// conservative per-round upper bound.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetPolicy {
    pub enforced: BudgetDimensions,
    pub limits: BudgetLimits,
    pub per_round_upper_bound: BudgetCharge,
}

/// One admitted reservation binding an identity to a materialized charge.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BudgetReservation<Identity = String> {
    pub reservation_id: Identity,
    pub charge: BudgetCharge,
}

/// The durable reserve-and-settle ledger state.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BudgetLedger<Identity: Ord = String> {
    pub policy: BudgetPolicy,
    pub consumed: BudgetCharge,
    pub reservations: BTreeMap<Identity, BudgetReservation<Identity>>,
    pub breached: bool,
}

impl<Identity: Ord> Default for BudgetLedger<Identity> {
    fn default() -> Self {
        Self {
            policy: BudgetPolicy::default(),
            consumed: BudgetCharge::default(),
            reservations: BTreeMap::new(),
            breached: false,
        }
    }
}

/// Outcome of a settlement decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetSettlement {
    pub breached: bool,
}

/// A named dimension with its projection and mutator pair.
type DimensionProjection = (&'static str, fn(&BudgetCharge) -> u64, fn(&mut BudgetCharge, u64));

const DIMENSIONS: [DimensionProjection; 5] = [
    ("rounds", |c| c.rounds, |c, v| c.rounds = v),
    ("agent invocations", |c| c.agent_invocations, |c, v| c.agent_invocations = v),
    ("tokens", |c| c.tokens, |c, v| c.tokens = v),
    ("monetary microunits", |c| c.monetary_microunits, |c, v| c.monetary_microunits = v),
    ("duration milliseconds", |c| c.duration_millis, |c, v| c.duration_millis = v),
];

// r[impl lattice.budget_ledger_incubation.boundary]
/// Reserve a planned charge under a policy, failing closed on any admission
/// violation. Returns the materialized charge on success.
pub fn reserve_budget(
    ledger: &mut BudgetLedger,
    reservation_id: &str,
    plan: BudgetChargePlan,
) -> Result<BudgetCharge, LedgerError> {
    reserve_budget_for(ledger, reservation_id.to_owned(), plan)
}

/// Reserve a planned charge under a caller-owned nominal identity type.
pub fn reserve_budget_for<Identity: Clone + Ord>(
    ledger: &mut BudgetLedger<Identity>,
    reservation_id: Identity,
    plan: BudgetChargePlan,
) -> Result<BudgetCharge, LedgerError> {
    if ledger.reservations.contains_key(&reservation_id) {
        return Err(LedgerError::DuplicateReservation);
    }
    let charge = admitted_charge(&ledger.policy, plan)?;
    let reserved = total_reserved(&ledger.reservations)?;
    let committed = checked_add_charge(ledger.consumed, reserved)?;
    let requested = checked_add_charge(committed, charge)?;
    ensure_within_limits(&ledger.policy, requested)?;
    ledger.reservations.insert(reservation_id.clone(), BudgetReservation { reservation_id, charge });
    Ok(charge)
}

// r[impl lattice.budget_ledger_incubation.boundary]
/// Settle a reservation with an observed charge, folding the observed charge
/// into the consumed total and marking breach when limits are crossed.
pub fn settle_budget(
    ledger: &mut BudgetLedger,
    reservation_id: &str,
    observed: BudgetCharge,
) -> Result<BudgetSettlement, LedgerError> {
    settle_budget_for(ledger, &reservation_id.to_owned(), observed)
}

/// Settle a charge under a caller-owned nominal identity type.
pub fn settle_budget_for<Identity: Ord>(
    ledger: &mut BudgetLedger<Identity>,
    reservation_id: &Identity,
    observed: BudgetCharge,
) -> Result<BudgetSettlement, LedgerError> {
    let reservation = ledger.reservations.remove(reservation_id).ok_or(LedgerError::MissingReservation)?;
    let is_breached = charge_exceeds(observed, reservation.charge);
    ledger.consumed = checked_add_charge(ledger.consumed, observed)?;
    if is_breached || ensure_within_limits(&ledger.policy, ledger.consumed).is_err() {
        ledger.breached = true;
    }
    Ok(BudgetSettlement {
        breached: ledger.breached,
    })
}

// r[impl lattice.budget_ledger_incubation.boundary]
/// Compute the remaining budget projected against consumed plus reserved totals.
pub fn remaining_budget(ledger: &BudgetLedger) -> BudgetCharge {
    debug_assert!(ledger.reservations.values().all(|reservation| !reservation.reservation_id.is_empty()));
    remaining_budget_for(ledger)
}

/// Compute remaining budget for a ledger with a caller-owned identity type.
pub fn remaining_budget_for<Identity: Ord>(ledger: &BudgetLedger<Identity>) -> BudgetCharge {
    let reserved = ledger
        .reservations
        .values()
        .fold(BudgetCharge::default(), |total, reservation| saturating_add_charge(total, reservation.charge));
    BudgetCharge {
        rounds: ledger.policy.limits.rounds.saturating_sub(ledger.consumed.rounds.saturating_add(reserved.rounds)),
        agent_invocations: ledger
            .policy
            .limits
            .agent_invocations
            .saturating_sub(ledger.consumed.agent_invocations.saturating_add(reserved.agent_invocations)),
        tokens: ledger.policy.limits.tokens.saturating_sub(ledger.consumed.tokens.saturating_add(reserved.tokens)),
        monetary_microunits: ledger
            .policy
            .limits
            .monetary_microunits
            .saturating_sub(ledger.consumed.monetary_microunits.saturating_add(reserved.monetary_microunits)),
        duration_millis: ledger
            .policy
            .limits
            .duration_millis
            .saturating_sub(ledger.consumed.duration_millis.saturating_add(reserved.duration_millis)),
    }
}

fn admitted_charge(policy: &BudgetPolicy, plan: BudgetChargePlan) -> Result<BudgetCharge, LedgerError> {
    let mut charge = BudgetCharge::default();
    let plan_values = [
        plan.rounds,
        plan.agent_invocations,
        plan.tokens,
        plan.monetary_microunits,
        plan.duration_millis,
    ];
    let enforced = [
        policy.enforced.rounds,
        policy.enforced.agent_invocations,
        policy.enforced.tokens,
        policy.enforced.monetary_microunits,
        policy.enforced.duration_millis,
    ];
    for (index, (label, _get, set)) in DIMENSIONS.iter().enumerate() {
        let admitted = if enforced[index] {
            plan_values[index].ok_or(LedgerError::MissingBudgetBound { label })?
        } else {
            plan_values[index].unwrap_or_default()
        };
        set(&mut charge, admitted);
    }
    Ok(charge)
}

fn total_reserved<Identity>(
    reservations: &BTreeMap<Identity, BudgetReservation<Identity>>,
) -> Result<BudgetCharge, LedgerError> {
    reservations
        .values()
        .try_fold(BudgetCharge::default(), |total, reservation| checked_add_charge(total, reservation.charge))
}

fn checked_add_charge(left: BudgetCharge, right: BudgetCharge) -> Result<BudgetCharge, LedgerError> {
    let mut out = BudgetCharge::default();
    for (label, get, set) in DIMENSIONS.iter() {
        let value = get(&left).checked_add(get(&right)).ok_or(LedgerError::BudgetOverflow { label })?;
        set(&mut out, value);
    }
    Ok(out)
}

fn ensure_within_limits(policy: &BudgetPolicy, charge: BudgetCharge) -> Result<(), LedgerError> {
    let enforced = [
        policy.enforced.rounds,
        policy.enforced.agent_invocations,
        policy.enforced.tokens,
        policy.enforced.monetary_microunits,
        policy.enforced.duration_millis,
    ];
    let limits = [
        policy.limits.rounds,
        policy.limits.agent_invocations,
        policy.limits.tokens,
        policy.limits.monetary_microunits,
        policy.limits.duration_millis,
    ];
    for (index, (label, get, _set)) in DIMENSIONS.iter().enumerate() {
        if enforced[index] && get(&charge) > limits[index] {
            return Err(LedgerError::BudgetExceeded { label });
        }
    }
    Ok(())
}

fn charge_exceeds(observed: BudgetCharge, reserved: BudgetCharge) -> bool {
    DIMENSIONS.iter().any(|(_label, get, _set)| get(&observed) > get(&reserved))
}

fn saturating_add_charge(left: BudgetCharge, right: BudgetCharge) -> BudgetCharge {
    let mut out = BudgetCharge::default();
    for (_label, get, set) in DIMENSIONS.iter() {
        set(&mut out, get(&left).saturating_add(get(&right)));
    }
    out
}

#[cfg(test)]
mod tests;
