const REQUEST_SHAPE_PROMPT_TEXT: &str = "prompt_text";
const STREAM_POLICY_NON_STREAMING: &str = "non_streaming";
const RECEIPT_POLICY_SAFE_REQUIRED: &str = "safe_receipt_required";
const CANCELLATION_CONTINUE: &str = "continue";
const CANCELLATION_CANCELLED: &str = "cancelled";
const TIMEOUT_WITHIN_DEADLINE: &str = "within_deadline";
const TIMEOUT_TIMED_OUT: &str = "timed_out";
const FALLBACK_DENY: &str = "deny";
const FALLBACK_REQUESTED: bool = true;
const FALLBACK_NOT_REQUESTED: bool = false;
const ISSUE_NONE: &str = "none";
const ISSUE_INVALID_CAPABILITY: &str = "invalid_capability";
const ISSUE_MISSING_PROVIDER: &str = "missing_provider";
const ISSUE_DENIED_CAPABILITY: &str = "denied_capability";
const ISSUE_BUDGET_OVERFLOW: &str = "budget_overflow";
const ISSUE_MALFORMED_REQUEST_SHAPE: &str = "malformed_request_shape";
const ISSUE_UNSUPPORTED_STREAM_POLICY: &str = "unsupported_stream_policy";
const ISSUE_MISSING_RECEIPT_POLICY: &str = "missing_receipt_policy";
const ISSUE_CANCELLED: &str = "cancelled";
const ISSUE_TIMED_OUT: &str = "timed_out";
const ISSUE_FALLBACK_DENIED: &str = "fallback_denied";

// r[impl agent_harness_runtime.model_dispatch.plan]
/// Pure model/provider dispatch plan status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDispatchPlanStatus {
    /// Model adapter may be called by the imperative shell.
    Admitted,
    /// Model adapter must not be called.
    Denied,
}

/// First fail-closed reason found by model dispatch planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDispatchPlanIssue {
    /// No issue was found.
    None,
    /// Required capability metadata was malformed.
    InvalidCapability,
    /// Required provider/capability was not registered exactly.
    MissingProvider,
    /// Capability policy denied dispatch.
    DeniedCapability,
    /// Prompt/content budget would be exceeded.
    BudgetOverflow,
    /// Request shape was malformed or unsupported.
    MalformedRequestShape,
    /// Stream policy was unsupported.
    UnsupportedStreamPolicy,
    /// Receipt policy was missing or unsupported.
    MissingReceiptPolicy,
    /// Cancellation gate stopped dispatch.
    Cancelled,
    /// Timeout gate stopped dispatch.
    TimedOut,
    /// Fallback was requested but fallback policy denies rerouting.
    FallbackDenied,
}

impl ModelDispatchPlanIssue {
    /// Returns the stable issue label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => ISSUE_NONE,
            Self::InvalidCapability => ISSUE_INVALID_CAPABILITY,
            Self::MissingProvider => ISSUE_MISSING_PROVIDER,
            Self::DeniedCapability => ISSUE_DENIED_CAPABILITY,
            Self::BudgetOverflow => ISSUE_BUDGET_OVERFLOW,
            Self::MalformedRequestShape => ISSUE_MALFORMED_REQUEST_SHAPE,
            Self::UnsupportedStreamPolicy => ISSUE_UNSUPPORTED_STREAM_POLICY,
            Self::MissingReceiptPolicy => ISSUE_MISSING_RECEIPT_POLICY,
            Self::Cancelled => ISSUE_CANCELLED,
            Self::TimedOut => ISSUE_TIMED_OUT,
            Self::FallbackDenied => ISSUE_FALLBACK_DENIED,
        }
    }
}

/// Supported provider-neutral model stream policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDispatchStreamPolicy {
    /// Execute as a non-streaming model call.
    NonStreaming,
}

impl ModelDispatchStreamPolicy {
    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NonStreaming => STREAM_POLICY_NON_STREAMING,
        }
    }
}

/// Supported model dispatch receipt policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDispatchReceiptPolicy {
    /// Safe receipt metadata is required; raw payloads are not receipt fields.
    SafeReceiptRequired,
}

impl ModelDispatchReceiptPolicy {
    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SafeReceiptRequired => RECEIPT_POLICY_SAFE_REQUIRED,
        }
    }
}

/// Cancellation posture before model dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDispatchCancellationPosture {
    /// No cancellation has been requested.
    Continue,
    /// Dispatch is cancelled.
    Cancelled,
}

impl ModelDispatchCancellationPosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Continue => CANCELLATION_CONTINUE,
            Self::Cancelled => CANCELLATION_CANCELLED,
        }
    }
}

/// Timeout posture before model dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDispatchTimeoutPosture {
    /// Dispatch is within its caller-owned deadline.
    WithinDeadline,
    /// Dispatch exceeded its caller-owned deadline.
    TimedOut,
}

impl ModelDispatchTimeoutPosture {
    /// Returns the stable posture label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WithinDeadline => TIMEOUT_WITHIN_DEADLINE,
            Self::TimedOut => TIMEOUT_TIMED_OUT,
        }
    }
}

/// Fallback behavior for model dispatch planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelDispatchFallbackPolicy {
    /// Do not route to a different provider/capability automatically.
    Deny,
}

impl ModelDispatchFallbackPolicy {
    /// Returns the stable fallback policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deny => FALLBACK_DENY,
        }
    }
}

/// Borrowed model dispatch policy inputs before pure admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelDispatchPolicyDraft<'a> {
    /// Maximum accepted prompt/content characters.
    pub prompt_budget_chars: u64,
    /// Request shape label.
    pub request_shape: &'a str,
    /// Stream policy label.
    pub stream_policy: &'a str,
    /// Receipt policy label.
    pub receipt_policy: &'a str,
    /// Cancellation posture label.
    pub cancellation_posture: &'a str,
    /// Timeout posture label.
    pub timeout_posture: &'a str,
    /// Fallback policy label.
    pub fallback_policy: &'a str,
    /// Whether dispatch requested fallback to another provider.
    pub fallback_requested: bool,
    /// Whether capability policy admitted the selected capability.
    pub capability_allowed: bool,
}

impl ModelDispatchPolicyDraft<'_> {
    /// Builds compatibility defaults for the existing simple model adapter path.
    #[must_use]
    pub const fn compatibility_defaults() -> Self {
        Self {
            prompt_budget_chars: animus_adapters::MAX_ADAPTER_FIELD_CHARS as u64,
            request_shape: REQUEST_SHAPE_PROMPT_TEXT,
            stream_policy: STREAM_POLICY_NON_STREAMING,
            receipt_policy: RECEIPT_POLICY_SAFE_REQUIRED,
            cancellation_posture: CANCELLATION_CONTINUE,
            timeout_posture: TIMEOUT_WITHIN_DEADLINE,
            fallback_policy: FALLBACK_DENY,
            fallback_requested: FALLBACK_NOT_REQUESTED,
            capability_allowed: true,
        }
    }
}

/// Borrowed pure planning request for one model dispatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModelDispatchPlanRequest<'a> {
    /// Validated model request selected by the loop fabric.
    pub request: &'a animus_adapters::ModelRequest,
    /// Required capability identity for this dispatch.
    pub required_capability: crate::RuntimeCapabilityDescriptorDraft<'a>,
    /// Host-supplied capability registry.
    pub registry: &'a crate::RuntimeCapabilityRegistry,
    /// Dispatch policy inputs.
    pub policy: ModelDispatchPolicyDraft<'a>,
}

/// Pure model/provider dispatch plan safe to inspect before adapter execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelDispatchPlan {
    status: ModelDispatchPlanStatus,
    issue: ModelDispatchPlanIssue,
    capability_label: String,
    capability_scope: Option<String>,
    request_contract_identity: &'static str,
    provider_contract_identity: &'static str,
    prompt_budget_chars: u64,
    stream_policy: Option<ModelDispatchStreamPolicy>,
    receipt_policy: Option<ModelDispatchReceiptPolicy>,
    cancellation_posture: Option<ModelDispatchCancellationPosture>,
    timeout_posture: Option<ModelDispatchTimeoutPosture>,
    fallback_policy: Option<ModelDispatchFallbackPolicy>,
}

impl ModelDispatchPlan {
    /// Returns the plan status.
    #[must_use]
    pub const fn status(&self) -> ModelDispatchPlanStatus {
        self.status
    }

    /// Returns the first issue found by planning.
    #[must_use]
    pub const fn issue(&self) -> ModelDispatchPlanIssue {
        self.issue
    }

    /// Returns true only when model dispatch may proceed.
    #[must_use]
    pub const fn is_admitted(&self) -> bool {
        matches!(self.status, ModelDispatchPlanStatus::Admitted)
    }

    /// Returns the selected capability label.
    #[must_use]
    pub fn capability_label(&self) -> &str {
        self.capability_label.as_str()
    }

    /// Returns the selected safe capability scope, when supplied.
    #[must_use]
    pub fn capability_scope(&self) -> Option<&str> {
        self.capability_scope.as_deref()
    }

    /// Returns the model request contract identity.
    #[must_use]
    pub const fn request_contract_identity(&self) -> &'static str {
        self.request_contract_identity
    }

    /// Returns the provider-neutral contract identity.
    #[must_use]
    pub const fn provider_contract_identity(&self) -> &'static str {
        self.provider_contract_identity
    }

    /// Returns the admitted prompt/content budget.
    #[must_use]
    pub const fn prompt_budget_chars(&self) -> u64 {
        self.prompt_budget_chars
    }

    /// Returns the parsed stream policy, if supported.
    #[must_use]
    pub const fn stream_policy(&self) -> Option<ModelDispatchStreamPolicy> {
        self.stream_policy
    }

    /// Returns the parsed receipt policy, if supported.
    #[must_use]
    pub const fn receipt_policy(&self) -> Option<ModelDispatchReceiptPolicy> {
        self.receipt_policy
    }

    /// Returns the parsed cancellation posture, if supported.
    #[must_use]
    pub const fn cancellation_posture(&self) -> Option<ModelDispatchCancellationPosture> {
        self.cancellation_posture
    }

    /// Returns the parsed timeout posture, if supported.
    #[must_use]
    pub const fn timeout_posture(&self) -> Option<ModelDispatchTimeoutPosture> {
        self.timeout_posture
    }

    /// Returns the parsed fallback policy, if supported.
    #[must_use]
    pub const fn fallback_policy(&self) -> Option<ModelDispatchFallbackPolicy> {
        self.fallback_policy
    }
}

/// Provider-neutral contract identity recorded by admitted model dispatch plans.
pub const MODEL_PROVIDER_NEUTRAL_CONTRACT_IDENTITY: &str = "animus.model-provider-neutral.v1";

/// Builds a pure model dispatch plan without touching provider clients.
#[must_use]
pub fn plan_model_dispatch(request: ModelDispatchPlanRequest<'_>) -> ModelDispatchPlan {
    // r[impl agent_harness_runtime.model_dispatch.runtime_gate]
    let parsed = ParsedPolicy::parse(request.policy);
    let required = match validated_required(request.required_capability) {
        Ok(required) => required,
        Err(_error) => return invalid_capability_plan(request, parsed),
    };
    debug_assert!(!required.label().is_empty());
    debug_assert!(required.scope().is_none_or(|scope| !scope.is_empty()));

    match select_denial_issue(request, &required, parsed) {
        Some(issue) => denied_with_required(issue, &required, request.policy, parsed),
        None => admitted_plan(&required, request.policy, parsed),
    }
}

fn validated_required(
    draft: crate::RuntimeCapabilityDescriptorDraft<'_>,
) -> Result<crate::RuntimeCapabilityDescriptor, crate::RuntimeCapabilityError> {
    crate::RuntimeCapabilityDescriptor::from_draft(draft)
}

fn invalid_capability_plan(
    request: ModelDispatchPlanRequest<'_>,
    parsed: ParsedPolicy,
) -> ModelDispatchPlan {
    denied_plan(
        ModelDispatchPlanIssue::InvalidCapability,
        &CapabilityIdentity {
            label: request.required_capability.label,
            scope: request.required_capability.scope,
        },
        request.policy,
        parsed,
    )
}

fn select_denial_issue(
    request: ModelDispatchPlanRequest<'_>,
    required: &crate::RuntimeCapabilityDescriptor,
    parsed: ParsedPolicy,
) -> Option<ModelDispatchPlanIssue> {
    debug_assert!(!required.label().is_empty());
    debug_assert!(!request.request.prompt().is_empty());
    capability_denial_issue(request, required).or_else(|| policy_denial_issue(request, parsed))
}

fn capability_denial_issue(
    request: ModelDispatchPlanRequest<'_>,
    required: &crate::RuntimeCapabilityDescriptor,
) -> Option<ModelDispatchPlanIssue> {
    if !matches!(
        request.registry.resolve_exact(required),
        crate::RuntimeCapabilityResolution::Resolved { .. }
    ) {
        return Some(ModelDispatchPlanIssue::MissingProvider);
    }
    (!request.policy.capability_allowed).then_some(ModelDispatchPlanIssue::DeniedCapability)
}

fn policy_denial_issue(
    request: ModelDispatchPlanRequest<'_>,
    parsed: ParsedPolicy,
) -> Option<ModelDispatchPlanIssue> {
    prompt_budget_issue(request)
        .or_else(|| request_shape_issue(request.policy))
        .or_else(|| parsed_policy_issue(parsed))
        .or_else(|| fallback_issue(request.policy, parsed))
}

fn prompt_budget_issue(request: ModelDispatchPlanRequest<'_>) -> Option<ModelDispatchPlanIssue> {
    let prompt_chars = count_prompt_chars(request.request.prompt());
    (prompt_chars > request.policy.prompt_budget_chars)
        .then_some(ModelDispatchPlanIssue::BudgetOverflow)
}

fn request_shape_issue(policy: ModelDispatchPolicyDraft<'_>) -> Option<ModelDispatchPlanIssue> {
    (policy.request_shape.trim() != REQUEST_SHAPE_PROMPT_TEXT)
        .then_some(ModelDispatchPlanIssue::MalformedRequestShape)
}

fn parsed_policy_issue(parsed: ParsedPolicy) -> Option<ModelDispatchPlanIssue> {
    if parsed.stream_policy.is_none() {
        return Some(ModelDispatchPlanIssue::UnsupportedStreamPolicy);
    }
    if parsed.receipt_policy.is_none() {
        return Some(ModelDispatchPlanIssue::MissingReceiptPolicy);
    }
    posture_issue(parsed)
}

fn posture_issue(parsed: ParsedPolicy) -> Option<ModelDispatchPlanIssue> {
    if matches!(
        parsed.cancellation_posture,
        Some(ModelDispatchCancellationPosture::Cancelled)
    ) {
        return Some(ModelDispatchPlanIssue::Cancelled);
    }
    matches!(
        parsed.timeout_posture,
        Some(ModelDispatchTimeoutPosture::TimedOut)
    )
    .then_some(ModelDispatchPlanIssue::TimedOut)
}

fn fallback_issue(
    policy: ModelDispatchPolicyDraft<'_>,
    parsed: ParsedPolicy,
) -> Option<ModelDispatchPlanIssue> {
    (policy.fallback_requested == FALLBACK_REQUESTED
        && matches!(
            parsed.fallback_policy,
            Some(ModelDispatchFallbackPolicy::Deny)
        ))
    .then_some(ModelDispatchPlanIssue::FallbackDenied)
}

fn count_prompt_chars(prompt: &str) -> u64 {
    let mut prompt_chars = 0_u64;
    for _character in prompt.chars() {
        prompt_chars = prompt_chars.saturating_add(1);
    }
    prompt_chars
}

fn admitted_plan(
    required: &crate::RuntimeCapabilityDescriptor,
    policy: ModelDispatchPolicyDraft<'_>,
    parsed: ParsedPolicy,
) -> ModelDispatchPlan {
    ModelDispatchPlan {
        status: ModelDispatchPlanStatus::Admitted,
        issue: ModelDispatchPlanIssue::None,
        capability_label: String::from(required.label()),
        capability_scope: required.scope().map(String::from),
        request_contract_identity: animus_adapters::MODEL_REQUEST_CONTRACT_IDENTITY,
        provider_contract_identity: MODEL_PROVIDER_NEUTRAL_CONTRACT_IDENTITY,
        prompt_budget_chars: policy.prompt_budget_chars,
        stream_policy: parsed.stream_policy,
        receipt_policy: parsed.receipt_policy,
        cancellation_posture: parsed.cancellation_posture,
        timeout_posture: parsed.timeout_posture,
        fallback_policy: parsed.fallback_policy,
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ParsedPolicy {
    stream_policy: Option<ModelDispatchStreamPolicy>,
    receipt_policy: Option<ModelDispatchReceiptPolicy>,
    cancellation_posture: Option<ModelDispatchCancellationPosture>,
    timeout_posture: Option<ModelDispatchTimeoutPosture>,
    fallback_policy: Option<ModelDispatchFallbackPolicy>,
}

impl ParsedPolicy {
    fn parse(policy: ModelDispatchPolicyDraft<'_>) -> Self {
        Self {
            stream_policy: parse_stream_policy(policy.stream_policy),
            receipt_policy: parse_receipt_policy(policy.receipt_policy),
            cancellation_posture: parse_cancellation_posture(policy.cancellation_posture),
            timeout_posture: parse_timeout_posture(policy.timeout_posture),
            fallback_policy: parse_fallback_policy(policy.fallback_policy),
        }
    }
}

fn parse_stream_policy(value: &str) -> Option<ModelDispatchStreamPolicy> {
    match value.trim() {
        STREAM_POLICY_NON_STREAMING => Some(ModelDispatchStreamPolicy::NonStreaming),
        _other => None,
    }
}

fn parse_receipt_policy(value: &str) -> Option<ModelDispatchReceiptPolicy> {
    match value.trim() {
        RECEIPT_POLICY_SAFE_REQUIRED => Some(ModelDispatchReceiptPolicy::SafeReceiptRequired),
        _other => None,
    }
}

fn parse_cancellation_posture(value: &str) -> Option<ModelDispatchCancellationPosture> {
    match value.trim() {
        CANCELLATION_CONTINUE => Some(ModelDispatchCancellationPosture::Continue),
        CANCELLATION_CANCELLED => Some(ModelDispatchCancellationPosture::Cancelled),
        _other => None,
    }
}

fn parse_timeout_posture(value: &str) -> Option<ModelDispatchTimeoutPosture> {
    match value.trim() {
        TIMEOUT_WITHIN_DEADLINE => Some(ModelDispatchTimeoutPosture::WithinDeadline),
        TIMEOUT_TIMED_OUT => Some(ModelDispatchTimeoutPosture::TimedOut),
        _other => None,
    }
}

fn parse_fallback_policy(value: &str) -> Option<ModelDispatchFallbackPolicy> {
    match value.trim() {
        FALLBACK_DENY => Some(ModelDispatchFallbackPolicy::Deny),
        _other => None,
    }
}

fn denied_with_required(
    issue: ModelDispatchPlanIssue,
    required: &crate::RuntimeCapabilityDescriptor,
    policy: ModelDispatchPolicyDraft<'_>,
    parsed: ParsedPolicy,
) -> ModelDispatchPlan {
    denied_plan(
        issue,
        &CapabilityIdentity {
            label: required.label(),
            scope: required.scope(),
        },
        policy,
        parsed,
    )
}

/// Labelled capability identity used to construct a denied dispatch plan.
struct CapabilityIdentity<'a> {
    label: &'a str,
    scope: Option<&'a str>,
}

fn denied_plan(
    issue: ModelDispatchPlanIssue,
    capability: &CapabilityIdentity<'_>,
    policy: ModelDispatchPolicyDraft<'_>,
    parsed: ParsedPolicy,
) -> ModelDispatchPlan {
    ModelDispatchPlan {
        status: ModelDispatchPlanStatus::Denied,
        issue,
        capability_label: String::from(capability.label.trim()),
        capability_scope: capability.scope.map(str::trim).map(String::from),
        request_contract_identity: animus_adapters::MODEL_REQUEST_CONTRACT_IDENTITY,
        provider_contract_identity: MODEL_PROVIDER_NEUTRAL_CONTRACT_IDENTITY,
        prompt_budget_chars: policy.prompt_budget_chars,
        stream_policy: parsed.stream_policy,
        receipt_policy: parsed.receipt_policy,
        cancellation_posture: parsed.cancellation_posture,
        timeout_posture: parsed.timeout_posture,
        fallback_policy: parsed.fallback_policy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROMPT: &str = "intent: build reliable systems";
    const TINY_BUDGET: u64 = animus_adapters::MIN_ADAPTER_FIELD_CHARS as u64;
    const CUSTOM_SCOPE: &str = "workspace-main";
    const WRONG_LABEL: &str = "other-model";
    const BAD_POLICY: &str = "provider-native-stream";
    const BAD_SHAPE: &str = "raw-provider-payload";

    #[test]
    fn model_dispatch_plan_admits_compatibility_defaults() {
        // r[verify agent_harness_runtime.model_dispatch.compatibility.equivalent]
        let request = model_request();
        let registry = crate::RuntimeCapabilityRegistry::compatibility_model_tool()
            .expect("compat registry should validate");
        let plan = plan_model_dispatch(ModelDispatchPlanRequest {
            request: &request,
            required_capability: crate::RuntimeCapabilityDescriptorDraft::default_model(),
            registry: &registry,
            policy: ModelDispatchPolicyDraft::compatibility_defaults(),
        });

        assert!(plan.is_admitted());
        assert_eq!(plan.issue(), ModelDispatchPlanIssue::None);
        assert_eq!(
            plan.capability_label(),
            crate::DEFAULT_MODEL_CAPABILITY_LABEL
        );
        assert_eq!(
            plan.request_contract_identity(),
            animus_adapters::MODEL_REQUEST_CONTRACT_IDENTITY
        );
        assert_eq!(
            plan.provider_contract_identity(),
            MODEL_PROVIDER_NEUTRAL_CONTRACT_IDENTITY
        );
    }

    #[test]
    fn model_dispatch_plan_accepts_exact_scoped_capability() {
        // r[verify agent_harness_runtime.scoped_capabilities.required_identity.validation]
        let request = model_request();
        let registry = crate::RuntimeCapabilityRegistry::from_drafts(&[
            crate::RuntimeCapabilityDescriptorDraft {
                label: crate::DEFAULT_MODEL_CAPABILITY_LABEL,
                service_kind: crate::RuntimeCapabilityServiceKind::Model.as_str(),
                scope: Some(CUSTOM_SCOPE),
            },
        ])
        .expect("scoped registry should validate");
        let plan = plan_model_dispatch(ModelDispatchPlanRequest {
            request: &request,
            required_capability: crate::RuntimeCapabilityDescriptorDraft {
                label: crate::DEFAULT_MODEL_CAPABILITY_LABEL,
                service_kind: crate::RuntimeCapabilityServiceKind::Model.as_str(),
                scope: Some(CUSTOM_SCOPE),
            },
            registry: &registry,
            policy: ModelDispatchPolicyDraft::compatibility_defaults(),
        });

        assert!(plan.is_admitted());
        assert_eq!(plan.capability_scope(), Some(CUSTOM_SCOPE));
    }

    #[test]
    fn model_dispatch_plan_denies_missing_denied_budget_stream_shape_and_receipt() {
        // r[verify agent_harness_runtime.model_dispatch.plan.validation]
        let request = model_request();
        let registry = crate::RuntimeCapabilityRegistry::compatibility_model_tool()
            .expect("compat registry should validate");
        let wrong_registry = wrong_registry();
        let default_capability = crate::RuntimeCapabilityDescriptorDraft::default_model();

        assert!(!request.prompt().is_empty());
        assert_eq!(
            default_capability.label,
            crate::DEFAULT_MODEL_CAPABILITY_LABEL
        );

        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &wrong_registry,
                policy: ModelDispatchPolicyDraft::compatibility_defaults(),
            },
            ModelDispatchPlanIssue::MissingProvider,
        );
        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_denied_capability(),
            },
            ModelDispatchPlanIssue::DeniedCapability,
        );
        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_budget_overflow(),
            },
            ModelDispatchPlanIssue::BudgetOverflow,
        );
        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_bad_stream(),
            },
            ModelDispatchPlanIssue::UnsupportedStreamPolicy,
        );
        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_bad_shape(),
            },
            ModelDispatchPlanIssue::MalformedRequestShape,
        );
        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_bad_receipt(),
            },
            ModelDispatchPlanIssue::MissingReceiptPolicy,
        );
    }

    #[test]
    fn model_dispatch_plan_denies_cancel_timeout_and_fallback() {
        // r[verify agent_harness_runtime.model_dispatch.plan.validation]
        let request = model_request();
        let registry = crate::RuntimeCapabilityRegistry::compatibility_model_tool()
            .expect("compat registry should validate");
        let default_capability = crate::RuntimeCapabilityDescriptorDraft::default_model();

        assert!(!request.prompt().is_empty());
        assert_eq!(
            default_capability.label,
            crate::DEFAULT_MODEL_CAPABILITY_LABEL
        );

        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_cancelled(),
            },
            ModelDispatchPlanIssue::Cancelled,
        );
        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_timed_out(),
            },
            ModelDispatchPlanIssue::TimedOut,
        );
        assert_denied_case(
            ModelDispatchPlanRequest {
                request: &request,
                required_capability: default_capability,
                registry: &registry,
                policy: policy_fallback_requested(),
            },
            ModelDispatchPlanIssue::FallbackDenied,
        );
    }

    fn assert_denied_case(
        plan_request: ModelDispatchPlanRequest<'_>,
        expected_issue: ModelDispatchPlanIssue,
    ) {
        let plan = plan_model_dispatch(plan_request);

        assert_eq!(plan.status(), ModelDispatchPlanStatus::Denied);
        assert_eq!(plan.issue(), expected_issue);
        assert!(!plan.is_admitted());
    }

    fn model_request() -> animus_adapters::ModelRequest {
        animus_adapters::ModelRequest::new(PROMPT).expect("prompt should validate")
    }

    fn wrong_registry() -> crate::RuntimeCapabilityRegistry {
        crate::RuntimeCapabilityRegistry::from_drafts(&[crate::RuntimeCapabilityDescriptorDraft {
            label: WRONG_LABEL,
            service_kind: crate::RuntimeCapabilityServiceKind::Model.as_str(),
            scope: None,
        }])
        .expect("wrong registry should validate")
    }

    const fn policy_denied_capability() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            capability_allowed: false,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }

    const fn policy_budget_overflow() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            prompt_budget_chars: TINY_BUDGET,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }

    const fn policy_bad_stream() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            stream_policy: BAD_POLICY,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }

    const fn policy_bad_shape() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            request_shape: BAD_SHAPE,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }

    const fn policy_bad_receipt() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            receipt_policy: BAD_POLICY,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }

    const fn policy_cancelled() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            cancellation_posture: CANCELLATION_CANCELLED,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }

    const fn policy_timed_out() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            timeout_posture: TIMEOUT_TIMED_OUT,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }

    const fn policy_fallback_requested() -> ModelDispatchPolicyDraft<'static> {
        ModelDispatchPolicyDraft {
            fallback_requested: FALLBACK_REQUESTED,
            ..ModelDispatchPolicyDraft::compatibility_defaults()
        }
    }
}
