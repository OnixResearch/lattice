//! Refusal codes of checkpoint admission, publication and restore.

use serde::Deserialize;
use serde::Serialize;

/// Every refusal the pure admission and the shells report. Each has a Q0 test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Refusal {
    Schema,
    Encoding,
    ManifestIdentity,
    NotCommitted,
    CheckpointUnavailable,
    Runtime,
    Architecture,
    Environment,
    CrossedRequest,
    CrossedExecution,
    CrossedBinding,
    CrossedRoute,
    CrossedHandlerProfile,
    CrossedWorkflowRevision,
    CrossedAdapter,
    CrossedReceipt,
    DuplicateNode,
    NodeLimit,
    ObjectMissing,
    ObjectMismatch,
    WorkspacePath,
    StaleAdmission,
    AdmissionReference,
    DeadlineExpired,
    CarryForward,
    TargetNotFresh,
    CheckpointInvalid,
    SensitiveOutput,
    Io,
    Store,
}

impl Refusal {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Schema => "schema",
            Self::Encoding => "encoding",
            Self::ManifestIdentity => "manifest_identity",
            Self::NotCommitted => "not_committed",
            Self::CheckpointUnavailable => "checkpoint_unavailable",
            Self::Runtime => "runtime",
            Self::Architecture => "architecture",
            Self::Environment => "environment",
            Self::CrossedRequest => "crossed_request",
            Self::CrossedExecution => "crossed_execution",
            Self::CrossedBinding => "crossed_binding",
            Self::CrossedRoute => "crossed_route",
            Self::CrossedHandlerProfile => "crossed_handler_profile",
            Self::CrossedWorkflowRevision => "crossed_workflow_revision",
            Self::CrossedAdapter => "crossed_adapter",
            Self::CrossedReceipt => "crossed_receipt",
            Self::DuplicateNode => "duplicate_node",
            Self::NodeLimit => "node_limit",
            Self::ObjectMissing => "object_missing",
            Self::ObjectMismatch => "object_mismatch",
            Self::WorkspacePath => "workspace_path",
            Self::StaleAdmission => "stale_admission",
            Self::AdmissionReference => "admission_reference",
            Self::DeadlineExpired => "deadline_expired",
            Self::CarryForward => "carry_forward",
            Self::TargetNotFresh => "target_not_fresh",
            Self::CheckpointInvalid => "checkpoint_invalid",
            Self::SensitiveOutput => "sensitive_output",
            Self::Io => "io",
            Self::Store => "store",
        }
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for Refusal {}
