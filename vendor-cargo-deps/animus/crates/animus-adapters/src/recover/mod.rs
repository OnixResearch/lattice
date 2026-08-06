mod core;
mod evidence;
mod identity;
mod policy;
mod states;
mod taxonomy;
#[cfg(test)]
mod tests;

pub use core::CheckpointError;
pub use core::CheckpointHash;

pub use evidence::CheckpointReceipt;
pub use evidence::CheckpointReceiptDraft;
pub use evidence::CheckpointReceiptLeak;
pub use evidence::validate_checkpoint_receipt_text;
pub use identity::CheckpointDraft;
pub use identity::CheckpointRecord;
pub use policy::CheckpointService;
pub use policy::CheckpointServiceRequest;
pub use policy::CheckpointServiceResponse;
pub use policy::ProtectedMutationDecision;
pub use policy::ProtectedMutationInput;
pub use policy::RollbackValidation;
pub use policy::RollbackValidationInput;
pub use policy::evaluate_protected_mutation;
pub use policy::validate_rollback;
pub use states::CheckpointPolicy;
pub use states::CheckpointServiceState;
pub use states::MutationProtection;
pub use states::RollbackConfirmationStatus;
pub use taxonomy::CheckpointBackendKind;
pub use taxonomy::CheckpointFailureClass;
pub use taxonomy::CheckpointOperation;
pub use taxonomy::CheckpointStatus;

const HASH_HEX_CHARS: usize = 64;
const MAX_CHANGED_FILE_COUNT: u32 = 1_000_000;
const RECEIPT_SCHEMA: &str = "animus.checkpoint.receipt.v1";
const NO_HASH_SENTINEL: &str = "none";
const FIELD_SEPARATOR: &str = "|";
const KEY_VALUE_SEPARATOR: &str = "=";

const _: () = assert!(HASH_HEX_CHARS > 0);
const _: () = assert!(MAX_CHANGED_FILE_COUNT > 0);

const CHECKPOINT_BACKEND_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_backend_kind");
const CHECKPOINT_ID_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("checkpoint_id");
const CHECKPOINT_WORKSPACE_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_workspace_label");
const CHECKPOINT_WORKSPACE_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_workspace_hash");
const CHECKPOINT_BEFORE_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_before_hash");
const CHECKPOINT_AFTER_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_after_hash");
const CHECKPOINT_SESSION_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_session_hash");
const CHECKPOINT_TOOL_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_tool_label");
const CHECKPOINT_STATUS_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("checkpoint_status");
const CHECKPOINT_OPERATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_operation");
const CHECKPOINT_FAILURE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_failure_class");
const CHECKPOINT_CONFIRMATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_confirmation_status");
const CHECKPOINT_MUTATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_mutation_protection");
const CHECKPOINT_POLICY_FIELD: crate::validation::FieldName = crate::validation::FieldName::new("checkpoint_policy");
const CHECKPOINT_SERVICE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_service_state");
const CHECKPOINT_RECEIPT_HASH_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("checkpoint_receipt_hash");
