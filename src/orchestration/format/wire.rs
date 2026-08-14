//! String-shaped compatibility records for orchestration formats.

// r[impl lattice.agent_orchestration.nominal_identities.wire_boundary]
// r[impl lattice.agent_orchestration.nominal_identities.compatibility]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RevisionRecord {
    workflow_name: String,
    content_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TriggerRecord {
    trigger_id: String,
    kind: super::super::core::OrchestrationTriggerKind,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DefinitionRecord {
    schema_version: u32,
    id: String,
    objective: String,
    workflow: RevisionRecord,
    completion: super::super::core::CompletionPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry: Option<super::super::core::RetryPolicy>,
    budgets: super::super::core::BudgetPolicy,
    authority: super::super::core::AuthorityPolicy,
    triggers: Vec<TriggerRecord>,
    redaction: super::super::core::RedactionPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProposalRecord {
    id: String,
    instance_id: String,
    base_definition_revision: String,
    base_epoch: u64,
    base_workflow_revision: RevisionRecord,
    successor: RevisionRecord,
    safe_summary: String,
}

impl From<&super::super::core::WorkflowRevisionRef> for RevisionRecord {
    fn from(revision: &super::super::core::WorkflowRevisionRef) -> Self {
        Self {
            workflow_name: revision.workflow_name.clone(),
            content_hash: revision.content_hash.to_string(),
        }
    }
}

impl TryFrom<RevisionRecord> for super::super::core::WorkflowRevisionRef {
    type Error = super::OrchestrationFormatError;

    fn try_from(revision: RevisionRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            workflow_name: revision.workflow_name,
            content_hash: super::super::identity::WorkflowRevision::try_from_hex(revision.content_hash)
                .map_err(identity_admission_error)?,
        })
    }
}

impl From<&super::super::core::OrchestrationDefinition> for DefinitionRecord {
    fn from(definition: &super::super::core::OrchestrationDefinition) -> Self {
        Self {
            schema_version: definition.schema_version,
            id: definition.id.to_string(),
            objective: definition.objective.clone(),
            workflow: RevisionRecord::from(&definition.workflow),
            completion: definition.completion,
            retry: definition.retry,
            budgets: definition.budgets,
            authority: definition.authority.clone(),
            triggers: definition
                .triggers
                .iter()
                .map(|binding| TriggerRecord {
                    trigger_id: binding.trigger_id.to_string(),
                    kind: binding.kind,
                })
                .collect(),
            redaction: definition.redaction,
        }
    }
}

impl TryFrom<DefinitionRecord> for super::super::core::OrchestrationDefinition {
    type Error = super::OrchestrationFormatError;

    fn try_from(definition: DefinitionRecord) -> Result<Self, Self::Error> {
        let triggers = definition
            .triggers
            .into_iter()
            .map(|binding| {
                Ok(super::super::core::OrchestrationTriggerBinding {
                    trigger_id: super::super::identity::TriggerId::try_from_text(binding.trigger_id)
                        .map_err(identity_admission_error)?,
                    kind: binding.kind,
                })
            })
            .collect::<Result<Vec<_>, super::OrchestrationFormatError>>()?;
        Ok(Self {
            schema_version: definition.schema_version,
            id: super::super::identity::DefinitionId::try_from_text(definition.id).map_err(identity_admission_error)?,
            objective: definition.objective,
            workflow: definition.workflow.try_into()?,
            completion: definition.completion,
            retry: definition.retry,
            budgets: definition.budgets,
            authority: definition.authority,
            triggers,
            redaction: definition.redaction,
        })
    }
}

impl From<&super::super::core::RevisionProposal> for ProposalRecord {
    fn from(proposal: &super::super::core::RevisionProposal) -> Self {
        Self {
            id: proposal.id.to_string(),
            instance_id: proposal.instance_id.to_string(),
            base_definition_revision: proposal.base_definition_revision.to_string(),
            base_epoch: proposal.base_epoch,
            base_workflow_revision: RevisionRecord::from(&proposal.base_workflow_revision),
            successor: RevisionRecord::from(&proposal.successor),
            safe_summary: proposal.safe_summary.clone(),
        }
    }
}

impl TryFrom<ProposalRecord> for super::super::core::RevisionProposal {
    type Error = super::OrchestrationFormatError;

    fn try_from(proposal: ProposalRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: super::super::identity::ProposalId::try_from_hex(proposal.id).map_err(identity_admission_error)?,
            instance_id: super::super::identity::InstanceId::try_from_hex(proposal.instance_id)
                .map_err(identity_admission_error)?,
            base_definition_revision: super::super::identity::DefinitionRevision::try_from_hex(
                proposal.base_definition_revision,
            )
            .map_err(identity_admission_error)?,
            base_epoch: proposal.base_epoch,
            base_workflow_revision: proposal.base_workflow_revision.try_into()?,
            successor: proposal.successor.try_into()?,
            safe_summary: proposal.safe_summary,
        })
    }
}

fn identity_admission_error(error: super::super::identity::AdmissionError) -> super::OrchestrationFormatError {
    super::OrchestrationFormatError::Json(error.to_string())
}
