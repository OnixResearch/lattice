use super::super::accounting_charge;
use crate::orchestration::core::BudgetCharge;
use crate::orchestration::core::BudgetDimensions;
use crate::orchestration::core::BudgetPolicy;
use crate::orchestration::usage::HostInterval;
use crate::orchestration::usage::ProviderReport;

#[test]
fn partial_provider_reports_preserve_unknown_remainder_and_overrun() {
    let reserved = BudgetCharge {
        tokens: 100,
        monetary_microunits: 50,
        duration_millis: 500,
        ..BudgetCharge::default()
    };
    let policy = BudgetPolicy {
        enforced: BudgetDimensions {
            tokens: true,
            monetary_microunits: true,
            duration_millis: true,
            ..BudgetDimensions::default()
        },
        ..BudgetPolicy::default()
    };
    let report = |tokens| ProviderReport {
        node_id: 1,
        provider: "openai_compatible".to_string(),
        model: Some("reported-model".to_string()),
        configured_runtime: None,
        tokens: Some(tokens),
        cost_microunits: Some(999),
    };
    let interval = HostInterval {
        started_epoch_ms: 10,
        finished_epoch_ms: 52,
    };
    let charged = accounting_charge(reserved, Some(&interval), &[report(20)], &policy);
    assert_eq!(charged.tokens, 100);
    assert_eq!(charged.monetary_microunits, 50, "unidentified currency cannot replace admitted money");
    assert_eq!(charged.duration_millis, 42);
    let overrun = accounting_charge(reserved, None, &[report(101), report(20)], &policy);
    assert_eq!(overrun.tokens, 121);
    assert_eq!(overrun.duration_millis, 500);
    assert_eq!(accounting_charge(reserved, None, &[], &policy), reserved);
}

#[test]
fn invalid_host_interval_never_refunds_duration() {
    let reserved = BudgetCharge {
        duration_millis: 500,
        ..BudgetCharge::default()
    };
    let policy = BudgetPolicy {
        enforced: BudgetDimensions {
            duration_millis: true,
            ..BudgetDimensions::default()
        },
        ..BudgetPolicy::default()
    };
    let interval = HostInterval {
        started_epoch_ms: 52,
        finished_epoch_ms: 10,
    };
    assert_eq!(accounting_charge(reserved, Some(&interval), &[], &policy), reserved);
}
