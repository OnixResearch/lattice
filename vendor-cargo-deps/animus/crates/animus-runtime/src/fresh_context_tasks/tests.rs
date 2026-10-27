// Cairn adaptation compatibility modules intentionally keep public DTO names and table-driven validation compact while drained changes are archived.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    borrowed_argument_types
)]
#![allow(
    compound_condition,
    excessive_file_length,
    function_length,
    non_trait_imports
)]
#![allow(
    numeric_units,
    path_segment_repetition,
    raw_arithmetic_overflow,
    unbounded_collection_growth
)]

use super::*;

const CHILD_RUN: &str = "child.run";
const DESCRIPTOR_ID: &str = "task.one";
const GOAL: &str = "Investigate cross-file context.";
const MODEL_POLICY: &str = "local-model";
const RESULT_SCHEMA: &str = "summary-with-citations";
const TIMEOUT: &str = "short";
const TOKEN_BUDGET: u32 = 1_000;
const TOOL_BUDGET: u32 = 4;

#[test]
fn missing_scheduler_and_inherited_authority_fail_closed() {
    // r[agent_harness_runtime.zerostack_fresh_context_tasks.descriptor.missing_scheduler]
    // r[agent_harness_runtime.zerostack_fresh_context_tasks.isolation.allowlist]
    let decision = evaluate_fresh_task_dispatch(
        &descriptor(FreshTaskMode::Investigation),
        &FreshTaskIsolation {
            parent_allowlist: FreshTaskInheritance::Inherited,
            hidden_prompts: FreshTaskInheritance::Isolated,
            parent_secrets: FreshTaskInheritance::Isolated,
            process_globals: FreshTaskInheritance::Isolated,
        },
        FreshTaskSchedulerState::Missing,
    );

    assert!(!decision.dispatch);
    assert!(
        decision
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == FreshTaskDiagnosticKind::MissingScheduler })
    );
    assert!(
        decision
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == FreshTaskDiagnosticKind::IsolationViolation })
    );
}

#[test]
fn side_question_denies_mutating_actions() {
    // r[agent_harness_runtime.zerostack_fresh_context_tasks.side_question.readonly]
    let task = descriptor(FreshTaskMode::SideQuestion);
    assert!(evaluate_side_question_action(&task, FreshTaskActionKind::Read).dispatch);
    assert!(evaluate_side_question_action(&task, FreshTaskActionKind::Search).dispatch);
    for action in [
        FreshTaskActionKind::Write,
        FreshTaskActionKind::Edit,
        FreshTaskActionKind::Shell,
        FreshTaskActionKind::Network,
        FreshTaskActionKind::LedgerMutation,
        FreshTaskActionKind::OtherTool,
    ] {
        let decision = evaluate_side_question_action(&task, action);
        assert!(!decision.dispatch);
        assert!(
            decision
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.kind == FreshTaskDiagnosticKind::ReadOnlyDenied })
        );
    }
}

#[test]
fn descriptor_and_result_validation_reject_malformed_and_unsafe_data() {
    // r[agent_harness_runtime.zerostack_fresh_context_tasks.results.malformed]
    let invalid = FreshTaskDescriptor {
        goal: String::new(),
        budget: FreshTaskBudget {
            tokens: 0,
            tool_calls: 0,
            timeout_label: String::new(),
        },
        ..descriptor(FreshTaskMode::Investigation)
    };
    assert!(validate_fresh_task_descriptor(&invalid).is_err());

    let result = FreshTaskResult {
        child_run_id: String::new(),
        status: FreshTaskResultStatus::Missing,
        evidence_summaries: vec!["tool_output: API_KEY".to_string()],
        output_hash: FreshTaskFingerprint::new("SHA256", "abc"),
        ..result(FreshTaskResultStatus::Completed)
    };
    let diagnostics = validate_fresh_task_result(&result).expect_err("unsafe result");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == FreshTaskDiagnosticKind::MalformedResult })
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.kind == FreshTaskDiagnosticKind::UnsafeResult })
    );
}

#[test]
fn cancelled_task_receipt_is_bounded() {
    // r[agent_harness_runtime.zerostack_fresh_context_tasks.receipts.cancelled]
    let receipt = FreshTaskReceipt {
        descriptor_id: DESCRIPTOR_ID.to_string(),
        scheduler_id: "scheduler.local".to_string(),
        status: FreshTaskResultStatus::Cancelled,
        cancellation_reason: FreshTaskCancellationReason::Timeout,
        budget_used: budget(),
        output_hash: fp("cancelled"),
        transcript_embedding: FreshTaskInheritance::Isolated,
    };

    assert_eq!(receipt.status, FreshTaskResultStatus::Cancelled);
    assert_eq!(
        receipt.cancellation_reason,
        FreshTaskCancellationReason::Timeout
    );
    assert_eq!(receipt.transcript_embedding, FreshTaskInheritance::Isolated);
    assert_eq!(receipt.output_hash.algorithm, BLAKE3_ALGORITHM);
}

fn budget() -> FreshTaskBudget {
    FreshTaskBudget {
        tokens: TOKEN_BUDGET,
        tool_calls: TOOL_BUDGET,
        timeout_label: TIMEOUT.to_string(),
    }
}

fn descriptor(mode: FreshTaskMode) -> FreshTaskDescriptor {
    FreshTaskDescriptor {
        descriptor_id: DESCRIPTOR_ID.to_string(),
        goal: GOAL.to_string(),
        context_references: vec!["ctx:one".to_string()],
        allowed_actions: vec![FreshTaskActionKind::Read, FreshTaskActionKind::Search],
        model_policy: MODEL_POLICY.to_string(),
        budget: budget(),
        result_schema: RESULT_SCHEMA.to_string(),
        mode,
    }
}

fn fp(label: &str) -> FreshTaskFingerprint {
    blake3_fresh_task_fingerprint(label, label.as_bytes())
}

fn result(status: FreshTaskResultStatus) -> FreshTaskResult {
    FreshTaskResult {
        child_run_id: CHILD_RUN.to_string(),
        descriptor_id: DESCRIPTOR_ID.to_string(),
        status,
        budget_used: budget(),
        citations: vec!["ctx:one".to_string()],
        uncertainty_markers: vec!["needs-review".to_string()],
        evidence_summaries: vec!["safe bounded evidence".to_string()],
        output_hash: fp("output"),
    }
}
