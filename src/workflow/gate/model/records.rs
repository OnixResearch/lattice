//! Persisted approval state and commands.

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedInputFact {
    pub port_id: crate::engine::workflow::PortId,
    pub port_label: String,
    pub value_hash: crate::workflow::gate::identity::ApprovalInputId,
    pub type_tag: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistedNodeOutcome {
    pub node_id: crate::engine::workflow::NodeId,
    pub state: PersistedNodeState,
    pub input_hash: String,
    pub outputs: std::collections::BTreeMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PersistedNodeState {
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalGraphEvent {
    pub node_id: crate::engine::workflow::NodeId,
    pub action: ApprovalGraphAction,
    pub suspension_class: Option<String>,
    pub suspension_reference: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalGraphAction {
    MarkReady,
    Start,
    Complete,
    Fail,
    Cancel,
    Suspend,
    ResumeSignal,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "profile")]
pub enum DecisionAuthorityEvidence {
    LocalOperator,
    BasaltUcanVerified(VerifiedDelegatedAuthorityFacts),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerifiedDelegatedAuthorityFacts {
    pub request_id: crate::workflow::gate::identity::ApprovalRequestId,
    pub workflow_revision: crate::orchestration::WorkflowRevision,
    pub node_id: crate::engine::workflow::NodeId,
    pub actor: crate::workflow::gate::identity::ActorId,
    pub ability: super::declaration::DecisionAbility,
    pub policy_reference: String,
    pub verification_receipt_references: Vec<String>,
    pub caveats_allow: bool,
    pub replay_allow: bool,
    pub current: bool,
    pub input_id: crate::workflow::gate::identity::ApprovalAuthorityInputId,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionCommand {
    pub request_id: crate::workflow::gate::identity::ApprovalRequestId,
    pub expected_workflow_revision: crate::orchestration::WorkflowRevision,
    pub actor: crate::workflow::gate::identity::ActorId,
    pub decision: super::declaration::DecisionClass,
    pub reason: Option<String>,
    pub observed_epoch_millis: i64,
    pub evidence: DecisionAuthorityEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalDecision {
    pub id: crate::workflow::gate::identity::ApprovalDecisionId,
    pub actor: crate::workflow::gate::identity::ActorId,
    pub decision: super::declaration::DecisionClass,
    pub ability: super::declaration::DecisionAbility,
    pub reason_identity: Option<String>,
    pub decided_epoch_millis: i64,
    pub authority_input_id: crate::workflow::gate::identity::ApprovalAuthorityInputId,
    pub authority_profile: super::declaration::AuthorityProfile,
    pub delegated_policy_reference: Option<String>,
    pub verification_receipt_references: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeValue {
    pub name: crate::workflow::gate::identity::ResumeBindingName,
    pub value: crate::engine::value::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeCredentialBinding {
    pub name: crate::workflow::gate::identity::ResumeBindingName,
    pub reference: crate::credentials::SecretRef,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeCommand {
    pub request_id: crate::workflow::gate::identity::ApprovalRequestId,
    pub expected_workflow_revision: crate::orchestration::WorkflowRevision,
    pub expected_event_head: crate::workflow::gate::identity::ApprovalEventId,
    pub owner: crate::workflow::gate::identity::ResumeOwnerId,
    pub observed_epoch_millis: i64,
    pub values: Vec<ResumeValue>,
    pub credentials: Vec<ResumeCredentialBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalEvent {
    pub id: crate::workflow::gate::identity::ApprovalEventId,
    pub previous: Option<crate::workflow::gate::identity::ApprovalEventId>,
    pub sequence: u64,
    pub kind: ApprovalEventKind,
    pub state_digest: crate::workflow::gate::identity::ApprovalStateDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalEventKind {
    Suspended,
    Approved,
    Rejected,
    Expired,
    ResumeClaimed,
    ResumeSignalled,
    Resumed,
    ResumeUncertain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRequestStatus {
    Suspended,
    Approved,
    Rejected,
    Expired,
    Resuming,
    Resumed,
    ResumeUncertain,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalRequest {
    pub id: crate::workflow::gate::identity::ApprovalRequestId,
    pub run_id: crate::engine::executor::RunId,
    pub workflow_name: String,
    pub workflow_revision: crate::orchestration::WorkflowRevision,
    pub node_id: crate::engine::workflow::NodeId,
    pub gate_id: crate::workflow::gate::identity::GateId,
    pub requester: crate::workflow::gate::identity::ActorId,
    pub safe_summary: String,
    pub selected_inputs: Vec<SelectedInputFact>,
    pub required_artifacts: Vec<crate::workflow::gate::identity::ApprovalArtifactId>,
    pub authority_policy_id: crate::workflow::gate::identity::ApprovalAuthorityPolicyId,
    pub expiry_policy_id: crate::workflow::gate::identity::ApprovalExpiryPolicyId,
    pub resume_schema_id: crate::workflow::gate::identity::ApprovalResumeSchemaId,
    pub required_non_claims: Vec<String>,
    pub requested_epoch_millis: i64,
    pub decision_deadline_epoch_millis: i64,
    pub status: ApprovalRequestStatus,
    pub decision: Option<ApprovalDecision>,
    pub active_resume_owner: Option<crate::workflow::gate::identity::ResumeOwnerId>,
    pub completed_resume_id: Option<crate::workflow::gate::identity::ApprovalResumeId>,
    pub completed_nodes: Vec<PersistedNodeOutcome>,
    pub graph_events: Vec<ApprovalGraphEvent>,
    pub events: Vec<ApprovalEvent>,
    pub choregraph_profile: String,
    pub suspension_class: String,
    pub suspension_reference: String,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalReceipt {
    pub id: crate::workflow::gate::identity::ApprovalReceiptId,
    pub request_id: crate::workflow::gate::identity::ApprovalRequestId,
    pub run_id: crate::engine::executor::RunId,
    pub workflow_revision: crate::orchestration::WorkflowRevision,
    pub node_id: crate::engine::workflow::NodeId,
    pub gate_id: crate::workflow::gate::identity::GateId,
    pub status: ApprovalRequestStatus,
    pub authority_profile: super::declaration::AuthorityProfile,
    pub authority_policy_id: crate::workflow::gate::identity::ApprovalAuthorityPolicyId,
    pub expiry_policy_id: crate::workflow::gate::identity::ApprovalExpiryPolicyId,
    pub delegated_policy_reference: Option<String>,
    pub verification_receipt_references: Vec<String>,
    pub clock_source: super::declaration::ClockSourceClass,
    pub event_head: crate::workflow::gate::identity::ApprovalEventId,
    pub event_count: usize,
    pub choregraph_profile: String,
    pub suspension_class: String,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumePlan {
    pub id: crate::workflow::gate::identity::ApprovalResumeId,
    pub request_id: crate::workflow::gate::identity::ApprovalRequestId,
    pub run_id: crate::engine::executor::RunId,
    pub workflow_revision: crate::orchestration::WorkflowRevision,
    pub node_id: crate::engine::workflow::NodeId,
    pub owner: crate::workflow::gate::identity::ResumeOwnerId,
    pub values: Vec<ResumeValueIdentity>,
    pub credentials: Vec<ResumeCredentialIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeValueIdentity {
    pub name: crate::workflow::gate::identity::ResumeBindingName,
    pub type_tag: String,
    pub value_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeCredentialIdentity {
    pub name: crate::workflow::gate::identity::ResumeBindingName,
    pub reference_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum DecisionOutcome {
    Recorded {
        decision_id: crate::workflow::gate::identity::ApprovalDecisionId,
    },
    Idempotent {
        decision_id: crate::workflow::gate::identity::ApprovalDecisionId,
    },
    Expired {
        terminal: super::declaration::TerminalOutcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum ResumeOutcome {
    Claimed {
        resume_id: crate::workflow::gate::identity::ApprovalResumeId,
    },
    Idempotent {
        resume_id: crate::workflow::gate::identity::ApprovalResumeId,
    },
}
