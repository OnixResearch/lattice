const REQUEST_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.request.v1";
const DECISION_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.decision.v1";
const AUTHORITY_INPUT_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.authority-input.v1";
const RESUME_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.resume.v1";
const EVENT_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.event.v1";
const STATE_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.state.v1";
const POLICY_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.authority-policy.v1";
const EXPIRY_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.expiry-policy.v1";
const RESUME_SCHEMA_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.resume-schema.v1";
const REASON_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.reason.v1";
const RECEIPT_HASH_DOMAIN: &[u8] = b"lattice.workflow-approval.receipt.v1";
const HASH_FRAME_SEPARATOR: &[u8] = b"\0";
const INITIAL_EVENT_SEQUENCE: u64 = 0;
const EVENT_SEQUENCE_INCREMENT: u64 = 1;
const EVENT_COUNT_INCREMENT: usize = 1;
const REQUEST_VERSION_INITIAL: u64 = 1;
const REQUEST_VERSION_INCREMENT: u64 = 1;
const MAX_SAFE_SUMMARY_BYTES: usize = 512;
const MAX_REASON_POLICY_BYTES: usize = 4_096;
const MAX_POLICY_ACTORS: usize = 128;
const MAX_SELECTED_INPUTS: usize = 128;
const MAX_REQUIRED_ARTIFACTS: usize = 128;
const MAX_RESUME_FIELDS: usize = 128;
const MAX_NON_CLAIMS: usize = 32;
const MAX_REFERENCE_BYTES: usize = 512;
const MAX_CHECKPOINT_NODES: usize = 4_096;
const MAX_GRAPH_EVENTS: usize = 12_288;
const MAX_REQUEST_EVENTS: usize = 5;
const MIN_TTL_MILLIS: u64 = 1;
const MILLIS_PER_SECOND: u64 = 1_000;
const SECONDS_PER_MINUTE: u64 = 60;
const MINUTES_PER_HOUR: u64 = 60;
const HOURS_PER_DAY: u64 = 24;
const DAYS_PER_YEAR: u64 = 366;
const MAX_TTL_MILLIS: u64 = MILLIS_PER_SECOND
    .saturating_mul(SECONDS_PER_MINUTE)
    .saturating_mul(MINUTES_PER_HOUR)
    .saturating_mul(HOURS_PER_DAY)
    .saturating_mul(DAYS_PER_YEAR);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationCode {
    UnsafeSummary,
    DuplicateValue,
    TooManyValues,
    MissingAuthority,
    InvalidExpiry,
    InvalidReasonPolicy,
    UnsafeResumeSchema,
    MissingNonClaim,
    InvalidReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub code: ValidationCode,
    pub message: String,
}

struct IssueSink<'a> {
    values: &'a mut Vec<ValidationIssue>,
}

impl IssueSink<'_> {
    fn push(&mut self, issue: ValidationIssue) {
        self.values.push(issue);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionCode {
    RequestMismatch,
    WorkflowRevisionMismatch,
    WrongStatus,
    AuthorityDenied,
    Expired,
    ReasonDenied,
    ConflictingDecision,
    ResumeSchemaMismatch,
    PlaintextSecretDenied,
    CredentialDenied,
    EventHeadMismatch,
    ActiveOwner,
    ResumeUncertain,
    IdentityFailure,
    TimeOverflow,
    UnsafeCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionError {
    pub code: AdmissionCode,
    pub message: String,
}

impl std::fmt::Display for AdmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AdmissionError {}

#[derive(Debug, Clone)]
pub struct RequestBuildInput<'a> {
    pub run_id: crate::engine::executor::RunId,
    pub workflow: &'a crate::engine::workflow::Workflow,
    pub node_id: crate::engine::workflow::NodeId,
    pub requester: crate::workflow::gate::identity::ActorId,
    pub observed_epoch_millis: i64,
    pub gate_inputs: &'a std::collections::HashMap<crate::engine::workflow::PortId, crate::engine::value::Value>,
    pub completed_nodes: Vec<crate::workflow::gate::model::PersistedNodeOutcome>,
    pub graph_events: Vec<crate::workflow::gate::model::ApprovalGraphEvent>,
}

#[derive(Debug, Clone)]
pub struct DecisionPlan {
    pub request: crate::workflow::gate::model::ApprovalRequest,
    pub outcome: crate::workflow::gate::model::DecisionOutcome,
}

#[derive(Debug, Clone)]
pub struct ResumeClaimPlan {
    pub request: crate::workflow::gate::model::ApprovalRequest,
    pub plan: crate::workflow::gate::model::ResumePlan,
    pub outcome: crate::workflow::gate::model::ResumeOutcome,
}

#[derive(serde::Serialize)]
struct RequestIdentityMaterial<'a> {
    run_id: crate::engine::executor::RunId,
    workflow_name: &'a str,
    workflow_revision: &'a crate::orchestration::WorkflowRevision,
    node_id: crate::engine::workflow::NodeId,
    gate_id: &'a crate::workflow::gate::identity::GateId,
    requester: &'a crate::workflow::gate::identity::ActorId,
    selected_inputs: &'a [crate::workflow::gate::model::SelectedInputFact],
    required_artifacts: &'a [crate::workflow::gate::identity::ApprovalArtifactId],
    authority_policy_id: &'a crate::workflow::gate::identity::ApprovalAuthorityPolicyId,
    expiry_policy_id: &'a crate::workflow::gate::identity::ApprovalExpiryPolicyId,
    resume_schema_id: &'a crate::workflow::gate::identity::ApprovalResumeSchemaId,
    requested_epoch_millis: i64,
    decision_deadline_epoch_millis: i64,
}

#[derive(serde::Serialize)]
struct AuthorityInputMaterial<'a> {
    request_id: &'a crate::workflow::gate::identity::ApprovalRequestId,
    workflow_revision: &'a crate::orchestration::WorkflowRevision,
    node_id: crate::engine::workflow::NodeId,
    actor: &'a crate::workflow::gate::identity::ActorId,
    ability: crate::workflow::gate::model::DecisionAbility,
    authority_policy_id: &'a crate::workflow::gate::identity::ApprovalAuthorityPolicyId,
    observed_epoch_millis: i64,
    profile: crate::workflow::gate::model::AuthorityProfile,
    verification_receipt_references: &'a [String],
}

#[derive(serde::Serialize)]
struct DecisionIdentityMaterial<'a> {
    request_id: &'a crate::workflow::gate::identity::ApprovalRequestId,
    workflow_revision: &'a crate::orchestration::WorkflowRevision,
    actor: &'a crate::workflow::gate::identity::ActorId,
    decision: crate::workflow::gate::model::DecisionClass,
    ability: crate::workflow::gate::model::DecisionAbility,
    reason_identity: &'a Option<String>,
    decided_epoch_millis: i64,
    authority_input_id: &'a crate::workflow::gate::identity::ApprovalAuthorityInputId,
}

#[derive(serde::Serialize)]
struct ReceiptIdentityMaterial<'a> {
    request_id: &'a crate::workflow::gate::identity::ApprovalRequestId,
    run_id: crate::engine::executor::RunId,
    workflow_revision: &'a crate::orchestration::WorkflowRevision,
    node_id: crate::engine::workflow::NodeId,
    gate_id: &'a crate::workflow::gate::identity::GateId,
    status: crate::workflow::gate::model::ApprovalRequestStatus,
    authority_profile: crate::workflow::gate::model::AuthorityProfile,
    authority_policy_id: &'a crate::workflow::gate::identity::ApprovalAuthorityPolicyId,
    expiry_policy_id: &'a crate::workflow::gate::identity::ApprovalExpiryPolicyId,
    delegated_policy_reference: &'a Option<String>,
    verification_receipt_references: &'a [String],
    clock_source: crate::workflow::gate::model::ClockSourceClass,
    event_head: &'a crate::workflow::gate::identity::ApprovalEventId,
    event_count: usize,
    choregraph_profile: &'a str,
    suspension_class: &'a str,
    required_non_claims: &'a [String],
}

#[derive(serde::Serialize)]
struct ResumeIdentityMaterial<'a> {
    request_id: &'a crate::workflow::gate::identity::ApprovalRequestId,
    run_id: crate::engine::executor::RunId,
    workflow_revision: &'a crate::orchestration::WorkflowRevision,
    node_id: crate::engine::workflow::NodeId,
    owner: &'a crate::workflow::gate::identity::ResumeOwnerId,
    values: &'a [crate::workflow::gate::model::ResumeValueIdentity],
    credentials: &'a [crate::workflow::gate::model::ResumeCredentialIdentity],
}

#[derive(serde::Serialize)]
struct StateIdentityMaterial<'a> {
    request_id: &'a crate::workflow::gate::identity::ApprovalRequestId,
    status: crate::workflow::gate::model::ApprovalRequestStatus,
    decision_id: Option<&'a crate::workflow::gate::identity::ApprovalDecisionId>,
    active_resume_owner: Option<&'a crate::workflow::gate::identity::ResumeOwnerId>,
    completed_resume_id: Option<&'a crate::workflow::gate::identity::ApprovalResumeId>,
    completed_nodes: &'a [crate::workflow::gate::model::PersistedNodeOutcome],
    graph_events: &'a [crate::workflow::gate::model::ApprovalGraphEvent],
    version: u64,
}

#[derive(serde::Serialize)]
struct EventIdentityMaterial<'a> {
    request_id: &'a crate::workflow::gate::identity::ApprovalRequestId,
    previous: Option<&'a crate::workflow::gate::identity::ApprovalEventId>,
    sequence: u64,
    kind: crate::workflow::gate::model::ApprovalEventKind,
    state_digest: &'a crate::workflow::gate::identity::ApprovalStateDigest,
}
