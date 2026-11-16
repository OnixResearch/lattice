// r[verify lattice.agent_runtime.configured_bindings.scenario.usage]
use super::tests::observation;
use super::*;
use crate::orchestration::core::BudgetCharge;

fn host_observation() -> CommandWorkerObservation {
    let mut observation = observation();
    observation.scope = Scope::HostRun;
    observation.node_id = 0;
    observation.worker_profile.clear();
    observation.agent_invocations = None;
    observation.model_requests = None;
    observation.subagent_calls = None;
    observation.tokens = None;
    observation.cost_microunits = None;
    observation.provenance = UsageProvenance::HostMeasured;
    observation.complete = false;
    observation.accounting = Some(RunAccounting {
        schema: RUN_USAGE_ACCOUNTING_SCHEMA.to_string(),
        reserved: BudgetCharge::default(),
        charged: BudgetCharge::default(),
        host_interval: Some(HostInterval {
            started_epoch_ms: 10,
            finished_epoch_ms: 52,
        }),
        provider_reports: Vec::new(),
        unknown_tokens: true,
        unknown_cost: true,
        unknown_model_requests: true,
        unknown_subagent_calls: true,
        diagnostic: "conservative host receipt".to_string(),
        outcome: None,
    });
    observation
}

fn report() -> ProviderReport {
    ProviderReport {
        node_id: 1,
        provider: "openai_compatible".to_string(),
        model: None,
        configured_runtime: None,
        tokens: Some(100),
        cost_microunits: None,
    }
}

#[test]
fn absent_optional_receipt_fields_remain_explicitly_unknown() {
    let json = observation().to_json().expect("worker observation");
    assert!(!json.contains("\"accounting\""));
    assert!(CommandWorkerObservation::from_json(&json).expect("missing accounting").accounting.is_none());
    let host = host_observation();
    assert_eq!(host.validate(), Ok(()));
    let json = host.to_json().expect("host observation");
    assert!(!json.contains("\"outcome\""));
    let decoded = CommandWorkerObservation::from_json(&json).expect("missing outcome");
    assert_eq!(decoded, host);
    assert_eq!(decoded.accounting.expect("accounting").outcome, None);
}

#[test]
fn host_receipt_validation_preserves_every_unknown_binding_and_charge_boundary() {
    type Mutation = fn(&mut CommandWorkerObservation);
    let mutations: &[Mutation] = &[
        |observation| observation.scope = Scope::Run,
        |observation| observation.complete = true,
        |observation| observation.provenance = UsageProvenance::ProviderReported,
        |observation| observation.agent_invocations = Some(0),
        |observation| observation.model_requests = Some(0),
        |observation| observation.subagent_calls = Some(0),
        |observation| observation.tokens = Some(0),
        |observation| observation.cost_microunits = Some(0),
        |observation| observation.accounting.as_mut().expect("accounting").unknown_tokens = false,
        |observation| observation.accounting.as_mut().expect("accounting").unknown_cost = false,
        |observation| observation.accounting.as_mut().expect("accounting").unknown_model_requests = false,
        |observation| observation.accounting.as_mut().expect("accounting").unknown_subagent_calls = false,
        |observation| observation.accounting.as_mut().expect("accounting").schema = "invalid".to_string(),
        |observation| {
            observation
                .accounting
                .as_mut()
                .expect("accounting")
                .host_interval
                .as_mut()
                .expect("interval")
                .finished_epoch_ms = 9;
        },
        |observation| observation.accounting.as_mut().expect("accounting").charged.rounds = 1,
        |observation| observation.accounting.as_mut().expect("accounting").charged.agent_invocations = 1,
        |observation| observation.accounting.as_mut().expect("accounting").reserved.tokens = 1,
        |observation| observation.accounting.as_mut().expect("accounting").reserved.monetary_microunits = 1,
        |observation| observation.duration_millis = None,
        |observation| observation.accounting.as_mut().expect("accounting").provider_reports = vec![report(), report()],
        |observation| {
            observation.accounting.as_mut().expect("accounting").provider_reports =
                vec![report(); MAX_PROVIDER_REPORTS + 1];
        },
        |observation| {
            observation.accounting.as_mut().expect("accounting").outcome = Some(RunOutcomeObservation {
                succeeded: true,
                cancelled: Some(true),
            });
        },
        |observation| {
            observation.accounting.as_mut().expect("accounting").diagnostic =
                "x".repeat(MAX_ACCOUNTING_DIAGNOSTIC_BYTES + 1);
        },
        |observation| observation.worker_profile = "unqualified".to_string(),
        |observation| observation.node_id = 1,
        |observation| observation.accounting = None,
    ];
    for mutate in mutations {
        let mut observation = host_observation();
        mutate(&mut observation);
        assert_eq!(observation.validate(), Err(ObservationError::InvalidHostReceipt));
    }
}

#[test]
fn configured_runtime_provenance_survives_restart_and_cannot_be_rebound() {
    let directory = tempfile::tempdir().expect("usage store directory");
    let path = directory.path().join("usage.redb");
    let store = crate::store::Store::new(&path).expect("store opens");
    let mut observation = host_observation();
    let mut provider = report();
    provider.configured_runtime = Some(crate::agent_node::ConfiguredBinding {
        schema: crate::agent_node::CONFIGURED_RUNTIME_SCHEMA.to_string(),
        target: crate::agent_node::Target::Pi,
        revision: format!("blake3:{}", "a".repeat(64)),
    });
    observation.accounting.as_mut().expect("accounting").provider_reports.push(provider);
    store.save_command_worker_usage(&observation).expect("binding persists");
    store.save_command_worker_usage(&observation).expect("exact duplicate is idempotent");
    drop(store);
    let store = crate::store::Store::new(&path).expect("store reopens");
    assert_eq!(
        store
            .load_command_worker_usage_for_run(crate::engine::RunId(observation.run_id))
            .expect("binding loads"),
        Some(observation.clone())
    );
    let mut changed = observation.clone();
    changed.accounting.as_mut().expect("accounting").provider_reports[0]
        .configured_runtime
        .as_mut()
        .expect("binding")
        .revision = format!("blake3:{}", "b".repeat(64));
    assert!(matches!(store.save_command_worker_usage(&changed), Err(crate::store::StoreError::Conflict(_))));
    changed.accounting.as_mut().expect("accounting").provider_reports[0]
        .configured_runtime
        .as_mut()
        .expect("binding")
        .revision = "not-a-revision".to_string();
    assert_eq!(changed.validate(), Err(ObservationError::InvalidHostReceipt));
    assert_eq!(
        store.load_command_worker_usage(&observation.attempt_id).expect("original receipt"),
        Some(observation)
    );
}
