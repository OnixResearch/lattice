use alloc::string::ToString;

use super::support::*;

#[test]
fn orders_visible_entries_and_counts_metadata() {
    let receipt = sample_receipt();
    let entries = [
        super::super::Entry::user(USER_ID, USER_TEXT).expect("user entry should build"),
        super::super::Entry::assistant(ASSISTANT_ID, ASSISTANT_TEXT)
            .expect("assistant entry should build"),
        super::super::Entry::tool(TOOL_ID, TOOL_LABEL, TOOL_TEXT).expect("tool entry should build"),
        super::super::Entry::summary(SUMMARY_ID, SUMMARY_TEXT).expect("summary entry should build"),
        super::super::Entry::usage(USAGE_ID, INPUT_TOKENS, OUTPUT_TOKENS)
            .expect("usage entry should build"),
        super::super::Entry::safe_receipt(RECEIPT_ID, receipt).expect("receipt entry should build"),
    ];

    let replayed = super::super::replay(&entries).expect("ledger replay should succeed");
    let context = replayed.context();
    let counts = replayed.counts();

    assert_eq!(context.len(), LINEAR_CONTEXT_ITEMS);
    assert_eq!(
        context[FIRST_CONTEXT_INDEX].role(),
        super::super::ContextRole::User
    );
    assert_eq!(context[FIRST_CONTEXT_INDEX].content(), USER_TEXT);
    assert_eq!(
        context[SECOND_CONTEXT_INDEX].role(),
        super::super::ContextRole::Assistant
    );
    assert_eq!(
        context[THIRD_CONTEXT_INDEX].role(),
        super::super::ContextRole::Tool
    );
    assert_eq!(
        context[FOURTH_CONTEXT_INDEX].role(),
        super::super::ContextRole::System
    );
    assert_eq!(counts.entries(), LINEAR_ENTRY_COUNT);
    assert_eq!(counts.model_visible_items(), LINEAR_CONTEXT_COUNT);
    assert_eq!(counts.summaries(), EXPECTED_ONE_U32);
    assert_eq!(counts.usage_observations(), EXPECTED_ONE_U32);
    assert_eq!(counts.safe_receipts(), EXPECTED_ONE_U32);
}

#[test]
fn rejects_unsupported_entry_without_payload() {
    let entries = [
        super::super::Entry::unsupported(UNSUPPORTED_ID, UNSUPPORTED_KIND)
            .expect("unsupported marker should build"),
    ];

    let error = super::super::replay(&entries).expect_err("unsupported entry should fail closed");

    assert_eq!(
        error,
        super::super::Error::UnsupportedEntry {
            entry_id: alloc::string::String::from(UNSUPPORTED_ID),
            kind_label: alloc::string::String::from(UNSUPPORTED_KIND),
            redaction: crate::ReceiptRedaction::OutputBounded,
        }
    );
    assert!(!error.to_string().contains("payload"));
}

#[test]
fn rejects_adapter_owned_entry_without_payload() {
    let entries = [super::super::Entry::adapter_owned(ADAPTER_ID, ADAPTER_KIND)
        .expect("adapter marker should build")];

    let error = super::super::replay(&entries).expect_err("adapter-owned entry should fail closed");

    assert_eq!(
        error,
        super::super::Error::UnsupportedEntry {
            entry_id: alloc::string::String::from(ADAPTER_ID),
            kind_label: alloc::string::String::from(ADAPTER_KIND),
            redaction: crate::ReceiptRedaction::OutputBounded,
        }
    );
    assert!(!error.to_string().contains("raw"));
}

#[test]
fn empty_input_fails_closed() {
    let error = super::super::replay(&[]).expect_err("empty ledger should fail closed");

    assert_eq!(error, super::super::Error::EmptyLedger);
    assert_eq!(
        error.to_string(),
        "ledger replay requires at least one entry"
    );
}

#[test]
fn missing_service_and_session_errors_fail_closed() {
    let request = super::super::Request::new(SESSION_LABEL).expect("request should build");
    let outcome = super::super::missing_service(&request);

    assert_eq!(outcome.kind(), crate::OutcomeKind::Absent);
    assert_eq!(
        outcome.receipt().service_kind(),
        crate::ServiceKind::SessionLedger
    );
    assert_eq!(
        outcome.receipt().capability(),
        crate::CapabilityClass::SessionPersistence
    );
    assert!(!outcome.is_completed());
    assert_eq!(
        super::super::missing_service_error(&request),
        super::super::Error::MissingLedgerService {
            session_label: alloc::string::String::from(SESSION_LABEL),
        }
    );
    assert_eq!(
        super::super::missing_session_error(&request),
        super::super::Error::MissingSession {
            session_label: alloc::string::String::from(SESSION_LABEL),
        }
    );
    assert_eq!(
        super::super::unavailable_service_error(&request),
        super::super::Error::UnavailableLedgerService {
            session_label: alloc::string::String::from(SESSION_LABEL),
        }
    );
}
