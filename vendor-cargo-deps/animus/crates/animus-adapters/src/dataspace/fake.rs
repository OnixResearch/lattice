const INITIAL_SEQUENCE: u64 = 0;
const SEQUENCE_STEP: u64 = 1;
const NO_UPDATES: usize = 0;
const UPDATE_STEP: usize = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct StoredAssertion {
    sequence: u64,
    assertion: super::Assertion,
}

impl StoredAssertion {
    pub(super) const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(super) const fn assertion(&self) -> &super::Assertion {
        &self.assertion
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Backend {
    next_sequence: u64,
    assertions: alloc::vec::Vec<StoredAssertion>,
}

impl Backend {
    pub(super) const fn new() -> Self {
        Self {
            next_sequence: INITIAL_SEQUENCE,
            assertions: alloc::vec::Vec::new(),
        }
    }

    pub(super) fn assertions(&self) -> &[StoredAssertion] {
        self.assertions.as_slice()
    }

    pub(super) fn publish_assertion(&mut self, assertion: super::Assertion) {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(SEQUENCE_STEP);
        self.assertions.push(StoredAssertion {
            sequence,
            assertion,
        });
    }

    pub(super) fn query_assertions(
        &self,
        subscription: &super::Subscription,
    ) -> alloc::vec::Vec<super::Assertion> {
        self.assertions
            .iter()
            .filter(|stored| {
                stored
                    .assertion
                    .scope()
                    .same_tenant_workspace(subscription.scope())
            })
            .filter(|stored| subscription.allows(stored.assertion.kind()))
            .map(|stored| stored.assertion.clone())
            .collect()
    }

    pub(super) fn apply_event(&mut self, event: &super::OrchestrationEvent) -> usize {
        match event.kind() {
            super::OrchestrationEventKind::Cancelled => {
                self.mark_scope(event.scope(), super::AssertionStatus::Cancelled)
            }
            super::OrchestrationEventKind::Expired => {
                self.mark_scope(event.scope(), super::AssertionStatus::Expired)
            }
            super::OrchestrationEventKind::Published
            | super::OrchestrationEventKind::OwnershipTransferred => NO_UPDATES,
        }
    }

    pub(super) fn cleanup_stale(&mut self, now: u64) -> usize {
        let before = self.assertions.len();
        self.assertions
            .retain(|stored| !stored.assertion.is_stale_at(now));
        before.saturating_sub(self.assertions.len())
    }

    fn mark_scope(&mut self, scope: &super::Scope, status: super::AssertionStatus) -> usize {
        let mut updated = NO_UPDATES;
        for stored in &mut self.assertions {
            if stored.assertion.scope().same_actor_scope(scope) {
                stored.assertion.set_status(status);
                updated = updated.saturating_add(UPDATE_STEP);
            }
        }
        updated
    }
}

impl super::AdapterService for Backend {
    type Error = core::convert::Infallible;

    fn publish(&mut self, assertion: &super::Assertion) -> Result<(), Self::Error> {
        self.publish_assertion(assertion.clone());
        Ok(())
    }

    fn subscribe(
        &mut self,
        subscription: &super::Subscription,
    ) -> Result<alloc::vec::Vec<super::Assertion>, Self::Error> {
        Ok(self.query_assertions(subscription))
    }

    fn handle_event(&mut self, event: &super::OrchestrationEvent) -> Result<(), Self::Error> {
        self.apply_event(event);
        Ok(())
    }
}
