//! Pure controller-continuation policy for runtime follow-up work.
//!
//! This module keeps continuation choice, acknowledgement validation, and safe
//! receipt construction deterministic. Host shells own provider calls, timers,
//! queue waits, persistence, UI events, and validation command execution.

#![allow(ambiguous_params)]

const PRIORITY_CANDIDATE_COUNT: usize = 3;
const CONTINUATION_POLICY_SCHEMA: &str = "animus.continuation.policy.v1";
const CONTINUATION_RECEIPT_SCHEMA: &str = "animus.continuation.receipt.v1";
const DEFAULT_POLICY_LABEL: &str = "default-continuation-priority";
const EMPTY_WORK_HASH_MARKER: &str = "no-pending-work";
const QUEUED_PROMPT_LABEL: &str = "queued_prompt";
const LOOP_CONTINUATION_LABEL: &str = "loop_continuation";
const VALIDATION_FOLLOW_UP_LABEL: &str = "validation_follow_up";
const BUSY_PROMPT_LABEL: &str = "busy_prompt";
const CANCELLATION_LABEL: &str = "cancellation";
const PENDING_DISPATCH_LABEL: &str = "pending_dispatch";
const REPLAY_QUEUED_PROMPT_LABEL: &str = "replay_queued_prompt";
const CONTINUE_LOOP_LABEL: &str = "continue_loop";
const RUN_VALIDATION_FOLLOW_UP_LABEL: &str = "run_validation_follow_up";
const AWAIT_COMPLETION_EVIDENCE_LABEL: &str = "await_completion_evidence";
const COMPLETE_LABEL: &str = "complete";
const CANCEL_LABEL: &str = "cancel";
const IDLE_LABEL: &str = "idle";
const BUSY_LABEL: &str = "busy";
const CANCELLED_LABEL: &str = "cancelled";
const QUEUED_USER_INPUT_REASON: &str = "queued_user_input";
const ACTIVE_LOOP_WORK_REASON: &str = "active_loop_work";
const VALIDATION_FOLLOW_UP_REASON: &str = "validation_follow_up";
const COMPLETION_EVIDENCE_REQUIRED_REASON: &str = "completion_evidence_required";
const NO_PENDING_WORK_REASON: &str = "no_pending_work";
const CANCELLATION_REQUESTED_REASON: &str = "cancellation_requested";
const FIELD_WORK_ID: &str = "work_id";
const FIELD_ACK_WORK_ID: &str = "acknowledgement_work_id";
const FIELD_POLICY_LABEL: &str = "policy_label";
const FIELD_PRIOR_STATE_CLASS: &str = "prior_state_class";
const FIELD_SELECTED_ACTION: &str = "selected_action";
const FIELD_PENDING_WORK_ID_HASH: &str = "pending_work_id_hash";
const FIELD_REASON: &str = "reason";
const FIELD_REJECTION_CLASS: &str = "rejection_class";
const FIELD_POLICY_HASH: &str = "policy_hash";

const LEAK_MARKERS: &[LeakMarker] = &[
    LeakMarker {
        needle: "raw_prompt:",
        marker: "raw_prompt",
    },
    LeakMarker {
        needle: "model_output:",
        marker: "model_output",
    },
    LeakMarker {
        needle: "tool_output:",
        marker: "tool_output",
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
        needle: "provider json",
        marker: "provider_payload",
    },
    LeakMarker {
        needle: "transcript:",
        marker: "transcript",
    },
    LeakMarker {
        needle: "full transcript",
        marker: "transcript",
    },
    LeakMarker {
        needle: "unbounded_log:",
        marker: "unbounded_log",
    },
    LeakMarker {
        needle: "unbounded_output:",
        marker: "unbounded_log",
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
];

type Text = String;
type Collection<T> = Vec<T>;

#[derive(Clone, Copy)]
struct LeakMarker {
    needle: &'static str,
    marker: &'static str,
}

include!("parts/kinds.rs");
include!("parts/work.rs");
include!("parts/state.rs");
include!("parts/decision.rs");
include!("parts/receipt.rs");
include!("parts/error.rs");
include!("parts/core.rs");

#[cfg(test)]
mod tests;
