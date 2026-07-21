//! Pure active-branch diagnosis, repair planning, and provider gating.

const BLAKE3_HEX_CHARS: usize = super::data::BLAKE3_HEX_CHARS;
const HASH_SEPARATOR: &str = super::data::HASH_SEPARATOR;
const INTERRUPTED_TOOL_ACTION_HASH_DOMAIN: &str = super::data::INTERRUPTED_TOOL_ACTION_HASH_DOMAIN;
const INTERRUPTED_TOOL_LEAF_HASH_DOMAIN: &str = super::data::INTERRUPTED_TOOL_LEAF_HASH_DOMAIN;
const INTERRUPTED_TOOL_PLAN_HASH_DOMAIN: &str = super::data::INTERRUPTED_TOOL_PLAN_HASH_DOMAIN;
const INTERRUPTED_TOOL_SNAPSHOT_HASH_DOMAIN: &str =
    super::data::INTERRUPTED_TOOL_SNAPSHOT_HASH_DOMAIN;
const MAX_BRANCH_ENTRIES: usize = super::data::MAX_BRANCH_ENTRIES;
const MAX_CALLS_PER_ENTRY: usize = super::data::MAX_CALLS_PER_ENTRY;
const MAX_INTERRUPTION_OBSERVATIONS: usize = super::data::MAX_INTERRUPTION_OBSERVATIONS;
const MAX_PREVIOUSLY_APPLIED_PLANS: usize = super::data::MAX_PREVIOUSLY_APPLIED_PLANS;
const MAX_SAFE_LABEL_CHARS: usize = super::data::MAX_SAFE_LABEL_CHARS;
const MAX_TOTAL_TOOL_CALLS: usize = super::data::MAX_TOTAL_TOOL_CALLS;

type InterruptedToolBranchEntry = super::data::InterruptedToolBranchEntry;
type InterruptedToolBranchSnapshot = super::data::InterruptedToolBranchSnapshot;
type InterruptedToolDiagnosis = super::data::InterruptedToolDiagnosis;
type InterruptedToolDiagnosisStatus = super::data::InterruptedToolDiagnosisStatus;
type InterruptedToolDurability = super::data::InterruptedToolDurability;
type InterruptedToolEntryKind = super::data::InterruptedToolEntryKind;
type InterruptedToolExecutorObservation = super::data::InterruptedToolExecutorObservation;
type InterruptedToolGap = super::data::InterruptedToolGap;
type InterruptedToolObservation = super::data::InterruptedToolObservation;
type InterruptedToolObservationKind = super::data::InterruptedToolObservationKind;
type InterruptedToolProviderGateDecision = super::data::InterruptedToolProviderGateDecision;
type InterruptedToolProviderGateStatus = super::data::InterruptedToolProviderGateStatus;
type InterruptedToolReason = super::data::InterruptedToolReason;
type InterruptedToolRepairAction = super::data::InterruptedToolRepairAction;
type InterruptedToolRepairApplicationDraft<'a> =
    super::data::InterruptedToolRepairApplicationDraft<'a>;
type InterruptedToolRepairApplicationOutcome = super::data::InterruptedToolRepairApplicationOutcome;
type InterruptedToolRepairApplicationRequest = super::data::InterruptedToolRepairApplicationRequest;
type InterruptedToolRepairApplicationStatus = super::data::InterruptedToolRepairApplicationStatus;
type InterruptedToolRepairEffect = super::data::InterruptedToolRepairEffect;
type InterruptedToolRepairError = super::data::InterruptedToolRepairError;
type InterruptedToolRepairIssue = super::data::InterruptedToolRepairIssue;
type InterruptedToolRepairMode = super::data::InterruptedToolRepairMode;
type InterruptedToolRepairPlan = super::data::InterruptedToolRepairPlan;
type InterruptedToolReplayState = super::data::InterruptedToolReplayState;

#[derive(Clone, Copy)]
pub(super) struct RepairField(pub(super) &'static str);

const FIELD_BRANCH: RepairField = RepairField("interrupted_tool_branch");
const FIELD_ENTRY: RepairField = RepairField("interrupted_tool_entry");
const FIELD_CALL: RepairField = RepairField("interrupted_tool_call");
const FIELD_RESULT: RepairField = RepairField("interrupted_tool_result");
const FIELD_LEAF: RepairField = RepairField("interrupted_tool_leaf");
const FIELD_OBSERVATION: RepairField = RepairField("interrupted_tool_observation");
const FIELD_PLAN: RepairField = RepairField("interrupted_tool_plan");
const FIELD_APPLICATION: RepairField = RepairField("interrupted_tool_application");
const SYNTHETIC_RESULT_PREFIX: &str = "interrupted_tool_result:";

#[derive(Clone)]
struct CallState {
    gap: InterruptedToolGap,
    has_result: bool,
}

struct CallAccumulator<'a> {
    indices: &'a mut std::collections::BTreeMap<String, usize>,
    states: &'a mut Vec<CallState>,
}

struct PlanParts {
    mode: InterruptedToolRepairMode,
    anchor_entry_id: String,
    affected_suffix_entry_hashes: Vec<String>,
    actions: Vec<InterruptedToolRepairAction>,
}

struct PlanIdentities {
    proposed_new_leaf_hash: String,
    plan_hash: String,
}

/// Diagnoses exact tool-call/result bindings on an already-loaded active branch.
///
/// # Errors
///
/// Returns [`InterruptedToolRepairError`] for malformed bounds or identities,
/// stale leaves, unsupported entry kinds, duplicate calls/results,
/// result-before-call ordering, or mismatched tool identities.
// r[impl agent_harness_runtime.tau_tool_repair.invariant]
pub fn diagnose_interrupted_tool_branch(
    snapshot: &InterruptedToolBranchSnapshot,
) -> Result<InterruptedToolDiagnosis, InterruptedToolRepairError> {
    validate_snapshot(snapshot)?;
    let mut call_indices = std::collections::BTreeMap::<String, usize>::new();
    let mut call_states = Vec::<CallState>::with_capacity(MAX_TOTAL_TOOL_CALLS);
    for (entry_index, entry) in snapshot.entries.iter().enumerate() {
        {
            let mut accumulator = CallAccumulator {
                indices: &mut call_indices,
                states: &mut call_states,
            };
            collect_entry_calls(entry, entry_index, &mut accumulator)?;
        }
        collect_entry_result(entry, &call_indices, &mut call_states)?;
    }
    let mut matched_call_count = u32::default();
    let mut unmatched = Vec::with_capacity(call_states.len());
    for state in call_states {
        if state.has_result {
            matched_call_count = matched_call_count
                .checked_add(1)
                .ok_or_else(|| error(InterruptedToolRepairIssue::InvalidInput, FIELD_CALL))?;
        } else {
            unmatched.push(state.gap);
        }
    }
    let status = if unmatched.is_empty() {
        InterruptedToolDiagnosisStatus::Clean
    } else {
        InterruptedToolDiagnosisStatus::Repairable
    };
    let diagnosis = InterruptedToolDiagnosis {
        status,
        snapshot_hash: snapshot_hash(snapshot),
        active_leaf_hash: snapshot.active_leaf_hash.clone(),
        matched_call_count,
        unmatched,
    };
    debug_assert_eq!(
        diagnosis.status == InterruptedToolDiagnosisStatus::Clean,
        diagnosis.unmatched.is_empty()
    );
    Ok(diagnosis)
}

/// Reduces correlated host evidence to one synthetic-result reason.
///
/// # Errors
///
/// Returns [`InterruptedToolRepairError`] when observations are malformed or
/// make conflicting causal claims for the same run and call.
// r[impl agent_harness_runtime.tau_tool_repair.reason]
pub fn reduce_interrupted_tool_reason(
    gap: &InterruptedToolGap,
    observations: &[InterruptedToolObservation],
) -> Result<InterruptedToolReason, InterruptedToolRepairError> {
    validate_observations(observations)?;
    let mut matched_kind = None;
    for observation in observations {
        let is_correlated_to_gap = observation.run_id == gap.run_id
            && observation
                .affected_call_ids
                .iter()
                .any(|call_id| call_id == &gap.call.call_id);
        if !is_correlated_to_gap {
            continue;
        }
        if let Some(existing) = matched_kind {
            if existing != observation.kind {
                return Err(error(
                    InterruptedToolRepairIssue::ConflictingEvidence,
                    FIELD_OBSERVATION,
                ));
            }
        } else {
            matched_kind = Some(observation.kind);
        }
    }
    let reason = matched_kind.map_or(
        InterruptedToolReason::UnknownInterruption,
        observation_reason,
    );
    debug_assert!(observations.len() <= MAX_INTERRUPTION_OBSERVATIONS);
    debug_assert!(!gap.call.call_id.is_empty());
    Ok(reason)
}

/// Builds an ordered append-only repair plan for a repairable diagnosis.
///
/// # Errors
///
/// Returns [`InterruptedToolRepairError`] when the diagnosis is stale, the
/// repair anchor is absent, evidence conflicts, or no repairable gap remains.
// r[impl agent_harness_runtime.tau_tool_repair.plan]
// r[impl agent_harness_runtime.tau_tool_repair.append_only]
pub fn plan_interrupted_tool_repair(
    snapshot: &InterruptedToolBranchSnapshot,
    diagnosis: &InterruptedToolDiagnosis,
    observations: &[InterruptedToolObservation],
) -> Result<InterruptedToolRepairPlan, InterruptedToolRepairError> {
    let current = diagnose_interrupted_tool_branch(snapshot)?;
    if &current != diagnosis || diagnosis.status != InterruptedToolDiagnosisStatus::Repairable {
        return Err(error(
            InterruptedToolRepairIssue::MissingRepairAnchor,
            FIELD_PLAN,
        ));
    }
    let parts = derive_plan_parts(snapshot, diagnosis, observations)?;
    let identities = derive_plan_identities(snapshot, diagnosis, &parts);
    debug_assert!(!parts.actions.is_empty());
    debug_assert_ne!(identities.proposed_new_leaf_hash, snapshot.active_leaf_hash);
    Ok(InterruptedToolRepairPlan {
        mode: parts.mode,
        session_id: snapshot.session_id.clone(),
        branch_id: snapshot.branch_id.clone(),
        old_leaf_hash: snapshot.active_leaf_hash.clone(),
        proposed_new_leaf_hash: identities.proposed_new_leaf_hash,
        repair_anchor_entry_id: parts.anchor_entry_id,
        affected_suffix_entry_hashes: parts.affected_suffix_entry_hashes,
        actions: parts.actions,
        snapshot_hash: diagnosis.snapshot_hash.clone(),
        plan_hash: identities.plan_hash,
    })
}

fn derive_plan_parts(
    snapshot: &InterruptedToolBranchSnapshot,
    diagnosis: &InterruptedToolDiagnosis,
    observations: &[InterruptedToolObservation],
) -> Result<PlanParts, InterruptedToolRepairError> {
    let first_gap = diagnosis
        .unmatched
        .first()
        .ok_or_else(|| error(InterruptedToolRepairIssue::MissingRepairAnchor, FIELD_PLAN))?;
    let anchor = snapshot
        .entries
        .get(first_gap.entry_index)
        .ok_or_else(|| error(InterruptedToolRepairIssue::MissingRepairAnchor, FIELD_PLAN))?;
    if anchor.entry_id != first_gap.entry_id || anchor.entry_hash != first_gap.entry_hash {
        return Err(error(
            InterruptedToolRepairIssue::MissingRepairAnchor,
            FIELD_PLAN,
        ));
    }
    let affected_suffix_entry_hashes = snapshot
        .entries
        .iter()
        .skip(first_gap.entry_index.saturating_add(1))
        .map(|entry| entry.entry_hash.clone())
        .collect::<Vec<_>>();
    let actions = build_repair_actions(snapshot, diagnosis, observations)?;
    debug_assert!(!actions.is_empty());
    debug_assert!(actions.len() <= MAX_TOTAL_TOOL_CALLS);
    Ok(PlanParts {
        mode: repair_mode(snapshot, first_gap.entry_index),
        anchor_entry_id: anchor.entry_id.clone(),
        affected_suffix_entry_hashes,
        actions,
    })
}

fn derive_plan_identities(
    snapshot: &InterruptedToolBranchSnapshot,
    diagnosis: &InterruptedToolDiagnosis,
    parts: &PlanParts,
) -> PlanIdentities {
    let action_hashes = parts
        .actions
        .iter()
        .map(|action| action.action_hash.clone())
        .collect::<Vec<_>>();
    let proposed_new_leaf_hash = hash_owned_parts(
        INTERRUPTED_TOOL_LEAF_HASH_DOMAIN,
        [
            vec![
                snapshot.session_id.clone(),
                snapshot.branch_id.clone(),
                snapshot.active_leaf_hash.clone(),
                parts.mode.as_str().to_string(),
            ],
            action_hashes.clone(),
            parts.affected_suffix_entry_hashes.clone(),
        ]
        .concat()
        .as_slice(),
    );
    let plan_hash = hash_owned_parts(
        INTERRUPTED_TOOL_PLAN_HASH_DOMAIN,
        [
            vec![
                snapshot.session_id.clone(),
                snapshot.branch_id.clone(),
                snapshot.active_leaf_hash.clone(),
                proposed_new_leaf_hash.clone(),
                parts.anchor_entry_id.clone(),
                diagnosis.snapshot_hash.clone(),
                parts.mode.as_str().to_string(),
            ],
            action_hashes,
            parts.affected_suffix_entry_hashes.clone(),
        ]
        .concat()
        .as_slice(),
    );
    debug_assert_eq!(proposed_new_leaf_hash.len(), BLAKE3_HEX_CHARS);
    debug_assert_eq!(plan_hash.len(), BLAKE3_HEX_CHARS);
    PlanIdentities {
        proposed_new_leaf_hash,
        plan_hash,
    }
}

/// Projects an admitted plan into the only supported host mutation request.
#[must_use]
pub fn interrupted_tool_repair_application_request(
    plan: &InterruptedToolRepairPlan,
) -> InterruptedToolRepairApplicationRequest {
    let request = InterruptedToolRepairApplicationRequest {
        effect: InterruptedToolRepairEffect::AppendSyntheticFailures,
        mode: plan.mode,
        session_id: plan.session_id.clone(),
        branch_id: plan.branch_id.clone(),
        plan_hash: plan.plan_hash.clone(),
        expected_old_leaf_hash: plan.old_leaf_hash.clone(),
        proposed_new_leaf_hash: plan.proposed_new_leaf_hash.clone(),
        repair_anchor_entry_id: plan.repair_anchor_entry_id.clone(),
        actions: plan.actions.clone(),
        action_hashes: plan
            .actions
            .iter()
            .map(|action| action.action_hash.clone())
            .collect(),
        affected_suffix_entry_hashes: plan.affected_suffix_entry_hashes.clone(),
    };
    debug_assert_eq!(request.actions.len(), plan.actions.len());
    debug_assert_eq!(request.action_hashes.len(), plan.actions.len());
    request
}

/// Evaluates host-supplied durable application evidence without performing I/O.
///
/// # Errors
///
/// Returns [`InterruptedToolRepairError`] when supplied identities are malformed.
// r[impl agent_harness_runtime.tau_tool_repair.provider_gate]
pub fn evaluate_interrupted_tool_repair_application(
    plan: &InterruptedToolRepairPlan,
    draft: &InterruptedToolRepairApplicationDraft<'_>,
) -> Result<InterruptedToolRepairApplicationOutcome, InterruptedToolRepairError> {
    validate_application_draft(draft)?;
    let expected_action_hashes = plan
        .actions
        .iter()
        .map(|action| action.action_hash.as_str())
        .collect::<Vec<_>>();
    let status = application_status(plan, draft, expected_action_hashes.as_slice());
    let committed_action_count = u32::try_from(draft.committed_action_hashes.len())
        .map_err(|_error| error(InterruptedToolRepairIssue::InvalidInput, FIELD_APPLICATION))?;
    let outcome = InterruptedToolRepairApplicationOutcome {
        status,
        plan_hash: plan.plan_hash.clone(),
        old_leaf_hash: plan.old_leaf_hash.clone(),
        new_leaf_hash: draft.observed_new_leaf_hash.to_string(),
        committed_action_count,
    };
    debug_assert_eq!(outcome.plan_hash, plan.plan_hash);
    Ok(outcome)
}

/// Gates provider continuation on either a clean branch or complete durable repair.
#[must_use]
pub fn evaluate_interrupted_tool_provider_gate(
    diagnosis: &InterruptedToolDiagnosis,
    plan: Option<&InterruptedToolRepairPlan>,
    outcome: Option<&InterruptedToolRepairApplicationOutcome>,
) -> InterruptedToolProviderGateDecision {
    if diagnosis.status == InterruptedToolDiagnosisStatus::Clean {
        let status = if plan.is_none() && outcome.is_none() {
            InterruptedToolProviderGateStatus::ReadyClean
        } else {
            InterruptedToolProviderGateStatus::BlockedApplication
        };
        return InterruptedToolProviderGateDecision {
            status,
            repair_plan_hash: plan.map(|value| value.plan_hash.clone()),
        };
    }
    let Some(plan) = plan else {
        return InterruptedToolProviderGateDecision {
            status: InterruptedToolProviderGateStatus::BlockedRepairRequired,
            repair_plan_hash: None,
        };
    };
    let has_passed = plan.snapshot_hash == diagnosis.snapshot_hash
        && outcome.is_some_and(|value| {
            value.plan_hash == plan.plan_hash
                && value.new_leaf_hash == plan.proposed_new_leaf_hash
                && value.status.permits_provider_continuation()
        });
    let decision = InterruptedToolProviderGateDecision {
        status: if has_passed {
            InterruptedToolProviderGateStatus::ReadyRepaired
        } else {
            InterruptedToolProviderGateStatus::BlockedApplication
        },
        repair_plan_hash: Some(plan.plan_hash.clone()),
    };
    debug_assert_eq!(decision.provider_dispatch_allowed(), has_passed);
    debug_assert_eq!(decision.repair_plan_hash.as_deref(), Some(plan.plan_hash()));
    decision
}

/// Projects synthetic failure messages into the canonical provider-neutral DTO.
///
/// # Errors
///
/// Returns [`InterruptedToolRepairError`] unless durable application passed or
/// when canonical message validation unexpectedly rejects validated metadata.
pub fn project_interrupted_tool_repair_results(
    plan: &InterruptedToolRepairPlan,
    outcome: &InterruptedToolRepairApplicationOutcome,
) -> Result<Vec<animus_adapters::CanonicalModelMessage>, InterruptedToolRepairError> {
    let is_ready = outcome.plan_hash == plan.plan_hash
        && outcome.new_leaf_hash == plan.proposed_new_leaf_hash
        && outcome.status.permits_provider_continuation();
    if !is_ready {
        return Err(error(
            InterruptedToolRepairIssue::ProviderGateBlocked,
            FIELD_APPLICATION,
        ));
    }
    let mut messages = Vec::with_capacity(plan.actions.len());
    for action in &plan.actions {
        let marker = format!("{SYNTHETIC_RESULT_PREFIX}{}", action.reason.as_str());
        let content = [animus_adapters::CanonicalMessageContentPartDraft::Text {
            text: marker.as_str(),
        }];
        let message = animus_adapters::CanonicalModelMessage::from_draft(
            animus_adapters::CanonicalModelMessageDraft {
                role: animus_adapters::CanonicalModelRole::Tool.as_str(),
                content: &content,
                tool_call_id: Some(action.call_id.as_str()),
            },
        )
        .map_err(|_error| {
            error(
                InterruptedToolRepairIssue::ProviderGateBlocked,
                FIELD_RESULT,
            )
        })?;
        messages.push(message);
    }
    debug_assert_eq!(messages.len(), plan.actions.len());
    Ok(messages)
}

// r[impl agent_harness_runtime.tau_tool_repair.verification]
fn validate_snapshot(
    snapshot: &InterruptedToolBranchSnapshot,
) -> Result<(), InterruptedToolRepairError> {
    safe_label(snapshot.session_id.as_str(), FIELD_BRANCH)?;
    safe_label(snapshot.branch_id.as_str(), FIELD_BRANCH)?;
    lowercase_blake3(snapshot.active_leaf_hash.as_str(), FIELD_LEAF)?;
    lowercase_blake3(snapshot.snapshot_leaf_hash.as_str(), FIELD_LEAF)?;
    if snapshot.entries.is_empty() || snapshot.entries.len() > MAX_BRANCH_ENTRIES {
        return Err(error(InterruptedToolRepairIssue::InvalidInput, FIELD_ENTRY));
    }
    if snapshot.active_leaf_hash != snapshot.snapshot_leaf_hash
        || snapshot
            .entries
            .last()
            .is_none_or(|entry| entry.entry_hash != snapshot.active_leaf_hash)
    {
        return Err(error(InterruptedToolRepairIssue::StaleLeaf, FIELD_LEAF));
    }
    let mut entry_ids = std::collections::BTreeSet::new();
    let mut entry_hashes = std::collections::BTreeSet::new();
    let mut total_calls = usize::default();
    for entry in &snapshot.entries {
        validate_entry(entry)?;
        if !entry_ids.insert(entry.entry_id.as_str())
            || !entry_hashes.insert(entry.entry_hash.as_str())
        {
            return Err(error(InterruptedToolRepairIssue::InvalidInput, FIELD_ENTRY));
        }
        total_calls = total_calls
            .checked_add(entry.tool_calls.len())
            .ok_or_else(|| error(InterruptedToolRepairIssue::InvalidInput, FIELD_CALL))?;
        if total_calls > MAX_TOTAL_TOOL_CALLS {
            return Err(error(InterruptedToolRepairIssue::InvalidInput, FIELD_CALL));
        }
    }
    debug_assert!(!entry_ids.is_empty());
    debug_assert!(total_calls <= MAX_TOTAL_TOOL_CALLS);
    Ok(())
}

fn validate_entry(entry: &InterruptedToolBranchEntry) -> Result<(), InterruptedToolRepairError> {
    safe_label(entry.entry_id.as_str(), FIELD_ENTRY)?;
    lowercase_blake3(entry.entry_hash.as_str(), FIELD_ENTRY)?;
    safe_label(entry.run_id.as_str(), FIELD_ENTRY)?;
    if entry.kind == InterruptedToolEntryKind::Unsupported {
        return Err(error(
            InterruptedToolRepairIssue::UnsupportedEntry,
            FIELD_ENTRY,
        ));
    }
    if entry.tool_calls.len() > MAX_CALLS_PER_ENTRY {
        return Err(error(InterruptedToolRepairIssue::InvalidInput, FIELD_CALL));
    }
    let is_shape_valid = match entry.kind {
        InterruptedToolEntryKind::AssistantToolCalls => {
            !entry.tool_calls.is_empty() && entry.tool_result.is_none()
        }
        InterruptedToolEntryKind::ToolResult | InterruptedToolEntryKind::SyntheticToolResult => {
            entry.tool_calls.is_empty() && entry.tool_result.is_some()
        }
        InterruptedToolEntryKind::Unsupported => false,
        _other => entry.tool_calls.is_empty() && entry.tool_result.is_none(),
    };
    if !is_shape_valid {
        return Err(error(
            InterruptedToolRepairIssue::InvalidEntryShape,
            FIELD_ENTRY,
        ));
    }
    for call in &entry.tool_calls {
        safe_label(call.call_id.as_str(), FIELD_CALL)?;
        lowercase_blake3(call.tool_identity_hash.as_str(), FIELD_CALL)?;
    }
    if let Some(result) = &entry.tool_result {
        safe_label(result.call_id.as_str(), FIELD_RESULT)?;
        lowercase_blake3(result.tool_identity_hash.as_str(), FIELD_RESULT)?;
    }
    debug_assert!(entry.tool_calls.len() <= MAX_CALLS_PER_ENTRY);
    debug_assert_ne!(entry.kind, InterruptedToolEntryKind::Unsupported);
    Ok(())
}

fn collect_entry_calls(
    entry: &InterruptedToolBranchEntry,
    entry_index: usize,
    accumulator: &mut CallAccumulator<'_>,
) -> Result<(), InterruptedToolRepairError> {
    for (call_index, call) in entry.tool_calls.iter().enumerate() {
        if accumulator.indices.contains_key(call.call_id.as_str()) {
            return Err(error(InterruptedToolRepairIssue::DuplicateCall, FIELD_CALL));
        }
        let state_index = accumulator.states.len();
        accumulator
            .indices
            .insert(call.call_id.clone(), state_index);
        accumulator.states.push(CallState {
            gap: InterruptedToolGap {
                entry_index,
                call_index,
                entry_id: entry.entry_id.clone(),
                entry_hash: entry.entry_hash.clone(),
                run_id: entry.run_id.clone(),
                call: call.clone(),
            },
            has_result: false,
        });
    }
    debug_assert!(accumulator.states.len() <= MAX_TOTAL_TOOL_CALLS);
    debug_assert_eq!(accumulator.indices.len(), accumulator.states.len());
    Ok(())
}

fn collect_entry_result(
    entry: &InterruptedToolBranchEntry,
    call_indices: &std::collections::BTreeMap<String, usize>,
    call_states: &mut [CallState],
) -> Result<(), InterruptedToolRepairError> {
    let Some(result) = &entry.tool_result else {
        return Ok(());
    };
    let Some(state_index) = call_indices.get(result.call_id.as_str()).copied() else {
        return Err(error(
            InterruptedToolRepairIssue::ResultBeforeCall,
            FIELD_RESULT,
        ));
    };
    let state = call_states
        .get_mut(state_index)
        .ok_or_else(|| error(InterruptedToolRepairIssue::ResultBeforeCall, FIELD_RESULT))?;
    if state.has_result {
        return Err(error(
            InterruptedToolRepairIssue::DuplicateResult,
            FIELD_RESULT,
        ));
    }
    if state.gap.run_id != entry.run_id {
        return Err(error(InterruptedToolRepairIssue::RunMismatch, FIELD_RESULT));
    }
    if state.gap.call.tool_identity_hash != result.tool_identity_hash {
        return Err(error(
            InterruptedToolRepairIssue::ToolMismatch,
            FIELD_RESULT,
        ));
    }
    state.has_result = true;
    debug_assert!(state.has_result);
    debug_assert_eq!(state.gap.call.call_id, result.call_id);
    Ok(())
}

fn validate_observations(
    observations: &[InterruptedToolObservation],
) -> Result<(), InterruptedToolRepairError> {
    if observations.len() > MAX_INTERRUPTION_OBSERVATIONS {
        return Err(error(
            InterruptedToolRepairIssue::InvalidInput,
            FIELD_OBSERVATION,
        ));
    }
    for observation in observations {
        safe_label(observation.run_id.as_str(), FIELD_OBSERVATION)?;
        if observation.affected_call_ids.is_empty()
            || observation.affected_call_ids.len() > MAX_TOTAL_TOOL_CALLS
        {
            return Err(error(
                InterruptedToolRepairIssue::InvalidInput,
                FIELD_OBSERVATION,
            ));
        }
        let mut calls = std::collections::BTreeSet::new();
        for call_id in &observation.affected_call_ids {
            safe_label(call_id.as_str(), FIELD_OBSERVATION)?;
            if !calls.insert(call_id.as_str()) {
                return Err(error(
                    InterruptedToolRepairIssue::ConflictingEvidence,
                    FIELD_OBSERVATION,
                ));
            }
        }
    }
    debug_assert!(observations.len() <= MAX_INTERRUPTION_OBSERVATIONS);
    debug_assert!(
        observations
            .iter()
            .all(|observation| { observation.affected_call_ids.len() <= MAX_TOTAL_TOOL_CALLS })
    );
    Ok(())
}

const fn observation_reason(kind: InterruptedToolObservationKind) -> InterruptedToolReason {
    match kind {
        InterruptedToolObservationKind::UserCancellation => InterruptedToolReason::UserCancelled,
        InterruptedToolObservationKind::HostCancellation => InterruptedToolReason::HostCancelled,
        InterruptedToolObservationKind::UncleanShutdown => InterruptedToolReason::UncleanShutdown,
    }
}

fn repair_mode(
    snapshot: &InterruptedToolBranchSnapshot,
    anchor_index: usize,
) -> InterruptedToolRepairMode {
    let has_later_non_result = snapshot
        .entries
        .iter()
        .skip(anchor_index.saturating_add(1))
        .any(|entry| {
            !matches!(
                entry.kind,
                InterruptedToolEntryKind::ToolResult
                    | InterruptedToolEntryKind::SyntheticToolResult
            )
        });
    if has_later_non_result {
        InterruptedToolRepairMode::HistoricalBranch
    } else {
        InterruptedToolRepairMode::TailAppend
    }
}

fn build_repair_actions(
    snapshot: &InterruptedToolBranchSnapshot,
    diagnosis: &InterruptedToolDiagnosis,
    observations: &[InterruptedToolObservation],
) -> Result<Vec<InterruptedToolRepairAction>, InterruptedToolRepairError> {
    let mut actions = Vec::with_capacity(diagnosis.unmatched.len());
    for gap in &diagnosis.unmatched {
        let reason = reduce_interrupted_tool_reason(gap, observations)?;
        let action_hash = hash_owned_parts(
            INTERRUPTED_TOOL_ACTION_HASH_DOMAIN,
            &[
                snapshot.session_id.clone(),
                snapshot.branch_id.clone(),
                snapshot.active_leaf_hash.clone(),
                gap.entry_hash.clone(),
                gap.entry_index.to_string(),
                gap.call_index.to_string(),
                gap.run_id.clone(),
                gap.call.call_id.clone(),
                gap.call.tool_identity_hash.clone(),
                reason.as_str().to_string(),
            ],
        );
        actions.push(InterruptedToolRepairAction {
            call_id: gap.call.call_id.clone(),
            tool_identity_hash: gap.call.tool_identity_hash.clone(),
            run_id: gap.run_id.clone(),
            anchor_entry_id: gap.entry_id.clone(),
            reason,
            action_hash,
        });
    }
    debug_assert_eq!(actions.len(), diagnosis.unmatched.len());
    debug_assert!(actions.len() <= MAX_TOTAL_TOOL_CALLS);
    Ok(actions)
}

fn validate_application_draft(
    draft: &InterruptedToolRepairApplicationDraft<'_>,
) -> Result<(), InterruptedToolRepairError> {
    if draft.committed_action_hashes.len() > MAX_TOTAL_TOOL_CALLS
        || draft.previously_applied_plan_hashes.len() > MAX_PREVIOUSLY_APPLIED_PLANS
    {
        return Err(error(
            InterruptedToolRepairIssue::InvalidInput,
            FIELD_APPLICATION,
        ));
    }
    lowercase_blake3(draft.plan_hash, FIELD_APPLICATION)?;
    lowercase_blake3(draft.observed_old_leaf_hash, FIELD_APPLICATION)?;
    lowercase_blake3(draft.observed_new_leaf_hash, FIELD_APPLICATION)?;
    for hash in draft
        .committed_action_hashes
        .iter()
        .chain(draft.previously_applied_plan_hashes.iter())
    {
        lowercase_blake3(hash, FIELD_APPLICATION)?;
    }
    debug_assert!(draft.committed_action_hashes.len() <= MAX_TOTAL_TOOL_CALLS);
    debug_assert!(draft.previously_applied_plan_hashes.len() <= MAX_PREVIOUSLY_APPLIED_PLANS);
    Ok(())
}

fn application_status(
    plan: &InterruptedToolRepairPlan,
    draft: &InterruptedToolRepairApplicationDraft<'_>,
    expected_action_hashes: &[&str],
) -> InterruptedToolRepairApplicationStatus {
    debug_assert_eq!(expected_action_hashes.len(), plan.actions.len());
    debug_assert!(plan.actions.len() <= MAX_TOTAL_TOOL_CALLS);
    if draft.executor_observation == InterruptedToolExecutorObservation::Invoked {
        return InterruptedToolRepairApplicationStatus::BlockedToolExecution;
    }
    if draft.plan_hash != plan.plan_hash {
        return InterruptedToolRepairApplicationStatus::BlockedStale;
    }
    let was_previously_applied = draft
        .previously_applied_plan_hashes
        .iter()
        .any(|hash| *hash == plan.plan_hash);
    if was_previously_applied {
        let has_consistent_actions = draft.committed_action_hashes.is_empty()
            || draft.committed_action_hashes == expected_action_hashes;
        if draft.observed_old_leaf_hash != plan.old_leaf_hash {
            return InterruptedToolRepairApplicationStatus::BlockedStale;
        }
        if draft.observed_new_leaf_hash != plan.proposed_new_leaf_hash {
            return InterruptedToolRepairApplicationStatus::BlockedStale;
        }
        if draft.durability != InterruptedToolDurability::Durable {
            return InterruptedToolRepairApplicationStatus::BlockedStale;
        }
        if draft.replay_state != InterruptedToolReplayState::Clean || !has_consistent_actions {
            return InterruptedToolRepairApplicationStatus::BlockedStale;
        }
        return InterruptedToolRepairApplicationStatus::AlreadyApplied;
    }
    if draft.observed_old_leaf_hash != plan.old_leaf_hash {
        return InterruptedToolRepairApplicationStatus::BlockedStale;
    }
    if draft.durability == InterruptedToolDurability::StoreFailure {
        return InterruptedToolRepairApplicationStatus::BlockedStore;
    }
    let has_complete_actions = draft.committed_action_hashes == expected_action_hashes;
    if draft.durability == InterruptedToolDurability::Partial || !has_complete_actions {
        return InterruptedToolRepairApplicationStatus::BlockedPartial;
    }
    if draft.observed_new_leaf_hash != plan.proposed_new_leaf_hash
        || draft.replay_state != InterruptedToolReplayState::Clean
    {
        return InterruptedToolRepairApplicationStatus::BlockedReplay;
    }
    InterruptedToolRepairApplicationStatus::Applied
}

fn snapshot_hash(snapshot: &InterruptedToolBranchSnapshot) -> String {
    let mut parts = vec![
        snapshot.session_id.clone(),
        snapshot.branch_id.clone(),
        snapshot.active_leaf_hash.clone(),
        snapshot.snapshot_leaf_hash.clone(),
    ];
    for entry in &snapshot.entries {
        parts.extend([
            entry.entry_id.clone(),
            entry.entry_hash.clone(),
            entry.run_id.clone(),
            entry.kind.as_str().to_string(),
        ]);
        for call in &entry.tool_calls {
            parts.extend([call.call_id.clone(), call.tool_identity_hash.clone()]);
        }
        if let Some(result) = &entry.tool_result {
            parts.extend([result.call_id.clone(), result.tool_identity_hash.clone()]);
        }
    }
    let hash = hash_owned_parts(INTERRUPTED_TOOL_SNAPSHOT_HASH_DOMAIN, parts.as_slice());
    debug_assert_eq!(hash.len(), BLAKE3_HEX_CHARS);
    debug_assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()));
    hash
}

pub(super) fn hash_owned_parts(domain: &str, parts: &[String]) -> String {
    let mut material = String::from(domain);
    for part in parts {
        material.push_str(HASH_SEPARATOR);
        material.push_str(part.as_str());
    }
    blake3::hash(material.as_bytes()).to_hex().to_string()
}

pub(super) fn safe_label(
    value: &str,
    field: RepairField,
) -> Result<(), InterruptedToolRepairError> {
    let has_valid_label_chars = !value.is_empty() && value.chars().count() <= MAX_SAFE_LABEL_CHARS;
    let has_valid_chars = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'-'));
    if !has_valid_label_chars || !has_valid_chars {
        return Err(error(InterruptedToolRepairIssue::InvalidInput, field));
    }
    Ok(())
}

pub(super) fn lowercase_blake3(
    value: &str,
    field: RepairField,
) -> Result<(), InterruptedToolRepairError> {
    let is_valid = value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !is_valid {
        return Err(error(InterruptedToolRepairIssue::InvalidInput, field));
    }
    Ok(())
}

pub(super) const fn error(
    issue: InterruptedToolRepairIssue,
    field: RepairField,
) -> InterruptedToolRepairError {
    InterruptedToolRepairError::new(issue, field.0)
}
