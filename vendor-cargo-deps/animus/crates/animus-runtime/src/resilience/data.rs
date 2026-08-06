//! Typed provider retry policy, decision, timer, event, and receipt values.

/// Generated retry-policy schema identity.
pub const PROVIDER_RETRY_POLICY_SCHEMA: &str = "animus.provider-retry-policy.v1";
/// BLAKE3 domain for canonical retry-policy content identity.
pub const PROVIDER_RETRY_POLICY_HASH_DOMAIN: &str = "animus.provider-retry-policy-content.v1";
/// BLAKE3 domain for exact retry invocation identity.
pub const PROVIDER_RETRY_INVOCATION_HASH_DOMAIN: &str = "animus.provider-retry-invocation.v1";
/// BLAKE3 domain for retry decisions.
pub const PROVIDER_RETRY_DECISION_HASH_DOMAIN: &str = "animus.provider-retry-decision.v1";
/// BLAKE3 domain for metadata-only retry receipts.
pub const PROVIDER_RETRY_RECEIPT_HASH_DOMAIN: &str = "animus.provider-retry-receipt.v1";
/// Retry receipt schema identity.
pub const PROVIDER_RETRY_RECEIPT_SCHEMA: &str = "animus.provider-retry-receipt.v1";

pub(super) const BLAKE3_HEX_CHARS: usize = 64;
pub(super) const MAX_ATTEMPT_COUNT: u32 = 8;
pub(super) const MAX_DELAY_MS: u64 = 300_000;
pub(super) const MAX_MAPPING_COUNT: usize = 128;
pub(super) const MAX_REASON_COUNT: usize = 64;
pub(super) const MAX_SAFE_LABEL_CHARS: usize = 64;
pub(super) const MIN_ATTEMPT_COUNT: u32 = 1;
pub(super) const HASH_SEPARATOR: &str = "\u{1f}";
pub(super) const REDACTION_MARKER: &str = "raw-provider-request-stream-credential-and-reasoning-material-omitted";
pub(super) const NON_CLAIM: &str = "retry evidence does not prove provider reliability, exactly-once billing, remote cancellation, fallback, or partial-stream resumability";

/// Canonical provider failure class admitted by retry policy.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ProviderRetryFailureClass {
    /// Provider rate limiting.
    RateLimited,
    /// Transient network or transport failure.
    TransientTransport,
    /// Provider service is temporarily unavailable.
    ServiceUnavailable,
    /// Provider request timed out before a safe commit.
    RequestTimeout,
    /// Provider reported a retryable conflict.
    Conflict,
    /// Credential authentication failed.
    Authentication,
    /// Caller is not authorized.
    Authorization,
    /// Account or model entitlement is absent.
    Entitlement,
    /// Host policy denied the request.
    PolicyDenied,
    /// Capability policy denied the request.
    CapabilityDenied,
    /// Provider request is invalid.
    InvalidRequest,
    /// Requested provider feature is unsupported.
    UnsupportedFeature,
    /// Canonical request or response is malformed.
    MalformedRequest,
    /// Provider output failed safety validation.
    UnsafeResponse,
    /// No provider service was injected.
    MissingService,
    /// Required credential handle was absent.
    MissingCredential,
    /// No admitted mapping classified the failure.
    Unknown,
}

impl ProviderRetryFailureClass {
    /// Parses a stable generated-policy label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "rate_limited" => Some(Self::RateLimited),
            "transient_transport" => Some(Self::TransientTransport),
            "service_unavailable" => Some(Self::ServiceUnavailable),
            "request_timeout" => Some(Self::RequestTimeout),
            "conflict" => Some(Self::Conflict),
            "authentication" => Some(Self::Authentication),
            "authorization" => Some(Self::Authorization),
            "entitlement" => Some(Self::Entitlement),
            "policy_denied" => Some(Self::PolicyDenied),
            "capability_denied" => Some(Self::CapabilityDenied),
            "invalid_request" => Some(Self::InvalidRequest),
            "unsupported_feature" => Some(Self::UnsupportedFeature),
            "malformed_request" => Some(Self::MalformedRequest),
            "unsafe_response" => Some(Self::UnsafeResponse),
            "missing_service" => Some(Self::MissingService),
            "missing_credential" => Some(Self::MissingCredential),
            "unknown" => Some(Self::Unknown),
            _other => None,
        }
    }

    /// Returns the stable policy and receipt label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RateLimited => "rate_limited",
            Self::TransientTransport => "transient_transport",
            Self::ServiceUnavailable => "service_unavailable",
            Self::RequestTimeout => "request_timeout",
            Self::Conflict => "conflict",
            Self::Authentication => "authentication",
            Self::Authorization => "authorization",
            Self::Entitlement => "entitlement",
            Self::PolicyDenied => "policy_denied",
            Self::CapabilityDenied => "capability_denied",
            Self::InvalidRequest => "invalid_request",
            Self::UnsupportedFeature => "unsupported_feature",
            Self::MalformedRequest => "malformed_request",
            Self::UnsafeResponse => "unsafe_response",
            Self::MissingService => "missing_service",
            Self::MissingCredential => "missing_credential",
            Self::Unknown => "unknown",
        }
    }
}

/// Generated provider-native failure mapping input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryMappingInput {
    /// Provider-family label.
    pub provider_family: String,
    /// Versioned mapping label.
    pub mapping_version: String,
    /// Safe provider-native status class.
    pub native_class: String,
    /// Canonical failure class label.
    pub canonical_class: String,
}

/// Generated provider retry policy input before Rust revalidation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryPolicyInput {
    /// Schema identity.
    pub schema: String,
    /// Policy label.
    pub policy_id: String,
    /// Total attempt bound, including the initial attempt.
    pub max_attempts: u32,
    /// Maximum admitted delay in milliseconds.
    pub max_delay_ms: u64,
    /// Delay for each possible retry after the initial attempt.
    pub delay_schedule_ms: Vec<u64>,
    /// Generated cancellation posture label.
    pub cancellation_posture: String,
    /// Generated semantic-stream posture label.
    pub stream_commit_policy: String,
    /// Canonical classes eligible for retry.
    pub retryable_classes: Vec<String>,
    /// Versioned provider-family mappings.
    pub mappings: Vec<ProviderRetryMappingInput>,
    /// Canonical policy content identity.
    pub policy_hash: String,
}

/// Validated provider-native failure mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryMapping {
    pub(super) provider_family: String,
    pub(super) mapping_version: String,
    pub(super) native_class: String,
    pub(super) canonical_class: ProviderRetryFailureClass,
}

impl ProviderRetryMapping {
    /// Returns the provider-family label.
    #[must_use]
    pub fn provider_family(&self) -> &str {
        self.provider_family.as_str()
    }

    /// Returns the mapping-version label.
    #[must_use]
    pub fn mapping_version(&self) -> &str {
        self.mapping_version.as_str()
    }

    /// Returns the safe provider-native class.
    #[must_use]
    pub fn native_class(&self) -> &str {
        self.native_class.as_str()
    }

    /// Returns the canonical class.
    #[must_use]
    pub const fn canonical_class(&self) -> ProviderRetryFailureClass {
        self.canonical_class
    }
}

/// Validated provider retry policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryPolicy {
    pub(super) policy_id: String,
    pub(super) max_attempts: u32,
    pub(super) max_delay_ms: u64,
    pub(super) delay_schedule_ms: Vec<u64>,
    pub(super) retryable_classes: Vec<ProviderRetryFailureClass>,
    pub(super) mappings: Vec<ProviderRetryMapping>,
    pub(super) policy_hash: String,
}

impl ProviderRetryPolicy {
    /// Returns the policy label.
    #[must_use]
    pub fn policy_id(&self) -> &str {
        self.policy_id.as_str()
    }

    /// Returns the total attempt bound.
    #[must_use]
    pub const fn max_attempts(&self) -> u32 {
        self.max_attempts
    }

    /// Returns the maximum delay in milliseconds.
    #[must_use]
    pub const fn max_delay_ms(&self) -> u64 {
        self.max_delay_ms
    }

    /// Returns the deterministic retry delay schedule.
    #[must_use]
    pub fn delay_schedule_ms(&self) -> &[u64] {
        self.delay_schedule_ms.as_slice()
    }

    /// Returns the canonical retryable classes.
    #[must_use]
    pub fn retryable_classes(&self) -> &[ProviderRetryFailureClass] {
        self.retryable_classes.as_slice()
    }

    /// Returns the versioned provider mappings.
    #[must_use]
    pub fn mappings(&self) -> &[ProviderRetryMapping] {
        self.mappings.as_slice()
    }

    /// Returns the canonical policy identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }
}

/// Exact invocation identity input bound across retry attempts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryInvocationInput {
    /// Provider identifier.
    pub provider_id: String,
    /// Model identifier.
    pub model_id: String,
    /// Canonical request BLAKE3 identity.
    pub request_hash: String,
    /// Model-runtime profile plan BLAKE3 identity.
    pub profile_plan_hash: String,
    /// Prompt/system BLAKE3 identity.
    pub prompt_hash: String,
    /// Tool-catalog BLAKE3 identity.
    pub tool_catalog_hash: String,
    /// Response-format BLAKE3 identity.
    pub response_format_hash: String,
    /// Reasoning-intent BLAKE3 identity.
    pub reasoning_intent_hash: String,
    /// Stream-intent BLAKE3 identity.
    pub stream_intent_hash: String,
    /// Credential-handle class BLAKE3 identity.
    pub credential_handle_hash: String,
    /// Retry-policy BLAKE3 identity.
    pub policy_hash: String,
    /// Canonical exact invocation identity.
    pub exact_plan_hash: String,
}

/// Validated exact invocation identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryInvocation {
    pub(super) provider_id: String,
    pub(super) model_id: String,
    pub(super) request_hash: String,
    pub(super) profile_plan_hash: String,
    pub(super) prompt_hash: String,
    pub(super) tool_catalog_hash: String,
    pub(super) response_format_hash: String,
    pub(super) reasoning_intent_hash: String,
    pub(super) stream_intent_hash: String,
    pub(super) credential_handle_hash: String,
    pub(super) policy_hash: String,
    pub(super) exact_plan_hash: String,
}

impl ProviderRetryInvocation {
    /// Returns the provider identifier.
    #[must_use]
    pub fn provider_id(&self) -> &str {
        self.provider_id.as_str()
    }

    /// Returns the model identifier.
    #[must_use]
    pub fn model_id(&self) -> &str {
        self.model_id.as_str()
    }

    /// Returns the canonical request identity.
    #[must_use]
    pub fn request_hash(&self) -> &str {
        self.request_hash.as_str()
    }

    /// Returns the exact invocation identity.
    #[must_use]
    pub fn exact_plan_hash(&self) -> &str {
        self.exact_plan_hash.as_str()
    }

    /// Returns the bound tool-catalog identity.
    #[must_use]
    pub fn tool_catalog_hash(&self) -> &str {
        self.tool_catalog_hash.as_str()
    }

    /// Returns the bound policy identity.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }
}

/// Safe provider-native failure observation supplied by a host adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryFailureObservation {
    /// Provider-family label.
    pub provider_family: String,
    /// Mapping-version label.
    pub mapping_version: String,
    /// Safe provider-native class label.
    pub native_class: String,
}

/// Semantic stream frame class relevant to replay safety.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetrySemanticFrame {
    /// Assistant-visible content.
    Content,
    /// Provider-exposed reasoning or thought.
    Reasoning,
    /// Any tool-call delta.
    ToolCall,
    /// Structured-output delta.
    StructuredOutput,
    /// Usage or other policy-declared non-replay-safe frame.
    Other,
}

impl ProviderRetrySemanticFrame {
    /// Returns the stable frame class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::Reasoning => "reasoning",
            Self::ToolCall => "tool_call",
            Self::StructuredOutput => "structured_output",
            Self::Other => "other",
        }
    }
}

/// Stream replay-safety state for one provider attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryStreamState {
    /// No provider transport was started.
    NotStarted,
    /// Transport started but no semantic frame was accepted.
    TransportStarted,
    /// A non-replay-safe semantic frame was accepted.
    Committed(ProviderRetrySemanticFrame),
}

/// Host stream observation reduced into replay-safety state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryStreamObservation {
    /// Provider transport started without a semantic frame.
    TransportStarted,
    /// A non-replay-safe semantic frame was accepted.
    Semantic(ProviderRetrySemanticFrame),
}

/// Correlated cancellation observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryCancellation {
    /// No correlated cancellation was accepted.
    None,
    /// A correlated cancellation was accepted before the next dispatch.
    Requested,
}

/// Retry state after a failed provider attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryAttemptState {
    /// Stable run identity.
    pub run_id: String,
    /// Number of completed provider attempts, including the failed attempt.
    pub completed_attempt_count: u32,
    /// Exact invocation identity used by the failed attempt.
    pub exact_plan_hash: String,
    /// Current stream replay-safety state.
    pub stream_state: ProviderRetryStreamState,
    /// Correlated cancellation state.
    pub cancellation: ProviderRetryCancellation,
}

/// Pure retry decision class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryDecisionStatus {
    /// Another same-plan attempt may be scheduled.
    Retry,
    /// Failure class is terminal under policy.
    Terminal,
    /// Attempt budget is exhausted.
    Exhausted,
    /// Invocation identity changed and requires fresh composition.
    RecompositionRequired,
    /// Semantic stream commit makes ordinary replay unsafe.
    PostCommitTerminal,
    /// Correlated cancellation stops retry.
    Cancelled,
}

impl ProviderRetryDecisionStatus {
    /// Returns the stable decision label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Retry => "retry",
            Self::Terminal => "terminal",
            Self::Exhausted => "exhausted",
            Self::RecompositionRequired => "recomposition_required",
            Self::PostCommitTerminal => "post_commit_terminal",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Inputs to one pure retry decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderRetryPlanInput<'a> {
    /// Validated retry policy.
    pub policy: &'a ProviderRetryPolicy,
    /// Exact invocation used by the failed attempt.
    pub current: &'a ProviderRetryInvocation,
    /// Exact invocation proposed for the next attempt.
    pub next: &'a ProviderRetryInvocation,
    /// State after the failed attempt.
    pub state: &'a ProviderRetryAttemptState,
    /// Safe provider failure observation.
    pub observation: &'a ProviderRetryFailureObservation,
}

/// Pure bounded retry decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryDecision {
    pub(super) status: ProviderRetryDecisionStatus,
    pub(super) reason: ProviderRetryFailureClass,
    pub(super) run_id: String,
    pub(super) completed_attempt_count: u32,
    pub(super) next_attempt_count: Option<u32>,
    pub(super) max_attempt_count: u32,
    pub(super) delay_ms: Option<u64>,
    pub(super) exact_plan_hash: String,
    pub(super) policy_hash: String,
    pub(super) decision_hash: String,
}

impl ProviderRetryDecision {
    /// Returns the decision status.
    #[must_use]
    pub const fn status(&self) -> ProviderRetryDecisionStatus {
        self.status
    }

    /// Returns the canonical failure class.
    #[must_use]
    pub const fn reason(&self) -> ProviderRetryFailureClass {
        self.reason
    }

    /// Returns the next attempt count when retry is admitted.
    #[must_use]
    pub const fn next_attempt_count(&self) -> Option<u32> {
        self.next_attempt_count
    }

    /// Returns the planned delay in milliseconds.
    #[must_use]
    pub const fn delay_ms(&self) -> Option<u64> {
        self.delay_ms
    }

    /// Returns the exact invocation identity.
    #[must_use]
    pub fn exact_plan_hash(&self) -> &str {
        self.exact_plan_hash.as_str()
    }

    /// Returns the decision identity.
    #[must_use]
    pub fn decision_hash(&self) -> &str {
        self.decision_hash.as_str()
    }
}

/// Host timer request for an admitted retry decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryTimerRequest {
    /// Stable run identity.
    pub run_id: String,
    /// Next exact attempt count.
    pub next_attempt_count: u32,
    /// Bounded delay in milliseconds.
    pub delay_ms: u64,
    /// Exact invocation identity.
    pub exact_plan_hash: String,
    /// Retry decision identity.
    pub decision_hash: String,
    /// Cancellation must be observed during the wait.
    pub cancellation_required: bool,
}

/// Threaded retry-wait lifecycle status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryWaitStatus {
    /// Timer outcome has not yet been accepted.
    Waiting,
    /// One correlated completion was accepted.
    Completed,
    /// Correlated cancellation closed the wait.
    Cancelled,
    /// Timer timeout, absence, or failure closed the wait.
    Blocked,
}

/// Threaded state for one exact retry wait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryWaitState {
    pub(super) run_id: String,
    pub(super) next_attempt_count: u32,
    pub(super) exact_plan_hash: String,
    pub(super) decision_hash: String,
    pub(super) status: ProviderRetryWaitStatus,
}

impl ProviderRetryWaitState {
    /// Returns the current wait lifecycle status.
    #[must_use]
    pub const fn status(&self) -> ProviderRetryWaitStatus {
        self.status
    }
}

/// Initial injected-timer request and its exact threaded state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryWaitPlan {
    pub(super) request: ProviderRetryTimerRequest,
    pub(super) state: ProviderRetryWaitState,
}

impl ProviderRetryWaitPlan {
    /// Returns the injected host timer request.
    #[must_use]
    pub const fn request(&self) -> &ProviderRetryTimerRequest {
        &self.request
    }

    /// Returns the initial exact threaded wait state.
    #[must_use]
    pub const fn state(&self) -> &ProviderRetryWaitState {
        &self.state
    }
}

/// Host timer outcome class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryTimerOutcome {
    /// Delay completed and dispatch may be considered.
    Completed,
    /// Correlated cancellation interrupted the wait.
    Cancelled,
    /// Host deadline expired.
    TimedOut,
    /// No timer service was injected or admitted.
    Unavailable,
    /// Timer service failed.
    Failed,
}

impl ProviderRetryTimerOutcome {
    /// Returns the stable timer label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Unavailable => "unavailable",
            Self::Failed => "failed",
        }
    }
}

/// Correlated host timer observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryTimerObservation {
    /// Stable run identity.
    pub run_id: String,
    /// Attempt count supplied to the timer.
    pub next_attempt_count: u32,
    /// Exact invocation identity supplied to the timer.
    pub exact_plan_hash: String,
    /// Retry decision identity supplied to the timer.
    pub decision_hash: String,
    /// Timer outcome.
    pub outcome: ProviderRetryTimerOutcome,
}

/// Provider dispatch gate after timer reduction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryDispatchGateStatus {
    /// Host may dispatch the same exact plan.
    Ready,
    /// Cancellation stopped the retry.
    Cancelled,
    /// Timer failed, timed out, or was unavailable.
    BlockedTimer,
    /// Timer observation did not match the decision.
    BlockedStale,
}

/// Exact provider dispatch request returned only after timer completion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryDispatchRequest {
    pub(super) run_id: String,
    pub(super) attempt_count: u32,
    pub(super) exact_plan_hash: String,
    pub(super) decision_hash: String,
}

impl ProviderRetryDispatchRequest {
    /// Returns the stable run identity.
    #[must_use]
    pub fn run_id(&self) -> &str {
        self.run_id.as_str()
    }

    /// Returns the exact next attempt count.
    #[must_use]
    pub const fn attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Returns the exact invocation identity.
    #[must_use]
    pub fn exact_plan_hash(&self) -> &str {
        self.exact_plan_hash.as_str()
    }

    /// Returns the retry decision identity.
    #[must_use]
    pub fn decision_hash(&self) -> &str {
        self.decision_hash.as_str()
    }
}

/// Provider dispatch gate result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryDispatchGate {
    pub(super) status: ProviderRetryDispatchGateStatus,
    pub(super) request: Option<ProviderRetryDispatchRequest>,
}

impl ProviderRetryDispatchGate {
    /// Returns the gate status.
    #[must_use]
    pub const fn status(&self) -> ProviderRetryDispatchGateStatus {
        self.status
    }

    /// Returns an exact dispatch request only when ready.
    #[must_use]
    pub const fn request(&self) -> Option<&ProviderRetryDispatchRequest> {
        self.request.as_ref()
    }
}

/// One pure timer reduction with the next threaded state and dispatch gate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryTimerEvaluation {
    pub(super) gate: ProviderRetryDispatchGate,
    pub(super) state: ProviderRetryWaitState,
}

impl ProviderRetryTimerEvaluation {
    /// Returns the dispatch gate produced by this observation.
    #[must_use]
    pub const fn gate(&self) -> &ProviderRetryDispatchGate {
        &self.gate
    }

    /// Returns the next threaded wait state.
    #[must_use]
    pub const fn state(&self) -> &ProviderRetryWaitState {
        &self.state
    }
}

/// Provider-neutral retry progress status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryProgressStatus {
    /// Retry was admitted and scheduled.
    Scheduled,
    /// Retry wait was cancelled.
    Cancelled,
    /// Same-plan provider dispatch is ready.
    DispatchReady,
    /// Failure is terminal under policy.
    Terminal,
    /// Attempt budget is exhausted.
    Exhausted,
    /// Stream commit blocked replay.
    PostCommit,
    /// Fresh composition is required.
    RecompositionRequired,
    /// Timer blocked retry.
    TimerBlocked,
}

impl ProviderRetryProgressStatus {
    /// Parses a stable event status label.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "scheduled" => Some(Self::Scheduled),
            "cancelled" => Some(Self::Cancelled),
            "dispatch_ready" => Some(Self::DispatchReady),
            "terminal" => Some(Self::Terminal),
            "exhausted" => Some(Self::Exhausted),
            "post_commit" => Some(Self::PostCommit),
            "recomposition_required" => Some(Self::RecompositionRequired),
            "timer_blocked" => Some(Self::TimerBlocked),
            _other => None,
        }
    }

    /// Returns the stable event status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Cancelled => "cancelled",
            Self::DispatchReady => "dispatch_ready",
            Self::Terminal => "terminal",
            Self::Exhausted => "exhausted",
            Self::PostCommit => "post_commit",
            Self::RecompositionRequired => "recomposition_required",
            Self::TimerBlocked => "timer_blocked",
        }
    }
}

/// Safe provider-neutral retry progress event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryProgressEvent {
    /// Stable run identity.
    pub run_id: String,
    /// Current or next attempt count.
    pub attempt_count: u32,
    /// Maximum total attempt count.
    pub max_attempt_count: u32,
    /// Planned delay in milliseconds when applicable.
    pub delay_ms: Option<u64>,
    /// Canonical reason class.
    pub reason: ProviderRetryFailureClass,
    /// Provider-neutral status.
    pub status: ProviderRetryProgressStatus,
    /// Exact invocation identity.
    pub exact_plan_hash: String,
}

/// Receipt terminal status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryReceiptStatus {
    /// A later provider attempt succeeded.
    Succeeded,
    /// Retry budget was exhausted.
    Exhausted,
    /// Failure was terminal without retry.
    Terminal,
    /// Correlated cancellation stopped retry.
    Cancelled,
    /// Timer or injected service blocked retry.
    Blocked,
}

impl ProviderRetryReceiptStatus {
    /// Returns the stable receipt status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Exhausted => "exhausted",
            Self::Terminal => "terminal",
            Self::Cancelled => "cancelled",
            Self::Blocked => "blocked",
        }
    }
}

/// Private material that must never enter a retry receipt.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProviderRetrySensitiveDraft<'a> {
    /// Raw canonical or provider request.
    pub raw_request: Option<&'a str>,
    /// Raw response body.
    pub response_body: Option<&'a str>,
    /// Raw response headers.
    pub response_headers: Option<&'a str>,
    /// Provider endpoint.
    pub endpoint: Option<&'a str>,
    /// Credential or secret.
    pub credential: Option<&'a str>,
    /// SDK exception body.
    pub exception_body: Option<&'a str>,
    /// Prompt or system text.
    pub prompt: Option<&'a str>,
    /// Hidden reasoning.
    pub reasoning: Option<&'a str>,
    /// Tool body.
    pub tool_body: Option<&'a str>,
    /// Private host path.
    pub private_path: Option<&'a str>,
}

/// Metadata-only retry receipt input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderRetryReceiptInput<'a> {
    /// Validated policy identity.
    pub policy_hash: &'a str,
    /// Exact invocation identity.
    pub exact_plan_hash: &'a str,
    /// Number of completed provider attempts.
    pub completed_attempt_count: u32,
    /// Canonical reason sequence.
    pub reason_classes: &'a [ProviderRetryFailureClass],
    /// Last timer outcome, when any wait occurred.
    pub timer_outcome: Option<ProviderRetryTimerOutcome>,
    /// Final retry status.
    pub status: ProviderRetryReceiptStatus,
    /// Forbidden private material.
    pub sensitive: ProviderRetrySensitiveDraft<'a>,
}

/// Metadata-only provider retry receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryReceipt {
    pub(super) schema: &'static str,
    pub(super) policy_hash: String,
    pub(super) exact_plan_hash: String,
    pub(super) completed_attempt_count: u32,
    pub(super) reason_classes: Vec<ProviderRetryFailureClass>,
    pub(super) timer_outcome: Option<ProviderRetryTimerOutcome>,
    pub(super) status: ProviderRetryReceiptStatus,
    pub(super) redaction_marker: &'static str,
    pub(super) non_claim: &'static str,
    pub(super) receipt_hash: String,
}

impl ProviderRetryReceipt {
    /// Returns the receipt schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Returns the final receipt status.
    #[must_use]
    pub const fn status(&self) -> ProviderRetryReceiptStatus {
        self.status
    }

    /// Returns the completed attempt count.
    #[must_use]
    pub const fn completed_attempt_count(&self) -> u32 {
        self.completed_attempt_count
    }

    /// Returns the canonical reason sequence.
    #[must_use]
    pub fn reason_classes(&self) -> &[ProviderRetryFailureClass] {
        self.reason_classes.as_slice()
    }

    /// Returns the explicit redaction marker.
    #[must_use]
    pub const fn redaction_marker(&self) -> &'static str {
        self.redaction_marker
    }

    /// Returns the explicit non-claim.
    #[must_use]
    pub const fn non_claim(&self) -> &'static str {
        self.non_claim
    }

    /// Returns the receipt identity.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Typed provider retry validation or admission issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderRetryIssue {
    /// Input shape, label, count, or hash was invalid.
    InvalidInput,
    /// Generated policy content identity was stale.
    StalePolicy,
    /// Invocation content identity was stale.
    StaleInvocation,
    /// Provider-family mapping was duplicated or malformed.
    InvalidMapping,
    /// Attempt or delay bound was invalid.
    InvalidSchedule,
    /// Timer observation was stale or mismatched.
    StaleTimer,
    /// Receipt input contained private material.
    ReceiptLeak,
}

/// Safe provider retry error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRetryError {
    issue: ProviderRetryIssue,
    field: &'static str,
}

impl ProviderRetryError {
    pub(super) const fn new(issue: ProviderRetryIssue, field: &'static str) -> Self {
        Self { issue, field }
    }

    /// Returns the issue class.
    #[must_use]
    pub const fn issue(&self) -> ProviderRetryIssue {
        self.issue
    }

    /// Returns the safe field label.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }
}

impl core::fmt::Display for ProviderRetryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "provider retry rejected {}", self.field)
    }
}

impl std::error::Error for ProviderRetryError {}
