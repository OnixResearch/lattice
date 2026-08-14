//! Product-owned data model for durable workflow approval gates.

pub const CHOREGRAPH_SUSPENSION_PROFILE: &str = "choregraph.lifecycle.durable-suspension.v1";
pub const CHOREGRAPH_SUSPENSION_CLASS: &str = "lattice/workflow-approval";
pub const APPROVAL_NON_CLAIM_COUNT: usize = 6;
pub const REQUIRED_APPROVAL_NON_CLAIMS: [&str; APPROVAL_NON_CLAIM_COUNT] = [
    "human identity is not proven",
    "independent review is not proven",
    "Basalt or UCAN verifier correctness is not proven",
    "persistence under arbitrary host loss is not proven",
    "downstream effects are not proven exactly once",
    "production readiness and release eligibility are not proven",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionClass {
    Approve,
    Reject,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize
)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAbility {
    Approve,
    Reject,
}

impl DecisionAbility {
    #[must_use]
    pub const fn for_decision(decision: DecisionClass) -> Self {
        match decision {
            DecisionClass::Approve => Self::Approve,
            DecisionClass::Reject => Self::Reject,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityProfile {
    LocalOperator,
    BasaltUcanVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalAuthorityPolicy {
    pub profile: AuthorityProfile,
    pub local_actors: Vec<crate::workflow::gate::identity::ActorId>,
    pub accepted_abilities: Vec<DecisionAbility>,
    pub prohibit_self_approval: bool,
    pub delegated_policy_reference: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClockSourceClass {
    UtcWallClock,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalExpiryPolicy {
    pub decision_ttl_millis: u64,
    pub resume_ttl_millis: u64,
    pub approval_must_be_fresh_at_resume: bool,
    pub clock_source: ClockSourceClass,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalReasonPolicy {
    pub required: bool,
    pub identity_bound: bool,
    pub max_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResumeValueField {
    pub name: crate::workflow::gate::identity::ResumeBindingName,
    pub type_tag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalResumeSchema {
    pub values: Vec<ResumeValueField>,
    pub credential_bindings: Vec<crate::workflow::gate::identity::ResumeBindingName>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminalOutcome {
    FailWorkflow,
    CancelWorkflow,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalTerminalPolicy {
    pub on_rejection: TerminalOutcome,
    pub on_expiry: TerminalOutcome,
}

// r[impl lattice.workflow_approval.node]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApprovalNodeConfig {
    pub gate_id: crate::workflow::gate::identity::GateId,
    pub safe_summary: String,
    pub selected_input_ports: Vec<String>,
    pub required_artifacts: Vec<crate::workflow::gate::identity::ApprovalArtifactId>,
    pub authority: ApprovalAuthorityPolicy,
    pub expiry: ApprovalExpiryPolicy,
    pub reason: ApprovalReasonPolicy,
    pub resume: ApprovalResumeSchema,
    pub terminal: ApprovalTerminalPolicy,
    pub required_non_claims: Vec<String>,
}
