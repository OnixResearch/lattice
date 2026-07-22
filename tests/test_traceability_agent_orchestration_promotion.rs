#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use std::collections::BTreeSet;

const BASELINE: &str = include_str!("fixtures/traceability-agent-orchestration-audit-current.json");
const CURRENT: &str = include_str!("fixtures/traceability-agent-orchestration-promotion-current.json");
const CORE: &str = include_str!("test_orchestration_core.rs");
const STORE: &str = include_str!("test_orchestration_store.rs");
const FORMAT: &str = include_str!("test_orchestration_format.rs");
const SHELL: &str = include_str!("test_orchestration_shell.rs");
const CLI: &str = include_str!("test_orchestration_cli.rs");
const RUN_STORE: &str = include_str!("test_store.rs");
const POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_COUNT: usize = 261;
const CURRENT_COUNT: usize = 222;
const PROMOTED_COUNT: usize = 39;
const IDS: &[&str] = &[
    "lattice.agent_orchestration.authority.scenario.ambient-denied",
    "lattice.agent_orchestration.authority.scenario.approved",
    "lattice.agent_orchestration.authority.scenario.stale",
    "lattice.agent_orchestration.authority.scenario.validation",
    "lattice.agent_orchestration.budgets.scenario.block",
    "lattice.agent_orchestration.budgets.scenario.concurrent",
    "lattice.agent_orchestration.budgets.scenario.overage",
    "lattice.agent_orchestration.budgets.scenario.reserve",
    "lattice.agent_orchestration.cli.scenario.invalid",
    "lattice.agent_orchestration.cli.scenario.status",
    "lattice.agent_orchestration.controls.scenario.cancel",
    "lattice.agent_orchestration.controls.scenario.pause",
    "lattice.agent_orchestration.core.scenario.complete",
    "lattice.agent_orchestration.core.scenario.terminal-mutation",
    "lattice.agent_orchestration.durability.scenario.duplicate",
    "lattice.agent_orchestration.durability.scenario.external-effects",
    "lattice.agent_orchestration.durability.scenario.reconcile",
    "lattice.agent_orchestration.events.scenario.non-claim",
    "lattice.agent_orchestration.events.scenario.replay",
    "lattice.agent_orchestration.events.scenario.tampered",
    "lattice.agent_orchestration.format.scenario.n8n-boundary",
    "lattice.agent_orchestration.format.scenario.round-trip",
    "lattice.agent_orchestration.format.scenario.secret",
    "lattice.agent_orchestration.model.scenario.revision-drift",
    "lattice.agent_orchestration.model.scenario.valid",
    "lattice.agent_orchestration.rounds.scenario.execute",
    "lattice.agent_orchestration.rounds.scenario.hidden-cycle",
    "lattice.agent_orchestration.run_identity.scenario.concurrent",
    "lattice.agent_orchestration.run_identity.scenario.duplicate",
    "lattice.agent_orchestration.run_identity.scenario.foreign",
    "lattice.agent_orchestration.run_identity.scenario.match",
    "lattice.agent_orchestration.triggers.scenario.duplicate",
    "lattice.agent_orchestration.triggers.scenario.start",
    "lattice.agent_orchestration.unobserved_abandon.scenario.acknowledged",
    "lattice.agent_orchestration.unobserved_abandon.scenario.authority",
    "lattice.agent_orchestration.unobserved_abandon.scenario.identity",
    "lattice.agent_orchestration.unobserved_abandon.scenario.observed",
    "lattice.agent_orchestration.unobserved_abandon.scenario.planned",
    "lattice.agent_orchestration.unobserved_abandon.scenario.replay",
];
const CORE_BINDINGS: &[(&str, &str)] = &[
    (IDS[0], "authority_shaped_round_output_remains_inert_without_an_approval_fact"),
    (IDS[2], "stale_approval_is_rejected"),
    (IDS[3], "proposal_requires_current_authority_and_successor_validation"),
    (IDS[4], "budget_reservation_rejects_missing_bounds_and_oversubscription"),
    (IDS[6], "observed_budget_overage_exhausts_new_work_without_reversing_usage"),
    (IDS[7], "budget_reservation_rejects_missing_bounds_and_oversubscription"),
    (IDS[9], "status_text_is_actionable_safe_and_explicitly_local_and_at_least_once"),
    (IDS[10], "pause_and_cancel_preserve_observed_runner_boundary"),
    (IDS[11], "pause_and_cancel_preserve_observed_runner_boundary"),
    (IDS[12], "successful_terminal_round_completes_and_cannot_reopen"),
    (IDS[13], "successful_terminal_round_completes_and_cannot_reopen"),
    (IDS[14], "duplicate_fact_is_idempotent"),
    (IDS[15], "status_text_is_actionable_safe_and_explicitly_local_and_at_least_once"),
    (IDS[17], "status_text_is_actionable_safe_and_explicitly_local_and_at_least_once"),
    (IDS[18], "replay_reconstructs_state_and_rejects_tampering"),
    (IDS[19], "replay_reconstructs_state_and_rejects_tampering"),
    (IDS[24], "valid_definition_binds_exact_workflow_revision"),
    (IDS[26], "feedback_cannot_continue_a_running_round_or_bypass_successor_validation"),
    (IDS[33], "acknowledged_abandonment_consumes_reservation_and_replays_without_remote_claim"),
    (IDS[34], "abandonment_rejects_missing_cancellation_authority_and_identity_drift"),
    (IDS[35], "abandonment_rejects_missing_cancellation_authority_and_identity_drift"),
    (IDS[37], "planned_abandonment_releases_reservation_without_consumption"),
    (IDS[38], "acknowledged_abandonment_consumes_reservation_and_replays_without_remote_claim"),
];
const SHELL_BINDINGS: &[(&str, &str)] = &[
    (IDS[1], "approved_successor_runs_as_a_new_immutable_round"),
    (IDS[16], "recovery_reconciles_persisted_run_without_redispatch"),
    (IDS[23], "definition_validation_rejects_stale_workflow_revision"),
    (IDS[25], "start_executes_one_seaglass_admitted_round_and_persists_replay"),
    (IDS[29], "foreign_run_record_cannot_satisfy_or_abandon_a_pending_dispatch"),
    (IDS[30], "recovery_reconciles_persisted_run_without_redispatch"),
    (IDS[31], "duplicate_trigger_returns_the_original_instance_without_another_run"),
    (IDS[32], "duplicate_trigger_returns_the_original_instance_without_another_run"),
    (IDS[36], "observed_pending_run_rejects_abandonment_and_uses_recovery"),
];
const STORE_BINDINGS: &[(&str, &str)] = &[(IDS[5], "stale_concurrent_transition_cannot_overwrite_newer_state")];
const FORMAT_BINDINGS: &[(&str, &str)] = &[
    (IDS[20], "default_n8n_export_rejects_unmapped_orchestration_semantics"),
    (IDS[21], "typed_nickel_definition_round_trip_preserves_meaning"),
    (IDS[22], "plaintext_credential_field_is_rejected"),
];
const RUN_STORE_BINDINGS: &[(&str, &str)] = &[
    (IDS[27], "concurrent_run_id_allocations_are_unique_before_any_run_is_saved"),
    (IDS[28], "duplicate_run_record_identity_is_rejected_without_overwrite"),
];
const CLI_BINDINGS: &[(&str, &str)] = &[
    (IDS[8], "unauthorized_start_fails_without_persisting_an_instance"),
    (IDS[8], "missing_instance_command_fails_without_creating_state"),
];
const UNRELATED: &str = concat!("n8n", "clone.tui_ux.add_node.cancel");
const ADDED: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING: &str = "lattice.example.dangling.scenario.marker";

fn receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage decodes")
}
fn codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.agent_orchestration_promotion]
// r[verify lattice.repository_validation.agent_orchestration_promotion.scenario.delta]
#[test]
fn exact_agent_orchestration_promotion_preserves_unrelated_gaps() {
    let baseline = receipt(BASELINE);
    let current = receipt(CURRENT);
    let issues = core::validate_promotion(&baseline, &current, IDS);
    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_COUNT);
    assert_eq!(current.missing.len(), CURRENT_COUNT);
    assert_eq!(IDS.len(), PROMOTED_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(!current.valid);
}

// r[verify lattice.repository_validation.agent_orchestration_promotion.scenario.bindings]
#[test]
fn agent_orchestration_markers_are_locally_bound_and_leave_policy() {
    for (name, issues) in [
        ("core", core::validate_marker_bindings(CORE, CORE_BINDINGS)),
        ("store", core::validate_marker_bindings(STORE, STORE_BINDINGS)),
        ("format", core::validate_marker_bindings(FORMAT, FORMAT_BINDINGS)),
        ("shell", core::validate_marker_bindings(SHELL, SHELL_BINDINGS)),
        ("cli", core::validate_marker_bindings(CLI, CLI_BINDINGS)),
        ("run-store", core::validate_marker_bindings(RUN_STORE, RUN_STORE_BINDINGS)),
    ] {
        assert!(issues.is_empty(), "{name}: {issues:?}");
    }
    let policy = core::validate_policy_excludes_promoted_ids(POLICY, IDS);
    assert!(policy.is_empty(), "policy: {policy:?}");
}

// r[verify lattice.repository_validation.agent_orchestration_promotion.scenario.fail_closed]
#[test]
fn agent_orchestration_promotion_drift_fails_closed() {
    let mut baseline = receipt(BASELINE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = receipt(CURRENT);
    current.missing.retain(|id| id != UNRELATED);
    current.missing.push(IDS[0].to_string());
    current.missing.push(IDS[0].to_string());
    current.missing.push(ADDED.to_string());
    current.dangling.push(DANGLING.to_string());
    current.valid = true;
    let mut allowlist = IDS.to_vec();
    allowlist.push(IDS[0]);
    let promotion_issues = core::validate_promotion(&baseline, &current, &allowlist);
    let promotion_codes = codes(&promotion_issues);
    let binding_issues = core::validate_marker_bindings(CORE, &[(IDS[0], "missing_test")]);
    let binding_codes = codes(&binding_issues);
    let stale = format!("requirement_id = \"{}\"", IDS[38]);
    let policy_issues = core::validate_policy_excludes_promoted_ids(&stale, IDS);
    let policy_codes = codes(&policy_issues);
    for code in [
        "baseline.duplicate_missing",
        "baseline.unsorted",
        "current.duplicate_missing",
        "current.unsorted",
        "current.dangling",
        "current.expected_invalid",
        "allowlist.duplicate",
        "transition.expected_removal_missing",
        "transition.unexpected_removed",
        "transition.unexpected_added",
    ] {
        assert!(promotion_codes.contains(code));
    }
    assert!(binding_codes.contains("binding.missing"));
    assert!(policy_codes.contains("policy.promoted_present"));
}
