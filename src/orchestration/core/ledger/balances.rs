// r[impl lattice.agent_orchestration.budgets]
// r[impl lattice.budget_ledger_incubation.boundary]
pub fn reserve_budget(
    ledger: &mut super::super::BudgetLedger,
    reservation_id: crate::orchestration::identity::ReservationId,
    plan: super::super::BudgetChargePlan,
) -> Result<super::super::BudgetCharge, super::super::Diagnostic> {
    budget_ledger_core::reserve_budget_for(ledger, reservation_id, plan).map_err(to_diagnostic)
}

pub fn settle_budget(
    ledger: &mut super::super::BudgetLedger,
    reservation_id: &crate::orchestration::identity::ReservationId,
    observed: super::super::BudgetCharge,
) -> Result<super::super::BudgetSettlement, super::super::Diagnostic> {
    budget_ledger_core::settle_budget_for(ledger, reservation_id, observed).map_err(to_diagnostic)
}

pub(super) fn to_diagnostic(error: budget_ledger_core::LedgerError) -> super::super::Diagnostic {
    match error {
        budget_ledger_core::LedgerError::MissingBudgetBound { label } => super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::MissingBudgetBound,
            format!("enforced {label} budget has no admitted upper bound"),
        ),
        budget_ledger_core::LedgerError::BudgetExceeded { label } => super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::BudgetExceeded,
            format!("{label} budget is exhausted"),
        ),
        budget_ledger_core::LedgerError::BudgetOverflow { label } => super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::BudgetOverflow,
            format!("{label} budget accounting overflowed"),
        ),
        budget_ledger_core::LedgerError::DuplicateReservation => super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::DuplicateReservation,
            "budget reservation identity was already admitted",
        ),
        budget_ledger_core::LedgerError::MissingReservation => super::super::model::failures::diagnostic(
            super::super::DiagnosticCode::MissingReservation,
            "budget settlement does not match a pending reservation",
        ),
    }
}
