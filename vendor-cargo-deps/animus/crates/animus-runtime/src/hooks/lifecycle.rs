#![allow(unknown_lints)]
#![allow(
    ambiguous_params,
    assertion_density,
    borrowed_argument_types,
    no_panic,
    numeric_units,
    path_segment_repetition,
    unbounded_collection_growth,
    usize_in_public_api
)]

const HOOK_DEFINITION_SCHEMA: &str = "animus.lifecycle_hook.definition.v1";
const DEFAULT_REDACTION_MARKER: &str = "body_redacted";
const EMPTY_SENSITIVE_BODY_HASH: &str = "empty_sensitive_body";
const BLAKE3_HEX_LENGTH: usize = 64;
const MAX_SAFE_LABEL_BYTES: usize = 128;
const MAX_SAFE_SUMMARY_BYTES: usize = 512;
const MAX_DECLARED_HOOK_CAPABILITIES: usize = 8;

const LEAK_MARKERS: &[&str] = &[
    "raw_prompt:",
    "credential:",
    "provider_payload:",
    "raw_tool_body:",
    "full_path:",
    "unbounded_output:",
];

// r[impl agent_harness_runtime.openai_codex_hook_trust_lifecycle.event_registry]
// r[impl agent_harness_runtime.claude_code_hooks.scope]
// r[impl agent_harness_runtime.claude_code_hooks.points]
/// Stable lifecycle hook point adapted from `Claude Code` and `OpenAI Codex` hook surfaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookEventKind {
    /// Session-start lifecycle event.
    SessionStart,
    /// Session-end lifecycle event.
    SessionEnd,
    /// User-prompt submission lifecycle event.
    UserPromptSubmit,
    /// Prompt assembly lifecycle event.
    PromptAssembly,
    /// Boundary before a loop-fabric turn is evaluated.
    PreTurn,
    /// Boundary after a loop-fabric turn result has been recorded.
    PostTurn,
    /// Boundary before a model request crosses the adapter.
    ModelRequest,
    /// Boundary after a model response has been validated.
    ModelResponse,
    /// Boundary before a tool request crosses the adapter.
    PreToolUse,
    /// Boundary where a permission decision is requested.
    PermissionRequest,
    /// Boundary after a tool response has been validated.
    PostToolUse,
    /// Boundary before a mutation crosses the shell adapter.
    PreMutation,
    /// Boundary after a mutation result has been validated.
    PostMutation,
    /// Boundary before compaction work is requested.
    PreCompact,
    /// Boundary after compaction work is summarized.
    PostCompact,
    /// Boundary before validation work is requested.
    Validation,
    /// Boundary where a receipt is emitted.
    ReceiptEmission,
    /// Boundary before a subagent starts.
    SubagentStart,
    /// Boundary after a subagent stops.
    SubagentStop,
    /// Workflow-completion lifecycle event.
    WorkflowCompletion,
    /// Turn or run stop lifecycle event.
    Stop,
}

impl LifecycleHookEventKind {
    /// Parses a stable event-kind label.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when `value` is not in the supported
    /// lifecycle hook event vocabulary.
    pub fn parse(value: &str) -> Result<Self, LifecycleHookError> {
        match value {
            "session_start" => Ok(Self::SessionStart),
            "session_end" => Ok(Self::SessionEnd),
            "user_prompt_submit" => Ok(Self::UserPromptSubmit),
            "prompt_assembly" => Ok(Self::PromptAssembly),
            "pre_turn" => Ok(Self::PreTurn),
            "post_turn" => Ok(Self::PostTurn),
            "model_request" => Ok(Self::ModelRequest),
            "model_response" => Ok(Self::ModelResponse),
            "pre_tool_use" => Ok(Self::PreToolUse),
            "permission_request" => Ok(Self::PermissionRequest),
            "post_tool_use" => Ok(Self::PostToolUse),
            "pre_mutation" => Ok(Self::PreMutation),
            "post_mutation" => Ok(Self::PostMutation),
            "pre_compact" => Ok(Self::PreCompact),
            "post_compact" => Ok(Self::PostCompact),
            "validation" => Ok(Self::Validation),
            "receipt_emission" => Ok(Self::ReceiptEmission),
            "subagent_start" => Ok(Self::SubagentStart),
            "subagent_stop" => Ok(Self::SubagentStop),
            "workflow_completion" => Ok(Self::WorkflowCompletion),
            "stop" => Ok(Self::Stop),
            unsupported => {
                Err(LifecycleHookError::new(LifecycleHookErrorKind::UnknownEvent, "event_kind", unsupported))
            }
        }
    }

    /// Returns the stable event-kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionStart => "session_start",
            Self::SessionEnd => "session_end",
            Self::UserPromptSubmit => "user_prompt_submit",
            Self::PromptAssembly => "prompt_assembly",
            Self::PreTurn => "pre_turn",
            Self::PostTurn => "post_turn",
            Self::ModelRequest => "model_request",
            Self::ModelResponse => "model_response",
            Self::PreToolUse => "pre_tool_use",
            Self::PermissionRequest => "permission_request",
            Self::PostToolUse => "post_tool_use",
            Self::PreMutation => "pre_mutation",
            Self::PostMutation => "post_mutation",
            Self::PreCompact => "pre_compact",
            Self::PostCompact => "post_compact",
            Self::Validation => "validation",
            Self::ReceiptEmission => "receipt_emission",
            Self::SubagentStart => "subagent_start",
            Self::SubagentStop => "subagent_stop",
            Self::WorkflowCompletion => "workflow_completion",
            Self::Stop => "stop",
        }
    }

    const fn supports_capability(self, capability: LifecycleHookCapability) -> bool {
        match capability {
            LifecycleHookCapability::Observe | LifecycleHookCapability::Annotate => true,
            LifecycleHookCapability::Deny => matches!(
                self,
                Self::ModelRequest
                    | Self::PreTurn
                    | Self::PreToolUse
                    | Self::PermissionRequest
                    | Self::PreMutation
                    | Self::Validation
                    | Self::Stop
            ),
            LifecycleHookCapability::RequestContinuation => {
                matches!(self, Self::PostTurn | Self::Stop)
            }
            LifecycleHookCapability::RequestRevalidation => {
                matches!(self, Self::ModelRequest | Self::PreToolUse | Self::PreMutation | Self::Validation)
            }
        }
    }

    const fn supports_effect(self, effect: LifecycleHookEffectKind) -> bool {
        match effect {
            LifecycleHookEffectKind::AdditionalContext => matches!(
                self,
                Self::SessionStart
                    | Self::UserPromptSubmit
                    | Self::PromptAssembly
                    | Self::PreTurn
                    | Self::PreCompact
                    | Self::SubagentStart
            ),
            LifecycleHookEffectKind::Warning => true,
            LifecycleHookEffectKind::Deny => self.supports_capability(LifecycleHookCapability::Deny),
            LifecycleHookEffectKind::RequestContinuation => {
                self.supports_capability(LifecycleHookCapability::RequestContinuation)
            }
            LifecycleHookEffectKind::RequestRevalidation => {
                self.supports_capability(LifecycleHookCapability::RequestRevalidation)
            }
            LifecycleHookEffectKind::RewriteToolInput
            | LifecycleHookEffectKind::ExecuteTool
            | LifecycleHookEffectKind::RewriteDeniedAction
            | LifecycleHookEffectKind::WeakenSandbox
            | LifecycleHookEffectKind::SuppressApproval => false,
        }
    }
}

// r[impl agent_harness_runtime.claude_code_hooks.execution]
/// Declared capability a host-owned lifecycle hook adapter is allowed to use.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum LifecycleHookCapability {
    /// Observe a redacted lifecycle event without effects.
    Observe,
    /// Attach bounded annotations such as context summaries or warnings.
    Annotate,
    /// Deny a guarded lifecycle action.
    Deny,
    /// Request continuation after a post-turn or stop boundary.
    RequestContinuation,
    /// Request normal runtime DTO, permission, and capability revalidation.
    RequestRevalidation,
}

impl LifecycleHookCapability {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observe => "observe",
            Self::Annotate => "annotate",
            Self::Deny => "deny",
            Self::RequestContinuation => "request_continuation",
            Self::RequestRevalidation => "request_revalidation",
        }
    }
}

/// Source or management scope for a lifecycle hook definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookDefinitionSource {
    /// Definition is supplied by managed host policy.
    Managed,
    /// Definition is supplied by a user-local source.
    User,
    /// Definition is supplied by project-local configuration.
    Project,
}

impl LifecycleHookDefinitionSource {
    /// Returns the stable source label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Managed => "managed",
            Self::User => "user",
            Self::Project => "project",
        }
    }
}

/// Trust scope attached to a reviewed lifecycle hook definition hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookTrustScope {
    /// Managed host policy trust scope.
    Managed,
    /// User-local trust scope.
    User,
    /// Project-local trust scope.
    Project,
}

impl LifecycleHookTrustScope {
    /// Returns the stable trust-scope label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Managed => "managed",
            Self::User => "user",
            Self::Project => "project",
        }
    }
}

/// Trust decision attached to a lifecycle hook trust record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookTrustDecision {
    /// The reviewed definition hash is allowed in its scope.
    Trusted,
    /// The reviewed definition hash is explicitly denied in its scope.
    Denied,
}

impl LifecycleHookTrustDecision {
    /// Returns the stable trust-decision label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Denied => "denied",
        }
    }
}

/// Draft lifecycle hook definition supplied by a host shell or policy loader.
pub struct LifecycleHookDefinitionDraft<'a> {
    /// Stable hook identifier used in reports and trust records.
    pub hook_id: &'a str,
    /// Stable event-kind label.
    pub event_kind: &'a str,
    /// Source or management scope for the definition.
    pub source: LifecycleHookDefinitionSource,
    /// Capabilities declared by the host-owned hook adapter.
    pub capabilities: &'a [LifecycleHookCapability],
    /// Raw definition body that is hashed but not stored by the runtime core.
    pub definition_body: &'a str,
}

// r[impl agent_harness_runtime.openai_codex_hook_trust_lifecycle.trust_identity]
/// Content-addressed lifecycle hook definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookDefinition {
    hook_id: String,
    event_kind: LifecycleHookEventKind,
    source: LifecycleHookDefinitionSource,
    capabilities: Vec<LifecycleHookCapability>,
    definition_hash: String,
}

impl LifecycleHookDefinition {
    /// Validates and content-addresses a lifecycle hook definition draft.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when identifiers are unsafe, the event kind
    /// is unsupported, the capability declaration is invalid, or the definition
    /// body is empty.
    pub fn validate(draft: &LifecycleHookDefinitionDraft<'_>) -> Result<Self, LifecycleHookError> {
        let hook_id = validate_safe_label("hook_id", draft.hook_id)?;
        let event_kind = LifecycleHookEventKind::parse(draft.event_kind)?;
        let capabilities = validate_capabilities(event_kind, draft.capabilities)?;
        if draft.definition_body.is_empty() {
            return Err(LifecycleHookError::new(LifecycleHookErrorKind::EmptyDefinition, "definition_body", ""));
        }
        let definition_hash = hash_definition(DefinitionHashInput {
            hook_id,
            event_kind,
            source: draft.source,
            capabilities: &capabilities,
            definition_body: draft.definition_body,
        });
        Ok(Self {
            hook_id: String::from(hook_id),
            event_kind,
            source: draft.source,
            capabilities,
            definition_hash,
        })
    }

    /// Returns the stable hook identifier.
    #[must_use]
    pub fn hook_id(&self) -> &str {
        self.hook_id.as_str()
    }

    /// Returns the lifecycle event kind served by this hook.
    #[must_use]
    pub const fn event_kind(&self) -> LifecycleHookEventKind {
        self.event_kind
    }

    /// Returns the hook definition source.
    #[must_use]
    pub const fn source(&self) -> LifecycleHookDefinitionSource {
        self.source
    }

    /// Returns the declared host adapter capabilities.
    #[must_use]
    pub fn capabilities(&self) -> &[LifecycleHookCapability] {
        self.capabilities.as_slice()
    }

    /// Returns true when the hook declares `capability`.
    #[must_use]
    pub fn declares_capability(&self, capability: LifecycleHookCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Returns the BLAKE3 hash of the reviewed hook definition content.
    #[must_use]
    pub fn definition_hash(&self) -> &str {
        self.definition_hash.as_str()
    }
}

/// Draft trust record for a reviewed lifecycle hook definition hash.
pub struct LifecycleHookTrustRecordDraft<'a> {
    /// Stable hook identifier being trusted or denied.
    pub hook_id: &'a str,
    /// BLAKE3 definition hash this decision applies to.
    pub definition_hash: &'a str,
    /// Trust scope that issued the decision.
    pub scope: LifecycleHookTrustScope,
    /// Trust decision for the reviewed definition hash.
    pub decision: LifecycleHookTrustDecision,
}

/// Reviewed trust record bound to one definition hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookTrustRecord {
    hook_id: String,
    definition_hash: String,
    scope: LifecycleHookTrustScope,
    decision: LifecycleHookTrustDecision,
}

impl LifecycleHookTrustRecord {
    /// Validates a lifecycle hook trust record.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when the hook id or hash is malformed.
    pub fn validate(draft: &LifecycleHookTrustRecordDraft<'_>) -> Result<Self, LifecycleHookError> {
        let hook_id = validate_safe_label("hook_id", draft.hook_id)?;
        let definition_hash = validate_blake3_hex("definition_hash", draft.definition_hash)?;
        Ok(Self {
            hook_id: String::from(hook_id),
            definition_hash: String::from(definition_hash),
            scope: draft.scope,
            decision: draft.decision,
        })
    }

    /// Builds a trusted record for an already validated definition.
    #[must_use]
    pub fn trusted(definition: &LifecycleHookDefinition, scope: LifecycleHookTrustScope) -> Self {
        Self {
            hook_id: String::from(definition.hook_id()),
            definition_hash: String::from(definition.definition_hash()),
            scope,
            decision: LifecycleHookTrustDecision::Trusted,
        }
    }

    /// Builds a denied record for an already validated definition.
    #[must_use]
    pub fn denied(definition: &LifecycleHookDefinition, scope: LifecycleHookTrustScope) -> Self {
        Self {
            hook_id: String::from(definition.hook_id()),
            definition_hash: String::from(definition.definition_hash()),
            scope,
            decision: LifecycleHookTrustDecision::Denied,
        }
    }

    /// Returns the hook identifier covered by this record.
    #[must_use]
    pub fn hook_id(&self) -> &str {
        self.hook_id.as_str()
    }

    /// Returns the definition hash covered by this record.
    #[must_use]
    pub fn definition_hash(&self) -> &str {
        self.definition_hash.as_str()
    }

    /// Returns the trust scope for this record.
    #[must_use]
    pub const fn scope(&self) -> LifecycleHookTrustScope {
        self.scope
    }

    /// Returns the trust decision for this record.
    #[must_use]
    pub const fn decision(&self) -> LifecycleHookTrustDecision {
        self.decision
    }
}

/// Evaluated trust status for one hook definition hash.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookTrustStatus {
    /// Matching trusted record was found.
    Trusted {
        /// Scope that issued the matching trusted record.
        scope: LifecycleHookTrustScope,
    },
    /// Matching denied record was found.
    Denied {
        /// Scope that issued the matching denied record.
        scope: LifecycleHookTrustScope,
    },
    /// No matching trust record covered the definition hash.
    Untrusted,
}

impl LifecycleHookTrustStatus {
    /// Returns the stable trust-status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted { .. } => "trusted",
            Self::Denied { .. } => "denied",
            Self::Untrusted => "untrusted",
        }
    }

    /// Returns true when the status authorizes hook dispatch.
    #[must_use]
    pub const fn is_trusted(self) -> bool {
        matches!(self, Self::Trusted { .. })
    }
}

/// Redaction state for a lifecycle hook event input summary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookRedaction {
    /// No sensitive body was supplied to summarize.
    NoSensitiveBody,
    /// A sensitive body was summarized by size and BLAKE3 hash only.
    BodyRedacted,
}

impl LifecycleHookRedaction {
    /// Returns the stable redaction label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NoSensitiveBody => "no_sensitive_body",
            Self::BodyRedacted => DEFAULT_REDACTION_MARKER,
        }
    }
}

// r[impl agent_harness_runtime.claude_code_hooks.payloads]
/// Draft event input that may include a sensitive body to hash and redact.
pub struct LifecycleHookInputDraft<'a> {
    /// Stable lifecycle event identifier.
    pub event_id: &'a str,
    /// Stable event-kind label.
    pub event_kind: &'a str,
    /// Safe resource class, such as `tool`, `prompt`, or `session`.
    pub resource_class: &'a str,
    /// Safe redacted label for the affected resource.
    pub safe_label: &'a str,
    /// Safe bounded event summary.
    pub summary: &'a str,
    /// Optional sensitive event body; it is hashed but not stored.
    pub sensitive_body: Option<&'a str>,
    /// Optional safe status class for the event.
    pub status: Option<&'a str>,
}

/// Already-redacted lifecycle input fields decoded from a safe report.
pub struct LifecycleHookInputReportDraft<'a> {
    /// Stable lifecycle event identifier.
    pub event_id: &'a str,
    /// Stable event-kind label.
    pub event_kind: &'a str,
    /// Safe resource class, such as `tool`, `prompt`, or `session`.
    pub resource_class: &'a str,
    /// Safe redacted label for the affected resource.
    pub safe_label: &'a str,
    /// Safe bounded event summary.
    pub summary: &'a str,
    /// Previously recorded sensitive body byte count.
    pub body_size_bytes: usize,
    /// BLAKE3 sensitive body hash or the no-body marker hash.
    pub body_hash: &'a str,
    /// Redaction class for the decoded input report.
    pub redaction: LifecycleHookRedaction,
    /// Optional safe status class for the event.
    pub status: Option<&'a str>,
}

/// Redacted lifecycle hook event input sent to shell-owned hook executors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookInput {
    event_id: String,
    event_kind: LifecycleHookEventKind,
    resource_class: String,
    safe_label: String,
    summary: String,
    body_size_bytes: usize,
    body_hash: String,
    redaction: LifecycleHookRedaction,
    status: Option<String>,
}

impl LifecycleHookInput {
    /// Validates and redacts lifecycle hook event input.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when safe metadata is invalid or the event
    /// kind is unknown.
    pub fn validate(draft: &LifecycleHookInputDraft<'_>) -> Result<Self, LifecycleHookError> {
        let event_id = validate_safe_label("event_id", draft.event_id)?;
        let event_kind = LifecycleHookEventKind::parse(draft.event_kind)?;
        let resource_class = validate_safe_label("resource_class", draft.resource_class)?;
        let safe_label = validate_safe_label("safe_label", draft.safe_label)?;
        let summary = validate_safe_summary("summary", draft.summary)?;
        let status = draft.status.map(|value| validate_safe_label("status", value).map(String::from)).transpose()?;
        let (body_size_bytes, body_hash, redaction) = summarize_sensitive_body(draft.sensitive_body);
        Ok(Self {
            event_id: String::from(event_id),
            event_kind,
            resource_class: String::from(resource_class),
            safe_label: String::from(safe_label),
            summary: String::from(summary),
            body_size_bytes,
            body_hash,
            redaction,
            status,
        })
    }

    /// Rebuilds a lifecycle hook input from already-redacted report fields.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when any safe metadata is malformed or the
    /// redaction/hash pair is inconsistent.
    pub fn from_redacted_report(draft: &LifecycleHookInputReportDraft<'_>) -> Result<Self, LifecycleHookError> {
        let event_id = validate_safe_label("event_id", draft.event_id)?;
        let event_kind = LifecycleHookEventKind::parse(draft.event_kind)?;
        let resource_class = validate_safe_label("resource_class", draft.resource_class)?;
        let safe_label = validate_safe_label("safe_label", draft.safe_label)?;
        let summary = validate_safe_summary("summary", draft.summary)?;
        let body_hash = validate_redacted_body_summary(draft.body_size_bytes, draft.body_hash, draft.redaction)?;
        let status = draft.status.map(|value| validate_safe_label("status", value).map(String::from)).transpose()?;
        Ok(Self {
            event_id: String::from(event_id),
            event_kind,
            resource_class: String::from(resource_class),
            safe_label: String::from(safe_label),
            summary: String::from(summary),
            body_size_bytes: draft.body_size_bytes,
            body_hash: String::from(body_hash),
            redaction: draft.redaction,
            status,
        })
    }

    /// Returns the stable event identifier.
    #[must_use]
    pub fn event_id(&self) -> &str {
        self.event_id.as_str()
    }

    /// Returns the lifecycle hook event kind.
    #[must_use]
    pub const fn event_kind(&self) -> LifecycleHookEventKind {
        self.event_kind
    }

    /// Returns the safe resource class.
    #[must_use]
    pub fn resource_class(&self) -> &str {
        self.resource_class.as_str()
    }

    /// Returns the safe redacted resource label.
    #[must_use]
    pub fn safe_label(&self) -> &str {
        self.safe_label.as_str()
    }

    /// Returns the safe bounded event summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        self.summary.as_str()
    }

    /// Returns the sensitive body byte length.
    #[must_use]
    pub const fn body_size_bytes(&self) -> usize {
        self.body_size_bytes
    }

    /// Returns the BLAKE3 sensitive body hash or a stable no-body marker.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }

    /// Returns the input redaction class.
    #[must_use]
    pub const fn redaction(&self) -> LifecycleHookRedaction {
        self.redaction
    }

    /// Returns the optional safe status class.
    #[must_use]
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

/// Typed hook request selected for shell-owned execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookRequest {
    hook_id: String,
    definition_hash: String,
    event_kind: LifecycleHookEventKind,
    input: LifecycleHookInput,
}

impl LifecycleHookRequest {
    /// Returns the hook identifier selected for execution.
    #[must_use]
    pub fn hook_id(&self) -> &str {
        self.hook_id.as_str()
    }

    /// Returns the trusted hook definition hash.
    #[must_use]
    pub fn definition_hash(&self) -> &str {
        self.definition_hash.as_str()
    }

    /// Returns the routed lifecycle event kind.
    #[must_use]
    pub const fn event_kind(&self) -> LifecycleHookEventKind {
        self.event_kind
    }

    /// Returns the redacted event input for hook execution.
    #[must_use]
    pub const fn input(&self) -> &LifecycleHookInput {
        &self.input
    }
}

/// Route plan for one lifecycle hook event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookRoutePlan {
    requests: Vec<LifecycleHookRequest>,
    outcomes: Vec<LifecycleHookOutcome>,
}

impl LifecycleHookRoutePlan {
    /// Returns trusted hook requests selected for execution.
    #[must_use]
    pub fn requests(&self) -> &[LifecycleHookRequest] {
        self.requests.as_slice()
    }

    /// Returns route-time outcomes for matching hooks that were not executable.
    #[must_use]
    pub fn outcomes(&self) -> &[LifecycleHookOutcome] {
        self.outcomes.as_slice()
    }
}

/// Draft effect returned by a shell-owned hook executor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifecycleHookEffectDraft<'a> {
    /// Stable effect kind label.
    pub kind: &'a str,
    /// Safe effect value, such as a warning or denial reason.
    pub value: &'a str,
}

/// Draft hook output returned by a shell-owned hook executor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifecycleHookOutputDraft<'a> {
    /// Ordered event-scoped effects returned by the hook.
    pub effects: &'a [LifecycleHookEffectDraft<'a>],
}

// r[impl agent_harness_runtime.claude_code_hooks.outcomes]
/// Event-scoped hook effect accepted by the runtime core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleHookEffect {
    /// Add bounded context metadata for events that accept context augmentation.
    AdditionalContext {
        /// Safe context summary.
        summary: String,
        /// BLAKE3 hash of the safe context summary.
        context_hash: String,
    },
    /// Emit a system-visible warning.
    Warning {
        /// Safe warning message.
        message: String,
    },
    /// Deny the guarded action.
    Deny {
        /// Safe denial reason.
        reason: String,
    },
    /// Request continuation after a stop boundary.
    RequestContinuation {
        /// Safe continuation reason.
        reason: String,
    },
    /// Request normal runtime revalidation before using the guarded action.
    RequestRevalidation {
        /// Safe revalidation reason.
        reason: String,
    },
}

impl LifecycleHookEffect {
    /// Returns the stable effect-kind label.
    #[must_use]
    pub const fn kind_label(&self) -> &'static str {
        match self {
            Self::AdditionalContext { .. } => "additional_context",
            Self::Warning { .. } => "warning",
            Self::Deny { .. } => "deny",
            Self::RequestContinuation { .. } => "request_continuation",
            Self::RequestRevalidation { .. } => "request_revalidation",
        }
    }
}

// r[impl agent_harness_runtime.openai_codex_hook_trust_lifecycle.io_contract]
/// Validated hook output whose effects are legal for the event kind.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookOutput {
    effects: Vec<LifecycleHookEffect>,
}

impl LifecycleHookOutput {
    /// Validates hook output effects against an event-specific contract.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when an effect is malformed, unsupported by
    /// the event kind, or attempts to bypass policy boundaries.
    pub fn validate(
        event_kind: LifecycleHookEventKind,
        draft: &LifecycleHookOutputDraft<'_>,
    ) -> Result<Self, LifecycleHookError> {
        let mut effects = Vec::with_capacity(draft.effects.len());
        for effect in draft.effects {
            effects.push(validate_effect(event_kind, effect)?);
        }
        Ok(Self { effects })
    }

    /// Validates hook output effects against a definition's event and declared capabilities.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when an effect is malformed, unsupported by
    /// the event kind, attempts to bypass policy boundaries, or uses an undeclared
    /// host hook capability.
    pub fn validate_for_definition(
        definition: &LifecycleHookDefinition,
        draft: &LifecycleHookOutputDraft<'_>,
    ) -> Result<Self, LifecycleHookError> {
        let output = Self::validate(definition.event_kind(), draft)?;
        for effect in output.effects() {
            let capability = required_capability_for_effect(effect);
            if !definition.declares_capability(capability) {
                return Err(LifecycleHookError::new(
                    LifecycleHookErrorKind::CapabilityDenied,
                    "effect_kind",
                    effect.kind_label(),
                ));
            }
        }
        Ok(output)
    }

    /// Returns the validated effects.
    #[must_use]
    pub fn effects(&self) -> &[LifecycleHookEffect] {
        self.effects.as_slice()
    }
}

/// Current policy state at a lifecycle boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LifecycleHookPolicyStatus {
    /// Policy currently allows the boundary action.
    Allowed,
    /// Policy currently denies the boundary action.
    Denied {
        /// Safe denial reason.
        reason: String,
    },
    /// Policy currently requires approval before dispatch.
    ApprovalRequired {
        /// Safe approval reason.
        reason: String,
    },
}

/// Class of a recorded lifecycle hook outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookOutcomeClass {
    /// Hook output was applied to the runtime boundary.
    Applied,
    /// Hook was skipped by policy.
    Skipped,
    /// Hook definition lacked a matching trust record.
    Untrusted,
    /// Hook executor or required service failed.
    Failed,
    /// Hook denied a boundary or was denied by trust policy.
    Denied,
    /// Hook output was malformed.
    Malformed,
    /// Hook output used an unsupported event-scoped effect.
    Unsupported,
}

impl LifecycleHookOutcomeClass {
    /// Returns the stable outcome-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Applied => "applied",
            Self::Skipped => "skipped",
            Self::Untrusted => "untrusted",
            Self::Failed => "failed",
            Self::Denied => "denied",
            Self::Malformed => "malformed",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Optional failure class attached to a lifecycle hook outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookFailureClass {
    /// Required hook service was unavailable in the host shell.
    RequiredHookUnavailable,
    /// Hook executor returned a failure.
    HookExecutionFailed,
    /// Trust policy denied the hook definition hash.
    HookDeniedByTrust,
    /// Hook output was malformed.
    MalformedOutput,
    /// Hook output requested an unsupported effect for the event.
    UnsupportedEffect,
    /// Hook attempted to bypass policy or adapter boundaries.
    PolicyBypass,
}

impl LifecycleHookFailureClass {
    /// Returns the stable failure-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequiredHookUnavailable => "required_hook_unavailable",
            Self::HookExecutionFailed => "hook_execution_failed",
            Self::HookDeniedByTrust => "hook_denied_by_trust",
            Self::MalformedOutput => "malformed_output",
            Self::UnsupportedEffect => "unsupported_effect",
            Self::PolicyBypass => "policy_bypass",
        }
    }
}

// r[impl agent_harness_runtime.openai_codex_hook_trust_lifecycle.outcome_reporting]
/// Safe lifecycle hook outcome metadata for runtime reports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookOutcome {
    hook_id: String,
    definition_hash: String,
    event_kind: LifecycleHookEventKind,
    trust_status: LifecycleHookTrustStatus,
    outcome_class: LifecycleHookOutcomeClass,
    failure_class: Option<LifecycleHookFailureClass>,
    input: LifecycleHookInput,
}

/// Borrowed lifecycle hook outcome fields for safe report construction.
#[derive(Clone, Copy)]
pub struct LifecycleHookOutcomeDraft<'a> {
    /// Reviewed lifecycle hook definition.
    pub definition: &'a LifecycleHookDefinition,
    /// Redacted lifecycle hook input.
    pub input: &'a LifecycleHookInput,
    /// Trust status evaluated for the hook definition hash.
    pub trust_status: LifecycleHookTrustStatus,
    /// Recorded outcome class for this hook and event.
    pub outcome_class: LifecycleHookOutcomeClass,
    /// Optional failure class attached to the outcome.
    pub failure_class: Option<LifecycleHookFailureClass>,
}

/// Already-redacted lifecycle hook outcome fields decoded from a safe report.
pub struct LifecycleHookOutcomeReportDraft<'a> {
    /// Stable hook identifier used in reports and trust records.
    pub hook_id: &'a str,
    /// BLAKE3 hash of the reviewed hook definition.
    pub definition_hash: &'a str,
    /// Trust status evaluated for the hook definition hash.
    pub trust_status: LifecycleHookTrustStatus,
    /// Recorded outcome class for this hook and event.
    pub outcome_class: LifecycleHookOutcomeClass,
    /// Optional failure class attached to the outcome.
    pub failure_class: Option<LifecycleHookFailureClass>,
    /// Redacted input summary associated with this outcome.
    pub input: &'a LifecycleHookInput,
}

impl LifecycleHookOutcome {
    /// Builds a safe outcome report for one lifecycle hook and event.
    #[must_use]
    pub fn new(draft: LifecycleHookOutcomeDraft<'_>) -> Self {
        Self {
            hook_id: String::from(draft.definition.hook_id()),
            definition_hash: String::from(draft.definition.definition_hash()),
            event_kind: draft.input.event_kind(),
            trust_status: draft.trust_status,
            outcome_class: draft.outcome_class,
            failure_class: draft.failure_class,
            input: draft.input.clone(),
        }
    }

    /// Returns a required-service-unavailable outcome for a trusted hook.
    #[must_use]
    pub fn required_service_unavailable(
        definition: &LifecycleHookDefinition,
        input: &LifecycleHookInput,
        trust_status: LifecycleHookTrustStatus,
    ) -> Self {
        Self::new(LifecycleHookOutcomeDraft {
            definition,
            input,
            trust_status,
            outcome_class: LifecycleHookOutcomeClass::Failed,
            failure_class: Some(LifecycleHookFailureClass::RequiredHookUnavailable),
        })
    }

    /// Rebuilds a lifecycle hook outcome from already-redacted report fields.
    ///
    /// # Errors
    ///
    /// Returns [`LifecycleHookError`] when the hook identifier or definition hash
    /// is malformed.
    pub fn from_redacted_report(draft: &LifecycleHookOutcomeReportDraft<'_>) -> Result<Self, LifecycleHookError> {
        let hook_id = validate_safe_label("hook_id", draft.hook_id)?;
        let definition_hash = validate_blake3_hex("definition_hash", draft.definition_hash)?;
        Ok(Self {
            hook_id: String::from(hook_id),
            definition_hash: String::from(definition_hash),
            event_kind: draft.input.event_kind(),
            trust_status: draft.trust_status,
            outcome_class: draft.outcome_class,
            failure_class: draft.failure_class,
            input: draft.input.clone(),
        })
    }

    /// Returns the hook identifier.
    #[must_use]
    pub fn hook_id(&self) -> &str {
        self.hook_id.as_str()
    }

    /// Returns the audited definition hash.
    #[must_use]
    pub fn definition_hash(&self) -> &str {
        self.definition_hash.as_str()
    }

    /// Returns the lifecycle event kind.
    #[must_use]
    pub const fn event_kind(&self) -> LifecycleHookEventKind {
        self.event_kind
    }

    /// Returns the trust status used for this outcome.
    #[must_use]
    pub const fn trust_status(&self) -> LifecycleHookTrustStatus {
        self.trust_status
    }

    /// Returns the outcome class.
    #[must_use]
    pub const fn outcome_class(&self) -> LifecycleHookOutcomeClass {
        self.outcome_class
    }

    /// Returns the optional failure class.
    #[must_use]
    pub const fn failure_class(&self) -> Option<LifecycleHookFailureClass> {
        self.failure_class
    }

    /// Returns the redacted input summary.
    #[must_use]
    pub const fn input(&self) -> &LifecycleHookInput {
        &self.input
    }
}

/// Lifecycle hook validation and policy-binding error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleHookError {
    kind: LifecycleHookErrorKind,
    field: &'static str,
    value: String,
}

impl LifecycleHookError {
    fn new(kind: LifecycleHookErrorKind, field: &'static str, value: &str) -> Self {
        Self {
            kind,
            field,
            value: String::from(value),
        }
    }

    /// Returns the stable error kind.
    #[must_use]
    pub const fn kind(&self) -> LifecycleHookErrorKind {
        self.kind
    }

    /// Returns the field that failed validation.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        self.field
    }

    /// Returns the rejected value or diagnostic label.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

/// Stable lifecycle hook error kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleHookErrorKind {
    /// Event kind is not in the supported hook vocabulary.
    UnknownEvent,
    /// Safe metadata label was empty, unsafe, or too long.
    InvalidLabel,
    /// BLAKE3 hash metadata was malformed.
    InvalidHash,
    /// Definition body was empty.
    EmptyDefinition,
    /// Hook output was malformed.
    MalformedOutput,
    /// Hook output requested an effect outside the event contract.
    UnsupportedEffect,
    /// Hook output attempted to bypass policy or adapter boundaries.
    PolicyBypass,
    /// Hook definition omitted required capability declarations.
    MissingCapability,
    /// Hook definition declared a duplicate capability.
    DuplicateCapability,
    /// Hook definition declared a capability unsupported by the hook point.
    UnsupportedCapability,
    /// Hook output used an undeclared capability.
    CapabilityDenied,
}

/// Evaluates trust for one lifecycle hook definition.
#[must_use]
pub fn evaluate_lifecycle_hook_trust(
    definition: &LifecycleHookDefinition,
    trust_records: &[LifecycleHookTrustRecord],
) -> LifecycleHookTrustStatus {
    if let Some(record) = matching_trust_record(definition, trust_records, LifecycleHookTrustDecision::Denied) {
        return LifecycleHookTrustStatus::Denied { scope: record.scope() };
    }
    if let Some(record) = matching_trust_record(definition, trust_records, LifecycleHookTrustDecision::Trusted) {
        return LifecycleHookTrustStatus::Trusted { scope: record.scope() };
    }
    LifecycleHookTrustStatus::Untrusted
}

/// Routes one lifecycle event to matching trusted hook definitions.
#[must_use]
pub fn route_lifecycle_hook_requests(
    input: &LifecycleHookInput,
    definitions: &[LifecycleHookDefinition],
    trust_records: &[LifecycleHookTrustRecord],
) -> LifecycleHookRoutePlan {
    let mut requests = Vec::new();
    let mut outcomes = Vec::new();
    for definition in definitions.iter().filter(|definition| definition.event_kind() == input.event_kind()) {
        let trust_status = evaluate_lifecycle_hook_trust(definition, trust_records);
        match trust_status {
            LifecycleHookTrustStatus::Trusted { .. } => requests.push(LifecycleHookRequest {
                hook_id: String::from(definition.hook_id()),
                definition_hash: String::from(definition.definition_hash()),
                event_kind: input.event_kind(),
                input: input.clone(),
            }),
            LifecycleHookTrustStatus::Denied { .. } => {
                outcomes.push(LifecycleHookOutcome::new(LifecycleHookOutcomeDraft {
                    definition,
                    input,
                    trust_status,
                    outcome_class: LifecycleHookOutcomeClass::Denied,
                    failure_class: Some(LifecycleHookFailureClass::HookDeniedByTrust),
                }));
            }
            LifecycleHookTrustStatus::Untrusted => {
                outcomes.push(LifecycleHookOutcome::new(LifecycleHookOutcomeDraft {
                    definition,
                    input,
                    trust_status,
                    outcome_class: LifecycleHookOutcomeClass::Untrusted,
                    failure_class: None,
                }));
            }
        }
    }
    LifecycleHookRoutePlan { requests, outcomes }
}

// r[impl agent_harness_runtime.openai_codex_hook_trust_lifecycle.policy_binding]
// r[impl agent_harness_runtime.claude_code_hooks.scope.no_bypass]
// r[impl agent_harness_runtime.claude_code_hooks.outcomes.invalid_modification]
/// Binds accepted hook output to the current runtime policy status.
///
/// Hook output can only keep the current status or make it stricter; it cannot
/// turn a denial into an allow decision or suppress required approval.
#[must_use]
pub fn bind_lifecycle_hook_output_to_policy(
    current: LifecycleHookPolicyStatus,
    output: &LifecycleHookOutput,
) -> LifecycleHookPolicyStatus {
    match current {
        LifecycleHookPolicyStatus::Denied { .. } => current,
        LifecycleHookPolicyStatus::ApprovalRequired { .. } => {
            first_hook_denial(output).map_or(current, |reason| LifecycleHookPolicyStatus::Denied { reason })
        }
        LifecycleHookPolicyStatus::Allowed => first_hook_denial(output)
            .map_or(LifecycleHookPolicyStatus::Allowed, |reason| LifecycleHookPolicyStatus::Denied { reason }),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LifecycleHookEffectKind {
    AdditionalContext,
    Warning,
    Deny,
    RequestContinuation,
    RequestRevalidation,
    RewriteToolInput,
    ExecuteTool,
    RewriteDeniedAction,
    WeakenSandbox,
    SuppressApproval,
}

impl LifecycleHookEffectKind {
    fn parse(value: &str) -> Result<Self, LifecycleHookError> {
        match value {
            "additional_context" => Ok(Self::AdditionalContext),
            "warning" => Ok(Self::Warning),
            "deny" => Ok(Self::Deny),
            "request_continuation" => Ok(Self::RequestContinuation),
            "request_revalidation" => Ok(Self::RequestRevalidation),
            "rewrite_tool_input" => Ok(Self::RewriteToolInput),
            "execute_tool" => Ok(Self::ExecuteTool),
            "rewrite_denied_action" => Ok(Self::RewriteDeniedAction),
            "weaken_sandbox" => Ok(Self::WeakenSandbox),
            "suppress_approval" => Ok(Self::SuppressApproval),
            unsupported => {
                Err(LifecycleHookError::new(LifecycleHookErrorKind::MalformedOutput, "effect_kind", unsupported))
            }
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::AdditionalContext => "additional_context",
            Self::Warning => "warning",
            Self::Deny => "deny",
            Self::RequestContinuation => "request_continuation",
            Self::RequestRevalidation => "request_revalidation",
            Self::RewriteToolInput => "rewrite_tool_input",
            Self::ExecuteTool => "execute_tool",
            Self::RewriteDeniedAction => "rewrite_denied_action",
            Self::WeakenSandbox => "weaken_sandbox",
            Self::SuppressApproval => "suppress_approval",
        }
    }

    const fn attempts_policy_bypass(self) -> bool {
        matches!(self, Self::ExecuteTool | Self::RewriteDeniedAction | Self::WeakenSandbox | Self::SuppressApproval)
    }
}

fn validate_effect(
    event_kind: LifecycleHookEventKind,
    draft: &LifecycleHookEffectDraft<'_>,
) -> Result<LifecycleHookEffect, LifecycleHookError> {
    let effect_kind = LifecycleHookEffectKind::parse(draft.kind)?;
    if effect_kind.attempts_policy_bypass() {
        return Err(LifecycleHookError::new(LifecycleHookErrorKind::PolicyBypass, "effect_kind", effect_kind.as_str()));
    }
    if !event_kind.supports_effect(effect_kind) {
        return Err(LifecycleHookError::new(
            LifecycleHookErrorKind::UnsupportedEffect,
            "effect_kind",
            effect_kind.as_str(),
        ));
    }
    let value = validate_safe_summary("effect_value", draft.value)?;
    Ok(match effect_kind {
        LifecycleHookEffectKind::AdditionalContext => LifecycleHookEffect::AdditionalContext {
            summary: String::from(value),
            context_hash: blake3_hex(value.as_bytes()),
        },
        LifecycleHookEffectKind::Warning => LifecycleHookEffect::Warning {
            message: String::from(value),
        },
        LifecycleHookEffectKind::Deny => LifecycleHookEffect::Deny {
            reason: String::from(value),
        },
        LifecycleHookEffectKind::RequestContinuation => LifecycleHookEffect::RequestContinuation {
            reason: String::from(value),
        },
        LifecycleHookEffectKind::RequestRevalidation => LifecycleHookEffect::RequestRevalidation {
            reason: String::from(value),
        },
        LifecycleHookEffectKind::RewriteToolInput
        | LifecycleHookEffectKind::ExecuteTool
        | LifecycleHookEffectKind::RewriteDeniedAction
        | LifecycleHookEffectKind::WeakenSandbox
        | LifecycleHookEffectKind::SuppressApproval => {
            unreachable!("bypass and unsupported effects return before construction")
        }
    })
}

fn first_hook_denial(output: &LifecycleHookOutput) -> Option<String> {
    output.effects().iter().find_map(|effect| match effect {
        LifecycleHookEffect::Deny { reason } => Some(reason.clone()),
        LifecycleHookEffect::AdditionalContext { .. }
        | LifecycleHookEffect::Warning { .. }
        | LifecycleHookEffect::RequestContinuation { .. }
        | LifecycleHookEffect::RequestRevalidation { .. } => None,
    })
}

fn required_capability_for_effect(effect: &LifecycleHookEffect) -> LifecycleHookCapability {
    match effect {
        LifecycleHookEffect::AdditionalContext { .. } | LifecycleHookEffect::Warning { .. } => {
            LifecycleHookCapability::Annotate
        }
        LifecycleHookEffect::Deny { .. } => LifecycleHookCapability::Deny,
        LifecycleHookEffect::RequestContinuation { .. } => LifecycleHookCapability::RequestContinuation,
        LifecycleHookEffect::RequestRevalidation { .. } => LifecycleHookCapability::RequestRevalidation,
    }
}

fn matching_trust_record<'a>(
    definition: &LifecycleHookDefinition,
    trust_records: &'a [LifecycleHookTrustRecord],
    decision: LifecycleHookTrustDecision,
) -> Option<&'a LifecycleHookTrustRecord> {
    trust_records.iter().find(|record| {
        record.hook_id() == definition.hook_id()
            && record.definition_hash() == definition.definition_hash()
            && record.decision() == decision
    })
}

fn validate_safe_label<'a>(field: &'static str, value: &'a str) -> Result<&'a str, LifecycleHookError> {
    validate_safe_text(field, value, MAX_SAFE_LABEL_BYTES)
}

fn validate_capabilities(
    event_kind: LifecycleHookEventKind,
    capabilities: &[LifecycleHookCapability],
) -> Result<Vec<LifecycleHookCapability>, LifecycleHookError> {
    if capabilities.is_empty() || capabilities.len() > MAX_DECLARED_HOOK_CAPABILITIES {
        return Err(LifecycleHookError::new(
            LifecycleHookErrorKind::MissingCapability,
            "capabilities",
            "missing_or_too_many",
        ));
    }
    let mut normalized = Vec::with_capacity(capabilities.len());
    for capability in capabilities {
        if normalized.contains(capability) {
            return Err(LifecycleHookError::new(
                LifecycleHookErrorKind::DuplicateCapability,
                "capabilities",
                capability.as_str(),
            ));
        }
        if !event_kind.supports_capability(*capability) {
            return Err(LifecycleHookError::new(
                LifecycleHookErrorKind::UnsupportedCapability,
                "capabilities",
                capability.as_str(),
            ));
        }
        normalized.push(*capability);
    }
    normalized.sort();
    Ok(normalized)
}

fn validate_safe_summary<'a>(field: &'static str, value: &'a str) -> Result<&'a str, LifecycleHookError> {
    validate_safe_text(field, value, MAX_SAFE_SUMMARY_BYTES)
}

fn validate_safe_text<'a>(field: &'static str, value: &'a str, max_len: usize) -> Result<&'a str, LifecycleHookError> {
    if value.is_empty() || value.len() > max_len || leak_marker(value).is_some() {
        return Err(LifecycleHookError::new(LifecycleHookErrorKind::InvalidLabel, field, value));
    }
    Ok(value)
}

fn validate_blake3_hex<'a>(field: &'static str, value: &'a str) -> Result<&'a str, LifecycleHookError> {
    if value.len() != BLAKE3_HEX_LENGTH || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(LifecycleHookError::new(LifecycleHookErrorKind::InvalidHash, field, value));
    }
    Ok(value)
}

fn leak_marker(value: &str) -> Option<&'static str> {
    let lower = value.to_ascii_lowercase();
    LEAK_MARKERS.iter().copied().find(|marker| lower.contains(marker))
}

fn validate_redacted_body_summary(
    body_size_bytes: usize,
    body_hash: &str,
    redaction: LifecycleHookRedaction,
) -> Result<&str, LifecycleHookError> {
    match redaction {
        LifecycleHookRedaction::NoSensitiveBody => {
            let expected_hash = no_sensitive_body_hash();
            if body_size_bytes == usize::default() && body_hash == expected_hash {
                Ok(body_hash)
            } else {
                Err(LifecycleHookError::new(LifecycleHookErrorKind::InvalidHash, "body_hash", body_hash))
            }
        }
        LifecycleHookRedaction::BodyRedacted => validate_blake3_hex("body_hash", body_hash),
    }
}

fn summarize_sensitive_body(body: Option<&str>) -> (usize, String, LifecycleHookRedaction) {
    body.map_or_else(
        || (usize::default(), no_sensitive_body_hash(), LifecycleHookRedaction::NoSensitiveBody),
        |body| (body.len(), blake3_hex(body.as_bytes()), LifecycleHookRedaction::BodyRedacted),
    )
}

fn no_sensitive_body_hash() -> String {
    blake3_hex(EMPTY_SENSITIVE_BODY_HASH.as_bytes())
}

#[derive(Clone, Copy)]
struct DefinitionHashInput<'a> {
    hook_id: &'a str,
    event_kind: LifecycleHookEventKind,
    source: LifecycleHookDefinitionSource,
    capabilities: &'a [LifecycleHookCapability],
    definition_body: &'a str,
}

fn hash_definition(input: DefinitionHashInput<'_>) -> String {
    let mut normalized = String::from(HOOK_DEFINITION_SCHEMA);
    push_hash_part(&mut normalized, input.hook_id);
    push_hash_part(&mut normalized, input.event_kind.as_str());
    push_hash_part(&mut normalized, input.source.as_str());
    for capability in input.capabilities {
        push_hash_part(&mut normalized, capability.as_str());
    }
    push_hash_part(&mut normalized, input.definition_body);
    blake3_hex(normalized.as_bytes())
}

fn push_hash_part(output: &mut String, part: &str) {
    output.push('|');
    output.push_str(part.len().to_string().as_str());
    output.push(':');
    output.push_str(part);
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOL_HOOK_ID: &str = "tool_guard";
    const PROMPT_HOOK_ID: &str = "prompt_context";
    const STOP_HOOK_ID: &str = "stop_continuation";
    const PERMISSION_HOOK_ID: &str = "permission_guard";
    const MODEL_HOOK_ID: &str = "model_guard";
    const PRE_TURN_HOOK_ID: &str = "pre_turn_context";
    const POST_TURN_HOOK_ID: &str = "post_turn_followup";
    const PRE_TURN_DEFINITION_BODY: &str = "event=pre_turn;effect=additional_context";
    const POST_TURN_DEFINITION_BODY: &str = "event=post_turn;effect=request_continuation";
    const USER_DEFINITION_BODY: &str = "event=pre_tool_use;effect=deny";
    const CHANGED_DEFINITION_BODY: &str = "event=pre_tool_use;effect=warning";
    const PROMPT_DEFINITION_BODY: &str = "event=user_prompt_submit;effect=additional_context";
    const STOP_DEFINITION_BODY: &str = "event=stop;effect=request_continuation";
    const PERMISSION_DEFINITION_BODY: &str = "event=permission_request;effect=deny";
    const MODEL_DEFINITION_BODY: &str = "event=model_request;effect=request_revalidation";
    const EVENT_ID: &str = "evt_safe_001";
    const TOOL_LABEL: &str = "shell_exec";
    const PROMPT_LABEL: &str = "prompt_redacted";
    const RESOURCE_TOOL: &str = "tool";
    const RESOURCE_PROMPT: &str = "prompt";
    const RESOURCE_RUN: &str = "run";
    const RESOURCE_TURN: &str = "turn";
    const SAFE_CONTEXT: &str = "safe bounded context";
    const SAFE_DENIAL: &str = "policy_denied";
    const SAFE_CONTINUATION: &str = "continue_after_stop";
    const SAFE_REVALIDATION: &str = "revalidate_action";
    const RAW_PROMPT_BODY: &str = "raw_prompt: secret user prompt";
    const RAW_COMMAND_BODY: &str = "rm -rf forbidden";
    const APPROVAL_REASON: &str = "approval_required";
    const PRIOR_DENIAL: &str = "sandbox_denied";

    fn tool_definition(body: &str) -> LifecycleHookDefinition {
        LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: TOOL_HOOK_ID,
            event_kind: LifecycleHookEventKind::PreToolUse.as_str(),
            source: LifecycleHookDefinitionSource::User,
            capabilities: &[
                LifecycleHookCapability::Observe,
                LifecycleHookCapability::Deny,
                LifecycleHookCapability::RequestRevalidation,
            ],
            definition_body: body,
        })
        .expect("tool hook definition should validate")
    }

    fn prompt_definition() -> LifecycleHookDefinition {
        LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: PROMPT_HOOK_ID,
            event_kind: LifecycleHookEventKind::UserPromptSubmit.as_str(),
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[LifecycleHookCapability::Observe, LifecycleHookCapability::Annotate],
            definition_body: PROMPT_DEFINITION_BODY,
        })
        .expect("prompt hook definition should validate")
    }

    fn stop_definition() -> LifecycleHookDefinition {
        LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: STOP_HOOK_ID,
            event_kind: LifecycleHookEventKind::Stop.as_str(),
            source: LifecycleHookDefinitionSource::Project,
            capabilities: &[
                LifecycleHookCapability::Observe,
                LifecycleHookCapability::Deny,
                LifecycleHookCapability::RequestContinuation,
            ],
            definition_body: STOP_DEFINITION_BODY,
        })
        .expect("stop hook definition should validate")
    }

    fn permission_definition() -> LifecycleHookDefinition {
        LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: PERMISSION_HOOK_ID,
            event_kind: LifecycleHookEventKind::PermissionRequest.as_str(),
            source: LifecycleHookDefinitionSource::Project,
            capabilities: &[LifecycleHookCapability::Observe, LifecycleHookCapability::Deny],
            definition_body: PERMISSION_DEFINITION_BODY,
        })
        .expect("permission hook definition should validate")
    }

    fn model_definition() -> LifecycleHookDefinition {
        LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: MODEL_HOOK_ID,
            event_kind: LifecycleHookEventKind::ModelRequest.as_str(),
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[
                LifecycleHookCapability::Observe,
                LifecycleHookCapability::RequestRevalidation,
            ],
            definition_body: MODEL_DEFINITION_BODY,
        })
        .expect("model hook definition should validate")
    }

    fn pre_turn_definition() -> LifecycleHookDefinition {
        LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: PRE_TURN_HOOK_ID,
            event_kind: LifecycleHookEventKind::PreTurn.as_str(),
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[
                LifecycleHookCapability::Observe,
                LifecycleHookCapability::Annotate,
                LifecycleHookCapability::Deny,
            ],
            definition_body: PRE_TURN_DEFINITION_BODY,
        })
        .expect("pre-turn hook definition should validate")
    }

    fn post_turn_definition() -> LifecycleHookDefinition {
        LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: POST_TURN_HOOK_ID,
            event_kind: LifecycleHookEventKind::PostTurn.as_str(),
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[
                LifecycleHookCapability::Observe,
                LifecycleHookCapability::Annotate,
                LifecycleHookCapability::RequestContinuation,
            ],
            definition_body: POST_TURN_DEFINITION_BODY,
        })
        .expect("post-turn hook definition should validate")
    }

    fn input(kind: LifecycleHookEventKind, label: &str, sensitive_body: Option<&str>) -> LifecycleHookInput {
        let resource_class = if kind == LifecycleHookEventKind::UserPromptSubmit {
            RESOURCE_PROMPT
        } else if matches!(kind, LifecycleHookEventKind::PreTurn | LifecycleHookEventKind::PostTurn) {
            RESOURCE_TURN
        } else if kind == LifecycleHookEventKind::Stop {
            RESOURCE_RUN
        } else {
            RESOURCE_TOOL
        };
        LifecycleHookInput::validate(&LifecycleHookInputDraft {
            event_id: EVENT_ID,
            event_kind: kind.as_str(),
            resource_class,
            safe_label: label,
            summary: label,
            sensitive_body,
            status: Some("pending"),
        })
        .expect("hook input should validate")
    }

    fn output(
        event_kind: LifecycleHookEventKind,
        effects: &[LifecycleHookEffectDraft<'_>],
    ) -> Result<LifecycleHookOutput, LifecycleHookError> {
        LifecycleHookOutput::validate(event_kind, &LifecycleHookOutputDraft { effects })
    }

    #[test]
    fn typed_event_registry_routes_matching_trusted_hook_only() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.event_registry]
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.event_registry.routing]
        let tool = tool_definition(USER_DEFINITION_BODY);
        let prompt = prompt_definition();
        let records = vec![
            LifecycleHookTrustRecord::trusted(&tool, LifecycleHookTrustScope::User),
            LifecycleHookTrustRecord::trusted(&prompt, LifecycleHookTrustScope::Managed),
        ];
        let definitions = vec![tool.clone(), prompt];
        let route = route_lifecycle_hook_requests(
            &input(LifecycleHookEventKind::PreToolUse, TOOL_LABEL, Some(RAW_COMMAND_BODY)),
            &definitions,
            &records,
        );

        assert_eq!(route.requests().len(), 1);
        assert_eq!(route.requests()[0].hook_id(), tool.hook_id());
        assert_eq!(route.requests()[0].event_kind(), LifecycleHookEventKind::PreToolUse);
        assert!(route.outcomes().is_empty());
    }

    #[test]
    fn unknown_event_kind_fails_validation() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.event_registry.routing]
        // r[verify agent_harness_runtime.claude_code_hooks.points.unknown]
        let error = LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: TOOL_HOOK_ID,
            event_kind: "surprise_event",
            source: LifecycleHookDefinitionSource::User,
            capabilities: &[LifecycleHookCapability::Observe],
            definition_body: USER_DEFINITION_BODY,
        })
        .expect_err("unknown events should fail");

        assert_eq!(error.kind(), LifecycleHookErrorKind::UnknownEvent);
        assert_eq!(error.field(), "event_kind");
    }

    #[test]
    fn claude_hook_points_cover_required_lifecycle_vocabulary() {
        // r[verify agent_harness_runtime.claude_code_hooks.points]
        let event_labels = [
            ("session_start", LifecycleHookEventKind::SessionStart),
            ("session_end", LifecycleHookEventKind::SessionEnd),
            ("user_prompt_submit", LifecycleHookEventKind::UserPromptSubmit),
            ("prompt_assembly", LifecycleHookEventKind::PromptAssembly),
            ("pre_turn", LifecycleHookEventKind::PreTurn),
            ("post_turn", LifecycleHookEventKind::PostTurn),
            ("model_request", LifecycleHookEventKind::ModelRequest),
            ("model_response", LifecycleHookEventKind::ModelResponse),
            ("pre_tool_use", LifecycleHookEventKind::PreToolUse),
            ("permission_request", LifecycleHookEventKind::PermissionRequest),
            ("post_tool_use", LifecycleHookEventKind::PostToolUse),
            ("pre_mutation", LifecycleHookEventKind::PreMutation),
            ("post_mutation", LifecycleHookEventKind::PostMutation),
            ("pre_compact", LifecycleHookEventKind::PreCompact),
            ("post_compact", LifecycleHookEventKind::PostCompact),
            ("validation", LifecycleHookEventKind::Validation),
            ("receipt_emission", LifecycleHookEventKind::ReceiptEmission),
            ("subagent_start", LifecycleHookEventKind::SubagentStart),
            ("subagent_stop", LifecycleHookEventKind::SubagentStop),
            ("workflow_completion", LifecycleHookEventKind::WorkflowCompletion),
            ("stop", LifecycleHookEventKind::Stop),
        ];

        for (label, event_kind) in event_labels {
            assert_eq!(LifecycleHookEventKind::parse(label), Ok(event_kind));
            assert_eq!(event_kind.as_str(), label);
        }
    }

    #[test]
    fn startup_alias_is_rejected_in_favor_of_session_start() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.scope.startup]
        let error = LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: "startup_alias",
            event_kind: "startup",
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[LifecycleHookCapability::Observe],
            definition_body: "event=startup;effect=observe",
        })
        .expect_err("startup alias should fail");

        assert_eq!(error.kind(), LifecycleHookErrorKind::UnknownEvent);
        assert_eq!(LifecycleHookEventKind::SessionStart.as_str(), "session_start");
    }

    #[test]
    fn turn_hook_definitions_accept_supported_capabilities() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.points]
        let pre_turn = pre_turn_definition();
        let post_turn = post_turn_definition();

        assert_eq!(pre_turn.event_kind(), LifecycleHookEventKind::PreTurn);
        assert!(pre_turn.declares_capability(LifecycleHookCapability::Deny));
        assert_eq!(post_turn.event_kind(), LifecycleHookEventKind::PostTurn);
        assert!(post_turn.declares_capability(LifecycleHookCapability::RequestContinuation));
    }

    #[test]
    fn post_turn_definition_rejects_pre_action_capabilities() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.points.negative]
        let denial = LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: POST_TURN_HOOK_ID,
            event_kind: LifecycleHookEventKind::PostTurn.as_str(),
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[LifecycleHookCapability::Deny],
            definition_body: POST_TURN_DEFINITION_BODY,
        })
        .expect_err("post-turn denial capability should fail");
        let revalidation = LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: POST_TURN_HOOK_ID,
            event_kind: LifecycleHookEventKind::PostTurn.as_str(),
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[LifecycleHookCapability::RequestRevalidation],
            definition_body: POST_TURN_DEFINITION_BODY,
        })
        .expect_err("post-turn revalidation capability should fail");

        assert_eq!(denial.kind(), LifecycleHookErrorKind::UnsupportedCapability);
        assert_eq!(revalidation.kind(), LifecycleHookErrorKind::UnsupportedCapability);
    }

    #[test]
    fn hook_definitions_require_supported_unique_capabilities() {
        // r[verify agent_harness_runtime.claude_code_hooks.execution]
        // r[verify agent_harness_runtime.claude_code_hooks.points.unknown]
        let missing = LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: TOOL_HOOK_ID,
            event_kind: LifecycleHookEventKind::PreToolUse.as_str(),
            source: LifecycleHookDefinitionSource::User,
            capabilities: &[],
            definition_body: USER_DEFINITION_BODY,
        })
        .expect_err("missing capabilities should fail");
        let duplicate = LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: TOOL_HOOK_ID,
            event_kind: LifecycleHookEventKind::PreToolUse.as_str(),
            source: LifecycleHookDefinitionSource::User,
            capabilities: &[LifecycleHookCapability::Observe, LifecycleHookCapability::Observe],
            definition_body: USER_DEFINITION_BODY,
        })
        .expect_err("duplicate capabilities should fail");
        let unsupported = LifecycleHookDefinition::validate(&LifecycleHookDefinitionDraft {
            hook_id: PROMPT_HOOK_ID,
            event_kind: LifecycleHookEventKind::PromptAssembly.as_str(),
            source: LifecycleHookDefinitionSource::Managed,
            capabilities: &[LifecycleHookCapability::Deny],
            definition_body: PROMPT_DEFINITION_BODY,
        })
        .expect_err("unsupported capabilities should fail");

        assert_eq!(missing.kind(), LifecycleHookErrorKind::MissingCapability);
        assert_eq!(duplicate.kind(), LifecycleHookErrorKind::DuplicateCapability);
        assert_eq!(unsupported.kind(), LifecycleHookErrorKind::UnsupportedCapability);
    }

    #[test]
    fn changed_user_hook_definition_loses_prior_trust() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.trust_identity]
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.trust_identity.changed]
        let original = tool_definition(USER_DEFINITION_BODY);
        let changed = tool_definition(CHANGED_DEFINITION_BODY);
        let records = vec![LifecycleHookTrustRecord::trusted(
            &original,
            LifecycleHookTrustScope::User,
        )];

        assert_ne!(original.definition_hash(), changed.definition_hash());
        assert_eq!(evaluate_lifecycle_hook_trust(&changed, &records), LifecycleHookTrustStatus::Untrusted);
    }

    #[test]
    fn managed_policy_can_cover_changed_hook_hash() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.trust_identity.changed]
        let original = tool_definition(USER_DEFINITION_BODY);
        let changed = tool_definition(CHANGED_DEFINITION_BODY);
        let records = vec![
            LifecycleHookTrustRecord::trusted(&original, LifecycleHookTrustScope::User),
            LifecycleHookTrustRecord::trusted(&changed, LifecycleHookTrustScope::Managed),
        ];

        assert_eq!(evaluate_lifecycle_hook_trust(&changed, &records), LifecycleHookTrustStatus::Trusted {
            scope: LifecycleHookTrustScope::Managed
        });
    }

    #[test]
    fn prompt_hook_accepts_event_scoped_additional_context() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.io_contract]
        let validated = output(LifecycleHookEventKind::UserPromptSubmit, &[LifecycleHookEffectDraft {
            kind: "additional_context",
            value: SAFE_CONTEXT,
        }])
        .expect("prompt context effect should validate");

        assert!(matches!(
            validated.effects(),
            [LifecycleHookEffect::AdditionalContext { summary, context_hash }]
                if summary == SAFE_CONTEXT && context_hash.len() == BLAKE3_HEX_LENGTH
        ));
    }

    #[test]
    fn permission_hook_accepts_deny_effect() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.io_contract]
        let validated = output(LifecycleHookEventKind::PermissionRequest, &[LifecycleHookEffectDraft {
            kind: "deny",
            value: SAFE_DENIAL,
        }])
        .expect("permission deny effect should validate");

        assert!(matches!(
            validated.effects(),
            [LifecycleHookEffect::Deny { reason }] if reason == SAFE_DENIAL
        ));
    }

    #[test]
    fn request_revalidation_effect_requires_declared_capability() {
        // r[verify agent_harness_runtime.claude_code_hooks.outcomes]
        // r[verify agent_harness_runtime.claude_code_hooks.outcomes.invalid_modification]
        let model = model_definition();
        let validated = LifecycleHookOutput::validate_for_definition(&model, &LifecycleHookOutputDraft {
            effects: &[LifecycleHookEffectDraft {
                kind: "request_revalidation",
                value: SAFE_REVALIDATION,
            }],
        })
        .expect("declared request-revalidation effect should validate");
        let tool = tool_definition(USER_DEFINITION_BODY);
        let denied = LifecycleHookOutput::validate_for_definition(&tool, &LifecycleHookOutputDraft {
            effects: &[LifecycleHookEffectDraft {
                kind: "warning",
                value: SAFE_CONTEXT,
            }],
        })
        .expect_err("undeclared annotate effect should fail");

        assert!(matches!(
            validated.effects(),
            [LifecycleHookEffect::RequestRevalidation { reason }] if reason == SAFE_REVALIDATION
        ));
        assert_eq!(denied.kind(), LifecycleHookErrorKind::CapabilityDenied);
    }

    #[test]
    fn stop_hook_accepts_continuation_request() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.io_contract]
        let validated = output(LifecycleHookEventKind::Stop, &[LifecycleHookEffectDraft {
            kind: "request_continuation",
            value: SAFE_CONTINUATION,
        }])
        .expect("stop continuation effect should validate");

        assert!(matches!(
            validated.effects(),
            [LifecycleHookEffect::RequestContinuation { reason }] if reason == SAFE_CONTINUATION
        ));
    }

    #[test]
    fn pre_turn_accepts_context_and_denial_effects() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.no_bypass]
        let context = output(LifecycleHookEventKind::PreTurn, &[LifecycleHookEffectDraft {
            kind: "additional_context",
            value: SAFE_CONTEXT,
        }])
        .expect("pre-turn context effect should validate");
        let denial = output(LifecycleHookEventKind::PreTurn, &[LifecycleHookEffectDraft {
            kind: "deny",
            value: SAFE_DENIAL,
        }])
        .expect("pre-turn denial effect should validate before evaluation");

        assert!(matches!(
            context.effects(),
            [LifecycleHookEffect::AdditionalContext { summary, .. }] if summary == SAFE_CONTEXT
        ));
        assert!(matches!(
            denial.effects(),
            [LifecycleHookEffect::Deny { reason }] if reason == SAFE_DENIAL
        ));
    }

    #[test]
    fn post_turn_accepts_warning_and_continuation_effects() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.no_bypass]
        let warning = output(LifecycleHookEventKind::PostTurn, &[LifecycleHookEffectDraft {
            kind: "warning",
            value: SAFE_CONTEXT,
        }])
        .expect("post-turn warning effect should validate");
        let continuation = output(LifecycleHookEventKind::PostTurn, &[LifecycleHookEffectDraft {
            kind: "request_continuation",
            value: SAFE_CONTINUATION,
        }])
        .expect("post-turn continuation effect should validate");

        assert!(matches!(
            warning.effects(),
            [LifecycleHookEffect::Warning { message }] if message == SAFE_CONTEXT
        ));
        assert!(matches!(
            continuation.effects(),
            [LifecycleHookEffect::RequestContinuation { reason }] if reason == SAFE_CONTINUATION
        ));
    }

    #[test]
    fn malformed_outputs_reject_empty_effect_values() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.io_contract.unsupported]
        let fixtures = [
            (LifecycleHookEventKind::UserPromptSubmit, "additional_context"),
            (LifecycleHookEventKind::UserPromptSubmit, "warning"),
            (LifecycleHookEventKind::PreToolUse, "deny"),
            (LifecycleHookEventKind::PreTurn, "deny"),
            (LifecycleHookEventKind::PostTurn, "request_continuation"),
            (LifecycleHookEventKind::Stop, "request_continuation"),
        ];

        for (event_kind, effect_kind) in fixtures {
            let error = output(event_kind, &[LifecycleHookEffectDraft {
                kind: effect_kind,
                value: "",
            }])
            .expect_err("empty effect values should fail");
            assert_eq!(error.kind(), LifecycleHookErrorKind::InvalidLabel);
        }
    }

    #[test]
    fn unsupported_effects_are_rejected_per_event_contract() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.io_contract.unsupported]
        let fixtures = [
            (LifecycleHookEventKind::PostToolUse, "additional_context"),
            (LifecycleHookEventKind::PostToolUse, "deny"),
            (LifecycleHookEventKind::PostTurn, "additional_context"),
            (LifecycleHookEventKind::PostTurn, "deny"),
            (LifecycleHookEventKind::PostTurn, "request_revalidation"),
            (LifecycleHookEventKind::PreToolUse, "request_continuation"),
            (LifecycleHookEventKind::UserPromptSubmit, "rewrite_tool_input"),
        ];

        for (event_kind, effect_kind) in fixtures {
            let error = output(event_kind, &[LifecycleHookEffectDraft {
                kind: effect_kind,
                value: SAFE_CONTEXT,
            }])
            .expect_err("unsupported effects should fail");
            assert_eq!(error.kind(), LifecycleHookErrorKind::UnsupportedEffect);
        }
    }

    #[test]
    fn post_turn_policy_bypass_effects_are_rejected_before_binding() {
        // r[verify agent_harness_runtime.turn_lifecycle_hooks.no_bypass.post_turn]
        let fixtures = [
            "execute_tool",
            "rewrite_denied_action",
            "weaken_sandbox",
            "suppress_approval",
        ];

        for effect_kind in fixtures {
            let error = output(LifecycleHookEventKind::PostTurn, &[LifecycleHookEffectDraft {
                kind: effect_kind,
                value: SAFE_CONTEXT,
            }])
            .expect_err("post-turn policy bypass effects should fail");
            assert_eq!(error.kind(), LifecycleHookErrorKind::PolicyBypass);
        }
    }

    #[test]
    fn hook_policy_binding_keeps_denied_action_denied() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.policy_binding]
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.policy_binding.bypass]
        // r[verify agent_harness_runtime.claude_code_hooks.scope]
        // r[verify agent_harness_runtime.claude_code_hooks.scope.no_bypass]
        let safe_context = output(LifecycleHookEventKind::UserPromptSubmit, &[LifecycleHookEffectDraft {
            kind: "additional_context",
            value: SAFE_CONTEXT,
        }])
        .expect("safe context should validate");
        let status = bind_lifecycle_hook_output_to_policy(
            LifecycleHookPolicyStatus::Denied {
                reason: String::from(PRIOR_DENIAL),
            },
            &safe_context,
        );

        assert_eq!(status, LifecycleHookPolicyStatus::Denied {
            reason: String::from(PRIOR_DENIAL)
        });
    }

    #[test]
    fn hook_deny_can_make_allowed_or_approval_required_boundary_stricter() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.policy_binding]
        let deny_output = output(LifecycleHookEventKind::PreToolUse, &[LifecycleHookEffectDraft {
            kind: "deny",
            value: SAFE_DENIAL,
        }])
        .expect("deny output should validate");

        assert_eq!(
            bind_lifecycle_hook_output_to_policy(LifecycleHookPolicyStatus::Allowed, &deny_output),
            LifecycleHookPolicyStatus::Denied {
                reason: String::from(SAFE_DENIAL)
            }
        );
        assert_eq!(
            bind_lifecycle_hook_output_to_policy(
                LifecycleHookPolicyStatus::ApprovalRequired {
                    reason: String::from(APPROVAL_REASON),
                },
                &deny_output,
            ),
            LifecycleHookPolicyStatus::Denied {
                reason: String::from(SAFE_DENIAL)
            }
        );
    }

    #[test]
    fn policy_bypass_effects_are_rejected_before_binding() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.policy_binding.bypass]
        // r[verify agent_harness_runtime.claude_code_hooks.scope.no_bypass]
        // r[verify agent_harness_runtime.claude_code_hooks.outcomes.invalid_modification]
        let fixtures = [
            "execute_tool",
            "rewrite_denied_action",
            "weaken_sandbox",
            "suppress_approval",
        ];

        for effect_kind in fixtures {
            let error = output(LifecycleHookEventKind::PreToolUse, &[LifecycleHookEffectDraft {
                kind: effect_kind,
                value: SAFE_CONTEXT,
            }])
            .expect_err("policy bypass effects should fail");
            assert_eq!(error.kind(), LifecycleHookErrorKind::PolicyBypass);
        }
    }

    #[test]
    fn hook_input_payload_exposes_bounded_metadata_without_leaks() {
        // r[verify agent_harness_runtime.claude_code_hooks.payloads]
        // r[verify agent_harness_runtime.claude_code_hooks.payloads.bounded]
        let input =
            input(LifecycleHookEventKind::PostToolUse, TOOL_LABEL, Some("tool result body that stays redacted"));
        let leak = LifecycleHookInput::validate(&LifecycleHookInputDraft {
            event_id: EVENT_ID,
            event_kind: LifecycleHookEventKind::PostToolUse.as_str(),
            resource_class: RESOURCE_TOOL,
            safe_label: TOOL_LABEL,
            summary: "credential: leaked",
            sensitive_body: None,
            status: Some("done"),
        })
        .expect_err("leaky summary should fail");

        assert_eq!(input.event_id(), EVENT_ID);
        assert_eq!(input.resource_class(), RESOURCE_TOOL);
        assert_eq!(input.safe_label(), TOOL_LABEL);
        assert_eq!(input.summary(), TOOL_LABEL);
        assert_eq!(input.status(), Some("pending"));
        assert_eq!(input.redaction(), LifecycleHookRedaction::BodyRedacted);
        assert!(input.body_size_bytes() > usize::default());
        assert_eq!(input.body_hash().len(), BLAKE3_HEX_LENGTH);
        assert_eq!(leak.kind(), LifecycleHookErrorKind::InvalidLabel);
    }

    #[test]
    fn outcome_reports_redact_sensitive_input_and_keep_identity_auditable() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.outcome_reporting]
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.outcome_reporting.redacted]
        // r[verify agent_harness_runtime.claude_code_hooks.payloads.bounded]
        let definition = prompt_definition();
        let trust_status = LifecycleHookTrustStatus::Trusted {
            scope: LifecycleHookTrustScope::Managed,
        };
        let input = input(LifecycleHookEventKind::UserPromptSubmit, PROMPT_LABEL, Some(RAW_PROMPT_BODY));
        let report = LifecycleHookOutcome::new(LifecycleHookOutcomeDraft {
            definition: &definition,
            input: &input,
            trust_status,
            outcome_class: LifecycleHookOutcomeClass::Applied,
            failure_class: None,
        });
        let debug = format!("{report:?}");

        assert_eq!(report.hook_id(), PROMPT_HOOK_ID);
        assert_eq!(report.definition_hash(), definition.definition_hash());
        assert_eq!(report.input().redaction(), LifecycleHookRedaction::BodyRedacted);
        assert_ne!(report.input().body_hash(), RAW_PROMPT_BODY);
        assert!(!debug.contains(RAW_PROMPT_BODY));
        assert!(debug.contains(definition.definition_hash()));
    }

    #[test]
    fn route_plan_records_untrusted_and_denied_matching_hooks() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.outcome_reporting]
        let untrusted = tool_definition(USER_DEFINITION_BODY);
        let denied = permission_definition();
        let records = vec![LifecycleHookTrustRecord::denied(
            &denied,
            LifecycleHookTrustScope::Project,
        )];
        let definitions = vec![untrusted, denied.clone()];
        let route = route_lifecycle_hook_requests(
            &input(LifecycleHookEventKind::PermissionRequest, TOOL_LABEL, None),
            &definitions,
            &records,
        );

        assert!(route.requests().is_empty());
        assert_eq!(route.outcomes().len(), 1);
        assert_eq!(route.outcomes()[0].hook_id(), denied.hook_id());
        assert_eq!(route.outcomes()[0].failure_class(), Some(LifecycleHookFailureClass::HookDeniedByTrust));
    }

    #[test]
    fn required_hook_unavailable_is_typed_failed_outcome() {
        // r[verify agent_harness_runtime.openai_codex_hook_trust_lifecycle.outcome_reporting]
        // r[verify agent_harness_runtime.claude_code_hooks.execution.missing_service]
        let definition = stop_definition();
        let trust_status = LifecycleHookTrustStatus::Trusted {
            scope: LifecycleHookTrustScope::Project,
        };
        let report = LifecycleHookOutcome::required_service_unavailable(
            &definition,
            &input(LifecycleHookEventKind::Stop, "run_stop", None),
            trust_status,
        );

        assert_eq!(report.outcome_class(), LifecycleHookOutcomeClass::Failed);
        assert_eq!(report.failure_class(), Some(LifecycleHookFailureClass::RequiredHookUnavailable));
        assert_eq!(report.trust_status(), trust_status);
    }
}
