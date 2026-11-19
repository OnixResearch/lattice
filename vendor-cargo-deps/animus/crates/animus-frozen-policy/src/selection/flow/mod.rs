//! The decision flow: request shape, artifact slot, binding, input, budget, inference, and
//! selection, then the fallback rule and the receipt.

/// What evaluating one request produced before the fallback rule applies.
pub(crate) struct Attempt {
    pub(crate) policy: Option<PolicyFacts>,
    pub(crate) scores: alloc::vec::Vec<i64>,
    pub(crate) cost: crate::CostUnits,
    pub(crate) selected: Result<(alloc::string::String, i64), super::Failure>,
}

/// The admitted artifact's identities a receipt binds.
pub(crate) struct PolicyFacts {
    pub(crate) artifact: alloc::string::String,
    pub(crate) model: alloc::string::String,
    pub(crate) policy_kind: &'static str,
}

impl Attempt {
    /// An attempt that ended with `failure` before inference.
    fn failed(policy: Option<PolicyFacts>, failure: super::Failure) -> Self {
        Self {
            policy,
            scores: alloc::vec::Vec::new(),
            cost: crate::CostUnits::default(),
            selected: Err(failure),
        }
    }
}

/// Checks the request's own shape and, if it holds, attempts the decision.
pub(super) fn run(
    policy: Result<&crate::AdmittedPolicy, &crate::AdmissionError>,
    request: &super::DecisionRequest,
) -> (Result<(), super::InputFault>, Attempt) {
    let shape = check_shape(request);
    let attempt = match shape {
        Ok(()) => attempt(policy, request),
        Err(fault) => Attempt::failed(None, super::Failure::MalformedInput(fault)),
    };
    debug_assert!(shape.is_ok() || attempt.scores.is_empty());
    debug_assert!(shape.is_ok() || attempt.cost.inference_operations == 0);
    (shape, attempt)
}

/// The request's own shape, independent of any artifact.
fn check_shape(request: &super::DecisionRequest) -> Result<(), super::InputFault> {
    let binding = &request.binding;
    let is_bound = [
        &binding.artifact,
        &binding.feature_schema,
        &binding.action_schema,
    ]
    .iter()
    .all(|identity| crate::digest::is_safe_identity(identity));
    if !is_bound {
        return Err(super::InputFault::Binding);
    }
    if request.features.len() > crate::MAX_FEATURES {
        return Err(super::InputFault::FeatureCount);
    }
    if request.eligible.is_empty() {
        return Err(super::InputFault::NoEligibleAction);
    }
    if request.eligible.len() > crate::MAX_ACTIONS {
        return Err(super::InputFault::TooManyActions);
    }
    let labels = request.eligible.iter().map(alloc::string::String::as_str);
    if !crate::digest::is_unique_label_set(labels) {
        return Err(super::InputFault::DuplicateAction);
    }
    if request
        .fallback
        .as_ref()
        .is_some_and(|fallback| !request.eligible.contains(fallback))
    {
        return Err(super::InputFault::FallbackNotEligible);
    }
    debug_assert!(!request.eligible.is_empty());
    debug_assert!(request.features.len() <= crate::MAX_FEATURES);
    Ok(())
}

/// Checks the binding and input against the admitted artifact, then evaluates and selects.
fn attempt(
    policy: Result<&crate::AdmittedPolicy, &crate::AdmissionError>,
    request: &super::DecisionRequest,
) -> Attempt {
    let admitted = match policy {
        Ok(admitted) => admitted,
        Err(crate::AdmissionError::Unavailable) => {
            return Attempt::failed(None, super::Failure::Unavailable);
        }
        Err(error) => {
            return Attempt::failed(None, super::Failure::IncompatibleArtifact(*error));
        }
    };
    let facts = || PolicyFacts {
        artifact: admitted.identity.clone(),
        model: admitted.model_identity.clone(),
        policy_kind: admitted.policy_kind,
    };
    let normalized = match admit_input(admitted, request) {
        Ok(normalized) => normalized,
        Err(failure) => return Attempt::failed(Some(facts()), failure),
    };
    let mut attempt = Attempt::failed(Some(facts()), super::Failure::OperationBudget);
    attempt.cost.normalization_operations = normalized.operations;
    if admitted.worst_case_operations > request.max_operations {
        return attempt;
    }
    attempt.selected = Err(super::Failure::ScoreOutOfRange);
    let Some((scores, operations)) =
        crate::inference::evaluate(&admitted.model, &normalized.values)
    else {
        return attempt;
    };
    debug_assert!(operations <= admitted.worst_case_operations);
    attempt.cost.inference_operations = operations;
    attempt.selected = best_eligible(admitted, &scores, &request.eligible);
    attempt.scores = scores;
    attempt
}

/// Normalized features and the normalizations taken.
struct Normalized {
    values: alloc::vec::Vec<i64>,
    operations: u64,
}

/// The binding check, the eligible labels' membership, and the normalized features.
fn admit_input(
    admitted: &crate::AdmittedPolicy,
    request: &super::DecisionRequest,
) -> Result<Normalized, super::Failure> {
    let binding = &request.binding;
    if binding.artifact != admitted.identity {
        return Err(super::Failure::Stale(super::StaleField::Artifact));
    }
    if binding.feature_schema != admitted.feature_schema {
        return Err(super::Failure::Stale(super::StaleField::FeatureSchema));
    }
    if binding.action_schema != admitted.action_schema {
        return Err(super::Failure::Stale(super::StaleField::ActionSchema));
    }
    if request.features.len() != admitted.features.len() {
        return Err(super::Failure::MalformedInput(
            super::InputFault::FeatureCount,
        ));
    }
    if !request
        .eligible
        .iter()
        .all(|label| admitted.actions.contains(label))
    {
        return Err(super::Failure::MalformedInput(
            super::InputFault::UnknownAction,
        ));
    }
    let mut normalized = Normalized {
        values: alloc::vec::Vec::with_capacity(request.features.len()),
        operations: 0,
    };
    for (feature, (value, bound)) in request.features.iter().zip(&admitted.features).enumerate() {
        let scaled = normalize(*value, bound).ok_or(super::Failure::InputOutOfRange { feature })?;
        normalized.values.push(scaled);
        normalized.operations = normalized.operations.saturating_add(1);
    }
    debug_assert_eq!(normalized.values.len(), admitted.features.len());
    debug_assert!(binding.artifact == admitted.identity);
    Ok(normalized)
}

/// `(value - min) * 1000 / (max - min)`, rounding down; `None` outside `[min, max]`.
fn normalize(value: i64, bound: &crate::FeatureBound) -> Option<i64> {
    if value < bound.min || value > bound.max {
        return None;
    }
    let above_min = value.checked_sub(bound.min)?;
    let span = bound.max.checked_sub(bound.min)?;
    let scaled = above_min.checked_mul(crate::PER_MILLE)?.checked_div(span)?;
    debug_assert!((0..=crate::PER_MILLE).contains(&scaled));
    Some(scaled)
}

/// The best eligible action; equal scores select the earliest declared action.
fn best_eligible(
    admitted: &crate::AdmittedPolicy,
    scores: &[i64],
    eligible: &[alloc::string::String],
) -> Result<(alloc::string::String, i64), super::Failure> {
    let (min, max) = admitted.score_bounds();
    if scores.len() != admitted.actions.len()
        || scores.iter().any(|score| !(min..=max).contains(score))
    {
        return Err(super::Failure::ScoreOutOfRange);
    }
    let mut best: Option<(&alloc::string::String, i64)> = None;
    for (label, score) in admitted.actions.iter().zip(scores) {
        let is_better = best.is_none_or(|(_, best_score)| *score > best_score);
        if eligible.contains(label) && is_better {
            best = Some((label, *score));
        }
    }
    let (action, score) = best.ok_or(super::Failure::MalformedInput(
        super::InputFault::NoEligibleAction,
    ))?;
    if admitted
        .abstain_below
        .is_some_and(|threshold| score < threshold)
    {
        return Err(super::Failure::Abstained);
    }
    debug_assert!(eligible.contains(action));
    Ok((action.clone(), score))
}
