//! Host-owned model-router execution service contracts.
//!
//! `animus-router` remains the no-std plan and policy decision core. This
//! adapter-family module defines the injected service seam, dispatch envelopes,
//! pure fan-out/quorum guards, fallback binding, and safe execution receipts
//! that host agents can implement without moving provider clients, credentials,
//! model discovery, network transport, or live scheduling into reusable crates.

#![cfg_attr(
    animus_octet,
    allow(tigerstyle::non_trait_imports, tigerstyle::path_segment_repetition)
)]

use alloc::string::String;
use alloc::vec::Vec;

const REQUEST_HANDLE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_execution_request_handle");
const CORRELATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_execution_correlation");
const MODEL_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_execution_model_label");
const RECIPE_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_execution_recipe_id");
const REDACTION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_execution_redaction");
const CONTRACT_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_execution_contract");
const SUMMARY_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("router_execution_summary");
const HASH_SEPARATOR: &str = "\u{1f}";
const RAW_PROMPT_MARKER: &str = "RAW_PROMPT:";
const PROVIDER_PAYLOAD_MARKER: &str = "PROVIDER_PAYLOAD:";
const WORKER_OUTPUT_MARKER: &str = "WORKER_OUTPUT:";
const JUDGE_RATIONALE_MARKER: &str = "JUDGE_RATIONALE:";
const HIDDEN_REASONING_MARKER: &str = "HIDDEN_REASONING:";
const CREDENTIAL_MARKER: &str = "CREDENTIAL:";
const TOKEN_MARKER: &str = "TOKEN=";
const HOME_PATH_MARKER: &str = "/home/";
const UNBOUNDED_TRACE_MARKER: &str = "UNBOUNDED_TRACE:";
const PRIVATE_KEY_MARKER: &str = "BEGIN PRIVATE KEY";
const API_KEY_MARKER: &str = "API_KEY=";
const AWS_SECRET_MARKER: &str = "AWS_SECRET_ACCESS_KEY";
const OPENAI_KEY_PREFIX_MARKER: &str = "sk-";
const MAX_REDACTION_MARKERS: usize = 16;
const MIN_DISPATCH_COUNT: u32 = 1;

const _: () = assert!(MAX_REDACTION_MARKERS > 0);
const _: () = assert!(MIN_DISPATCH_COUNT > 0);

const UNSAFE_RECEIPT_MARKERS: &[&str] = &[
    RAW_PROMPT_MARKER,
    PROVIDER_PAYLOAD_MARKER,
    WORKER_OUTPUT_MARKER,
    JUDGE_RATIONALE_MARKER,
    HIDDEN_REASONING_MARKER,
    CREDENTIAL_MARKER,
    TOKEN_MARKER,
    HOME_PATH_MARKER,
    UNBOUNDED_TRACE_MARKER,
    PRIVATE_KEY_MARKER,
    API_KEY_MARKER,
    AWS_SECRET_MARKER,
    OPENAI_KEY_PREFIX_MARKER,
];

/// Stable contract identity for model-router execution receipts.
pub const ROUTER_EXECUTION_RECEIPT_CONTRACT_IDENTITY: &str = "animus.router-execution-receipt.v1";

/// Error raised by router execution service dispatch or validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouterExecutionError {
    /// Generic adapter field validation failed.
    Contract(crate::AdapterContractError),
    /// No execution service was injected.
    MissingService,
    /// Host policy denied the service before provider dispatch.
    ServiceDenied,
    /// A known service was unavailable.
    ServiceUnavailable,
    /// A candidate label was not authorized by the materialized plan.
    UnauthorizedCandidate,
    /// Requested parallelism exceeded plan or budget bounds.
    ParallelismExceeded,
    /// Requested quorum could not be satisfied by dispatched candidates.
    ImpossibleQuorum,
    /// Candidate, parallelism, synthesis, or repair budget was exhausted.
    BudgetExceeded,
    /// Provider-side fallback attempted to bypass the router fallback chain.
    ProviderBypassDenied,
    /// Router fallback policy denied fallback or repair.
    FallbackDenied,
    /// The receipt or envelope used an unsupported contract identity.
    SchemaMismatch,
    /// The execution outcome was malformed for the dispatch envelope.
    MalformedOutcome,
    /// Receipt text contained raw prompt, provider, credential, path, or trace material.
    UnsafeReceipt {
        /// Field containing unsafe text.
        field: &'static str,
        /// Marker that caused rejection.
        marker: &'static str,
    },
    /// A collection exceeded its safe bound.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
}

impl From<crate::AdapterContractError> for RouterExecutionError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl core::fmt::Display for RouterExecutionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "router execution contract error: {error}"),
            Self::MissingService => formatter.write_str("router execution service is missing"),
            Self::ServiceDenied => formatter.write_str("router execution service was denied"),
            Self::ServiceUnavailable => {
                formatter.write_str("router execution service is unavailable")
            }
            Self::UnauthorizedCandidate => {
                formatter.write_str("router candidate is not plan-authorized")
            }
            Self::ParallelismExceeded => {
                formatter.write_str("router execution parallelism exceeded bounds")
            }
            Self::ImpossibleQuorum => formatter.write_str("router execution quorum is impossible"),
            Self::BudgetExceeded => formatter.write_str("router execution budget was exceeded"),
            Self::ProviderBypassDenied => {
                formatter.write_str("provider-side fallback bypass was denied")
            }
            Self::FallbackDenied => formatter.write_str("router fallback was denied"),
            Self::SchemaMismatch => formatter.write_str("router execution schema mismatch"),
            Self::MalformedOutcome => formatter.write_str("router execution outcome is malformed"),
            Self::UnsafeReceipt { field, marker } => {
                write!(formatter, "{field} contains unsafe marker {marker}")
            }
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} must contain at most {max_items} items; got {actual_items}"
            ),
        }
    }
}

/// Cancellation posture supplied to router execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterExecutionCancellation {
    /// Request may be cancelled by the host.
    Cancellable,
    /// Request is not cancellable.
    NotCancellable,
    /// Request was already cancelled before dispatch.
    Cancelled,
}

impl RouterExecutionCancellation {
    /// Returns the stable cancellation label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cancellable => "cancellable",
            Self::NotCancellable => "not_cancellable",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Safe execution control metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionControl {
    cancellation: RouterExecutionCancellation,
    deadline: crate::router::DeadlineClass,
    correlation: String,
    policy_hash: crate::SafeHash,
}

/// Borrowed input used to build execution control metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterExecutionControlInput<'a> {
    /// Cancellation posture.
    pub cancellation: RouterExecutionCancellation,
    /// Timeout/deadline class.
    pub deadline: crate::router::DeadlineClass,
    /// Safe correlation label.
    pub correlation: &'a str,
    /// Safe BLAKE3 policy hash.
    pub policy_hash: &'a str,
}

impl RouterExecutionControl {
    /// Builds safe execution control metadata.
    ///
    /// # Errors
    ///
    /// Returns [`RouterExecutionError`] when labels or hashes are invalid.
    pub fn new(input: RouterExecutionControlInput<'_>) -> Result<Self, RouterExecutionError> {
        Ok(Self {
            cancellation: input.cancellation,
            deadline: input.deadline,
            correlation: safe_text(CORRELATION_FIELD, input.correlation)?,
            policy_hash: crate::SafeHash::new(input.policy_hash)?,
        })
    }

    /// Returns cancellation posture.
    #[must_use]
    pub const fn cancellation(&self) -> RouterExecutionCancellation {
        self.cancellation
    }

    /// Returns timeout/deadline class.
    #[must_use]
    pub const fn deadline(&self) -> crate::router::DeadlineClass {
        self.deadline
    }

    /// Returns safe correlation label.
    #[must_use]
    pub fn correlation(&self) -> &str {
        self.correlation.as_str()
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::SafeHash {
        &self.policy_hash
    }
}

/// Plan-authorized role fan-out envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterRoleDispatch {
    role: crate::router::Role,
    candidate_labels: Vec<String>,
    parallelism_cap: u32,
    quorum: u32,
}

impl RouterRoleDispatch {
    /// Returns the router role.
    #[must_use]
    pub const fn role(&self) -> crate::router::Role {
        self.role
    }

    /// Returns authorized candidate labels.
    #[must_use]
    pub fn candidate_labels(&self) -> &[String] {
        self.candidate_labels.as_slice()
    }

    /// Returns the parallelism cap.
    #[must_use]
    pub const fn parallelism_cap(&self) -> u32 {
        self.parallelism_cap
    }

    /// Returns quorum threshold.
    #[must_use]
    pub const fn quorum(&self) -> u32 {
        self.quorum
    }
}

/// Builds a role dispatch envelope from a materialized role step and requested labels.
///
/// # Errors
///
/// Returns [`RouterExecutionError`] when candidates are not plan-authorized,
/// exceed parallelism, or cannot satisfy quorum.
///
/// # Panics
///
/// Panics if requested parallelism post-conditions are violated after validation.
pub fn authorize_router_role_fanout(
    step: &crate::router::RoleStep,
    requested_candidates: &[&str],
    requested_parallelism: u32,
) -> Result<RouterRoleDispatch, RouterExecutionError> {
    if requested_parallelism < MIN_DISPATCH_COUNT || requested_parallelism > step.parallelism_cap()
    {
        return Err(RouterExecutionError::ParallelismExceeded);
    }
    let requested_len = u32::try_from(requested_candidates.len())
        .map_err(|_error| RouterExecutionError::ParallelismExceeded)?;
    if requested_len > requested_parallelism {
        return Err(RouterExecutionError::ParallelismExceeded);
    }
    if requested_len < step.quorum() {
        return Err(RouterExecutionError::ImpossibleQuorum);
    }
    assert!(requested_parallelism >= MIN_DISPATCH_COUNT);
    assert!(requested_parallelism <= step.parallelism_cap());
    let mut candidate_labels = Vec::with_capacity(requested_candidates.len());
    for candidate in requested_candidates {
        let safe_candidate = safe_text(MODEL_LABEL_FIELD, candidate)?;
        if !step
            .candidate_labels()
            .iter()
            .any(|allowed| allowed == &safe_candidate)
        {
            return Err(RouterExecutionError::UnauthorizedCandidate);
        }
        candidate_labels.push(safe_candidate);
    }
    Ok(RouterRoleDispatch {
        role: step.role(),
        candidate_labels,
        parallelism_cap: requested_parallelism,
        quorum: step.quorum(),
    })
}

/// Borrowed input used to build a dispatch request from a validated router plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionDispatchInput<'a> {
    /// Validated materialized router execution plan.
    pub plan: &'a crate::router::RouterExecutionPlan,
    /// Safe handle for the already-validated model request.
    pub request_handle: &'a str,
    /// Safe execution control metadata.
    pub control: RouterExecutionControl,
    /// Current budget ledger state.
    pub ledger: &'a crate::router::BudgetLedger,
    /// Explicit fallback policy for this execution.
    pub fallback_policy: crate::router::FallbackPolicy,
}

/// Bounded router execution dispatch request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionDispatchRequest {
    recipe_id: String,
    family: crate::router::RouteFamily,
    selected_models: Vec<String>,
    output_contract: crate::router::OutputContract,
    request_handle: String,
    control: RouterExecutionControl,
    role_dispatches: Vec<RouterRoleDispatch>,
    candidate_budget_remaining: u32,
    fallback_policy: crate::router::FallbackPolicy,
}

impl RouterExecutionDispatchRequest {
    /// Builds a bounded dispatch request from a validated router plan.
    ///
    /// # Errors
    ///
    /// Returns [`RouterExecutionError`] when labels are unsafe, fan-out exceeds
    /// plan/budget bounds, or quorum is impossible.
    pub fn new(input: RouterExecutionDispatchInput<'_>) -> Result<Self, RouterExecutionError> {
        if input.control.cancellation() == RouterExecutionCancellation::Cancelled {
            return Err(RouterExecutionError::ServiceDenied);
        }
        let mut role_dispatches = Vec::with_capacity(input.plan.role_steps().len());
        for step in input.plan.role_steps() {
            let candidate_refs: Vec<&str> =
                step.candidate_labels().iter().map(String::as_str).collect();
            let dispatch = authorize_router_role_fanout(
                step,
                candidate_refs.as_slice(),
                step.parallelism_cap(),
            )?;
            if dispatch.parallelism_cap() > input.ledger.remaining_parallelism() {
                return Err(RouterExecutionError::ParallelismExceeded);
            }
            role_dispatches.push(dispatch);
        }
        let candidate_count = total_candidate_count(role_dispatches.as_slice())?;
        if candidate_count > input.ledger.remaining_candidates() {
            return Err(RouterExecutionError::BudgetExceeded);
        }
        Ok(Self {
            recipe_id: safe_text(RECIPE_FIELD, input.plan.recipe_id())?,
            family: input.plan.family(),
            selected_models: input.plan.selected_models().to_vec(),
            output_contract: input.plan.output_contract(),
            request_handle: safe_text(REQUEST_HANDLE_FIELD, input.request_handle)?,
            control: input.control,
            role_dispatches,
            candidate_budget_remaining: input.ledger.remaining_candidates(),
            fallback_policy: input.fallback_policy,
        })
    }

    /// Returns recipe identity.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }

    /// Returns route family.
    #[must_use]
    pub const fn family(&self) -> crate::router::RouteFamily {
        self.family
    }

    /// Returns selected model labels.
    #[must_use]
    pub fn selected_models(&self) -> &[String] {
        self.selected_models.as_slice()
    }

    /// Returns output contract.
    #[must_use]
    pub const fn output_contract(&self) -> crate::router::OutputContract {
        self.output_contract
    }

    /// Returns safe request handle.
    #[must_use]
    pub fn request_handle(&self) -> &str {
        self.request_handle.as_str()
    }

    /// Returns execution control metadata.
    #[must_use]
    pub const fn control(&self) -> &RouterExecutionControl {
        &self.control
    }

    /// Returns role dispatch envelopes.
    #[must_use]
    pub fn role_dispatches(&self) -> &[RouterRoleDispatch] {
        self.role_dispatches.as_slice()
    }

    /// Returns candidate budget remaining at dispatch time.
    #[must_use]
    pub const fn candidate_budget_remaining(&self) -> u32 {
        self.candidate_budget_remaining
    }

    /// Returns fallback policy.
    #[must_use]
    pub const fn fallback_policy(&self) -> crate::router::FallbackPolicy {
        self.fallback_policy
    }
}

/// Step outcome status class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterStepOutcomeStatus {
    /// Candidate succeeded.
    Success,
    /// Candidate failed safely.
    Failed,
    /// Candidate was cancelled.
    Cancelled,
    /// Candidate timed out.
    TimedOut,
    /// Candidate output was malformed.
    Malformed,
    /// Candidate was denied before dispatch.
    Denied,
}

impl RouterStepOutcomeStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Malformed => "malformed",
            Self::Denied => "denied",
        }
    }
}

/// Borrowed input used to build a role-step outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterStepOutcomeInput<'a> {
    /// Router role.
    pub role: crate::router::Role,
    /// Candidate model label.
    pub candidate_label: &'a str,
    /// Status class.
    pub status: RouterStepOutcomeStatus,
    /// Optional safe output hash.
    pub output_hash: Option<&'a str>,
    /// Duration bucket.
    pub duration: crate::router::DurationBucket,
    /// Redaction markers.
    pub redactions: &'a [&'a str],
}

/// Bounded role-step outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterStepOutcome {
    role: crate::router::Role,
    candidate_label: String,
    status: RouterStepOutcomeStatus,
    output_hash: Option<crate::SafeHash>,
    duration: crate::router::DurationBucket,
    redactions: Vec<String>,
}

impl RouterStepOutcome {
    /// Builds a bounded role-step outcome.
    ///
    /// # Errors
    ///
    /// Returns [`RouterExecutionError`] when labels, hashes, or redactions are invalid.
    pub fn new(input: RouterStepOutcomeInput<'_>) -> Result<Self, RouterExecutionError> {
        validate_redaction_count(input.redactions.len())?;
        Ok(Self {
            role: input.role,
            candidate_label: safe_text(MODEL_LABEL_FIELD, input.candidate_label)?,
            status: input.status,
            output_hash: input.output_hash.map(crate::SafeHash::new).transpose()?,
            duration: input.duration,
            redactions: safe_text_list(REDACTION_FIELD, input.redactions)?,
        })
    }

    /// Returns role.
    #[must_use]
    pub const fn role(&self) -> crate::router::Role {
        self.role
    }

    /// Returns candidate label.
    #[must_use]
    pub fn candidate_label(&self) -> &str {
        self.candidate_label.as_str()
    }

    /// Returns status.
    #[must_use]
    pub const fn status(&self) -> RouterStepOutcomeStatus {
        self.status
    }

    /// Returns output hash when present.
    #[must_use]
    pub const fn output_hash(&self) -> Option<&crate::SafeHash> {
        self.output_hash.as_ref()
    }

    /// Returns duration bucket.
    #[must_use]
    pub const fn duration(&self) -> crate::router::DurationBucket {
        self.duration
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redactions(&self) -> &[String] {
        self.redactions.as_slice()
    }
}

/// Overall execution status class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterExecutionStatus {
    /// Quorum was met and output contract was satisfied.
    Completed,
    /// Some candidates failed, but usable evidence exists.
    Partial,
    /// Quorum was not met.
    QuorumFailed,
    /// Execution was cancelled.
    Cancelled,
    /// Execution timed out.
    TimedOut,
    /// Execution was denied before dispatch.
    Denied,
}

impl RouterExecutionStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Partial => "partial",
            Self::QuorumFailed => "quorum_failed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Denied => "denied",
        }
    }
}

/// Output contract validation status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterOutputContractStatus {
    /// Output contract was satisfied.
    Satisfied,
    /// Output contract was malformed.
    Malformed,
    /// Output contract was missing.
    Missing,
}

impl RouterOutputContractStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::Malformed => "malformed",
            Self::Missing => "missing",
        }
    }
}

/// Repair class recorded in receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterRepairClass {
    /// No repair was attempted.
    None,
    /// Repair was authorized by budget.
    Authorized,
    /// Repair was denied by budget or policy.
    Denied,
}

impl RouterRepairClass {
    /// Returns the stable repair class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Authorized => "authorized",
            Self::Denied => "denied",
        }
    }
}

/// Fallback class recorded in receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterFallbackClass {
    /// No fallback was attempted.
    None,
    /// Fallback was authorized by the materialized chain.
    ChainAuthorized,
    /// Fallback was denied.
    Denied,
    /// Fallback failed the route.
    FailedRoute,
}

impl RouterFallbackClass {
    /// Returns the stable fallback class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::ChainAuthorized => "chain_authorized",
            Self::Denied => "denied",
            Self::FailedRoute => "failed_route",
        }
    }
}

/// Borrowed input used to evaluate router fallback safely.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterFallbackEvaluationInput<'a> {
    /// Failed model label.
    pub failed_model: &'a str,
    /// Whether the step is repairable.
    pub repairable: bool,
    /// Current budget ledger.
    pub ledger: &'a crate::router::BudgetLedger,
    /// Materialized fallback chain.
    pub chain: &'a crate::router::FallbackChain,
    /// Explicit fallback policy.
    pub policy: crate::router::FallbackPolicy,
    /// Whether a provider reported its own fallback outside the chain.
    pub provider_bypass_claimed: bool,
}

/// Evaluates fallback or repair while denying provider-side bypasses.
///
/// # Errors
///
/// Returns [`RouterExecutionError::ProviderBypassDenied`] when a provider-side
/// fallback bypass is claimed.
pub fn evaluate_router_execution_fallback(
    input: &RouterFallbackEvaluationInput<'_>,
) -> Result<crate::router::FallbackDecision, RouterExecutionError> {
    if input.provider_bypass_claimed {
        return Err(RouterExecutionError::ProviderBypassDenied);
    }
    let safe_model = safe_text(MODEL_LABEL_FIELD, input.failed_model)?;
    let failure = crate::router::StepFailure::new(safe_model.as_str(), input.repairable);
    Ok(crate::router::evaluate_fallback(
        &failure,
        input.ledger,
        input.chain,
        input.policy,
    ))
}

/// Borrowed input used to build a safe execution receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionReceiptDraft<'a> {
    /// Contract identity.
    pub contract_identity: &'a str,
    /// Recipe identity.
    pub recipe_id: &'a str,
    /// Route family.
    pub family: crate::router::RouteFamily,
    /// Execution status.
    pub status: RouterExecutionStatus,
    /// Selected model labels.
    pub selected_models: &'a [&'a str],
    /// Bounded candidate count.
    pub candidate_count: u32,
    /// Bounded success count.
    pub success_count: u32,
    /// Quorum class.
    pub quorum: crate::router::QuorumClass,
    /// Fallback class.
    pub fallback: RouterFallbackClass,
    /// Repair class.
    pub repair: RouterRepairClass,
    /// Output contract status.
    pub output_contract: RouterOutputContractStatus,
    /// Policy hash.
    pub policy_hash: &'a str,
    /// Duration bucket.
    pub duration: crate::router::DurationBucket,
    /// Redaction markers.
    pub redactions: &'a [&'a str],
}

/// Safe execution receipt for durable model-router evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionReceipt {
    recipe_id: String,
    family: crate::router::RouteFamily,
    status: RouterExecutionStatus,
    selected_models: Vec<String>,
    candidate_count: u32,
    success_count: u32,
    quorum: crate::router::QuorumClass,
    fallback: RouterFallbackClass,
    repair: RouterRepairClass,
    output_contract: RouterOutputContractStatus,
    policy_hash: crate::SafeHash,
    duration: crate::router::DurationBucket,
    redactions: Vec<String>,
    identity_hash: crate::SafeHash,
}

impl RouterExecutionReceipt {
    /// Builds a safe execution receipt.
    ///
    /// # Errors
    ///
    /// Returns [`RouterExecutionError`] when labels, hashes, redactions, or the
    /// contract identity are malformed.
    pub fn new(draft: &RouterExecutionReceiptDraft<'_>) -> Result<Self, RouterExecutionError> {
        let contract = safe_text(CONTRACT_FIELD, draft.contract_identity)?;
        if contract != ROUTER_EXECUTION_RECEIPT_CONTRACT_IDENTITY {
            return Err(RouterExecutionError::SchemaMismatch);
        }
        validate_redaction_count(draft.redactions.len())?;
        let recipe_id = safe_text(RECIPE_FIELD, draft.recipe_id)?;
        let selected_models = safe_text_list(MODEL_LABEL_FIELD, draft.selected_models)?;
        let policy_hash = crate::SafeHash::new(draft.policy_hash)?;
        let candidate_count = alloc::format!("{}", draft.candidate_count);
        let success_count = alloc::format!("{}", draft.success_count);
        let identity_hash = blake3_router_execution_hash(&[
            recipe_id.as_str(),
            draft.family.as_str(),
            draft.status.as_str(),
            candidate_count.as_str(),
            success_count.as_str(),
            draft.quorum.as_str(),
            draft.fallback.as_str(),
            draft.repair.as_str(),
            draft.output_contract.as_str(),
            policy_hash.as_str(),
            draft.duration.as_str(),
        ]);
        Ok(Self {
            recipe_id,
            family: draft.family,
            status: draft.status,
            selected_models,
            candidate_count: draft.candidate_count,
            success_count: draft.success_count,
            quorum: draft.quorum,
            fallback: draft.fallback,
            repair: draft.repair,
            output_contract: draft.output_contract,
            policy_hash,
            duration: draft.duration,
            redactions: safe_text_list(REDACTION_FIELD, draft.redactions)?,
            identity_hash,
        })
    }

    /// Returns recipe identity.
    #[must_use]
    pub fn recipe_id(&self) -> &str {
        self.recipe_id.as_str()
    }

    /// Returns route family.
    #[must_use]
    pub const fn family(&self) -> crate::router::RouteFamily {
        self.family
    }

    /// Returns execution status.
    #[must_use]
    pub const fn status(&self) -> RouterExecutionStatus {
        self.status
    }

    /// Returns selected model labels.
    #[must_use]
    pub fn selected_models(&self) -> &[String] {
        self.selected_models.as_slice()
    }

    /// Returns candidate count.
    #[must_use]
    pub const fn candidate_count(&self) -> u32 {
        self.candidate_count
    }

    /// Returns success count.
    #[must_use]
    pub const fn success_count(&self) -> u32 {
        self.success_count
    }

    /// Returns quorum class.
    #[must_use]
    pub const fn quorum(&self) -> crate::router::QuorumClass {
        self.quorum
    }

    /// Returns fallback class.
    #[must_use]
    pub const fn fallback(&self) -> RouterFallbackClass {
        self.fallback
    }

    /// Returns repair class.
    #[must_use]
    pub const fn repair(&self) -> RouterRepairClass {
        self.repair
    }

    /// Returns output contract status.
    #[must_use]
    pub const fn output_contract(&self) -> RouterOutputContractStatus {
        self.output_contract
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::SafeHash {
        &self.policy_hash
    }

    /// Returns duration bucket.
    #[must_use]
    pub const fn duration(&self) -> crate::router::DurationBucket {
        self.duration
    }

    /// Returns redaction markers.
    #[must_use]
    pub fn redactions(&self) -> &[String] {
        self.redactions.as_slice()
    }

    /// Returns receipt identity hash.
    #[must_use]
    pub const fn identity_hash(&self) -> &crate::SafeHash {
        &self.identity_hash
    }
}

/// Bounded overall execution outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionOutcome {
    status: RouterExecutionStatus,
    step_outcomes: Vec<RouterStepOutcome>,
    fallback: RouterFallbackClass,
    repair: RouterRepairClass,
    receipt: RouterExecutionReceipt,
}

/// Borrowed input used to build an execution outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterExecutionOutcomeInput {
    /// Overall execution status.
    pub status: RouterExecutionStatus,
    /// Step outcomes.
    pub step_outcomes: Vec<RouterStepOutcome>,
    /// Fallback class.
    pub fallback: RouterFallbackClass,
    /// Repair class.
    pub repair: RouterRepairClass,
    /// Safe receipt.
    pub receipt: RouterExecutionReceipt,
}

impl RouterExecutionOutcome {
    /// Builds a bounded execution outcome.
    ///
    /// # Errors
    ///
    /// Returns [`RouterExecutionError`] when status and receipt data disagree.
    pub fn new(input: RouterExecutionOutcomeInput) -> Result<Self, RouterExecutionError> {
        let success_count = input
            .step_outcomes
            .iter()
            .filter(|outcome| outcome.status() == RouterStepOutcomeStatus::Success)
            .count();
        let success_count = u32::try_from(success_count)
            .map_err(|_error| RouterExecutionError::MalformedOutcome)?;
        if success_count != input.receipt.success_count() {
            return Err(RouterExecutionError::MalformedOutcome);
        }
        if input.status != input.receipt.status() {
            return Err(RouterExecutionError::MalformedOutcome);
        }
        Ok(Self {
            status: input.status,
            step_outcomes: input.step_outcomes,
            fallback: input.fallback,
            repair: input.repair,
            receipt: input.receipt,
        })
    }

    /// Returns overall status.
    #[must_use]
    pub const fn status(&self) -> RouterExecutionStatus {
        self.status
    }

    /// Returns step outcomes.
    #[must_use]
    pub fn step_outcomes(&self) -> &[RouterStepOutcome] {
        self.step_outcomes.as_slice()
    }

    /// Returns fallback class.
    #[must_use]
    pub const fn fallback(&self) -> RouterFallbackClass {
        self.fallback
    }

    /// Returns repair class.
    #[must_use]
    pub const fn repair(&self) -> RouterRepairClass {
        self.repair
    }

    /// Returns safe receipt.
    #[must_use]
    pub const fn receipt(&self) -> &RouterExecutionReceipt {
        &self.receipt
    }
}

/// Host-injected model-router execution service.
pub trait RouterExecutionService {
    /// Executes a bounded router dispatch request.
    ///
    /// # Errors
    ///
    /// Returns [`RouterExecutionError`] when the host denies execution, is
    /// unavailable, times out, is cancelled, or returns malformed bounded data.
    fn execute_router_plan(
        &mut self,
        request: &RouterExecutionDispatchRequest,
    ) -> Result<RouterExecutionOutcome, RouterExecutionError>;
}

/// Executes a router plan through an injected service.
///
/// # Errors
///
/// Returns [`RouterExecutionError::MissingService`] when no service was supplied,
/// or the injected service's typed fail-closed error.
pub fn execute_router_plan(
    service: Option<&mut dyn RouterExecutionService>,
    request: &RouterExecutionDispatchRequest,
) -> Result<RouterExecutionOutcome, RouterExecutionError> {
    let Some(service) = service else {
        return Err(RouterExecutionError::MissingService);
    };
    service.execute_router_plan(request)
}

/// Builds a BLAKE3 safe hash from deterministic safe parts.
#[must_use]
pub fn blake3_router_execution_hash(parts: &[&str]) -> crate::SafeHash {
    let mut canonical = String::new();
    for part in parts {
        canonical.push_str(part);
        canonical.push_str(HASH_SEPARATOR);
    }
    crate::SafeHash::from_blake3_bytes(canonical.as_bytes())
}

/// Validates free-form router execution receipt text for durable evidence.
///
/// # Errors
///
/// Returns [`RouterExecutionError::UnsafeReceipt`] when raw prompts, provider
/// payloads, worker outputs, judge rationales, hidden reasoning, credentials,
/// full paths, or unbounded traces are present.
pub fn validate_router_execution_receipt_text(text: &str) -> Result<(), RouterExecutionError> {
    let normalized = safe_text(SUMMARY_FIELD, text)?;
    reject_unsafe_text(SUMMARY_FIELD, normalized.as_str())
}

fn total_candidate_count(dispatches: &[RouterRoleDispatch]) -> Result<u32, RouterExecutionError> {
    let mut total = 0_u32;
    for dispatch in dispatches {
        let count = u32::try_from(dispatch.candidate_labels().len())
            .map_err(|_error| RouterExecutionError::BudgetExceeded)?;
        total = total
            .checked_add(count)
            .ok_or(RouterExecutionError::BudgetExceeded)?;
    }
    Ok(total)
}

fn validate_redaction_count(actual_items: usize) -> Result<(), RouterExecutionError> {
    if actual_items > MAX_REDACTION_MARKERS {
        return Err(RouterExecutionError::TooManyItems {
            field: REDACTION_FIELD.as_str(),
            max_items: MAX_REDACTION_MARKERS,
            actual_items,
        });
    }
    Ok(())
}

fn safe_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<String, RouterExecutionError> {
    let normalized = crate::validation::normalized_adapter_field(field, value)?;
    reject_unsafe_text(field, normalized.as_str())?;
    Ok(normalized)
}

fn safe_text_list(
    field: crate::validation::FieldName,
    values: &[&str],
) -> Result<Vec<String>, RouterExecutionError> {
    values.iter().map(|value| safe_text(field, value)).collect()
}

fn reject_unsafe_text(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<(), RouterExecutionError> {
    for marker in UNSAFE_RECEIPT_MARKERS {
        if contains_unsafe_marker(value, marker) {
            return Err(RouterExecutionError::UnsafeReceipt {
                field: field.as_str(),
                marker,
            });
        }
    }
    Ok(())
}

#[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
fn contains_unsafe_marker(value: &str, marker: &'static str) -> bool {
    if marker == OPENAI_KEY_PREFIX_MARKER {
        return value.starts_with(marker) || value.contains(" sk-");
    }
    value.contains(marker)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    const RECIPE_SINGLE: &str = "recipe:single";
    const RECIPE_REMOM: &str = "recipe:remom";
    const MODEL_PRIMARY: &str = "model:primary";
    const MODEL_WORKER_A: &str = "model:worker:a";
    const MODEL_WORKER_B: &str = "model:worker:b";
    const MODEL_SYNTHESIZER: &str = "model:synthesizer";
    const MODEL_SYNTHESIZER_B: &str = "model:synthesizer:b";
    const MODEL_FALLBACK: &str = "model:fallback";
    const BACKEND_LOCAL: &str = "backend:local";
    const BACKEND_REMOTE: &str = "backend:remote";
    const REQUEST_HANDLE: &str = "request:hash-only";
    const CORRELATION: &str = "correlation:router-fixture";
    const REDACTION: &str = "prompt_omitted";
    const POLICY_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";
    const MAX_CANDIDATES_FOUR: u32 = 4;
    const MAX_PARALLELISM_TWO: u32 = 2;
    const QUORUM_ONE: u32 = 1;
    const QUORUM_TWO: u32 = 2;
    const SYNTHESIS_ONE: u32 = 1;
    const REPAIR_ONE: u32 = 1;
    const EXPECTED_TWO_ITEMS: usize = 2;

    #[derive(Default)]
    struct FixtureExecutor {
        calls: usize,
    }

    struct FailingExecutor {
        error: RouterExecutionError,
    }

    impl RouterExecutionService for FailingExecutor {
        fn execute_router_plan(
            &mut self,
            _request: &RouterExecutionDispatchRequest,
        ) -> Result<RouterExecutionOutcome, RouterExecutionError> {
            Err(self.error.clone())
        }
    }

    impl RouterExecutionService for FixtureExecutor {
        fn execute_router_plan(
            &mut self,
            request: &RouterExecutionDispatchRequest,
        ) -> Result<RouterExecutionOutcome, RouterExecutionError> {
            self.calls = self
                .calls
                .checked_add(1)
                .expect("fixture calls remain bounded");
            let mut outcomes = Vec::with_capacity(request.role_dispatches().len());
            for dispatch in request.role_dispatches() {
                let first = dispatch
                    .candidate_labels()
                    .first()
                    .expect("validated dispatch has candidates");
                outcomes.push(step_outcome(
                    dispatch.role(),
                    first.as_str(),
                    RouterStepOutcomeStatus::Success,
                )?);
            }
            let receipt = execution_receipt(request, RouterExecutionStatus::Completed, &outcomes)?;
            RouterExecutionOutcome::new(RouterExecutionOutcomeInput {
                status: RouterExecutionStatus::Completed,
                step_outcomes: outcomes,
                fallback: RouterFallbackClass::None,
                repair: RouterRepairClass::None,
                receipt,
            })
        }
    }

    #[test]
    fn fixture_executor_executes_plan_fanout_fallback_and_safe_receipt() {
        // r[verify agent_harness_runtime.model_router_execution.service_contract]
        // r[verify agent_harness_runtime.model_router_execution.dispatch_envelopes]
        // r[verify agent_harness_runtime.model_router_execution.fanout_quorum]
        // r[verify agent_harness_runtime.model_router_execution.fallback_repair]
        // r[verify agent_harness_runtime.model_router_execution.receipts]
        // r[verify agent_harness_runtime.model_router_execution.verification]
        let plan = remom_plan(QUORUM_ONE);
        let ledger = crate::router::BudgetLedger::from(plan.bounds());
        let request = RouterExecutionDispatchRequest::new(RouterExecutionDispatchInput {
            plan: &plan,
            request_handle: REQUEST_HANDLE,
            control: control(RouterExecutionCancellation::Cancellable),
            ledger: &ledger,
            fallback_policy: crate::router::FallbackPolicy::BestValidEvidence,
        })
        .expect("dispatch request should validate");
        assert_eq!(request.role_dispatches().len(), EXPECTED_TWO_ITEMS);
        assert_eq!(
            request.output_contract(),
            crate::router::OutputContract::Json
        );

        let mut executor = FixtureExecutor::default();
        let outcome = execute_router_plan(Some(&mut executor), &request)
            .expect("fixture executor should complete");
        assert_eq!(outcome.status(), RouterExecutionStatus::Completed);
        assert_eq!(outcome.receipt().quorum(), crate::router::QuorumClass::Met);
        assert_eq!(executor.calls, 1);

        let fallback = evaluate_router_execution_fallback(&RouterFallbackEvaluationInput {
            failed_model: MODEL_WORKER_A,
            repairable: false,
            ledger: &ledger,
            chain: plan.fallback_chain(),
            policy: crate::router::FallbackPolicy::BestValidEvidence,
            provider_bypass_claimed: false,
        })
        .expect("fallback evaluation should pass");
        assert!(matches!(
            fallback,
            crate::router::FallbackDecision::FallbackTo(_)
        ));
        assert!(
            validate_router_execution_receipt_text(
                "recipe=remom status=completed redactions=prompt_omitted"
            )
            .is_ok()
        );
    }

    #[test]
    fn missing_denied_and_unavailable_services_fail_closed() {
        // r[verify agent_harness_runtime.model_router_execution.service_contract]
        // r[verify agent_harness_runtime.model_router_execution.verification]
        let request = dispatch_request(QUORUM_TWO);
        let missing = execute_router_plan(None, &request).expect_err("missing service should fail");
        assert_eq!(missing, RouterExecutionError::MissingService);

        let mut denied = FailingExecutor {
            error: RouterExecutionError::ServiceDenied,
        };
        let denied_error = execute_router_plan(Some(&mut denied), &request)
            .expect_err("denied service should fail closed");
        assert_eq!(denied_error, RouterExecutionError::ServiceDenied);
        let mut unavailable = FailingExecutor {
            error: RouterExecutionError::ServiceUnavailable,
        };
        let unavailable_error = execute_router_plan(Some(&mut unavailable), &request)
            .expect_err("unavailable service should fail closed");
        assert_eq!(unavailable_error, RouterExecutionError::ServiceUnavailable);
    }

    #[test]
    fn ambient_candidates_overparallel_quorum_and_bypass_fail_closed() {
        // r[verify agent_harness_runtime.model_router_execution.fanout_quorum]
        // r[verify agent_harness_runtime.model_router_execution.fallback_repair]
        // r[verify agent_harness_runtime.model_router_execution.verification]
        let plan = remom_plan(QUORUM_TWO);
        let ledger = crate::router::BudgetLedger::from(plan.bounds());
        let first_step = plan.role_steps().first().expect("plan has a role step");
        let ambient = authorize_router_role_fanout(
            first_step,
            &["model:ambient", MODEL_WORKER_A],
            MAX_PARALLELISM_TWO,
        )
        .expect_err("ambient candidate should fail");
        assert_eq!(ambient, RouterExecutionError::UnauthorizedCandidate);
        let over_parallel =
            authorize_router_role_fanout(first_step, &[MODEL_WORKER_A, MODEL_WORKER_B], QUORUM_ONE)
                .expect_err("over-parallel fan-out should fail");
        assert_eq!(over_parallel, RouterExecutionError::ParallelismExceeded);
        let impossible_quorum =
            authorize_router_role_fanout(first_step, &[MODEL_WORKER_A], QUORUM_TWO)
                .expect_err("single candidate cannot satisfy quorum two");
        assert_eq!(impossible_quorum, RouterExecutionError::ImpossibleQuorum);

        let bypass = evaluate_router_execution_fallback(&RouterFallbackEvaluationInput {
            failed_model: MODEL_WORKER_A,
            repairable: false,
            ledger: &ledger,
            chain: plan.fallback_chain(),
            policy: crate::router::FallbackPolicy::BestValidEvidence,
            provider_bypass_claimed: true,
        })
        .expect_err("provider fallback bypass should fail");
        assert_eq!(bypass, RouterExecutionError::ProviderBypassDenied);
    }

    #[test]
    fn receipt_leaks_and_schema_mismatch_fail_closed() {
        // r[verify agent_harness_runtime.model_router_execution.receipts]
        // r[verify agent_harness_runtime.model_router_execution.verification]
        let leak = validate_router_execution_receipt_text("WORKER_OUTPUT: raw answer")
            .expect_err("raw worker output should fail");
        assert!(matches!(leak, RouterExecutionError::UnsafeReceipt { .. }));
        let bad_schema = RouterExecutionReceipt::new(&RouterExecutionReceiptDraft {
            contract_identity: "animus.router-execution-receipt.v0",
            recipe_id: RECIPE_REMOM,
            family: crate::router::RouteFamily::Remom,
            status: RouterExecutionStatus::Denied,
            selected_models: &[MODEL_WORKER_A],
            candidate_count: QUORUM_ONE,
            success_count: 0,
            quorum: crate::router::QuorumClass::Missing,
            fallback: RouterFallbackClass::Denied,
            repair: RouterRepairClass::Denied,
            output_contract: RouterOutputContractStatus::Missing,
            policy_hash: POLICY_HASH,
            duration: crate::router::DurationBucket::Bounded,
            redactions: &[REDACTION],
        })
        .expect_err("schema mismatch should fail");
        assert_eq!(bad_schema, RouterExecutionError::SchemaMismatch);
    }

    fn dispatch_request(quorum: u32) -> RouterExecutionDispatchRequest {
        let plan = remom_plan(quorum);
        let ledger = crate::router::BudgetLedger::from(plan.bounds());
        RouterExecutionDispatchRequest::new(RouterExecutionDispatchInput {
            plan: &plan,
            request_handle: REQUEST_HANDLE,
            control: control(RouterExecutionCancellation::Cancellable),
            ledger: &ledger,
            fallback_policy: crate::router::FallbackPolicy::BestValidEvidence,
        })
        .expect("dispatch request should validate")
    }

    fn control(cancellation: RouterExecutionCancellation) -> RouterExecutionControl {
        RouterExecutionControl::new(RouterExecutionControlInput {
            cancellation,
            deadline: crate::router::DeadlineClass::Standard,
            correlation: CORRELATION,
            policy_hash: POLICY_HASH,
        })
        .expect("control should validate")
    }

    fn remom_plan(quorum: u32) -> crate::router::RouterExecutionPlan {
        assert!(quorum >= QUORUM_ONE);
        assert!(quorum <= MAX_PARALLELISM_TWO);
        let recipes = vec![
            recipe(RECIPE_SINGLE, "single", &["primary"], QUORUM_ONE),
            recipe(RECIPE_REMOM, "remom", &["worker", "synthesizer"], quorum),
        ];
        let catalog = catalog();
        let budget = budget();
        let signals = crate::router::Signals::project(crate::router::SignalDraft {
            task_shape: "reasoning",
            risk_band: "high",
            contract_pressure: "strict",
            context_pressure: "medium",
            latency_band: "medium",
            cost_band: "medium",
            privacy_band: "remote_allowed",
            fallback_eligibility: "eligible",
        })
        .expect("signals should project");
        let selected = crate::router::select_recipe(&signals, &recipes, &catalog, &budget)
            .expect("recipe should select");
        crate::router::materialize_execution_plan(&selected, &catalog, &budget)
            .expect("execution plan should materialize")
    }

    #[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
    fn recipe(id: &str, family: &str, roles: &[&str], quorum: u32) -> crate::router::Recipe {
        crate::router::Recipe::new(crate::router::RecipeDraft {
            id,
            family,
            roles,
            output_contract: "json",
            max_candidates: MAX_CANDIDATES_FOUR,
            max_parallelism: MAX_PARALLELISM_TWO,
            min_success_quorum: quorum,
            timeout_class: "standard",
            max_synthesis_attempts: SYNTHESIS_ONE,
            max_repair_attempts: Some(REPAIR_ONE),
            fallback_policy: "best_valid_evidence",
        })
        .expect("recipe should validate")
    }

    fn catalog() -> Vec<crate::router::CatalogEntry> {
        vec![
            catalog_entry(MODEL_PRIMARY, BACKEND_LOCAL, "primary", true),
            catalog_entry(MODEL_WORKER_A, BACKEND_REMOTE, "worker", false),
            catalog_entry(MODEL_WORKER_B, BACKEND_REMOTE, "worker", false),
            catalog_entry(MODEL_SYNTHESIZER, BACKEND_REMOTE, "synthesizer", false),
            catalog_entry(MODEL_SYNTHESIZER_B, BACKEND_REMOTE, "synthesizer", false),
            catalog_entry(MODEL_FALLBACK, BACKEND_LOCAL, "fallback", true),
        ]
    }

    #[cfg_attr(animus_octet, allow(tigerstyle::ambiguous_params))]
    fn catalog_entry(
        label: &str,
        backend: &str,
        role: &str,
        local: bool,
    ) -> crate::router::CatalogEntry {
        crate::router::CatalogEntry::new(crate::router::CatalogEntryDraft {
            label,
            backend,
            role,
            local,
        })
        .expect("catalog entry should validate")
    }

    fn budget() -> crate::router::Budget {
        crate::router::Budget::new(MAX_CANDIDATES_FOUR, &[BACKEND_LOCAL, BACKEND_REMOTE])
            .expect("budget should validate")
    }

    fn step_outcome(
        role: crate::router::Role,
        candidate_label: &str,
        status: RouterStepOutcomeStatus,
    ) -> Result<RouterStepOutcome, RouterExecutionError> {
        let hash = blake3_router_execution_hash(&[role.as_str(), candidate_label]);
        RouterStepOutcome::new(RouterStepOutcomeInput {
            role,
            candidate_label,
            status,
            output_hash: Some(hash.as_str()),
            duration: crate::router::DurationBucket::Bounded,
            redactions: &[REDACTION],
        })
    }

    fn execution_receipt(
        request: &RouterExecutionDispatchRequest,
        status: RouterExecutionStatus,
        outcomes: &[RouterStepOutcome],
    ) -> Result<RouterExecutionReceipt, RouterExecutionError> {
        assert!(!request.selected_models().is_empty());
        assert!(outcomes.len() <= request.role_dispatches().len());
        let selected: Vec<&str> = request
            .selected_models()
            .iter()
            .map(String::as_str)
            .collect();
        let success_count = outcomes
            .iter()
            .filter(|outcome| outcome.status() == RouterStepOutcomeStatus::Success)
            .count();
        let success_count = u32::try_from(success_count)
            .map_err(|_error| RouterExecutionError::MalformedOutcome)?;
        RouterExecutionReceipt::new(&RouterExecutionReceiptDraft {
            contract_identity: ROUTER_EXECUTION_RECEIPT_CONTRACT_IDENTITY,
            recipe_id: request.recipe_id(),
            family: request.family(),
            status,
            selected_models: selected.as_slice(),
            candidate_count: request.candidate_budget_remaining(),
            success_count,
            quorum: crate::router::QuorumClass::Met,
            fallback: RouterFallbackClass::None,
            repair: RouterRepairClass::None,
            output_contract: RouterOutputContractStatus::Satisfied,
            policy_hash: request.control().policy_hash().as_str(),
            duration: crate::router::DurationBucket::Bounded,
            redactions: &[REDACTION],
        })
    }
}
