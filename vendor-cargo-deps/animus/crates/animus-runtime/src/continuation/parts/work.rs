/// Priority policy for continuation candidates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContinuationPriority {
    order: [ContinuationCandidate; PRIORITY_CANDIDATE_COUNT],
}

impl ContinuationPriority {
    /// Builds an explicit continuation priority order.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when a candidate appears more than once.
    pub fn new(
        order: [ContinuationCandidate; PRIORITY_CANDIDATE_COUNT],
    ) -> Result<Self, ContinuationError> {
        reject_duplicate_candidates(order)?;
        Ok(Self { order })
    }

    /// Returns candidates in priority order.
    #[must_use]
    pub fn order(&self) -> &[ContinuationCandidate; PRIORITY_CANDIDATE_COUNT] {
        &self.order
    }
}

impl Default for ContinuationPriority {
    fn default() -> Self {
        Self {
            order: [
                ContinuationCandidate::QueuedPrompt,
                ContinuationCandidate::LoopContinuation,
                ContinuationCandidate::ValidationFollowUp,
            ],
        }
    }
}

/// Deterministic continuation policy configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationPolicy {
    label: Text,
    priority: ContinuationPriority,
}

impl ContinuationPolicy {
    /// Builds a continuation policy with an explicit priority order.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when `label` is unsafe or `priority`
    /// contains duplicates.
    pub fn new(label: &str, priority: ContinuationPriority) -> Result<Self, ContinuationError> {
        Ok(Self {
            label: validate_safe_label(FIELD_POLICY_LABEL, label)?,
            priority,
        })
    }

    /// Returns the policy label.
    #[must_use]
    pub fn label(&self) -> &str {
        self.label.as_str()
    }

    /// Returns the priority order.
    #[must_use]
    pub fn priority(&self) -> ContinuationPriority {
        self.priority
    }

    /// Returns a BLAKE3 hash over safe policy metadata.
    #[must_use]
    pub fn policy_hash(&self) -> Text {
        let mut normalized = Text::from(CONTINUATION_POLICY_SCHEMA);
        push_hash_part(&mut normalized, self.label.as_str());
        for candidate in self.priority.order() {
            push_hash_part(&mut normalized, candidate.as_str());
        }
        blake3_hex(normalized.as_bytes())
    }
}

impl Default for ContinuationPolicy {
    fn default() -> Self {
        Self {
            label: Text::from(DEFAULT_POLICY_LABEL),
            priority: ContinuationPriority::default(),
        }
    }
}

/// Pending dispatch selected by the pure policy and awaiting shell acknowledgement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingContinuationDispatch {
    work_id: Text,
    action: ContinuationAction,
}

impl PendingContinuationDispatch {
    /// Returns the pending work identifier.
    #[must_use]
    pub fn work_id(&self) -> &str {
        self.work_id.as_str()
    }

    /// Returns the selected action.
    #[must_use]
    pub fn action(&self) -> ContinuationAction {
        self.action
    }
}

/// Last accepted completion acknowledgement stored as safe state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContinuationCompletionRecord {
    work_id_hash: Text,
    action: ContinuationAction,
}

impl ContinuationCompletionRecord {
    /// Returns the completed work identifier hash.
    #[must_use]
    pub fn work_id_hash(&self) -> &str {
        self.work_id_hash.as_str()
    }

    /// Returns the completed action.
    #[must_use]
    pub fn action(&self) -> ContinuationAction {
        self.action
    }
}
