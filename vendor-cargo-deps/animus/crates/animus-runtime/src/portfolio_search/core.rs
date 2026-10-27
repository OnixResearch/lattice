#![cfg_attr(animus_octet, allow(tigerstyle::non_trait_imports))]

use std::collections::{BTreeMap, BTreeSet};

use super::{
    BLAKE3_ALGORITHM, BLAKE3_HEX_LENGTH, MAX_REFERENCE_COUNT, MAX_SAFE_LABEL_CHARS,
    PORTFOLIO_HASH_DOMAIN, REQUIRED_COUNT,
    data::{
        ApproachRecord, ApproachState, AuditOutcome, GapStrength, PortfolioAudit,
        PortfolioAuditFinding, PortfolioAuditRisk, PortfolioBlocker, PortfolioBlockerClass,
        PortfolioContract, PortfolioDispatch, PortfolioError, PortfolioErrorKind,
        PortfolioFingerprint, PortfolioPhase, PortfolioReceipt, PortfolioResult, PortfolioState,
        PortfolioUsage, PortfolioValidationReceipt, PortfolioValidationStatus, ReopenNovelty,
    },
};

#[derive(Clone, Copy)]
struct ErrorField(&'static str);

#[derive(Clone, Copy)]
struct CheckedAdd<T> {
    current: T,
    increment: T,
}

struct DispatchUsageProjection {
    rounds: u32,
    tool_calls: u32,
    tokens: u64,
}

const FIELD_APPROACH: ErrorField = ErrorField("approach");
const FIELD_AUDIT: ErrorField = ErrorField("audit");
const FIELD_BLOCKER: ErrorField = ErrorField("blocker");
const FIELD_BUDGET: ErrorField = ErrorField("budget");
const FIELD_CONTRACT: ErrorField = ErrorField("contract");
const FIELD_DISPATCH: ErrorField = ErrorField("dispatch");
const FIELD_FINGERPRINT: ErrorField = ErrorField("fingerprint");
const FIELD_PHASE: ErrorField = ErrorField("phase");
const FIELD_RECEIPT: ErrorField = ErrorField("receipt");
const RAW_HOME_PATH_MARKER: &str = "/home/";
const RAW_USER_PATH_MARKER: &str = "/users/";
const RAW_PROMPT_MARKER: &str = "raw prompt";
const RAW_TOOL_OUTPUT_MARKER: &str = "tool_output:";
const RAW_PROVIDER_PAYLOAD_MARKER: &str = "provider_payload:";
const RAW_SECRET_MARKER: &str = "api_key";
const RAW_SECRET_ASSIGNMENT_MARKER: &str = "secret=";

/// Computes a domain-separated BLAKE3 fingerprint.
#[must_use]
pub fn blake3_portfolio_fingerprint(label: &str, bytes: &[u8]) -> PortfolioFingerprint {
    let mut hasher = blake3::Hasher::new();
    hash_part(&mut hasher, PORTFOLIO_HASH_DOMAIN.as_bytes());
    hash_part(&mut hasher, label.as_bytes());
    hash_part(&mut hasher, bytes);
    PortfolioFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Computes the canonical identity of a validated portfolio contract.
#[must_use]
pub fn portfolio_contract_fingerprint(contract: &PortfolioContract) -> PortfolioFingerprint {
    let mut hasher = portfolio_hasher("contract");
    hash_part(&mut hasher, contract.contract_id.as_bytes());
    hash_fingerprint(&mut hasher, &contract.objective);
    hash_text_collection(&mut hasher, &contract.completion_evidence);
    hash_text_collection(&mut hasher, &contract.false_completion_cases);
    for risk in &contract.required_audit_risks {
        hash_audit_risk(&mut hasher, risk);
    }
    hash_fingerprint(&mut hasher, &contract.source_policy);
    hash_fingerprint(&mut hasher, &contract.validator_policy);
    hash_part(&mut hasher, &contract.budget.rounds.to_le_bytes());
    hash_part(
        &mut hasher,
        &contract.budget.approach_capacity.to_le_bytes(),
    );
    hash_part(&mut hasher, &contract.budget.tool_calls.to_le_bytes());
    hash_part(&mut hasher, &contract.budget.tokens.to_le_bytes());
    let fingerprint =
        PortfolioFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string());
    debug_assert_eq!(fingerprint.algorithm, BLAKE3_ALGORITHM);
    debug_assert_eq!(fingerprint.value_hex.len(), BLAKE3_HEX_LENGTH);
    fingerprint
}

/// Computes the selected candidate and audit evidence identity.
#[must_use]
pub fn portfolio_evidence_fingerprint(
    approach: &ApproachRecord,
    audit: &PortfolioAudit,
) -> PortfolioFingerprint {
    let mut hasher = portfolio_hasher("evidence");
    hash_part(&mut hasher, approach.approach_id.as_bytes());
    hash_fingerprint(&mut hasher, &approach.mechanism);
    hash_fingerprint(&mut hasher, &approach.claim);
    hash_text_collection(&mut hasher, &approach.artifact_refs);
    hash_text_collection(&mut hasher, &approach.evidence_refs);
    for (risk, finding) in &audit.findings {
        hash_audit_risk(&mut hasher, risk);
        hash_part(&mut hasher, audit_outcome_label(finding.outcome).as_bytes());
        hash_part(&mut hasher, finding.evidence_ref.as_bytes());
        hash_part(&mut hasher, finding.residual_uncertainty.as_bytes());
    }
    PortfolioFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string())
}

/// Validates a portfolio contract before any exploration begins.
///
/// # Errors
///
/// Returns a typed deterministic error for malformed fingerprints, missing
/// success/failure/audit criteria, unsafe labels, duplicate audit risks, or
/// non-positive budgets.
pub fn validate_portfolio_contract(contract: &PortfolioContract) -> PortfolioResult<()> {
    require_safe_text(&contract.contract_id, FIELD_CONTRACT)?;
    validate_fingerprint(&contract.objective)?;
    validate_fingerprint(&contract.source_policy)?;
    validate_fingerprint(&contract.validator_policy)?;
    require_safe_text_collection(&contract.completion_evidence, FIELD_CONTRACT, true)?;
    require_safe_text_collection(&contract.false_completion_cases, FIELD_CONTRACT, true)?;
    if contract.required_audit_risks.is_empty()
        || contract.required_audit_risks.len() > MAX_REFERENCE_COUNT
    {
        return Err(error(
            PortfolioErrorKind::InvalidContract,
            FIELD_CONTRACT,
            "portfolio contract requires a bounded non-empty audit-risk set",
        ));
    }
    let mut risks = BTreeSet::new();
    for risk in &contract.required_audit_risks {
        validate_audit_risk(risk)?;
        if !risks.insert(risk.clone()) {
            return Err(error(
                PortfolioErrorKind::InvalidContract,
                FIELD_CONTRACT,
                "portfolio contract contains a duplicate audit risk",
            ));
        }
    }
    let is_budget_positive = [
        contract.budget.rounds >= REQUIRED_COUNT,
        contract.budget.approach_capacity >= REQUIRED_COUNT,
        contract.budget.tool_calls >= REQUIRED_COUNT,
        contract.budget.tokens >= u64::from(REQUIRED_COUNT),
    ]
    .into_iter()
    .all(|is_positive| is_positive);
    if !is_budget_positive {
        return Err(error(
            PortfolioErrorKind::BudgetInvalid,
            FIELD_BUDGET,
            "portfolio limits must all be positive",
        ));
    }
    debug_assert!(!risks.is_empty());
    debug_assert_eq!(risks.len(), contract.required_audit_risks.len());
    Ok(())
}

/// Creates a validated portfolio in the framing phase.
///
/// # Errors
///
/// Returns the contract validation error when framing cannot start.
pub fn initialize_portfolio(contract: &PortfolioContract) -> PortfolioResult<PortfolioState> {
    validate_portfolio_contract(contract)?;
    Ok(PortfolioState {
        contract: contract.clone(),
        phase: PortfolioPhase::Framing,
        approaches: BTreeMap::new(),
        usage: PortfolioUsage::default(),
        selected_approach: None,
        audit: None,
        terminal_blocker: None,
        validation_receipt: None,
    })
}

/// Moves a framed portfolio into independent exploration.
///
/// # Errors
///
/// Returns a typed phase or state validation error.
pub fn begin_independent_exploration(state: &PortfolioState) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(state, &[PortfolioPhase::Framing])?;
    let mut next = state.clone();
    next.phase = PortfolioPhase::IndependentExploration;
    Ok(next)
}

/// Validates one host dispatch against phase, independence, references, and budgets.
///
/// # Errors
///
/// Returns a typed error for unsafe metadata, early cross-pollination, unknown
/// references, invalid requests, or budget overflow.
pub fn validate_portfolio_dispatch(
    state: &PortfolioState,
    dispatch: &PortfolioDispatch,
) -> PortfolioResult<()> {
    validate_state(state)?;
    require_phase(
        state,
        &[
            PortfolioPhase::IndependentExploration,
            PortfolioPhase::TargetedRedirection,
        ],
    )?;
    require_safe_text(&dispatch.dispatch_id, FIELD_DISPATCH)?;
    require_safe_text(&dispatch.family_hint, FIELD_DISPATCH)?;
    require_safe_text_collection(&dispatch.disclosed_approach_ids, FIELD_DISPATCH, false)?;
    if dispatch.requested_tokens < u64::from(REQUIRED_COUNT) {
        return Err(error(
            PortfolioErrorKind::BudgetInvalid,
            FIELD_DISPATCH,
            "portfolio dispatch requires a positive token request",
        ));
    }
    if state.phase == PortfolioPhase::IndependentExploration
        && !dispatch.disclosed_approach_ids.is_empty()
    {
        return Err(error(
            PortfolioErrorKind::CrossPollination,
            FIELD_DISPATCH,
            "independent exploration cannot disclose prior approaches",
        ));
    }
    for approach_id in &dispatch.disclosed_approach_ids {
        if !state.approaches.contains_key(approach_id) {
            return Err(error(
                PortfolioErrorKind::MissingApproach,
                FIELD_DISPATCH,
                "dispatch disclosed an unknown approach",
            ));
        }
    }
    let projected = project_dispatch_usage(state, dispatch)?;
    if projected.rounds > state.contract.budget.rounds
        || projected.tool_calls > state.contract.budget.tool_calls
        || projected.tokens > state.contract.budget.tokens
    {
        return Err(error(
            PortfolioErrorKind::BudgetExceeded,
            FIELD_BUDGET,
            "portfolio dispatch exceeds a finite contract limit",
        ));
    }
    debug_assert!(projected.rounds <= state.contract.budget.rounds);
    debug_assert!(projected.tokens <= state.contract.budget.tokens);
    Ok(())
}

fn project_dispatch_usage(
    state: &PortfolioState,
    dispatch: &PortfolioDispatch,
) -> PortfolioResult<DispatchUsageProjection> {
    let rounds = checked_add_u32(
        CheckedAdd {
            current: state.usage.rounds,
            increment: REQUIRED_COUNT,
        },
        FIELD_BUDGET,
    )?;
    let tool_calls = checked_add_u32(
        CheckedAdd {
            current: state.usage.tool_calls,
            increment: dispatch.requested_tool_calls,
        },
        FIELD_BUDGET,
    )?;
    let tokens = checked_add_u64(
        CheckedAdd {
            current: state.usage.tokens,
            increment: dispatch.requested_tokens,
        },
        FIELD_BUDGET,
    )?;
    let projected = DispatchUsageProjection {
        rounds,
        tool_calls,
        tokens,
    };
    debug_assert!(projected.rounds > state.usage.rounds);
    debug_assert!(projected.tokens >= state.usage.tokens);
    Ok(projected)
}

/// Records one validated dispatch and its budget usage.
///
/// # Errors
///
/// Returns the deterministic dispatch validation error without changing state.
pub fn record_portfolio_dispatch(
    state: &PortfolioState,
    dispatch: &PortfolioDispatch,
) -> PortfolioResult<PortfolioState> {
    validate_portfolio_dispatch(state, dispatch)?;
    let projected = project_dispatch_usage(state, dispatch)?;
    let mut next = state.clone();
    next.usage.rounds = projected.rounds;
    next.usage.tool_calls = projected.tool_calls;
    next.usage.tokens = projected.tokens;
    debug_assert!(next.usage.rounds > state.usage.rounds);
    debug_assert!(next.usage.tokens > state.usage.tokens);
    Ok(next)
}

/// Validates one approach registry entry.
///
/// # Errors
///
/// Returns a typed error for unsafe metadata, malformed fingerprints, missing
/// artifacts, inconsistent blockers, or reduction-theater gap states.
pub fn validate_approach_record(approach: &ApproachRecord) -> PortfolioResult<()> {
    require_safe_text(&approach.approach_id, FIELD_APPROACH)?;
    require_safe_text(&approach.family, FIELD_APPROACH)?;
    require_safe_text(&approach.next_check, FIELD_APPROACH)?;
    validate_fingerprint(&approach.mechanism)?;
    validate_fingerprint(&approach.claim)?;
    require_safe_text_collection(&approach.artifact_refs, FIELD_APPROACH, true)?;
    require_safe_text_collection(&approach.evidence_refs, FIELD_APPROACH, false)?;
    validate_approach_blocker_invariant(approach)?;
    Ok(())
}

/// Registers a distinct approach during independent or targeted exploration.
///
/// # Errors
///
/// Returns a typed error for phase violations, duplicate IDs or mechanisms,
/// malformed route state, or approach-budget exhaustion.
pub fn register_approach(
    state: &PortfolioState,
    approach: ApproachRecord,
) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(
        state,
        &[
            PortfolioPhase::IndependentExploration,
            PortfolioPhase::TargetedRedirection,
        ],
    )?;
    validate_approach_record(&approach)?;
    if !is_registration_state_admitted(state.phase, approach.state) {
        return Err(error(
            PortfolioErrorKind::InvalidApproach,
            FIELD_APPROACH,
            "approach state does not match its registration phase",
        ));
    }
    if state.approaches.contains_key(&approach.approach_id) {
        return Err(error(
            PortfolioErrorKind::DuplicateApproachId,
            FIELD_APPROACH,
            "approach identifier already exists",
        ));
    }
    if let Some(existing) = find_mechanism_owner(state, &approach.mechanism, None) {
        return Err(error(
            PortfolioErrorKind::DuplicateMechanism,
            FIELD_APPROACH,
            format!("mechanism already belongs to approach `{existing}`"),
        ));
    }
    let approach_count = checked_add_u32(
        CheckedAdd {
            current: state.usage.approaches,
            increment: REQUIRED_COUNT,
        },
        FIELD_BUDGET,
    )?;
    if approach_count > state.contract.budget.approach_capacity {
        return Err(error(
            PortfolioErrorKind::BudgetExceeded,
            FIELD_BUDGET,
            "approach registration exceeds the finite contract limit",
        ));
    }
    let approach_id = approach.approach_id.clone();
    let mut next = state.clone();
    next.approaches.insert(approach_id.clone(), approach);
    next.usage.approaches = approach_count;
    debug_assert!(next.approaches.contains_key(&approach_id));
    debug_assert_eq!(next.usage.approaches, approach_count);
    Ok(next)
}

/// Moves independent or redirected routes into synthesis.
///
/// # Errors
///
/// Returns a typed error for an illegal phase or empty registry.
pub fn begin_synthesis(state: &PortfolioState) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(
        state,
        &[
            PortfolioPhase::IndependentExploration,
            PortfolioPhase::TargetedRedirection,
        ],
    )?;
    if state.approaches.is_empty() {
        return Err(error(
            PortfolioErrorKind::MissingApproach,
            FIELD_APPROACH,
            "synthesis requires at least one registered approach",
        ));
    }
    let mut next = state.clone();
    for approach in next.approaches.values_mut() {
        if approach.state == ApproachState::Independent {
            approach.state = ApproachState::Active;
        }
    }
    next.phase = PortfolioPhase::Synthesis;
    debug_assert_eq!(next.phase, PortfolioPhase::Synthesis);
    debug_assert!(
        next.approaches
            .values()
            .all(|approach| approach.state != ApproachState::Independent)
    );
    Ok(next)
}

/// Moves synthesis into evidence-driven targeted redirection.
///
/// # Errors
///
/// Returns a typed phase or state validation error.
pub fn begin_targeted_redirection(state: &PortfolioState) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(state, &[PortfolioPhase::Synthesis])?;
    let mut next = state.clone();
    next.phase = PortfolioPhase::TargetedRedirection;
    Ok(next)
}

/// Returns lexically ordered live families with the minimum observed route count.
///
/// # Errors
///
/// Returns a state validation error when registry invariants do not hold.
pub fn underexplored_live_families(state: &PortfolioState) -> PortfolioResult<Vec<String>> {
    validate_state(state)?;
    let mut counts = BTreeMap::<String, u32>::new();
    for approach in state.approaches.values() {
        if is_live_approach(approach.state) {
            let count = counts.entry(approach.family.clone()).or_default();
            *count = checked_add_u32(
                CheckedAdd {
                    current: *count,
                    increment: REQUIRED_COUNT,
                },
                FIELD_APPROACH,
            )?;
        }
    }
    let Some(minimum) = counts.values().copied().min() else {
        return Ok(Vec::new());
    };
    let families = counts
        .iter()
        .filter_map(|(family, count)| (*count == minimum).then_some(family.clone()))
        .collect::<Vec<_>>();
    debug_assert!(!families.is_empty());
    debug_assert!(families.len() <= counts.len());
    Ok(families)
}

/// Blocks or falsifies one route with an exact gap-consistent blocker.
///
/// # Errors
///
/// Returns a typed phase, route, metadata, or blocker mismatch error.
pub fn block_approach(
    state: &PortfolioState,
    approach_id: &str,
    gap_strength: GapStrength,
    blocker: PortfolioBlocker,
) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(
        state,
        &[
            PortfolioPhase::IndependentExploration,
            PortfolioPhase::Synthesis,
            PortfolioPhase::TargetedRedirection,
        ],
    )?;
    require_safe_text(approach_id, FIELD_APPROACH)?;
    validate_blocker(&blocker)?;
    validate_gap_blocker(gap_strength, blocker.class)?;
    let Some(current) = state.approaches.get(approach_id) else {
        return Err(error(
            PortfolioErrorKind::MissingApproach,
            FIELD_APPROACH,
            "cannot block an unknown approach",
        ));
    };
    if matches!(
        current.state,
        ApproachState::Audit | ApproachState::Validated
    ) {
        return Err(error(
            PortfolioErrorKind::InvalidApproach,
            FIELD_APPROACH,
            "audited or validated approaches require their dedicated transitions",
        ));
    }
    let mut next = state.clone();
    let approach = next
        .approaches
        .get_mut(approach_id)
        .ok_or_else(missing_approach_after_validation)?;
    approach.gap_strength = gap_strength;
    approach.state = if blocker.class == PortfolioBlockerClass::Contradiction {
        ApproachState::Falsified
    } else {
        ApproachState::Blocked
    };
    if let Some(evidence_ref) = &blocker.evidence_ref {
        if !approach.evidence_refs.contains(evidence_ref) {
            approach.evidence_refs.push(evidence_ref.clone());
        }
    }
    approach.blocker = Some(blocker);
    debug_assert_eq!(approach.gap_strength, gap_strength);
    debug_assert!(approach.blocker.is_some());
    Ok(next)
}

/// Reopens a blocked route only when supplied metadata is genuinely novel.
///
/// # Errors
///
/// Returns a typed error for phase, route, unsafe metadata, duplicate mechanism,
/// non-simpler gap, or missing novelty.
pub fn reopen_approach(
    state: &PortfolioState,
    approach_id: &str,
    novelty: &ReopenNovelty,
) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(state, &[PortfolioPhase::TargetedRedirection])?;
    require_safe_text(approach_id, FIELD_APPROACH)?;
    require_safe_text(&novelty.next_check, FIELD_APPROACH)?;
    if novelty.gap_strength != GapStrength::Simpler {
        return Err(error(
            PortfolioErrorKind::GapNotSimpler,
            FIELD_APPROACH,
            "reopened routes must establish a materially simpler remaining gap",
        ));
    }
    let Some(current) = state.approaches.get(approach_id) else {
        return Err(error(
            PortfolioErrorKind::MissingApproach,
            FIELD_APPROACH,
            "cannot reopen an unknown approach",
        ));
    };
    if current.state != ApproachState::Blocked {
        return Err(error(
            PortfolioErrorKind::InvalidApproach,
            FIELD_APPROACH,
            "only a blocked approach can be reopened",
        ));
    }
    let (is_new_mechanism, is_new_evidence) =
        validate_reopen_novelty(state, approach_id, current, novelty)?;
    let mut next = state.clone();
    let approach = next
        .approaches
        .get_mut(approach_id)
        .ok_or_else(missing_approach_after_validation)?;
    if is_new_mechanism {
        approach.mechanism = novelty
            .mechanism
            .clone()
            .ok_or_else(missing_novelty_after_validation)?;
    }
    if is_new_evidence {
        approach.evidence_refs.push(
            novelty
                .evidence_ref
                .clone()
                .ok_or_else(missing_novelty_after_validation)?,
        );
    }
    approach.gap_strength = GapStrength::Simpler;
    approach.state = ApproachState::Active;
    approach.blocker = None;
    approach.next_check.clone_from(&novelty.next_check);
    debug_assert_eq!(approach.state, ApproachState::Active);
    debug_assert!(approach.blocker.is_none());
    Ok(next)
}

fn validate_reopen_novelty(
    state: &PortfolioState,
    approach_id: &str,
    current: &ApproachRecord,
    novelty: &ReopenNovelty,
) -> PortfolioResult<(bool, bool)> {
    let is_new_mechanism = if let Some(mechanism) = &novelty.mechanism {
        validate_fingerprint(mechanism)?;
        if mechanism == &current.mechanism {
            false
        } else {
            if let Some(existing) = find_mechanism_owner(state, mechanism, Some(approach_id)) {
                return Err(error(
                    PortfolioErrorKind::DuplicateMechanism,
                    FIELD_APPROACH,
                    format!("replacement mechanism already belongs to approach `{existing}`"),
                ));
            }
            true
        }
    } else {
        false
    };
    let is_new_evidence = if let Some(evidence_ref) = &novelty.evidence_ref {
        require_safe_text(evidence_ref, FIELD_APPROACH)?;
        let is_repeated_blocker_evidence = current
            .blocker
            .as_ref()
            .and_then(|blocker| blocker.evidence_ref.as_ref())
            == Some(evidence_ref);
        !current.evidence_refs.contains(evidence_ref) && !is_repeated_blocker_evidence
    } else {
        false
    };
    if !is_new_mechanism && !is_new_evidence {
        return Err(error(
            PortfolioErrorKind::MissingNovelty,
            FIELD_APPROACH,
            "blocked route requires a new mechanism or evidence reference",
        ));
    }
    debug_assert_ne!((is_new_mechanism, is_new_evidence), (false, false));
    debug_assert_eq!(novelty.gap_strength, GapStrength::Simpler);
    Ok((is_new_mechanism, is_new_evidence))
}

/// Selects one simpler-gap route for adversarial audit.
///
/// # Errors
///
/// Returns a typed error for phase, missing route, ineligible state or gap, or
/// absent route evidence.
pub fn begin_adversarial_audit(
    state: &PortfolioState,
    approach_id: &str,
) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(
        state,
        &[
            PortfolioPhase::Synthesis,
            PortfolioPhase::TargetedRedirection,
        ],
    )?;
    require_safe_text(approach_id, FIELD_APPROACH)?;
    let Some(candidate) = state.approaches.get(approach_id) else {
        return Err(error(
            PortfolioErrorKind::MissingApproach,
            FIELD_APPROACH,
            "cannot audit an unknown approach",
        ));
    };
    if candidate.state != ApproachState::Active {
        return Err(error(
            PortfolioErrorKind::InvalidApproach,
            FIELD_APPROACH,
            "adversarial audit requires an active approach",
        ));
    }
    if candidate.gap_strength != GapStrength::Simpler {
        return Err(error(
            PortfolioErrorKind::GapNotSimpler,
            FIELD_APPROACH,
            "equivalent, stronger, or unknown gaps cannot enter audit",
        ));
    }
    if candidate.evidence_refs.is_empty() {
        return Err(error(
            PortfolioErrorKind::InvalidApproach,
            FIELD_APPROACH,
            "adversarial audit requires concrete route evidence",
        ));
    }
    let mut next = state.clone();
    let selected = next
        .approaches
        .get_mut(approach_id)
        .ok_or_else(missing_approach_after_validation)?;
    selected.state = ApproachState::Audit;
    next.phase = PortfolioPhase::AdversarialAudit;
    next.selected_approach = Some(approach_id.to_string());
    next.audit = None;
    debug_assert_eq!(next.phase, PortfolioPhase::AdversarialAudit);
    debug_assert_eq!(next.selected_approach.as_deref(), Some(approach_id));
    Ok(next)
}

/// Records a complete bounded adversarial audit packet.
///
/// # Errors
///
/// Returns a typed error for phase, candidate mismatch, missing risk classes,
/// key/finding mismatch, or unsafe evidence metadata.
pub fn record_adversarial_audit(
    state: &PortfolioState,
    audit: &PortfolioAudit,
) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(state, &[PortfolioPhase::AdversarialAudit])?;
    require_safe_text(&audit.approach_id, FIELD_AUDIT)?;
    if state.selected_approach.as_deref() != Some(audit.approach_id.as_str()) {
        return Err(error(
            PortfolioErrorKind::AuditIncomplete,
            FIELD_AUDIT,
            "audit approach does not match the selected candidate",
        ));
    }
    if audit.findings.len() > MAX_REFERENCE_COUNT {
        return Err(error(
            PortfolioErrorKind::AuditIncomplete,
            FIELD_AUDIT,
            "audit exceeds the bounded finding count",
        ));
    }
    for required in &state.contract.required_audit_risks {
        if !audit.findings.contains_key(required) {
            return Err(error(
                PortfolioErrorKind::AuditIncomplete,
                FIELD_AUDIT,
                format!(
                    "audit is missing required risk `{}`",
                    audit_risk_label(required)
                ),
            ));
        }
    }
    for (risk, finding) in &audit.findings {
        validate_audit_risk(risk)?;
        if risk != &finding.risk {
            return Err(error(
                PortfolioErrorKind::AuditIncomplete,
                FIELD_AUDIT,
                "audit map key does not match its finding risk",
            ));
        }
        validate_audit_finding(finding)?;
    }
    let mut next = state.clone();
    next.audit = Some(audit.clone());
    debug_assert_eq!(next.phase, PortfolioPhase::AdversarialAudit);
    debug_assert_eq!(next.audit.as_ref(), Some(audit));
    Ok(next)
}

/// Enters validation only after every required adversarial risk passes.
///
/// # Errors
///
/// Returns a typed error for phase, missing audit, missing finding, or any failed
/// or unresolved required finding.
pub fn begin_validation(state: &PortfolioState) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(state, &[PortfolioPhase::AdversarialAudit])?;
    let Some(audit) = &state.audit else {
        return Err(error(
            PortfolioErrorKind::AuditIncomplete,
            FIELD_AUDIT,
            "validation requires a recorded adversarial audit",
        ));
    };
    for required in &state.contract.required_audit_risks {
        let Some(finding) = audit.findings.get(required) else {
            return Err(error(
                PortfolioErrorKind::AuditIncomplete,
                FIELD_AUDIT,
                format!(
                    "validation is missing required risk `{}`",
                    audit_risk_label(required)
                ),
            ));
        };
        if finding.outcome != AuditOutcome::Pass {
            return Err(error(
                PortfolioErrorKind::AuditFailed,
                FIELD_AUDIT,
                format!(
                    "required audit risk `{}` did not pass",
                    audit_risk_label(required)
                ),
            ));
        }
    }
    let mut next = state.clone();
    next.phase = PortfolioPhase::Validation;
    debug_assert_eq!(next.phase, PortfolioPhase::Validation);
    debug_assert!(next.audit.is_some());
    Ok(next)
}

/// Applies a matching external pass receipt to one audited bounded result.
///
/// # Errors
///
/// Returns a typed error for phase, unsafe or malformed receipt metadata,
/// non-pass status, stale contract/route/evidence identity, or ineligible route.
pub fn validate_portfolio_result(
    state: &PortfolioState,
    receipt: &PortfolioValidationReceipt,
) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    require_phase(state, &[PortfolioPhase::Validation])?;
    validate_fingerprint(&receipt.contract)?;
    validate_fingerprint(&receipt.mechanism)?;
    validate_fingerprint(&receipt.evidence)?;
    validate_fingerprint(&receipt.validator_policy)?;
    require_safe_text(&receipt.approach_id, FIELD_RECEIPT)?;
    if receipt.status != PortfolioValidationStatus::Passed {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "portfolio validation requires an explicit pass status",
        ));
    }
    let expected_contract = portfolio_contract_fingerprint(&state.contract);
    if receipt.contract != expected_contract {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "validation receipt contract identity is stale or mismatched",
        ));
    }
    if receipt.validator_policy != state.contract.validator_policy {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "validation receipt policy is not admitted by the portfolio contract",
        ));
    }
    if state.selected_approach.as_deref() != Some(receipt.approach_id.as_str()) {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "validation receipt approach does not match the selected candidate",
        ));
    }
    let Some(candidate) = state.approaches.get(&receipt.approach_id) else {
        return Err(error(
            PortfolioErrorKind::MissingApproach,
            FIELD_RECEIPT,
            "validation receipt references an unknown approach",
        ));
    };
    let Some(audit) = &state.audit else {
        return Err(error(
            PortfolioErrorKind::AuditIncomplete,
            FIELD_AUDIT,
            "validation state is missing its adversarial audit",
        ));
    };
    validate_audited_candidate_receipt(candidate, audit, receipt)?;
    let mut next = state.clone();
    let selected = next
        .approaches
        .get_mut(&receipt.approach_id)
        .ok_or_else(missing_approach_after_validation)?;
    selected.state = ApproachState::Validated;
    next.phase = PortfolioPhase::Validated;
    next.validation_receipt = Some(receipt.clone());
    debug_assert_eq!(next.phase, PortfolioPhase::Validated);
    debug_assert_eq!(next.validation_receipt.as_ref(), Some(receipt));
    Ok(next)
}

fn validate_audited_candidate_receipt(
    candidate: &ApproachRecord,
    audit: &PortfolioAudit,
    receipt: &PortfolioValidationReceipt,
) -> PortfolioResult<()> {
    let expected_evidence = portfolio_evidence_fingerprint(candidate, audit);
    let is_matching_candidate = [
        candidate.state == ApproachState::Audit,
        candidate.gap_strength == GapStrength::Simpler,
        receipt.mechanism == candidate.mechanism,
        receipt.evidence == expected_evidence,
    ]
    .into_iter()
    .all(|is_match| is_match);
    if !is_matching_candidate {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "validation receipt does not match audited route and evidence identity",
        ));
    }
    Ok(())
}

/// Returns whether any explicit portfolio limit has been consumed.
#[must_use]
pub fn is_portfolio_budget_exhausted(state: &PortfolioState) -> bool {
    state.usage.rounds >= state.contract.budget.rounds
        || state.usage.approaches >= state.contract.budget.approach_capacity
        || state.usage.tool_calls >= state.contract.budget.tool_calls
        || state.usage.tokens >= state.contract.budget.tokens
}

/// Ends a portfolio honestly when budget or viable routes are exhausted.
///
/// # Errors
///
/// Returns a typed error for invalid state, an existing terminal phase, or a
/// premature exhaustion request while budget and live routes remain.
pub fn exhaust_portfolio(state: &PortfolioState) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    reject_terminal(state)?;
    let is_every_route_terminal = !state.approaches.is_empty()
        && state
            .approaches
            .values()
            .all(|approach| is_terminal_approach(approach.state));
    if !is_portfolio_budget_exhausted(state) && !is_every_route_terminal {
        return Err(error(
            PortfolioErrorKind::BudgetInvalid,
            FIELD_BUDGET,
            "portfolio cannot exhaust while budget and live routes remain",
        ));
    }
    let mut next = state.clone();
    next.phase = PortfolioPhase::Exhausted;
    next.terminal_blocker = None;
    next.validation_receipt = None;
    debug_assert_eq!(next.phase, PortfolioPhase::Exhausted);
    debug_assert!(next.validation_receipt.is_none());
    Ok(next)
}

/// Ends a portfolio with an exact user, authority, policy, or tool blocker.
///
/// # Errors
///
/// Returns a typed error for invalid state, terminal state, unsafe metadata, or
/// an approach-only blocker class.
pub fn block_portfolio(
    state: &PortfolioState,
    blocker: PortfolioBlocker,
) -> PortfolioResult<PortfolioState> {
    validate_state(state)?;
    reject_terminal(state)?;
    validate_blocker(&blocker)?;
    if !matches!(
        blocker.class,
        PortfolioBlockerClass::AuthorityDenied
            | PortfolioBlockerClass::PolicyDenied
            | PortfolioBlockerClass::ToolUnavailable
            | PortfolioBlockerClass::UserDecisionRequired
    ) {
        return Err(error(
            PortfolioErrorKind::BlockerMismatch,
            FIELD_BLOCKER,
            "terminal portfolio blocker must name user, authority, policy, or tool state",
        ));
    }
    let mut next = state.clone();
    next.phase = PortfolioPhase::Blocked;
    next.terminal_blocker = Some(blocker);
    next.validation_receipt = None;
    debug_assert_eq!(next.phase, PortfolioPhase::Blocked);
    debug_assert!(next.terminal_blocker.is_some());
    Ok(next)
}

/// Projects a bounded terminal receipt without prompts or raw tool/provider data.
///
/// # Errors
///
/// Returns a typed state or receipt validation error when the portfolio is not
/// terminal or its projected fields violate receipt invariants.
pub fn project_portfolio_receipt(state: &PortfolioState) -> PortfolioResult<PortfolioReceipt> {
    validate_state(state)?;
    if !state.phase.is_terminal() {
        return Err(error(
            PortfolioErrorKind::TerminalState,
            FIELD_PHASE,
            "portfolio receipts require a terminal phase",
        ));
    }
    let selected = state
        .selected_approach
        .as_ref()
        .and_then(|approach_id| state.approaches.get(approach_id));
    let contract = portfolio_contract_fingerprint(&state.contract);
    let selected_approach = selected.map(|approach| approach.approach_id.clone());
    let selected_mechanism = selected.map(|approach| approach.mechanism.clone());
    let evidence_refs = selected
        .map(|approach| approach.evidence_refs.clone())
        .unwrap_or_default();
    let blocker = if state.phase == PortfolioPhase::Blocked {
        state.terminal_blocker.clone()
    } else {
        None
    };
    let receipt_id = portfolio_receipt_fingerprint(&PortfolioReceiptFingerprintInput {
        contract: &contract,
        phase: state.phase,
        usage: &state.usage,
        selected_approach: selected_approach.as_deref(),
        selected_mechanism: selected_mechanism.as_ref(),
        evidence_refs: &evidence_refs,
        blocker: blocker.as_ref(),
    });
    let receipt = PortfolioReceipt {
        receipt_id,
        contract,
        phase: state.phase,
        usage: state.usage.clone(),
        selected_approach,
        selected_mechanism,
        evidence_refs,
        blocker,
    };
    validate_portfolio_receipt(&receipt)?;
    debug_assert!(receipt.phase.is_terminal());
    debug_assert_eq!(receipt.receipt_id.algorithm, BLAKE3_ALGORITHM);
    Ok(receipt)
}

/// Validates a redacted terminal portfolio receipt.
///
/// # Errors
///
/// Returns a typed error for nonterminal status, malformed hashes, unsafe
/// references, inconsistent terminal fields, or receipt-identity mismatch.
pub fn validate_portfolio_receipt(receipt: &PortfolioReceipt) -> PortfolioResult<()> {
    validate_fingerprint(&receipt.receipt_id)?;
    validate_fingerprint(&receipt.contract)?;
    if !receipt.phase.is_terminal() {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "portfolio receipt phase is not terminal",
        ));
    }
    if let Some(approach_id) = &receipt.selected_approach {
        require_safe_text(approach_id, FIELD_RECEIPT)?;
    }
    if let Some(mechanism) = &receipt.selected_mechanism {
        validate_fingerprint(mechanism)?;
    }
    require_safe_text_collection(&receipt.evidence_refs, FIELD_RECEIPT, false)?;
    if receipt.selected_approach.is_some() != receipt.selected_mechanism.is_some() {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "selected approach and mechanism must be present together",
        ));
    }
    validate_receipt_terminal_fields(receipt)?;
    let expected = portfolio_receipt_fingerprint(&PortfolioReceiptFingerprintInput {
        contract: &receipt.contract,
        phase: receipt.phase,
        usage: &receipt.usage,
        selected_approach: receipt.selected_approach.as_deref(),
        selected_mechanism: receipt.selected_mechanism.as_ref(),
        evidence_refs: &receipt.evidence_refs,
        blocker: receipt.blocker.as_ref(),
    });
    if receipt.receipt_id != expected {
        return Err(error(
            PortfolioErrorKind::ReceiptMismatch,
            FIELD_RECEIPT,
            "portfolio receipt identity does not match its fields",
        ));
    }
    debug_assert!(receipt.phase.is_terminal());
    debug_assert_eq!(receipt.receipt_id, expected);
    Ok(())
}

fn validate_receipt_terminal_fields(receipt: &PortfolioReceipt) -> PortfolioResult<()> {
    match receipt.phase {
        PortfolioPhase::Validated => {
            if receipt.selected_approach.is_none() || receipt.blocker.is_some() {
                return Err(error(
                    PortfolioErrorKind::ReceiptMismatch,
                    FIELD_RECEIPT,
                    "validated receipt requires a selected route and no blocker",
                ));
            }
        }
        PortfolioPhase::Blocked => {
            let Some(blocker) = &receipt.blocker else {
                return Err(error(
                    PortfolioErrorKind::ReceiptMismatch,
                    FIELD_RECEIPT,
                    "blocked receipt requires an exact blocker",
                ));
            };
            validate_blocker(blocker)?;
        }
        PortfolioPhase::Exhausted => {
            if receipt.blocker.is_some() {
                return Err(error(
                    PortfolioErrorKind::ReceiptMismatch,
                    FIELD_RECEIPT,
                    "exhausted receipt cannot fabricate a terminal blocker",
                ));
            }
        }
        PortfolioPhase::Framing
        | PortfolioPhase::IndependentExploration
        | PortfolioPhase::Synthesis
        | PortfolioPhase::TargetedRedirection
        | PortfolioPhase::AdversarialAudit
        | PortfolioPhase::Validation => {
            return Err(error(
                PortfolioErrorKind::ReceiptMismatch,
                FIELD_RECEIPT,
                "portfolio receipt phase is not terminal",
            ));
        }
    }
    debug_assert!(receipt.phase.is_terminal());
    debug_assert_eq!(
        receipt.selected_approach.is_some(),
        receipt.selected_mechanism.is_some()
    );
    Ok(())
}

fn validate_state(state: &PortfolioState) -> PortfolioResult<()> {
    validate_portfolio_contract(&state.contract)?;
    validate_state_budget(state)?;
    validate_state_registry(state)?;
    validate_state_phase_fields(state)?;
    Ok(())
}

fn validate_state_budget(state: &PortfolioState) -> PortfolioResult<()> {
    let is_within_budget = [
        state.usage.rounds <= state.contract.budget.rounds,
        state.usage.approaches <= state.contract.budget.approach_capacity,
        state.usage.tool_calls <= state.contract.budget.tool_calls,
        state.usage.tokens <= state.contract.budget.tokens,
    ]
    .into_iter()
    .all(|is_within_limit| is_within_limit);
    if !is_within_budget {
        return Err(error(
            PortfolioErrorKind::BudgetExceeded,
            FIELD_BUDGET,
            "portfolio state exceeds its contract budget",
        ));
    }
    debug_assert!(is_within_budget);
    debug_assert!(state.usage.rounds <= state.contract.budget.rounds);
    Ok(())
}

fn validate_state_registry(state: &PortfolioState) -> PortfolioResult<()> {
    let registry_count = u32::try_from(state.approaches.len()).map_err(|_| {
        error(
            PortfolioErrorKind::BudgetExceeded,
            FIELD_APPROACH,
            "approach registry cannot fit the bounded counter representation",
        )
    })?;
    if state.usage.approaches != registry_count {
        return Err(error(
            PortfolioErrorKind::InvalidApproach,
            FIELD_APPROACH,
            "approach counter does not match registry size",
        ));
    }
    let mut mechanisms = BTreeSet::new();
    for (approach_id, approach) in &state.approaches {
        if approach_id != &approach.approach_id {
            return Err(error(
                PortfolioErrorKind::InvalidApproach,
                FIELD_APPROACH,
                "approach registry key does not match record identity",
            ));
        }
        validate_approach_record(approach)?;
        if !mechanisms.insert(approach.mechanism.clone()) {
            return Err(error(
                PortfolioErrorKind::DuplicateMechanism,
                FIELD_APPROACH,
                "portfolio state contains a duplicate mechanism",
            ));
        }
    }
    if let Some(selected) = &state.selected_approach {
        require_safe_text(selected, FIELD_APPROACH)?;
        if !state.approaches.contains_key(selected) {
            return Err(error(
                PortfolioErrorKind::MissingApproach,
                FIELD_APPROACH,
                "selected approach is absent from the registry",
            ));
        }
    }
    if let Some(audit) = &state.audit {
        if state.selected_approach.as_deref() != Some(audit.approach_id.as_str()) {
            return Err(error(
                PortfolioErrorKind::AuditIncomplete,
                FIELD_AUDIT,
                "recorded audit does not match selected approach",
            ));
        }
    }
    debug_assert_eq!(state.usage.approaches, registry_count);
    debug_assert!(
        state
            .approaches
            .iter()
            .all(|(approach_id, approach)| approach_id == &approach.approach_id)
    );
    Ok(())
}

fn validate_state_phase_fields(state: &PortfolioState) -> PortfolioResult<()> {
    match state.phase {
        PortfolioPhase::Validated => {
            if state.selected_approach.is_none() || state.validation_receipt.is_none() {
                return Err(error(
                    PortfolioErrorKind::ReceiptMismatch,
                    FIELD_RECEIPT,
                    "validated state requires a selected route and pass receipt",
                ));
            }
        }
        PortfolioPhase::Blocked => {
            let Some(blocker) = &state.terminal_blocker else {
                return Err(error(
                    PortfolioErrorKind::BlockerMismatch,
                    FIELD_BLOCKER,
                    "blocked state requires an exact terminal blocker",
                ));
            };
            validate_blocker(blocker)?;
        }
        PortfolioPhase::AdversarialAudit | PortfolioPhase::Validation => {
            if state.selected_approach.is_none() {
                return Err(error(
                    PortfolioErrorKind::MissingApproach,
                    FIELD_APPROACH,
                    "audit and validation phases require a selected route",
                ));
            }
        }
        PortfolioPhase::Framing
        | PortfolioPhase::IndependentExploration
        | PortfolioPhase::Synthesis
        | PortfolioPhase::TargetedRedirection
        | PortfolioPhase::Exhausted => {}
    }
    debug_assert!(has_required_phase_selection(state));
    debug_assert!(has_required_terminal_fields(state));
    Ok(())
}

fn has_required_phase_selection(state: &PortfolioState) -> bool {
    match state.phase {
        PortfolioPhase::AdversarialAudit | PortfolioPhase::Validation => {
            state.selected_approach.is_some()
        }
        PortfolioPhase::Framing
        | PortfolioPhase::IndependentExploration
        | PortfolioPhase::Synthesis
        | PortfolioPhase::TargetedRedirection
        | PortfolioPhase::Validated
        | PortfolioPhase::Exhausted
        | PortfolioPhase::Blocked => true,
    }
}

fn has_required_terminal_fields(state: &PortfolioState) -> bool {
    match state.phase {
        PortfolioPhase::Validated => {
            state.selected_approach.is_some() && state.validation_receipt.is_some()
        }
        PortfolioPhase::Blocked => state.terminal_blocker.is_some(),
        PortfolioPhase::Framing
        | PortfolioPhase::IndependentExploration
        | PortfolioPhase::Synthesis
        | PortfolioPhase::TargetedRedirection
        | PortfolioPhase::AdversarialAudit
        | PortfolioPhase::Validation
        | PortfolioPhase::Exhausted => true,
    }
}

fn validate_approach_blocker_invariant(approach: &ApproachRecord) -> PortfolioResult<()> {
    let is_blocker_required = matches!(
        approach.state,
        ApproachState::Blocked | ApproachState::Falsified
    );
    if is_blocker_required != approach.blocker.is_some() {
        return Err(error(
            PortfolioErrorKind::InvalidApproach,
            FIELD_BLOCKER,
            "blocked and falsified routes require exactly one blocker",
        ));
    }
    if let Some(blocker) = &approach.blocker {
        validate_blocker(blocker)?;
    }
    match approach.gap_strength {
        GapStrength::Simpler => {
            if let Some(blocker) = &approach.blocker {
                if matches!(
                    blocker.class,
                    PortfolioBlockerClass::EquivalentStrength
                        | PortfolioBlockerClass::StrongerThanObjective
                        | PortfolioBlockerClass::UnknownGapStrength
                ) {
                    return Err(error(
                        PortfolioErrorKind::BlockerMismatch,
                        FIELD_BLOCKER,
                        "simpler route cannot use a non-simpler gap blocker",
                    ));
                }
            }
        }
        GapStrength::Equivalent => {
            require_exact_gap_blocker(approach, PortfolioBlockerClass::EquivalentStrength)?;
        }
        GapStrength::Stronger => {
            require_exact_gap_blocker(approach, PortfolioBlockerClass::StrongerThanObjective)?;
        }
        GapStrength::Unknown => {
            require_exact_gap_blocker(approach, PortfolioBlockerClass::UnknownGapStrength)?;
        }
    }
    if approach.state == ApproachState::Falsified {
        let blocker_class = approach.blocker.as_ref().map(|blocker| blocker.class);
        if blocker_class != Some(PortfolioBlockerClass::Contradiction) {
            return Err(error(
                PortfolioErrorKind::BlockerMismatch,
                FIELD_BLOCKER,
                "falsified route requires a contradiction blocker",
            ));
        }
        debug_assert_eq!(blocker_class, Some(PortfolioBlockerClass::Contradiction));
    }
    debug_assert_eq!(is_blocker_required, approach.blocker.is_some());
    Ok(())
}

fn require_exact_gap_blocker(
    approach: &ApproachRecord,
    expected: PortfolioBlockerClass,
) -> PortfolioResult<()> {
    if approach.state != ApproachState::Blocked
        || approach.blocker.as_ref().map(|blocker| blocker.class) != Some(expected)
    {
        return Err(error(
            PortfolioErrorKind::BlockerMismatch,
            FIELD_BLOCKER,
            "equivalent, stronger, and unknown gaps require their exact blocker class",
        ));
    }
    Ok(())
}

fn validate_gap_blocker(
    gap_strength: GapStrength,
    blocker_class: PortfolioBlockerClass,
) -> PortfolioResult<()> {
    if !is_gap_blocker_matching(gap_strength, blocker_class) {
        return Err(error(
            PortfolioErrorKind::BlockerMismatch,
            FIELD_BLOCKER,
            "blocker class does not match remaining-gap strength",
        ));
    }
    Ok(())
}

fn is_gap_blocker_matching(
    gap_strength: GapStrength,
    blocker_class: PortfolioBlockerClass,
) -> bool {
    match gap_strength {
        GapStrength::Simpler => !matches!(
            blocker_class,
            PortfolioBlockerClass::EquivalentStrength
                | PortfolioBlockerClass::StrongerThanObjective
                | PortfolioBlockerClass::UnknownGapStrength
        ),
        GapStrength::Equivalent => blocker_class == PortfolioBlockerClass::EquivalentStrength,
        GapStrength::Stronger => blocker_class == PortfolioBlockerClass::StrongerThanObjective,
        GapStrength::Unknown => blocker_class == PortfolioBlockerClass::UnknownGapStrength,
    }
}

fn validate_blocker(blocker: &PortfolioBlocker) -> PortfolioResult<()> {
    require_safe_text(&blocker.detail, FIELD_BLOCKER)?;
    if let Some(evidence_ref) = &blocker.evidence_ref {
        require_safe_text(evidence_ref, FIELD_BLOCKER)?;
    }
    Ok(())
}

fn validate_audit_finding(finding: &PortfolioAuditFinding) -> PortfolioResult<()> {
    validate_audit_risk(&finding.risk)?;
    require_safe_text(&finding.evidence_ref, FIELD_AUDIT)?;
    require_safe_text(&finding.residual_uncertainty, FIELD_AUDIT)?;
    Ok(())
}

fn validate_audit_risk(risk: &PortfolioAuditRisk) -> PortfolioResult<()> {
    if let PortfolioAuditRisk::Named(label) = risk {
        require_safe_text(label, FIELD_AUDIT)?;
    }
    Ok(())
}

fn validate_fingerprint(fingerprint: &PortfolioFingerprint) -> PortfolioResult<()> {
    let is_valid_hex = fingerprint.value_hex.len() == BLAKE3_HEX_LENGTH
        && fingerprint
            .value_hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if fingerprint.algorithm != BLAKE3_ALGORITHM || !is_valid_hex {
        return Err(error(
            PortfolioErrorKind::InvalidFingerprint,
            FIELD_FINGERPRINT,
            "portfolio fingerprint must be lowercase BLAKE3 hex",
        ));
    }
    Ok(())
}

fn require_safe_text(text: &str, field: ErrorField) -> PortfolioResult<()> {
    let character_count_chars = text.chars().count();
    if text.trim().is_empty()
        || character_count_chars > MAX_SAFE_LABEL_CHARS
        || text.chars().any(char::is_control)
    {
        return Err(error(
            PortfolioErrorKind::UnsafeMetadata,
            field,
            "portfolio metadata must be non-empty, bounded, and control-free",
        ));
    }
    let lowercase = text.to_ascii_lowercase();
    let has_raw_marker = lowercase.contains(RAW_HOME_PATH_MARKER)
        || lowercase.contains(RAW_USER_PATH_MARKER)
        || lowercase.contains(RAW_PROMPT_MARKER)
        || lowercase.contains(RAW_TOOL_OUTPUT_MARKER)
        || lowercase.contains(RAW_PROVIDER_PAYLOAD_MARKER)
        || lowercase.contains(RAW_SECRET_MARKER)
        || lowercase.contains(RAW_SECRET_ASSIGNMENT_MARKER);
    if has_raw_marker {
        return Err(error(
            PortfolioErrorKind::UnsafeMetadata,
            field,
            "portfolio metadata contains a raw-material or secret marker",
        ));
    }
    debug_assert_ne!(text.trim(), "");
    debug_assert!(character_count_chars <= MAX_SAFE_LABEL_CHARS);
    Ok(())
}

fn require_safe_text_collection(
    values: &[String],
    field: ErrorField,
    require_non_empty: bool,
) -> PortfolioResult<()> {
    if values.len() > MAX_REFERENCE_COUNT || (require_non_empty && values.is_empty()) {
        return Err(error(
            PortfolioErrorKind::UnsafeMetadata,
            field,
            "portfolio metadata collection is empty when required or exceeds its bound",
        ));
    }
    let mut unique = BTreeSet::new();
    for value in values {
        require_safe_text(value, field)?;
        if !unique.insert(value) {
            return Err(error(
                PortfolioErrorKind::UnsafeMetadata,
                field,
                "portfolio metadata collection contains a duplicate value",
            ));
        }
    }
    debug_assert!(values.len() <= MAX_REFERENCE_COUNT);
    debug_assert_eq!(unique.len(), values.len());
    Ok(())
}

fn require_phase(state: &PortfolioState, allowed: &[PortfolioPhase]) -> PortfolioResult<()> {
    if state.phase.is_terminal() {
        return Err(error(
            PortfolioErrorKind::TerminalState,
            FIELD_PHASE,
            "terminal portfolio state rejects further transitions",
        ));
    }
    if !allowed.contains(&state.phase) {
        return Err(error(
            PortfolioErrorKind::IllegalPhase,
            FIELD_PHASE,
            "portfolio transition is not admitted from the current phase",
        ));
    }
    Ok(())
}

fn reject_terminal(state: &PortfolioState) -> PortfolioResult<()> {
    if state.phase.is_terminal() {
        return Err(error(
            PortfolioErrorKind::TerminalState,
            FIELD_PHASE,
            "terminal portfolio state rejects further transitions",
        ));
    }
    Ok(())
}

fn find_mechanism_owner<'a>(
    state: &'a PortfolioState,
    mechanism: &PortfolioFingerprint,
    except_approach: Option<&str>,
) -> Option<&'a str> {
    state.approaches.values().find_map(|approach| {
        let is_excepted = except_approach == Some(approach.approach_id.as_str());
        (!is_excepted && &approach.mechanism == mechanism).then_some(approach.approach_id.as_str())
    })
}

const fn is_registration_state_admitted(
    phase: PortfolioPhase,
    approach_state: ApproachState,
) -> bool {
    match phase {
        PortfolioPhase::IndependentExploration => matches!(
            approach_state,
            ApproachState::Independent | ApproachState::Blocked
        ),
        PortfolioPhase::TargetedRedirection => matches!(
            approach_state,
            ApproachState::Active | ApproachState::Blocked
        ),
        PortfolioPhase::Framing
        | PortfolioPhase::Synthesis
        | PortfolioPhase::AdversarialAudit
        | PortfolioPhase::Validation
        | PortfolioPhase::Validated
        | PortfolioPhase::Exhausted
        | PortfolioPhase::Blocked => false,
    }
}

const fn is_live_approach(state: ApproachState) -> bool {
    matches!(
        state,
        ApproachState::Independent | ApproachState::Active | ApproachState::Audit
    )
}

const fn is_terminal_approach(state: ApproachState) -> bool {
    matches!(
        state,
        ApproachState::Blocked | ApproachState::Falsified | ApproachState::Validated
    )
}

fn checked_add_u32(addends: CheckedAdd<u32>, field: ErrorField) -> PortfolioResult<u32> {
    addends
        .current
        .checked_add(addends.increment)
        .ok_or_else(|| {
            error(
                PortfolioErrorKind::BudgetExceeded,
                field,
                "portfolio counter overflowed its integer representation",
            )
        })
}

fn checked_add_u64(addends: CheckedAdd<u64>, field: ErrorField) -> PortfolioResult<u64> {
    addends
        .current
        .checked_add(addends.increment)
        .ok_or_else(|| {
            error(
                PortfolioErrorKind::BudgetExceeded,
                field,
                "portfolio counter overflowed its integer representation",
            )
        })
}

fn missing_approach_after_validation() -> PortfolioError {
    error(
        PortfolioErrorKind::MissingApproach,
        FIELD_APPROACH,
        "approach disappeared after deterministic state validation",
    )
}

fn missing_novelty_after_validation() -> PortfolioError {
    error(
        PortfolioErrorKind::MissingNovelty,
        FIELD_APPROACH,
        "novelty disappeared after deterministic validation",
    )
}

fn error(kind: PortfolioErrorKind, field: ErrorField, detail: impl Into<String>) -> PortfolioError {
    PortfolioError::new(kind, field.0, detail)
}

fn portfolio_hasher(label: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    hash_part(&mut hasher, PORTFOLIO_HASH_DOMAIN.as_bytes());
    hash_part(&mut hasher, label.as_bytes());
    hasher
}

fn hash_part(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length_bytes = bytes.len().to_string();
    hasher.update(length_bytes.as_bytes());
    hasher.update(b":");
    hasher.update(bytes);
}

fn hash_fingerprint(hasher: &mut blake3::Hasher, fingerprint: &PortfolioFingerprint) {
    hash_part(hasher, fingerprint.algorithm.as_bytes());
    hash_part(hasher, fingerprint.value_hex.as_bytes());
}

fn hash_text_collection(hasher: &mut blake3::Hasher, values: &[String]) {
    hash_part(hasher, values.len().to_string().as_bytes());
    for value in values {
        hash_part(hasher, value.as_bytes());
    }
}

fn hash_audit_risk(hasher: &mut blake3::Hasher, risk: &PortfolioAuditRisk) {
    hash_part(hasher, audit_risk_label(risk).as_bytes());
    if let PortfolioAuditRisk::Named(label) = risk {
        hash_part(hasher, label.as_bytes());
    }
}

fn audit_risk_label(risk: &PortfolioAuditRisk) -> &'static str {
    match risk {
        PortfolioAuditRisk::Preconditions => "preconditions",
        PortfolioAuditRisk::Vacuity => "vacuity",
        PortfolioAuditRisk::QuantifierDomain => "quantifier-domain",
        PortfolioAuditRisk::ArithmeticBounds => "arithmetic-bounds",
        PortfolioAuditRisk::IndexBounds => "index-bounds",
        PortfolioAuditRisk::Decreases => "decreases",
        PortfolioAuditRisk::ExecSpecCorrespondence => "exec-spec-correspondence",
        PortfolioAuditRisk::TrustBoundary => "trust-boundary",
        PortfolioAuditRisk::FailureModes => "failure-modes",
        PortfolioAuditRisk::EvidenceLinkage => "evidence-linkage",
        PortfolioAuditRisk::Named(_) => "named",
    }
}

const fn audit_outcome_label(outcome: AuditOutcome) -> &'static str {
    match outcome {
        AuditOutcome::Pass => "pass",
        AuditOutcome::Fail => "fail",
        AuditOutcome::Unresolved => "unresolved",
    }
}

const fn phase_label(phase: PortfolioPhase) -> &'static str {
    match phase {
        PortfolioPhase::Framing => "framing",
        PortfolioPhase::IndependentExploration => "independent-exploration",
        PortfolioPhase::Synthesis => "synthesis",
        PortfolioPhase::TargetedRedirection => "targeted-redirection",
        PortfolioPhase::AdversarialAudit => "adversarial-audit",
        PortfolioPhase::Validation => "validation",
        PortfolioPhase::Validated => "validated",
        PortfolioPhase::Exhausted => "exhausted",
        PortfolioPhase::Blocked => "blocked",
    }
}

const fn blocker_class_label(blocker_class: PortfolioBlockerClass) -> &'static str {
    match blocker_class {
        PortfolioBlockerClass::EquivalentStrength => "equivalent-strength",
        PortfolioBlockerClass::StrongerThanObjective => "stronger-than-objective",
        PortfolioBlockerClass::UnknownGapStrength => "unknown-gap-strength",
        PortfolioBlockerClass::MissingEvidence => "missing-evidence",
        PortfolioBlockerClass::Contradiction => "contradiction",
        PortfolioBlockerClass::AuthorityDenied => "authority-denied",
        PortfolioBlockerClass::PolicyDenied => "policy-denied",
        PortfolioBlockerClass::ToolUnavailable => "tool-unavailable",
        PortfolioBlockerClass::UserDecisionRequired => "user-decision-required",
    }
}

struct PortfolioReceiptFingerprintInput<'a> {
    contract: &'a PortfolioFingerprint,
    phase: PortfolioPhase,
    usage: &'a PortfolioUsage,
    selected_approach: Option<&'a str>,
    selected_mechanism: Option<&'a PortfolioFingerprint>,
    evidence_refs: &'a [String],
    blocker: Option<&'a PortfolioBlocker>,
}

fn portfolio_receipt_fingerprint(
    input: &PortfolioReceiptFingerprintInput<'_>,
) -> PortfolioFingerprint {
    let mut hasher = portfolio_hasher("receipt");
    hash_fingerprint(&mut hasher, input.contract);
    hash_part(&mut hasher, phase_label(input.phase).as_bytes());
    hash_part(&mut hasher, &input.usage.rounds.to_le_bytes());
    hash_part(&mut hasher, &input.usage.approaches.to_le_bytes());
    hash_part(&mut hasher, &input.usage.tool_calls.to_le_bytes());
    hash_part(&mut hasher, &input.usage.tokens.to_le_bytes());
    if let Some(approach_id) = input.selected_approach {
        hash_part(&mut hasher, approach_id.as_bytes());
    } else {
        hash_part(&mut hasher, b"no-selected-approach");
    }
    if let Some(mechanism) = input.selected_mechanism {
        hash_fingerprint(&mut hasher, mechanism);
    } else {
        hash_part(&mut hasher, b"no-selected-mechanism");
    }
    hash_text_collection(&mut hasher, input.evidence_refs);
    if let Some(exact_blocker) = input.blocker {
        hash_part(
            &mut hasher,
            blocker_class_label(exact_blocker.class).as_bytes(),
        );
        hash_part(&mut hasher, exact_blocker.detail.as_bytes());
        if let Some(evidence_ref) = &exact_blocker.evidence_ref {
            hash_part(&mut hasher, evidence_ref.as_bytes());
        }
    } else {
        hash_part(&mut hasher, b"no-terminal-blocker");
    }
    let fingerprint =
        PortfolioFingerprint::new(BLAKE3_ALGORITHM, hasher.finalize().to_hex().to_string());
    debug_assert_eq!(fingerprint.algorithm, BLAKE3_ALGORITHM);
    debug_assert_eq!(fingerprint.value_hex.len(), BLAKE3_HEX_LENGTH);
    fingerprint
}
