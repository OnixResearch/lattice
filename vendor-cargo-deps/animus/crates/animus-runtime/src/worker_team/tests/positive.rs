// Fixture names intentionally describe count-sensitive one-item merge cases.
#![allow(unknown_lints)]
#![allow(numeric_units)]

#[test]
fn valid_task_contract_preserves_scope_and_parent_policy() {
    // r[verify agent_harness_runtime.claude_code_agents.task_contract]
    // r[verify agent_harness_runtime.claude_code_agents.scope.no_bypass]
    let task = super::support::valid_task();

    assert_eq!(task.task_id(), super::support::TASK_ID);
    assert_eq!(task.objective(), super::support::OBJECTIVE);
    assert_eq!(task.scope().label, super::support::SCOPE_LABEL);
    assert_eq!(task.context_packet().items[0].kind, super::super::WorkerContextItemKind::Summary);
    assert!(task.authority().parent_policy_required);
    assert!(task.allowed_capabilities().iter().any(|capability| matches!(
        capability,
        super::super::WorkerCapability::RequestMutation { mutation_kind }
            if mutation_kind == super::support::MUTATION_KIND
    )));
}

#[test]
fn safe_context_packet_validates_without_full_parent_transcript() {
    // r[verify agent_harness_runtime.claude_code_agents.context_packet]
    let packet = super::support::valid_context_packet();

    super::super::validate_context_packet(&packet).expect("safe packet should validate");

    assert!(packet.items.iter().all(|item| item.kind.is_default_safe()));
}

#[test]
fn injected_service_handles_enqueue_status_cancel_and_collect() {
    // r[verify agent_harness_runtime.claude_code_agents.worker_service]
    let task = super::support::valid_task();
    let mut service = super::support::RecordingService::default();
    let handle = super::super::enqueue_task(Some(&mut service), &task).expect("injected service should enqueue");
    let status =
        super::super::request_status(Some(&mut service), &handle).expect("injected service should return status");
    let cancel = super::super::request_cancel(Some(&mut service), &handle).expect("injected service should cancel");
    let collected = super::super::collect_report(Some(&mut service), &handle).expect("injected service should collect");

    assert_eq!(handle.id(), super::support::HANDLE_ID);
    assert_eq!(status, super::super::WorkerStatus::Running);
    assert_eq!(cancel, super::super::WorkerCancelOutcome::Accepted);
    assert_eq!(collected.worker_id, super::support::WORKER_ID);
    assert_eq!(service.enqueued, super::support::EXPECTED_SINGLE_CALL);
    assert_eq!(service.status_queries, super::support::EXPECTED_SINGLE_CALL);
    assert_eq!(service.cancellations, super::support::EXPECTED_SINGLE_CALL);
    assert_eq!(service.collections, super::support::EXPECTED_SINGLE_CALL);
}

#[test]
fn compatible_reports_merge_with_unresolved_mutation_intents() {
    // r[verify agent_harness_runtime.claude_code_agents.report_merge]
    let task = super::support::valid_task();
    let merge_inputs = [super::support::valid_report()];
    let outcome = super::super::merge_reports(&task, merge_inputs.as_slice()).expect("compatible reports should merge");

    let super::super::ReportMergeOutcome::Merged(merged) = outcome else {
        panic!("expected merged outcome");
    };

    assert_eq!(merged.evidence.len(), super::support::EXPECTED_SINGLE_CALL);
    assert_eq!(merged.claims.len(), super::support::EXPECTED_SINGLE_CALL);
    assert_eq!(merged.unresolved_mutation_intents.len(), super::support::EXPECTED_SINGLE_CALL);
    assert!(merged.denied_mutation_intents.is_empty());
    assert_eq!(merged.evidence[0].evidence_id, super::support::EVIDENCE_ID);
}
