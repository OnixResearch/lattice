pub(super) const REQUEST_SCHEMA: &str = "lattice.agent-evaluation-driver-request.v1";
pub(super) const RESPONSE_SCHEMA: &str = "lattice.agent-evaluation-driver-response.v1";

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Operation {
    WorkspacePrepare,
    AgentExecute,
    WorkflowExecute,
    Deploy,
    Verify,
    Judge,
    CollectArtifacts,
    PersistArtifacts,
    CapturePostState,
    Cleanup,
}

impl Operation {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::WorkspacePrepare => "workspace_prepare",
            Self::AgentExecute => "agent_execute",
            Self::WorkflowExecute => "workflow_execute",
            Self::Deploy => "deploy",
            Self::Verify => "verify",
            Self::Judge => "judge",
            Self::CollectArtifacts => "collect_artifacts",
            Self::PersistArtifacts => "persist_artifacts",
            Self::CapturePostState => "capture_post_state",
            Self::Cleanup => "cleanup",
        }
    }
}

#[derive(serde::Serialize)]
pub(super) struct Request<'a> {
    pub schema: &'static str,
    pub driver_identity: &'a str,
    pub operation: Operation,
    pub case_definition: &'a crate::agent_evaluation::model::CaseDefinition,
    pub case_revision_identity: &'a str,
    pub trial: &'a crate::agent_evaluation::planning::TrialPlan,
    pub execution_output_identity: Option<&'a str>,
    pub live_admission: Option<&'a crate::agent_evaluation::live::Admission>,
    pub live_observation: Option<&'a crate::agent_evaluation::live::Observation>,
    pub artifact_manifest: Option<&'a crate::agent_evaluation::artifacts::ArtifactManifest>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Response {
    pub schema: String,
    pub driver_identity: String,
    pub trial_identity: String,
    pub result: ResultValue,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub(super) enum ResultValue {
    Identity {
        identity: String,
    },
    Execution {
        output_identity: String,
        animus_fact: Option<crate::agent_evaluation::evidence::AnimusFact>,
    },
    Gates {
        evidence: Vec<crate::agent_evaluation::oracles::GateEvidence>,
    },
    Judgment {
        evidence: crate::agent_evaluation::oracles::JudgeEvidence,
    },
    Deployment {
        observation: crate::agent_evaluation::live::Observation,
    },
    Cleanup {
        confirmed: bool,
    },
    Artifacts {
        candidates: Vec<crate::agent_evaluation::artifacts::ArtifactCandidate>,
    },
    Persisted,
}

impl ResultValue {
    pub(super) fn into_identity(self) -> Result<String, crate::agent_evaluation::Error> {
        match self {
            Self::Identity { identity } => Ok(identity),
            Self::Execution { .. }
            | Self::Gates { .. }
            | Self::Judgment { .. }
            | Self::Deployment { .. }
            | Self::Cleanup { .. }
            | Self::Artifacts { .. }
            | Self::Persisted => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }

    pub(super) fn into_execution(
        self,
    ) -> Result<crate::agent_evaluation::shell::ExecutionFact, crate::agent_evaluation::Error> {
        match self {
            Self::Execution {
                output_identity,
                animus_fact,
            } => Ok(crate::agent_evaluation::shell::ExecutionFact {
                output_identity,
                animus_fact,
            }),
            Self::Identity { .. }
            | Self::Gates { .. }
            | Self::Judgment { .. }
            | Self::Deployment { .. }
            | Self::Cleanup { .. }
            | Self::Artifacts { .. }
            | Self::Persisted => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }

    pub(super) fn into_gates(
        self,
    ) -> Result<Vec<crate::agent_evaluation::oracles::GateEvidence>, crate::agent_evaluation::Error> {
        match self {
            Self::Gates { evidence } => Ok(evidence),
            Self::Identity { .. }
            | Self::Execution { .. }
            | Self::Judgment { .. }
            | Self::Deployment { .. }
            | Self::Cleanup { .. }
            | Self::Artifacts { .. }
            | Self::Persisted => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }

    pub(super) fn into_judgment(
        self,
    ) -> Result<crate::agent_evaluation::oracles::JudgeEvidence, crate::agent_evaluation::Error> {
        match self {
            Self::Judgment { evidence } => Ok(evidence),
            Self::Identity { .. }
            | Self::Execution { .. }
            | Self::Gates { .. }
            | Self::Deployment { .. }
            | Self::Cleanup { .. }
            | Self::Artifacts { .. }
            | Self::Persisted => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }

    pub(super) fn into_deployment(
        self,
    ) -> Result<crate::agent_evaluation::live::Observation, crate::agent_evaluation::Error> {
        match self {
            Self::Deployment { observation } => Ok(observation),
            Self::Identity { .. }
            | Self::Execution { .. }
            | Self::Gates { .. }
            | Self::Judgment { .. }
            | Self::Cleanup { .. }
            | Self::Artifacts { .. }
            | Self::Persisted => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }

    pub(super) fn into_cleanup(self) -> Result<bool, crate::agent_evaluation::Error> {
        match self {
            Self::Cleanup { confirmed } => Ok(confirmed),
            Self::Identity { .. }
            | Self::Execution { .. }
            | Self::Gates { .. }
            | Self::Judgment { .. }
            | Self::Deployment { .. }
            | Self::Artifacts { .. }
            | Self::Persisted => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }

    pub(super) fn into_artifacts(
        self,
    ) -> Result<Vec<crate::agent_evaluation::artifacts::ArtifactCandidate>, crate::agent_evaluation::Error> {
        match self {
            Self::Artifacts { candidates } => Ok(candidates),
            Self::Identity { .. }
            | Self::Execution { .. }
            | Self::Gates { .. }
            | Self::Judgment { .. }
            | Self::Deployment { .. }
            | Self::Cleanup { .. }
            | Self::Persisted => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }

    pub(super) fn into_persisted(self) -> Result<(), crate::agent_evaluation::Error> {
        match self {
            Self::Persisted => Ok(()),
            Self::Identity { .. }
            | Self::Execution { .. }
            | Self::Gates { .. }
            | Self::Judgment { .. }
            | Self::Deployment { .. }
            | Self::Cleanup { .. }
            | Self::Artifacts { .. } => Err(crate::agent_evaluation::Error::ServiceFailure),
        }
    }
}
