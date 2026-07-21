/// Borrowed receipt fields before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContinuationReceiptDraft<'a> {
    /// Safe class of the prior state.
    pub prior_state_class: &'a str,
    /// Safe selected action label.
    pub selected_action: &'a str,
    /// BLAKE3 hash of pending work ID, never the raw ID.
    pub pending_work_id_hash: &'a str,
    /// Safe decision reason.
    pub reason: &'a str,
    /// Optional safe rejection class.
    pub rejection_class: Option<&'a str>,
    /// BLAKE3 hash of safe policy metadata.
    pub policy_hash: &'a str,
}

// r[impl agent_harness_runtime.continuation_policy.receipts]
/// Safe continuation receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationReceipt {
    prior_state_class: ContinuationStateClass,
    selected_action: ContinuationAction,
    pending_work_id_hash: Text,
    reason: Text,
    rejection_class: Option<Text>,
    policy_hash: Text,
    receipt_hash: Text,
}

impl ContinuationReceipt {
    /// Validates safe continuation receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when a field is empty, unsupported, or
    /// contains raw prompt, model output, tool output, provider payload,
    /// transcript, credential, full path, or unbounded-log leak markers.
    pub fn validate(draft: &ContinuationReceiptDraft<'_>) -> Result<Self, ContinuationError> {
        let prior_state_class = ContinuationStateClass::parse(validate_safe_metadata(
            FIELD_PRIOR_STATE_CLASS,
            draft.prior_state_class,
        )?)?;
        let selected_action = ContinuationAction::parse(validate_safe_metadata(
            FIELD_SELECTED_ACTION,
            draft.selected_action,
        )?)?;
        let pending_work_id_hash =
            validate_hash_metadata(FIELD_PENDING_WORK_ID_HASH, draft.pending_work_id_hash)?;
        let reason = validate_safe_metadata(FIELD_REASON, draft.reason)?;
        let rejection_class = draft
            .rejection_class
            .map(|value| validate_safe_metadata(FIELD_REJECTION_CLASS, value).map(Text::from))
            .transpose()?;
        let policy_hash = validate_hash_metadata(FIELD_POLICY_HASH, draft.policy_hash)?;
        let receipt_hash = continuation_receipt_hash(ReceiptHashParts {
            prior_state_class,
            selected_action,
            pending_work_id_hash,
            reason,
            rejection_class: rejection_class.as_deref(),
            policy_hash,
        });
        Ok(Self {
            prior_state_class,
            selected_action,
            pending_work_id_hash: Text::from(pending_work_id_hash),
            reason: Text::from(reason),
            rejection_class,
            policy_hash: Text::from(policy_hash),
            receipt_hash,
        })
    }

    /// Builds a safe receipt from a prior state, decision, and policy.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when optional rejection metadata is unsafe.
    pub fn from_decision(
        prior_state_class: ContinuationStateClass,
        decision: &ContinuationDecision,
        policy: &ContinuationPolicy,
        rejection_class: Option<&str>,
    ) -> Result<Self, ContinuationError> {
        let pending_work_id_hash = decision.pending_work_id_hash();
        let policy_hash = policy.policy_hash();
        Self::validate(&ContinuationReceiptDraft {
            prior_state_class: prior_state_class.as_str(),
            selected_action: decision.action().as_str(),
            pending_work_id_hash: pending_work_id_hash.as_str(),
            reason: decision.reason().as_str(),
            rejection_class,
            policy_hash: policy_hash.as_str(),
        })
    }

    /// Returns the prior state class.
    #[must_use]
    pub fn prior_state_class(&self) -> ContinuationStateClass {
        self.prior_state_class
    }

    /// Returns the selected action.
    #[must_use]
    pub fn selected_action(&self) -> ContinuationAction {
        self.selected_action
    }

    /// Returns the pending work ID hash.
    #[must_use]
    pub fn pending_work_id_hash(&self) -> &str {
        self.pending_work_id_hash.as_str()
    }

    /// Returns the safe reason label.
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason.as_str()
    }

    /// Returns the optional rejection class.
    #[must_use]
    pub fn rejection_class(&self) -> Option<&str> {
        self.rejection_class.as_deref()
    }

    /// Returns the policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the deterministic BLAKE3 receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

