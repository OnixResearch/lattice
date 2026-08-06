use super::super::*;

pub const RUN_ID: &str = "review-run-1";
pub const NEXT_RUN_ID: &str = "review-run-2";
pub const TARGET_LABEL: &str = "src/lib.rs";
pub const SECOND_TARGET_LABEL: &str = "src/runtime.rs";
pub const COMMIT_REF: &str = "rev-abc123";
pub const BASE_REF: &str = "main";
pub const HEAD_REF: &str = "topic";
pub const PATCH_REF: &str = "patch-42";
pub const FINDING_ID: &str = "finding-1";
pub const NEXT_FINDING_ID: &str = "finding-2";
pub const RATIONALE: &str = "branch ignores the error result";
pub const REMEDIATION: &str = "return the error before continuing";
pub const EVIDENCE: &str = "if let Err(error) = result { continue; }";
pub const RAW_PROMPT_SNIPPET: &str = "RAW_PROMPT: hidden developer message";
pub const FIRST_LINE: u32 = 1;
pub const SECOND_LINE: u32 = 2;
pub const FIRST_COLUMN: u32 = 1;
pub const SECOND_COLUMN: u32 = 8;
pub const EVIDENCE_LINE_COUNT: u32 = 1;
pub const EVIDENCE_BYTE_COUNT: u64 = 40;
pub const RAW_PROMPT_SNIPPET_BYTES: u64 = 36;
pub const HIGH_CONFIDENCE: u8 = 90;
pub const MAX_CONFIDENCE_SCORE_EXPECTED: u8 = 100;
pub const OVER_CONFIDENCE_SCORE: u8 = 101;
pub const MAX_EVIDENCE_LINES_EXPECTED: u64 = 20;
pub const OVERSIZED_LINE_COUNT: u32 = 21;
pub const OVERSIZED_LINE_COUNT_EXPECTED: u64 = 21;
pub const EXPECTED_RERUN_RECORDS: usize = 3;
pub const OPEN_RECORD_INDEX: usize = 0;
pub const DISMISSED_RECORD_INDEX: usize = 1;
pub const NEW_RECORD_INDEX: usize = 2;
pub const FIXED_RECORD_INDEX: usize = 0;

#[derive(Clone, Copy)]
pub struct FindingInput<'a> {
    pub finding_id: &'a str,
    pub source_label: &'a str,
}

pub fn request(
    selector: DiffReviewSelectorDraft<'_>,
    capabilities: &[DiffReviewSelectorKind],
) -> Result<DiffReviewRequest, DiffReviewError> {
    DiffReviewRequest::validate(&DiffReviewRequestDraft {
        run_id: RUN_ID,
        selector,
        adapter_capabilities: capabilities,
        target_labels: &[TARGET_LABEL],
    })
}

pub fn report(run_id: &str, findings: &[DiffReviewFindingDraft<'_>]) -> Result<DiffReviewReport, DiffReviewError> {
    DiffReviewReport::validate(&DiffReviewReportDraft {
        run_id,
        reviewed_source_labels: &[TARGET_LABEL],
        findings,
        policy_denials: &[DiffReviewDenialReason::FileMutation],
    })
}

pub fn finding(input: FindingInput<'_>) -> DiffReviewFindingDraft<'_> {
    DiffReviewFindingDraft {
        finding_id: input.finding_id,
        severity: DiffReviewFindingSeverity::High,
        category: DiffReviewFindingCategory::Correctness,
        span: DiffReviewSpanDraft {
            source_label: input.source_label,
            start_line: FIRST_LINE,
            end_line: FIRST_LINE,
            start_column: Some(FIRST_COLUMN),
            end_column: Some(SECOND_COLUMN),
        },
        evidence: evidence(input.source_label),
        rationale: RATIONALE,
        confidence: HIGH_CONFIDENCE,
        remediation: REMEDIATION,
    }
}

pub fn evidence(source_label: &str) -> DiffReviewEvidenceDraft<'_> {
    DiffReviewEvidenceDraft {
        kind: DiffReviewEvidenceKind::PatchHunk,
        source_label,
        snippet: EVIDENCE,
        line_count: EVIDENCE_LINE_COUNT,
        byte_count: EVIDENCE_BYTE_COUNT,
    }
}
