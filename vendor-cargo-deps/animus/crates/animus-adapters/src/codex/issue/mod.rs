mod loc;
mod proof;
mod record;

pub use loc::{DiffReviewSpan, DiffReviewSpanDraft};
pub use proof::{DiffReviewEvidence, DiffReviewEvidenceDraft, DiffReviewEvidenceKind};
pub use record::{
    DiffReviewFinding, DiffReviewFindingCategory, DiffReviewFindingDraft, DiffReviewFindingSeverity,
};
