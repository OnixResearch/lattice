/// Exec-policy decision for an escalation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Decision {
    /// Permit the command outside the active sandbox.
    Allow,
    /// Require an explicit host/user prompt before dispatch.
    Prompt,
    /// Deny command dispatch.
    Forbid,
}

impl Decision {
    /// Returns the stable decision label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => super::super::ALLOW_LABEL,
            Self::Prompt => super::super::PROMPT_LABEL,
            Self::Forbid => super::super::FORBID_LABEL,
        }
    }

    /// Returns true when dispatch can proceed without prompting.
    #[must_use]
    pub const fn dispatch_allowed(self) -> bool {
        matches!(self, Self::Allow)
    }

    pub(super) const fn rank(self) -> u8 {
        match self {
            Self::Allow => super::super::DECISION_RANK_ALLOW,
            Self::Prompt => super::super::DECISION_RANK_PROMPT,
            Self::Forbid => super::super::DECISION_RANK_FORBID,
        }
    }

    pub(in crate::effect::exec_policy) const fn most_restrictive(self, other: Self) -> Self {
        if self.rank() >= other.rank() {
            self
        } else {
            other
        }
    }
}
