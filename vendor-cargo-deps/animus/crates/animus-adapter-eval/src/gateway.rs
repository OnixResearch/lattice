//! Pure bounded model gateway projections and ordering gates.

use alloc::{format, string::String};

use animus_eval_core::{hash_identity, validate_identity};

/// Maximum token count accepted in one bounded usage projection.
pub const MAX_MODEL_CALL_TOKENS: u64 = 16_777_216;

/// Reviewed model protocol dialect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDialect {
    /// OpenAI-compatible chat completions.
    OpenAiChatCompletions,
    /// `OpenAI` Responses.
    OpenAiResponses,
    /// Anthropic Messages.
    AnthropicMessages,
}

impl ModelDialect {
    /// Return the stable dialect label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::OpenAiChatCompletions => "openai_chat_completions",
            Self::OpenAiResponses => "openai_responses",
            Self::AnthropicMessages => "anthropic_messages",
        }
    }

    /// Return the reviewed endpoint class.
    #[must_use]
    pub const fn endpoint(self) -> EndpointClass {
        match self {
            Self::OpenAiChatCompletions => EndpointClass::ChatCompletions,
            Self::OpenAiResponses => EndpointClass::Responses,
            Self::AnthropicMessages => EndpointClass::Messages,
        }
    }
}

/// Canonical model endpoint class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointClass {
    /// Chat-completions model turn.
    ChatCompletions,
    /// Responses model turn.
    Responses,
    /// Anthropic Messages model turn.
    Messages,
    /// Auxiliary non-model endpoint.
    Auxiliary,
}

impl EndpointClass {
    const fn label(self) -> &'static str {
        match self {
            Self::ChatCompletions => "chat_completions",
            Self::Responses => "responses",
            Self::Messages => "messages",
            Self::Auxiliary => "auxiliary",
        }
    }
}

/// Canonical finish class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelFinishClass {
    /// Normal stop.
    Stop,
    /// Output length limit.
    Length,
    /// Tool call requested.
    ToolCalls,
    /// Provider refusal.
    Refusal,
    /// No terminal finish was observed.
    Missing,
}

impl ModelFinishClass {
    const fn label(self) -> &'static str {
        match self {
            Self::Stop => "stop",
            Self::Length => "length",
            Self::ToolCalls => "tool_calls",
            Self::Refusal => "refusal",
            Self::Missing => "missing",
        }
    }
}

/// Safe provider error class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelErrorClass {
    /// No model-call error.
    None,
    /// Provider policy denial.
    Denied,
    /// Provider or route unavailable.
    Unavailable,
    /// Request was malformed.
    Malformed,
    /// Request exceeded a bound.
    Overflow,
    /// Call was cancelled.
    Cancelled,
    /// Call reached its deadline.
    TimedOut,
    /// Other bounded infrastructure failure.
    Infrastructure,
}

impl ModelErrorClass {
    const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Denied => "denied",
            Self::Unavailable => "unavailable",
            Self::Malformed => "malformed",
            Self::Overflow => "overflow",
            Self::Cancelled => "cancelled",
            Self::TimedOut => "timed_out",
            Self::Infrastructure => "infrastructure",
        }
    }
}

/// Finite token usage supplied by a host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelUsage {
    /// Non-cached input tokens.
    pub input_tokens: u64,
    /// Output tokens.
    pub output_tokens: u64,
    /// Cached input tokens.
    pub cached_input_tokens: u64,
    /// Reasoning tokens reported by the provider.
    pub reasoning_tokens: u64,
}

/// Bounded model-call projection input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelCallProjectionDraft<'a> {
    /// Selected dialect.
    pub dialect: ModelDialect,
    /// Exact endpoint class observed by the host.
    pub endpoint: EndpointClass,
    /// Effective model identity after host policy.
    pub effective_model_identity: &'a str,
    /// Canonical sampling identity.
    pub sampling_identity: &'a str,
    /// Tool-catalog identity.
    pub tool_catalog_identity: &'a str,
    /// Canonical request identity.
    pub request_identity: &'a str,
    /// Exact run identity.
    pub run_identity: &'a str,
    /// Exact policy identity.
    pub policy_identity: &'a str,
    /// Positive attempt number.
    pub attempt: u32,
    /// Whether the call used bounded streaming.
    pub streaming: bool,
    /// Usage metadata.
    pub usage: ModelUsage,
    /// Finish class.
    pub finish: ModelFinishClass,
    /// Error class.
    pub error: ModelErrorClass,
    /// Whether any raw provider body would be retained.
    pub retains_raw_provider_body: bool,
    /// Whether the projection would rewrite a provider response.
    pub rewrites_response: bool,
    /// Whether model and mutable state share one capability.
    pub shares_model_and_state_capability: bool,
    /// Whether the request or stream material is unbounded.
    pub unbounded_material: bool,
}

/// Validated metadata-only model-call projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelCallProjection {
    /// Canonical projection identity.
    pub identity: String,
    /// Dialect.
    pub dialect: ModelDialect,
    /// Effective model identity.
    pub effective_model_identity: String,
    /// Sampling identity.
    pub sampling_identity: String,
    /// Endpoint class.
    pub endpoint: EndpointClass,
    /// Usage.
    pub usage: ModelUsage,
    /// Finish class.
    pub finish: ModelFinishClass,
    /// Error class.
    pub error: ModelErrorClass,
    /// Whether the call streamed under bounded host handling.
    pub streaming: bool,
}

/// Gateway validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayError {
    /// An exact identity is malformed.
    Identity,
    /// Endpoint does not match the dialect.
    Endpoint,
    /// Attempt or usage exceeds a bound.
    Bound,
    /// Draft requests prohibited retention, rewriting, sharing, or unbounded handling.
    Unsafe,
    /// Terminal evidence precedes or differs from commit evidence.
    CommitOrder,
    /// Retry coalescing lacks exact full-request equivalence.
    Coalescing,
}

/// Project safe model-call metadata from already-bounded host facts.
///
/// # Errors
/// Returns a typed identity, endpoint, bound, or unsafe class.
pub fn project_model_call(
    draft: &ModelCallProjectionDraft<'_>,
) -> Result<ModelCallProjection, GatewayError> {
    for identity in [
        draft.effective_model_identity,
        draft.sampling_identity,
        draft.tool_catalog_identity,
        draft.request_identity,
        draft.run_identity,
        draft.policy_identity,
    ] {
        validate_identity(identity).map_err(|_| GatewayError::Identity)?;
    }
    if draft.endpoint != draft.dialect.endpoint() || draft.endpoint == EndpointClass::Auxiliary {
        return Err(GatewayError::Endpoint);
    }
    if draft.attempt == 0
        || draft.usage.input_tokens > MAX_MODEL_CALL_TOKENS
        || draft.usage.output_tokens > MAX_MODEL_CALL_TOKENS
        || draft.usage.cached_input_tokens > MAX_MODEL_CALL_TOKENS
        || draft.usage.reasoning_tokens > MAX_MODEL_CALL_TOKENS
    {
        return Err(GatewayError::Bound);
    }
    if draft.retains_raw_provider_body
        || draft.rewrites_response
        || draft.shares_model_and_state_capability
        || draft.unbounded_material
    {
        return Err(GatewayError::Unsafe);
    }
    let attempt = format!("{}", draft.attempt);
    let usage = format!(
        "{}:{}:{}:{}",
        draft.usage.input_tokens,
        draft.usage.output_tokens,
        draft.usage.cached_input_tokens,
        draft.usage.reasoning_tokens
    );
    let identity = hash_identity(
        "animus.native-evaluation.model-call.v1",
        &[
            draft.dialect.label(),
            draft.endpoint.label(),
            draft.effective_model_identity,
            draft.sampling_identity,
            draft.tool_catalog_identity,
            draft.request_identity,
            draft.run_identity,
            draft.policy_identity,
            &attempt,
            &usage,
            draft.finish.label(),
            draft.error.label(),
            if draft.streaming { "stream" } else { "single" },
        ],
    );
    Ok(ModelCallProjection {
        identity,
        dialect: draft.dialect,
        effective_model_identity: draft.effective_model_identity.into(),
        sampling_identity: draft.sampling_identity.into(),
        endpoint: draft.endpoint,
        usage: draft.usage,
        finish: draft.finish,
        error: draft.error,
        streaming: draft.streaming,
    })
}

/// Exact active-turn commit evidence for terminal delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TurnCommitEvidence<'a> {
    /// Run identity.
    pub run_identity: &'a str,
    /// Request identity.
    pub request_identity: &'a str,
    /// Active-turn identity.
    pub turn_identity: &'a str,
    /// Policy identity.
    pub policy_identity: &'a str,
    /// Attempt number.
    pub attempt: u32,
    /// Durable or policy-admitted commit identity.
    pub commit_identity: &'a str,
    /// Exact result identity bound by the commit.
    pub result_identity: &'a str,
    /// Whether commit completed before terminal delivery.
    pub complete: bool,
}

/// Terminal stream evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TerminalStreamEvidence<'a> {
    /// Run identity.
    pub run_identity: &'a str,
    /// Request identity.
    pub request_identity: &'a str,
    /// Active-turn identity.
    pub turn_identity: &'a str,
    /// Policy identity.
    pub policy_identity: &'a str,
    /// Attempt number.
    pub attempt: u32,
    /// Terminal-result identity.
    pub result_identity: &'a str,
    /// Matching commit identity observed by the terminal gate.
    pub commit_identity: &'a str,
}

/// Admit terminal delivery only after exact matching commit evidence.
///
/// # Errors
/// Returns commit-order for early, stale, or mismatched terminal evidence.
pub fn admit_terminal_after_commit(
    terminal: &TerminalStreamEvidence<'_>,
    commit: &TurnCommitEvidence<'_>,
) -> Result<String, GatewayError> {
    for identity in [
        terminal.run_identity,
        terminal.request_identity,
        terminal.turn_identity,
        terminal.policy_identity,
        terminal.result_identity,
        terminal.commit_identity,
        commit.run_identity,
        commit.request_identity,
        commit.turn_identity,
        commit.policy_identity,
        commit.commit_identity,
        commit.result_identity,
    ] {
        validate_identity(identity).map_err(|_| GatewayError::Identity)?;
    }
    if !commit.complete
        || terminal.run_identity != commit.run_identity
        || terminal.request_identity != commit.request_identity
        || terminal.turn_identity != commit.turn_identity
        || terminal.policy_identity != commit.policy_identity
        || terminal.attempt == 0
        || terminal.attempt != commit.attempt
        || terminal.commit_identity != commit.commit_identity
        || terminal.result_identity != commit.result_identity
    {
        return Err(GatewayError::CommitOrder);
    }
    let attempt = format!("{}", terminal.attempt);
    Ok(hash_identity(
        "animus.native-evaluation.terminal-after-commit.v1",
        &[
            terminal.run_identity,
            terminal.request_identity,
            terminal.turn_identity,
            terminal.policy_identity,
            &attempt,
            terminal.result_identity,
            terminal.commit_identity,
        ],
    ))
}

/// Exact non-streaming retry-coalescing evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoalescingEvidence<'a> {
    /// First canonical request identity.
    pub first_request_identity: &'a str,
    /// Later canonical request identity.
    pub later_request_identity: &'a str,
    /// First run identity.
    pub first_run_identity: &'a str,
    /// Later run identity.
    pub later_run_identity: &'a str,
    /// First model identity.
    pub first_model_identity: &'a str,
    /// Later model identity.
    pub later_model_identity: &'a str,
    /// First sampling identity.
    pub first_sampling_identity: &'a str,
    /// Later sampling identity.
    pub later_sampling_identity: &'a str,
    /// First policy identity.
    pub first_policy_identity: &'a str,
    /// Later policy identity.
    pub later_policy_identity: &'a str,
    /// First active-turn identity.
    pub first_turn_identity: &'a str,
    /// Later active-turn identity.
    pub later_turn_identity: &'a str,
    /// Shared attempt-lineage identity.
    pub attempt_lineage_identity: &'a str,
    /// First bounded result identity.
    pub first_result_identity: &'a str,
    /// First attempt number.
    pub first_attempt: u32,
    /// Later attempt number.
    pub later_attempt: u32,
    /// Whether byte equality was observed by the host.
    pub identical_bounded_request: bool,
    /// Whether both calls are non-streaming.
    pub non_streaming: bool,
}

/// Admit exact retry coalescing without using content equality alone.
///
/// # Errors
/// Returns coalescing when any material identity or mode differs.
pub fn admit_identical_request_coalescing(
    evidence: &CoalescingEvidence<'_>,
) -> Result<String, GatewayError> {
    for identity in [
        evidence.first_request_identity,
        evidence.later_request_identity,
        evidence.first_run_identity,
        evidence.later_run_identity,
        evidence.first_model_identity,
        evidence.later_model_identity,
        evidence.first_sampling_identity,
        evidence.later_sampling_identity,
        evidence.first_policy_identity,
        evidence.later_policy_identity,
        evidence.first_turn_identity,
        evidence.later_turn_identity,
        evidence.attempt_lineage_identity,
        evidence.first_result_identity,
    ] {
        validate_identity(identity).map_err(|_| GatewayError::Identity)?;
    }
    if !evidence.identical_bounded_request
        || !evidence.non_streaming
        || evidence.first_attempt == 0
        || evidence.later_attempt == 0
        || evidence.later_attempt <= evidence.first_attempt
        || evidence.first_request_identity != evidence.later_request_identity
        || evidence.first_run_identity != evidence.later_run_identity
        || evidence.first_model_identity != evidence.later_model_identity
        || evidence.first_sampling_identity != evidence.later_sampling_identity
        || evidence.first_policy_identity != evidence.later_policy_identity
        || evidence.first_turn_identity != evidence.later_turn_identity
    {
        return Err(GatewayError::Coalescing);
    }
    let first_attempt = format!("{}", evidence.first_attempt);
    let later_attempt = format!("{}", evidence.later_attempt);
    Ok(hash_identity(
        "animus.native-evaluation.coalescing.v1",
        &[
            evidence.first_request_identity,
            evidence.first_run_identity,
            evidence.first_model_identity,
            evidence.first_sampling_identity,
            evidence.first_policy_identity,
            evidence.first_turn_identity,
            evidence.attempt_lineage_identity,
            evidence.first_result_identity,
            &first_attempt,
            &later_attempt,
        ],
    ))
}
