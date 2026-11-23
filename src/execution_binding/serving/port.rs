//! Port and sink implementations. None has a reserve, drain or release path.

// r[impl lattice.serving_binding.port_boundary.scenario.unavailable]
impl super::OwnerPort for super::UnavailableOwner {
    fn observe_snapshot(&mut self, _query: &super::SnapshotQuery) -> super::SnapshotReply {
        super::SnapshotReply::Unavailable
    }

    fn admit_request(
        &mut self,
        _query: &animus_adapters::serving::AdmissionQuery,
    ) -> animus_adapters::serving::OwnerReply {
        animus_adapters::serving::OwnerReply::Unavailable
    }

    fn recheck(&mut self, _query: &animus_adapters::serving::AdmissionQuery) -> super::Current {
        super::Current::Unavailable
    }

    fn provenance(&self) -> super::PortProvenance {
        super::PortProvenance::Unavailable
    }
}

impl super::OwnerPort for super::FixtureOwner {
    fn observe_snapshot(&mut self, query: &super::SnapshotQuery) -> super::SnapshotReply {
        self.calls.push(super::Call::Snapshot(query.clone()));
        self.snapshots.pop_front().unwrap_or(super::SnapshotReply::Unavailable)
    }

    fn admit_request(
        &mut self,
        query: &animus_adapters::serving::AdmissionQuery,
    ) -> animus_adapters::serving::OwnerReply {
        self.calls.push(super::Call::Admit {
            request_id: query.request_id().to_owned(),
        });
        match self.grants.pop_front() {
            None => animus_adapters::serving::OwnerReply::Unavailable,
            Some(super::OwnerScript::Reply(reply)) => reply,
            Some(super::OwnerScript::GrantQueried {
                grant_ref,
                admission_epoch,
                issued_tick,
                valid_until_tick,
            }) => granted(query, &grant_ref, admission_epoch, (issued_tick, valid_until_tick)),
        }
    }

    fn recheck(&mut self, query: &animus_adapters::serving::AdmissionQuery) -> super::Current {
        self.calls.push(super::Call::Recheck {
            request_id: query.request_id().to_owned(),
        });
        self.rechecks.pop_front().unwrap_or(super::Current::Unavailable)
    }

    fn provenance(&self) -> super::PortProvenance {
        super::PortProvenance::Fixture
    }
}

impl super::FixtureOwner {
    /// Script snapshot, owner and recheck replies, each returned in order.
    #[must_use]
    pub fn new(
        snapshots: Vec<super::SnapshotReply>,
        grants: Vec<super::OwnerScript>,
        rechecks: Vec<super::Current>,
    ) -> Self {
        Self {
            snapshots: snapshots.into(),
            grants: grants.into(),
            rechecks: rechecks.into(),
            calls: Vec::new(),
        }
    }

    /// Every call the port received, in order.
    #[must_use]
    pub fn calls(&self) -> &[super::Call] {
        &self.calls
    }
}

/// A grant for exactly the queried request; an invalid script answers a port issue.
fn granted(
    query: &animus_adapters::serving::AdmissionQuery,
    grant_ref: &str,
    admission_epoch: u64,
    validity_ticks: (u64, u64),
) -> animus_adapters::serving::OwnerReply {
    let deployment = query.deployment();
    debug_assert!(deployment.deployment_generation() > 0, "producer generations are positive");
    debug_assert!(!query.request_id().is_empty(), "queries carry a request id");
    let replica = match query.selected_replica() {
        animus_adapters::serving::ReplicaFact::Observed(token) => Some(token.as_str()),
        animus_adapters::serving::ReplicaFact::Unknown => None,
    };
    animus_adapters::serving::Grant::new(&animus_adapters::serving::GrantDraft {
        grant_ref,
        request_id: query.request_id(),
        service_id: deployment.service_id(),
        deployment_id: deployment.deployment_id(),
        deployment_generation: deployment.deployment_generation(),
        model_id: query.model().model_id(),
        configuration_ref: query.model().configuration_ref(),
        admission_epoch,
        issued_tick: validity_ticks.0,
        valid_until_tick: validity_ticks.1,
        replica,
        policy_generation: query.policy_generation(),
    })
    .map_or(animus_adapters::serving::OwnerReply::PortIssue, animus_adapters::serving::OwnerReply::granted)
}

impl super::Sink for super::MemorySink {
    fn create_intent(&mut self, intent: &super::Intent) -> Result<(), String> {
        if self.intents.iter().any(|existing| existing.request_id == intent.request_id) {
            return Err("model request intent already exists".into());
        }
        self.intents.push(intent.clone());
        Ok(())
    }

    fn highest_ordinal(&self, route_name: &str) -> Result<u64, String> {
        let intents = self.intents.iter().filter(|intent| intent.route_name == route_name).map(|intent| intent.ordinal);
        let records = self.records.iter().filter(|record| record.route_name == route_name).map(|record| record.ordinal);
        Ok(intents.chain(records).max().unwrap_or(0))
    }

    fn write_record(&mut self, record: &super::Record) -> Result<(), String> {
        if is_new(&self.records, record, |existing| existing.request_id == record.request_id)? {
            self.records.push(record.clone());
        }
        Ok(())
    }
}

/// Identical repeats are idempotent; a different value for the same request is refused.
fn is_new<T: PartialEq>(items: &[T], item: &T, is_same_key: impl Fn(&T) -> bool) -> Result<bool, String> {
    match items.iter().find(|existing| is_same_key(existing)) {
        Some(existing) if existing == item => Ok(false),
        Some(_) => Err("request evidence already exists with different content".into()),
        None => Ok(true),
    }
}
