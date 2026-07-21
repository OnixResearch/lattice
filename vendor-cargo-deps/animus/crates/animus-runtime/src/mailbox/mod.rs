//! Pure queued-message and interrupt control contracts.
//!
//! Hosts own UI affordances, transport cancellation, persistence, storage, clocks,
//! and model dispatch. This module keeps queue state transitions, interrupt
//! classification, and safe receipt construction deterministic over in-memory
//! inputs.

#![allow(ambiguous_params)]

mod core;
mod data;
mod digest;
mod errors;
mod record;

pub use core::{apply_message_queue_operation, classify_interrupt_signal};
pub use data::{
    ActiveWorkReference, ActiveWorkStatus, CancellationHandleClass, CancellationRequest,
    DispatchAcknowledgementDraft, ImmediateDispatchRequest, InterruptKeyword,
    InterruptQueueAdmission, InterruptSendMode, InterruptSignalDraft, MessageBudgetClass,
    MessageDispatch, MessageDispatchAcknowledgementOutcome, MessageEditDraft,
    MessageItemOperationDraft, MessageQueueEvent, MessageQueueOperation, MessageQueueOperationKind,
    MessageQueuePolicy, MessageQueuePolicyDraft, MessageQueueState, MessageQueueTransition,
    MessageRedactionState, MessageReorderDraft, MessageSafetyClass, MessageScopeOperationDraft,
    PauseRequest, QueueItemEvent, QueuedMessage, QueuedMessageDraft, RedirectionPayload,
    RedirectionPayloadDraft, UserControlEvent,
};
pub use digest::blake3_message_queue_hash;
pub use errors::{MessageQueueError, MessageQueueErrorKind};
pub use record::{
    MessageQueueReceipt, MessageQueueReceiptDraft, validate_message_queue_receipt_text,
};

pub(crate) const MESSAGE_QUEUE_SCHEMA: &str = "animus.message-queue-control.v1";
pub(crate) const MESSAGE_QUEUE_RECEIPT_SCHEMA: &str = "animus.message-queue-receipt.v1";
#[cfg(test)]
pub(crate) const DEFAULT_QUEUE_POLICY_LABEL: &str = "default-message-queue-policy";
pub(crate) const HASH_HEX_LENGTH: usize = 64;
pub(crate) const MIN_MAX_QUEUE_ITEMS: u32 = 1;
pub(crate) const MAX_SAFE_METADATA_CHARS: usize = 160;
pub(crate) const MIN_POLICY_SOURCE_COUNT: usize = 1;
pub(crate) const MAX_POLICY_SOURCE_COUNT: usize = 64;
pub(crate) const HASH_PART_SEPARATOR: char = '|';
pub(crate) const EMPTY_HASH_MARKER: &str = "none";

pub(crate) const FIELD_ACTIVE_WORK_ID: &str = "active_work_id";
pub(crate) const FIELD_ACTIVE_WORK_HASH: &str = "active_work_hash";
pub(crate) const FIELD_BODY_HASH: &str = "body_hash";
pub(crate) const FIELD_BUDGET_CLASS: &str = "budget_class";
pub(crate) const FIELD_CANCELLATION_HANDLE_CLASS: &str = "cancellation_handle_class";
pub(crate) const FIELD_DECISION_CLASS: &str = "decision_class";
pub(crate) const FIELD_DENIAL_CLASS: &str = "denial_class";
pub(crate) const FIELD_ITEM_ID: &str = "queue_item_id";
pub(crate) const FIELD_ITEM_HASH: &str = "queue_item_hash";
pub(crate) const FIELD_OPERATION_KIND: &str = "operation_kind";
pub(crate) const FIELD_POLICY_HASH: &str = "policy_hash";
pub(crate) const FIELD_POLICY_LABEL: &str = "policy_label";
pub(crate) const FIELD_RECEIPT_TEXT: &str = "receipt_text";
pub(crate) const FIELD_REDACTION_STATE: &str = "redaction_state";
pub(crate) const FIELD_REDIRECTION_HASH: &str = "redirection_hash";
pub(crate) const FIELD_SOURCE: &str = "source";

pub(crate) const LEAK_MARKERS: &[LeakMarker] = &[
    LeakMarker {
        needle: "raw_prompt:",
        marker: "raw_prompt",
    },
    LeakMarker {
        needle: "prompt:",
        marker: "raw_prompt",
    },
    LeakMarker {
        needle: "provider payload",
        marker: "provider_payload",
    },
    LeakMarker {
        needle: "provider_json",
        marker: "provider_payload",
    },
    LeakMarker {
        needle: "tool_output:",
        marker: "tool_output",
    },
    LeakMarker {
        needle: "command_output:",
        marker: "command_output",
    },
    LeakMarker {
        needle: "authorization:",
        marker: "credential",
    },
    LeakMarker {
        needle: "api_key",
        marker: "credential",
    },
    LeakMarker {
        needle: "secret=",
        marker: "credential",
    },
    LeakMarker {
        needle: "/home/",
        marker: "full_path",
    },
    LeakMarker {
        needle: "unbounded_queue_body",
        marker: "unbounded_queue_body",
    },
];

#[derive(Clone, Copy)]
pub(crate) struct LeakMarker {
    pub(crate) needle: &'static str,
    pub(crate) marker: &'static str,
}

#[cfg(test)]
mod tests;
