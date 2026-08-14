fn invocation<'a>(
    operation: super::protocol::Operation,
    case: &'a crate::agent_evaluation::model::AdmittedCase,
    trial: &'a crate::agent_evaluation::planning::TrialPlan,
) -> super::Invocation<'a> {
    super::Invocation {
        operation,
        case,
        trial,
        execution_output_identity: None,
        live_admission: None,
        live_observation: None,
        artifact_manifest: None,
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::WorkspacePort for super::Driver<P> {
    fn prepare(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
    ) -> Result<String, crate::agent_evaluation::Error> {
        self.invoke(invocation(super::protocol::Operation::WorkspacePrepare, case, trial))?.into_identity()
    }

    fn capture_post_state(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
    ) -> Result<String, crate::agent_evaluation::Error> {
        self.invoke(invocation(super::protocol::Operation::CapturePostState, case, trial))?.into_identity()
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::AgentPort for super::Driver<P> {
    fn execute(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
    ) -> Result<crate::agent_evaluation::shell::ExecutionFact, crate::agent_evaluation::Error> {
        self.invoke(invocation(super::protocol::Operation::AgentExecute, case, trial))?.into_execution()
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::WorkflowPort for super::Driver<P> {
    fn execute(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
    ) -> Result<crate::agent_evaluation::shell::ExecutionFact, crate::agent_evaluation::Error> {
        self.invoke(invocation(super::protocol::Operation::WorkflowExecute, case, trial))?.into_execution()
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::VerifierPort for super::Driver<P> {
    fn verify(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        execution: &crate::agent_evaluation::shell::ExecutionFact,
    ) -> Result<Vec<crate::agent_evaluation::oracles::GateEvidence>, crate::agent_evaluation::Error> {
        let mut request = invocation(super::protocol::Operation::Verify, case, trial);
        request.execution_output_identity = Some(&execution.output_identity);
        self.invoke(request)?.into_gates()
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::JudgePort for super::Driver<P> {
    fn judge(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        execution: &crate::agent_evaluation::shell::ExecutionFact,
    ) -> Result<crate::agent_evaluation::oracles::JudgeEvidence, crate::agent_evaluation::Error> {
        let mut request = invocation(super::protocol::Operation::Judge, case, trial);
        request.execution_output_identity = Some(&execution.output_identity);
        self.invoke(request)?.into_judgment()
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::DeploymentPort for super::Driver<P> {
    fn deploy(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        admission: &crate::agent_evaluation::live::Admission,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        execution: &crate::agent_evaluation::shell::ExecutionFact,
    ) -> Result<crate::agent_evaluation::live::Observation, crate::agent_evaluation::Error> {
        let mut request = invocation(super::protocol::Operation::Deploy, case, trial);
        request.execution_output_identity = Some(&execution.output_identity);
        request.live_admission = Some(admission);
        self.invoke(request)?.into_deployment()
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::CleanupPort for super::Driver<P> {
    fn cleanup(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        admission: &crate::agent_evaluation::live::Admission,
        observation: &crate::agent_evaluation::live::Observation,
    ) -> Result<bool, crate::agent_evaluation::Error> {
        let mut request = invocation(super::protocol::Operation::Cleanup, case, trial);
        request.live_admission = Some(admission);
        request.live_observation = Some(observation);
        self.invoke(request)?.into_cleanup()
    }
}

impl<P: super::DriverProcessPort> crate::agent_evaluation::shell::ArtifactPort for super::Driver<P> {
    fn collect(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        execution: &crate::agent_evaluation::shell::ExecutionFact,
    ) -> Result<Vec<crate::agent_evaluation::artifacts::ArtifactCandidate>, crate::agent_evaluation::Error> {
        let mut request = invocation(super::protocol::Operation::CollectArtifacts, case, trial);
        request.execution_output_identity = Some(&execution.output_identity);
        self.invoke(request)?.into_artifacts()
    }

    fn persist(
        &self,
        case: &crate::agent_evaluation::model::AdmittedCase,
        trial: &crate::agent_evaluation::planning::TrialPlan,
        manifest: &crate::agent_evaluation::artifacts::ArtifactManifest,
    ) -> Result<(), crate::agent_evaluation::Error> {
        let mut request = invocation(super::protocol::Operation::PersistArtifacts, case, trial);
        request.artifact_manifest = Some(manifest);
        self.invoke(request)?.into_persisted()
    }
}

impl<P> crate::agent_evaluation::shell::ClockPort for super::Driver<P> {
    fn observed_epoch_ms(&self) -> Result<u64, crate::agent_evaluation::Error> {
        self.clock_observations
            .borrow_mut()
            .pop_front()
            .ok_or(crate::agent_evaluation::Error::ServiceUnavailable)
    }
}
