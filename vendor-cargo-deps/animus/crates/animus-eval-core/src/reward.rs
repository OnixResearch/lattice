//! Finite attributed reward reduction.

use alloc::{string::String, vec::Vec};

use crate::{EvaluationPolicy, MissingRewardPolicy, hash_identity, validate_identity};

/// Meaning claimed for a reward contribution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RewardClaim {
    /// Bounded advisory or measured evidence only.
    Evidence,
    /// An invalid authority claim.
    Authorization,
    /// An invalid behavioral-correctness claim.
    CorrectnessProof,
    /// An invalid completion claim.
    Completion,
}

/// Planned reward dimension and exact attribution bindings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RewardDimension<'a> {
    /// Unique dimension name.
    pub name: &'a str,
    /// Inclusive minimum domain value.
    pub minimum: f64,
    /// Inclusive maximum domain value.
    pub maximum: f64,
    /// Non-negative finite aggregate weight.
    pub weight: f64,
    /// Scorer identity.
    pub scorer_identity: &'a str,
    /// Verifier identity.
    pub verifier_identity: &'a str,
    /// Rubric identity.
    pub rubric_identity: &'a str,
    /// Subject identity.
    pub subject_identity: &'a str,
    /// Trajectory identity.
    pub trajectory_identity: &'a str,
}

/// One supplied reward contribution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RewardContribution<'a> {
    /// Dimension name.
    pub dimension: &'a str,
    /// Optional finite value.
    pub value: Option<f64>,
    /// Scorer identity.
    pub scorer_identity: &'a str,
    /// Verifier identity.
    pub verifier_identity: &'a str,
    /// Rubric identity.
    pub rubric_identity: &'a str,
    /// Subject identity.
    pub subject_identity: &'a str,
    /// Trajectory identity.
    pub trajectory_identity: &'a str,
    /// Evidence identity.
    pub evidence_identity: &'a str,
    /// Whether a model or agent judge produced the value.
    pub judge_produced: bool,
    /// Claimed meaning.
    pub claim: RewardClaim,
}

/// Reduced dimension result.
#[derive(Clone, Debug, PartialEq)]
pub struct RewardDimensionResult {
    /// Dimension name.
    pub name: String,
    /// Accepted value, or `None` when policy excludes a missing value.
    pub value: Option<f64>,
    /// Configured weight.
    pub weight: f64,
    /// Accepted contribution count.
    pub contribution_count: u32,
    /// Whether the value remains advisory.
    pub advisory: bool,
    /// Exact evidence identity when a contribution was present.
    pub evidence_identity: Option<String>,
}

/// Deterministic reward summary.
#[derive(Clone, Debug, PartialEq)]
pub struct RewardReduction {
    /// Canonical reduction identity.
    pub identity: String,
    /// Results in configured dimension order.
    pub dimensions: Vec<RewardDimensionResult>,
    /// Weighted aggregate over included dimensions.
    pub aggregate: Option<f64>,
    /// Number of supplied values.
    pub contribution_count: u32,
    /// Number of missing values.
    pub missing_count: u32,
}

/// Reward reduction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RewardError {
    /// A dimension or attribution identity is malformed.
    Identity,
    /// Dimensions or contributions are duplicated or unknown.
    Dimension,
    /// A value, domain, or weight is not finite.
    NonFinite,
    /// A value is outside its declared domain.
    OutOfDomain,
    /// Attribution does not match the planned dimension.
    Binding,
    /// Required value is missing.
    Missing,
    /// A contribution claims authority, correctness, or completion.
    Overclaim,
    /// A count exceeds policy or representation bounds.
    Bound,
}

/// Reduce reward contributions in configured dimension order.
///
/// # Errors
/// Returns a deterministic validation class. Values are never silently clamped.
pub fn reduce_reward_dimensions(
    dimensions: &[RewardDimension<'_>],
    contributions: &[RewardContribution<'_>],
    policy: &EvaluationPolicy,
) -> Result<RewardReduction, RewardError> {
    if dimensions.is_empty()
        || dimensions.len()
            > usize::try_from(policy.max_reward_dimensions).map_err(|_| RewardError::Bound)?
    {
        return Err(RewardError::Bound);
    }
    validate_unique_dimensions(dimensions)?;
    validate_unique_contributions(contributions, dimensions)?;

    let mut results = Vec::with_capacity(dimensions.len());
    let mut weighted_total = 0.0f64;
    let mut included_weight = 0.0f64;
    let mut contribution_count = 0u32;
    let mut missing_count = 0u32;
    for dimension in dimensions {
        validate_dimension(dimension)?;
        let contribution = contributions
            .iter()
            .find(|candidate| candidate.dimension == dimension.name);
        let result = reduce_dimension(dimension, contribution, policy.missing_reward)?;
        if let Some(value) = result.value {
            weighted_total += value * result.weight;
            included_weight += result.weight;
        }
        contribution_count = contribution_count
            .checked_add(result.contribution_count)
            .ok_or(RewardError::Bound)?;
        if result.contribution_count == 0 {
            missing_count = missing_count.checked_add(1).ok_or(RewardError::Bound)?;
        }
        results.push(result);
    }
    let aggregate = if included_weight > 0.0 {
        Some(weighted_total / included_weight)
    } else {
        None
    };
    if aggregate.is_some_and(|value| !value.is_finite()) {
        return Err(RewardError::NonFinite);
    }
    let evidence_material = results
        .iter()
        .map(|result| result.evidence_identity.as_deref().unwrap_or("missing"))
        .collect::<Vec<_>>()
        .join(",");
    let dimension_material = results
        .iter()
        .map(|result| result.name.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let identity = hash_identity(
        "animus.native-evaluation.reward-reduction.v1",
        &[&dimension_material, &evidence_material, &policy.identity],
    );
    Ok(RewardReduction {
        identity,
        dimensions: results,
        aggregate,
        contribution_count,
        missing_count,
    })
}

fn validate_unique_dimensions(dimensions: &[RewardDimension<'_>]) -> Result<(), RewardError> {
    for (index, dimension) in dimensions.iter().enumerate() {
        if dimensions[..index]
            .iter()
            .any(|prior| prior.name == dimension.name)
        {
            return Err(RewardError::Dimension);
        }
    }
    Ok(())
}

fn validate_unique_contributions(
    contributions: &[RewardContribution<'_>],
    dimensions: &[RewardDimension<'_>],
) -> Result<(), RewardError> {
    for (index, contribution) in contributions.iter().enumerate() {
        if contributions[..index]
            .iter()
            .any(|prior| prior.dimension == contribution.dimension)
            || !dimensions
                .iter()
                .any(|dimension| dimension.name == contribution.dimension)
        {
            return Err(RewardError::Dimension);
        }
    }
    Ok(())
}

fn validate_dimension(dimension: &RewardDimension<'_>) -> Result<(), RewardError> {
    for identity in [
        dimension.name,
        dimension.scorer_identity,
        dimension.verifier_identity,
        dimension.rubric_identity,
        dimension.subject_identity,
        dimension.trajectory_identity,
    ] {
        validate_identity(identity).map_err(|_| RewardError::Identity)?;
    }
    if !dimension.minimum.is_finite()
        || !dimension.maximum.is_finite()
        || !dimension.weight.is_finite()
    {
        return Err(RewardError::NonFinite);
    }
    if dimension.minimum > dimension.maximum || dimension.weight < 0.0 {
        return Err(RewardError::OutOfDomain);
    }
    Ok(())
}

fn reduce_dimension(
    dimension: &RewardDimension<'_>,
    contribution: Option<&RewardContribution<'_>>,
    missing_policy: MissingRewardPolicy,
) -> Result<RewardDimensionResult, RewardError> {
    let Some(contribution) = contribution else {
        return match missing_policy {
            MissingRewardPolicy::Error => Err(RewardError::Missing),
            MissingRewardPolicy::Exclude => Ok(missing_result(dimension, None)),
            MissingRewardPolicy::Zero => {
                if dimension.minimum > 0.0 || dimension.maximum < 0.0 {
                    return Err(RewardError::OutOfDomain);
                }
                Ok(missing_result(dimension, Some(0.0)))
            }
        };
    };
    for identity in [
        contribution.dimension,
        contribution.scorer_identity,
        contribution.verifier_identity,
        contribution.rubric_identity,
        contribution.subject_identity,
        contribution.trajectory_identity,
        contribution.evidence_identity,
    ] {
        validate_identity(identity).map_err(|_| RewardError::Identity)?;
    }
    if contribution.claim != RewardClaim::Evidence {
        return Err(RewardError::Overclaim);
    }
    if contribution.scorer_identity != dimension.scorer_identity
        || contribution.verifier_identity != dimension.verifier_identity
        || contribution.rubric_identity != dimension.rubric_identity
        || contribution.subject_identity != dimension.subject_identity
        || contribution.trajectory_identity != dimension.trajectory_identity
    {
        return Err(RewardError::Binding);
    }
    let Some(value) = contribution.value else {
        return match missing_policy {
            MissingRewardPolicy::Error => Err(RewardError::Missing),
            MissingRewardPolicy::Exclude => Ok(missing_result(dimension, None)),
            MissingRewardPolicy::Zero => {
                if dimension.minimum > 0.0 || dimension.maximum < 0.0 {
                    return Err(RewardError::OutOfDomain);
                }
                Ok(missing_result(dimension, Some(0.0)))
            }
        };
    };
    if !value.is_finite() {
        return Err(RewardError::NonFinite);
    }
    if value < dimension.minimum || value > dimension.maximum {
        return Err(RewardError::OutOfDomain);
    }
    Ok(RewardDimensionResult {
        name: dimension.name.into(),
        value: Some(value),
        weight: dimension.weight,
        contribution_count: 1,
        advisory: contribution.judge_produced,
        evidence_identity: Some(contribution.evidence_identity.into()),
    })
}

fn missing_result(dimension: &RewardDimension<'_>, value: Option<f64>) -> RewardDimensionResult {
    RewardDimensionResult {
        name: dimension.name.into(),
        value,
        weight: dimension.weight,
        contribution_count: 0,
        advisory: true,
        evidence_identity: None,
    }
}
