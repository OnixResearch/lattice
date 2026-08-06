//! Metadata-only active-turn recovery receipts.

// Receipt field names remain explicit and count conversion stays checked at the public boundary.
#![allow(
    unknown_lints,
    assertion_density,
    bool_naming,
    non_trait_imports,
    path_segment_repetition,
    usize_in_public_api
)]

use super::data::ACTIVE_TURN_RECEIPT_HASH_DOMAIN;
use super::data::ACTIVE_TURN_RECOVERY_SCHEMA;
use super::data::ActiveTurnRecoveryError;
use super::data::ActiveTurnRecoveryIssue;
use super::data::ActiveTurnRecoveryPlan;
use super::data::ActiveTurnRecoveryPolicy;
use super::data::ActiveTurnRecoveryReceipt;
use super::data::ActiveTurnRecoverySensitiveDraft;

const FIELD_RECEIPT: &str = "active_turn_recovery_receipt";
const REDACTION_MARKER: &str = "message-reasoning-tool-provider-store-and-path-material-omitted";
const NON_CLAIM: &str = "recovery does not prove store correctness, power-loss durability, provider acceptance, exactly-once effects, task completion, or production readiness";

/// Projects a bounded receipt from exact policy, checkpoint, and recovery facts.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for private material, unsupported claims,
/// malformed identities, or an entry-count overflow.
// r[impl agent_harness_runtime.active_turn_recovery.evidence]
pub fn project_active_turn_recovery_receipt(
    policy: &ActiveTurnRecoveryPolicy,
    plan: &ActiveTurnRecoveryPlan,
    entry_count: usize,
    sensitive: ActiveTurnRecoverySensitiveDraft<'_>,
) -> Result<ActiveTurnRecoveryReceipt, ActiveTurnRecoveryError> {
    reject_sensitive(sensitive)?;
    super::core::lowercase_blake3(policy.identity.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(plan.turn_hash.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(plan.checkpoint_hash.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(plan.plan_hash.as_str(), FIELD_RECEIPT)?;
    let entry_count = u32::try_from(entry_count)
        .map_err(|_error| super::core::error(ActiveTurnRecoveryIssue::InvalidInput, FIELD_RECEIPT))?;
    if entry_count > policy.max_entries() {
        return Err(super::core::error(ActiveTurnRecoveryIssue::InvalidInput, FIELD_RECEIPT));
    }
    let receipt_hash = super::core::hash_owned_parts(ACTIVE_TURN_RECEIPT_HASH_DOMAIN, &[
        ACTIVE_TURN_RECOVERY_SCHEMA.to_string(),
        policy.identity.clone(),
        plan.turn_hash.clone(),
        plan.checkpoint_hash.clone(),
        plan.plan_hash.clone(),
        plan.generation.to_string(),
        entry_count.to_string(),
        plan.action.as_str().to_string(),
        plan.terminal.as_str().to_string(),
        REDACTION_MARKER.to_string(),
        NON_CLAIM.to_string(),
    ]);
    Ok(ActiveTurnRecoveryReceipt {
        schema: ACTIVE_TURN_RECOVERY_SCHEMA,
        policy_hash: policy.identity.clone(),
        turn_hash: plan.turn_hash.clone(),
        checkpoint_hash: plan.checkpoint_hash.clone(),
        plan_hash: plan.plan_hash.clone(),
        generation: plan.generation,
        entry_count,
        action: plan.action,
        terminal: plan.terminal,
        redaction_marker: REDACTION_MARKER,
        non_claim: NON_CLAIM,
        receipt_hash,
    })
}

fn reject_sensitive(sensitive: ActiveTurnRecoverySensitiveDraft<'_>) -> Result<(), ActiveTurnRecoveryError> {
    let leaked = [
        sensitive.message,
        sensitive.reasoning,
        sensitive.tool_body,
        sensitive.provider_payload,
        sensitive.credential,
        sensitive.environment_value,
        sensitive.physical_path,
        sensitive.store_error,
        sensitive.unsupported_claim,
    ]
    .into_iter()
    .any(|value| value.is_some());
    if leaked {
        return Err(super::core::error(ActiveTurnRecoveryIssue::ReceiptLeak, FIELD_RECEIPT));
    }
    debug_assert!(!leaked);
    Ok(())
}
