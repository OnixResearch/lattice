//! Data-only dedicated Codex diff-review mode contracts.

mod check;
mod err;
mod gate;
mod input;
mod issue;
mod summary;

#[cfg(test)]
mod tests;

pub use err::DiffReviewError;
pub use gate::DiffReviewActionKind;
pub use gate::DiffReviewDenialReason;
pub use gate::DiffReviewPolicyDecision;
pub use gate::DiffReviewPolicyProfile;
pub use input::DiffReviewRequest;
pub use input::DiffReviewRequestDraft;
pub use input::DiffReviewSelector;
pub use input::DiffReviewSelectorDraft;
pub use input::DiffReviewSelectorKind;
pub use issue::DiffReviewEvidence;
pub use issue::DiffReviewEvidenceDraft;
pub use issue::DiffReviewEvidenceKind;
pub use issue::DiffReviewFinding;
pub use issue::DiffReviewFindingCategory;
pub use issue::DiffReviewFindingDraft;
pub use issue::DiffReviewFindingSeverity;
pub use issue::DiffReviewSpan;
pub use issue::DiffReviewSpanDraft;
pub use summary::DiffReviewFindingState;
pub use summary::DiffReviewFindingStateRecord;
pub use summary::DiffReviewReport;
pub use summary::DiffReviewReportDraft;
pub use summary::compare_diff_review_rerun;

pub(super) type Text = alloc::string::String;
pub(super) type Collection<T> = alloc::vec::Vec<T>;

pub(super) const RUN_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("diff_review_run_id");
pub(super) const SELECTOR_REF_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_selector_ref");
pub(super) const TARGET_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_target_label");
pub(super) const SOURCE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_source_label");
pub(super) const FINDING_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_finding_id");
pub(super) const FINDING_RATIONALE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_finding_rationale");
pub(super) const FINDING_REMEDIATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_finding_remediation");
pub(super) const EVIDENCE_SNIPPET_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_evidence_snippet");
pub(super) const DISMISSED_ID_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("diff_review_dismissed_id");

pub(super) const HASH_SCHEMA: &str = "animus.diff-review.v1";
pub(super) const HASH_SEPARATOR: &str = "|";
pub(super) const MAX_TARGET_LABELS: usize = 32;
pub(super) const MAX_REVIEW_FINDINGS: usize = 128;
pub(super) const MAX_EVIDENCE_LINES: u32 = 20;
pub(super) const MAX_EVIDENCE_BYTES: u64 = 4_096;
pub(super) const MAX_CONFIDENCE_SCORE: u8 = 100;
pub(super) const MIN_REVIEW_LINE: u32 = 1;
pub(super) const MIN_REVIEW_COLUMN: u32 = 1;

pub(super) const RAW_MATERIAL_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "UNBOUNDED_OUTPUT:",
    "BEGIN OPENSSH PRIVATE KEY",
    "AWS_SECRET_ACCESS_KEY",
    "/home/",
];

const _: () = assert!(MAX_TARGET_LABELS > 0);
const _: () = assert!(MAX_REVIEW_FINDINGS > 0);
const _: () = assert!(MAX_EVIDENCE_LINES > 0);
const _: () = assert!(MAX_EVIDENCE_BYTES > 0);
const _: () = assert!(MAX_CONFIDENCE_SCORE > 0);

/// Effects that dedicated diff-review mode must not perform by default.
pub const PROHIBITED_DIFF_REVIEW_EFFECTS: &[&str] = &[
    "file mutation",
    "write-capable tool dispatch",
    "approval escalation",
    "commit creation",
    "pull request management",
];

pub(super) fn hash_parts(parts: &[&str]) -> Text {
    let mut text = Text::new();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            text.push_str(HASH_SEPARATOR);
        }
        text.push_str(part);
    }
    alloc::format!("{}", blake3::hash(text.as_bytes()).to_hex())
}
