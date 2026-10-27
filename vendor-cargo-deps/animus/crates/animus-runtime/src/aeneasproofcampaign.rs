//! Pure typed contracts for bounded Aeneas proof campaigns.
//!
//! This module validates already-loaded campaign metadata. It does not read
//! files, evaluate Nickel, dispatch providers, execute proof tools, access a
//! clock or network, persist artifacts, or decide authoritative completion.
//!
//! Octet-only lint allowances reflect this module's validation-boundary shape:
//! campaign-prefixed public DTOs avoid ambiguous imports, boolean observations
//! retain domain names, untrusted inputs return diagnostics rather than assert,
//! and pure reducers keep all related checks visible in one control-flow path.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::ambiguous_params,
        tigerstyle::assertion_density,
        tigerstyle::bool_naming,
        tigerstyle::compound_condition,
        tigerstyle::function_length,
        tigerstyle::non_trait_imports,
        tigerstyle::numeric_units,
        tigerstyle::path_segment_repetition,
        tigerstyle::raw_arithmetic_overflow
    )
)]

use std::collections::BTreeSet;

// r[impl agent_harness_runtime.aeneas_proof_campaign.profile]
// r[verify agent_harness_runtime.aeneas_proof_campaign.profile]
// r[impl agent_harness_runtime.aeneas_proof_campaign.skill_source]
// r[verify agent_harness_runtime.aeneas_proof_campaign.skill_source]
// r[impl agent_harness_runtime.aeneas_proof_campaign.verification]
// r[verify agent_harness_runtime.aeneas_proof_campaign.verification]

const BLAKE3_HEX_CHARS: usize = 64;
const MAX_SAFE_LABEL_BYTES: usize = 256;
const PROFILE_ID: &str = "aeneas-proof-campaign";
const SKILL_IDENTITY: &str = "f8f4d41384958e468c952289e8a548d78b7f6e69616abedd0ed629dfec327492";
const RECIPE_ID: &str = "goose.recipe.workflow";
const COMPLETION_GATE_ID: &str = "host.authoritative-completion";
const RECEIPT_SCHEMA: &str = "animus.aeneas-proof-campaign-receipt.v1";
const REQUIRED_REDACTIONS: &[&str] = &[
    "raw_prompt",
    "hidden_reasoning",
    "credential",
    "private_path",
    "source_body",
    "proof_body",
    "tool_arguments",
    "tool_output",
    "provider_payload",
    "unbounded_log",
];
const FORBIDDEN_SAFE_TEXT: &[&str] = &[
    "RAW_PROMPT",
    "HIDDEN_REASONING",
    "BEGIN PRIVATE KEY",
    "API_KEY=",
    "/home/",
    "provider_payload",
    "tool_output",
];

/// Campaign validation issue class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CampaignIssue {
    /// A required identifier or BLAKE3 identity is malformed or stale.
    Identity,
    /// A role assignment or capability ceiling is invalid.
    Role,
    /// Required separation dimensions overlap or the subject was mutated.
    Separation,
    /// The theorem statement or declared trust closure is inadequate.
    Statement,
    /// Generated and maintained artifact roles overlap or are stale.
    Artifact,
    /// A named budget is invalid or exhausted.
    Budget,
    /// A host service request or observation does not match admission.
    Service,
    /// A terminal outcome was promoted or conflated.
    Outcome,
    /// A metadata receipt leaks unsafe data or claims completion authority.
    Receipt,
}

/// Deterministic campaign diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignDiagnostic {
    /// Stable issue class.
    pub issue: CampaignIssue,
    /// Stable field or dimension label.
    pub field: &'static str,
    /// Bounded explanatory detail.
    pub detail: String,
}

impl CampaignDiagnostic {
    fn new(issue: CampaignIssue, field: &'static str, detail: impl Into<String>) -> Self {
        Self {
            issue,
            field,
            detail: detail.into(),
        }
    }
}

/// Specialized role in the proof campaign.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CampaignRole {
    /// Authors the candidate theorem statement.
    StatementAuthor,
    /// Reviews statement adequacy without mutation.
    StatementReviewer,
    /// Maintains theorem source and requests bounded proof execution.
    Prover,
    /// Runs supplied verification services without subject mutation.
    Verifier,
}

/// Existing semantic role class composed by a campaign assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CampaignRoleClass {
    /// Narrow implementation-class assignment.
    Implementation,
    /// Read-only review-class assignment.
    Review,
    /// Verification-service-only assignment.
    Verification,
}

/// Assignment-local mutation ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CampaignMutation {
    /// Candidate statement mutation only.
    Statement,
    /// Maintained theorem mutation only.
    MaintainedTheorem,
    /// No campaign-subject mutation.
    None,
}

/// Campaign capability vocabulary.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CampaignCapability {
    /// Read admitted source or artifact metadata.
    Read,
    /// Mutate only the candidate statement.
    WriteStatement,
    /// Review the immutable statement.
    ReviewStatement,
    /// Mutate only maintained theorem source.
    WriteMaintainedTheorem,
    /// Request the injected proof service.
    RequestProof,
    /// Request the injected verification service.
    RequestVerification,
}

/// Campaign role assignment supplied to the pure planner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignAssignment {
    /// Stable assignment identity.
    pub assignment_id: String,
    /// Stable actor identity supplied by the host.
    pub actor_id: String,
    /// Specialized campaign role.
    pub role: CampaignRole,
    /// Existing semantic role class.
    pub role_class: CampaignRoleClass,
    /// Exact admitted model-profile invocation identity.
    pub model_profile_identity: String,
    /// Assignment-local mutation ceiling.
    pub mutation: CampaignMutation,
    /// Exact capability ceiling.
    pub capabilities: Vec<CampaignCapability>,
    /// Exact input-contract identity.
    pub input_contract: String,
    /// Exact output-contract identity.
    pub output_contract: String,
    /// Exact injected-service ceiling.
    pub service_ceiling: Vec<String>,
    /// Exact evidence-role ceiling.
    pub evidence_ceiling: Vec<CampaignArtifactRole>,
}

/// Named finite campaign budgets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CampaignBudgets {
    /// Host-measured proof time budget in milliseconds.
    pub proof_time_ms: u64,
    /// Maximum campaign stage calls.
    pub stage_calls: u32,
    /// Maximum injected tool/service calls.
    pub tool_calls: u32,
    /// Maximum bounded output bytes.
    pub output_bytes: u64,
    /// Maximum retrieved items.
    pub retrieval_items: u32,
    /// Maximum retained log bytes.
    pub log_bytes: u64,
    /// Maximum retained artifacts.
    pub retained_artifacts: u32,
}

/// Measured campaign resource use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CampaignUsage {
    /// Measured proof time in milliseconds.
    pub proof_time_ms: u64,
    /// Observed stage calls.
    pub stage_calls: u32,
    /// Observed tool/service calls.
    pub tool_calls: u32,
    /// Observed output bytes.
    pub output_bytes: u64,
    /// Observed retrieval items.
    pub retrieval_items: u32,
    /// Observed retained log bytes.
    pub log_bytes: u64,
    /// Observed retained artifacts.
    pub retained_artifacts: u32,
}

/// Artifact role in a campaign plan.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CampaignArtifactRole {
    /// Admitted Rust source.
    Source,
    /// Maintained candidate theorem statement.
    StatementCandidate,
    /// Generated translated model.
    GeneratedModel,
    /// Generated translation/provider manifest.
    GeneratedManifest,
    /// Maintained theorem source.
    MaintainedTheorem,
    /// Statement review record.
    StatementReview,
    /// Proof-service receipt.
    ProofReceipt,
    /// Final verification receipt.
    VerificationReceipt,
    /// Bounded diagnostic log.
    BoundedLog,
}

impl CampaignArtifactRole {
    fn generated(self) -> bool {
        matches!(self, Self::GeneratedModel | Self::GeneratedManifest)
    }
}

/// Metadata-only artifact descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignArtifact {
    /// Safe logical locator, never a private absolute path.
    pub locator: String,
    /// Artifact role.
    pub role: CampaignArtifactRole,
    /// Exact BLAKE3 identity.
    pub identity: String,
    /// Generation identity for generated artifacts.
    pub generation_identity: Option<String>,
    /// Assignment allowed to mutate the artifact, when any.
    pub mutable_by: Option<CampaignRole>,
}

/// Already-loaded campaign plan draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignPlanDraft {
    /// Campaign profile identifier.
    pub profile_id: String,
    /// Canonical skill source BLAKE3.
    pub skill_identity: String,
    /// Existing recipe workflow identity.
    pub recipe_identity: String,
    /// Separate host-owned completion gate identity.
    pub completion_gate_identity: String,
    /// Exact immutable campaign subject identity.
    pub subject_identity: String,
    /// Exact source identity.
    pub source_identity: String,
    /// Exact generated-model identity.
    pub model_identity: String,
    /// Exact type/overflow-model identity.
    pub type_model_identity: String,
    /// Role assignments.
    pub assignments: Vec<CampaignAssignment>,
    /// Artifact descriptors.
    pub artifacts: Vec<CampaignArtifact>,
    /// Named finite budgets.
    pub budgets: CampaignBudgets,
}

/// Admitted campaign plan identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignPlan {
    /// Deterministic plan BLAKE3 identity.
    pub identity: String,
    /// Immutable subject identity.
    pub subject_identity: String,
    /// Source identity.
    pub source_identity: String,
    /// Generated-model identity.
    pub model_identity: String,
    /// Type/overflow-model identity.
    pub type_model_identity: String,
    /// Named finite budgets.
    pub budgets: CampaignBudgets,
    /// Admitted assignments.
    pub assignments: Vec<CampaignAssignment>,
    /// Admitted artifacts.
    pub artifacts: Vec<CampaignArtifact>,
}

/// Separation dimension reviewed between two assignments.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SeparationDimension {
    /// Assignment identities differ.
    Assignment,
    /// Actor identities differ.
    Actor,
    /// Run/thread identities differ.
    Run,
    /// Context identities differ.
    Context,
    /// Workspace identities differ.
    Workspace,
    /// Model profile identities differ.
    ModelProfile,
    /// Mutation lineage identities differ.
    MutationLineage,
    /// Statement identity remained immutable.
    Statement,
    /// Generated-model identity remained immutable.
    Model,
    /// Verifier/check identity differs from the producer service.
    VerifierCheck,
    /// Subject is immutable and was not mutated by reviewer/verifier.
    SubjectImmutable,
}

/// Supplied observations for review or verification separation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeparationObservation {
    /// Prior assignment identity.
    pub prior_assignment: String,
    /// Current assignment identity.
    pub current_assignment: String,
    /// Prior actor identity.
    pub prior_actor: String,
    /// Current actor identity.
    pub current_actor: String,
    /// Prior run/thread identity.
    pub prior_run: String,
    /// Current run/thread identity.
    pub current_run: String,
    /// Prior context identity.
    pub prior_context: String,
    /// Current context identity.
    pub current_context: String,
    /// Prior workspace identity.
    pub prior_workspace: String,
    /// Current workspace identity.
    pub current_workspace: String,
    /// Prior model-profile identity.
    pub prior_model_profile: String,
    /// Current model-profile identity.
    pub current_model_profile: String,
    /// Prior mutation-lineage identity.
    pub prior_mutation_lineage: String,
    /// Current mutation-lineage identity.
    pub current_mutation_lineage: String,
    /// Approved statement identity before and after the handoff.
    pub statement_unchanged: bool,
    /// Generated model identity before and after the handoff.
    pub model_unchanged: bool,
    /// Producer service identity.
    pub producer_service: String,
    /// Verifier/check service identity.
    pub verifier_check: String,
    /// Whether the current assignment mutated the subject.
    pub subject_mutated: bool,
}

/// Reviewed separation decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeparationDecision {
    /// Whether every required dimension passed.
    pub admitted: bool,
    /// Exact failed dimensions.
    pub failed_dimensions: Vec<SeparationDimension>,
    /// BLAKE3 identity of supplied observations and the decision.
    pub decision_identity: String,
}

/// Statement adequacy decision supplied by the reviewer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatementAdequacy {
    /// Exact statement was reviewed and accepted as non-vacuous for this campaign.
    Accepted,
    /// Statement is vacuous or disconnected from the requirement.
    Vacuous,
    /// Statement contains unsupported proof holes or declarations.
    Unsupported,
    /// Reviewer could not establish adequacy.
    Unknown,
}

/// Declared trust-input kind.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum TrustInputKind {
    /// Declared model or semantic assumption.
    Model,
    /// Declared theorem assumption.
    Assumption,
    /// Declared axiom.
    Axiom,
    /// Declared opaque definition or external theorem.
    OpaqueDefinition,
}

/// One metadata-only trust-closure entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustInput {
    /// Trust-input kind.
    pub kind: TrustInputKind,
    /// Stable label.
    pub label: String,
    /// Exact BLAKE3 identity.
    pub identity: String,
}

/// Statement-review approval draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatementApprovalDraft {
    /// Requirement identifier.
    pub requirement_id: String,
    /// Approved theorem-statement BLAKE3.
    pub statement_identity: String,
    /// Bounded claim-class label.
    pub claim_class: String,
    /// Exact source identity.
    pub source_identity: String,
    /// Exact generated-model identity.
    pub model_identity: String,
    /// Exact type/overflow-model identity.
    pub type_model_identity: String,
    /// Reviewer adequacy decision.
    pub adequacy: StatementAdequacy,
    /// Whether the statement contains `sorry` or `admit`.
    pub contains_proof_hole: bool,
    /// Whether the declared trust closure is complete for the reviewed statement.
    pub closure_declared_complete: bool,
    /// Required trust kinds for this claim class.
    pub required_trust_kinds: Vec<TrustInputKind>,
    /// Declared trust closure.
    pub trust_inputs: Vec<TrustInput>,
}

/// Admitted immutable statement approval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatementApproval {
    /// Requirement identifier.
    pub requirement_id: String,
    /// Approved statement identity.
    pub statement_identity: String,
    /// Claim class.
    pub claim_class: String,
    /// Source identity.
    pub source_identity: String,
    /// Model identity.
    pub model_identity: String,
    /// Type/overflow-model identity.
    pub type_model_identity: String,
    /// Deterministic trust-closure identity.
    pub trust_closure_identity: String,
}

/// Host-owned proof/evidence service request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofServiceRequest {
    /// Named injected service.
    pub service_id: String,
    /// Assignment identity.
    pub assignment_id: String,
    /// Campaign plan identity.
    pub plan_identity: String,
    /// Subject identity.
    pub subject_identity: String,
    /// Approved statement identity.
    pub statement_identity: String,
    /// Generated model identity.
    pub model_identity: String,
    /// Budget-policy identity.
    pub budget_identity: String,
    /// Capability-decision identity.
    pub capability_identity: String,
}

/// Host-supplied bounded service observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofServiceObservation {
    /// Request identity computed before host execution.
    pub request_identity: String,
    /// Exact service identity actually used.
    pub service_id: String,
    /// Exact bounded evidence identity.
    pub evidence_identity: String,
    /// Whether the service completed its declared check successfully.
    pub passed: bool,
}

/// Typed terminal campaign outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CampaignOutcome {
    /// Proof and required verification evidence passed.
    Proved,
    /// Campaign reached an explicit blocker.
    Blocked,
    /// A counterexample was produced.
    Counterexample,
    /// Statement approval is invalid or stale.
    InvalidStatement,
    /// Generated model is invalid or stale.
    InvalidModel,
    /// A named budget was exhausted.
    BudgetExhausted,
    /// Injected proof/evidence service failed.
    ToolFailure,
    /// Policy or capability admission denied work.
    Denied,
    /// Host cancellation was observed.
    Cancelled,
    /// Agent or reviewer abstained.
    Abstained,
}

/// Terminal observation supplied to pure outcome reduction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalObservation {
    /// Exact outcome observed by the host/service boundary.
    pub outcome: CampaignOutcome,
    /// Measured bounded usage.
    pub usage: CampaignUsage,
    /// Required evidence identities.
    pub evidence_identities: Vec<String>,
    /// Whether final verification passed for the immutable subject.
    pub verification_passed: bool,
}

/// Pure terminal decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalDecision {
    /// Exact preserved outcome, or budget exhaustion when usage exceeded limits.
    pub outcome: CampaignOutcome,
    /// Whether this is only a candidate for a separate completion gate.
    pub completion_candidate: bool,
    /// Deterministic terminal-decision identity.
    pub decision_identity: String,
}

/// Host-measured bounded counts included in a safe campaign receipt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CampaignReceiptCounts {
    /// Observed stage calls.
    pub stage_calls: u32,
    /// Observed injected service/tool calls.
    pub tool_calls: u32,
    /// Number of evidence identities.
    pub evidence_identities: u32,
    /// Number of bounded issue classes.
    pub issue_classes: u32,
    /// Number of explicit redaction markers.
    pub redaction_markers: u32,
}

/// Safe metadata-only campaign receipt draft.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignReceiptDraft {
    /// Profile identity.
    pub profile_identity: String,
    /// Skill source identity.
    pub skill_identity: String,
    /// Plan identity.
    pub plan_identity: String,
    /// Specialized role-profile identity.
    pub role_identity: String,
    /// Exact assignment identity.
    pub assignment_identity: String,
    /// Subject identity.
    pub subject_identity: String,
    /// Statement identity.
    pub statement_identity: String,
    /// Model identity.
    pub model_identity: String,
    /// Budget identity.
    pub budget_identity: String,
    /// Exact terminal stage identity.
    pub stage_identity: String,
    /// Host-measured bounded counts.
    pub bounded_counts: CampaignReceiptCounts,
    /// Terminal outcome.
    pub outcome: CampaignOutcome,
    /// Bounded evidence identities.
    pub evidence_identities: Vec<String>,
    /// Explicit redaction marker names.
    pub redaction_markers: Vec<String>,
    /// Bounded issue classes.
    pub issue_classes: Vec<String>,
    /// Must remain false; completion belongs to another host gate.
    pub claims_authoritative_completion: bool,
}

/// Validated safe campaign receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignReceipt {
    /// Receipt schema.
    pub schema: &'static str,
    /// Receipt identity.
    pub identity: String,
    /// Profile identity.
    pub profile_identity: String,
    /// Skill identity.
    pub skill_identity: String,
    /// Plan identity.
    pub plan_identity: String,
    /// Role identity.
    pub role_identity: String,
    /// Assignment identity.
    pub assignment_identity: String,
    /// Subject identity.
    pub subject_identity: String,
    /// Statement identity.
    pub statement_identity: String,
    /// Model identity.
    pub model_identity: String,
    /// Budget identity.
    pub budget_identity: String,
    /// Terminal stage identity.
    pub stage_identity: String,
    /// Bounded counts.
    pub bounded_counts: CampaignReceiptCounts,
    /// Terminal outcome.
    pub outcome: CampaignOutcome,
    /// Evidence identities.
    pub evidence_identities: Vec<String>,
    /// Bounded issue classes.
    pub issue_classes: Vec<String>,
    /// Redaction markers.
    pub redaction_markers: Vec<String>,
    /// Always false for this receipt type.
    pub authoritative_completion: bool,
}

// r[impl agent_harness_runtime.aeneas_proof_campaign.roles]
// r[impl agent_harness_runtime.aeneas_proof_campaign.artifact_boundary]
/// Validate and bind an already-loaded campaign plan.
///
/// # Errors
///
/// Returns a deterministic diagnostic for malformed identities, roles, budgets,
/// artifacts, or authority-promoting profile references.
pub fn validate_campaign_plan(
    draft: &CampaignPlanDraft,
) -> Result<CampaignPlan, CampaignDiagnostic> {
    require_exact(&draft.profile_id, PROFILE_ID, "profile_id")?;
    require_exact(&draft.skill_identity, SKILL_IDENTITY, "skill_identity")?;
    require_exact(&draft.recipe_identity, RECIPE_ID, "recipe_identity")?;
    require_exact(
        &draft.completion_gate_identity,
        COMPLETION_GATE_ID,
        "completion_gate_identity",
    )?;
    require_hash(&draft.subject_identity, "subject_identity")?;
    require_hash(&draft.source_identity, "source_identity")?;
    require_hash(&draft.model_identity, "model_identity")?;
    require_hash(&draft.type_model_identity, "type_model_identity")?;
    validate_budgets(draft.budgets)?;
    validate_assignments(&draft.assignments)?;
    validate_artifacts(&draft.artifacts)?;
    let identity = hash_parts(
        std::iter::once(draft.profile_id.as_str())
            .chain(std::iter::once(draft.skill_identity.as_str()))
            .chain(std::iter::once(draft.subject_identity.as_str()))
            .chain(std::iter::once(draft.source_identity.as_str()))
            .chain(std::iter::once(draft.model_identity.as_str()))
            .chain(
                draft
                    .assignments
                    .iter()
                    .map(|assignment| assignment.assignment_id.as_str()),
            )
            .chain(
                draft
                    .artifacts
                    .iter()
                    .map(|artifact| artifact.identity.as_str()),
            ),
    );
    Ok(CampaignPlan {
        identity,
        subject_identity: draft.subject_identity.clone(),
        source_identity: draft.source_identity.clone(),
        model_identity: draft.model_identity.clone(),
        type_model_identity: draft.type_model_identity.clone(),
        budgets: draft.budgets,
        assignments: draft.assignments.clone(),
        artifacts: draft.artifacts.clone(),
    })
}

// r[impl agent_harness_runtime.aeneas_proof_campaign.separation]
/// Evaluate explicit review or verification separation dimensions.
#[must_use]
pub fn evaluate_campaign_separation(observation: &SeparationObservation) -> SeparationDecision {
    let checks = [
        (
            observation.prior_assignment != observation.current_assignment,
            SeparationDimension::Assignment,
        ),
        (
            observation.prior_actor != observation.current_actor,
            SeparationDimension::Actor,
        ),
        (
            observation.prior_run != observation.current_run,
            SeparationDimension::Run,
        ),
        (
            observation.prior_context != observation.current_context,
            SeparationDimension::Context,
        ),
        (
            observation.prior_workspace != observation.current_workspace,
            SeparationDimension::Workspace,
        ),
        (
            observation.prior_model_profile != observation.current_model_profile,
            SeparationDimension::ModelProfile,
        ),
        (
            observation.prior_mutation_lineage != observation.current_mutation_lineage,
            SeparationDimension::MutationLineage,
        ),
        (
            observation.statement_unchanged,
            SeparationDimension::Statement,
        ),
        (observation.model_unchanged, SeparationDimension::Model),
        (
            observation.producer_service != observation.verifier_check,
            SeparationDimension::VerifierCheck,
        ),
        (
            !observation.subject_mutated,
            SeparationDimension::SubjectImmutable,
        ),
    ];
    let failed_dimensions = checks
        .into_iter()
        .filter_map(|(passed, dimension)| (!passed).then_some(dimension))
        .collect::<Vec<_>>();
    let decision_identity = hash_parts([
        observation.prior_assignment.as_str(),
        observation.current_assignment.as_str(),
        observation.prior_actor.as_str(),
        observation.current_actor.as_str(),
        observation.prior_run.as_str(),
        observation.current_run.as_str(),
        observation.prior_context.as_str(),
        observation.current_context.as_str(),
        observation.prior_workspace.as_str(),
        observation.current_workspace.as_str(),
        observation.prior_model_profile.as_str(),
        observation.current_model_profile.as_str(),
        observation.prior_mutation_lineage.as_str(),
        observation.current_mutation_lineage.as_str(),
        observation.producer_service.as_str(),
        observation.verifier_check.as_str(),
        if failed_dimensions.is_empty() {
            "admitted"
        } else {
            "denied"
        },
    ]);
    SeparationDecision {
        admitted: failed_dimensions.is_empty(),
        failed_dimensions,
        decision_identity,
    }
}

// r[impl agent_harness_runtime.aeneas_proof_campaign.statement_gate]
/// Validate theorem-statement approval and complete declared trust closure.
///
/// # Errors
///
/// Returns a statement diagnostic for vacuity, proof holes, malformed/stale
/// identities, duplicate trust entries, or incomplete required trust kinds.
pub fn approve_campaign_statement(
    draft: &StatementApprovalDraft,
) -> Result<StatementApproval, CampaignDiagnostic> {
    require_label(
        &draft.requirement_id,
        "requirement_id",
        CampaignIssue::Statement,
    )?;
    require_hash(&draft.statement_identity, "statement_identity")?;
    require_label(&draft.claim_class, "claim_class", CampaignIssue::Statement)?;
    require_hash(&draft.source_identity, "source_identity")?;
    require_hash(&draft.model_identity, "model_identity")?;
    require_hash(&draft.type_model_identity, "type_model_identity")?;
    if draft.adequacy != StatementAdequacy::Accepted {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Statement,
            "adequacy",
            "statement must be explicitly accepted as non-vacuous",
        ));
    }
    if draft.contains_proof_hole {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Statement,
            "proof_hole",
            "statement closure contains sorry or admit",
        ));
    }
    if !draft.closure_declared_complete {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Statement,
            "trust_closure",
            "declared model/assumption/axiom/opaque closure is incomplete",
        ));
    }
    let mut entries = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for input in &draft.trust_inputs {
        require_label(&input.label, "trust_label", CampaignIssue::Statement)?;
        require_hash(&input.identity, "trust_identity")?;
        if !entries.insert((input.kind, input.label.as_str())) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Statement,
                "trust_closure",
                "duplicate trust input",
            ));
        }
        kinds.insert(input.kind);
    }
    if draft
        .required_trust_kinds
        .iter()
        .any(|required| !kinds.contains(required))
    {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Statement,
            "trust_closure",
            "required trust-input kind is undeclared",
        ));
    }
    let trust_closure_identity = hash_parts(draft.trust_inputs.iter().flat_map(|input| {
        [
            trust_kind_label(input.kind),
            input.label.as_str(),
            input.identity.as_str(),
        ]
    }));
    Ok(StatementApproval {
        requirement_id: draft.requirement_id.clone(),
        statement_identity: draft.statement_identity.clone(),
        claim_class: draft.claim_class.clone(),
        source_identity: draft.source_identity.clone(),
        model_identity: draft.model_identity.clone(),
        type_model_identity: draft.type_model_identity.clone(),
        trust_closure_identity,
    })
}

// r[impl agent_harness_runtime.aeneas_proof_campaign.host_boundary]
/// Admit an exact host-owned proof service request.
///
/// # Errors
///
/// Returns a diagnostic when assignment, plan, subject, statement, model,
/// budget, capability, or service identities are malformed or stale.
pub fn admit_proof_service_request(
    plan: &CampaignPlan,
    approval: &StatementApproval,
    request: &ProofServiceRequest,
) -> Result<String, CampaignDiagnostic> {
    require_label(&request.service_id, "service_id", CampaignIssue::Service)?;
    if !matches!(
        request.service_id.as_str(),
        "octet-aeneas-proof" | "octet-artifact-verify"
    ) {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Service,
            "service_id",
            "unknown proof/evidence service",
        ));
    }
    require_hash(&request.budget_identity, "budget_identity")?;
    require_hash(&request.capability_identity, "capability_identity")?;
    require_exact(&request.plan_identity, &plan.identity, "plan_identity")?;
    require_exact(
        &request.subject_identity,
        &plan.subject_identity,
        "subject_identity",
    )?;
    require_exact(
        &request.statement_identity,
        &approval.statement_identity,
        "statement_identity",
    )?;
    require_exact(
        &request.model_identity,
        &plan.model_identity,
        "model_identity",
    )?;
    if approval.source_identity != plan.source_identity
        || approval.model_identity != plan.model_identity
        || approval.type_model_identity != plan.type_model_identity
    {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Statement,
            "approval_binding",
            "statement approval does not bind the admitted source/model/type identities",
        ));
    }
    let assignment = plan
        .assignments
        .iter()
        .find(|assignment| assignment.assignment_id == request.assignment_id)
        .ok_or_else(|| {
            CampaignDiagnostic::new(CampaignIssue::Role, "assignment_id", "unknown assignment")
        })?;
    let required_role = if request.service_id == "octet-artifact-verify" {
        CampaignRole::Verifier
    } else {
        CampaignRole::Prover
    };
    if assignment.role != required_role {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Role,
            "assignment_id",
            "service requested by the wrong campaign role",
        ));
    }
    Ok(hash_parts([
        request.service_id.as_str(),
        request.assignment_id.as_str(),
        request.plan_identity.as_str(),
        request.subject_identity.as_str(),
        request.statement_identity.as_str(),
        request.model_identity.as_str(),
        request.budget_identity.as_str(),
        request.capability_identity.as_str(),
    ]))
}

/// Validate host-supplied service evidence against its admitted request.
///
/// # Errors
///
/// Returns a service diagnostic for request/service drift or malformed evidence.
pub fn validate_proof_service_observation(
    admitted_request_identity: &str,
    request: &ProofServiceRequest,
    observation: &ProofServiceObservation,
) -> Result<(), CampaignDiagnostic> {
    require_exact(
        &observation.request_identity,
        admitted_request_identity,
        "request_identity",
    )?;
    require_exact(&observation.service_id, &request.service_id, "service_id")?;
    require_hash(&observation.evidence_identity, "evidence_identity")?;
    if !observation.passed {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Service,
            "service_status",
            "proof/evidence service did not pass",
        ));
    }
    Ok(())
}

// r[impl agent_harness_runtime.aeneas_proof_campaign.outcomes]
/// Preserve an honest terminal outcome under named campaign budgets.
///
/// Budget overrun always becomes [`CampaignOutcome::BudgetExhausted`]. A proved
/// outcome is only a completion candidate when verification passed and exact
/// evidence identities are present.
pub fn evaluate_campaign_terminal(
    budgets: CampaignBudgets,
    observation: &TerminalObservation,
) -> TerminalDecision {
    let budget_exhausted = usage_exceeds(budgets, observation.usage);
    let evidence_valid = !observation.evidence_identities.is_empty()
        && observation
            .evidence_identities
            .iter()
            .all(|identity| is_hash(identity));
    let outcome = if budget_exhausted {
        CampaignOutcome::BudgetExhausted
    } else {
        observation.outcome
    };
    let completion_candidate =
        outcome == CampaignOutcome::Proved && observation.verification_passed && evidence_valid;
    let decision_identity = hash_parts(
        std::iter::once(outcome_label(outcome))
            .chain(std::iter::once(if completion_candidate {
                "candidate"
            } else {
                "not-candidate"
            }))
            .chain(observation.evidence_identities.iter().map(String::as_str)),
    );
    TerminalDecision {
        outcome,
        completion_candidate,
        decision_identity,
    }
}

// r[impl agent_harness_runtime.aeneas_proof_campaign.receipts]
/// Project a safe metadata-only campaign receipt.
///
/// # Errors
///
/// Returns a receipt diagnostic for malformed identities, unsafe metadata,
/// missing redactions, duplicate evidence, or authoritative-completion claims.
pub fn project_campaign_receipt(
    draft: &CampaignReceiptDraft,
) -> Result<CampaignReceipt, CampaignDiagnostic> {
    for (field, value) in [
        ("profile_identity", draft.profile_identity.as_str()),
        ("skill_identity", draft.skill_identity.as_str()),
        ("plan_identity", draft.plan_identity.as_str()),
        ("role_identity", draft.role_identity.as_str()),
        ("assignment_identity", draft.assignment_identity.as_str()),
        ("subject_identity", draft.subject_identity.as_str()),
        ("statement_identity", draft.statement_identity.as_str()),
        ("model_identity", draft.model_identity.as_str()),
        ("budget_identity", draft.budget_identity.as_str()),
        ("stage_identity", draft.stage_identity.as_str()),
    ] {
        require_hash(value, field)?;
    }
    if draft.claims_authoritative_completion {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Receipt,
            "authoritative_completion",
            "campaign receipt cannot authorize completion",
        ));
    }
    validate_unique_hashes(&draft.evidence_identities, "evidence_identities")?;
    validate_safe_labels(&draft.issue_classes, "issue_classes")?;
    validate_receipt_counts(draft)?;
    validate_redaction_markers(&draft.redaction_markers)?;
    for required in REQUIRED_REDACTIONS {
        if !draft
            .redaction_markers
            .iter()
            .any(|marker| marker == required)
        {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Receipt,
                "redaction_markers",
                format!("missing required redaction marker `{required}`"),
            ));
        }
    }
    let stage_calls = draft.bounded_counts.stage_calls.to_string();
    let tool_calls = draft.bounded_counts.tool_calls.to_string();
    let evidence_count = draft.bounded_counts.evidence_identities.to_string();
    let issue_count = draft.bounded_counts.issue_classes.to_string();
    let redaction_count = draft.bounded_counts.redaction_markers.to_string();
    let identity = hash_parts(
        [
            draft.profile_identity.as_str(),
            draft.skill_identity.as_str(),
            draft.plan_identity.as_str(),
            draft.role_identity.as_str(),
            draft.assignment_identity.as_str(),
            draft.subject_identity.as_str(),
            draft.statement_identity.as_str(),
            draft.model_identity.as_str(),
            draft.budget_identity.as_str(),
            draft.stage_identity.as_str(),
            outcome_label(draft.outcome),
            stage_calls.as_str(),
            tool_calls.as_str(),
            evidence_count.as_str(),
            issue_count.as_str(),
            redaction_count.as_str(),
        ]
        .into_iter()
        .chain(draft.evidence_identities.iter().map(String::as_str))
        .chain(draft.issue_classes.iter().map(String::as_str))
        .chain(draft.redaction_markers.iter().map(String::as_str)),
    );
    Ok(CampaignReceipt {
        schema: RECEIPT_SCHEMA,
        identity,
        profile_identity: draft.profile_identity.clone(),
        skill_identity: draft.skill_identity.clone(),
        plan_identity: draft.plan_identity.clone(),
        role_identity: draft.role_identity.clone(),
        assignment_identity: draft.assignment_identity.clone(),
        subject_identity: draft.subject_identity.clone(),
        statement_identity: draft.statement_identity.clone(),
        model_identity: draft.model_identity.clone(),
        budget_identity: draft.budget_identity.clone(),
        stage_identity: draft.stage_identity.clone(),
        bounded_counts: draft.bounded_counts,
        outcome: draft.outcome,
        evidence_identities: draft.evidence_identities.clone(),
        issue_classes: draft.issue_classes.clone(),
        redaction_markers: draft.redaction_markers.clone(),
        authoritative_completion: false,
    })
}

fn validate_receipt_counts(draft: &CampaignReceiptDraft) -> Result<(), CampaignDiagnostic> {
    let counts_match = count_matches(
        draft.bounded_counts.evidence_identities,
        draft.evidence_identities.len(),
    ) && count_matches(
        draft.bounded_counts.issue_classes,
        draft.issue_classes.len(),
    ) && count_matches(
        draft.bounded_counts.redaction_markers,
        draft.redaction_markers.len(),
    );
    if !counts_match {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Receipt,
            "bounded_counts",
            "receipt collection counts do not match retained metadata",
        ));
    }
    Ok(())
}

fn count_matches(observed: u32, actual: usize) -> bool {
    usize::try_from(observed).is_ok_and(|observed| observed == actual)
}

fn validate_assignments(assignments: &[CampaignAssignment]) -> Result<(), CampaignDiagnostic> {
    let required = [
        CampaignRole::StatementAuthor,
        CampaignRole::StatementReviewer,
        CampaignRole::Prover,
        CampaignRole::Verifier,
    ];
    if assignments.len() != required.len() {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Role,
            "assignments",
            "campaign requires exactly four specialized assignments",
        ));
    }
    let mut roles = BTreeSet::new();
    let mut assignment_ids = BTreeSet::new();
    let mut actor_ids = BTreeSet::new();
    let mut model_profile_identities = BTreeSet::new();
    for assignment in assignments {
        require_label(
            &assignment.assignment_id,
            "assignment_id",
            CampaignIssue::Role,
        )?;
        require_label(&assignment.actor_id, "actor_id", CampaignIssue::Role)?;
        require_label(
            &assignment.model_profile_identity,
            "model_profile_identity",
            CampaignIssue::Role,
        )?;
        if !roles.insert(assignment.role)
            || !assignment_ids.insert(assignment.assignment_id.as_str())
        {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Role,
                "assignments",
                "duplicate role or assignment identity",
            ));
        }
        if !actor_ids.insert(assignment.actor_id.as_str()) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Role,
                "actor_id",
                "specialized assignments require distinct actors",
            ));
        }
        if !model_profile_identities.insert(assignment.model_profile_identity.as_str()) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Role,
                "model_profile_identity",
                "specialized assignments require distinct model-profile invocation identities",
            ));
        }
        require_label(
            &assignment.input_contract,
            "input_contract",
            CampaignIssue::Role,
        )?;
        require_label(
            &assignment.output_contract,
            "output_contract",
            CampaignIssue::Role,
        )?;
        for service in &assignment.service_ceiling {
            require_label(service, "service_ceiling", CampaignIssue::Role)?;
        }
        let (expected_class, expected_capabilities) = role_contract(assignment.role);
        if assignment.role_class != expected_class
            || assignment.capabilities != expected_capabilities
            || !assignment_matches_role_contract(assignment)
        {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Role,
                "capability_ceiling",
                "role class, mutation, input, output, service, capability, or evidence ceiling drifted",
            ));
        }
    }
    if required.into_iter().any(|role| !roles.contains(&role)) {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Role,
            "assignments",
            "required specialized role is missing",
        ));
    }
    Ok(())
}

fn assignment_matches_role_contract(assignment: &CampaignAssignment) -> bool {
    match assignment.role {
        CampaignRole::StatementAuthor => {
            assignment.mutation == CampaignMutation::Statement
                && assignment.input_contract == "requirement-and-source"
                && assignment.output_contract == "statement-candidate"
                && assignment.service_ceiling.is_empty()
                && assignment.evidence_ceiling == [CampaignArtifactRole::StatementCandidate]
        }
        CampaignRole::StatementReviewer => {
            assignment.mutation == CampaignMutation::None
                && assignment.input_contract == "statement-candidate"
                && assignment.output_contract == "statement-approval"
                && assignment.service_ceiling.is_empty()
                && assignment.evidence_ceiling == [CampaignArtifactRole::StatementReview]
        }
        CampaignRole::Prover => {
            assignment.mutation == CampaignMutation::MaintainedTheorem
                && assignment.input_contract == "approved-statement-and-model"
                && assignment.output_contract == "proof-candidate"
                && labels_match(
                    &assignment.service_ceiling,
                    &["octet-aeneas-generation", "octet-aeneas-proof"],
                )
                && assignment.evidence_ceiling
                    == [
                        CampaignArtifactRole::GeneratedModel,
                        CampaignArtifactRole::GeneratedManifest,
                        CampaignArtifactRole::MaintainedTheorem,
                        CampaignArtifactRole::ProofReceipt,
                        CampaignArtifactRole::BoundedLog,
                    ]
        }
        CampaignRole::Verifier => {
            assignment.mutation == CampaignMutation::None
                && assignment.input_contract == "proof-receipt"
                && assignment.output_contract == "verification-report"
                && labels_match(&assignment.service_ceiling, &["octet-artifact-verify"])
                && assignment.evidence_ceiling == [CampaignArtifactRole::VerificationReceipt]
        }
    }
}

fn labels_match(observed: &[String], expected: &[&str]) -> bool {
    observed.len() == expected.len()
        && observed
            .iter()
            .zip(expected)
            .all(|(observed, expected)| observed == expected)
}

fn role_contract(role: CampaignRole) -> (CampaignRoleClass, Vec<CampaignCapability>) {
    match role {
        CampaignRole::StatementAuthor => (
            CampaignRoleClass::Implementation,
            vec![CampaignCapability::Read, CampaignCapability::WriteStatement],
        ),
        CampaignRole::StatementReviewer => (
            CampaignRoleClass::Review,
            vec![
                CampaignCapability::Read,
                CampaignCapability::ReviewStatement,
            ],
        ),
        CampaignRole::Prover => (
            CampaignRoleClass::Implementation,
            vec![
                CampaignCapability::Read,
                CampaignCapability::WriteMaintainedTheorem,
                CampaignCapability::RequestProof,
            ],
        ),
        CampaignRole::Verifier => (
            CampaignRoleClass::Verification,
            vec![
                CampaignCapability::Read,
                CampaignCapability::RequestVerification,
            ],
        ),
    }
}

fn validate_artifacts(artifacts: &[CampaignArtifact]) -> Result<(), CampaignDiagnostic> {
    let required = [
        CampaignArtifactRole::Source,
        CampaignArtifactRole::StatementCandidate,
        CampaignArtifactRole::GeneratedModel,
        CampaignArtifactRole::GeneratedManifest,
        CampaignArtifactRole::MaintainedTheorem,
        CampaignArtifactRole::StatementReview,
        CampaignArtifactRole::ProofReceipt,
        CampaignArtifactRole::VerificationReceipt,
        CampaignArtifactRole::BoundedLog,
    ];
    let mut locators = BTreeSet::new();
    let mut roles = BTreeSet::new();
    for artifact in artifacts {
        validate_artifact(artifact)?;
        if !locators.insert(artifact.locator.as_str()) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Artifact,
                "artifact_locator",
                "duplicate artifact locator",
            ));
        }
        roles.insert(artifact.role);
    }
    if required.into_iter().any(|role| !roles.contains(&role)) {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Artifact,
            "artifact_roles",
            "required artifact role is missing",
        ));
    }
    Ok(())
}

fn validate_artifact(artifact: &CampaignArtifact) -> Result<(), CampaignDiagnostic> {
    require_label(
        &artifact.locator,
        "artifact_locator",
        CampaignIssue::Artifact,
    )?;
    require_hash(&artifact.identity, "artifact_identity")?;
    validate_artifact_generation(artifact)?;
    validate_artifact_owner(artifact)
}

fn validate_artifact_generation(artifact: &CampaignArtifact) -> Result<(), CampaignDiagnostic> {
    if artifact.role.generated() {
        let generation_identity = artifact.generation_identity.as_deref().ok_or_else(|| {
            CampaignDiagnostic::new(
                CampaignIssue::Artifact,
                "generation_identity",
                "generated artifact lacks a freshness identity",
            )
        })?;
        require_exact(
            generation_identity,
            &artifact.identity,
            "generation_identity",
        )?;
        if artifact.mutable_by.is_some() {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Artifact,
                "mutable_by",
                "generated artifacts are immutable provider outputs",
            ));
        }
    } else if artifact.generation_identity.is_some() {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Artifact,
            "generation_identity",
            "maintained or recorded artifact cannot pose as generated",
        ));
    }
    Ok(())
}

fn validate_artifact_owner(artifact: &CampaignArtifact) -> Result<(), CampaignDiagnostic> {
    let expected_owner = match artifact.role {
        CampaignArtifactRole::StatementCandidate => Some(CampaignRole::StatementAuthor),
        CampaignArtifactRole::MaintainedTheorem => Some(CampaignRole::Prover),
        CampaignArtifactRole::StatementReview => Some(CampaignRole::StatementReviewer),
        CampaignArtifactRole::Source
        | CampaignArtifactRole::GeneratedModel
        | CampaignArtifactRole::GeneratedManifest
        | CampaignArtifactRole::ProofReceipt
        | CampaignArtifactRole::VerificationReceipt
        | CampaignArtifactRole::BoundedLog => None,
    };
    if artifact.mutable_by != expected_owner {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Artifact,
            "mutable_by",
            "artifact owner does not match its generated or maintained role",
        ));
    }
    Ok(())
}

fn validate_budgets(budgets: CampaignBudgets) -> Result<(), CampaignDiagnostic> {
    let positive = budgets.proof_time_ms > 0
        && budgets.stage_calls > 0
        && budgets.tool_calls > 0
        && budgets.output_bytes > 0
        && budgets.retrieval_items > 0
        && budgets.log_bytes > 0
        && budgets.retained_artifacts > 0;
    if !positive {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Budget,
            "budgets",
            "every named campaign budget must be positive",
        ));
    }
    Ok(())
}

fn usage_exceeds(budget: CampaignBudgets, usage: CampaignUsage) -> bool {
    usage.proof_time_ms > budget.proof_time_ms
        || usage.stage_calls > budget.stage_calls
        || usage.tool_calls > budget.tool_calls
        || usage.output_bytes > budget.output_bytes
        || usage.retrieval_items > budget.retrieval_items
        || usage.log_bytes > budget.log_bytes
        || usage.retained_artifacts > budget.retained_artifacts
}

fn validate_unique_hashes(
    values: &[String],
    field: &'static str,
) -> Result<(), CampaignDiagnostic> {
    if values.is_empty() {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Receipt,
            field,
            "at least one exact evidence identity is required",
        ));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_hash(value, field)?;
        if !seen.insert(value.as_str()) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Receipt,
                field,
                "duplicate evidence identity",
            ));
        }
    }
    Ok(())
}

fn validate_redaction_markers(values: &[String]) -> Result<(), CampaignDiagnostic> {
    if values.is_empty() {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Receipt,
            "redaction_markers",
            "redaction marker collection must be non-empty",
        ));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if value.len() > MAX_SAFE_LABEL_BYTES || value.chars().any(char::is_control) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Receipt,
                "redaction_markers",
                "unsafe or unbounded redaction marker",
            ));
        }
        if !seen.insert(value.as_str()) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Receipt,
                "redaction_markers",
                "duplicate redaction marker",
            ));
        }
    }
    Ok(())
}

fn validate_safe_labels(values: &[String], field: &'static str) -> Result<(), CampaignDiagnostic> {
    if values.is_empty() {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Receipt,
            field,
            "safe label collection must be non-empty",
        ));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_label(value, field, CampaignIssue::Receipt)?;
        if !seen.insert(value.as_str()) {
            return Err(CampaignDiagnostic::new(
                CampaignIssue::Receipt,
                field,
                "duplicate safe label",
            ));
        }
    }
    Ok(())
}

fn require_hash(value: &str, field: &'static str) -> Result<(), CampaignDiagnostic> {
    if !is_hash(value) {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Identity,
            field,
            "expected lowercase 64-character BLAKE3 identity",
        ));
    }
    Ok(())
}

fn is_hash(value: &str) -> bool {
    value.len() == BLAKE3_HEX_CHARS
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn require_exact(
    value: &str,
    expected: &str,
    field: &'static str,
) -> Result<(), CampaignDiagnostic> {
    if value != expected {
        return Err(CampaignDiagnostic::new(
            CampaignIssue::Identity,
            field,
            "identity or configured reference drifted",
        ));
    }
    Ok(())
}

fn require_label(
    value: &str,
    field: &'static str,
    issue: CampaignIssue,
) -> Result<(), CampaignDiagnostic> {
    let forbidden = FORBIDDEN_SAFE_TEXT
        .iter()
        .any(|fragment| value.contains(fragment));
    if value.is_empty()
        || value.len() > MAX_SAFE_LABEL_BYTES
        || value.chars().any(char::is_control)
        || forbidden
    {
        return Err(CampaignDiagnostic::new(
            issue,
            field,
            "unsafe or unbounded metadata label",
        ));
    }
    Ok(())
}

fn hash_parts<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        let length = u64::try_from(part.len()).map_or(u64::MAX, std::convert::identity);
        hasher.update(&length.to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn trust_kind_label(kind: TrustInputKind) -> &'static str {
    match kind {
        TrustInputKind::Model => "model",
        TrustInputKind::Assumption => "assumption",
        TrustInputKind::Axiom => "axiom",
        TrustInputKind::OpaqueDefinition => "opaque-definition",
    }
}

fn outcome_label(outcome: CampaignOutcome) -> &'static str {
    match outcome {
        CampaignOutcome::Proved => "proved",
        CampaignOutcome::Blocked => "blocked",
        CampaignOutcome::Counterexample => "counterexample",
        CampaignOutcome::InvalidStatement => "invalid-statement",
        CampaignOutcome::InvalidModel => "invalid-model",
        CampaignOutcome::BudgetExhausted => "budget-exhausted",
        CampaignOutcome::ToolFailure => "tool-failure",
        CampaignOutcome::Denied => "denied",
        CampaignOutcome::Cancelled => "cancelled",
        CampaignOutcome::Abstained => "abstained",
    }
}

// r[verify agent_harness_runtime.aeneas_proof_campaign.roles]
// r[verify agent_harness_runtime.aeneas_proof_campaign.separation]
// r[verify agent_harness_runtime.aeneas_proof_campaign.statement_gate]
// r[verify agent_harness_runtime.aeneas_proof_campaign.artifact_boundary]
// r[verify agent_harness_runtime.aeneas_proof_campaign.outcomes]
// r[verify agent_harness_runtime.aeneas_proof_campaign.host_boundary]
// r[verify agent_harness_runtime.aeneas_proof_campaign.receipts]
#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const TEST_HASH_VARIANT_COUNT: usize = 4;
    const TEST_GENERATED_MODEL_INDEX: usize = 2;
    const TEST_REQUIRED_ASSIGNMENT_COUNT: usize = 4;
    const TEST_BUDGET_LIMIT: u64 = 10;
    const TEST_CALL_LIMIT: u32 = 10;

    fn assignment(role: CampaignRole, id: &str, actor: &str) -> CampaignAssignment {
        let (role_class, capabilities) = role_contract(role);
        let (mutation, input_contract, output_contract, service_ceiling, evidence_ceiling) =
            assignment_boundaries(role);
        CampaignAssignment {
            assignment_id: id.into(),
            actor_id: actor.into(),
            role,
            role_class,
            model_profile_identity: format!("model-{id}"),
            mutation,
            capabilities,
            input_contract: input_contract.into(),
            output_contract: output_contract.into(),
            service_ceiling: service_ceiling.into_iter().map(str::to_string).collect(),
            evidence_ceiling,
        }
    }

    fn assignment_boundaries(
        role: CampaignRole,
    ) -> (
        CampaignMutation,
        &'static str,
        &'static str,
        Vec<&'static str>,
        Vec<CampaignArtifactRole>,
    ) {
        match role {
            CampaignRole::StatementAuthor => (
                CampaignMutation::Statement,
                "requirement-and-source",
                "statement-candidate",
                vec![],
                vec![CampaignArtifactRole::StatementCandidate],
            ),
            CampaignRole::StatementReviewer => (
                CampaignMutation::None,
                "statement-candidate",
                "statement-approval",
                vec![],
                vec![CampaignArtifactRole::StatementReview],
            ),
            CampaignRole::Prover => (
                CampaignMutation::MaintainedTheorem,
                "approved-statement-and-model",
                "proof-candidate",
                vec!["octet-aeneas-generation", "octet-aeneas-proof"],
                vec![
                    CampaignArtifactRole::GeneratedModel,
                    CampaignArtifactRole::GeneratedManifest,
                    CampaignArtifactRole::MaintainedTheorem,
                    CampaignArtifactRole::ProofReceipt,
                    CampaignArtifactRole::BoundedLog,
                ],
            ),
            CampaignRole::Verifier => (
                CampaignMutation::None,
                "proof-receipt",
                "verification-report",
                vec!["octet-artifact-verify"],
                vec![CampaignArtifactRole::VerificationReceipt],
            ),
        }
    }

    fn artifact(role: CampaignArtifactRole, index: usize) -> CampaignArtifact {
        let hashes = [HASH_A, HASH_B, HASH_C, HASH_D];
        let identity = hashes[index % TEST_HASH_VARIANT_COUNT].to_string();
        CampaignArtifact {
            locator: format!("artifact-{index}"),
            role,
            identity: identity.clone(),
            generation_identity: role.generated().then_some(identity),
            mutable_by: match role {
                CampaignArtifactRole::StatementCandidate => Some(CampaignRole::StatementAuthor),
                CampaignArtifactRole::MaintainedTheorem => Some(CampaignRole::Prover),
                CampaignArtifactRole::StatementReview => Some(CampaignRole::StatementReviewer),
                CampaignArtifactRole::Source
                | CampaignArtifactRole::GeneratedModel
                | CampaignArtifactRole::GeneratedManifest
                | CampaignArtifactRole::ProofReceipt
                | CampaignArtifactRole::VerificationReceipt
                | CampaignArtifactRole::BoundedLog => None,
            },
        }
    }

    fn budgets() -> CampaignBudgets {
        CampaignBudgets {
            proof_time_ms: TEST_BUDGET_LIMIT,
            stage_calls: TEST_CALL_LIMIT,
            tool_calls: TEST_CALL_LIMIT,
            output_bytes: TEST_BUDGET_LIMIT,
            retrieval_items: TEST_CALL_LIMIT,
            log_bytes: TEST_BUDGET_LIMIT,
            retained_artifacts: TEST_CALL_LIMIT,
        }
    }

    fn plan_draft() -> CampaignPlanDraft {
        let roles = [
            CampaignArtifactRole::Source,
            CampaignArtifactRole::StatementCandidate,
            CampaignArtifactRole::GeneratedModel,
            CampaignArtifactRole::GeneratedManifest,
            CampaignArtifactRole::MaintainedTheorem,
            CampaignArtifactRole::StatementReview,
            CampaignArtifactRole::ProofReceipt,
            CampaignArtifactRole::VerificationReceipt,
            CampaignArtifactRole::BoundedLog,
        ];
        CampaignPlanDraft {
            profile_id: PROFILE_ID.into(),
            skill_identity: SKILL_IDENTITY.into(),
            recipe_identity: RECIPE_ID.into(),
            completion_gate_identity: COMPLETION_GATE_ID.into(),
            subject_identity: HASH_A.into(),
            source_identity: HASH_B.into(),
            model_identity: HASH_C.into(),
            type_model_identity: HASH_D.into(),
            assignments: vec![
                assignment(CampaignRole::StatementAuthor, "statement-author", "actor-a"),
                assignment(
                    CampaignRole::StatementReviewer,
                    "statement-reviewer",
                    "actor-b",
                ),
                assignment(CampaignRole::Prover, "prover", "actor-c"),
                assignment(CampaignRole::Verifier, "verifier", "actor-d"),
            ],
            artifacts: roles
                .into_iter()
                .enumerate()
                .map(|(index, role)| artifact(role, index))
                .collect(),
            budgets: budgets(),
        }
    }

    fn approval_draft() -> StatementApprovalDraft {
        StatementApprovalDraft {
            requirement_id: "requirement-safe-add".into(),
            statement_identity: HASH_A.into(),
            claim_class: "exact-functional-correctness".into(),
            source_identity: HASH_B.into(),
            model_identity: HASH_C.into(),
            type_model_identity: HASH_D.into(),
            adequacy: StatementAdequacy::Accepted,
            contains_proof_hole: false,
            closure_declared_complete: true,
            required_trust_kinds: vec![TrustInputKind::Model],
            trust_inputs: vec![TrustInput {
                kind: TrustInputKind::Model,
                label: "aeneas-integer-model".into(),
                identity: HASH_B.into(),
            }],
        }
    }

    fn separation() -> SeparationObservation {
        SeparationObservation {
            prior_assignment: "prover".into(),
            current_assignment: "verifier".into(),
            prior_actor: "actor-c".into(),
            current_actor: "actor-d".into(),
            prior_run: "run-a".into(),
            current_run: "run-b".into(),
            prior_context: "context-a".into(),
            current_context: "context-b".into(),
            prior_workspace: "workspace-a".into(),
            current_workspace: "workspace-b".into(),
            prior_model_profile: "model-a".into(),
            current_model_profile: "model-b".into(),
            prior_mutation_lineage: "lineage-a".into(),
            current_mutation_lineage: "lineage-b".into(),
            statement_unchanged: true,
            model_unchanged: true,
            producer_service: "octet-aeneas-proof".into(),
            verifier_check: "octet-artifact-verify".into(),
            subject_mutated: false,
        }
    }

    fn receipt_draft() -> CampaignReceiptDraft {
        CampaignReceiptDraft {
            profile_identity: HASH_A.into(),
            skill_identity: HASH_B.into(),
            plan_identity: HASH_C.into(),
            role_identity: HASH_D.into(),
            assignment_identity: HASH_A.into(),
            subject_identity: HASH_D.into(),
            statement_identity: HASH_A.into(),
            model_identity: HASH_B.into(),
            budget_identity: HASH_C.into(),
            stage_identity: HASH_D.into(),
            bounded_counts: CampaignReceiptCounts {
                stage_calls: 1,
                tool_calls: 1,
                evidence_identities: 1,
                issue_classes: 1,
                redaction_markers: collection_count(REQUIRED_REDACTIONS.len()),
            },
            outcome: CampaignOutcome::Proved,
            evidence_identities: vec![HASH_D.into()],
            redaction_markers: REQUIRED_REDACTIONS
                .iter()
                .map(ToString::to_string)
                .collect(),
            issue_classes: vec!["none".into()],
            claims_authoritative_completion: false,
        }
    }

    fn collection_count(length: usize) -> u32 {
        u32::try_from(length).map_or(u32::MAX, std::convert::identity)
    }

    #[test]
    fn valid_plan_binds_roles_budgets_artifacts_and_external_completion_gate() {
        let plan = validate_campaign_plan(&plan_draft()).expect("valid campaign plan");
        assert!(is_hash(&plan.identity));
        assert_eq!(plan.assignments.len(), TEST_REQUIRED_ASSIGNMENT_COUNT);
    }

    #[test]
    fn plan_rejects_capability_widening() {
        let mut draft = plan_draft();
        draft.assignments[1]
            .capabilities
            .push(CampaignCapability::WriteMaintainedTheorem);
        let error = validate_campaign_plan(&draft).expect_err("widening must fail");
        assert_eq!(error.issue, CampaignIssue::Role);
    }

    #[test]
    fn plan_rejects_input_output_service_and_evidence_widening() {
        let mut input = plan_draft();
        input.assignments[1].input_contract = "mutable-prover-context".into();
        assert_eq!(
            validate_campaign_plan(&input)
                .expect_err("input widening must fail")
                .issue,
            CampaignIssue::Role
        );
        let mut output = plan_draft();
        output.assignments[1].output_contract = "authoritative-completion".into();
        assert_eq!(
            validate_campaign_plan(&output)
                .expect_err("output widening must fail")
                .issue,
            CampaignIssue::Role
        );
        let mut service = plan_draft();
        service.assignments[1]
            .service_ceiling
            .push("octet-aeneas-proof".into());
        assert_eq!(
            validate_campaign_plan(&service)
                .expect_err("service widening must fail")
                .issue,
            CampaignIssue::Role
        );
        let mut evidence = plan_draft();
        evidence.assignments[1]
            .evidence_ceiling
            .push(CampaignArtifactRole::MaintainedTheorem);
        assert_eq!(
            validate_campaign_plan(&evidence)
                .expect_err("evidence widening must fail")
                .issue,
            CampaignIssue::Role
        );
    }

    #[test]
    fn plan_rejects_role_alias_and_inherited_actor() {
        let mut alias = plan_draft();
        alias.assignments[1].role = CampaignRole::StatementAuthor;
        assert_eq!(
            validate_campaign_plan(&alias)
                .expect_err("role alias must fail")
                .issue,
            CampaignIssue::Role
        );
        let mut inherited_actor = plan_draft();
        inherited_actor.assignments[1].actor_id = inherited_actor.assignments[0].actor_id.clone();
        assert_eq!(
            validate_campaign_plan(&inherited_actor)
                .expect_err("actor overlap must fail")
                .field,
            "actor_id"
        );
    }

    #[test]
    fn plan_rejects_generated_artifact_mutation() {
        let mut draft = plan_draft();
        draft.artifacts[TEST_GENERATED_MODEL_INDEX].mutable_by = Some(CampaignRole::Prover);
        let error =
            validate_campaign_plan(&draft).expect_err("generated artifact mutation must fail");
        assert_eq!(error.issue, CampaignIssue::Artifact);
    }

    #[test]
    fn plan_rejects_stale_generation_and_output_overlap() {
        let mut stale = plan_draft();
        stale.artifacts[TEST_GENERATED_MODEL_INDEX].generation_identity = Some(HASH_D.into());
        assert_eq!(
            validate_campaign_plan(&stale)
                .expect_err("stale generation must fail")
                .field,
            "generation_identity"
        );
        let mut overlap = plan_draft();
        overlap.artifacts[TEST_GENERATED_MODEL_INDEX].locator =
            overlap.artifacts[0].locator.clone();
        assert_eq!(
            validate_campaign_plan(&overlap)
                .expect_err("overlapping destination must fail")
                .field,
            "artifact_locator"
        );
    }

    #[test]
    fn separation_accepts_distinct_immutable_subject_paths() {
        let decision = evaluate_campaign_separation(&separation());
        assert!(decision.admitted);
        assert!(decision.failed_dimensions.is_empty());
    }

    #[test]
    fn separation_reports_exact_overlap_and_subject_mutation() {
        let mut observation = separation();
        observation
            .current_actor
            .clone_from(&observation.prior_actor);
        observation.subject_mutated = true;
        let decision = evaluate_campaign_separation(&observation);
        assert!(!decision.admitted);
        assert_eq!(
            decision.failed_dimensions,
            vec![
                SeparationDimension::Actor,
                SeparationDimension::SubjectImmutable
            ]
        );
    }

    #[test]
    fn separation_rejects_self_review_inherited_context_and_stale_subjects() {
        let mut observation = separation();
        observation.current_assignment = observation.prior_assignment.clone();
        observation.current_run = observation.prior_run.clone();
        observation.current_context = observation.prior_context.clone();
        observation.current_workspace = observation.prior_workspace.clone();
        observation.current_model_profile = observation.prior_model_profile.clone();
        observation.current_mutation_lineage = observation.prior_mutation_lineage.clone();
        observation.statement_unchanged = false;
        observation.model_unchanged = false;
        observation.verifier_check = observation.producer_service.clone();
        let decision = evaluate_campaign_separation(&observation);
        for dimension in [
            SeparationDimension::Assignment,
            SeparationDimension::Run,
            SeparationDimension::Context,
            SeparationDimension::Workspace,
            SeparationDimension::ModelProfile,
            SeparationDimension::MutationLineage,
            SeparationDimension::Statement,
            SeparationDimension::Model,
            SeparationDimension::VerifierCheck,
        ] {
            assert!(decision.failed_dimensions.contains(&dimension));
        }
    }

    #[test]
    fn statement_approval_binds_complete_declared_trust_closure() {
        let approval = approve_campaign_statement(&approval_draft()).expect("valid approval");
        assert!(is_hash(&approval.trust_closure_identity));
        assert_eq!(approval.model_identity, HASH_C);
    }

    #[test]
    fn statement_approval_rejects_proof_hole() {
        let mut draft = approval_draft();
        draft.contains_proof_hole = true;
        let error = approve_campaign_statement(&draft).expect_err("proof hole must fail");
        assert_eq!(error.field, "proof_hole");
    }

    #[test]
    fn statement_approval_rejects_vacuity_and_undeclared_opaque_dependency() {
        let mut vacuous = approval_draft();
        vacuous.adequacy = StatementAdequacy::Vacuous;
        assert_eq!(
            approve_campaign_statement(&vacuous)
                .expect_err("vacuity must fail")
                .field,
            "adequacy"
        );
        let mut opaque = approval_draft();
        opaque.required_trust_kinds = vec![TrustInputKind::OpaqueDefinition];
        assert_eq!(
            approve_campaign_statement(&opaque)
                .expect_err("undeclared opaque dependency must fail")
                .field,
            "trust_closure"
        );
    }

    #[test]
    fn statement_approval_rejects_missing_required_trust_kind() {
        let mut draft = approval_draft();
        draft.required_trust_kinds = vec![TrustInputKind::Axiom];
        let error = approve_campaign_statement(&draft).expect_err("incomplete closure must fail");
        assert_eq!(error.field, "trust_closure");
    }

    #[test]
    fn service_request_and_observation_bind_exact_admission() {
        let plan = validate_campaign_plan(&plan_draft()).expect("valid plan");
        let approval = approve_campaign_statement(&approval_draft()).expect("valid approval");
        let request = ProofServiceRequest {
            service_id: "octet-aeneas-proof".into(),
            assignment_id: "prover".into(),
            plan_identity: plan.identity.clone(),
            subject_identity: plan.subject_identity.clone(),
            statement_identity: approval.statement_identity.clone(),
            model_identity: plan.model_identity.clone(),
            budget_identity: HASH_A.into(),
            capability_identity: HASH_B.into(),
        };
        let request_identity =
            admit_proof_service_request(&plan, &approval, &request).expect("admitted request");
        let observation = ProofServiceObservation {
            request_identity: request_identity.clone(),
            service_id: request.service_id.clone(),
            evidence_identity: HASH_C.into(),
            passed: true,
        };
        validate_proof_service_observation(&request_identity, &request, &observation)
            .expect("bound evidence");
        let changed = ProofServiceObservation {
            request_identity,
            service_id: "octet-artifact-verify".into(),
            evidence_identity: HASH_C.into(),
            passed: true,
        };
        assert_eq!(
            validate_proof_service_observation(&changed.request_identity, &request, &changed)
                .expect_err("changed service evidence must fail")
                .field,
            "service_id"
        );
    }

    #[test]
    fn service_request_rejects_post_review_subject_and_model_drift() {
        let plan = validate_campaign_plan(&plan_draft()).expect("valid plan");
        let approval = approve_campaign_statement(&approval_draft()).expect("valid approval");
        let mut request = ProofServiceRequest {
            service_id: "octet-aeneas-proof".into(),
            assignment_id: "prover".into(),
            plan_identity: plan.identity.clone(),
            subject_identity: HASH_D.into(),
            statement_identity: approval.statement_identity.clone(),
            model_identity: plan.model_identity.clone(),
            budget_identity: HASH_A.into(),
            capability_identity: HASH_B.into(),
        };
        assert_eq!(
            admit_proof_service_request(&plan, &approval, &request)
                .expect_err("subject drift must fail")
                .field,
            "subject_identity"
        );
        request.subject_identity = plan.subject_identity.clone();
        request.model_identity = HASH_D.into();
        assert_eq!(
            admit_proof_service_request(&plan, &approval, &request)
                .expect_err("model drift must fail")
                .field,
            "model_identity"
        );
    }

    #[test]
    fn service_request_rejects_role_impersonation() {
        let plan = validate_campaign_plan(&plan_draft()).expect("valid plan");
        let approval = approve_campaign_statement(&approval_draft()).expect("valid approval");
        let request = ProofServiceRequest {
            service_id: "octet-aeneas-proof".into(),
            assignment_id: "statement-reviewer".into(),
            plan_identity: plan.identity.clone(),
            subject_identity: plan.subject_identity.clone(),
            statement_identity: approval.statement_identity.clone(),
            model_identity: plan.model_identity.clone(),
            budget_identity: HASH_A.into(),
            capability_identity: HASH_B.into(),
        };
        let error = admit_proof_service_request(&plan, &approval, &request)
            .expect_err("wrong role must fail");
        assert_eq!(error.issue, CampaignIssue::Role);
    }

    #[test]
    fn terminal_preserves_non_success_and_enforces_budget_exhaustion() {
        let observation = TerminalObservation {
            outcome: CampaignOutcome::Counterexample,
            usage: CampaignUsage {
                proof_time_ms: 1,
                stage_calls: 1,
                tool_calls: 1,
                output_bytes: 1,
                retrieval_items: 1,
                log_bytes: 1,
                retained_artifacts: 1,
            },
            evidence_identities: vec![HASH_A.into()],
            verification_passed: false,
        };
        let decision = evaluate_campaign_terminal(budgets(), &observation);
        assert_eq!(decision.outcome, CampaignOutcome::Counterexample);
        assert!(!decision.completion_candidate);
        let overrun = TerminalObservation {
            usage: CampaignUsage {
                tool_calls: budgets().tool_calls.saturating_add(1),
                ..observation.usage
            },
            ..observation
        };
        assert_eq!(
            evaluate_campaign_terminal(budgets(), &overrun).outcome,
            CampaignOutcome::BudgetExhausted
        );
    }

    #[test]
    fn proved_outcome_is_only_an_external_completion_candidate() {
        let observation = TerminalObservation {
            outcome: CampaignOutcome::Proved,
            usage: CampaignUsage {
                proof_time_ms: 1,
                stage_calls: 1,
                tool_calls: 1,
                output_bytes: 1,
                retrieval_items: 1,
                log_bytes: 1,
                retained_artifacts: 1,
            },
            evidence_identities: vec![HASH_A.into()],
            verification_passed: true,
        };
        let decision = evaluate_campaign_terminal(budgets(), &observation);
        assert!(decision.completion_candidate);
        assert_eq!(decision.outcome, CampaignOutcome::Proved);
    }

    #[test]
    fn receipt_projects_safe_metadata_without_completion_authority() {
        let receipt = project_campaign_receipt(&receipt_draft()).expect("valid receipt");
        assert_eq!(receipt.schema, RECEIPT_SCHEMA);
        assert!(!receipt.authoritative_completion);
        assert!(is_hash(&receipt.identity));
    }

    #[test]
    fn receipt_rejects_unsafe_label_and_completion_claim() {
        for unsafe_label in [
            "RAW_PROMPT",
            "HIDDEN_REASONING",
            "/home/private/source",
            "tool_output",
            "provider_payload",
        ] {
            let mut unsafe_draft = receipt_draft();
            unsafe_draft.issue_classes = vec![unsafe_label.into()];
            let unsafe_error =
                project_campaign_receipt(&unsafe_draft).expect_err("unsafe metadata must fail");
            assert_eq!(unsafe_error.issue, CampaignIssue::Receipt);
        }
        let mut count_draft = receipt_draft();
        count_draft.bounded_counts.evidence_identities = count_draft
            .bounded_counts
            .evidence_identities
            .saturating_add(1);
        let count_error =
            project_campaign_receipt(&count_draft).expect_err("count mismatch must fail");
        assert_eq!(count_error.field, "bounded_counts");
        let mut authority_draft = receipt_draft();
        authority_draft.claims_authoritative_completion = true;
        let authority_error =
            project_campaign_receipt(&authority_draft).expect_err("authority claim must fail");
        assert_eq!(authority_error.field, "authoritative_completion");
    }
}
