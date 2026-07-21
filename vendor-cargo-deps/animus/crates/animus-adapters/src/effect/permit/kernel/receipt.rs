/// Borrowed permission-kernel receipt fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelReceiptDraft<'a> {
    /// Guarded action class.
    pub action: super::super::GuardedAction,
    /// Safe resource hash.
    pub resource_hash: &'a crate::SafeHash,
    /// Authority status supplied by stronger authority checks.
    pub authority_status: super::super::AuthorizationStatus,
    /// Selected mode outcome.
    pub mode_outcome: super::Outcome,
    /// Matched rule identity, if any.
    pub matched_rule_hash: Option<&'a crate::SafeHash>,
    /// Repeated-action status.
    pub repeated_action_status: RepeatedActionStatus,
    /// Prompt status.
    pub prompt_status: super::PromptStatus,
    /// Final decision status.
    pub final_status: super::DecisionStatus,
    /// Redaction markers proving unsafe request material was omitted.
    pub redaction_markers: &'a [&'a str],
}

/// Bounded permission-kernel receipt with safe metadata only.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KernelReceipt {
    action: super::super::GuardedAction,
    resource_hash: crate::SafeHash,
    authority_status: super::super::AuthorizationStatus,
    mode_outcome: super::Outcome,
    matched_rule_hash: Option<crate::SafeHash>,
    repeated_action_status: RepeatedActionStatus,
    prompt_status: super::PromptStatus,
    final_status: super::DecisionStatus,
    redaction_markers: super::Collection<super::Text>,
}

impl KernelReceipt {
    /// Builds a safe permission-kernel receipt.
    ///
    /// # Errors
    ///
    /// Returns [`super::Error`] when redaction labels are malformed or exceed
    /// collection bounds.
    // r[impl agent_harness_runtime.zerostack_permission_kernel.receipts]
    pub fn new(draft: KernelReceiptDraft<'_>) -> Result<Self, super::Error> {
        super::evidence::validate_redaction_count(draft.redaction_markers.len())?;
        Ok(Self {
            action: draft.action,
            resource_hash: draft.resource_hash.clone(),
            authority_status: draft.authority_status,
            mode_outcome: draft.mode_outcome,
            matched_rule_hash: draft.matched_rule_hash.cloned(),
            repeated_action_status: draft.repeated_action_status,
            prompt_status: draft.prompt_status,
            final_status: draft.final_status,
            redaction_markers: super::evidence::redactions(draft.redaction_markers)?,
        })
    }

    /// Returns action class.
    #[must_use]
    pub const fn action(&self) -> super::super::GuardedAction {
        self.action
    }

    /// Returns resource hash.
    #[must_use]
    pub const fn resource_hash(&self) -> &crate::SafeHash {
        &self.resource_hash
    }

    /// Returns authority status.
    #[must_use]
    pub const fn authority_status(&self) -> super::super::AuthorizationStatus {
        self.authority_status
    }

    /// Returns mode outcome.
    #[must_use]
    pub const fn mode_outcome(&self) -> super::Outcome {
        self.mode_outcome
    }

    /// Returns matched rule hash, if any.
    #[must_use]
    pub const fn matched_rule_hash(&self) -> Option<&crate::SafeHash> {
        self.matched_rule_hash.as_ref()
    }

    /// Returns repeated-action status.
    #[must_use]
    pub const fn repeated_action_status(&self) -> RepeatedActionStatus {
        self.repeated_action_status
    }

    /// Returns prompt status.
    #[must_use]
    pub const fn prompt_status(&self) -> super::PromptStatus {
        self.prompt_status
    }

    /// Returns final decision status.
    #[must_use]
    pub const fn final_status(&self) -> super::DecisionStatus {
        self.final_status
    }

    /// Returns sorted redaction markers.
    #[must_use]
    pub fn redaction_markers(&self) -> &[super::Text] {
        self.redaction_markers.as_slice()
    }
}
