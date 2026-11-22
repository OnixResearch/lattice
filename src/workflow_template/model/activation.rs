#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationFacts {
    pub explicit_save_requested: bool,
    pub explicit_start_requested: bool,
    pub saved_workflow_revision: Option<crate::orchestration::WorkflowRevisionRef>,
    pub current_orchestration_revision: Option<crate::orchestration::DefinitionRevision>,
    pub runtime_admitted: bool,
    pub effects_admitted: bool,
    pub credentials_admitted: bool,
    pub triggers_admitted: bool,
    pub positive_budgets_admitted: bool,
    pub actor_authority_admitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationPlan {
    PreviewOnly,
    SaveWorkflow {
        bundle: ContentIdentity,
        workflow: crate::orchestration::WorkflowRevisionRef,
    },
    StartOrchestration {
        bundle: ContentIdentity,
        workflow: crate::orchestration::WorkflowRevisionRef,
        orchestration: crate::orchestration::DefinitionRevision,
    },
    Blocked(ActivationBlock),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ActivationReceipt {
    pub identity: ContentIdentity,
    pub bundle: ContentIdentity,
    pub workflow: crate::orchestration::WorkflowRevisionRef,
    pub orchestration: crate::orchestration::DefinitionRevision,
    pub authority_policy: crate::orchestration::AuthorityPolicyIdentity,
    pub actor: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationBlock {
    ParamsUnbound,
    ModeViolation,
    WorkflowNotSaved,
    WorkflowDrift,
    OrchestrationMissing,
    OrchestrationDrift,
    RuntimeNotAdmitted,
    EffectsNotAdmitted,
    CredentialsNotAdmitted,
    TriggersNotAdmitted,
    BudgetsNotAdmitted,
    AuthorityNotAdmitted,
}
