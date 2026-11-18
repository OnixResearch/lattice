//! Port implementations. Neither has a write path.

impl super::ComputeAllocationPort for super::UnavailableComputePort {
    fn observe_allocation(
        &mut self,
        _query: &super::AllocationQuery,
    ) -> Result<super::AllocationReply, super::ComputePortIssue> {
        Ok(super::AllocationReply::Unavailable {
            reason: super::PortUnavailableReason::ProviderNotConfigured,
        })
    }
}

impl super::ComputeAllocationPort for super::FixtureComputePort {
    fn observe_allocation(
        &mut self,
        query: &super::AllocationQuery,
    ) -> Result<super::AllocationReply, super::ComputePortIssue> {
        self.calls.push(query.clone());
        self.replies.pop_front().unwrap_or(Err(super::ComputePortIssue::new(
            super::PortIssueKind::MalformedReply,
            "fixture port has no scripted reply",
        )))
    }
}

impl super::FixtureComputePort {
    /// Script replies in the order they will be returned.
    #[must_use]
    pub fn new(replies: Vec<Result<super::AllocationReply, super::ComputePortIssue>>) -> Self {
        Self {
            replies: replies.into(),
            calls: Vec::new(),
        }
    }

    /// Every query the port received, in order.
    #[must_use]
    pub fn calls(&self) -> &[super::AllocationQuery] {
        &self.calls
    }
}

impl super::ComputePortIssue {
    /// Build an issue, keeping at most 256 bytes of provider text on a
    /// character boundary.
    #[must_use]
    pub fn new(kind: super::PortIssueKind, detail: &str) -> Self {
        let end_bytes = detail.floor_char_boundary(super::MAX_ISSUE_DETAIL_BYTES);
        Self {
            kind,
            detail: detail.get(..end_bytes).unwrap_or_default().to_owned(),
        }
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}
