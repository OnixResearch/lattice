/// Errors raised while constructing or evaluating security review DTOs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// A shared adapter field contract failed.
    Contract(crate::AdapterContractError),
    /// A bounded collection exceeded its safe item ceiling.
    CollectionTooLarge {
        /// Boundary collection field name.
        field: &'static str,
        /// Maximum accepted item count.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// A supposedly safe label or receipt contained unsafe raw material.
    UnsafeText {
        /// Boundary field name.
        field: &'static str,
        /// Unsafe material class detected in the field.
        leak: super::Leak,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::CollectionTooLarge {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} must contain at most {max_items} items; got {actual_items}"
            ),
            Self::UnsafeText { field, leak } => {
                write!(
                    formatter,
                    "{field} contains unsafe {} material",
                    leak.as_str()
                )
            }
        }
    }
}

impl From<crate::AdapterContractError> for Error {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

/// Security review action class used to select scanner and fail behavior.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionClass {
    /// Read-only action with no host mutation or external transfer.
    ReadOnly,
    /// Model-context transformation without direct host effects.
    ModelContext,
    /// Filesystem or workspace mutation.
    FilesystemMutation,
    /// Network transfer or upload.
    NetworkTransfer,
    /// Process execution or command launch.
    ProcessExecution,
    /// Privileged operation such as escalation to root/admin capabilities.
    PrivilegedOperation,
    /// Workspace mutation that may cross declared scope boundaries.
    WorkspaceMutation,
}

impl ActionClass {
    /// Returns the stable action label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::ModelContext => "model_context",
            Self::FilesystemMutation => "filesystem_mutation",
            Self::NetworkTransfer => "network_transfer",
            Self::ProcessExecution => "process_execution",
            Self::PrivilegedOperation => "privileged_operation",
            Self::WorkspaceMutation => "workspace_mutation",
        }
    }

    /// Returns the policy risk class for this action.
    #[must_use]
    pub const fn risk_class(self) -> RiskClass {
        match self {
            Self::ReadOnly | Self::ModelContext => RiskClass::Low,
            Self::FilesystemMutation
            | Self::NetworkTransfer
            | Self::ProcessExecution
            | Self::PrivilegedOperation
            | Self::WorkspaceMutation => RiskClass::High,
        }
    }
}

/// Coarse risk class used by explicit fail behavior policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RiskClass {
    /// Low-risk review may degrade to deterministic local behavior.
    Low,
    /// High-risk review should fail closed or require explicit confirmation.
    High,
}

impl RiskClass {
    /// Returns the stable risk label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::High => "high",
        }
    }
}

/// Advisory classifier or adversary-review outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdvisoryOutcome {
    /// Reviewer recommends allowing, subject to existing authority.
    Allow,
    /// Reviewer recommends warning, subject to existing authority.
    Warn,
    /// Reviewer recommends blocking dispatch.
    Block,
    /// Reviewer service was unavailable.
    Unavailable,
    /// Reviewer timed out.
    Timeout,
    /// Reviewer output was malformed.
    Malformed,
}

impl AdvisoryOutcome {
    /// Returns the stable outcome label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Warn => "warn",
            Self::Block => "block",
            Self::Unavailable => "unavailable",
            Self::Timeout => "timeout",
            Self::Malformed => "malformed",
        }
    }

    pub(super) const fn blocks_dispatch(self) -> bool {
        matches!(self, Self::Block)
    }

    pub(super) const fn is_outage(self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout | Self::Malformed)
    }
}

/// Borrowed advisory review before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdvisoryReviewDraft<'a> {
    /// Safe reviewer label, such as classifier or adversary-reviewer.
    pub reviewer_label: Option<&'a str>,
    /// Advisory outcome.
    pub outcome: AdvisoryOutcome,
    /// Safe explanation label without raw prompt, provider, or tool content.
    pub explanation_label: Option<&'a str>,
    /// Optional raw review evidence to hash and discard.
    pub raw_evidence: &'a [&'a str],
}

/// Validated advisory review that cannot grant authority by itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryReview {
    reviewer_label: super::Text,
    outcome: AdvisoryOutcome,
    explanation_label: super::Text,
    evidence_hashes: super::Collection<super::Text>,
}

impl AdvisoryReview {
    /// Validates an advisory review without giving it authority.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when safe labels are missing, unsafe material
    /// appears, collections exceed bounds, or raw evidence cannot be hashed
    /// within adapter limits.
    pub fn validate(draft: &AdvisoryReviewDraft<'_>) -> Result<Self, super::Error> {
        let reviewer_label =
            super::input::required_safe_text(super::REVIEWER_LABEL_FIELD, draft.reviewer_label)?;
        let explanation_label = super::input::required_safe_text(
            super::REVIEW_EXPLANATION_FIELD,
            draft.explanation_label,
        )?;
        let evidence_hashes = super::input::optional_raw_evidence_hashes(
            super::RAW_EVIDENCE_FIELD,
            draft.raw_evidence,
            super::MAX_ADVISORY_EVIDENCE_ITEMS,
        )?;
        Ok(Self {
            reviewer_label,
            outcome: draft.outcome,
            explanation_label,
            evidence_hashes,
        })
    }

    /// Returns the safe reviewer label.
    #[must_use]
    pub fn reviewer_label(&self) -> &str {
        self.reviewer_label.as_str()
    }

    /// Returns the advisory outcome.
    #[must_use]
    pub const fn outcome(&self) -> AdvisoryOutcome {
        self.outcome
    }

    /// Returns the safe explanation label.
    #[must_use]
    pub fn explanation_label(&self) -> &str {
        self.explanation_label.as_str()
    }

    /// Returns optional redacted evidence identities.
    #[must_use]
    pub fn evidence_hashes(&self) -> &[super::Text] {
        self.evidence_hashes.as_slice()
    }

    pub(super) const fn blocks_dispatch(&self) -> bool {
        self.outcome.blocks_dispatch()
    }

    pub(super) const fn is_outage(&self) -> bool {
        self.outcome.is_outage()
    }
}
