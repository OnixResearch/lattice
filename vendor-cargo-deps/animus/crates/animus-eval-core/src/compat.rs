//! Compatibility values used by the optional Harbor projection.

use alloc::{collections::BTreeSet, string::String, vec::Vec};

use crate::{hash_identity, validate_identity};

/// Maximum admitted compatibility trials.
pub const MAX_COMPAT_TRIALS: usize = 256;
/// Maximum admitted compatibility concurrency.
pub const MAX_COMPAT_CONCURRENCY: u32 = 16;

/// Explicit network posture for one compatibility task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatNetworkPosture {
    /// No network access.
    None,
    /// Explicit private-network request for a host to interpret.
    Private,
}

/// Compatibility task profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatTask<'a> {
    /// Task identity.
    pub identity: &'a str,
    /// Environment identity.
    pub environment: &'a str,
    /// Verifier identity.
    pub verifier: &'a str,
    /// Rubric identity.
    pub rubric: &'a str,
    /// Network posture.
    pub network: CompatNetworkPosture,
    /// Separate verifier requirement.
    pub separate_verifier: bool,
}

/// Compatibility agent/model route.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatRoute<'a> {
    /// Agent identity.
    pub agent: &'a str,
    /// Model route identity.
    pub model: &'a str,
}

/// Compatibility trial plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompatTrialPlan {
    /// Stable identity.
    pub identity: String,
    /// Task identity.
    pub task: String,
    /// Agent identity.
    pub agent: String,
    /// Model identity.
    pub model: String,
    /// Stable order.
    pub order: usize,
}

/// Compatibility planning failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatPlanError {
    /// Empty, duplicate, or unsafe identity.
    Identity,
    /// Missing separate verifier isolation.
    VerifierIsolation,
    /// Empty or excessive matrix.
    TrialLimit,
    /// Invalid concurrency.
    Concurrency,
    /// Resume evidence did not bind a current trial.
    StaleResume,
}

/// Plan a stable task-major compatibility matrix through the native core.
///
/// # Errors
/// Returns a fail-closed validation class.
pub fn plan_compat_trials(
    tasks: &[CompatTask<'_>],
    routes: &[CompatRoute<'_>],
    concurrency: u32,
    completed: &[&str],
) -> Result<Vec<CompatTrialPlan>, CompatPlanError> {
    if concurrency == 0 || concurrency > MAX_COMPAT_CONCURRENCY {
        return Err(CompatPlanError::Concurrency);
    }
    let trial_count = tasks.len().saturating_mul(routes.len());
    if tasks.is_empty() || routes.is_empty() || trial_count > MAX_COMPAT_TRIALS {
        return Err(CompatPlanError::TrialLimit);
    }
    let mut identities = BTreeSet::new();
    let mut plans = Vec::with_capacity(trial_count);
    for task in tasks {
        if !task.separate_verifier {
            return Err(CompatPlanError::VerifierIsolation);
        }
        for identity in [task.identity, task.environment, task.verifier, task.rubric] {
            validate_identity(identity).map_err(|_| CompatPlanError::Identity)?;
        }
        for route in routes {
            validate_identity(route.agent).map_err(|_| CompatPlanError::Identity)?;
            validate_identity(route.model).map_err(|_| CompatPlanError::Identity)?;
            let identity = hash_identity(
                "animus.harbor.trial.v1",
                &[
                    task.identity,
                    route.agent,
                    route.model,
                    task.environment,
                    task.verifier,
                    task.rubric,
                ],
            );
            if !identities.insert(identity.clone()) {
                return Err(CompatPlanError::Identity);
            }
            plans.push(CompatTrialPlan {
                identity,
                task: task.identity.into(),
                agent: route.agent.into(),
                model: route.model.into(),
                order: plans.len(),
            });
        }
    }
    if completed
        .iter()
        .any(|identity| !identities.contains(*identity))
    {
        return Err(CompatPlanError::StaleResume);
    }
    plans.retain(|plan| !completed.contains(&plan.identity.as_str()));
    Ok(plans)
}

/// Compatibility reward input.
#[derive(Clone, Debug, PartialEq)]
pub struct CompatReward<'a> {
    /// Trial identity.
    pub trial: &'a str,
    /// Metric name.
    pub metric: &'a str,
    /// Finite value.
    pub value: f64,
    /// Verifier identity.
    pub verifier: &'a str,
    /// Whether this is advisory judgment.
    pub advisory: bool,
}

/// Compatibility reward summary.
#[derive(Clone, Debug, PartialEq)]
pub struct CompatRewardSummary {
    /// Metric name.
    pub metric: String,
    /// Ordered finite mean.
    pub mean: f64,
    /// Value count.
    pub count: u32,
    /// Advisory value count.
    pub advisory_count: u32,
}

/// Compatibility reward failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatRewardError {
    /// Missing or malformed identity.
    Identity,
    /// Duplicate trial/metric reward.
    Duplicate,
    /// Non-finite value.
    NonFinite,
}

/// Reduce compatibility rewards in supplied order.
///
/// # Errors
/// Returns identity, duplicate, or non-finite failures.
pub fn reduce_compat_rewards(
    rewards: &[CompatReward<'_>],
) -> Result<Vec<CompatRewardSummary>, CompatRewardError> {
    let mut seen = BTreeSet::new();
    let mut metrics: Vec<CompatRewardSummary> = Vec::with_capacity(rewards.len());
    for reward in rewards {
        for identity in [reward.trial, reward.metric, reward.verifier] {
            validate_identity(identity).map_err(|_| CompatRewardError::Identity)?;
        }
        if !reward.value.is_finite() {
            return Err(CompatRewardError::NonFinite);
        }
        if !seen.insert((reward.trial, reward.metric)) {
            return Err(CompatRewardError::Duplicate);
        }
        if let Some(summary) = metrics.iter_mut().find(|item| item.metric == reward.metric) {
            let next_count = summary.count.saturating_add(1);
            summary.mean =
                (summary.mean * f64::from(summary.count) + reward.value) / f64::from(next_count);
            summary.count = next_count;
            summary.advisory_count = summary
                .advisory_count
                .saturating_add(u32::from(reward.advisory));
        } else {
            metrics.push(CompatRewardSummary {
                metric: reward.metric.into(),
                mean: reward.value,
                count: 1,
                advisory_count: u32::from(reward.advisory),
            });
        }
    }
    Ok(metrics)
}
