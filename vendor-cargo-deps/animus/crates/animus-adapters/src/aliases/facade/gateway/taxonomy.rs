/// Remote or headless entrypoint requesting admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Entrypoint {
    /// Headless RPC entrypoint.
    HeadlessRpc,
    /// MCP bridge entrypoint.
    Mcp,
    /// Remote session entrypoint.
    RemoteSession,
    /// Automation or batch entrypoint.
    Automation,
    /// Unsupported entrypoint marker that must deny before effects.
    Unsupported,
}

impl Entrypoint {
    /// Returns a stable entrypoint label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::HeadlessRpc => "headless_rpc",
            Self::Mcp => "mcp",
            Self::RemoteSession => "remote_session",
            Self::Automation => "automation",
            Self::Unsupported => "unsupported",
        }
    }
}

/// Entry action requiring admission before side effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// Create a session.
    SessionCreate,
    /// Attach to or resume a session.
    AttachResume,
    /// Submit a prompt.
    PromptSubmit,
    /// Execute a tool.
    ToolExecute,
    /// Mutate memory.
    MemoryMutate,
    /// Deliver an artifact.
    ArtifactDeliver,
    /// Run a workflow command.
    WorkflowCommand,
}

impl Action {
    /// Returns a stable action label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionCreate => "session_create",
            Self::AttachResume => "attach_resume",
            Self::PromptSubmit => "prompt_submit",
            Self::ToolExecute => "tool_execute",
            Self::MemoryMutate => "memory_mutate",
            Self::ArtifactDeliver => "artifact_deliver",
            Self::WorkflowCommand => "workflow_command",
        }
    }
}

/// Local caveat result for the admission request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionCaveat {
    /// Local policy allows the request to continue if credential authority also passes.
    Allow,
    /// Local policy denies the request.
    Deny,
    /// Local policy requires host confirmation before the request may continue.
    PromptRequired,
}

impl AdmissionCaveat {
    /// Returns a stable caveat label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
            Self::PromptRequired => "prompt_required",
        }
    }
}

/// Admission verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionVerdict {
    /// Request is admitted for host dispatch.
    Allow,
    /// Request is denied before side effects.
    Deny,
}

impl AdmissionVerdict {
    /// Returns a stable verdict label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

/// Admission denial reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionReason {
    /// The entrypoint is unsupported by reusable admission logic.
    UnsupportedEntrypoint,
    /// Required policy hash was not supplied.
    MissingPolicy,
    /// Required credential evidence was absent.
    MissingCredential,
    /// Credential evidence targets a different audience.
    WrongAudience,
    /// Credential evidence is expired.
    ExpiredCredential,
    /// Credential evidence is revoked.
    RevokedCredential,
    /// Credential verifier is unavailable.
    VerifierUnavailable,
    /// Credential evidence does not cover the requested resource and ability.
    MissingGrant,
    /// Local caveat denies or requires prompt before dispatch.
    CaveatDenied,
}

impl AdmissionReason {
    /// Returns a stable reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedEntrypoint => "unsupported_entrypoint",
            Self::MissingPolicy => "missing_policy",
            Self::MissingCredential => "missing_credential",
            Self::WrongAudience => "wrong_audience",
            Self::ExpiredCredential => "expired_credential",
            Self::RevokedCredential => "revoked_credential",
            Self::VerifierUnavailable => "verifier_unavailable",
            Self::MissingGrant => "missing_grant",
            Self::CaveatDenied => "caveat_denied",
        }
    }
}
