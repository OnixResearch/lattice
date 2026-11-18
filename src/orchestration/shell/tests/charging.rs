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

fn settled_host_receipt(has_attempt_provenance: bool) -> crate::orchestration::usage::RunAccounting {
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
    let interval = HostInterval {
        started_epoch_ms: 10,
        finished_epoch_ms: 52,
    };
    let directory = tempfile::tempdir().expect("store directory");
    let store = crate::store::Store::new(directory.path().join("settlement.redb")).expect("store");
    if has_attempt_provenance {
        let bound = crate::orchestration::usage::tests::attempt_observation();
        store.save_command_worker_usage(&bound).expect("bound attempt receipt");
    }
    let charged = accounting_charge(reserved, Some(&interval), &[], &policy);
    let mut receipt = crate::orchestration::usage::tests::observation();
    receipt.scope = crate::orchestration::usage::Scope::HostRun;
    receipt.node_id = 0;
    receipt.worker_profile.clear();
    receipt.agent_invocations = None;
    receipt.model_requests = None;
    receipt.subagent_calls = None;
    receipt.tokens = None;
    receipt.cost_microunits = None;
    receipt.provenance = crate::orchestration::usage::UsageProvenance::HostMeasured;
    receipt.complete = false;
    receipt.accounting = Some(crate::orchestration::usage::RunAccounting {
        schema: crate::orchestration::usage::RUN_USAGE_ACCOUNTING_SCHEMA.to_string(),
        reserved,
        charged,
        host_interval: Some(interval),
        provider_reports: Vec::new(),
        unknown_tokens: true,
        unknown_cost: true,
        unknown_model_requests: true,
        unknown_subagent_calls: true,
        diagnostic: "settlement invariance".to_string(),
        outcome: None,
    });
    store.save_command_worker_usage(&receipt).expect("host receipt");
    store
        .load_command_worker_usage_for_run(crate::engine::RunId(receipt.run_id))
        .expect("load")
        .and_then(|loaded| loaded.accounting)
        .expect("host accounting")
}

// r[verify lattice.execution_binding.usage_provenance.scenario.replay]
#[test]
fn attempt_binding_provenance_never_changes_host_run_settlement() {
    let without = settled_host_receipt(false);
    let with = settled_host_receipt(true);
    assert_eq!((with.reserved, with.charged), (without.reserved, without.charged));
    assert_eq!(with.charged.duration_millis, 42);
    let mut bound_host = crate::orchestration::usage::tests::observation();
    bound_host.scope = crate::orchestration::usage::Scope::HostRun;
    bound_host.execution_binding = crate::orchestration::usage::tests::attempt_observation().execution_binding;
    assert!(bound_host.validate().is_err(), "orchestration host runs carry no execution binding");
}
