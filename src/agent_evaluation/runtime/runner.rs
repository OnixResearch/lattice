#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionFact {
    pub output_identity: String,
    pub animus_fact: Option<super::evidence::AnimusFact>,
}

pub trait WorkspacePort {
    fn prepare(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
    ) -> Result<String, super::Error>;
    fn capture_post_state(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
    ) -> Result<String, super::Error>;
}

pub trait AgentPort {
    fn execute(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
    ) -> Result<ExecutionFact, super::Error>;
}

pub trait WorkflowPort {
    fn execute(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
    ) -> Result<ExecutionFact, super::Error>;
}

pub trait VerifierPort {
    fn verify(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
        execution: &ExecutionFact,
    ) -> Result<Vec<super::oracles::GateEvidence>, super::Error>;
}

pub trait JudgePort {
    fn judge(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
        execution: &ExecutionFact,
    ) -> Result<super::oracles::JudgeEvidence, super::Error>;
}

pub trait DeploymentPort {
    fn deploy(
        &self,
        case: &super::model::AdmittedCase,
        admission: &super::live::Admission,
        trial: &super::planning::TrialPlan,
        execution: &ExecutionFact,
    ) -> Result<super::live::Observation, super::Error>;
}

pub trait CleanupPort {
    fn cleanup(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
        admission: &super::live::Admission,
        observation: &super::live::Observation,
    ) -> Result<bool, super::Error>;
}

pub trait ArtifactPort {
    fn collect(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
        execution: &ExecutionFact,
    ) -> Result<Vec<super::artifacts::ArtifactCandidate>, super::Error>;
    fn persist(
        &self,
        case: &super::model::AdmittedCase,
        trial: &super::planning::TrialPlan,
        manifest: &super::artifacts::ArtifactManifest,
    ) -> Result<(), super::Error>;
}

pub trait ClockPort {
    fn observed_epoch_ms(&self) -> Result<u64, super::Error>;
}

pub trait TrialStorePort {
    fn record_start(&self, trial: &super::planning::TrialPlan, observed_epoch_ms: u64) -> Result<(), super::Error>;
    fn cancellation_requested(&self, trial: &super::planning::TrialPlan) -> Result<bool, super::Error>;
    fn record_cancelled(&self, trial: &super::planning::TrialPlan, observed_epoch_ms: u64) -> Result<(), super::Error>;
    fn record_unknown(&self, trial: &super::planning::TrialPlan, observed_epoch_ms: u64) -> Result<(), super::Error>;
    fn record_receipt(&self, receipt: &super::evidence::Receipt, observed_epoch_ms: u64) -> Result<(), super::Error>;
}

pub struct Services<'a> {
    pub workspace: &'a dyn WorkspacePort,
    pub agent: &'a dyn AgentPort,
    pub workflow: &'a dyn WorkflowPort,
    pub verifier: &'a dyn VerifierPort,
    pub judge: &'a dyn JudgePort,
    pub deployment: &'a dyn DeploymentPort,
    pub cleanup: &'a dyn CleanupPort,
    pub artifact: &'a dyn ArtifactPort,
    pub clock: &'a dyn ClockPort,
    pub store: &'a dyn TrialStorePort,
}

pub struct RunRequest<'a> {
    pub catalog: &'a super::model::AdmittedCatalog,
    pub trial: &'a super::planning::TrialPlan,
    pub policy_identity: &'a str,
    pub live_authority: Option<&'a super::planning::LiveAuthorityFact>,
}

// r[impl lattice.agent_evaluation.execution]
pub fn run_trial(request: &RunRequest<'_>, services: &Services<'_>) -> Result<super::evidence::Receipt, super::Error> {
    debug_assert!(!request.catalog.identity.is_empty());
    debug_assert!(!request.trial.trial_identity.is_empty());
    match request.trial.resume {
        super::planning::ResumeDecision::SkipExact => return Err(super::Error::AlreadyComplete),
        super::planning::ResumeDecision::BlockUnknown => return Err(super::Error::RecoveryBlocked),
        super::planning::ResumeDecision::Run => {}
    }
    let case = request
        .catalog
        .cases
        .iter()
        .find(|case| {
            case.definition.id == request.trial.case_id
                && case.revision_identity == request.trial.case_revision_identity
        })
        .ok_or(super::Error::CaseDrift)?;
    let is_policy_mismatch = request.policy_identity != request.trial.policy_identity
        || request.policy_identity != case.definition.policy_identity;
    if is_policy_mismatch {
        return Err(super::Error::CaseDrift);
    }
    let pre_state_identity = services.workspace.prepare(case, request.trial)?;
    if pre_state_identity != request.trial.subject_identity {
        return Err(super::Error::SubjectDrift);
    }
    let started_at = services.clock.observed_epoch_ms()?;
    services.store.record_start(request.trial, started_at)?;
    if services.store.cancellation_requested(request.trial)? {
        services.store.record_cancelled(request.trial, started_at)?;
        return Err(super::Error::CancellationRequested);
    }
    let result = run_started(request, case, &pre_state_identity, services);
    match result {
        Ok(receipt) => {
            let completed_at = services.clock.observed_epoch_ms()?;
            services.store.record_receipt(&receipt, completed_at)?;
            Ok(receipt)
        }
        Err(super::Error::CancellationRequested) => Err(super::Error::CancellationRequested),
        Err(error) => {
            let observed_at = services.clock.observed_epoch_ms().unwrap_or(started_at);
            services
                .store
                .record_unknown(request.trial, observed_at)
                .map_err(|_| super::Error::RecoveryBlocked)?;
            Err(error)
        }
    }
}

fn run_started(
    request: &RunRequest<'_>,
    case: &super::model::AdmittedCase,
    pre_state_identity: &str,
    services: &Services<'_>,
) -> Result<super::evidence::Receipt, super::Error> {
    debug_assert_eq!(request.trial.case_id, case.definition.id);
    debug_assert_eq!(pre_state_identity, request.trial.subject_identity);
    let execution = match request.trial.route {
        super::model::RouteKind::Agent => services.agent.execute(case, request.trial)?,
        super::model::RouteKind::Workflow => services.workflow.execute(case, request.trial)?,
    };
    super::identity::validate(&execution.output_identity)?;
    if services.store.cancellation_requested(request.trial)? {
        let observed_at = services.clock.observed_epoch_ms()?;
        services.store.record_cancelled(request.trial, observed_at)?;
        return Err(super::Error::CancellationRequested);
    }
    let (deployment_identity, live_status) = run_live(request, case, &execution, services)?;
    let gate_evidence = services.verifier.verify(case, request.trial, &execution)?;
    let judge_evidence = if case.definition.rubric.is_some() {
        Some(services.judge.judge(case, request.trial, &execution)?)
    } else {
        None
    };
    let oracle_decision = super::oracles::reduce(case, &gate_evidence, judge_evidence.as_ref())?;
    let status = combine_status(oracle_decision.status, live_status)?;
    let candidates = services.artifact.collect(case, request.trial, &execution)?;
    let artifact_manifest = super::artifacts::admit(case, &candidates)?;
    services.artifact.persist(case, request.trial, &artifact_manifest)?;
    let post_state_identity = services.workspace.capture_post_state(case, request.trial)?;
    super::evidence::build_receipt(&super::evidence::ReceiptInput {
        catalog_identity: &request.catalog.identity,
        case,
        trial: request.trial,
        pre_state_identity,
        post_state_identity: &post_state_identity,
        policy_identity: request.policy_identity,
        oracles: &oracle_decision,
        artifacts: &artifact_manifest,
        deployment_identity: deployment_identity.as_deref(),
        status,
        animus_fact: execution.animus_fact.as_ref(),
    })
}

fn run_live(
    request: &RunRequest<'_>,
    case: &super::model::AdmittedCase,
    execution: &ExecutionFact,
    services: &Services<'_>,
) -> Result<(Option<String>, Option<super::model::TrialStatus>), super::Error> {
    if !case.definition.live.enabled {
        if request.live_authority.is_some() {
            return Err(super::Error::AuthorityDenied);
        }
        return Ok((None, None));
    }
    let authority = request.live_authority.ok_or(super::Error::AuthorityDenied)?;
    let admission = super::live::admit(case, authority)?;
    let mut observation = services.deployment.deploy(case, &admission, request.trial, execution)?;
    if case.definition.live.cleanup_required {
        observation.cleanup_confirmed = services.cleanup.cleanup(case, request.trial, &admission, &observation)?;
    }
    let deployment_identity = observation.deployment_identity.clone();
    let status = super::live::reduce(&admission, &observation)?;
    Ok((deployment_identity, Some(status)))
}

fn combine_status(
    oracle: super::model::TrialStatus,
    live: Option<super::model::TrialStatus>,
) -> Result<super::model::TrialStatus, super::Error> {
    match live {
        None | Some(super::model::TrialStatus::Passed) => Ok(oracle),
        Some(super::model::TrialStatus::Failed) => Ok(super::model::TrialStatus::Failed),
        Some(super::model::TrialStatus::Cancelled) => Ok(super::model::TrialStatus::Cancelled),
        Some(super::model::TrialStatus::Partial) => Ok(super::model::TrialStatus::Partial),
        Some(super::model::TrialStatus::CleanupRequired) => Ok(super::model::TrialStatus::CleanupRequired),
        Some(super::model::TrialStatus::Unknown) => Ok(super::model::TrialStatus::Unknown),
        Some(super::model::TrialStatus::Planned | super::model::TrialStatus::Started) => {
            Err(super::Error::LiveOutcomeUnknown)
        }
    }
}
