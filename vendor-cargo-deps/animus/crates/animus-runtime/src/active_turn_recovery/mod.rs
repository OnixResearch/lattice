//! Pure active-turn durability and crash-recovery contracts.
//!
//! Hosts own storage, atomic transactions, clocks, providers, tools, cancellation,
//! and cleanup. This module validates generated policy, derives BLAKE3 identities,
//! plans atomic acceptance and checkpoint operations, gates continuation, and emits
//! metadata-only receipts. No recovery type can construct a tool request.

mod core;
mod data;
mod receipt;

pub use core::active_turn_checkpoint_hash;
pub use core::active_turn_provider_dispatch_allowed;
pub use core::active_turn_steering_delivery_allowed;
pub use core::evaluate_active_turn_acceptance;
pub use core::evaluate_active_turn_checkpoint_replacement;
pub use core::plan_active_turn_acceptance;
pub use core::plan_active_turn_recovery;
pub use core::plan_active_turn_steering_append;
pub use core::prepare_active_turn_checkpoint_replacement;
pub use core::validate_active_turn_recovery_policy;

pub use data::ACTIVE_TURN_CHECKPOINT_HASH_DOMAIN;
pub use data::ACTIVE_TURN_HASH_DOMAIN;
pub use data::ACTIVE_TURN_PLAN_HASH_DOMAIN;
pub use data::ACTIVE_TURN_POLICY_HASH_DOMAIN;
pub use data::ACTIVE_TURN_RECEIPT_HASH_DOMAIN;
pub use data::ACTIVE_TURN_RECOVERY_SCHEMA;
pub use data::ACTIVE_TURN_STEERING_HASH_DOMAIN;
pub use data::ACTIVE_TURN_TRANSACTION_HASH_DOMAIN;
pub use data::ActiveTurnAcceptanceDraft;
pub use data::ActiveTurnAcceptanceOutcome;
pub use data::ActiveTurnAcceptancePlan;
pub use data::ActiveTurnAcceptanceState;
pub use data::ActiveTurnAcceptanceStore;
pub use data::ActiveTurnCancellationState;
pub use data::ActiveTurnCheckpointStore;
pub use data::ActiveTurnCleanupState;
pub use data::ActiveTurnContinuationAuthority;
pub use data::ActiveTurnJournalEntryKind;
pub use data::ActiveTurnPreparedEntry;
pub use data::ActiveTurnRecoveryAction;
pub use data::ActiveTurnRecoveryError;
pub use data::ActiveTurnRecoveryIssue;
pub use data::ActiveTurnRecoveryPlan;
pub use data::ActiveTurnRecoveryPolicy;
pub use data::ActiveTurnRecoveryPolicyInput;
pub use data::ActiveTurnRecoveryReceipt;
pub use data::ActiveTurnRecoverySensitiveDraft;
pub use data::ActiveTurnRecoverySnapshot;
pub use data::ActiveTurnRepairState;
pub use data::ActiveTurnReplacementDraft;
pub use data::ActiveTurnReplacementPlan;
pub use data::ActiveTurnSteeringDraft;
pub use data::ActiveTurnSteeringPlan;
pub use data::ActiveTurnSteeringStore;
pub use data::ActiveTurnTerminalClass;
pub use data::ActiveTurnTransactionObservation;
pub use data::ActiveTurnTransactionStatus;
pub use receipt::project_active_turn_recovery_receipt;

#[cfg(test)]
mod tests;
