//! Bounded worker-team delegation contracts.
//!
//! Hosts own worker scheduling, processes, remote sessions, clocks, storage, and
//! cancellation mechanics. This module validates already-assembled worker task
//! data, context packets, reports, mutation intents, and injected worker-service
//! calls without discovering or launching anything by ambient authority.

#[allow(excessive_file_length)]
mod core;
mod data;
mod error;

pub use core::{
    collect_report, enqueue_task, merge_reports, request_cancel, request_status,
    validate_context_packet, validate_report, validate_task,
};
pub use data::{
    Claim, Conflict, ConflictReport, ContextAdmissionPolicy, EvidenceReference, MutationIntent,
    ParentTrace, Report, ReportMergeOutcome, WorkerAuthority, WorkerBudgets, WorkerCancelOutcome,
    WorkerCapability, WorkerContextItem, WorkerContextItemKind, WorkerContextPacket,
    WorkerContextRedaction, WorkerHandle, WorkerMergedReport, WorkerOutputSchema, WorkerScope,
    WorkerService, WorkerStatus, WorkerTask, WorkerTaskDraft,
};
pub use error::{DelegationFailure, FailureKind};

pub(crate) const TASK_ID_FIELD: &str = "task_id";
pub(crate) const OBJECTIVE_FIELD: &str = "objective";
pub(crate) const SCOPE_FIELD: &str = "scope";
pub(crate) const SCOPE_TARGET_FIELD: &str = "scope.allowed_targets";
pub(crate) const CONTEXT_PACKET_FIELD: &str = "context_packet";
pub(crate) const CONTEXT_ITEM_FIELD: &str = "context_packet.items";
pub(crate) const CAPABILITY_FIELD: &str = "allowed_capabilities";
pub(crate) const BUDGET_FIELD: &str = "budgets";
pub(crate) const OUTPUT_SCHEMA_FIELD: &str = "output_schema";
pub(crate) const OUTPUT_SCHEMA_REQUIRED_FIELD: &str = "output_schema.required_fields";
pub(crate) const PARENT_TRACE_FIELD: &str = "parent_trace";
pub(crate) const POLICY_HASH_FIELD: &str = "parent_trace.policy_hash";
pub(crate) const AUTHORITY_FIELD: &str = "authority";
pub(crate) const WORKER_SERVICE_FIELD: &str = "worker_service";
pub(crate) const WORKER_HANDLE_FIELD: &str = "worker_handle";
pub(crate) const WORKER_REPORT_FIELD: &str = "worker_report";
pub(crate) const WORKER_REPORT_FIELD_MAP: &str = "worker_report.fields";
pub(crate) const WORKER_EVIDENCE_FIELD: &str = "worker_report.evidence";
pub(crate) const WORKER_CLAIM_FIELD: &str = "worker_report.claims";
pub(crate) const WORKER_MUTATION_FIELD: &str = "worker_report.mutation_intents";

pub(crate) const MAX_LABEL_CHARS: usize = 128;
pub(crate) const MAX_TEXT_CHARS: usize = 4_096;
pub(crate) const MAX_CONTEXT_ITEMS: usize = 32;
pub(crate) const MAX_REPORT_FIELDS: usize = 32;
pub(crate) const BLAKE3_HEX_CHARS: usize = 64;
pub(crate) const MIN_BUDGET_VALUE: u16 = 1;
pub(crate) const WILDCARD_CAPABILITY_LABEL: &str = "*";
pub(crate) const WILDCARD_CAPABILITY_WORD: &str = "all";
pub(crate) const CONFLICT_CLAIM_CODE: &str = "worker-claim-conflict";
pub(crate) const CONFLICT_MUTATION_CODE: &str = "worker-mutation-conflict";
pub(crate) const MISSING_FIELD_REASON: &str = "required field is missing";
pub(crate) const EMPTY_FIELD_REASON: &str = "field must be non-empty";
pub(crate) const TOO_LONG_REASON: &str = "field exceeds safe length limit";
pub(crate) const UNSAFE_TEXT_REASON: &str = "field contains unsafe raw or leaking marker";
pub(crate) const OVERBROAD_REASON: &str = "authority must remain bounded by parent policy";
pub(crate) const MISSING_SERVICE_REASON: &str =
    "worker delegation requires an injected host service";
pub(crate) const MALFORMED_REPORT_REASON: &str = "worker report does not satisfy output schema";

pub(crate) const UNSAFE_TEXT_MARKERS: &[&str] = &[
    "RAW_PROMPT:",
    "PROVIDER_PAYLOAD:",
    "UNBOUNDED_OUTPUT:",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "AWS_SECRET_ACCESS_KEY",
    "sk-",
    "/home/",
];

#[cfg(test)]
mod tests;
