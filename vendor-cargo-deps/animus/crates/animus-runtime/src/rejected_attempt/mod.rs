//! Pure, VCS-neutral rejected-attempt context planning.
//!
//! Hosts supply validated review, current VCS, redaction, and classification
//! facts. This module does not inspect repositories, read files, build prompts,
//! create runs, mutate tasks, or grant retry or completion authority.

mod core;
mod data;

pub use core::{plan_context, project_receipt, validate_receipt};
// These aliases keep the established downstream API while concise names own the implementation.
#[cfg_attr(animus_octet, allow(tigerstyle::renamed_imports))]
pub use core::{
    plan_context as plan_rejected_attempt_context,
    project_receipt as project_rejected_attempt_receipt,
    validate_receipt as validate_rejected_attempt_receipt,
};
pub use data::{
    AttemptIdentity, ContentIdentity, ContextPlan, EvidenceReceipt, Issue, PlanningContract,
    RawDiffDisposition, ReceiptIdentity, RejectionContextFacts, Request, ReviewDisposition,
    ReviewEvidenceProfile, ReviewHandoffFacts, ReviewHandoffIdentity, RevisionIdentity,
    RunIdentity, SummaryIdentity, TaskIdentity, VcsFacts, VcsProvider, WorkspaceSnapshotIdentity,
};
// These type aliases preserve existing DTO paths without duplicating their definitions.
#[cfg_attr(animus_octet, allow(tigerstyle::renamed_imports))]
pub use data::{
    ContextPlan as RejectedAttemptContextPlan, EvidenceReceipt as RejectedAttemptReceipt,
    Issue as RejectedAttemptIssue, PlanningContract as RejectedAttemptPolicy,
    Request as RejectedAttemptRequest, VcsFacts as VcsAttemptFacts,
};

#[cfg(test)]
mod tests;
