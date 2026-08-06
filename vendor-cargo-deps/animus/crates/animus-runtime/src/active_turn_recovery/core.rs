//! Pure active-turn policy, transaction, journal, and recovery planning.

// This archived Cairn adaptation keeps explicit DTO vocabulary and bounded validation branches.
#![allow(
    unknown_lints,
    ambiguous_params,
    assertion_density,
    bool_naming,
    compound_condition,
    function_length,
    nested_conditionals,
    non_trait_imports,
    numeric_units,
    path_segment_repetition
)]

use super::data::ACTIVE_TURN_CHECKPOINT_HASH_DOMAIN;
use super::data::ACTIVE_TURN_HASH_DOMAIN;
use super::data::ACTIVE_TURN_PLAN_HASH_DOMAIN;
use super::data::ACTIVE_TURN_POLICY_HASH_DOMAIN;
use super::data::ACTIVE_TURN_RECOVERY_SCHEMA;
use super::data::ACTIVE_TURN_STEERING_HASH_DOMAIN;
use super::data::ACTIVE_TURN_TRANSACTION_HASH_DOMAIN;
use super::data::ActiveTurnAcceptanceDraft;
use super::data::ActiveTurnAcceptanceOutcome;
use super::data::ActiveTurnAcceptancePlan;
use super::data::ActiveTurnAcceptanceState;
use super::data::ActiveTurnCancellationState;
use super::data::ActiveTurnCleanupState;
use super::data::ActiveTurnContinuationAuthority;
use super::data::ActiveTurnJournalEntryKind;
use super::data::ActiveTurnPreparedEntry;
use super::data::ActiveTurnRecoveryAction;
use super::data::ActiveTurnRecoveryError;
use super::data::ActiveTurnRecoveryIssue;
use super::data::ActiveTurnRecoveryPlan;
use super::data::ActiveTurnRecoveryPolicy;
use super::data::ActiveTurnRecoveryPolicyInput;
use super::data::ActiveTurnRecoverySnapshot;
use super::data::ActiveTurnRepairState;
use super::data::ActiveTurnReplacementDraft;
use super::data::ActiveTurnReplacementPlan;
use super::data::ActiveTurnSteeringDraft;
use super::data::ActiveTurnSteeringPlan;
use super::data::ActiveTurnTerminalClass;
use super::data::ActiveTurnTransactionObservation;
use super::data::ActiveTurnTransactionStatus;
use super::data::BLAKE3_HEX_CHARS;
use super::data::HASH_SEPARATOR;
use super::data::MAX_POLICY_COLLECTION_ITEMS;
use super::data::MAX_SAFE_LABEL_CHARS;

const FIELD_POLICY: &str = "active_turn_policy";
const FIELD_ACCEPTANCE: &str = "active_turn_acceptance";
const FIELD_TRANSACTION: &str = "active_turn_transaction";
const FIELD_CHECKPOINT: &str = "active_turn_checkpoint";
const FIELD_STEERING: &str = "active_turn_steering";
const FIELD_RECOVERY: &str = "active_turn_recovery";
const CORRUPTION_POSTURE: &str = "fail_closed";
const REPLACEMENT_POSTURE: &str = "compare_and_swap_atomic";
const REQUIRED_RECEIPT_FIELDS: [&str; 7] = [
    "policy_hash",
    "turn_hash",
    "checkpoint_hash",
    "plan_hash",
    "generation",
    "action",
    "terminal",
];
const REQUIRED_NON_CLAIMS: [&str; 6] = [
    "store_correctness",
    "power_loss_durability",
    "provider_acceptance",
    "exactly_once_effects",
    "task_completion",
    "production_readiness",
];

/// Validates generated policy and derives a domain-separated policy identity.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for unsupported schemas, unbounded values,
/// unsafe postures, duplicate fields, missing non-claims, or malformed hashes.
// r[impl agent_harness_runtime.active_turn_recovery.policy]
pub fn validate_active_turn_recovery_policy(
    input: &ActiveTurnRecoveryPolicyInput,
) -> Result<ActiveTurnRecoveryPolicy, ActiveTurnRecoveryError> {
    if input.schema != ACTIVE_TURN_RECOVERY_SCHEMA
        || input.max_entries == 0
        || input.max_entry_bytes == 0
        || input.max_aggregate_bytes == 0
        || input.max_steering_entries == 0
        || input.corruption_posture != CORRUPTION_POSTURE
        || input.replacement_posture != REPLACEMENT_POSTURE
        || !input.transaction_evidence_required
    {
        return Err(error(ActiveTurnRecoveryIssue::InvalidPolicy, FIELD_POLICY));
    }
    safe_label(input.policy_id.as_str(), FIELD_POLICY)?;
    safe_label(input.retention_class.as_str(), FIELD_POLICY)?;
    lowercase_blake3(input.policy_hash.as_str(), FIELD_POLICY)?;
    validate_required_labels(input.receipt_fields.as_slice(), REQUIRED_RECEIPT_FIELDS.as_slice())?;
    validate_required_labels(input.non_claims.as_slice(), REQUIRED_NON_CLAIMS.as_slice())?;
    if u64::from(input.max_entry_bytes) > input.max_aggregate_bytes {
        return Err(error(ActiveTurnRecoveryIssue::InvalidPolicy, FIELD_POLICY));
    }
    let mut parts = vec![
        input.schema.clone(),
        input.policy_id.clone(),
        input.max_entries.to_string(),
        input.max_entry_bytes.to_string(),
        input.max_aggregate_bytes.to_string(),
        input.max_steering_entries.to_string(),
        input.corruption_posture.clone(),
        input.replacement_posture.clone(),
        input.transaction_evidence_required.to_string(),
        input.retention_class.clone(),
        input.policy_hash.clone(),
    ];
    parts.extend(input.receipt_fields.iter().cloned());
    parts.extend(input.non_claims.iter().cloned());
    let identity = hash_owned_parts(ACTIVE_TURN_POLICY_HASH_DOMAIN, parts.as_slice());
    debug_assert_eq!(identity.len(), BLAKE3_HEX_CHARS);
    Ok(ActiveTurnRecoveryPolicy {
        input: input.clone(),
        identity,
    })
}

/// Plans one atomic accepted-message and active-marker transaction.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for malformed identities or generation overflow.
// r[impl agent_harness_runtime.active_turn_recovery.boundary]
// r[impl agent_harness_runtime.active_turn_recovery.journal]
// r[impl agent_harness_runtime.active_turn_recovery.identity]
pub fn plan_active_turn_acceptance(
    policy: &ActiveTurnRecoveryPolicy,
    draft: &ActiveTurnAcceptanceDraft,
) -> Result<ActiveTurnAcceptancePlan, ActiveTurnRecoveryError> {
    safe_label(draft.session_id.as_str(), FIELD_ACCEPTANCE)?;
    safe_label(draft.branch_id.as_str(), FIELD_ACCEPTANCE)?;
    lowercase_blake3(draft.accepted_message_hash.as_str(), FIELD_ACCEPTANCE)?;
    let next_generation = draft
        .expected_generation
        .checked_add(1)
        .ok_or_else(|| error(ActiveTurnRecoveryIssue::InvalidInput, FIELD_ACCEPTANCE))?;
    let turn_hash = hash_owned_parts(ACTIVE_TURN_HASH_DOMAIN, &[
        draft.session_id.clone(),
        draft.branch_id.clone(),
        draft.turn_sequence.to_string(),
        draft.accepted_message_hash.clone(),
        policy.identity.clone(),
    ]);
    let transaction_hash = hash_owned_parts(ACTIVE_TURN_TRANSACTION_HASH_DOMAIN, &[
        "acceptance".to_string(),
        turn_hash.clone(),
        draft.expected_generation.to_string(),
        next_generation.to_string(),
        policy.identity.clone(),
    ]);
    debug_assert_ne!(draft.expected_generation, next_generation);
    Ok(ActiveTurnAcceptancePlan {
        session_id: draft.session_id.clone(),
        branch_id: draft.branch_id.clone(),
        turn_sequence: draft.turn_sequence,
        accepted_message_hash: draft.accepted_message_hash.clone(),
        expected_generation: draft.expected_generation,
        next_generation,
        policy_hash: policy.identity.clone(),
        turn_hash,
        transaction_hash,
    })
}

/// Evaluates host durable-acceptance evidence before provider eligibility.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for malformed observation identities.
// r[impl agent_harness_runtime.active_turn_recovery.journal]
pub fn evaluate_active_turn_acceptance(
    plan: &ActiveTurnAcceptancePlan,
    observation: &ActiveTurnTransactionObservation,
) -> Result<ActiveTurnAcceptanceOutcome, ActiveTurnRecoveryError> {
    validate_transaction_observation(observation)?;
    let exact = observation.transaction_hash == plan.transaction_hash
        && observation.observed_prior_generation == plan.expected_generation
        && observation.observed_generation == plan.next_generation
        && observation.status == ActiveTurnTransactionStatus::Committed
        && observation.complete_manifest;
    let status = if observation.transaction_hash != plan.transaction_hash
        || observation.observed_prior_generation != plan.expected_generation
    {
        ActiveTurnTransactionStatus::Stale
    } else if !observation.complete_manifest
        || (observation.status == ActiveTurnTransactionStatus::Committed
            && observation.observed_generation != plan.next_generation)
    {
        ActiveTurnTransactionStatus::Partial
    } else {
        observation.status
    };
    let outcome = ActiveTurnAcceptanceOutcome {
        turn_hash: plan.turn_hash.clone(),
        generation: observation.observed_generation,
        status,
        provider_eligible: exact,
    };
    debug_assert_eq!(outcome.provider_eligible, exact);
    Ok(outcome)
}

/// Returns provider eligibility only for exact durable acceptance evidence.
#[must_use]
pub fn active_turn_provider_dispatch_allowed(
    plan: &ActiveTurnAcceptancePlan,
    outcome: Option<&ActiveTurnAcceptanceOutcome>,
) -> bool {
    outcome.is_some_and(|value| {
        value.provider_eligible && value.turn_hash == plan.turn_hash && value.generation == plan.next_generation
    })
}

/// Validates a fully prepared checkpoint and returns one atomic replacement plan.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for partial, corrupt, duplicate, stale,
/// overbound, or unsupported prepared entries.
// r[impl agent_harness_runtime.active_turn_recovery.journal]
pub fn prepare_active_turn_checkpoint_replacement(
    policy: &ActiveTurnRecoveryPolicy,
    draft: &ActiveTurnReplacementDraft,
) -> Result<ActiveTurnReplacementPlan, ActiveTurnRecoveryError> {
    lowercase_blake3(draft.turn_hash.as_str(), FIELD_CHECKPOINT)?;
    validate_entries(policy, draft.entries.as_slice())?;
    let next_generation = draft
        .expected_generation
        .checked_add(1)
        .ok_or_else(|| error(ActiveTurnRecoveryIssue::InvalidInput, FIELD_CHECKPOINT))?;
    let aggregate_bytes = draft.entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(u64::from(entry.prepared_bytes))
            .ok_or_else(|| error(ActiveTurnRecoveryIssue::IncompleteReplacement, FIELD_CHECKPOINT))
    })?;
    if aggregate_bytes > policy.max_aggregate_bytes() {
        return Err(error(ActiveTurnRecoveryIssue::IncompleteReplacement, FIELD_CHECKPOINT));
    }
    let entry_hashes = draft.entries.iter().map(|entry| entry.entry_hash.clone()).collect::<Vec<_>>();
    let mut checkpoint_parts = vec![
        draft.turn_hash.clone(),
        next_generation.to_string(),
        aggregate_bytes.to_string(),
        policy.identity.clone(),
    ];
    checkpoint_parts.extend(entry_hashes.iter().cloned());
    let checkpoint_hash = hash_owned_parts(ACTIVE_TURN_CHECKPOINT_HASH_DOMAIN, checkpoint_parts.as_slice());
    let transaction_hash = hash_owned_parts(ACTIVE_TURN_TRANSACTION_HASH_DOMAIN, &[
        "replace_checkpoint".to_string(),
        draft.turn_hash.clone(),
        draft.expected_generation.to_string(),
        next_generation.to_string(),
        checkpoint_hash.clone(),
    ]);
    Ok(ActiveTurnReplacementPlan {
        turn_hash: draft.turn_hash.clone(),
        expected_generation: draft.expected_generation,
        next_generation,
        entry_hashes,
        aggregate_bytes,
        checkpoint_hash,
        transaction_hash,
    })
}

/// Evaluates checkpoint replacement evidence without mutating storage.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for malformed transaction evidence.
pub fn evaluate_active_turn_checkpoint_replacement(
    plan: &ActiveTurnReplacementPlan,
    observation: &ActiveTurnTransactionObservation,
) -> Result<ActiveTurnTransactionStatus, ActiveTurnRecoveryError> {
    validate_transaction_observation(observation)?;
    if observation.transaction_hash != plan.transaction_hash
        || observation.observed_prior_generation != plan.expected_generation
    {
        return Ok(ActiveTurnTransactionStatus::Stale);
    }
    if observation.status == ActiveTurnTransactionStatus::Committed
        && (!observation.complete_manifest || observation.observed_generation != plan.next_generation)
    {
        return Ok(ActiveTurnTransactionStatus::Partial);
    }
    Ok(observation.status)
}

/// Plans a durable steering append bound to the active turn and sequence.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for malformed identities, an overbound
/// steering queue, or generation overflow.
// r[impl agent_harness_runtime.active_turn_recovery.journal]
pub fn plan_active_turn_steering_append(
    policy: &ActiveTurnRecoveryPolicy,
    draft: &ActiveTurnSteeringDraft,
) -> Result<ActiveTurnSteeringPlan, ActiveTurnRecoveryError> {
    lowercase_blake3(draft.turn_hash.as_str(), FIELD_STEERING)?;
    lowercase_blake3(draft.steering_body_hash.as_str(), FIELD_STEERING)?;
    if draft.pending_count >= policy.max_steering_entries() || draft.steering_sequence == 0 {
        return Err(error(ActiveTurnRecoveryIssue::InvalidInput, FIELD_STEERING));
    }
    let steering_hash = hash_owned_parts(ACTIVE_TURN_STEERING_HASH_DOMAIN, &[
        draft.turn_hash.clone(),
        draft.steering_sequence.to_string(),
        draft.steering_body_hash.clone(),
        policy.identity.clone(),
    ]);
    let append_hash = hash_owned_parts(ACTIVE_TURN_TRANSACTION_HASH_DOMAIN, &[
        "append_steering".to_string(),
        draft.turn_hash.clone(),
        draft.expected_generation.to_string(),
        steering_hash.clone(),
    ]);
    Ok(ActiveTurnSteeringPlan {
        turn_hash: draft.turn_hash.clone(),
        steering_sequence: draft.steering_sequence,
        steering_hash,
        expected_generation: draft.expected_generation,
        append_hash,
    })
}

/// Returns steering delivery eligibility only for exact durable append evidence.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for malformed evidence.
pub fn active_turn_steering_delivery_allowed(
    plan: &ActiveTurnSteeringPlan,
    observation: &ActiveTurnTransactionObservation,
) -> Result<bool, ActiveTurnRecoveryError> {
    validate_transaction_observation(observation)?;
    Ok(observation.transaction_hash == plan.append_hash
        && observation.observed_prior_generation == plan.expected_generation
        && observation.status == ActiveTurnTransactionStatus::Committed
        && observation.complete_manifest)
}

/// Produces one deterministic recovery action from a validated supplied snapshot.
///
/// No returned type can construct or dispatch a tool request.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] for corrupt, stale, unsupported, or
/// conflicting journal facts.
// r[impl agent_harness_runtime.active_turn_recovery.recovery]
pub fn plan_active_turn_recovery(
    policy: &ActiveTurnRecoveryPolicy,
    snapshot: &ActiveTurnRecoverySnapshot,
) -> Result<ActiveTurnRecoveryPlan, ActiveTurnRecoveryError> {
    validate_recovery_snapshot(policy, snapshot)?;
    let action = if snapshot.terminal == ActiveTurnTerminalClass::Conflicting {
        return Err(error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_RECOVERY));
    } else if snapshot.terminal != ActiveTurnTerminalClass::Active {
        if snapshot.cleanup == ActiveTurnCleanupState::Complete {
            ActiveTurnRecoveryAction::ObserveTerminal
        } else {
            ActiveTurnRecoveryAction::CleanupRequired
        }
    } else if snapshot.acceptance != ActiveTurnAcceptanceState::Durable
        || snapshot.authority != ActiveTurnContinuationAuthority::Allowed
        || snapshot.cancellation == ActiveTurnCancellationState::Cancelled
        || snapshot.repair_state == ActiveTurnRepairState::Blocked
    {
        ActiveTurnRecoveryAction::Blocked
    } else if snapshot.repair_state == ActiveTurnRepairState::RepairRequired {
        ActiveTurnRecoveryAction::PlanInterruptedToolRepair
    } else {
        ActiveTurnRecoveryAction::ContinueProvider
    };
    let plan_hash = hash_owned_parts(ACTIVE_TURN_PLAN_HASH_DOMAIN, &[
        snapshot.turn_hash.clone(),
        snapshot.checkpoint_hash.clone(),
        snapshot.generation.to_string(),
        action.as_str().to_string(),
        snapshot.terminal.as_str().to_string(),
        policy.identity.clone(),
    ]);
    let plan = ActiveTurnRecoveryPlan {
        turn_hash: snapshot.turn_hash.clone(),
        checkpoint_hash: snapshot.checkpoint_hash.clone(),
        generation: snapshot.generation,
        action,
        terminal: snapshot.terminal,
        plan_hash,
    };
    debug_assert!(
        !matches!(plan.action, ActiveTurnRecoveryAction::ContinueProvider)
            || (snapshot.acceptance == ActiveTurnAcceptanceState::Durable
                && snapshot.authority == ActiveTurnContinuationAuthority::Allowed
                && snapshot.cancellation == ActiveTurnCancellationState::Active)
    );
    Ok(plan)
}

fn validate_recovery_snapshot(
    policy: &ActiveTurnRecoveryPolicy,
    snapshot: &ActiveTurnRecoverySnapshot,
) -> Result<(), ActiveTurnRecoveryError> {
    lowercase_blake3(snapshot.turn_hash.as_str(), FIELD_RECOVERY)?;
    lowercase_blake3(snapshot.checkpoint_hash.as_str(), FIELD_RECOVERY)?;
    validate_entries(policy, snapshot.entries.as_slice())?;
    let expected_checkpoint_hash =
        checkpoint_hash(policy, snapshot.turn_hash.as_str(), snapshot.generation, snapshot.entries.as_slice())?;
    if expected_checkpoint_hash != snapshot.checkpoint_hash {
        return Err(error(ActiveTurnRecoveryIssue::StaleGeneration, FIELD_RECOVERY));
    }
    Ok(())
}

fn checkpoint_hash(
    policy: &ActiveTurnRecoveryPolicy,
    turn_hash: &str,
    generation: u64,
    entries: &[ActiveTurnPreparedEntry],
) -> Result<String, ActiveTurnRecoveryError> {
    let aggregate_bytes = entries.iter().try_fold(0_u64, |total, entry| {
        total
            .checked_add(u64::from(entry.prepared_bytes))
            .ok_or_else(|| error(ActiveTurnRecoveryIssue::IncompleteReplacement, FIELD_CHECKPOINT))
    })?;
    let mut parts = vec![
        turn_hash.to_string(),
        generation.to_string(),
        aggregate_bytes.to_string(),
        policy.identity.clone(),
    ];
    parts.extend(entries.iter().map(|entry| entry.entry_hash.clone()));
    Ok(hash_owned_parts(ACTIVE_TURN_CHECKPOINT_HASH_DOMAIN, parts.as_slice()))
}

/// Derives the checkpoint identity for already validated entries.
///
/// # Errors
///
/// Returns [`ActiveTurnRecoveryError`] when supplied entries are invalid.
pub fn active_turn_checkpoint_hash(
    policy: &ActiveTurnRecoveryPolicy,
    turn_hash: &str,
    generation: u64,
    entries: &[ActiveTurnPreparedEntry],
) -> Result<String, ActiveTurnRecoveryError> {
    lowercase_blake3(turn_hash, FIELD_CHECKPOINT)?;
    validate_entries(policy, entries)?;
    checkpoint_hash(policy, turn_hash, generation, entries)
}

fn validate_entries(
    policy: &ActiveTurnRecoveryPolicy,
    entries: &[ActiveTurnPreparedEntry],
) -> Result<(), ActiveTurnRecoveryError> {
    let entry_bound = usize::try_from(policy.max_entries())
        .map_err(|_error| error(ActiveTurnRecoveryIssue::InvalidPolicy, FIELD_POLICY))?;
    if entries.is_empty() || entries.len() > entry_bound {
        return Err(error(ActiveTurnRecoveryIssue::IncompleteReplacement, FIELD_CHECKPOINT));
    }
    let mut hashes = std::collections::BTreeSet::new();
    let mut open_calls = std::collections::BTreeSet::<String>::new();
    let mut terminal_seen = false;
    for (index, entry) in entries.iter().enumerate() {
        let expected_sequence =
            u32::try_from(index).map_err(|_error| error(ActiveTurnRecoveryIssue::InvalidInput, FIELD_CHECKPOINT))?;
        if entry.sequence != expected_sequence
            || entry.kind == ActiveTurnJournalEntryKind::Unsupported
            || entry.prepared_bytes == 0
            || entry.prepared_bytes > policy.max_entry_bytes()
            || !hashes.insert(entry.entry_hash.as_str())
            || terminal_seen
        {
            return Err(error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT));
        }
        lowercase_blake3(entry.entry_hash.as_str(), FIELD_CHECKPOINT)?;
        match entry.kind {
            ActiveTurnJournalEntryKind::ToolCall => {
                let call = entry
                    .tool_call_id
                    .as_deref()
                    .ok_or_else(|| error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT))?;
                safe_label(call, FIELD_CHECKPOINT)?;
                if !open_calls.insert(call.to_string()) {
                    return Err(error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT));
                }
            }
            ActiveTurnJournalEntryKind::ToolResult | ActiveTurnJournalEntryKind::SyntheticToolResult => {
                let call = entry
                    .tool_call_id
                    .as_deref()
                    .ok_or_else(|| error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT))?;
                if !open_calls.remove(call) {
                    return Err(error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT));
                }
            }
            ActiveTurnJournalEntryKind::Terminal => {
                terminal_seen = true;
                if entry.tool_call_id.is_some() {
                    return Err(error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT));
                }
            }
            _other => {
                if entry.tool_call_id.is_some() {
                    return Err(error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT));
                }
            }
        }
    }
    if entries.first().is_none_or(|entry| entry.kind != ActiveTurnJournalEntryKind::AcceptedMessage) {
        return Err(error(ActiveTurnRecoveryIssue::CorruptJournal, FIELD_CHECKPOINT));
    }
    Ok(())
}

fn validate_required_labels(values: &[String], required: &[&str]) -> Result<(), ActiveTurnRecoveryError> {
    if values.is_empty() || values.len() > MAX_POLICY_COLLECTION_ITEMS {
        return Err(error(ActiveTurnRecoveryIssue::InvalidPolicy, FIELD_POLICY));
    }
    let mut unique = std::collections::BTreeSet::new();
    for value in values {
        safe_label(value.as_str(), FIELD_POLICY)?;
        if !unique.insert(value.as_str()) {
            return Err(error(ActiveTurnRecoveryIssue::InvalidPolicy, FIELD_POLICY));
        }
    }
    if required.iter().any(|item| !unique.contains(item)) {
        return Err(error(ActiveTurnRecoveryIssue::InvalidPolicy, FIELD_POLICY));
    }
    Ok(())
}

fn validate_transaction_observation(
    observation: &ActiveTurnTransactionObservation,
) -> Result<(), ActiveTurnRecoveryError> {
    lowercase_blake3(observation.transaction_hash.as_str(), FIELD_TRANSACTION)
}

pub(super) fn hash_owned_parts(domain: &str, parts: &[String]) -> String {
    let mut material = String::from(domain);
    for part in parts {
        material.push_str(HASH_SEPARATOR);
        material.push_str(part.as_str());
    }
    blake3::hash(material.as_bytes()).to_hex().to_string()
}

pub(super) fn lowercase_blake3(value: &str, field: &'static str) -> Result<(), ActiveTurnRecoveryError> {
    let valid = value.len() == BLAKE3_HEX_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !valid {
        return Err(error(ActiveTurnRecoveryIssue::InvalidInput, field));
    }
    Ok(())
}

fn safe_label(value: &str, field: &'static str) -> Result<(), ActiveTurnRecoveryError> {
    let valid_length = !value.is_empty() && value.chars().count() <= MAX_SAFE_LABEL_CHARS;
    let valid_chars =
        value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'));
    if !valid_length || !valid_chars {
        return Err(error(ActiveTurnRecoveryIssue::InvalidInput, field));
    }
    Ok(())
}

pub(super) const fn error(issue: ActiveTurnRecoveryIssue, field: &'static str) -> ActiveTurnRecoveryError {
    ActiveTurnRecoveryError::new(issue, field)
}
