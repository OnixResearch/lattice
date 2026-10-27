use super::super::*;
use super::support::*;

#[test]
fn rejects_unbounded_or_unsafe_payloads() {
    let full_file = DiffReviewEvidence::validate(&DiffReviewEvidenceDraft {
        kind: DiffReviewEvidenceKind::FullFile,
        ..evidence(TARGET_LABEL)
    })
    .expect_err("full files should not be embeddable evidence");
    assert_eq!(
        full_file,
        DiffReviewError::UnsupportedEvidenceKind {
            kind: DiffReviewEvidenceKind::FullFile,
        }
    );

    let too_many_lines_result = DiffReviewEvidence::validate(&DiffReviewEvidenceDraft {
        line_count: OVERSIZED_LINE_COUNT,
        ..evidence(TARGET_LABEL)
    })
    .expect_err("oversized snippets should fail");
    assert_eq!(
        too_many_lines_result,
        DiffReviewError::UnboundedEvidence {
            field: "diff_review_evidence_snippet",
            max_value: MAX_EVIDENCE_LINES_EXPECTED,
            actual_value: OVERSIZED_LINE_COUNT_EXPECTED,
        }
    );

    let unsafe_payload = DiffReviewEvidence::validate(&DiffReviewEvidenceDraft {
        snippet: RAW_PROMPT_SNIPPET,
        byte_count: RAW_PROMPT_SNIPPET_BYTES,
        ..evidence(TARGET_LABEL)
    })
    .expect_err("raw prompt material should fail");
    assert_eq!(
        unsafe_payload,
        DiffReviewError::UnsafeEvidence {
            field: "diff_review_evidence_snippet",
            marker: "RAW_PROMPT:",
        }
    );
}
