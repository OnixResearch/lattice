//! Integration tests for epoch::deadline.

use hardened_wasmtime_runtime::epoch::deadline::*;

#[test]
// r[verify onix.wasmtime_runtime.epoch_deadline.valid]
fn positive_budgets_admit() {
    assert_eq!(
        admit(Budget { budget_ticks: 10 }),
        Ok(Budget { budget_ticks: 10 })
    );
}

#[test]
// r[verify onix.wasmtime_runtime.epoch_deadline.exhausted]
fn zero_budgets_fail_closed() {
    assert_eq!(admit(Budget { budget_ticks: 0 }), Err(Error::ZeroBudget));
}

#[test]
// r[verify onix.wasmtime_runtime.epoch_deadline.service]
fn a_missing_service_denies_the_invocation() {
    assert!(require(Some(ServicePresent)).is_ok());
    assert!(require(None).is_err());
}
