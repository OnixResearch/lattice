use crate::*;

// r[verify lattice.budget_ledger_incubation.compatibility]

fn enforced_policy(rounds: u64, tokens: u64) -> BudgetPolicy {
    BudgetPolicy {
        enforced: BudgetDimensions {
            rounds: true,
            tokens: true,
            ..BudgetDimensions::default()
        },
        limits: BudgetLimits {
            rounds,
            tokens,
            ..BudgetLimits::default()
        },
        per_round_upper_bound: BudgetCharge::default(),
    }
}

fn charge(rounds: u64, tokens: u64) -> BudgetCharge {
    BudgetCharge {
        rounds,
        tokens,
        ..BudgetCharge::default()
    }
}

fn plan(rounds: Option<u64>, tokens: Option<u64>) -> BudgetChargePlan {
    BudgetChargePlan {
        rounds,
        tokens,
        ..BudgetChargePlan::default()
    }
}

// ---- Positive: admitted bounds ----

#[test]
fn reserve_admits_an_enforced_plan_within_limits() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    let admitted = reserve_budget(&mut ledger, "dispatch-1", plan(Some(3), Some(40))).expect("admission must succeed");
    assert_eq!(admitted, charge(3, 40));
    assert_eq!(ledger.reservations.len(), 1);
    assert_eq!(ledger.consumed, BudgetCharge::default());
    assert!(!ledger.breached);
}

// ---- Positive: independent reservations ----

#[test]
fn independent_reservations_total_reserved_across_identities() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "dispatch-1", plan(Some(2), Some(10))).expect("first reservation must fit");
    reserve_budget(&mut ledger, "dispatch-2", plan(Some(3), Some(20))).expect("second reservation must fit");
    assert_eq!(ledger.reservations.len(), 2);
    let remaining = remaining_budget(&ledger);
    // limit 10 rounds / 100 tokens minus reserved 5 / 30
    assert_eq!(remaining.rounds, 5);
    assert_eq!(remaining.tokens, 70);
}

// ---- Positive: total reserved accounting ----

#[test]
fn total_reserved_rounds_into_committed_limit_check() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(8, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "a", plan(Some(5), Some(10))).expect("a fits");
    // Consumed nothing yet; reserved 5/10; limit 8/100 so 3/90 remain.
    reserve_budget(&mut ledger, "b", plan(Some(3), Some(30))).expect("b must fit against committed plus reserved");
    let remaining = remaining_budget(&ledger);
    assert_eq!(remaining.rounds, 0);
    assert_eq!(remaining.tokens, 60);
}

// ---- Positive: below/at-bound settlement ----

#[test]
fn settlement_below_and_at_reservation_folds_consumed_and_clears_reservation() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "dispatch-1", plan(Some(4), Some(50))).expect("admission");
    let below = settle_budget(&mut ledger, "dispatch-1", charge(3, 40)).expect("below-bound settlement");
    // r[verify lattice.budget_ledger_incubation.compatibility.scenario.accepted]
    assert!(!below.breached);
    assert_eq!(ledger.consumed, charge(3, 40));
    assert!(ledger.reservations.is_empty());

    reserve_budget(&mut ledger, "dispatch-2", plan(Some(7), Some(60))).expect("admission");
    let at = settle_budget(&mut ledger, "dispatch-2", charge(7, 60)).expect("at-bound settlement");
    assert!(!at.breached);
    assert_eq!(ledger.consumed, charge(10, 100));
    assert_eq!(remaining_budget(&ledger), BudgetCharge::default());
}

// ---- Positive: unenforced dimensions default to zero ----

#[test]
fn unenforced_dimensions_accept_missing_plans_as_zero_and_stay_unbounded() {
    let mut ledger = BudgetLedger {
        policy: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: 10,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge::default(),
        },
        ..BudgetLedger::default()
    };
    // rounds enforced, everything else unenforced; plan omits all but rounds.
    let admitted = reserve_budget(&mut ledger, "dispatch-1", plan(Some(2), None)).expect("admission");
    assert_eq!(admitted.rounds, 2);
    assert_eq!(admitted.tokens, 0);
    assert_eq!(admitted.agent_invocations, 0);
}

// ---- Positive: remaining balances after partial settlement ----

#[test]
fn remaining_balance_reflects_consumed_plus_reserved() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "open", plan(Some(2), Some(20))).expect("open reservation");
    reserve_budget(&mut ledger, "settled", plan(Some(4), Some(30))).expect("to-be-settled reservation");
    settle_budget(&mut ledger, "settled", charge(3, 25)).expect("settle below reservation");
    // consumed 3/25, reserved 2/20 -> remaining 5/55.
    let remaining = remaining_budget(&ledger);
    assert_eq!(remaining.rounds, 5);
    assert_eq!(remaining.tokens, 55);
    // duration_millis is unenforced with a default limit of zero.
    assert_eq!(remaining.duration_millis, 0);
}

// ---- Negative: missing enforced bounds ----

#[test]
fn reserve_rejects_missing_bound_for_enforced_dimension() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    let before = ledger.clone();
    let error = reserve_budget(&mut ledger, "dispatch-1", plan(None, Some(10))).expect_err("rounds bound is required");
    assert!(matches!(error, LedgerError::MissingBudgetBound { label } if label == "rounds"));
    assert_eq!(ledger, before);
}

// ---- Negative: duplicate reservation ----

#[test]
fn reserve_rejects_duplicate_reservation_identity() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "dispatch-1", plan(Some(2), Some(10))).expect("first admission");
    let before = ledger.clone();
    let error = reserve_budget(&mut ledger, "dispatch-1", plan(Some(3), Some(10))).expect_err("duplicate rejected");
    assert!(matches!(error, LedgerError::DuplicateReservation));
    // r[verify lattice.budget_ledger_incubation.compatibility.scenario.rejected]
    assert_eq!(ledger, before);
}

// ---- Negative: missing settlement ----

#[test]
fn settle_rejects_unknown_reservation_identity() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    let before = ledger.clone();
    let error = settle_budget(&mut ledger, "never-reserved", charge(1, 1)).expect_err("settlement without reservation");
    assert!(matches!(error, LedgerError::MissingReservation));
    assert_eq!(ledger, before);
}

// ---- Negative: limit excess ----

#[test]
fn reserve_rejects_total_that_exceeds_limits() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "first", plan(Some(9), Some(90))).expect("first fits");
    let before = ledger.clone();
    let error = reserve_budget(&mut ledger, "second", plan(Some(2), Some(20))).expect_err("exceeds limit");
    assert!(matches!(error, LedgerError::BudgetExceeded { label } if label == "rounds"));
    assert_eq!(ledger, before);
}

// ---- Negative: arithmetic overflow ----

#[test]
fn reserve_rejects_summed_overflow() {
    let mut ledger = BudgetLedger {
        policy: BudgetPolicy {
            enforced: BudgetDimensions {
                rounds: true,
                ..BudgetDimensions::default()
            },
            limits: BudgetLimits {
                rounds: u64::MAX,
                ..BudgetLimits::default()
            },
            per_round_upper_bound: BudgetCharge::default(),
        },
        consumed: charge(u64::MAX, 0),
        ..BudgetLedger::default()
    };
    let before = ledger.clone();
    let error = reserve_budget(&mut ledger, "dispatch-1", plan(Some(1), None)).expect_err("overflow rejected");
    assert!(matches!(error, LedgerError::BudgetOverflow { label } if label == "rounds"));
    assert_eq!(ledger, before);
}

// ---- Negative: observed overage marks breach ----

#[test]
fn settlement_observed_overage_marks_ledger_breached() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "dispatch-1", plan(Some(4), Some(40))).expect("admission");
    let settlement = settle_budget(&mut ledger, "dispatch-1", charge(6, 40)).expect("settlement succeeds");
    assert!(settlement.breached);
    assert!(ledger.breached);
    assert_eq!(ledger.consumed, charge(6, 40));
}

// ---- Negative: stale identity after settlement ----

#[test]
fn settle_rejects_stale_identity_already_settled() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(10, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "dispatch-1", plan(Some(2), Some(20))).expect("admission");
    settle_budget(&mut ledger, "dispatch-1", charge(2, 20)).expect("first settlement");
    let before = ledger.clone();
    let error = settle_budget(&mut ledger, "dispatch-1", charge(1, 1)).expect_err("stale identity rejected");
    assert!(matches!(error, LedgerError::MissingReservation));
    assert_eq!(ledger, before);
}

#[test]
fn settlement_overflow_preserves_reservation_consumption_and_breach_state() {
    let mut ledger = BudgetLedger {
        policy: enforced_policy(u64::MAX, u64::MAX),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "dispatch-1", plan(Some(1), Some(1))).expect("reservation fits");
    ledger.consumed = charge(u64::MAX - 1, u64::MAX);
    let before = ledger.clone();

    let error = settle_budget(&mut ledger, "dispatch-1", charge(1, 1)).expect_err("observed total overflows");

    // r[verify lattice.budget_ledger_incubation.compatibility.scenario.rejected]
    assert_eq!(error, LedgerError::BudgetOverflow { label: "tokens" });
    assert_eq!(ledger, before);
}

// ---- Negative: invalid orchestration integration ----

#[test]
fn exhausted_ledger_rejects_further_reservation_after_reaching_limit() {
    // Consuming to the enforced limit must leave the ledger closed to further
    // reservations even when each individual plan would fit alone.
    let mut ledger = BudgetLedger {
        policy: enforced_policy(5, 100),
        ..BudgetLedger::default()
    };
    reserve_budget(&mut ledger, "a", plan(Some(3), Some(30))).expect("a admits");
    let settlement = settle_budget(&mut ledger, "a", charge(3, 30)).expect("settle a");
    assert!(!settlement.breached);
    assert_eq!(ledger.consumed, charge(3, 30));

    let error = reserve_budget(&mut ledger, "b", plan(Some(3), Some(20)))
        .expect_err("exhausted ledger must reject committed plus reserved beyond the limit");
    assert!(matches!(error, LedgerError::BudgetExceeded { label } if label == "rounds"));
    assert!(ledger.reservations.is_empty());
}
