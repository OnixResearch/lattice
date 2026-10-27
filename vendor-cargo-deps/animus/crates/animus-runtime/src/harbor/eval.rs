//! Pure deterministic planning, ATIF projection, reward reduction, and receipts.

pub use animus_adapter_atif::{AtifImportProjection, ValidatedTrajectory};

pub use animus_eval_core::{
    CompatNetworkPosture as NetworkPosture, CompatPlanError as PlanError, CompatRoute as Route,
    CompatTask as Task, CompatTrialPlan as TrialPlan, MAX_COMPAT_CONCURRENCY as MAX_CONCURRENCY,
    MAX_COMPAT_TRIALS as MAX_TRIALS, MAX_IDENTITY_BYTES,
};

/// Maximum ATIF steps.
pub const MAX_ATIF_STEPS: usize = 4_096;
/// Maximum tool calls or observations in one step.
pub const MAX_STEP_ITEMS: usize = 128;

/// Build a stable task-major, route-minor trial matrix through the native core.
///
/// # Errors
/// Returns a fail-closed planning error for malformed inputs, bounds, or stale resume entries.
pub fn plan_trials(
    tasks: &[Task<'_>],
    routes: &[Route<'_>],
    concurrency: u32,
    completed: &[&str],
) -> Result<Vec<TrialPlan>, PlanError> {
    animus_eval_core::plan_compat_trials(tasks, routes, concurrency, completed)
}

/// ATIF source role.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifSource {
    /// User input.
    User,
    /// Agent output.
    Agent,
    /// System event.
    System,
}

/// ATIF tool-call reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCall<'a> {
    /// Call identity.
    pub id: &'a str,
    /// Function identity.
    pub function: &'a str,
}

/// ATIF observation reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationRef<'a> {
    /// Source call identity when correlated.
    pub source_call_id: Option<&'a str>,
    /// Embedded subagent trajectory identity.
    pub subagent_trajectory_id: Option<&'a str>,
}

/// Bounded ATIF step facts; raw message/reasoning/tool payloads stay outside safe projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifStep<'a> {
    /// One-based step identity.
    pub id: usize,
    /// Source role.
    pub source: AtifSource,
    /// Tool calls.
    pub calls: &'a [ToolCall<'a>],
    /// Observations.
    pub observations: &'a [ObservationRef<'a>],
    /// Whether hidden reasoning was present and omitted.
    pub hidden_reasoning_omitted: bool,
    /// Terminal marker.
    pub terminal: bool,
}

/// ATIF trajectory facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifTrajectory<'a> {
    /// Exact schema version.
    pub schema: &'a str,
    /// Safe trajectory identity.
    pub identity: &'a str,
    /// Ordered steps.
    pub steps: &'a [AtifStep<'a>],
    /// Declared embedded subagent identities.
    pub subagents: &'a [&'a str],
}

/// Safe ATIF projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtifProjection {
    /// Trajectory identity.
    pub identity: String,
    /// Step count.
    pub steps: usize,
    /// Tool-call count.
    pub calls: usize,
    /// Observation count.
    pub observations: usize,
    /// Hidden-reasoning redaction count.
    pub hidden_reasoning_redactions: usize,
}

/// ATIF validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtifError {
    /// Unsupported version.
    Version,
    /// Invalid step sequence or missing terminal state.
    Sequence,
    /// Duplicate/orphan tool observation.
    Correlation,
    /// Invalid subagent lineage.
    Lineage,
    /// Collection or identity bound exceeded.
    Bound,
}

/// Validate exact `ATIF-v1.7` facts and project metadata only.
///
/// # Errors
/// Returns a deterministic schema, sequence, correlation, lineage, or bound error.
pub fn project_atif(value: &AtifTrajectory<'_>) -> Result<AtifProjection, AtifError> {
    if value.schema != animus_adapters::harbor::ATIF_SCHEMA {
        return Err(AtifError::Version);
    }
    if !safe(value.identity) || value.steps.is_empty() || value.steps.len() > MAX_ATIF_STEPS {
        return Err(AtifError::Bound);
    }
    debug_assert_eq!(value.schema, animus_adapters::harbor::ATIF_SCHEMA);
    debug_assert!(safe(value.identity));
    debug_assert!(!value.steps.is_empty());
    debug_assert!(value.steps.len() <= MAX_ATIF_STEPS);

    let subagents: std::collections::BTreeSet<_> = value.subagents.iter().copied().collect();
    if subagents.len() != value.subagents.len() || value.subagents.iter().any(|item| !safe(item)) {
        return Err(AtifError::Lineage);
    }
    debug_assert_eq!(subagents.len(), value.subagents.len());
    let mut calls_total = 0usize;
    let mut observations_total = 0usize;
    let mut redactions = 0usize;
    for (index, step) in value.steps.iter().enumerate() {
        let expected_step_id = index.saturating_add(1);
        if step.id != expected_step_id
            || step.calls.len() > MAX_STEP_ITEMS
            || step.observations.len() > MAX_STEP_ITEMS
        {
            return Err(AtifError::Sequence);
        }
        let calls: std::collections::BTreeSet<_> = step.calls.iter().map(|call| call.id).collect();
        if calls.len() != step.calls.len()
            || step
                .calls
                .iter()
                .any(|call| !safe(call.id) || !safe(call.function))
        {
            return Err(AtifError::Correlation);
        }
        for observation in step.observations {
            if observation
                .source_call_id
                .is_some_and(|id| !calls.contains(id))
            {
                return Err(AtifError::Correlation);
            }
            if observation
                .subagent_trajectory_id
                .is_some_and(|id| !subagents.contains(id))
            {
                return Err(AtifError::Lineage);
            }
        }
        calls_total = calls_total.saturating_add(step.calls.len());
        observations_total = observations_total.saturating_add(step.observations.len());
        redactions = redactions.saturating_add(usize::from(step.hidden_reasoning_omitted));
    }
    if !value.steps.last().is_some_and(|step| step.terminal) {
        return Err(AtifError::Sequence);
    }
    debug_assert!(value.steps.last().is_some_and(|step| step.terminal));
    debug_assert!(redactions <= value.steps.len());
    Ok(AtifProjection {
        identity: value.identity.into(),
        steps: value.steps.len(),
        calls: calls_total,
        observations: observations_total,
        hidden_reasoning_redactions: redactions,
    })
}

/// Project metadata from the complete ATIF adapter validation and import lanes.
///
/// This entrypoint does not infer or require a terminal trial outcome. The Harbor
/// evaluation host must supply that outcome through its existing explicit owner.
///
/// # Errors
/// Returns a bound or identity error when supplied projections do not match.
pub fn project_validated_atif(
    validated: &ValidatedTrajectory,
    imported: &AtifImportProjection,
) -> Result<AtifProjection, AtifError> {
    if imported.semantic_identity != validated.semantic_identity() {
        return Err(AtifError::Correlation);
    }
    let counts = validated.counts();
    let steps = usize::try_from(counts.steps).map_err(|_| AtifError::Bound)?;
    let calls = usize::try_from(counts.tool_calls).map_err(|_| AtifError::Bound)?;
    let observations = usize::try_from(counts.observation_results).map_err(|_| AtifError::Bound)?;
    let hidden_reasoning_redactions =
        usize::try_from(imported.hidden_reasoning_redactions).map_err(|_| AtifError::Bound)?;
    if steps == 0 || steps > MAX_ATIF_STEPS || hidden_reasoning_redactions > steps {
        return Err(AtifError::Bound);
    }
    Ok(AtifProjection {
        identity: validated.semantic_identity().into(),
        steps,
        calls,
        observations,
        hidden_reasoning_redactions,
    })
}

pub use animus_eval_core::{
    CompatReward as Reward, CompatRewardError as RewardError, CompatRewardSummary as RewardSummary,
};

/// Reduce rewards through the native core in supplied plan order and metric order.
///
/// # Errors
/// Returns identity, duplicate, or non-finite failures without dropping invalid values.
pub fn reduce_rewards(rewards: &[Reward<'_>]) -> Result<Vec<RewardSummary>, RewardError> {
    animus_eval_core::reduce_compat_rewards(rewards)
}

/// Metadata-only evaluation receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationReceipt {
    /// Job identity.
    pub job_identity: String,
    /// Planned trials.
    pub planned: u32,
    /// Completed trials.
    pub completed: u32,
    /// ATIF projection identities.
    pub trajectories: Vec<String>,
    /// Explicit non-claims.
    pub non_claims: Vec<&'static str>,
}

/// Project a bounded receipt from explicit current facts.
///
/// # Errors
/// Returns an identity error for unsafe job identities or impossible completion counts.
pub fn receipt(
    job: &str,
    plans: &[TrialPlan],
    completed: u32,
    trajectories: &[AtifProjection],
) -> Result<EvaluationReceipt, PlanError> {
    let planned = u32::try_from(plans.len()).map_err(|_| PlanError::TrialLimit)?;
    if !safe(job) {
        return Err(PlanError::Identity);
    }
    if completed > planned {
        return Err(PlanError::Identity);
    }
    debug_assert!(safe(job));
    debug_assert!(completed <= planned);

    let trajectory_identities = trajectories
        .iter()
        .map(|item| item.identity.clone())
        .collect::<Vec<_>>();
    debug_assert_eq!(trajectory_identities.len(), trajectories.len());
    Ok(EvaluationReceipt {
        job_identity: job.into(),
        planned,
        completed,
        trajectories: trajectory_identities,
        non_claims: vec![
            "not-verifier-soundness",
            "not-benchmark-superiority",
            "not-automatic-completion",
            "not-production-or-release-readiness",
        ],
    })
}

fn safe(value: &str) -> bool {
    animus_eval_core::is_safe_identity(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn vertical_slice_is_deterministic_and_metadata_only() {
        let tasks = [Task {
            identity: "task:1",
            environment: "env:1",
            verifier: "verifier:1",
            rubric: "rubric:1",
            network: NetworkPosture::None,
            separate_verifier: true,
        }];
        let routes = [Route {
            agent: "agent:1",
            model: "model:1",
        }];
        let plans = plan_trials(&tasks, &routes, 1, &[]).expect("plan");
        let calls = [ToolCall {
            id: "call:1",
            function: "tool:fixture",
        }];
        let observations = [ObservationRef {
            source_call_id: Some("call:1"),
            subagent_trajectory_id: None,
        }];
        let steps = [AtifStep {
            id: 1,
            source: AtifSource::Agent,
            calls: &calls,
            observations: &observations,
            hidden_reasoning_omitted: true,
            terminal: true,
        }];
        let projection = project_atif(&AtifTrajectory {
            schema: "ATIF-v1.7",
            identity: "trajectory:1",
            steps: &steps,
            subagents: &[],
        })
        .expect("atif");
        let rewards = reduce_rewards(&[Reward {
            trial: &plans[0].identity,
            metric: "correctness",
            value: 1.0,
            verifier: "verifier:1",
            advisory: false,
        }])
        .expect("reward");
        assert!((rewards[0].mean - 1.0).abs() < f64::EPSILON);
        assert_eq!(projection.hidden_reasoning_redactions, 1);
        assert_eq!(
            receipt("job:1", &plans, 1, &[projection])
                .expect("receipt")
                .completed,
            1
        );
    }
    #[test]
    fn planning_atif_rewards_and_receipts_fail_closed() {
        let task = Task {
            identity: "task:1",
            environment: "env:1",
            verifier: "verifier:1",
            rubric: "rubric:1",
            network: NetworkPosture::None,
            separate_verifier: false,
        };
        assert_eq!(
            plan_trials(
                &[task],
                &[Route {
                    agent: "agent:1",
                    model: "model:1"
                }],
                1,
                &[]
            ),
            Err(PlanError::VerifierIsolation)
        );
        let steps = [AtifStep {
            id: 2,
            source: AtifSource::Agent,
            calls: &[],
            observations: &[],
            hidden_reasoning_omitted: false,
            terminal: false,
        }];
        assert_eq!(
            project_atif(&AtifTrajectory {
                schema: "ATIF-v1.6",
                identity: "trajectory:1",
                steps: &steps,
                subagents: &[]
            }),
            Err(AtifError::Version)
        );
        assert_eq!(
            reduce_rewards(&[Reward {
                trial: "trial:1",
                metric: "score",
                value: f64::NAN,
                verifier: "verifier:1",
                advisory: true
            }]),
            Err(RewardError::NonFinite)
        );
        assert_eq!(
            receipt("/secret/path", &[], 1, &[]),
            Err(PlanError::Identity)
        );
    }
}
