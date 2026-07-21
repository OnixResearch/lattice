use super::*;

mod confirmation;

const VALID_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const UPPERCASE_HASH: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const SHORT_HASH: &str = "abcd";
const NO_PAYLOADS: u32 = 0;

#[test]
fn safe_hash_accepts_lowercase_blake3_hex() {
    let hash = SafeHash::new(VALID_HASH).expect("valid effect hash should parse");

    assert_eq!(hash.as_str(), VALID_HASH);
}

#[test]
fn safe_hash_rejects_uppercase_hex() {
    let error = SafeHash::new(UPPERCASE_HASH).expect_err("uppercase effect hash should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::InvalidEvidenceHash {
            field: super::audit::test_support::HASH_FIELD.as_str(),
            reason: super::audit::test_support::HASH_NOT_HEX_REASON,
        }
    );
}

#[test]
fn safe_hash_rejects_wrong_length() {
    let error = SafeHash::new(SHORT_HASH).expect_err("short effect hash should fail");

    assert_eq!(
        error,
        crate::AdapterContractError::InvalidEvidenceHash {
            field: super::audit::test_support::HASH_FIELD.as_str(),
            reason: super::audit::test_support::HASH_LENGTH_REASON,
        }
    );
}

#[test]
fn service_outcome_marks_non_completed_outcomes_as_blocking() {
    let outcomes: [ServiceOutcome<()>; 6] = [
        ServiceOutcome::absent(blocking_receipt(OutcomeKind::Absent)),
        ServiceOutcome::unavailable(blocking_receipt(OutcomeKind::Unavailable)),
        ServiceOutcome::denied(None, blocking_receipt(OutcomeKind::Denied)),
        ServiceOutcome::cancelled(blocking_receipt(OutcomeKind::Cancelled)),
        ServiceOutcome::timed_out(blocking_receipt(OutcomeKind::TimedOut)),
        ServiceOutcome::malformed_output(blocking_receipt(OutcomeKind::MalformedOutput)),
    ];

    for outcome in outcomes {
        assert!(!outcome.is_completed());
        assert!(!outcome.receipt().outcome().is_completed());
    }
}

#[test]
fn completed_service_outcome_carries_completed_receipt() {
    let receipt = Receipt::new(
        ReceiptDraft::new(
            ServiceKind::ModelProvider,
            CapabilityClass::ModelDispatch,
            OutcomeKind::Completed,
            Reason::Completed,
        )
        .with_input_hash(SafeHash::new(VALID_HASH).expect("input hash should parse"))
        .with_output_hash(SafeHash::new(VALID_HASH).expect("output hash should parse"))
        .with_payload_count(NO_PAYLOADS)
        .with_redaction(RedactionMarker::PayloadHashOnly),
    );
    let outcome = ServiceOutcome::completed((), receipt);

    assert!(outcome.is_completed());
    assert_eq!(outcome.kind(), OutcomeKind::Completed);
}

fn blocking_receipt(outcome: OutcomeKind) -> Receipt {
    Receipt::new(
        ReceiptDraft::new(
            ServiceKind::ToolExecutor,
            CapabilityClass::ToolExecution,
            outcome,
            blocking_reason(outcome),
        )
        .with_payload_count(NO_PAYLOADS)
        .with_redaction(RedactionMarker::PayloadNotCaptured),
    )
}

const fn blocking_reason(outcome: OutcomeKind) -> Reason {
    match outcome {
        OutcomeKind::Absent => Reason::NotInjected,
        OutcomeKind::Unavailable => Reason::HostUnavailable,
        OutcomeKind::Denied => Reason::CapabilityDenied,
        OutcomeKind::Cancelled => Reason::Cancelled,
        OutcomeKind::TimedOut => Reason::DeadlineExceeded,
        OutcomeKind::MalformedOutput => Reason::InvalidBoundaryPayload,
        OutcomeKind::Completed => Reason::Completed,
    }
}
