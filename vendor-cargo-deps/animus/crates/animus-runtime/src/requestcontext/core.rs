//! Pure request-context reduction, lifetime, and binding logic.

const BLAKE3_HEX_CHARS: usize = 64;
const MAX_SAFE_LABEL_BYTES: usize = 96;
const MAX_KEY_BYTES: usize = 128;
const MAX_SUPERSESSION_COUNT: usize = 64;
const MAX_SKILL_BODY_BYTES: usize = 65_536;
const MIN_PROVIDER_ATTEMPT: u16 = 1;
const PLAN_HASH_DOMAIN: &str = "animus.request-context-plan.v1";
const PAYLOAD_HASH_DOMAIN: &str = "animus.request-context-payload.v1";
const HASH_SEPARATOR: &[u8] = b"\0";

/// Computes the domain-separated BLAKE3 identity for one payload.
#[must_use]
pub fn blake3_payload_identity(body: &[u8]) -> super::data::PayloadIdentity {
    let mut hasher = blake3::Hasher::new();
    hasher.update(PAYLOAD_HASH_DOMAIN.as_bytes());
    hasher.update(HASH_SEPARATOR);
    hasher.update(body);
    super::data::PayloadIdentity::new(hasher.finalize().to_hex().to_string())
}

/// Validates one contribution supplied by an existing owner.
///
/// # Errors
///
/// Returns a bounded issue when metadata, identity, or finite bounds are invalid.
// r[impl agent_harness_runtime.request_context_contributions.contract]
pub fn validate_contribution(
    draft: super::data::ContributionDraft<'_>,
) -> Result<super::data::Contribution, super::data::ContributionIssue> {
    validate_safe_identity(draft.contribution_id)?;
    validate_blake3(draft.payload_identity)?;
    validate_safe_key(draft.deduplication_key)?;
    validate_optional_safe_key(draft.conflict_group)?;
    validate_safe_label(draft.safe_label)?;
    debug_assert!(!draft.contribution_id.is_empty());
    debug_assert_eq!(draft.payload_identity.len(), BLAKE3_HEX_CHARS);
    if draft.budget_cost == 0 || draft.supersedes.len() > MAX_SUPERSESSION_COUNT {
        return Err(super::data::ContributionIssue::BoundExceeded);
    }
    let contribution_id = super::data::ContributionIdentity::new(draft.contribution_id);
    let mut supersedes = Vec::with_capacity(draft.supersedes.len());
    for identity in draft.supersedes {
        validate_safe_identity(identity)?;
        if *identity == draft.contribution_id {
            return Err(super::data::ContributionIssue::InvalidSupersession);
        }
        supersedes.push(super::data::ContributionIdentity::new(*identity));
    }
    supersedes.sort();
    if supersedes.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(super::data::ContributionIssue::InvalidSupersession);
    }
    Ok(super::data::Contribution {
        id: contribution_id,
        source: draft.source,
        trust: draft.trust,
        posture: draft.posture,
        priority: draft.priority,
        budget_cost: draft.budget_cost,
        payload_identity: super::data::PayloadIdentity::new(draft.payload_identity),
        deduplication_key: draft.deduplication_key.to_string(),
        supersedes,
        conflict_group: draft.conflict_group.map(str::to_string),
        lifetime: draft.lifetime,
        sensitivity: draft.sensitivity,
        safe_label: draft.safe_label.to_string(),
    })
}

/// Produces a contribution through the same validation path for every existing owner.
///
/// # Errors
///
/// Returns a bounded issue when the owner-supplied draft is invalid.
pub fn contribution_from_owner(
    draft: super::data::ContributionDraft<'_>,
) -> Result<super::data::Contribution, super::data::ContributionIssue> {
    validate_contribution(draft)
}

/// Reduces contributions after validating all caller-declared mandatory identities.
///
/// # Errors
///
/// Returns a missing-mandatory or normal reduction issue before provider composition.
// r[impl agent_harness_runtime.request_context_contributions.reducer]
pub fn reduce_required_contributions(
    input: super::data::ContributionReductionInput<'_>,
) -> Result<super::data::ContributionPlan, super::data::ContributionIssue> {
    validate_required_contributions(input.candidates, input.required)?;
    reduce_contributions(input.candidates, input.binding, input.policy)
}

/// Reduces admitted contribution metadata into one deterministic request plan.
///
/// # Errors
///
/// Returns a bounded fail-closed issue before provider composition.
///
/// # Panics
///
/// Panics only if selected budget accounting violates internal checked arithmetic.
pub fn reduce_contributions(
    candidates: &[super::data::Contribution],
    binding: &super::data::ContributionBinding,
    policy: super::data::ContributionPolicy,
) -> Result<super::data::ContributionPlan, super::data::ContributionIssue> {
    validate_binding(binding)?;
    validate_policy(policy, candidates.len())?;
    let canonical = canonical_candidates(candidates)?;
    let supersession = apply_supersession(canonical.as_slice())?;
    let deduplication = apply_deduplication(supersession.selected)?;
    let conflicts = apply_conflicts(deduplication.selected)?;
    let budget_selection = apply_budget(conflicts.selected, policy.context_budget)?;
    let mut selected = budget_selection.selected;
    let selected_budget = budget_selection.used;
    let mut omitted = Vec::with_capacity(candidates.len());
    omitted.extend(supersession.omitted);
    omitted.extend(deduplication.omitted);
    omitted.extend(conflicts.omitted);
    omitted.extend(budget_selection.omitted);
    if omitted.len() > usize::from(policy.max_omissions) {
        return Err(super::data::ContributionIssue::BoundExceeded);
    }
    selected.sort_by(selection_order);
    omitted.sort_by(|left, right| left.contribution_id.cmp(&right.contribution_id));
    let plan_id = plan_identity(
        binding,
        selected.as_slice(),
        omitted.as_slice(),
        selected_budget,
    );
    assert!(selected_budget <= policy.context_budget);
    assert!(!plan_id.as_str().is_empty());
    Ok(super::data::ContributionPlan {
        plan_id,
        binding: binding.clone(),
        selected,
        omissions: omitted,
        selected_budget,
    })
}

/// Builds the exact contribution binding below an existing model-invocation plan.
#[must_use]
// r[impl agent_harness_runtime.request_context_contributions.composition]
pub fn binding_from_model_invocation(
    plan: &crate::ModelInvocationPlan,
    provider_request: super::data::ProviderRequestIdentity,
) -> super::data::ContributionBinding {
    super::data::ContributionBinding {
        target: super::data::ModelTargetIdentity::new(plan.target().target_hash()),
        profile: super::data::ModelProfileIdentity::new(plan.profile_hash()),
        prompt_plan: super::data::PromptPlanIdentity::new(plan.prompt_plan_hash()),
        invocation: super::data::ModelInvocationIdentity::new(plan.plan_hash()),
        policy: super::data::PolicyIdentity::new(plan.policy_hash()),
        authority: super::data::AuthorityIdentity::new(plan.capability_plan_hash()),
        provider_request,
    }
}

/// Validates that a selected contribution plan still binds the current invocation facts.
///
/// # Errors
///
/// Returns invocation drift when any exact binding changed.
pub fn validate_invocation_binding(
    plan: &super::data::ContributionPlan,
    current: &super::data::ContributionBinding,
) -> Result<(), super::data::ContributionIssue> {
    if plan.binding != *current {
        return Err(super::data::ContributionIssue::InvocationDrift);
    }
    Ok(())
}

/// Validates contribution binding against a current model-invocation plan.
///
/// # Errors
///
/// Returns invocation drift after model fallback, profile, prompt, policy, or authority change.
pub fn validate_model_invocation_binding(
    contribution_plan: &super::data::ContributionPlan,
    model_plan: &crate::ModelInvocationPlan,
    provider_request: super::data::ProviderRequestIdentity,
) -> Result<(), super::data::ContributionIssue> {
    let current = binding_from_model_invocation(model_plan, provider_request);
    validate_invocation_binding(contribution_plan, &current)
}

/// Validates payload views at the last prompt-assembly boundary.
///
/// # Errors
///
/// Returns a bounded issue for missing, duplicate, stale, oversized, or unselected payloads.
pub fn validate_selected_payloads(
    plan: &super::data::ContributionPlan,
    payloads: &[super::data::ContributionPayloadView<'_>],
) -> Result<(), super::data::ContributionIssue> {
    if payloads.len() != plan.selected.len() {
        return Err(super::data::ContributionIssue::InvocationDrift);
    }
    debug_assert_eq!(payloads.len(), plan.selected.len());
    debug_assert!(!plan.plan_id.as_str().is_empty());
    let mut seen = std::collections::BTreeSet::new();
    for payload in payloads {
        if !seen.insert(payload.contribution_id.as_str()) {
            return Err(super::data::ContributionIssue::DuplicateContribution);
        }
        let Some(selected) = plan
            .selected
            .iter()
            .find(|candidate| candidate.id == *payload.contribution_id)
        else {
            return Err(super::data::ContributionIssue::InvocationDrift);
        };
        if selected.payload_identity != *payload.payload_identity
            || blake3_payload_identity(payload.body.as_bytes()) != *payload.payload_identity
        {
            return Err(super::data::ContributionIssue::InvalidPayloadIdentity);
        }
    }
    Ok(())
}

/// Starts an exact skill turn before body loading.
///
/// # Errors
///
/// Returns a bounded identity issue for malformed selection facts.
// r[impl agent_harness_runtime.request_context_contributions.skill_turn]
pub fn begin_skill_turn(
    binding: super::data::SkillSelectionBinding,
) -> Result<super::data::SkillTurn, super::data::ContributionIssue> {
    validate_skill_binding(&binding)?;
    Ok(super::data::SkillTurn {
        binding,
        state: super::data::SkillTurnState::Selected,
    })
}

/// Admits the exact current body and creates one next-response contribution.
///
/// # Errors
///
/// Returns a stale-body, transition, bound, or validation issue.
pub fn admit_loaded_skill_body(
    turn: &super::data::SkillTurn,
    loaded: &super::data::LoadedSkillBody,
    policy: super::data::SkillContributionPolicy<'_>,
) -> Result<
    (
        super::data::SkillTurn,
        super::data::Contribution,
        super::data::SkillSelectionResult,
    ),
    super::data::ContributionIssue,
> {
    if turn.state != super::data::SkillTurnState::Selected {
        return Err(super::data::ContributionIssue::InvalidSkillTransition);
    }
    if loaded.body.is_empty() || loaded.body.len() > MAX_SKILL_BODY_BYTES {
        return Err(super::data::ContributionIssue::BoundExceeded);
    }
    if loaded.body_id != turn.binding.body
        || blake3_payload_identity(loaded.body.as_bytes()) != loaded.body_id
    {
        return Err(super::data::ContributionIssue::StaleSkillBody);
    }
    debug_assert!(!loaded.body.is_empty());
    debug_assert_eq!(loaded.body_id, turn.binding.body);
    let contribution = validate_contribution(super::data::ContributionDraft {
        contribution_id: turn.binding.contribution.as_str(),
        source: super::data::ContributionSource::Skill,
        trust: super::data::ContributionTrust::Trusted,
        posture: super::data::ContributionPosture::Mandatory,
        priority: policy.priority,
        budget_cost: policy.budget_cost,
        payload_identity: loaded.body_id.as_str(),
        deduplication_key: turn.binding.selection.as_str(),
        supersedes: &[],
        conflict_group: None,
        lifetime: super::data::ContributionLifetime::UntilNextProviderResponse,
        sensitivity: super::data::ContributionSensitivity::Sensitive,
        safe_label: policy.safe_label,
    })?;
    let next_turn = super::data::SkillTurn {
        binding: turn.binding.clone(),
        state: super::data::SkillTurnState::BodyLoaded,
    };
    let result = super::data::SkillSelectionResult {
        selection_id: turn.binding.selection.clone(),
        manifest_id: turn.binding.manifest.clone(),
        body_id: turn.binding.body.clone(),
        contribution_id: turn.binding.contribution.clone(),
        lifetime: super::data::ContributionLifetime::UntilNextProviderResponse,
        next_response_required: true,
    };
    Ok((next_turn, contribution, result))
}

/// Applies the exact terminal observation for one finite contribution lifetime.
///
/// # Errors
///
/// Returns invocation drift when the observation does not match the bound lifetime.
// r[impl agent_harness_runtime.request_context_contributions.retention]
pub fn advance_contribution_lifetime(
    contribution: &super::data::Contribution,
    bound_request: &super::data::ProviderRequestIdentity,
    bound_turn: &super::data::LogicalTurnIdentity,
    observation: &super::data::ContributionTerminalObservation,
) -> Result<super::data::ContributionLifetimeState, super::data::ContributionIssue> {
    validate_safe_identity(bound_request.as_str())?;
    validate_safe_identity(bound_turn.as_str())?;
    debug_assert!(!contribution.id.as_str().is_empty());
    debug_assert!(contribution.budget_cost > 0);
    let is_exact = match (contribution.lifetime, observation) {
        (
            super::data::ContributionLifetime::CurrentRequest,
            super::data::ContributionTerminalObservation::RequestAdmitted(request_id),
        ) => request_id == bound_request,
        (
            super::data::ContributionLifetime::UntilNextProviderResponse,
            super::data::ContributionTerminalObservation::ProviderResponse {
                request_id,
                response_id,
            },
        ) => request_id == bound_request && validate_safe_identity(response_id.as_str()).is_ok(),
        (
            super::data::ContributionLifetime::CurrentLogicalTurn,
            super::data::ContributionTerminalObservation::LogicalTurnCompleted(turn_id),
        ) => turn_id == bound_turn,
        (
            super::data::ContributionLifetime::CurrentRequest,
            super::data::ContributionTerminalObservation::ProviderResponse { .. }
            | super::data::ContributionTerminalObservation::LogicalTurnCompleted(_),
        )
        | (
            super::data::ContributionLifetime::UntilNextProviderResponse,
            super::data::ContributionTerminalObservation::RequestAdmitted(_)
            | super::data::ContributionTerminalObservation::LogicalTurnCompleted(_),
        )
        | (
            super::data::ContributionLifetime::CurrentLogicalTurn,
            super::data::ContributionTerminalObservation::RequestAdmitted(_)
            | super::data::ContributionTerminalObservation::ProviderResponse { .. },
        ) => false,
    };
    if !is_exact {
        return Err(super::data::ContributionIssue::InvocationDrift);
    }
    Ok(super::data::ContributionLifetimeState::Expired)
}

/// Binds a loaded skill contribution to one exact provider request or same-request retry.
///
/// This compatibility name keeps the request boundary explicit in the public API.
///
/// # Errors
///
/// Returns drift or an invalid transition before provider dispatch.
pub fn bind_skill_provider(
    turn: &super::data::SkillTurn,
    plan: &super::data::ContributionPlan,
    observation: &super::data::SkillProviderRequestObservation,
) -> Result<super::data::SkillTurn, super::data::ContributionIssue> {
    validate_skill_provider_binding(turn, plan, observation)?;
    debug_assert!(observation.contribution_selected);
    debug_assert!(
        plan.selected
            .iter()
            .any(|candidate| candidate.id == turn.binding.contribution)
    );
    match &turn.state {
        super::data::SkillTurnState::BodyLoaded => {
            if observation.attempt != MIN_PROVIDER_ATTEMPT {
                return Err(super::data::ContributionIssue::InvalidSkillTransition);
            }
        }
        super::data::SkillTurnState::RequestBound {
            request_id,
            attempt,
        } => {
            if request_id != &observation.request_id || observation.attempt <= *attempt {
                return Err(super::data::ContributionIssue::InvalidSkillTransition);
            }
        }
        super::data::SkillTurnState::Selected
        | super::data::SkillTurnState::Terminal { .. }
        | super::data::SkillTurnState::Expired => {
            return Err(super::data::ContributionIssue::InvalidSkillTransition);
        }
    }
    Ok(super::data::SkillTurn {
        binding: turn.binding.clone(),
        state: super::data::SkillTurnState::RequestBound {
            request_id: observation.request_id.clone(),
            attempt: observation.attempt,
        },
    })
}

/// Applies one correlated terminal provider observation.
///
/// # Errors
///
/// Returns an invalid transition for stale or uncorrelated responses.
pub fn observe_skill_provider_response(
    turn: &super::data::SkillTurn,
    observation: &super::data::SkillProviderResponseObservation,
) -> Result<super::data::SkillTurn, super::data::ContributionIssue> {
    let super::data::SkillTurnState::RequestBound { request_id, .. } = &turn.state else {
        return Err(super::data::ContributionIssue::InvalidSkillTransition);
    };
    if request_id != &observation.request_id {
        return Err(super::data::ContributionIssue::InvalidSkillTransition);
    }
    validate_safe_identity(observation.response_id.as_str())?;
    Ok(super::data::SkillTurn {
        binding: turn.binding.clone(),
        state: super::data::SkillTurnState::Terminal {
            response_id: observation.response_id.clone(),
            outcome: observation.outcome,
        },
    })
}

/// Expires a selected or loaded skill through an explicit pre-dispatch failure path.
///
/// # Errors
///
/// Returns response-pending after a provider request consumed the contribution.
pub fn expire_unconsumed_skill_turn(
    turn: &super::data::SkillTurn,
) -> Result<super::data::SkillTurn, super::data::ContributionIssue> {
    match turn.state {
        super::data::SkillTurnState::Selected | super::data::SkillTurnState::BodyLoaded => {
            Ok(super::data::SkillTurn {
                binding: turn.binding.clone(),
                state: super::data::SkillTurnState::Expired,
            })
        }
        super::data::SkillTurnState::RequestBound { .. } => {
            Err(super::data::ContributionIssue::SkillResponsePending)
        }
        super::data::SkillTurnState::Terminal { .. } | super::data::SkillTurnState::Expired => {
            Err(super::data::ContributionIssue::InvalidSkillTransition)
        }
    }
}

/// Checks whether logical turn completion can proceed.
///
/// # Errors
///
/// Returns response-pending until a terminal response or explicit pre-dispatch expiration exists.
pub fn admit_skill_turn_completion(
    turn: &super::data::SkillTurn,
) -> Result<(), super::data::ContributionIssue> {
    match turn.state {
        super::data::SkillTurnState::Terminal { .. } | super::data::SkillTurnState::Expired => {
            Ok(())
        }
        super::data::SkillTurnState::Selected
        | super::data::SkillTurnState::BodyLoaded
        | super::data::SkillTurnState::RequestBound { .. } => {
            Err(super::data::ContributionIssue::SkillResponsePending)
        }
    }
}

fn validate_required_contributions(
    candidates: &[super::data::Contribution],
    required: &[super::data::ContributionIdentity],
) -> Result<(), super::data::ContributionIssue> {
    let mut seen = std::collections::BTreeSet::new();
    for required_id in required {
        validate_safe_identity(required_id.as_str())?;
        if !seen.insert(required_id) {
            return Err(super::data::ContributionIssue::DuplicateContribution);
        }
        let has_required = candidates.iter().any(|candidate| {
            candidate.id == *required_id
                && candidate.posture == super::data::ContributionPosture::Mandatory
        });
        if !has_required {
            return Err(super::data::ContributionIssue::MissingMandatoryContribution);
        }
    }
    debug_assert_eq!(seen.len(), required.len());
    debug_assert!(required.len() <= candidates.len());
    Ok(())
}

fn canonical_candidates(
    candidates: &[super::data::Contribution],
) -> Result<Vec<super::data::Contribution>, super::data::ContributionIssue> {
    let mut canonical = candidates.to_vec();
    canonical.sort_by(|left, right| left.id.cmp(&right.id));
    if canonical.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(super::data::ContributionIssue::DuplicateContribution);
    }
    Ok(canonical)
}

struct ArbitrationSelection {
    selected: Vec<super::data::Contribution>,
    omitted: Vec<super::data::ContributionOmission>,
}

fn apply_supersession(
    candidates: &[super::data::Contribution],
) -> Result<ArbitrationSelection, super::data::ContributionIssue> {
    debug_assert!(
        candidates
            .iter()
            .all(|candidate| !candidate.id.as_str().is_empty())
    );
    debug_assert!(candidates.iter().all(|candidate| candidate.budget_cost > 0));
    let known = candidates
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut superseded_by = std::collections::BTreeMap::new();
    for candidate in candidates {
        for superseded in &candidate.supersedes {
            if !known.contains(superseded) || superseded_by.contains_key(superseded) {
                return Err(super::data::ContributionIssue::InvalidSupersession);
            }
            if superseded_by.len() >= candidates.len() {
                return Err(super::data::ContributionIssue::BoundExceeded);
            }
            superseded_by.insert(superseded.clone(), candidate);
        }
    }
    let mut selected = Vec::with_capacity(candidates.len());
    let mut omitted = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let Some(replacement) = superseded_by.get(&candidate.id) else {
            selected.push(candidate.clone());
            continue;
        };
        if candidate.posture == super::data::ContributionPosture::Mandatory
            && replacement.posture != super::data::ContributionPosture::Mandatory
        {
            return Err(super::data::ContributionIssue::MandatoryOmitted);
        }
        omitted.push(omission(
            candidate,
            super::data::ContributionOmissionClass::Superseded,
        ));
    }
    Ok(ArbitrationSelection { selected, omitted })
}

fn apply_deduplication(
    candidates: Vec<super::data::Contribution>,
) -> Result<ArbitrationSelection, super::data::ContributionIssue> {
    debug_assert!(
        candidates
            .iter()
            .all(|candidate| !candidate.deduplication_key.is_empty())
    );
    debug_assert!(candidates.iter().all(|candidate| candidate.budget_cost > 0));
    let candidate_count = candidates.len();
    let mut groups: std::collections::BTreeMap<String, Vec<super::data::Contribution>> =
        std::collections::BTreeMap::new();
    for candidate in candidates {
        groups
            .entry(candidate.deduplication_key.clone())
            .or_default()
            .push(candidate);
    }
    let mut selected = Vec::with_capacity(groups.len());
    let mut omitted = Vec::with_capacity(candidate_count);
    for mut group in groups.into_values() {
        group.sort_by(selection_order);
        let winner = group.remove(0);
        if group
            .iter()
            .any(|candidate| candidate.posture == super::data::ContributionPosture::Mandatory)
        {
            return Err(super::data::ContributionIssue::MandatoryOmitted);
        }
        for candidate in group {
            omitted.push(omission(
                &candidate,
                super::data::ContributionOmissionClass::Deduplicated,
            ));
        }
        selected.push(winner);
    }
    Ok(ArbitrationSelection { selected, omitted })
}

fn apply_conflicts(
    candidates: Vec<super::data::Contribution>,
) -> Result<ArbitrationSelection, super::data::ContributionIssue> {
    debug_assert!(
        candidates
            .iter()
            .all(|candidate| !candidate.id.as_str().is_empty())
    );
    debug_assert!(candidates.iter().all(|candidate| candidate.budget_cost > 0));
    let candidate_count = candidates.len();
    let mut selected = Vec::with_capacity(candidate_count);
    let mut groups: std::collections::BTreeMap<String, Vec<super::data::Contribution>> =
        std::collections::BTreeMap::new();
    for candidate in candidates {
        if let Some(group) = candidate.conflict_group.clone() {
            groups.entry(group).or_default().push(candidate);
        } else {
            selected.push(candidate);
        }
    }
    let mut omitted = Vec::with_capacity(candidate_count);
    for mut group in groups.into_values() {
        let mandatory_count = group
            .iter()
            .filter(|candidate| candidate.posture == super::data::ContributionPosture::Mandatory)
            .count();
        if mandatory_count > 1 {
            return Err(super::data::ContributionIssue::MandatoryConflict);
        }
        group.sort_by(selection_order);
        let winner = group.remove(0);
        if group
            .iter()
            .any(|candidate| candidate.posture == super::data::ContributionPosture::Mandatory)
        {
            return Err(super::data::ContributionIssue::MandatoryOmitted);
        }
        for candidate in group {
            omitted.push(omission(
                &candidate,
                super::data::ContributionOmissionClass::Conflict,
            ));
        }
        selected.push(winner);
    }
    Ok(ArbitrationSelection { selected, omitted })
}

struct BudgetSelection {
    selected: Vec<super::data::Contribution>,
    omitted: Vec<super::data::ContributionOmission>,
    used: u32,
}

fn apply_budget(
    candidates: Vec<super::data::Contribution>,
    context_budget: u32,
) -> Result<BudgetSelection, super::data::ContributionIssue> {
    debug_assert!(context_budget > 0);
    debug_assert!(candidates.iter().all(|candidate| candidate.budget_cost > 0));
    let mut mandatory = candidates
        .iter()
        .filter(|candidate| candidate.posture == super::data::ContributionPosture::Mandatory)
        .cloned()
        .collect::<Vec<_>>();
    let mut advisory = candidates
        .into_iter()
        .filter(|candidate| candidate.posture == super::data::ContributionPosture::Advisory)
        .collect::<Vec<_>>();
    mandatory.sort_by(selection_order);
    advisory.sort_by(selection_order);
    let mut used = checked_budget_sum(mandatory.as_slice())?;
    if used > context_budget {
        return Err(super::data::ContributionIssue::MandatoryBudgetExceeded);
    }
    let mut selected = mandatory;
    let mut omitted = Vec::with_capacity(advisory.len());
    for candidate in advisory {
        let Some(next_used) = used.checked_add(candidate.budget_cost) else {
            return Err(super::data::ContributionIssue::BoundExceeded);
        };
        if next_used <= context_budget {
            used = next_used;
            selected.push(candidate);
        } else {
            omitted.push(omission(
                &candidate,
                super::data::ContributionOmissionClass::Budget,
            ));
        }
    }
    Ok(BudgetSelection {
        selected,
        omitted,
        used,
    })
}

fn checked_budget_sum(
    candidates: &[super::data::Contribution],
) -> Result<u32, super::data::ContributionIssue> {
    let mut total = 0u32;
    for candidate in candidates {
        total = total
            .checked_add(candidate.budget_cost)
            .ok_or(super::data::ContributionIssue::BoundExceeded)?;
    }
    Ok(total)
}

fn selection_order(
    left: &super::data::Contribution,
    right: &super::data::Contribution,
) -> std::cmp::Ordering {
    posture_rank(right.posture)
        .cmp(&posture_rank(left.posture))
        .then_with(|| right.priority.cmp(&left.priority))
        .then_with(|| left.id.cmp(&right.id))
}

const fn posture_rank(posture: super::data::ContributionPosture) -> u8 {
    match posture {
        super::data::ContributionPosture::Mandatory => 1,
        super::data::ContributionPosture::Advisory => 0,
    }
}

fn omission(
    contribution: &super::data::Contribution,
    class: super::data::ContributionOmissionClass,
) -> super::data::ContributionOmission {
    super::data::ContributionOmission {
        contribution_id: contribution.id.clone(),
        class,
        safe_label: contribution.safe_label.clone(),
    }
}

fn plan_identity(
    binding: &super::data::ContributionBinding,
    selected: &[super::data::Contribution],
    omitted: &[super::data::ContributionOmission],
    selected_budget: u32,
) -> super::data::ContributionPlanIdentity {
    debug_assert!(
        selected
            .iter()
            .all(|candidate| !candidate.id.as_str().is_empty())
    );
    debug_assert!(omitted.iter().all(|entry| !entry.safe_label.is_empty()));
    let mut hasher = blake3::Hasher::new();
    update_hash(&mut hasher, PLAN_HASH_DOMAIN);
    for value in [
        binding.target.as_str(),
        binding.profile.as_str(),
        binding.prompt_plan.as_str(),
        binding.invocation.as_str(),
        binding.policy.as_str(),
        binding.authority.as_str(),
        binding.provider_request.as_str(),
    ] {
        update_hash(&mut hasher, value);
    }
    update_hash(&mut hasher, selected_budget.to_string().as_str());
    for candidate in selected {
        update_contribution_hash(&mut hasher, candidate);
    }
    for entry in omitted {
        update_hash(&mut hasher, entry.contribution_id.as_str());
        update_hash(&mut hasher, entry.class.as_str());
        update_hash(&mut hasher, entry.safe_label.as_str());
    }
    super::data::ContributionPlanIdentity::new(hasher.finalize().to_hex().to_string())
}

fn update_contribution_hash(hasher: &mut blake3::Hasher, value: &super::data::Contribution) {
    debug_assert!(!value.id.as_str().is_empty());
    debug_assert_eq!(value.payload_identity.as_str().len(), BLAKE3_HEX_CHARS);
    for part in [
        value.id.as_str(),
        value.source.as_str(),
        value.trust.as_str(),
        value.posture.as_str(),
        value.payload_identity.as_str(),
        value.deduplication_key.as_str(),
        value.conflict_group.as_deref().unwrap_or("none"),
        value.lifetime.as_str(),
        value.sensitivity.as_str(),
        value.safe_label.as_str(),
    ] {
        update_hash(hasher, part);
    }
    update_hash(hasher, value.priority.to_string().as_str());
    update_hash(hasher, value.budget_cost.to_string().as_str());
    for superseded in &value.supersedes {
        update_hash(hasher, superseded.as_str());
    }
}

fn update_hash(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update(HASH_SEPARATOR);
}

fn validate_policy(
    policy: super::data::ContributionPolicy,
    candidate_count: usize,
) -> Result<(), super::data::ContributionIssue> {
    if policy.max_contributions == 0 {
        return Err(super::data::ContributionIssue::BoundExceeded);
    }
    if policy.context_budget == 0 {
        return Err(super::data::ContributionIssue::BoundExceeded);
    }
    if policy.max_omissions == 0 {
        return Err(super::data::ContributionIssue::BoundExceeded);
    }
    if candidate_count > usize::from(policy.max_contributions) {
        return Err(super::data::ContributionIssue::BoundExceeded);
    }
    Ok(())
}

fn validate_binding(
    binding: &super::data::ContributionBinding,
) -> Result<(), super::data::ContributionIssue> {
    for value in [
        binding.target.as_str(),
        binding.profile.as_str(),
        binding.prompt_plan.as_str(),
        binding.invocation.as_str(),
        binding.policy.as_str(),
        binding.authority.as_str(),
        binding.provider_request.as_str(),
    ] {
        validate_safe_identity(value)?;
    }
    Ok(())
}

fn validate_skill_binding(
    binding: &super::data::SkillSelectionBinding,
) -> Result<(), super::data::ContributionIssue> {
    for value in [
        binding.session.as_str(),
        binding.turn.as_str(),
        binding.target.as_str(),
        binding.profile.as_str(),
        binding.manifest.as_str(),
        binding.selection.as_str(),
        binding.contribution.as_str(),
    ] {
        validate_safe_identity(value)?;
    }
    validate_blake3(binding.body.as_str())
}

fn validate_skill_provider_binding(
    turn: &super::data::SkillTurn,
    plan: &super::data::ContributionPlan,
    observation: &super::data::SkillProviderRequestObservation,
) -> Result<(), super::data::ContributionIssue> {
    let is_selected = plan
        .selected
        .iter()
        .any(|candidate| candidate.id == turn.binding.contribution);
    let is_exact = observation.target_id == turn.binding.target
        && observation.profile_id == turn.binding.profile
        && observation.contribution_plan_id == plan.plan_id
        && observation.contribution_selected
        && is_selected
        && observation.request_id == plan.binding.provider_request;
    if !is_exact {
        return Err(super::data::ContributionIssue::InvocationDrift);
    }
    Ok(())
}

fn validate_safe_identity(value: &str) -> Result<(), super::data::ContributionIssue> {
    if !safe_metadata(value, MAX_KEY_BYTES) {
        return Err(super::data::ContributionIssue::MissingIdentity);
    }
    Ok(())
}

fn validate_safe_key(value: &str) -> Result<(), super::data::ContributionIssue> {
    if !safe_metadata(value, MAX_KEY_BYTES) {
        return Err(super::data::ContributionIssue::UnsafeMetadata);
    }
    Ok(())
}

fn validate_optional_safe_key(value: Option<&str>) -> Result<(), super::data::ContributionIssue> {
    if value.is_some_and(|candidate| !safe_metadata(candidate, MAX_KEY_BYTES)) {
        return Err(super::data::ContributionIssue::UnsafeMetadata);
    }
    Ok(())
}

fn validate_safe_label(value: &str) -> Result<(), super::data::ContributionIssue> {
    if !safe_metadata(value, MAX_SAFE_LABEL_BYTES) {
        return Err(super::data::ContributionIssue::UnsafeMetadata);
    }
    Ok(())
}

fn safe_metadata(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/')
        })
        && !value.contains("..")
        && !value.starts_with('/')
}

fn validate_blake3(value: &str) -> Result<(), super::data::ContributionIssue> {
    let is_valid = value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !is_valid {
        return Err(super::data::ContributionIssue::InvalidPayloadIdentity);
    }
    Ok(())
}
