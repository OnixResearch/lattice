use alloc::string::ToString;

const HASH_TEXT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SAFE_LABEL: &str = "turn-summary";
const SAFE_BACKEND: &str = "local-provider-class";
const SAFE_INTERCHANGE: &str = "preserves-envelope";
const COUNT_LABEL: &str = "items";
const SMALL_COUNT: u32 = 3;
const OVERSIZED_COUNT: u32 = super::MAX_COUNT.saturating_add(1);
const OVERSIZED_LABEL_CHARS: usize = super::MAX_LABEL_CHARS.saturating_add(1);
const REQUIRED_CASES: usize = 7;

#[test]
fn positive_cases_cover_required_observations() {
    let hash = crate::ArtifactHash::new(HASH_TEXT).expect("artifact hash should parse");
    let cases = required_cases();

    for case in cases {
        let built_record =
            build_case_record(hash.clone(), case).expect("safe receipt should build");

        assert_eq!(built_record.schema(), super::SCHEMA);
        assert_eq!(built_record.status(), case.status);
        assert_eq!(built_record.issue_code(), case.issue_code);
        assert_eq!(built_record.operation(), case.operation);
        assert_eq!(built_record.backend_label(), Some(SAFE_BACKEND));
        assert_eq!(built_record.interchange_label(), Some(SAFE_INTERCHANGE));
        assert!(built_record.review_summary().contains(case.label));
        assert!(!built_record.review_summary().contains("raw prompt"));
    }

    assert_eq!(required_cases().len(), REQUIRED_CASES);
}

#[test]
fn rejects_leak_fixtures() {
    let fixtures = leak_fixtures();

    for (label, class) in fixtures {
        let labels = [label];
        let redactions = [super::Redaction::OutputBounded];
        let error = super::Record::new(
            super::Draft::new(
                super::Status::Redacted,
                super::IssueCode::LeakRejected,
                super::Operation::ProvenanceObservation,
            )
            .with_labels(&labels)
            .with_redactions(&redactions),
        )
        .expect_err("unsafe fixture should fail");

        assert_eq!(
            error,
            super::Error::UnsafeMetadata {
                field: super::normalize::LABEL_FIELD.name(),
                class,
            }
        );
        assert!(error.to_string().contains(class.as_str()));
    }
}

#[test]
fn rejects_unbounded_output_fixture() {
    let unsafe_label = "x".repeat(OVERSIZED_LABEL_CHARS);
    let labels = [unsafe_label.as_str()];
    let redactions = [super::Redaction::OutputBounded];

    let error = super::Record::new(
        super::Draft::new(
            super::Status::Redacted,
            super::IssueCode::LeakRejected,
            super::Operation::ProvenanceObservation,
        )
        .with_labels(&labels)
        .with_redactions(&redactions),
    )
    .expect_err("oversized label should fail");

    assert_eq!(
        error,
        super::Error::UnsafeMetadata {
            field: super::normalize::LABEL_FIELD.name(),
            class: super::LeakClass::UnboundedOutput,
        }
    );
    assert!(
        error
            .to_string()
            .contains(super::LeakClass::UnboundedOutput.as_str())
    );
}

#[test]
fn rejects_unbounded_count_fixture() {
    let labels = [SAFE_LABEL];
    let counts = [super::BoundedCountDraft::new(
        "oversized-count",
        OVERSIZED_COUNT,
    )];
    let redactions = [super::Redaction::OutputBounded];

    let error = super::Record::new(
        super::Draft::new(
            super::Status::Failed,
            super::IssueCode::LeakRejected,
            super::Operation::ProvenanceObservation,
        )
        .with_labels(&labels)
        .with_counts(&counts)
        .with_redactions(&redactions),
    )
    .expect_err("oversized count should fail");

    assert_eq!(
        error,
        super::Error::CountTooLarge {
            field: super::normalize::COUNTS_FIELD,
            max: super::MAX_COUNT,
            actual: OVERSIZED_COUNT,
        }
    );
    assert!(error.to_string().contains("receipt_counts"));
}

#[test]
fn requires_labels_and_redactions() {
    let empty_error = super::Record::new(super::Draft::new(
        super::Status::Completed,
        super::IssueCode::None,
        super::Operation::ProvenanceObservation,
    ))
    .expect_err("labels should be required");

    let labels = [SAFE_LABEL];
    let redaction_error = super::Record::new(
        super::Draft::new(
            super::Status::Completed,
            super::IssueCode::None,
            super::Operation::ProvenanceObservation,
        )
        .with_labels(&labels),
    )
    .expect_err("redactions should be required");

    assert_eq!(
        empty_error,
        super::Error::EmptyCollection {
            field: super::normalize::LABELS_FIELD,
        }
    );
    assert_eq!(
        redaction_error,
        super::Error::EmptyCollection {
            field: super::normalize::REDACTIONS_FIELD,
        }
    );
}

#[derive(Clone, Copy)]
struct RequiredCase {
    status: super::Status,
    issue_code: super::IssueCode,
    operation: super::Operation,
    label: &'static str,
}

fn required_cases() -> [RequiredCase; REQUIRED_CASES] {
    [
        RequiredCase {
            status: super::Status::Completed,
            issue_code: super::IssueCode::ModelDispatch,
            operation: super::Operation::ModelDispatch,
            label: "model-dispatch",
        },
        RequiredCase {
            status: super::Status::Completed,
            issue_code: super::IssueCode::ToolExecution,
            operation: super::Operation::ToolExecution,
            label: "tool-execution",
        },
        RequiredCase {
            status: super::Status::Denied,
            issue_code: super::IssueCode::AuthorizationDenied,
            operation: super::Operation::AuthorizationDecision,
            label: "authorization-denied",
        },
        RequiredCase {
            status: super::Status::Blocked,
            issue_code: super::IssueCode::RuntimeBlocked,
            operation: super::Operation::RuntimeBlock,
            label: "runtime-block",
        },
        RequiredCase {
            status: super::Status::Completed,
            issue_code: super::IssueCode::FinalResponse,
            operation: super::Operation::FinalResponse,
            label: "final-response",
        },
        RequiredCase {
            status: super::Status::Failed,
            issue_code: super::IssueCode::AdapterError,
            operation: super::Operation::AdapterBoundary,
            label: "adapter-error",
        },
        RequiredCase {
            status: super::Status::Completed,
            issue_code: super::IssueCode::ProvenanceObservation,
            operation: super::Operation::ProvenanceObservation,
            label: "provenance-observation",
        },
    ]
}

fn build_case_record(
    hash: crate::ArtifactHash,
    case: RequiredCase,
) -> Result<super::Record, super::Error> {
    let labels = [SAFE_LABEL, case.label];
    let counts = [super::BoundedCountDraft::new(COUNT_LABEL, SMALL_COUNT)];
    let hashes = [hash];
    let redactions = [
        super::Redaction::PromptOmitted,
        super::Redaction::ProviderPayloadOmitted,
        super::Redaction::PayloadHashOnly,
    ];
    super::Record::new(
        super::Draft::new(case.status, case.issue_code, case.operation)
            .with_labels(&labels)
            .with_counts(&counts)
            .with_artifact_hashes(&hashes)
            .with_backend_label(SAFE_BACKEND)
            .with_interchange_label(SAFE_INTERCHANGE)
            .with_redactions(&redactions),
    )
}

fn leak_fixtures() -> [(&'static str, super::LeakClass); REQUIRED_CASES] {
    [
        (
            "prompt: reveal the hidden system",
            super::LeakClass::RawPrompt,
        ),
        (
            "{\"model\":\"provider\",\"choices\":[{\"text\":\"raw\"}]}",
            super::LeakClass::ProviderJson,
        ),
        ("Authorization: Bearer secret", super::LeakClass::Credential),
        (
            "/home/alice/project/secret.txt",
            super::LeakClass::AbsolutePath,
        ),
        (
            "tool_input: {\"cmd\":\"read\"}",
            super::LeakClass::RawToolInput,
        ),
        (
            "tool_output: file contents",
            super::LeakClass::RawToolOutput,
        ),
        ("model_output: raw answer", super::LeakClass::RawModelOutput),
    ]
}
