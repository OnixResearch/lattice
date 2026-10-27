//! Pure transactional daemon handoff contracts.
//!
//! The module validates strict idle, reduces one monotonic lease handoff,
//! binds an authenticated replacement claim, gates readiness on exact durable
//! state, and blocks rollback until complete owned process-tree teardown.
//! Hosts retain ingress, storage, lease, process, endpoint, clock, transport,
//! authentication, teardown, and worker-exit authority.

mod core;
mod data;
#[path = "evidence.rs"]
mod proof;

pub use core::{
    BLAKE3_HEX_BYTES, HANDOFF_PROTOCOL_VERSION, MAX_IDENTITY_BYTES, admit_claim,
    commit_cancellation, commit_readiness, complete_handoff, host_control_status, pause_ingress,
    plan_cancellation, prepare_handoff, project_termination_identity, release_lease, restore_lease,
    start_replacement, validate_handoff_transaction, validate_lease_restoration_preconditions,
    validate_strict_idle,
};
pub use data::{
    ActiveTurnGeneration, ActivityState, AuthenticationObservation, AuthorityProfileIdentity,
    BranchIdentity, CancellationCause, CancellationCommitObservation, CancellationIdentity,
    CancellationPlan, ClaimIdentity, ClaimantIdentity, CommitIdentity, CompletionObservation,
    ComposerDraftIdentity, DurableHandoffState, EndpointIdentity, Error, FreshnessNonce,
    HandoffBinding, HandoffIdentity, HandoffReceipt, HandoffState, HandoffTransaction,
    IngressPauseObservation, Issue, LeaseGeneration, LeaseReleaseObservation,
    LeaseRestoreObservation, ObservationIdentity, PolicyIdentity, ProcessAttemptIdentity,
    ProcessIdentity, ProcessStartObservation, ProcessTreeTerminationObservation, QueueGeneration,
    ReadinessCheck, ReadinessIdentity, ReadinessObservation, ReplacementClaim, SensitiveEvidence,
    ServiceReadinessIdentity, SessionHeadIdentity, SessionIdentity, SnapshotIdentity,
    StrictIdleDecision, StrictIdleSnapshot, TerminationIdentity, TransitionIdentity,
    TransitionSequence, VaultGeneration, WorkClass, WorkerIdentity,
};
pub use proof::{
    HANDOFF_REDACTION, NOT_PROCESS_MEMORY_CONTINUITY, NOT_PRODUCTION_READINESS, NOT_PTY_MIGRATION,
    NOT_SANDBOX_OR_REMOTE_SECURITY, project_handoff_receipt, validate_handoff_receipt,
};

#[cfg(test)]
mod tests;
