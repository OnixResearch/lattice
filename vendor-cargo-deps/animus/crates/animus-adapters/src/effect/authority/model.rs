const COUNT_LIMIT: u32 = u32::MAX;

/// Pure input for the authorization issue decision boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Input {
    /// Whether the reviewed policy contract permits the request shape.
    pub(crate) contract_permits: bool,
    /// Number of verified grants matching the request.
    pub(crate) matching_grants: u32,
    /// Number of local policy caveats that deny the request.
    pub(crate) local_denies: u32,
    /// Required sandbox state after descriptor validation.
    pub(crate) sandbox: Sandbox,
}

/// Pure sandbox state for authorization issue selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Sandbox {
    /// No sandbox constraint applies.
    NotRequired,
    /// Required sandbox is enforceable.
    Enforceable,
    /// Required sandbox descriptor is malformed.
    Malformed,
    /// Required sandbox cannot be enforced by the selected adapter.
    Unavailable,
}

impl Input {
    /// Builds model input from the production authorization context without I/O.
    #[must_use]
    pub(crate) fn from_context(context: &super::AuthorizationContext<'_>) -> Self {
        Self {
            contract_permits: context.contract.permits(context.request),
            matching_grants: count(
                context
                    .grants
                    .iter()
                    .filter(|grant| grant.authorizes(context.request))
                    .count(),
            ),
            local_denies: count(
                context
                    .caveats
                    .iter()
                    .filter(|caveat| matches!(caveat.decision(), super::LocalCaveatDecision::Deny))
                    .count(),
            ),
            sandbox: Sandbox::from_constraint(context.sandbox),
        }
    }
}

impl Sandbox {
    fn from_constraint(sandbox: &super::SandboxConstraint) -> Self {
        match sandbox {
            super::SandboxConstraint::NotRequired => Self::NotRequired,
            super::SandboxConstraint::Required {
                state: super::SandboxState::Enforceable,
                ..
            } => Self::Enforceable,
            super::SandboxConstraint::Required {
                state: super::SandboxState::Malformed,
                ..
            } => Self::Malformed,
            super::SandboxConstraint::Required {
                state: super::SandboxState::Unavailable,
                ..
            } => Self::Unavailable,
        }
    }
}

/// Selects the expected authorization issue for pure model inputs.
#[must_use]
pub(crate) const fn issue(input: Input) -> super::AuthorizationIssue {
    if !input.contract_permits {
        return super::AuthorizationIssue::ContractMismatch;
    }
    if input.local_denies > 0 {
        return super::AuthorizationIssue::LocalPolicyDenied;
    }
    if input.matching_grants == 0 {
        return super::AuthorizationIssue::MissingUcanGrant;
    }
    match input.sandbox {
        Sandbox::NotRequired | Sandbox::Enforceable => super::AuthorizationIssue::None,
        Sandbox::Malformed => super::AuthorizationIssue::SandboxMalformed,
        Sandbox::Unavailable => super::AuthorizationIssue::SandboxUnavailable,
    }
}

fn count(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(COUNT_LIMIT)
}
