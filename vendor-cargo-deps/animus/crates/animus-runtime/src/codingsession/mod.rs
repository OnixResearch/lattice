//! Pure coding-session lifecycle composition over explicit effect requests.
//!
//! The reducer owns deterministic run, queue, durable-boundary, recovery,
//! cancellation, compaction, and terminal transitions. It never calls a model,
//! tool, store, timer, policy service, cancellation handle, summarizer, terminal,
//! filesystem, network, process, clock, or credential source.

#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::function_length,
        reason = "session reducers and adversarial fixtures return typed transition errors instead of panicking on untrusted facts"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::bool_naming,
        tigerstyle::numeric_units,
        reason = "published session and queue vocabulary uses field names fixed by its versioned contract"
    )
)]

mod core;
mod data;
#[path = "receipt.rs"]
mod evidence;

pub use core::{
    coding_session_effect_step_bound, coding_session_queue_precondition,
    coding_session_queue_text_identity, decode_coding_session_message,
    encode_coding_session_message, new_coding_session_state,
    plan_coding_session_authority_transition, project_coding_session_queue_conflict_event,
    project_coding_session_queue_safe_snapshot, project_coding_session_queue_success_event,
    reduce_coding_session_command, reduce_coding_session_frontend_event,
    reduce_coding_session_outcome,
};
pub use data::{
    CODING_SESSION_EFFECT_HASH_DOMAIN, CODING_SESSION_EMPTY_HEAD_HASH,
    CODING_SESSION_MESSAGE_HASH_DOMAIN, CODING_SESSION_RECEIPT_HASH_DOMAIN,
    CODING_SESSION_RECEIPT_SCHEMA, CodingSessionAfterCommit, CodingSessionCancellationRequest,
    CodingSessionCommand, CodingSessionCommitRequest, CodingSessionConfig,
    CodingSessionEffectOutcome, CodingSessionEffectRequest, CodingSessionEffectService,
    CodingSessionError, CodingSessionFrontendState, CodingSessionInput, CodingSessionInputPosture,
    CodingSessionIssue, CodingSessionMessage, CodingSessionMessageKind, CodingSessionProviderReply,
    CodingSessionProviderRequest, CodingSessionQueueAcceptedMutation, CodingSessionQueueBehavior,
    CodingSessionQueueConflict, CodingSessionQueueConflictKind, CodingSessionQueueGeneration,
    CodingSessionQueueMutationEvent, CodingSessionQueueMutationOperation,
    CodingSessionQueueMutationPrecondition, CodingSessionQueueMutationReceipt,
    CodingSessionQueueProfile, CodingSessionQueueReceiptError, CodingSessionQueueSafeEntry,
    CodingSessionQueueSafeSnapshot, CodingSessionQueueSensitiveDraft, CodingSessionReceipt,
    CodingSessionRecoveryPosture, CodingSessionRecoveryRequest, CodingSessionRecoverySnapshot,
    CodingSessionRecoveryStatus, CodingSessionRun, CodingSessionRunStatus,
    CodingSessionSensitiveDraft, CodingSessionServiceIssue, CodingSessionServicePosture,
    CodingSessionServiceSet, CodingSessionState, CodingSessionSummarizerRequest,
    CodingSessionTerminalKind, CodingSessionToolRequest, CodingSessionTransition,
};
pub use evidence::{
    project_coding_session_queue_conflict_receipt, project_coding_session_queue_success_receipt,
    project_coding_session_receipt,
};

#[cfg(test)]
mod tests;
