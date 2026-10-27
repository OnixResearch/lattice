use super::super::*;
use super::support::*;

#[test]
fn final_output_comes_from_terminal_completion_not_progress_logs() {
    let stream = valid_stream();
    let final_output = select_final_output(stream.as_slice()).expect("final output should select");
    let failed = select_final_output(&[AutomationEnvelope::validate(&visible_event(
        THREAD_SEQUENCE,
        AutomationEventKind::Failure,
        FAILURE_BODY,
    ))
    .expect("failure event should validate")])
    .expect_err("failed run should not produce final output");
    let missing = select_final_output(&[]).expect_err("empty stream should fail");

    assert_eq!(final_output.content(), FINAL_BODY);
    assert_ne!(final_output.content(), PROGRESS_BODY);
    assert_eq!(
        failed,
        AutomationError::RunFailed {
            reason: Text::from(FAILURE_BODY),
        }
    );
    assert_eq!(missing, AutomationError::MissingTerminalResponse);
}

#[test]
fn completion_receipts_validate_success_and_safe_failure_statuses() {
    let success = CompletionReceipt::validate(&CompletionReceiptDraft {
        status: CompletionStatus::Success.as_str(),
        evidence_references: &[EVIDENCE_REFERENCE],
        final_response: Some(FINAL_BODY),
    })
    .expect("success receipt should validate");
    let denied = CompletionReceipt::validate(&CompletionReceiptDraft {
        status: CompletionStatus::Denied.as_str(),
        evidence_references: &[EVIDENCE_REFERENCE],
        final_response: None,
    })
    .expect("denied receipt should validate");
    let missing_final = CompletionReceipt::validate(&CompletionReceiptDraft {
        status: CompletionStatus::Success.as_str(),
        evidence_references: &[EVIDENCE_REFERENCE],
        final_response: None,
    })
    .expect_err("success receipt needs final response");
    let duplicate_evidence = CompletionReceipt::validate(&CompletionReceiptDraft {
        status: CompletionStatus::ExecutionFailed.as_str(),
        evidence_references: &[EVIDENCE_REFERENCE, EVIDENCE_REFERENCE],
        final_response: None,
    })
    .expect_err("duplicate evidence should fail");

    assert_eq!(success.status(), CompletionStatus::Success);
    assert_eq!(success.final_response(), Some(FINAL_BODY));
    assert!(success.status().allows_guarded_dispatch());
    assert_eq!(denied.status(), CompletionStatus::Denied);
    assert!(!denied.status().allows_guarded_dispatch());
    assert_eq!(
        missing_final,
        AutomationError::EmptyField {
            field: FINAL_RESPONSE_FIELD
        }
    );
    assert_eq!(
        duplicate_evidence,
        AutomationError::DuplicateValue {
            field: EVIDENCE_REFERENCE_FIELD,
            value: Text::from(EVIDENCE_REFERENCE),
        }
    );
}

#[test]
fn unresolved_permission_ask_becomes_prompt_required_without_guarded_dispatch() {
    let receipt = prompt_required_completion(EVIDENCE_REFERENCE)
        .expect("prompt-required receipt should validate");

    assert_eq!(receipt.status(), CompletionStatus::PromptRequired);
    assert_eq!(
        receipt.evidence_references(),
        &[Text::from(EVIDENCE_REFERENCE)]
    );
    assert_eq!(receipt.final_response(), None);
    assert!(!receipt.status().allows_guarded_dispatch());
}
