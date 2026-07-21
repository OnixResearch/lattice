// Fixture names intentionally describe count-sensitive one-item merge cases.
#![allow(unknown_lints)]
#![allow(numeric_units)]

#[test]
fn task_rejects_missing_fields_and_overbroad_authority() {
    // r[verify agent_harness_runtime.claude_code_agents.task_contract.overbroad]
    let missing_scope = super::support::valid_task_draft();
    let missing_capabilities = super::support::valid_task_draft();
    let missing_schema = super::support::valid_task_draft();
    let overbroad_authority = super::support::valid_task_draft();
    let bypass_capability = super::support::valid_task_draft();
    let cases = [
        (
            super::super::WorkerTaskDraft {
                scope: None,
                ..missing_scope
            },
            super::super::FailureKind::MissingField,
        ),
        (
            super::super::WorkerTaskDraft {
                allowed_capabilities: Vec::new(),
                ..missing_capabilities
            },
            super::super::FailureKind::EmptyField,
        ),
        (
            super::super::WorkerTaskDraft {
                output_schema: None,
                ..missing_schema
            },
            super::super::FailureKind::MissingField,
        ),
        (
            super::super::WorkerTaskDraft {
                authority: Some(super::super::WorkerAuthority {
                    parent_policy_required: false,
                    ..super::support::valid_authority()
                }),
                ..overbroad_authority
            },
            super::super::FailureKind::OverbroadAuthority,
        ),
        (
            super::super::WorkerTaskDraft {
                allowed_capabilities: vec![
                    super::super::WorkerCapability::ReadContext,
                    super::super::WorkerCapability::BypassParentPolicy,
                ],
                ..bypass_capability
            },
            super::super::FailureKind::OverbroadAuthority,
        ),
    ];

    assert_eq!(cases.len(), super::support::INVALID_TASK_CASES);
    for (draft, expected_kind) in cases {
        let error = super::super::validate_task(draft)
            .expect_err("invalid worker task must fail before enqueue");
        assert_eq!(error.kind(), expected_kind);
        assert!(!error.reason().is_empty());
    }
}

#[test]
fn context_packets_reject_parent_context_leaks() {
    // r[verify agent_harness_runtime.claude_code_agents.context_packet.leak]
    let cases = [
        super::support::unsafe_context_packet(
            super::super::WorkerContextItemKind::FullTranscript,
            "full parent transcript",
        ),
        super::support::unsafe_context_packet(
            super::super::WorkerContextItemKind::RawHiddenMemory,
            "hidden memory body",
        ),
        super::support::unsafe_context_packet(
            super::super::WorkerContextItemKind::ProviderPayload,
            "provider native payload",
        ),
        super::support::unsafe_context_packet(
            super::super::WorkerContextItemKind::UnboundedToolOutput,
            "long tool output",
        ),
        super::super::WorkerContextPacket {
            items: vec![super::super::WorkerContextItem {
                kind: super::super::WorkerContextItemKind::MemoryDescriptor,
                label: String::from(super::support::CONTEXT_LABEL),
                body: String::from("API_KEY=credential-looking value"),
                redaction: super::super::WorkerContextRedaction::SafeSummary,
            }],
            ..super::support::valid_context_packet()
        },
    ];

    assert_eq!(cases.len(), super::support::CONTEXT_LEAK_CASES);
    for packet in cases {
        let error = super::super::validate_context_packet(&packet)
            .expect_err("leaking context must fail before worker start");
        assert!(matches!(
            error.kind(),
            super::super::FailureKind::ContextLeak | super::super::FailureKind::UnsafeText
        ));
        assert!(!error.reason().is_empty());
    }
}

#[test]
fn missing_service_fails_without_fallback_dispatch() {
    // r[verify agent_harness_runtime.claude_code_agents.worker_service.missing]
    let task = super::support::valid_task();
    let error =
        super::super::enqueue_task(None, &task).expect_err("missing service must fail closed");

    assert_eq!(
        error.kind(),
        super::super::FailureKind::WorkerServiceUnavailable
    );
    assert_eq!(error.field(), super::super::WORKER_SERVICE_FIELD);
}

#[test]
fn malformed_report_is_not_successful_completion() {
    let task = super::support::valid_task();
    let malformed = super::super::Report {
        fields: std::collections::BTreeMap::from([(
            String::from(super::support::SUMMARY_FIELD),
            String::from("safe summary"),
        )]),
        ..super::support::valid_report()
    };
    let error = super::super::merge_reports(&task, &[malformed])
        .expect_err("missing required report field must fail");

    assert_eq!(error.kind(), super::super::FailureKind::MalformedReport);
}

#[test]
fn conflicting_reports_require_arbitration() {
    // r[verify agent_harness_runtime.claude_code_agents.report_merge.conflict]
    let task = super::support::valid_task();
    let conflicting = super::super::Report {
        worker_id: String::from("worker-b"),
        claims: vec![super::super::Claim {
            subject: String::from(super::support::CLAIM_SUBJECT),
            value: String::from(super::support::CONFLICTING_CLAIM_VALUE),
            evidence_ids: vec![String::from(super::support::EVIDENCE_ID)],
        }],
        ..super::support::valid_report()
    };
    let merge_inputs = [super::support::valid_report(), conflicting];
    let outcome = super::super::merge_reports(&task, merge_inputs.as_slice())
        .expect("valid but incompatible reports should produce conflict outcome");

    let super::super::ReportMergeOutcome::Conflict(conflict) = outcome else {
        panic!("expected conflict outcome");
    };

    assert_eq!(
        conflict.conflicts[0].code,
        super::super::CONFLICT_CLAIM_CODE
    );
    assert_eq!(conflict.conflicts[0].subject, super::support::CLAIM_SUBJECT);
    assert!(!conflict.evidence.is_empty());
}

#[test]
fn unauthorized_mutation_is_denied_not_dispatched() {
    // r[verify agent_harness_runtime.claude_code_agents.scope.no_bypass]
    let task = super::super::validate_task(super::super::WorkerTaskDraft {
        allowed_capabilities: vec![
            super::super::WorkerCapability::ReadContext,
            super::super::WorkerCapability::ReportFindings,
        ],
        ..super::support::valid_task_draft()
    })
    .expect("task without mutation capability remains valid");
    let merge_inputs = [super::support::valid_report()];
    let outcome = super::super::merge_reports(&task, merge_inputs.as_slice())
        .expect("report is valid but mutation is denied");

    let super::super::ReportMergeOutcome::Merged(merged) = outcome else {
        panic!("expected merged outcome with denied mutation");
    };

    assert!(merged.unresolved_mutation_intents.is_empty());
    assert_eq!(
        merged.denied_mutation_intents.len(),
        super::support::EXPECTED_SINGLE_CALL
    );
}
