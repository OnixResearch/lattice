/// Permission resolution request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolutionRequest<'a> {
    /// Guarded action under evaluation.
    pub action: super::super::GuardedAction,
    /// Safe resource label under evaluation.
    pub resource_label: &'a str,
    /// Authority status from Nickel/UCAN/sandbox checks.
    pub authority_status: super::super::AuthorizationStatus,
    /// Authority issue from Nickel/UCAN/sandbox checks.
    pub authority_issue: super::super::AuthorizationIssue,
    /// Selected local permission policy.
    pub policy: Option<&'a super::Policy>,
    /// Optional host prompt decision for ask-mode continuation.
    pub prompt_decision: Option<&'a super::Decision>,
    /// Session label used to scope prompt tokens.
    pub session_label: &'a str,
    /// Redacted action summary supplied to host prompts.
    pub redacted_summary: &'a str,
}

/// Pure permission resolution output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    status: DecisionStatus,
    reason: DecisionReason,
    local_outcome: Option<super::Outcome>,
    prompt_status: PromptStatus,
    prompt_request: Option<super::PromptRequest>,
}

impl Resolution {
    /// Returns final permission status.
    #[must_use]
    pub const fn status(&self) -> DecisionStatus {
        self.status
    }

    /// Returns final permission reason.
    #[must_use]
    pub const fn reason(&self) -> DecisionReason {
        self.reason
    }

    /// Returns selected local outcome, if any policy matched.
    #[must_use]
    pub const fn local_outcome(&self) -> Option<super::Outcome> {
        self.local_outcome
    }

    /// Returns prompt status.
    #[must_use]
    pub const fn prompt_status(&self) -> PromptStatus {
        self.prompt_status
    }

    /// Returns prompt request for ask-mode pauses.
    #[must_use]
    pub const fn prompt_request(&self) -> Option<&super::PromptRequest> {
        self.prompt_request.as_ref()
    }

    /// Returns true only when guarded dispatch may proceed.
    #[must_use]
    pub const fn dispatch_allowed(&self) -> bool {
        matches!(self.status, DecisionStatus::Execute)
    }
}

/// Final permission status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionStatus {
    /// Dispatch may proceed.
    Execute,
    /// Host prompt is required before dispatch.
    PromptRequired,
    /// Dispatch is denied.
    Denied,
}

/// Permission decision reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionReason {
    /// No issue.
    None,
    /// Authority failed before local allow could apply.
    MissingAuthority,
    /// No selected local policy matched the action/resource.
    NoMatchingPolicy,
    /// Local policy denied dispatch.
    LocalDeny,
    /// Ask mode requires a host prompt before dispatch.
    PromptRequired,
    /// Host prompt denied dispatch.
    PromptDenied,
    /// Prompt token was stale or mismatched.
    StalePromptToken,
}

/// Prompt status for receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptStatus {
    /// Local outcome did not require prompting.
    NotRequired,
    /// Prompt must be issued before dispatch.
    Required,
    /// Prompt approved dispatch.
    Approved,
    /// Prompt denied dispatch.
    Denied,
    /// Prompt token was stale or mismatched.
    Stale,
}

/// Resolves local permission mode after authority checks.
#[must_use]
pub fn resolve(request: ResolutionRequest<'_>) -> Resolution {
    assert!(request.resource_label.len() <= crate::MAX_ADAPTER_FIELD_CHARS, "resource label stays bounded");
    assert!(request.session_label.len() <= crate::MAX_ADAPTER_FIELD_CHARS, "session label stays bounded");
    if request.authority_status == super::super::AuthorizationStatus::Denied {
        return missing_authority(request.policy);
    }
    let Some(policy) = request.policy else {
        return no_policy(None);
    };
    if !policy.matches_request(request.action, request.resource_label) {
        return no_policy(Some(policy.outcome()));
    }
    resolve_policy(request, policy)
}

fn resolve_policy(request: ResolutionRequest<'_>, policy: &super::Policy) -> Resolution {
    match policy.outcome() {
        super::Outcome::Allow => parts(
            DecisionStatus::Execute,
            DecisionReason::None,
            Some(super::Outcome::Allow),
            PromptStatus::NotRequired,
            None,
        ),
        super::Outcome::Deny => parts(
            DecisionStatus::Denied,
            DecisionReason::LocalDeny,
            Some(super::Outcome::Deny),
            PromptStatus::NotRequired,
            None,
        ),
        super::Outcome::Ask => resolve_ask_policy(request, policy),
    }
}

fn resolve_ask_policy(request: ResolutionRequest<'_>, policy: &super::Policy) -> Resolution {
    assert_eq!(policy.outcome(), super::Outcome::Ask, "ask helper gets ask policy");
    assert!(!request.redacted_summary.is_empty(), "ask prompts require a redacted summary");
    let Ok(prompt_request) = super::PromptRequest::new(super::PromptRequestDraft {
        action: request.action,
        resource_label: request.resource_label,
        session_label: request.session_label,
        policy_hash: policy.policy_hash(),
        redacted_summary: request.redacted_summary,
    }) else {
        return stale_prompt();
    };
    continue_ask(request.prompt_decision, prompt_request)
}

fn continue_ask(decision: Option<&super::Decision>, prompt_request: super::PromptRequest) -> Resolution {
    let Some(decision) = decision else {
        return parts(
            DecisionStatus::PromptRequired,
            DecisionReason::PromptRequired,
            Some(super::Outcome::Ask),
            PromptStatus::Required,
            Some(prompt_request),
        );
    };
    if !prompt_matches(&prompt_request, decision) {
        return stale_prompt();
    }
    prompt_outcome(decision.outcome)
}

fn prompt_outcome(outcome: super::DecisionOutcome) -> Resolution {
    match outcome {
        super::DecisionOutcome::Approved => parts(
            DecisionStatus::Execute,
            DecisionReason::None,
            Some(super::Outcome::Ask),
            PromptStatus::Approved,
            None,
        ),
        super::DecisionOutcome::Denied => parts(
            DecisionStatus::Denied,
            DecisionReason::PromptDenied,
            Some(super::Outcome::Ask),
            PromptStatus::Denied,
            None,
        ),
    }
}

fn missing_authority(policy: Option<&super::Policy>) -> Resolution {
    parts(
        DecisionStatus::Denied,
        DecisionReason::MissingAuthority,
        policy.map(super::Policy::outcome),
        PromptStatus::NotRequired,
        None,
    )
}

fn no_policy(local_outcome: Option<super::Outcome>) -> Resolution {
    parts(
        DecisionStatus::Denied,
        DecisionReason::NoMatchingPolicy,
        local_outcome,
        PromptStatus::NotRequired,
        None,
    )
}

fn stale_prompt() -> Resolution {
    parts(
        DecisionStatus::Denied,
        DecisionReason::StalePromptToken,
        Some(super::Outcome::Ask),
        PromptStatus::Stale,
        None,
    )
}

#[expect(
    clippy::too_many_arguments,
    reason = "decision fixture helper preserves matrix readability"
)]
fn parts(
    status: DecisionStatus,
    reason: DecisionReason,
    local_outcome: Option<super::Outcome>,
    prompt_status: PromptStatus,
    prompt_request: Option<super::PromptRequest>,
) -> Resolution {
    Resolution {
        status,
        reason,
        local_outcome,
        prompt_status,
        prompt_request,
    }
}

fn prompt_matches(request: &super::PromptRequest, decision: &super::Decision) -> bool {
    request.action() == decision.action
        && request.resource_label() == decision.resource_label
        && request.session_label() == decision.session_label
        && request.policy_hash() == decision.hash
        && request.decision_token() == decision.token
}
