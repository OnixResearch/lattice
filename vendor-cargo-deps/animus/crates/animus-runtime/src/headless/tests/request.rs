use super::super::*;
use super::support::*;

#[test]
fn run_request_accepts_valid_noninteractive_inputs() {
    let request = RunRequest::validate(&RunRequestDraft {
        output_mode: OUTPUT_STRUCTURED,
        structured_schema_id: Some(SCHEMA_ID),
        resume: Some(ThreadResumeDraft {
            thread_id: THREAD_ID,
            ledger_hash: LEDGER_HASH,
            expected_ledger_hash: Some(LEDGER_HASH),
        }),
        ..base_request()
    })
    .expect("request should validate");

    assert_eq!(request.objective(), OBJECTIVE);
    assert_eq!(request.identity().session_id(), Some(SESSION_ID));
    assert_eq!(request.identity().workspace_id(), Some(WORKSPACE_ID));
    assert_eq!(request.input_references(), &[Text::from(INPUT_REFERENCE)]);
    assert_eq!(request.prompt(), PROMPT);
    assert_eq!(
        request.external_context_handles(),
        &[Text::from(CONTEXT_HANDLE)]
    );
    assert_eq!(request.policy_profile(), POLICY_PROFILE);
    assert_eq!(
        request.allowed_capabilities(),
        &[Text::from(CAPABILITY_READ)]
    );
    assert_eq!(request.sandbox_profile(), SandboxProfile::ReadOnlyWorkspace);
    assert_eq!(request.output_preference().mode_label(), OUTPUT_STRUCTURED);
    assert_eq!(request.output_preference().schema_id(), Some(SCHEMA_ID));
    assert_eq!(request.validation_plan(), VALIDATION_PLAN);
    assert_eq!(
        request.noninteractive_behavior(),
        NoninteractiveBehavior::FailClosed
    );
    assert_eq!(
        request.resume().map(ThreadResume::thread_id),
        Some(THREAD_ID)
    );
}

#[test]
fn missing_contract_fields_fail_before_execution() {
    let objective_error = RunRequest::validate(&RunRequestDraft {
        objective: " ",
        ..base_request()
    })
    .expect_err("missing objective should fail");
    let identity_error = RunRequest::validate(&RunRequestDraft {
        session_id: " ",
        workspace_id: " ",
        ..base_request()
    })
    .expect_err("missing identity should fail");
    let policy_error = RunRequest::validate(&RunRequestDraft {
        policy_profile: " ",
        ..base_request()
    })
    .expect_err("missing policy profile should fail");

    assert_eq!(
        objective_error,
        AutomationError::EmptyField {
            field: OBJECTIVE_FIELD
        }
    );
    assert_eq!(
        identity_error,
        AutomationError::EmptyField {
            field: RUN_IDENTITY_FIELD
        }
    );
    assert_eq!(
        policy_error,
        AutomationError::EmptyField {
            field: POLICY_PROFILE_FIELD
        }
    );
}

#[test]
fn malformed_contract_values_fail_before_execution() {
    let output_contract_error = RunRequest::validate(&RunRequestDraft {
        output_mode: OUTPUT_STRUCTURED,
        structured_schema_id: None,
        ..base_request()
    })
    .expect_err("missing structured schema should fail");
    let behavior_error = RunRequest::validate(&RunRequestDraft {
        noninteractive_behavior: "ask_user",
        ..base_request()
    })
    .expect_err("unsupported noninteractive behavior should fail");
    let input_reference_error = RunRequest::validate(&RunRequestDraft {
        input_references: &[INPUT_REFERENCE, DUPLICATE_INPUT_REFERENCE],
        ..base_request()
    })
    .expect_err("duplicate input reference should fail");

    assert_eq!(
        output_contract_error,
        AutomationError::EmptyField {
            field: SCHEMA_ID_FIELD
        }
    );
    assert_eq!(
        behavior_error,
        AutomationError::UnsupportedValue {
            field: NONINTERACTIVE_BEHAVIOR_FIELD,
            value: Text::from("ask_user"),
        }
    );
    assert_eq!(
        input_reference_error,
        AutomationError::DuplicateValue {
            field: INPUT_REFERENCE_FIELD,
            value: Text::from(DUPLICATE_INPUT_REFERENCE),
        }
    );
}

#[test]
fn run_request_rejects_missing_prompt_invalid_sandbox_and_resume_mismatch() {
    let missing_prompt = RunRequest::validate(&RunRequestDraft {
        prompt: " ",
        ..base_request()
    })
    .expect_err("missing prompt should fail");
    let invalid_sandbox = RunRequest::validate(&RunRequestDraft {
        sandbox_profile: "ambient-host",
        ..base_request()
    })
    .expect_err("invalid sandbox should fail");
    let resume_mismatch = RunRequest::validate(&RunRequestDraft {
        resume: Some(ThreadResumeDraft {
            thread_id: THREAD_ID,
            ledger_hash: LEDGER_HASH,
            expected_ledger_hash: Some(OTHER_LEDGER_HASH),
        }),
        ..base_request()
    })
    .expect_err("resume mismatch should fail");
    let duplicate_context = RunRequest::validate(&RunRequestDraft {
        external_context_handles: &[CONTEXT_HANDLE, DUPLICATE_CONTEXT],
        ..base_request()
    })
    .expect_err("duplicate context handle should fail");
    let duplicate_capability = RunRequest::validate(&RunRequestDraft {
        allowed_capabilities: &[CAPABILITY_READ, DUPLICATE_CAPABILITY],
        ..base_request()
    })
    .expect_err("duplicate capability should fail");

    assert_eq!(
        missing_prompt,
        AutomationError::EmptyField {
            field: PROMPT_FIELD
        }
    );
    assert_eq!(
        invalid_sandbox,
        AutomationError::UnsupportedValue {
            field: SANDBOX_FIELD,
            value: Text::from("ambient-host"),
        }
    );
    assert_eq!(
        resume_mismatch,
        AutomationError::ResumeMismatch {
            thread_id: Text::from(THREAD_ID),
            expected_ledger_hash: Text::from(OTHER_LEDGER_HASH),
            actual_ledger_hash: Text::from(LEDGER_HASH),
        }
    );
    assert_eq!(
        duplicate_context,
        AutomationError::DuplicateValue {
            field: CONTEXT_FIELD,
            value: Text::from(DUPLICATE_CONTEXT),
        }
    );
    assert_eq!(
        duplicate_capability,
        AutomationError::DuplicateValue {
            field: CAPABILITY_FIELD,
            value: Text::from(DUPLICATE_CAPABILITY),
        }
    );
}
