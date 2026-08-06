//! Pure coding-session lifecycle composition over explicit effect requests.
//!
//! The reducer owns deterministic run, queue, durable-boundary, recovery,
//! cancellation, compaction, and terminal transitions. It never calls a model,
//! tool, store, timer, policy service, cancellation handle, summarizer, terminal,
//! filesystem, network, process, clock, or credential source.

mod core;
mod data;
#[path = "receipt.rs"]
mod evidence;

pub use core::coding_session_effect_step_bound;
pub use core::decode_coding_session_message;
pub use core::encode_coding_session_message;
pub use core::new_coding_session_state;
pub use core::plan_coding_session_authority_transition;
pub use core::reduce_coding_session_command;
pub use core::reduce_coding_session_frontend_event;
pub use core::reduce_coding_session_outcome;

pub use data::CODING_SESSION_EFFECT_HASH_DOMAIN;
pub use data::CODING_SESSION_EMPTY_HEAD_HASH;
pub use data::CODING_SESSION_MESSAGE_HASH_DOMAIN;
pub use data::CODING_SESSION_RECEIPT_HASH_DOMAIN;
pub use data::CODING_SESSION_RECEIPT_SCHEMA;
pub use data::CodingSessionAfterCommit;
pub use data::CodingSessionCancellationRequest;
pub use data::CodingSessionCommand;
pub use data::CodingSessionCommitRequest;
pub use data::CodingSessionConfig;
pub use data::CodingSessionEffectOutcome;
pub use data::CodingSessionEffectRequest;
pub use data::CodingSessionEffectService;
pub use data::CodingSessionError;
pub use data::CodingSessionFrontendState;
pub use data::CodingSessionInput;
pub use data::CodingSessionInputPosture;
pub use data::CodingSessionIssue;
pub use data::CodingSessionMessage;
pub use data::CodingSessionMessageKind;
pub use data::CodingSessionProviderReply;
pub use data::CodingSessionProviderRequest;
pub use data::CodingSessionQueueBehavior;
pub use data::CodingSessionQueueProfile;
pub use data::CodingSessionReceipt;
pub use data::CodingSessionRecoveryPosture;
pub use data::CodingSessionRecoveryRequest;
pub use data::CodingSessionRecoverySnapshot;
pub use data::CodingSessionRecoveryStatus;
pub use data::CodingSessionRun;
pub use data::CodingSessionRunStatus;
pub use data::CodingSessionSensitiveDraft;
pub use data::CodingSessionServiceIssue;
pub use data::CodingSessionServicePosture;
pub use data::CodingSessionServiceSet;
pub use data::CodingSessionState;
pub use data::CodingSessionSummarizerRequest;
pub use data::CodingSessionTerminalKind;
pub use data::CodingSessionToolRequest;
pub use data::CodingSessionTransition;
pub use evidence::project_coding_session_receipt;

#[cfg(test)]
mod tests;
