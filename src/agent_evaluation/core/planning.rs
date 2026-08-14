mod admission;

const PLAN_DOMAIN: &str = "lattice.agent-evaluation.matrix-plan.v1";
const TRIAL_DOMAIN: &str = "lattice.agent-evaluation.trial.v1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectObservation {
    pub case_id: String,
    pub subject_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiveAuthorityFact {
    pub case_id: String,
    pub actor: String,
    pub target_namespace: String,
    pub credential_handles: Vec<String>,
    pub max_deployments: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BudgetAvailability {
    pub case_id: String,
    pub trials: u32,
    pub concurrency: u32,
    pub tokens: u64,
    pub duration_ms: u64,
    pub deployments: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRequest {
    pub selected_case_ids: Vec<String>,
    pub subject_observations: Vec<SubjectObservation>,
    pub available_services: std::collections::BTreeSet<super::model::ServiceKind>,
    pub budget_availability: Vec<BudgetAvailability>,
    pub live_authority: Vec<LiveAuthorityFact>,
    pub prior_trials: Vec<PriorTrial>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TrialPlan {
    pub order: u32,
    pub case_id: String,
    pub case_revision_identity: String,
    pub subject_identity: String,
    pub route: super::model::RouteKind,
    pub agent_profile_identity: String,
    pub environment_identity: String,
    pub policy_identity: String,
    pub trial_identity: String,
    pub live: bool,
    pub resume: ResumeDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MatrixPlan {
    pub catalog_identity: String,
    pub trials: Vec<TrialPlan>,
    pub identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeDecision {
    Run,
    SkipExact,
    BlockUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorTrial {
    pub trial_identity: String,
    pub case_revision_identity: String,
    pub subject_identity: String,
    pub agent_profile_identity: String,
    pub environment_identity: String,
    pub policy_identity: String,
    pub gate_set_identity: String,
    pub rubric_identity: Option<String>,
    pub reference_set_identity: String,
    pub artifact_policy_identity: String,
    pub status: super::model::TrialStatus,
    pub receipt_identity: Option<String>,
    pub expected_receipt_identity: Option<String>,
}

struct ResumeContext<'a> {
    subject_identity: &'a str,
    trial_identity: &'a str,
}

// r[impl lattice.agent_evaluation.planning]
pub fn plan_matrix(catalog: &super::model::AdmittedCatalog, request: &PlanRequest) -> Result<MatrixPlan, super::Error> {
    debug_assert!(catalog.cases.len() <= super::model::MAX_CASES);
    debug_assert!(!catalog.identity.is_empty());
    if request.selected_case_ids.is_empty() {
        return Err(super::Error::Selection);
    }
    admission::require_services(&request.available_services, false)?;
    let mut selected_ids = std::collections::BTreeSet::new();
    let mut trials = Vec::new();
    for selected_id in &request.selected_case_ids {
        if !selected_ids.insert(selected_id) {
            return Err(super::Error::Duplicate);
        }
        let case = catalog
            .cases
            .iter()
            .find(|candidate| candidate.definition.id == *selected_id)
            .ok_or(super::Error::Selection)?;
        let is_live = case.definition.live.enabled;
        admission::require_services(&request.available_services, is_live)?;
        admission::require_route_service(case.definition.route, &request.available_services)?;
        let observed_subject = unique_subject(request, selected_id)?;
        let expected_subject =
            super::identity::hash_value("lattice.agent-evaluation.subject.v1", &case.definition.subject)?;
        if observed_subject != expected_subject {
            return Err(super::Error::SubjectDrift);
        }
        admission::validate_budget_availability(case, request)?;
        if is_live {
            admission::validate_live_authority(case, request)?;
        }
        let case_trials = plan_case_trials(case, observed_subject, request)?;
        let planned_count = trials.len().checked_add(case_trials.len()).ok_or(super::Error::InvalidBudget)?;
        if planned_count > super::model::MAX_PLANNED_TRIALS {
            return Err(super::Error::InvalidBudget);
        }
        trials.reserve_exact(case_trials.len());
        trials.extend(case_trials);
    }
    trials.sort_by(|left, right| left.case_id.cmp(&right.case_id).then(left.order.cmp(&right.order)));
    for (index, trial) in trials.iter_mut().enumerate() {
        trial.order = u32::try_from(index).map_err(|_| super::Error::InvalidBudget)?;
    }
    let identity = super::identity::hash_value(PLAN_DOMAIN, &(catalog.identity.as_str(), trials.as_slice()))?;
    debug_assert_eq!(selected_ids.len(), request.selected_case_ids.len());
    debug_assert!(!trials.is_empty());
    Ok(MatrixPlan {
        catalog_identity: catalog.identity.clone(),
        trials,
        identity,
    })
}

fn plan_case_trials(
    case: &super::model::AdmittedCase,
    subject_identity: &str,
    request: &PlanRequest,
) -> Result<Vec<TrialPlan>, super::Error> {
    debug_assert!(case.definition.budget.trials > 0);
    debug_assert!(case.definition.budget.trials <= super::model::MAX_TRIALS_PER_CASE);
    let maximum_trials = usize::try_from(case.definition.budget.trials).map_err(|_| super::Error::InvalidBudget)?;
    let mut trials = Vec::with_capacity(maximum_trials);
    for ordinal in 0..case.definition.budget.trials {
        let trial_identity = super::identity::hash_value(
            TRIAL_DOMAIN,
            &(
                case.revision_identity.as_str(),
                subject_identity,
                case.definition.agent_profile_identity.as_str(),
                case.definition.environment.identity.as_str(),
                case.definition.policy_identity.as_str(),
                ordinal,
            ),
        )?;
        let context = ResumeContext {
            subject_identity,
            trial_identity: &trial_identity,
        };
        let resume = resume_decision(case, &context, request)?;
        trials.push(TrialPlan {
            order: ordinal,
            case_id: case.definition.id.clone(),
            case_revision_identity: case.revision_identity.clone(),
            subject_identity: subject_identity.to_string(),
            route: case.definition.route,
            agent_profile_identity: case.definition.agent_profile_identity.clone(),
            environment_identity: case.definition.environment.identity.clone(),
            policy_identity: case.definition.policy_identity.clone(),
            trial_identity,
            live: case.definition.live.enabled,
            resume,
        });
    }
    debug_assert!(!trials.is_empty());
    debug_assert!(trials.len() <= maximum_trials);
    Ok(trials)
}

fn resume_decision(
    case: &super::model::AdmittedCase,
    context: &ResumeContext<'_>,
    request: &PlanRequest,
) -> Result<ResumeDecision, super::Error> {
    debug_assert!(!context.subject_identity.is_empty());
    debug_assert!(!context.trial_identity.is_empty());
    let prior = request
        .prior_trials
        .iter()
        .filter(|prior| prior.trial_identity == context.trial_identity)
        .collect::<Vec<_>>();
    if prior.is_empty() {
        return Ok(ResumeDecision::Run);
    }
    if prior.len() != 1 {
        return Err(super::Error::Duplicate);
    }
    let prior = prior[0];
    if matches!(
        prior.status,
        super::model::TrialStatus::Started
            | super::model::TrialStatus::Unknown
            | super::model::TrialStatus::CleanupRequired
            | super::model::TrialStatus::Partial
    ) {
        return Ok(ResumeDecision::BlockUnknown);
    }
    if prior.status != super::model::TrialStatus::Passed {
        return Ok(ResumeDecision::Run);
    }
    let receipt_identity = prior.receipt_identity.as_deref().ok_or(super::Error::InvalidReceipt)?;
    let expected_receipt_identity = prior.expected_receipt_identity.as_deref().ok_or(super::Error::InvalidReceipt)?;
    super::identity::validate(receipt_identity)?;
    super::identity::validate(expected_receipt_identity)?;
    let gate_set_identity =
        super::identity::hash_value("lattice.agent-evaluation.gate-set.v1", &case.definition.gates)?;
    let rubric_identity = case.definition.rubric.as_ref().map(|rubric| rubric.identity.clone());
    let reference_set_identity =
        super::identity::hash_value("lattice.agent-evaluation.reference-set.v1", &case.definition.references)?;
    let is_exact = prior.status == super::model::TrialStatus::Passed
        && prior.case_revision_identity == case.revision_identity
        && prior.subject_identity == context.subject_identity
        && prior.agent_profile_identity == case.definition.agent_profile_identity
        && prior.environment_identity == case.definition.environment.identity
        && prior.policy_identity == case.definition.policy_identity
        && prior.gate_set_identity == gate_set_identity
        && prior.rubric_identity == rubric_identity
        && prior.reference_set_identity == reference_set_identity
        && prior.artifact_policy_identity == case.definition.artifacts.identity
        && receipt_identity == expected_receipt_identity;
    debug_assert_eq!(prior.trial_identity, context.trial_identity);
    debug_assert!(prior.status != super::model::TrialStatus::Planned);
    Ok(if is_exact {
        ResumeDecision::SkipExact
    } else {
        ResumeDecision::Run
    })
}

fn unique_subject<'a>(request: &'a PlanRequest, case_id: &str) -> Result<&'a str, super::Error> {
    let observations = request
        .subject_observations
        .iter()
        .filter(|observation| observation.case_id == case_id)
        .collect::<Vec<_>>();
    if observations.len() != 1 {
        return Err(super::Error::SubjectDrift);
    }
    super::identity::validate(&observations[0].subject_identity)?;
    Ok(&observations[0].subject_identity)
}
