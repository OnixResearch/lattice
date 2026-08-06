pub(crate) struct ReceiptHashInput<'a> {
    pub(crate) operation_kind: &'a str,
    pub(crate) source: &'a str,
    pub(crate) active_work_hash: &'a str,
    pub(crate) queue_item_hash: &'a str,
    pub(crate) decision_class: &'a str,
    pub(crate) cancellation_handle_class: &'a str,
    pub(crate) budget_class: &'a str,
    pub(crate) policy_hash: &'a str,
    pub(crate) denial_class: Option<&'a str>,
    pub(crate) redaction_state: &'a str,
}

/// Borrowed safe queue receipt metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MessageQueueReceiptDraft<'a> {
    /// Operation kind label.
    pub operation_kind: &'a str,
    /// Actor or source label.
    pub source: &'a str,
    /// Hash of active work identity.
    pub active_work_hash: &'a str,
    /// Hash of queued item identity/body, or `none` when no item applies.
    pub queue_item_hash: &'a str,
    /// Decision class label.
    pub decision_class: &'a str,
    /// Cancellation handle class.
    pub cancellation_handle_class: &'a str,
    /// Budget class.
    pub budget_class: &'a str,
    /// Policy hash.
    pub policy_hash: &'a str,
    /// Optional denial class.
    pub denial_class: Option<&'a str>,
    /// Redaction state.
    pub redaction_state: &'a str,
}

/// Safe queue and interrupt control receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageQueueReceipt {
    operation_kind: super::MessageQueueOperationKind,
    source: String,
    active_work_hash: String,
    queue_item_hash: String,
    decision_class: String,
    cancellation_handle_class: super::CancellationHandleClass,
    budget_class: super::MessageBudgetClass,
    policy_hash: String,
    denial_class: Option<String>,
    redaction_state: super::MessageRedactionState,
    receipt_hash: String,
}

impl MessageQueueReceipt {
    // r[impl agent_harness_runtime.goose_message_queue.receipts]
    /// Validates safe queue receipt metadata and computes its deterministic hash.
    ///
    /// # Errors
    ///
    /// Returns [`super::MessageQueueError`] when metadata is empty, unsafe,
    /// malformed, or contains raw prompt/provider/tool/credential/path/output
    /// material.
    pub fn validate(draft: MessageQueueReceiptDraft<'_>) -> Result<Self, super::MessageQueueError> {
        validate_message_queue_receipt_text(draft.operation_kind)?;
        validate_message_queue_receipt_text(draft.source)?;
        validate_message_queue_receipt_text(draft.decision_class)?;
        validate_message_queue_receipt_text(draft.cancellation_handle_class)?;
        validate_message_queue_receipt_text(draft.budget_class)?;
        validate_message_queue_receipt_text(draft.redaction_state)?;
        if let Some(denial_class) = draft.denial_class {
            validate_message_queue_receipt_text(denial_class)?;
        }
        let operation_kind = super::MessageQueueOperationKind::parse(
            super::data::normalize_safe_metadata(super::FIELD_OPERATION_KIND, draft.operation_kind)?.as_str(),
        )?;
        let source = super::data::normalize_safe_metadata(super::FIELD_SOURCE, draft.source)?;
        let active_work_hash =
            super::data::validate_hash_metadata(super::FIELD_ACTIVE_WORK_HASH, draft.active_work_hash)?;
        let queue_item_hash = super::data::validate_hash_metadata(super::FIELD_ITEM_HASH, draft.queue_item_hash)?;
        let decision_class = super::data::normalize_safe_metadata(super::FIELD_DECISION_CLASS, draft.decision_class)?;
        let cancellation_handle_class = super::CancellationHandleClass::parse(
            super::data::normalize_safe_metadata(
                super::FIELD_CANCELLATION_HANDLE_CLASS,
                draft.cancellation_handle_class,
            )?
            .as_str(),
        )?;
        let budget_class = super::MessageBudgetClass::parse(
            super::data::normalize_safe_metadata(super::FIELD_BUDGET_CLASS, draft.budget_class)?.as_str(),
        )?;
        let policy_hash = super::data::validate_hash_metadata(super::FIELD_POLICY_HASH, draft.policy_hash)?;
        let denial_class = draft
            .denial_class
            .map(|value| super::data::normalize_safe_metadata(super::FIELD_DENIAL_CLASS, value))
            .transpose()?;
        let redaction_state = super::MessageRedactionState::parse(
            super::data::normalize_safe_metadata(super::FIELD_REDACTION_STATE, draft.redaction_state)?.as_str(),
        )?;
        let receipt_hash = super::digest::receipt_hash(&ReceiptHashInput {
            operation_kind: operation_kind.as_str(),
            source: source.as_str(),
            active_work_hash: active_work_hash.as_str(),
            queue_item_hash: queue_item_hash.as_str(),
            decision_class: decision_class.as_str(),
            cancellation_handle_class: cancellation_handle_class.as_str(),
            budget_class: budget_class.as_str(),
            policy_hash: policy_hash.as_str(),
            denial_class: denial_class.as_deref(),
            redaction_state: redaction_state.as_str(),
        });
        Ok(Self {
            operation_kind,
            source,
            active_work_hash,
            queue_item_hash,
            decision_class,
            cancellation_handle_class,
            budget_class,
            policy_hash,
            denial_class,
            redaction_state,
            receipt_hash,
        })
    }

    /// Returns the operation kind.
    #[must_use]
    pub const fn operation_kind(&self) -> super::MessageQueueOperationKind {
        self.operation_kind
    }

    /// Returns the actor or source label.
    #[must_use]
    pub fn source(&self) -> &str {
        self.source.as_str()
    }

    /// Returns the active-work hash.
    #[must_use]
    pub fn active_work_hash(&self) -> &str {
        self.active_work_hash.as_str()
    }

    /// Returns the queue-item hash.
    #[must_use]
    pub fn queue_item_hash(&self) -> &str {
        self.queue_item_hash.as_str()
    }

    /// Returns the decision class.
    #[must_use]
    pub fn decision_class(&self) -> &str {
        self.decision_class.as_str()
    }

    /// Returns the cancellation handle class.
    #[must_use]
    pub const fn cancellation_handle_class(&self) -> super::CancellationHandleClass {
        self.cancellation_handle_class
    }

    /// Returns the budget class.
    #[must_use]
    pub const fn budget_class(&self) -> super::MessageBudgetClass {
        self.budget_class
    }

    /// Returns the policy hash.
    #[must_use]
    pub fn policy_hash(&self) -> &str {
        self.policy_hash.as_str()
    }

    /// Returns the optional denial class.
    #[must_use]
    pub fn denial_class(&self) -> Option<&str> {
        self.denial_class.as_deref()
    }

    /// Returns the redaction state.
    #[must_use]
    pub const fn redaction_state(&self) -> super::MessageRedactionState {
        self.redaction_state
    }

    /// Returns the deterministic BLAKE3 receipt hash.
    #[must_use]
    pub fn receipt_hash(&self) -> &str {
        self.receipt_hash.as_str()
    }
}

/// Validates text intended for durable queue receipts.
///
/// # Errors
///
/// Returns [`super::MessageQueueError`] if text contains raw prompt, provider,
/// credential, path, tool output, command output, or unbounded queue body markers.
pub fn validate_message_queue_receipt_text(text: &str) -> Result<(), super::MessageQueueError> {
    if let Some(marker) = super::data::leak_marker(text) {
        return Err(super::MessageQueueError::new(
            super::MessageQueueErrorKind::ReceiptLeak,
            super::FIELD_RECEIPT_TEXT,
            format!("receipt text contains unsafe marker: {marker}"),
        ));
    }
    let _ = bounded_receipt_fragments(text)?;
    Ok(())
}

fn bounded_receipt_fragments(text: &str) -> Result<Vec<String>, super::MessageQueueError> {
    text.split_whitespace()
        .map(|fragment| super::data::normalize_safe_metadata(super::FIELD_RECEIPT_TEXT, fragment))
        .collect()
}
