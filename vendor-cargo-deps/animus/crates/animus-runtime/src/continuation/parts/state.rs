// r[impl agent_harness_runtime.continuation_policy.state]
/// Pure continuation state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContinuationState {
    queued_prompt: Option<PendingContinuationWork>,
    active_loop: Option<PendingContinuationWork>,
    validation_follow_up: Option<PendingContinuationWork>,
    busy_prompt: Option<PendingContinuationWork>,
    cancellation: Option<PendingContinuationWork>,
    pending_dispatch: Option<PendingContinuationDispatch>,
    completion_acknowledgement: Option<ContinuationCompletionRecord>,
    completed_work_hashes: Collection<Text>,
}

impl ContinuationState {
    /// Returns empty continuation state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds queued prompt work.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when `work_id` is unsafe.
    pub fn with_queued_prompt(mut self, work_id: &str) -> Result<Self, ContinuationError> {
        self.queued_prompt = Some(PendingContinuationWork::new(
            work_id,
            ContinuationWorkKind::QueuedPrompt,
        )?);
        Ok(self)
    }

    /// Adds active loop work.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when `work_id` is unsafe.
    pub fn with_active_loop(mut self, work_id: &str) -> Result<Self, ContinuationError> {
        self.active_loop = Some(PendingContinuationWork::new(
            work_id,
            ContinuationWorkKind::LoopContinuation,
        )?);
        Ok(self)
    }

    /// Adds validation or auto-test follow-up work.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when `work_id` is unsafe.
    pub fn with_validation_follow_up(mut self, work_id: &str) -> Result<Self, ContinuationError> {
        self.validation_follow_up = Some(PendingContinuationWork::new(
            work_id,
            ContinuationWorkKind::ValidationFollowUp,
        )?);
        Ok(self)
    }

    /// Adds busy prompt work that must be completed by exact ID.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when `work_id` is unsafe.
    pub fn with_busy_prompt(mut self, work_id: &str) -> Result<Self, ContinuationError> {
        self.busy_prompt = Some(PendingContinuationWork::new(
            work_id,
            ContinuationWorkKind::BusyPrompt,
        )?);
        Ok(self)
    }

    /// Adds a cancellation request.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when `work_id` is unsafe.
    pub fn with_cancellation(mut self, work_id: &str) -> Result<Self, ContinuationError> {
        self.cancellation = Some(PendingContinuationWork::new(
            work_id,
            ContinuationWorkKind::Cancellation,
        )?);
        Ok(self)
    }

    /// Returns queued prompt work, when present.
    #[must_use]
    pub fn queued_prompt(&self) -> Option<&PendingContinuationWork> {
        self.queued_prompt.as_ref()
    }

    /// Returns active loop work, when present.
    #[must_use]
    pub fn active_loop(&self) -> Option<&PendingContinuationWork> {
        self.active_loop.as_ref()
    }

    /// Returns validation follow-up work, when present.
    #[must_use]
    pub fn validation_follow_up(&self) -> Option<&PendingContinuationWork> {
        self.validation_follow_up.as_ref()
    }

    /// Returns busy prompt work, when present.
    #[must_use]
    pub fn busy_prompt(&self) -> Option<&PendingContinuationWork> {
        self.busy_prompt.as_ref()
    }

    /// Returns cancellation work, when present.
    #[must_use]
    pub fn cancellation(&self) -> Option<&PendingContinuationWork> {
        self.cancellation.as_ref()
    }

    /// Returns pending dispatch state, when present.
    #[must_use]
    pub fn pending_dispatch(&self) -> Option<&PendingContinuationDispatch> {
        self.pending_dispatch.as_ref()
    }

    /// Returns the last completion acknowledgement, when present.
    #[must_use]
    pub fn completion_acknowledgement(&self) -> Option<&ContinuationCompletionRecord> {
        self.completion_acknowledgement.as_ref()
    }

    /// Returns safe hashes of completed work IDs.
    #[must_use]
    pub fn completed_work_hashes(&self) -> &[Text] {
        self.completed_work_hashes.as_slice()
    }

    /// Returns the safe state class for receipts.
    #[must_use]
    pub fn state_class(&self) -> ContinuationStateClass {
        if self.cancellation.is_some() {
            return ContinuationStateClass::Cancelled;
        }
        if self.pending_dispatch.is_some() {
            return ContinuationStateClass::PendingDispatch;
        }
        if self.busy_prompt.is_some() {
            return ContinuationStateClass::Busy;
        }
        ContinuationStateClass::Idle
    }

    /// Stages a dispatch decision for exact shell acknowledgement.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when the decision is not dispatchable or
    /// no current candidate matches the selected work ID.
    pub fn begin_dispatch(
        mut self,
        decision: &ContinuationDecision,
    ) -> Result<Self, ContinuationError> {
        let ContinuationDecision::Dispatch(dispatch) = decision else {
            return Err(ContinuationError::UnsupportedValue {
                field: FIELD_SELECTED_ACTION,
                value: Text::from(decision.action().as_str()),
            });
        };
        if !self.has_candidate(dispatch.work_id()) {
            return Err(ContinuationError::StaleWork {
                work_id_hash: blake3_hex(dispatch.work_id().as_bytes()),
            });
        }
        self.pending_dispatch = Some(PendingContinuationDispatch {
            work_id: Text::from(dispatch.work_id()),
            action: dispatch.action(),
        });
        Ok(self)
    }

    // r[impl agent_harness_runtime.continuation_policy.acknowledgements]
    /// Applies an exact dispatch acknowledgement.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when the acknowledgement is missing,
    /// stale, duplicate, mismatched, or no dispatch is pending.
    pub fn acknowledge_dispatch(
        mut self,
        acknowledgement: &DispatchAcknowledgement<'_>,
    ) -> Result<Self, ContinuationError> {
        let acknowledged_id = validate_ack_work_id(acknowledgement.work_id)?;
        let pending = self
            .pending_dispatch
            .clone()
            .ok_or(ContinuationError::MissingPendingDispatch)?;
        if pending.work_id != acknowledged_id {
            return Err(mismatched_work_error(
                pending.work_id.as_str(),
                acknowledged_id.as_str(),
            ));
        }
        self.pending_dispatch = None;
        match acknowledgement.outcome {
            DispatchAcknowledgementOutcome::Accepted => {
                self.remove_candidate(pending.work_id.as_str());
                self.busy_prompt = Some(PendingContinuationWork::new(
                    pending.work_id.as_str(),
                    ContinuationWorkKind::BusyPrompt,
                )?);
                Ok(self)
            }
            DispatchAcknowledgementOutcome::Rejected => Ok(self),
        }
    }

    // r[impl agent_harness_runtime.continuation_policy.acknowledgements]
    /// Applies an exact completion acknowledgement.
    ///
    /// # Errors
    ///
    /// Returns [`ContinuationError`] when the acknowledgement is missing,
    /// stale, duplicate, mismatched, or no busy work is present.
    pub fn acknowledge_completion(
        mut self,
        acknowledgement: &CompletionAcknowledgement<'_>,
    ) -> Result<Self, ContinuationError> {
        let acknowledged_id = validate_ack_work_id(acknowledgement.work_id)?;
        let acknowledged_hash = blake3_hex(acknowledged_id.as_bytes());
        if self
            .completed_work_hashes
            .iter()
            .any(|hash| hash == &acknowledged_hash)
        {
            return Err(ContinuationError::DuplicateCompletion {
                work_id_hash: acknowledged_hash,
            });
        }
        let busy = self
            .busy_prompt
            .clone()
            .ok_or(ContinuationError::MissingBusyWork)?;
        if busy.work_id != acknowledged_id {
            return Err(mismatched_work_error(
                busy.work_id.as_str(),
                acknowledged_id.as_str(),
            ));
        }
        self.busy_prompt = None;
        self.completion_acknowledgement = Some(ContinuationCompletionRecord {
            work_id_hash: acknowledged_hash.clone(),
            action: acknowledgement.action,
        });
        self.completed_work_hashes.push(acknowledged_hash);
        Ok(self)
    }

    #[must_use]
    fn candidate(&self, candidate: ContinuationCandidate) -> Option<&PendingContinuationWork> {
        match candidate {
            ContinuationCandidate::QueuedPrompt => self.queued_prompt.as_ref(),
            ContinuationCandidate::LoopContinuation => self.active_loop.as_ref(),
            ContinuationCandidate::ValidationFollowUp => self.validation_follow_up.as_ref(),
        }
    }

    #[must_use]
    fn has_candidate(&self, work_id: &str) -> bool {
        self.queued_prompt
            .as_ref()
            .is_some_and(|work| work.work_id() == work_id)
            || self
                .active_loop
                .as_ref()
                .is_some_and(|work| work.work_id() == work_id)
            || self
                .validation_follow_up
                .as_ref()
                .is_some_and(|work| work.work_id() == work_id)
    }

    fn remove_candidate(&mut self, work_id: &str) {
        clear_if_matches(&mut self.queued_prompt, work_id);
        clear_if_matches(&mut self.active_loop, work_id);
        clear_if_matches(&mut self.validation_follow_up, work_id);
    }
}
