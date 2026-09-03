#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskOutcome {
    Passed,
    Failed,
    NotObserved,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PilotObservation {
    pub task_id: String,
    pub profile: super::StructuredInputProjection,
    pub prompt_tokens: Option<u64>,
    pub outcome: TaskOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PilotBindings {
    pub model_identity: String,
    pub endpoint_class: String,
    pub system_prompt_identity: String,
    pub user_prompt_identity: String,
    pub corpus_identity: String,
    pub comparator_identity: String,
    pub runtime_profile_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectionPilotInput {
    pub expected: PilotBindings,
    pub observed: PilotBindings,
    pub observations: Vec<PilotObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PilotVerdict {
    EligibleForExactExplicitUse,
    InconclusiveMissingUsage,
    InconclusiveMissingOutcome,
    InconclusiveIncompleteCohort,
    BlockedTaskRegression,
    BlockedNoStrictTokenBenefit,
    BlockedBindingDrift,
    BlockedContradictoryObservation,
    BlockedTokenOverflow,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProjectionPilotReport {
    pub model_identity: String,
    pub corpus_identity: String,
    pub comparator_identity: String,
    pub task_count: usize,
    pub compact_prompt_tokens: Option<u64>,
    pub toon_v3_prompt_tokens: Option<u64>,
    pub verdict: PilotVerdict,
    pub recommends_default_promotion: bool,
    pub claims_billing_truth: bool,
    pub claims_equal_accuracy: bool,
    pub claims_production_readiness: bool,
}

// r[impl lattice.agent_runtime.projection_evaluation]
// r[impl lattice.agent_runtime.projection_evaluation.scenario.eligible]
// r[impl lattice.agent_runtime.projection_evaluation.scenario.blocked]
pub fn evaluate_pilot(input: &ProjectionPilotInput) -> ProjectionPilotReport {
    let mut report = empty_report(&input.observed);
    debug_assert!(!report.recommends_default_promotion);
    debug_assert!(!report.claims_billing_truth);
    if input.observed != input.expected {
        report.verdict = PilotVerdict::BlockedBindingDrift;
        return report;
    }

    let mut by_task = std::collections::BTreeMap::new();
    for observation in &input.observations {
        if !matches!(
            observation.profile,
            super::StructuredInputProjection::JsonCompact | super::StructuredInputProjection::ToonV3
        ) {
            continue;
        }
        let key = (observation.task_id.clone(), observation.profile);
        if by_task.insert(key, observation).is_some() {
            report.verdict = PilotVerdict::BlockedContradictoryObservation;
            return report;
        }
    }

    let task_ids = input
        .observations
        .iter()
        .map(|observation| observation.task_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    report.task_count = task_ids.len();
    debug_assert!(!task_ids.is_empty());
    if task_ids.is_empty() {
        report.verdict = PilotVerdict::InconclusiveIncompleteCohort;
        return report;
    }

    let mut totals = (0_u64, 0_u64);
    for task_id in task_ids {
        let previous = totals;
        if let Err(verdict) = fold_task_totals(&by_task, task_id, &mut totals) {
            report.verdict = verdict;
            return report;
        }
        debug_assert!(totals.0 >= previous.0);
        debug_assert!(totals.1 >= previous.1);
    }
    let (compact_total, toon_total) = totals;

    report.compact_prompt_tokens = Some(compact_total);
    report.toon_v3_prompt_tokens = Some(toon_total);
    debug_assert_eq!(report.compact_prompt_tokens, Some(compact_total));
    report.verdict = if toon_total < compact_total {
        PilotVerdict::EligibleForExactExplicitUse
    } else {
        PilotVerdict::BlockedNoStrictTokenBenefit
    };
    report
}

fn fold_task_totals(
    by_task: &std::collections::BTreeMap<(String, super::StructuredInputProjection), &PilotObservation>,
    task_id: &str,
    totals: &mut (u64, u64),
) -> Result<(), PilotVerdict> {
    let compact_key = (task_id.to_string(), super::StructuredInputProjection::JsonCompact);
    let toon_key = (task_id.to_string(), super::StructuredInputProjection::ToonV3);
    let (Some(compact), Some(toon)) = (by_task.get(&compact_key), by_task.get(&toon_key)) else {
        return Err(PilotVerdict::InconclusiveIncompleteCohort);
    };
    if compact.outcome == TaskOutcome::NotObserved || toon.outcome == TaskOutcome::NotObserved {
        return Err(PilotVerdict::InconclusiveMissingOutcome);
    }
    if compact.outcome == TaskOutcome::Passed && toon.outcome != TaskOutcome::Passed {
        return Err(PilotVerdict::BlockedTaskRegression);
    }
    let (Some(compact_tokens), Some(toon_tokens)) = (compact.prompt_tokens, toon.prompt_tokens) else {
        return Err(PilotVerdict::InconclusiveMissingUsage);
    };
    debug_assert!(compact.prompt_tokens.is_some());
    debug_assert!(toon.prompt_tokens.is_some());
    totals.0 = totals.0.checked_add(compact_tokens).ok_or(PilotVerdict::BlockedTokenOverflow)?;
    totals.1 = totals.1.checked_add(toon_tokens).ok_or(PilotVerdict::BlockedTokenOverflow)?;
    debug_assert!(totals.0 >= compact_tokens);
    debug_assert!(totals.1 >= toon_tokens);
    Ok(())
}

fn empty_report(bindings: &PilotBindings) -> ProjectionPilotReport {
    ProjectionPilotReport {
        model_identity: bindings.model_identity.clone(),
        corpus_identity: bindings.corpus_identity.clone(),
        comparator_identity: bindings.comparator_identity.clone(),
        task_count: 0,
        compact_prompt_tokens: None,
        toon_v3_prompt_tokens: None,
        verdict: PilotVerdict::InconclusiveIncompleteCohort,
        recommends_default_promotion: false,
        claims_billing_truth: false,
        claims_equal_accuracy: false,
        claims_production_readiness: false,
    }
}
