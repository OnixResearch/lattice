//! Worker-team DTO groups.

mod context;
mod contract;
mod report;
mod service;
mod task;

pub use context::{
    ContextAdmissionPolicy, WorkerContextItem, WorkerContextItemKind, WorkerContextPacket,
    WorkerContextRedaction,
};
pub use contract::{
    ParentTrace, WorkerAuthority, WorkerBudgets, WorkerCapability, WorkerOutputSchema, WorkerScope,
};
pub use report::{
    Claim, Conflict, ConflictReport, EvidenceReference, MutationIntent, Report, ReportMergeOutcome,
    WorkerMergedReport,
};
pub use service::{WorkerCancelOutcome, WorkerHandle, WorkerService, WorkerStatus};
pub use task::{WorkerTask, WorkerTaskDraft};
