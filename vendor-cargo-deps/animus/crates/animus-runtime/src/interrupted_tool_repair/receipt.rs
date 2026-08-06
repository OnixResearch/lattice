//! Metadata-only interrupted-tool repair receipts.

const INTERRUPTED_TOOL_RECEIPT_HASH_DOMAIN: &str = super::data::INTERRUPTED_TOOL_RECEIPT_HASH_DOMAIN;
const INTERRUPTED_TOOL_REPAIR_SCHEMA: &str = super::data::INTERRUPTED_TOOL_REPAIR_SCHEMA;
const FIELD_RECEIPT: super::core::RepairField = super::core::RepairField("interrupted_tool_repair_receipt");

type InterruptedToolReason = super::data::InterruptedToolReason;
type InterruptedToolRepairApplicationOutcome = super::data::InterruptedToolRepairApplicationOutcome;
type InterruptedToolRepairError = super::data::InterruptedToolRepairError;
type InterruptedToolRepairIssue = super::data::InterruptedToolRepairIssue;
type InterruptedToolRepairPlan = super::data::InterruptedToolRepairPlan;
type InterruptedToolRepairReceipt = super::data::InterruptedToolRepairReceipt;
type InterruptedToolRepairSensitiveDraft<'a> = super::data::InterruptedToolRepairSensitiveDraft<'a>;
const REDACTION_MARKER: &str = "raw-transcript-tool-and-provider-material-omitted";
const NON_CLAIM: &str =
    "repair does not prove interruption cause, provider acceptance, or exactly-once external effects";

/// Projects a bounded receipt from an immutable plan and host application outcome.
///
/// # Errors
///
/// Returns [`InterruptedToolRepairError`] for mismatched plan/application data,
/// malformed identities, count overflow, or any supplied private material.
// r[impl agent_harness_runtime.tau_tool_repair.receipts]
pub fn project_interrupted_tool_repair_receipt(
    plan: &InterruptedToolRepairPlan,
    outcome: &InterruptedToolRepairApplicationOutcome,
    sensitive: InterruptedToolRepairSensitiveDraft<'_>,
) -> Result<InterruptedToolRepairReceipt, InterruptedToolRepairError> {
    reject_sensitive(sensitive)?;
    validate_sources(plan, outcome)?;
    let action_count = u32::try_from(plan.actions.len())
        .map_err(|_error| super::core::error(InterruptedToolRepairIssue::InvalidInput, FIELD_RECEIPT))?;
    let mut reason_classes = plan.actions.iter().map(|action| action.reason).collect::<Vec<InterruptedToolReason>>();
    reason_classes.sort_unstable();
    reason_classes.dedup();
    let reason_labels = reason_classes.iter().map(|reason| reason.as_str().to_string()).collect::<Vec<_>>();
    let receipt_hash = super::core::hash_owned_parts(
        INTERRUPTED_TOOL_RECEIPT_HASH_DOMAIN,
        [
            vec![
                INTERRUPTED_TOOL_REPAIR_SCHEMA.to_string(),
                outcome.status.as_str().to_string(),
                plan.mode.as_str().to_string(),
                plan.old_leaf_hash.clone(),
                outcome.new_leaf_hash.clone(),
                plan.plan_hash.clone(),
                action_count.to_string(),
                REDACTION_MARKER.to_string(),
                NON_CLAIM.to_string(),
            ],
            reason_labels,
        ]
        .concat()
        .as_slice(),
    );
    debug_assert!(!receipt_hash.is_empty());
    Ok(InterruptedToolRepairReceipt {
        schema: INTERRUPTED_TOOL_REPAIR_SCHEMA,
        status: outcome.status,
        mode: plan.mode,
        old_leaf_hash: plan.old_leaf_hash.clone(),
        new_leaf_hash: outcome.new_leaf_hash.clone(),
        plan_hash: plan.plan_hash.clone(),
        action_count,
        reason_classes,
        redaction_marker: REDACTION_MARKER,
        non_claim: NON_CLAIM,
        receipt_hash,
    })
}

fn validate_sources(
    plan: &InterruptedToolRepairPlan,
    outcome: &InterruptedToolRepairApplicationOutcome,
) -> Result<(), InterruptedToolRepairError> {
    super::core::lowercase_blake3(plan.plan_hash.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(plan.old_leaf_hash.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(plan.proposed_new_leaf_hash.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(outcome.plan_hash.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(outcome.old_leaf_hash.as_str(), FIELD_RECEIPT)?;
    super::core::lowercase_blake3(outcome.new_leaf_hash.as_str(), FIELD_RECEIPT)?;
    if outcome.plan_hash != plan.plan_hash || outcome.old_leaf_hash != plan.old_leaf_hash {
        return Err(super::core::error(InterruptedToolRepairIssue::InvalidInput, FIELD_RECEIPT));
    }
    if outcome.status.permits_provider_continuation() && outcome.new_leaf_hash != plan.proposed_new_leaf_hash {
        return Err(super::core::error(InterruptedToolRepairIssue::StaleLeaf, FIELD_RECEIPT));
    }
    debug_assert_eq!(outcome.plan_hash, plan.plan_hash);
    debug_assert_eq!(outcome.old_leaf_hash, plan.old_leaf_hash);
    Ok(())
}

fn reject_sensitive(sensitive: InterruptedToolRepairSensitiveDraft<'_>) -> Result<(), InterruptedToolRepairError> {
    let has_private_material = [
        sensitive.raw_prompt,
        sensitive.raw_tool_arguments,
        sensitive.raw_tool_output,
        sensitive.provider_payload,
        sensitive.credential,
        sensitive.private_path,
        sensitive.exception_body,
        sensitive.transcript,
    ]
    .into_iter()
    .any(|value| value.is_some());
    if has_private_material {
        return Err(super::core::error(InterruptedToolRepairIssue::ReceiptLeak, FIELD_RECEIPT));
    }
    debug_assert!(!has_private_material);
    debug_assert!(sensitive.raw_prompt.is_none());
    Ok(())
}
