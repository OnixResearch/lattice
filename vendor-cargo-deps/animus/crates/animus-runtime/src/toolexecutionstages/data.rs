use crate::tool_executor_kit::{
    ToolExecutorCancellation, ToolExecutorDeadline, ToolExecutorOutputPolicy, ToolMutationPosture,
};

/// Stable contract identity for staged tool execution.
pub const TOOL_EXECUTION_STAGE_CONTRACT_IDENTITY: &str = "animus.tool-execution-stages.v1";
/// Stable public receipt schema for staged tool execution.
pub const TOOL_EXECUTION_STAGE_RECEIPT_SCHEMA: &str = "animus.tool-execution-stage-receipt.v1";
/// Marker proving that raw tool values are absent from public stage evidence.
pub const TOOL_EXECUTION_STAGE_REDACTION_MARKER: &str = "raw-tool-values-omitted";
const TOOL_EXECUTION_STAGE_PHASE_COUNT: usize = 8;
const TOOL_EXECUTION_STAGE_NON_CLAIM_COUNT: usize = 8;
/// Fixed phase order for every staged tool plan.
pub const TOOL_EXECUTION_STAGE_PHASES: [ToolExecutionStagePhase; TOOL_EXECUTION_STAGE_PHASE_COUNT] = [
    ToolExecutionStagePhase::PreAdmission,
    ToolExecutionStagePhase::FinalAdmission,
    ToolExecutionStagePhase::PreExecution,
    ToolExecutionStagePhase::Execution,
    ToolExecutionStagePhase::OutputNarrowing,
    ToolExecutionStagePhase::FinalValidation,
    ToolExecutionStagePhase::DurableCommit,
    ToolExecutionStagePhase::PostCommitObservation,
];
/// Explicit bounded non-claims for public stage evidence.
pub const TOOL_EXECUTION_STAGE_NON_CLAIMS: [&str; TOOL_EXECUTION_STAGE_NON_CLAIM_COUNT] = [
    "not-tool-correctness",
    "not-effect-rollback",
    "not-handler-honesty",
    "not-sandboxing",
    "not-task-success",
    "not-provider-correctness",
    "not-production-readiness",
    "not-release-eligibility",
];

/// One fixed tool-execution phase.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ToolExecutionStagePhase {
    /// Optional denial-only review before the authoritative admission barrier.
    PreAdmission,
    /// Existing `plan_tool_execution` final admission barrier.
    FinalAdmission,
    /// Optional denial or declared wrapper selection before execution.
    PreExecution,
    /// Existing injected executor boundary.
    Execution,
    /// Optional bounded model-visible output narrowing.
    OutputNarrowing,
    /// Existing executor-kit output and receipt validation barrier.
    FinalValidation,
    /// Coding-session-owned exact durable result commit barrier.
    DurableCommit,
    /// Metadata-only observation after the exact durable commit.
    PostCommitObservation,
}

impl ToolExecutionStagePhase {
    /// Returns the stable phase label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PreAdmission => "pre_admission",
            Self::FinalAdmission => "final_admission",
            Self::PreExecution => "pre_execution",
            Self::Execution => "execution",
            Self::OutputNarrowing => "output_narrowing",
            Self::FinalValidation => "final_validation",
            Self::DurableCommit => "durable_commit",
            Self::PostCommitObservation => "post_commit_observation",
        }
    }

    /// Returns whether host-selected handlers can exist in this phase.
    #[must_use]
    pub const fn allows_handlers(self) -> bool {
        matches!(
            self,
            Self::PreAdmission
                | Self::PreExecution
                | Self::OutputNarrowing
                | Self::PostCommitObservation
        )
    }
}

/// Borrowed handler facts before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageHandlerDraft<'input> {
    /// Extensible phase that owns this handler.
    pub phase: ToolExecutionStagePhase,
    /// Immutable BLAKE3 handler identity.
    pub handler_identity: &'input str,
    /// Immutable BLAKE3 handler-cohort identity.
    pub cohort_identity: &'input str,
}

/// One validated host-selected handler descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageHandler {
    pub(super) phase: ToolExecutionStagePhase,
    pub(super) handler_identity: String,
    pub(super) cohort_identity: String,
}

impl ToolExecutionStageHandler {
    /// Returns the owned extensible phase.
    #[must_use]
    pub const fn phase(&self) -> ToolExecutionStagePhase {
        self.phase
    }

    /// Returns the immutable handler identity.
    #[must_use]
    pub fn handler_identity(&self) -> &str {
        self.handler_identity.as_str()
    }

    /// Returns the immutable handler-cohort identity.
    #[must_use]
    pub fn cohort_identity(&self) -> &str {
        self.cohort_identity.as_str()
    }
}

/// Borrowed exact inputs for one immutable stage plan.
#[derive(Clone, Copy, Debug)]
pub struct ToolExecutionStagePlanDraft<'input> {
    /// Proposed phase order. Validation requires the fixed order.
    pub phase_order: &'input [ToolExecutionStagePhase],
    /// Ordered optional handler descriptors.
    pub handlers: &'input [ToolExecutionStageHandler],
    /// Existing authoritative executor-kit plan.
    pub tool_plan: &'input crate::tool_executor_kit::ToolExecutionPlan,
    /// Exact validated tool request used to create `tool_plan`.
    pub request: &'input animus_adapters::ToolRequest,
    /// Exact BLAKE3 runtime-authority seal identity.
    pub authority_seal_identity: &'input str,
    /// Exact BLAKE3 durable commit-owner identity.
    pub commit_owner_identity: &'input str,
}

/// One immutable exact staged tool-execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStagePlan {
    pub(super) plan_identity: String,
    pub(super) handlers: Vec<ToolExecutionStageHandler>,
    pub(super) tool_name: String,
    pub(super) schema_identity: String,
    pub(super) request_identity: String,
    pub(super) executor_binding: Option<String>,
    pub(super) authority_seal_identity: String,
    pub(super) capability_labels: Vec<String>,
    pub(super) mutation_targets: Vec<String>,
    pub(super) mutation_posture: Option<ToolMutationPosture>,
    pub(super) output_policy: Option<ToolExecutorOutputPolicy>,
    pub(super) cancellation: ToolExecutorCancellation,
    pub(super) deadline: ToolExecutorDeadline,
    pub(super) commit_owner_identity: String,
}

impl ToolExecutionStagePlan {
    /// Returns the immutable stage-plan identity.
    #[must_use]
    pub fn plan_identity(&self) -> &str {
        self.plan_identity.as_str()
    }

    /// Returns ordered selected handler descriptors.
    #[must_use]
    pub fn handlers(&self) -> &[ToolExecutionStageHandler] {
        self.handlers.as_slice()
    }

    /// Returns the exact tool name.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the exact tool-schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns the exact tool-request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }

    /// Returns the exact executor binding, when final admission found one.
    #[must_use]
    pub fn executor_binding(&self) -> Option<&str> {
        self.executor_binding.as_deref()
    }

    /// Returns the exact runtime-authority seal identity.
    #[must_use]
    pub fn authority_seal_identity(&self) -> &str {
        self.authority_seal_identity.as_str()
    }

    /// Returns exact required capability labels.
    #[must_use]
    pub fn capability_labels(&self) -> &[String] {
        self.capability_labels.as_slice()
    }

    /// Returns exact declared mutation targets.
    #[must_use]
    pub fn mutation_targets(&self) -> &[String] {
        self.mutation_targets.as_slice()
    }

    /// Returns the admitted mutation posture, when a definition matched.
    #[must_use]
    pub const fn mutation_posture(&self) -> Option<ToolMutationPosture> {
        self.mutation_posture
    }

    /// Returns the admitted output policy, when a definition matched.
    #[must_use]
    pub const fn output_policy(&self) -> Option<ToolExecutorOutputPolicy> {
        self.output_policy
    }

    /// Returns the exact cancellation posture.
    #[must_use]
    pub const fn cancellation(&self) -> ToolExecutorCancellation {
        self.cancellation
    }

    /// Returns the exact deadline posture.
    #[must_use]
    pub const fn deadline(&self) -> ToolExecutorDeadline {
        self.deadline
    }

    /// Returns the coding-session-owned durable commit identity.
    #[must_use]
    pub fn commit_owner_identity(&self) -> &str {
        self.commit_owner_identity.as_str()
    }
}

/// Candidate admission result for one extensible phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionStageDisposition {
    /// The candidate does not deny later processing.
    Admitted,
    /// The candidate denies all later effects and continuation.
    Denied,
}

/// Stable execution outcome class that output handlers cannot widen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionStageOutcomeClass {
    /// No executor outcome exists yet.
    Pending,
    /// The injected executor returned a value.
    Succeeded,
    /// The injected executor failed.
    Failed,
    /// Cancellation stopped the executor.
    Cancelled,
    /// The caller-owned deadline stopped the executor.
    TimedOut,
}

impl ToolExecutionStageOutcomeClass {
    /// Returns the stable outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
        }
    }
}

/// Borrowed candidate values before stage-plan comparison.
#[derive(Clone, Copy, Debug)]
pub struct ToolExecutionStageCandidateDraft<'input> {
    /// Candidate phase.
    pub phase: ToolExecutionStagePhase,
    /// Immutable stage-plan identity.
    pub stage_plan_identity: &'input str,
    /// Ordered selected-handler-set identity for this phase.
    pub handler_set_identity: &'input str,
    /// Exact request identity.
    pub request_identity: &'input str,
    /// Exact tool-schema identity.
    pub schema_identity: &'input str,
    /// Exact executor binding, when available.
    pub executor_binding: Option<&'input str>,
    /// Exact runtime-authority seal identity.
    pub authority_seal_identity: &'input str,
    /// Candidate capability labels.
    pub capability_labels: &'input [String],
    /// Exact mutation targets.
    pub mutation_targets: &'input [String],
    /// Candidate mutation posture.
    pub mutation_posture: Option<ToolMutationPosture>,
    /// Candidate visible-character bound.
    pub max_visible_chars: Option<u32>,
    /// Candidate raw-output byte bound.
    pub max_output_bytes: Option<u64>,
    /// Whether later output must remain redacted.
    pub required_redaction: bool,
    /// Candidate phase disposition.
    pub disposition: ToolExecutionStageDisposition,
    /// Candidate executor outcome class.
    pub outcome_class: ToolExecutionStageOutcomeClass,
}

/// One owned candidate returned by an injected phase service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageCandidate {
    pub(super) phase: ToolExecutionStagePhase,
    pub(super) stage_plan_identity: String,
    pub(super) handler_set_identity: String,
    pub(super) request_identity: String,
    pub(super) schema_identity: String,
    pub(super) executor_binding: Option<String>,
    pub(super) authority_seal_identity: String,
    pub(super) capability_labels: Vec<String>,
    pub(super) mutation_targets: Vec<String>,
    pub(super) mutation_posture: Option<ToolMutationPosture>,
    pub(super) max_visible_chars: Option<u32>,
    pub(super) max_output_bytes: Option<u64>,
    pub(super) required_redaction: bool,
    pub(super) disposition: ToolExecutionStageDisposition,
    pub(super) outcome_class: ToolExecutionStageOutcomeClass,
}

impl ToolExecutionStageCandidate {
    /// Returns the candidate phase.
    #[must_use]
    pub const fn phase(&self) -> ToolExecutionStagePhase {
        self.phase
    }

    /// Returns the candidate disposition.
    #[must_use]
    pub const fn disposition(&self) -> ToolExecutionStageDisposition {
        self.disposition
    }

    /// Returns the candidate outcome class.
    #[must_use]
    pub const fn outcome_class(&self) -> ToolExecutionStageOutcomeClass {
        self.outcome_class
    }
}

/// Current pure staged execution status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionStageStatus {
    /// The plan can advance through the next exact phase.
    Active,
    /// One phase denied all later effects and continuation.
    Denied,
    /// Final validation passed and exact durable commit is required.
    AwaitingCommit,
    /// Exact durable commit passed and post-commit observation is next.
    Committed,
    /// All selected post-commit observation completed.
    Observed,
}

/// Pure state for one immutable staged execution plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageState {
    pub(super) stage_plan_identity: String,
    pub(super) next_phase: Option<ToolExecutionStagePhase>,
    pub(super) status: ToolExecutionStageStatus,
    pub(super) outcome_class: ToolExecutionStageOutcomeClass,
    pub(super) max_visible_chars: Option<u32>,
    pub(super) max_output_bytes: Option<u64>,
    pub(super) required_redaction: bool,
    pub(super) truncated: bool,
    pub(super) raw_output_identity: Option<String>,
    pub(super) raw_output_byte_count: Option<u64>,
    pub(super) raw_output_char_count: Option<u64>,
    pub(super) narrowed_output_identity: Option<String>,
    pub(super) result_identity: Option<String>,
    pub(super) executor_receipt_identity: Option<String>,
    pub(super) durable_head_identity: Option<String>,
    pub(super) durable_commit_identity: Option<String>,
}

impl ToolExecutionStageState {
    /// Returns the immutable stage-plan identity.
    #[must_use]
    pub fn stage_plan_identity(&self) -> &str {
        self.stage_plan_identity.as_str()
    }

    /// Returns the next exact phase, if any.
    #[must_use]
    pub const fn next_phase(&self) -> Option<ToolExecutionStagePhase> {
        self.next_phase
    }

    /// Returns the current stage status.
    #[must_use]
    pub const fn status(&self) -> ToolExecutionStageStatus {
        self.status
    }

    /// Returns the current sticky executor outcome class.
    #[must_use]
    pub const fn outcome_class(&self) -> ToolExecutionStageOutcomeClass {
        self.outcome_class
    }

    /// Returns whether later visible output must remain redacted.
    #[must_use]
    pub const fn required_redaction(&self) -> bool {
        self.required_redaction
    }

    /// Returns whether final output is truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the exact final result identity after final validation.
    #[must_use]
    pub fn result_identity(&self) -> Option<&str> {
        self.result_identity.as_deref()
    }
}

/// Exact raw executor observation before output handling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageRawObservation {
    /// Immutable stage-plan identity.
    pub stage_plan_identity: String,
    /// Exact request identity.
    pub request_identity: String,
    /// BLAKE3 identity of transient output.
    pub output_identity: String,
    /// Typed executor outcome class.
    pub outcome_class: ToolExecutionStageOutcomeClass,
    /// Transient output byte count.
    pub output_byte_count: u64,
    /// Transient output character count.
    pub output_char_count: u64,
    /// Whether output already requires redaction.
    pub required_redaction: bool,
}

/// Exact output-narrowing observation after one selected handler cohort.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageOutputObservation {
    /// Common phase candidate.
    pub candidate: ToolExecutionStageCandidate,
    /// Exact transient input identity.
    pub input_output_identity: String,
    /// BLAKE3 narrowed-output identity.
    pub narrowed_output_identity: String,
    /// Narrowed output byte count.
    pub output_byte_count: u64,
    /// Narrowed output character count.
    pub output_char_count: u64,
    /// Whether the narrowed result is already truncated.
    pub truncated: bool,
}

/// Exact existing-owner final-validation observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageFinalObservation {
    /// Immutable stage-plan identity.
    pub stage_plan_identity: String,
    /// Exact narrowed-output identity.
    pub narrowed_output_identity: String,
    /// Exact BLAKE3 bounded-result identity.
    pub result_identity: String,
    /// Exact BLAKE3 executor-kit receipt identity.
    pub executor_receipt_identity: String,
    /// Sticky outcome class.
    pub outcome_class: ToolExecutionStageOutcomeClass,
    /// Whether the final bounded result is truncated.
    pub truncated: bool,
}

/// Exact coding-session-owned durable commit observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageCommitObservation {
    /// Immutable stage-plan identity.
    pub stage_plan_identity: String,
    /// Exact final result identity.
    pub result_identity: String,
    /// Exact durable commit-owner identity.
    pub commit_owner_identity: String,
    /// New durable BLAKE3 head identity.
    pub durable_head_identity: String,
    /// Whether the complete correlated result was committed.
    pub complete: bool,
    /// Whether synchronization completed.
    pub synchronized: bool,
}

/// Safe public terminal stage receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageReceipt {
    pub(super) schema: &'static str,
    pub(super) contract_identity: &'static str,
    pub(super) stage_plan_identity: String,
    pub(super) tool_name: String,
    pub(super) schema_identity: String,
    pub(super) request_identity: String,
    pub(super) authority_seal_identity: String,
    pub(super) commit_owner_identity: String,
    pub(super) status: ToolExecutionStageStatus,
    pub(super) outcome_class: ToolExecutionStageOutcomeClass,
    pub(super) handler_count: u32,
    pub(super) result_identity: Option<String>,
    pub(super) executor_receipt_identity: Option<String>,
    pub(super) durable_head_identity: Option<String>,
    pub(super) durable_commit_identity: Option<String>,
    pub(super) required_redaction: bool,
    pub(super) truncated: bool,
    pub(super) redaction_marker: &'static str,
    pub(super) non_claims: Vec<&'static str>,
    pub(super) receipt_identity: String,
}

impl ToolExecutionStageReceipt {
    /// Returns the receipt schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns the stable contract identity.
    #[must_use]
    pub const fn contract_identity(&self) -> &'static str {
        self.contract_identity
    }

    /// Returns the immutable stage-plan identity.
    #[must_use]
    pub fn stage_plan_identity(&self) -> &str {
        self.stage_plan_identity.as_str()
    }

    /// Returns the safe tool label.
    #[must_use]
    pub fn tool_name(&self) -> &str {
        self.tool_name.as_str()
    }

    /// Returns the safe tool-schema identity.
    #[must_use]
    pub fn schema_identity(&self) -> &str {
        self.schema_identity.as_str()
    }

    /// Returns the exact tool-request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.request_identity.as_str()
    }

    /// Returns the exact runtime-authority seal identity.
    #[must_use]
    pub fn authority_seal_identity(&self) -> &str {
        self.authority_seal_identity.as_str()
    }

    /// Returns the exact durable commit-owner identity.
    #[must_use]
    pub fn commit_owner_identity(&self) -> &str {
        self.commit_owner_identity.as_str()
    }

    /// Returns denied or observed terminal status.
    #[must_use]
    pub const fn status(&self) -> ToolExecutionStageStatus {
        self.status
    }

    /// Returns the sticky executor outcome class.
    #[must_use]
    pub const fn outcome_class(&self) -> ToolExecutionStageOutcomeClass {
        self.outcome_class
    }

    /// Returns the selected handler count.
    #[must_use]
    pub const fn handler_count(&self) -> u32 {
        self.handler_count
    }

    /// Returns the final bounded-result identity, when execution reached validation.
    #[must_use]
    pub fn result_identity(&self) -> Option<&str> {
        self.result_identity.as_deref()
    }

    /// Returns the executor-kit receipt identity, when validation completed.
    #[must_use]
    pub fn executor_receipt_identity(&self) -> Option<&str> {
        self.executor_receipt_identity.as_deref()
    }

    /// Returns the exact durable head identity, when commit completed.
    #[must_use]
    pub fn durable_head_identity(&self) -> Option<&str> {
        self.durable_head_identity.as_deref()
    }

    /// Returns the exact staged durable commit identity, when commit completed.
    #[must_use]
    pub fn durable_commit_identity(&self) -> Option<&str> {
        self.durable_commit_identity.as_deref()
    }

    /// Returns whether later visible output required redaction.
    #[must_use]
    pub const fn required_redaction(&self) -> bool {
        self.required_redaction
    }

    /// Returns whether the final bounded output was truncated.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the fixed raw-value omission marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns the BLAKE3 receipt identity.
    #[must_use]
    pub fn receipt_identity(&self) -> &str {
        self.receipt_identity.as_str()
    }

    /// Returns bounded explicit non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[&'static str] {
        self.non_claims.as_slice()
    }
}

/// Private values that public receipt projection must reject.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ToolExecutionStageSensitiveDraft<'input> {
    /// Raw tool arguments.
    pub raw_arguments: Option<&'input str>,
    /// Raw executor output.
    pub raw_output: Option<&'input str>,
    /// Raw narrowed output.
    pub narrowed_output: Option<&'input str>,
    /// Credential material.
    pub credential: Option<&'input str>,
    /// Private filesystem path.
    pub private_path: Option<&'input str>,
    /// Provider-native payload.
    pub provider_payload: Option<&'input str>,
    /// Raw handler diagnostic.
    pub raw_handler_error: Option<&'input str>,
}

/// Typed fail-closed stage issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolExecutionStageIssue {
    /// Fixed phase order was changed.
    InvalidPhaseOrder,
    /// One required BLAKE3 identity was malformed.
    InvalidIdentity,
    /// A handler was selected for a fixed owner phase.
    InvalidHandlerPhase,
    /// Selected handler count exceeded the named bound.
    HandlerBoundExceeded,
    /// A handler identity was duplicated.
    DuplicateHandler,
    /// The executor-kit plan and exact request differed.
    PlanBindingMismatch,
    /// A handler or cohort set was substituted.
    HandlerSubstitution,
    /// A phase ran before its fixed predecessor.
    PhaseOrderViolation,
    /// A later candidate tried to reverse sticky denial.
    StickyDenial,
    /// A candidate added a capability.
    CapabilityWidening,
    /// A candidate changed the runtime-authority seal.
    AuthorityWidening,
    /// A candidate widened mutation posture or targets.
    MutationWidening,
    /// A candidate increased an admitted bound.
    BoundWidening,
    /// A candidate changed failure, cancellation, or timeout into success.
    OutcomeWidening,
    /// Fixed tool, schema, request, executor, or arguments changed.
    RequestSubstitution,
    /// A candidate removed required redaction.
    RedactionRemoved,
    /// Output identity or output counts were not narrowing.
    OutputSubstitution,
    /// Final output must commit before observation or continuation.
    CommitRequired,
    /// Durable result commit was stale, partial, or mismatched.
    CommitMismatch,
    /// Public evidence contained private or inconsistent material.
    UnsafeEvidence,
}

/// Safe stage validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolExecutionStageError {
    pub(super) issue: ToolExecutionStageIssue,
}

impl ToolExecutionStageError {
    /// Returns the fail-closed issue class.
    #[must_use]
    pub const fn issue(self) -> ToolExecutionStageIssue {
        self.issue
    }
}
