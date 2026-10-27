//! Pure pending-entry recovery and injected host service contracts.
//!
//! The core receives explicit durable facts. It does not discover stores, read
//! bodies, inspect clocks, submit input, dispatch providers, or delete state.

mod core;
mod data;
mod evidence;
mod ports;

pub use core::{
    inspect_projection, ordinary_ingress_allowed, plan_disposition, plan_recovery_control,
    project_startup, reduce_commit, validate_boundary, validate_completed,
    validate_disposition_plan, validate_entry, validate_policy, validate_recovery_plan,
};
pub use data::{
    AdmissionRequest, AttachmentIdentity, AuthorityIdentity, BLAKE3_HEX_LENGTH, BodyIdentity,
    Boundary, BranchIdentity, ByteCount, CleanupState, CommitObservation, CommitOutcome,
    CommitStatus, CompletedDisposition, ConversationIdentity, CorruptionClass, DispositionDecision,
    DispositionIdentity, DispositionKind, DispositionPlan, DispositionRequest, Entry, EntryClass,
    EntryDraft, EntryIdentity, Error, Generation, Identity, Inspection, InspectionEntry, Issue,
    MAX_ATTACHMENT_COUNT, MAX_ENTRY_COUNT, MAX_METADATA_BYTES, PlanIdentity, Policy,
    PolicyIdentity, Projection, ProvenanceIdentity, Receipt, ReceiptIdentity, RecoveryControlKind,
    RecoveryControlPlan, RecoveryControlRequest, RecoveryState, SensitiveDraft, Sequence,
    StartupFacts, StoreIdentity,
};
pub use evidence::{project_receipt, validate_receipt};
pub use ports::{
    CleanupObservation, LoadRequest, LoadedBody, RecoveryControlObservation, Service, ServiceIssue,
    cleanup_completed, commit_plan, execute_recovery_control, load_admitted_body, load_startup,
};

#[cfg(test)]
mod tests;
