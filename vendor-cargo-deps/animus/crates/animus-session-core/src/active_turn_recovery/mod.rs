//! Pure active-turn durability and crash-recovery contracts.
//!
//! Hosts own storage, atomic transactions, clocks, providers, tools, cancellation,
//! and cleanup. This module validates generated policy, derives BLAKE3 identities,
//! plans atomic acceptance and checkpoint operations, gates continuation, and emits
//! metadata-only receipts. No recovery type can construct a tool request.

#[allow(unused_imports)]
use crate::prelude::*;

mod core;
mod data;
mod receipt;

pub use core::{
    active_turn_checkpoint_hash, active_turn_provider_dispatch_allowed,
    active_turn_steering_delivery_allowed, evaluate_active_turn_acceptance,
    evaluate_active_turn_checkpoint_replacement, plan_active_turn_acceptance,
    plan_active_turn_recovery, plan_active_turn_steering_append,
    prepare_active_turn_checkpoint_replacement, prepare_active_turn_control_entry,
    validate_active_turn_recovery_policy,
};
pub use data::{
    ACTIVE_TURN_CHECKPOINT_HASH_DOMAIN, ACTIVE_TURN_CONTROL_HASH_DOMAIN, ACTIVE_TURN_HASH_DOMAIN,
    ACTIVE_TURN_PLAN_HASH_DOMAIN, ACTIVE_TURN_POLICY_HASH_DOMAIN, ACTIVE_TURN_RECEIPT_HASH_DOMAIN,
    ACTIVE_TURN_RECOVERY_SCHEMA, ACTIVE_TURN_STEERING_HASH_DOMAIN,
    ACTIVE_TURN_TRANSACTION_HASH_DOMAIN, ActiveTurnAcceptanceDraft, ActiveTurnAcceptanceOutcome,
    ActiveTurnAcceptancePlan, ActiveTurnAcceptanceState, ActiveTurnAcceptanceStore,
    ActiveTurnCancellationState, ActiveTurnCheckpointStore, ActiveTurnCleanupState,
    ActiveTurnContinuationAuthority, ActiveTurnControlEntryDraft, ActiveTurnJournalEntryKind,
    ActiveTurnPreparedEntry, ActiveTurnRecoveryAction, ActiveTurnRecoveryError,
    ActiveTurnRecoveryIssue, ActiveTurnRecoveryPlan, ActiveTurnRecoveryPolicy,
    ActiveTurnRecoveryPolicyInput, ActiveTurnRecoveryReceipt, ActiveTurnRecoverySensitiveDraft,
    ActiveTurnRecoverySnapshot, ActiveTurnRepairState, ActiveTurnReplacementDraft,
    ActiveTurnReplacementPlan, ActiveTurnSteeringDraft, ActiveTurnSteeringPlan,
    ActiveTurnSteeringStore, ActiveTurnTerminalClass, ActiveTurnTransactionObservation,
    ActiveTurnTransactionStatus,
};
pub use receipt::project_active_turn_recovery_receipt;

#[cfg(test)]
mod tests;
