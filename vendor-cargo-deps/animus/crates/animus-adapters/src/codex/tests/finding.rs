use super::super::*;
use super::support::*;

#[test]
fn structured_findings_validate_with_bounded_evidence() {
    let finding_record = DiffReviewFinding::validate(&finding(FindingInput {
        finding_id: FINDING_ID,
        source_label: TARGET_LABEL,
    }))
    .expect("valid finding should pass");
    assert_eq!(finding_record.finding_id(), FINDING_ID);
    assert_eq!(finding_record.severity(), DiffReviewFindingSeverity::High);
    assert_eq!(finding_record.category(), DiffReviewFindingCategory::Correctness);
    assert_eq!(finding_record.span().source_label(), TARGET_LABEL);
    assert_eq!(finding_record.evidence().snippet(), EVIDENCE);
    assert_eq!(finding_record.evidence().line_count(), EVIDENCE_LINE_COUNT);
    assert_eq!(finding_record.confidence(), HIGH_CONFIDENCE);
    assert_eq!(finding_record.remediation(), REMEDIATION);
    assert_eq!(finding_record.evidence().snippet_hash().len(), crate::artifact::BLAKE3_HASH_HEX_CHARS);
}

#[test]
fn reports_reject_invalid_spans_duplicates_and_out_of_scope_sources() {
    let reversed_span = DiffReviewFinding::validate(&DiffReviewFindingDraft {
        span: DiffReviewSpanDraft {
            source_label: TARGET_LABEL,
            start_line: SECOND_LINE,
            end_line: FIRST_LINE,
            start_column: None,
            end_column: None,
        },
        ..finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: TARGET_LABEL,
        })
    })
    .expect_err("reversed span should fail");
    assert_eq!(reversed_span, DiffReviewError::InvalidSpan {
        reason: "start line must not exceed end line",
    });

    let duplicate_findings = [
        finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: TARGET_LABEL,
        }),
        finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: TARGET_LABEL,
        }),
    ];
    let duplicate = DiffReviewReport::validate(&DiffReviewReportDraft {
        run_id: RUN_ID,
        reviewed_source_labels: &[TARGET_LABEL],
        findings: &duplicate_findings,
        policy_denials: &[],
    })
    .expect_err("duplicate finding ids should fail");
    assert_eq!(duplicate, DiffReviewError::DuplicateValue {
        field: "diff_review_finding_id",
        value: alloc::string::String::from(FINDING_ID),
    });

    let outside = DiffReviewReport::validate(&DiffReviewReportDraft {
        run_id: RUN_ID,
        reviewed_source_labels: &[TARGET_LABEL],
        findings: &[finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: SECOND_TARGET_LABEL,
        })],
        policy_denials: &[],
    })
    .expect_err("out-of-scope source should fail");
    assert_eq!(outside, DiffReviewError::FindingOutsideReviewScope {
        finding_id: alloc::string::String::from(FINDING_ID),
        source_label: alloc::string::String::from(SECOND_TARGET_LABEL),
    });
}

#[test]
fn invalid_confidence_and_missing_evidence_fail_validation() {
    let high_confidence = DiffReviewFinding::validate(&DiffReviewFindingDraft {
        confidence: OVER_CONFIDENCE_SCORE,
        ..finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: TARGET_LABEL,
        })
    })
    .expect_err("confidence over range should fail");
    assert_eq!(high_confidence, DiffReviewError::InvalidConfidence {
        max_score: MAX_CONFIDENCE_SCORE_EXPECTED,
        actual_score: OVER_CONFIDENCE_SCORE,
    });

    let missing_evidence = DiffReviewFinding::validate(&DiffReviewFindingDraft {
        evidence: DiffReviewEvidenceDraft {
            snippet: "",
            byte_count: 0,
            ..evidence(TARGET_LABEL)
        },
        ..finding(FindingInput {
            finding_id: FINDING_ID,
            source_label: TARGET_LABEL,
        })
    })
    .expect_err("empty evidence should fail");
    assert_eq!(
        missing_evidence,
        DiffReviewError::Contract(crate::AdapterContractError::EmptyField {
            field: "diff_review_evidence_snippet",
        })
    );
}
