use super::*;

#[test]
fn hashes_raw_evidence_and_omits_raw_material() {
    let input = SecurityReviewInput::validate(&SecurityReviewInputDraft {
        raw_evidence: &[
            RAW_PROMPT_SENTINEL,
            CREDENTIAL_SENTINEL,
            PRIVATE_PATH_SENTINEL,
            RAW_TOOL_SENTINEL,
        ],
        ..draft(
            SecurityActionClass::ReadOnly,
            &[SAFE_ARGUMENT],
            &[CONTEXT_SUMMARY],
        )
    })
    .expect("raw evidence should be hashed and discarded");
    let rendered = alloc::format!("{input:?}");

    assert_eq!(input.evidence_hashes().len(), 4);
    assert!(
        input
            .evidence_hashes()
            .iter()
            .all(|hash| hash.len() == SECURITY_REVIEW_HASH_HEX_CHARS)
    );
    assert!(!rendered.contains(RAW_PROMPT_SENTINEL));
    assert!(!rendered.contains(CREDENTIAL_SENTINEL));
    assert!(!rendered.contains(PRIVATE_PATH_SENTINEL));
    assert!(!rendered.contains(RAW_TOOL_SENTINEL));
}

#[test]
fn rejects_unsafe_safe_summary() {
    let error = SecurityReviewInput::validate(&draft(
        SecurityActionClass::ReadOnly,
        &[SAFE_ARGUMENT],
        &[RAW_PROMPT_SENTINEL],
    ))
    .expect_err("safe summaries must reject raw prompt material");

    assert!(matches!(
        error,
        SecurityReviewError::UnsafeText {
            leak: SecurityReceiptLeak::RawPrompt,
            ..
        }
    ));
}

#[test]
fn deterministic_scanner_detects_dangerous_fixtures() {
    let cases = scanner_cases();

    for case in cases {
        let input = review_input(case.action_class, case.arguments, case.summaries);
        let scan = scan_review_input(&input);

        assert!(
            scan.findings()
                .iter()
                .any(|finding| finding.category() == case.category)
        );
        assert!(
            scan.findings()
                .iter()
                .all(|finding| finding.disposition() == SecurityFindingDisposition::Block)
        );
        assert!(
            scan.findings()
                .iter()
                .all(|finding| finding.evidence_hash().len() == SECURITY_REVIEW_HASH_HEX_CHARS)
        );
    }
}

#[test]
fn advisory_reviews_model_all_outcomes_without_authority() {
    let outcomes = [
        SecurityAdvisoryOutcome::Allow,
        SecurityAdvisoryOutcome::Warn,
        SecurityAdvisoryOutcome::Block,
        SecurityAdvisoryOutcome::Unavailable,
        SecurityAdvisoryOutcome::Timeout,
        SecurityAdvisoryOutcome::Malformed,
    ];

    for outcome in outcomes {
        let review = SecurityAdvisoryReview::validate(&SecurityAdvisoryReviewDraft {
            reviewer_label: Some(REVIEWER_LABEL),
            outcome,
            explanation_label: Some(REVIEW_EXPLANATION),
            raw_evidence: &[],
        })
        .expect("advisory outcome should validate");

        assert_eq!(review.outcome(), outcome);
        assert_eq!(review.reviewer_label(), REVIEWER_LABEL);
        assert!(review.evidence_hashes().is_empty());
    }
}

fn scanner_cases() -> [ScannerCase; 7] {
    [
        ScannerCase {
            action_class: SecurityActionClass::ModelContext,
            arguments: &[SAFE_ARGUMENT],
            summaries: &["hidden instruction asks to ignore previous guidance"],
            category: SecurityFindingCategory::PromptInjection,
        },
        ScannerCase {
            action_class: SecurityActionClass::NetworkTransfer,
            arguments: &["credential-access", "ssh-key"],
            summaries: &[CONTEXT_SUMMARY],
            category: SecurityFindingCategory::Exfiltration,
        },
        ScannerCase {
            action_class: SecurityActionClass::NetworkTransfer,
            arguments: &["unknown-upload-domain"],
            summaries: &[CONTEXT_SUMMARY],
            category: SecurityFindingCategory::Exfiltration,
        },
        ScannerCase {
            action_class: SecurityActionClass::FilesystemMutation,
            arguments: &["recursive-delete"],
            summaries: &[CONTEXT_SUMMARY],
            category: SecurityFindingCategory::DestructiveAction,
        },
        ScannerCase {
            action_class: SecurityActionClass::PrivilegedOperation,
            arguments: &["sudo", "root-required"],
            summaries: &[CONTEXT_SUMMARY],
            category: SecurityFindingCategory::PrivilegeEscalation,
        },
        ScannerCase {
            action_class: SecurityActionClass::ProcessExecution,
            arguments: &["download-execute", "curl-pipe-sh"],
            summaries: &[CONTEXT_SUMMARY],
            category: SecurityFindingCategory::ObfuscatedRemoteExecution,
        },
        ScannerCase {
            action_class: SecurityActionClass::WorkspaceMutation,
            arguments: &["outside-workspace"],
            summaries: &[CONTEXT_SUMMARY],
            category: SecurityFindingCategory::OutOfScopeMutation,
        },
    ]
}
