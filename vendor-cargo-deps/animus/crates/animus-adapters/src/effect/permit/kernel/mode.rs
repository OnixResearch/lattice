/// UX-facing permission mode used only as an advisory local default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionMode {
    /// Deny every guarded action unless a narrower explicit rule asks later.
    Restrictive,
    /// Allow read-like model/tool/resource inspection and ask for broader work.
    ReadOnly,
    /// Ask for mutating or effectful work after authority already passes.
    Guarded,
    /// Allow routine work after authority already passes while asking for high-risk effects.
    Standard,
    /// Locally allow after authority already passes; never bypasses authority or sandbox checks.
    Yolo,
}

impl PermissionMode {
    /// Returns the stable mode label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Restrictive => RESTRICTIVE_MODE_LABEL,
            Self::ReadOnly => READ_ONLY_MODE_LABEL,
            Self::Guarded => GUARDED_MODE_LABEL,
            Self::Standard => STANDARD_MODE_LABEL,
            Self::Yolo => YOLO_MODE_LABEL,
        }
    }
}

/// Borrowed input for mode-default evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModeDefaultRequest {
    /// Active UX permission mode.
    pub mode: PermissionMode,
    /// Guarded action class.
    pub action: super::super::GuardedAction,
    /// Authority status supplied by the stronger authority layer.
    pub authority_status: super::super::AuthorizationStatus,
    /// Whether the sandbox permits the action.
    pub sandbox_allows: bool,
    /// Whether redaction checks permit the action metadata to be recorded.
    pub redaction_allows: bool,
}

/// Pure mode-default decision before pattern, prompt, and allowlist layers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModeDefaultResolution {
    status: super::DecisionStatus,
    reason: ModeDefaultReason,
    local_outcome: super::Outcome,
}

impl ModeDefaultResolution {
    /// Returns final status from the mode-default layer.
    #[must_use]
    pub const fn status(self) -> super::DecisionStatus {
        self.status
    }

    /// Returns mode-default reason.
    #[must_use]
    pub const fn reason(self) -> ModeDefaultReason {
        self.reason
    }

    /// Returns advisory local outcome selected by the mode.
    #[must_use]
    pub const fn local_outcome(self) -> super::Outcome {
        self.local_outcome
    }

    /// Returns true only when this layer permits dispatch to continue.
    #[must_use]
    pub const fn dispatch_allowed(self) -> bool {
        matches!(self.status, super::DecisionStatus::Execute)
    }
}

/// Reason selected by mode-default evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModeDefaultReason {
    /// Mode locally allowed and stronger gates already allowed.
    ModeAllowed,
    /// Mode requires a prompt.
    ModeRequiresPrompt,
    /// Mode locally denied.
    ModeDenied,
    /// Stronger authority denied before local allow could apply.
    MissingAuthority,
    /// Sandbox denied before local allow could apply.
    SandboxDenied,
    /// Redaction policy denied before local allow could apply.
    RedactionDenied,
}

/// Resolves advisory permission-mode defaults below stronger authority gates.
#[must_use]
// r[impl agent_harness_runtime.zerostack_permission_kernel.modes]
// r[impl agent_harness_runtime.zerostack_permission_kernel.modes.authority]
pub fn resolve_mode_default(request: ModeDefaultRequest) -> ModeDefaultResolution {
    assert!(!request.mode.as_str().is_empty(), "mode label is stable");
    assert!(
        !request.action.as_str().is_empty(),
        "action label is stable"
    );
    if request.authority_status == super::super::AuthorizationStatus::Denied {
        return mode_resolution(
            super::DecisionStatus::Denied,
            ModeDefaultReason::MissingAuthority,
            mode_outcome(request.mode, request.action),
        );
    }
    if !request.sandbox_allows {
        return mode_resolution(
            super::DecisionStatus::Denied,
            ModeDefaultReason::SandboxDenied,
            mode_outcome(request.mode, request.action),
        );
    }
    if !request.redaction_allows {
        return mode_resolution(
            super::DecisionStatus::Denied,
            ModeDefaultReason::RedactionDenied,
            mode_outcome(request.mode, request.action),
        );
    }
    let outcome = mode_outcome(request.mode, request.action);
    match outcome {
        super::Outcome::Allow => mode_resolution(
            super::DecisionStatus::Execute,
            ModeDefaultReason::ModeAllowed,
            outcome,
        ),
        super::Outcome::Ask => mode_resolution(
            super::DecisionStatus::PromptRequired,
            ModeDefaultReason::ModeRequiresPrompt,
            outcome,
        ),
        super::Outcome::Deny => mode_resolution(
            super::DecisionStatus::Denied,
            ModeDefaultReason::ModeDenied,
            outcome,
        ),
    }
}

fn mode_resolution(
    status: super::DecisionStatus,
    reason: ModeDefaultReason,
    local_outcome: super::Outcome,
) -> ModeDefaultResolution {
    ModeDefaultResolution {
        status,
        reason,
        local_outcome,
    }
}

fn mode_outcome(mode: PermissionMode, action: super::super::GuardedAction) -> super::Outcome {
    match mode {
        PermissionMode::Restrictive => super::Outcome::Deny,
        PermissionMode::ReadOnly => read_only_outcome(action),
        PermissionMode::Guarded => guarded_outcome(action),
        PermissionMode::Standard => standard_outcome(action),
        PermissionMode::Yolo => super::Outcome::Allow,
    }
}

fn read_only_outcome(action: super::super::GuardedAction) -> super::Outcome {
    match action {
        super::super::GuardedAction::Model | super::super::GuardedAction::Tool => {
            super::Outcome::Allow
        }
        super::super::GuardedAction::Filesystem
        | super::super::GuardedAction::Exec
        | super::super::GuardedAction::Network
        | super::super::GuardedAction::Steel
        | super::super::GuardedAction::AgentRole => super::Outcome::Ask,
    }
}

fn guarded_outcome(action: super::super::GuardedAction) -> super::Outcome {
    match action {
        super::super::GuardedAction::Model => super::Outcome::Allow,
        super::super::GuardedAction::Tool
        | super::super::GuardedAction::Filesystem
        | super::super::GuardedAction::Exec
        | super::super::GuardedAction::Network
        | super::super::GuardedAction::Steel
        | super::super::GuardedAction::AgentRole => super::Outcome::Ask,
    }
}

fn standard_outcome(action: super::super::GuardedAction) -> super::Outcome {
    match action {
        super::super::GuardedAction::Model
        | super::super::GuardedAction::Tool
        | super::super::GuardedAction::Filesystem
        | super::super::GuardedAction::Steel => super::Outcome::Allow,
        super::super::GuardedAction::Exec
        | super::super::GuardedAction::Network
        | super::super::GuardedAction::AgentRole => super::Outcome::Ask,
    }
}
