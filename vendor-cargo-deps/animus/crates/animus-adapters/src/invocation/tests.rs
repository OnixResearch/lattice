use super::*;

const SAFE_MESSAGE: &str = "capability denied";
const SAFE_CORRELATION: &str = "request-7";
const SAFE_RECEIPT: &str = "receipt:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const UNSAFE_MESSAGE: &str = "RAW_PROMPT: summarize private text";
const BLANK_TEXT: &str = "   ";

#[test]
fn adapter_call_outcome_accepts_completed_and_failed_envelopes() {
    // r[verify agent_harness_runtime.adapter_outcomes.typed_envelope.validation]
    let response = crate::ModelResponse::new("model answer").expect("response should parse");
    let completed = AdapterCallOutcome::completed(response);
    let failed = AdapterCallOutcome::<crate::ModelResponse>::failed(
        AdapterCallFailureClass::Denied,
        AdapterCallFailureMetadataDraft {
            message: SAFE_MESSAGE,
            correlation_id: Some(SAFE_CORRELATION),
            receipt_reference: Some(SAFE_RECEIPT),
        },
    )
    .expect("safe failure metadata should parse");

    assert!(completed.is_completed());
    assert_eq!(completed.failure_class(), None);
    assert_eq!(failed.failure_class(), Some(AdapterCallFailureClass::Denied));
    assert_eq!(
        AdapterCallOutcome::<crate::ModelResponse>::contract_identity(),
        ADAPTER_CALL_OUTCOME_CONTRACT_IDENTITY
    );
    let AdapterCallOutcome::Failed { metadata, .. } = failed else {
        panic!("failed outcome should carry metadata");
    };
    assert_eq!(metadata.correlation_id(), Some(SAFE_CORRELATION));
    assert_eq!(metadata.receipt_reference(), Some(SAFE_RECEIPT));
}

#[test]
fn adapter_call_outcome_rejects_unsafe_or_empty_metadata() {
    // r[verify agent_harness_runtime.adapter_outcomes.typed_envelope.validation]
    let unsafe_metadata = AdapterCallOutcome::<crate::ToolResponse>::failed(
        AdapterCallFailureClass::ExecutionFailure,
        AdapterCallFailureMetadataDraft {
            message: UNSAFE_MESSAGE,
            correlation_id: None,
            receipt_reference: None,
        },
    )
    .expect_err("unsafe message should fail");
    let empty_correlation = AdapterCallOutcome::<crate::ToolResponse>::failed(
        AdapterCallFailureClass::TimedOut,
        AdapterCallFailureMetadataDraft {
            message: SAFE_MESSAGE,
            correlation_id: Some(BLANK_TEXT),
            receipt_reference: None,
        },
    )
    .expect_err("empty correlation should fail");

    assert!(matches!(unsafe_metadata, crate::AdapterContractError::UnsupportedStatusState { .. }));
    assert!(matches!(empty_correlation, crate::AdapterContractError::EmptyField { .. }));
}

#[test]
fn failure_classes_map_from_service_outcomes_without_treating_completed_as_failure() {
    // r[verify agent_harness_runtime.adapter_outcomes.compatibility.equivalent]
    assert_eq!(
        AdapterCallFailureClass::from_outcome_kind(crate::OutcomeKind::Absent),
        Some(AdapterCallFailureClass::Absent)
    );
    assert_eq!(
        AdapterCallFailureClass::from_outcome_kind(crate::OutcomeKind::MalformedOutput),
        Some(AdapterCallFailureClass::MalformedOutput)
    );
    assert_eq!(AdapterCallFailureClass::from_outcome_kind(crate::OutcomeKind::Completed), None);
}
