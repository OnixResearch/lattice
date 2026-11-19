//! Conversions between the Lattice serving declaration and records and the
//! producer's validated serving values.

// r[impl lattice.serving_binding.request_observation]

type Admitted = lattice_core::execution_binding::admission::bound::Admitted;
type Record = lattice_core::execution_binding::serving::record::Observation;
type Intent = lattice_core::execution_binding::serving::record::DispatchIntent;

/// What a refusal saw: the screening, the selected route and the owner reply.
pub(super) type Seen<'a> = (
    &'a animus_adapters::serving::Screening,
    Option<&'a animus_adapters::serving::EligibleRoute>,
    Option<&'a animus_adapters::serving::OwnerReply>,
);

/// One request's route, service, identity and ordinal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Slot {
    pub(super) route_name: String,
    pub(super) service_id: String,
    pub(super) request_id: String,
    pub(super) ordinal: u64,
}

/// Binding and caller identities every record carries.
pub(super) struct Context {
    admitted: Admitted,
    worker_incarnation: String,
    provenance: super::PortProvenance,
    caller: animus_adapters::serving::CallerBinding,
}

impl Context {
    pub(super) fn new(
        admitted: &Admitted,
        worker_incarnation: &str,
        provenance: super::PortProvenance,
    ) -> Result<Self, String> {
        let caller = animus_adapters::serving::CallerBinding::new(animus_adapters::serving::CallerDraft {
            agent: &admitted.profile_revision,
            session: worker_incarnation,
            attempt: &admitted.attempt.attempt_id,
        })
        .map_err(|issue| format!("caller identities are not serving tokens: {issue:?}"))?;
        debug_assert_eq!(caller.attempt_id(), admitted.attempt.attempt_id);
        debug_assert_eq!(caller.session_id(), worker_incarnation);
        Ok(Self {
            admitted: admitted.clone(),
            worker_incarnation: worker_incarnation.to_owned(),
            provenance,
            caller,
        })
    }

    pub(super) const fn caller(&self) -> &animus_adapters::serving::CallerBinding {
        &self.caller
    }

    pub(super) fn attempt_id(&self) -> &str {
        &self.admitted.attempt.attempt_id
    }

    pub(super) fn binding_identity(&self) -> &str {
        &self.admitted.binding_identity
    }

    /// The producer receipt label: `live-endpoint` only for an admitted call
    /// through an embedder-supplied port; everything else is not live.
    pub(super) const fn evidence(&self, is_dispatched: bool) -> animus_adapters::serving::EvidenceLabel {
        match self.record_evidence(is_dispatched) {
            lattice_core::execution_binding::serving::record::Evidence::LiveEndpoint => {
                animus_adapters::serving::EvidenceLabel::LiveEndpoint
            }
            lattice_core::execution_binding::serving::record::Evidence::Fixture
            | lattice_core::execution_binding::serving::record::Evidence::NoUpstreamContact => {
                animus_adapters::serving::EvidenceLabel::Fixture
            }
        }
    }

    // r[impl lattice.serving_binding.non_claims.scenario.fixture]
    // r[impl lattice.serving_binding.non_claims.scenario.provenance]
    const fn record_evidence(&self, is_dispatched: bool) -> lattice_core::execution_binding::serving::record::Evidence {
        match (self.provenance, is_dispatched) {
            (super::PortProvenance::Fixture, _) => lattice_core::execution_binding::serving::record::Evidence::Fixture,
            (super::PortProvenance::Embedder, true) => {
                lattice_core::execution_binding::serving::record::Evidence::LiveEndpoint
            }
            (super::PortProvenance::Embedder | super::PortProvenance::Unavailable, false)
            | (super::PortProvenance::Unavailable, true) => {
                lattice_core::execution_binding::serving::record::Evidence::NoUpstreamContact
            }
        }
    }

    pub(super) fn intent(
        &self,
        slot: &Slot,
        selected: &animus_adapters::serving::EligibleRoute,
        permit: &animus_adapters::serving::DispatchPermit,
    ) -> Intent {
        let deployment = selected.deployment();
        Intent {
            schema: lattice_core::execution_binding::serving::record::INTENT_SCHEMA.to_owned(),
            binding_identity: self.admitted.binding_identity.clone(),
            attempt_id: self.admitted.attempt.attempt_id.clone(),
            fence: self.admitted.attempt.fence,
            route_name: slot.route_name.clone(),
            request_id: slot.request_id.clone(),
            ordinal: slot.ordinal,
            service_id: deployment.service_id().to_owned(),
            deployment_id: deployment.deployment_id().to_owned(),
            deployment_generation: deployment.deployment_generation(),
            grant_ref: permit.grant_ref().to_owned(),
            admission_epoch: permit.admission_epoch(),
        }
    }

    /// A record carrying the producer receipt's facts and identity.
    pub(super) fn record(
        &self,
        slot: &Slot,
        receipt: &animus_adapters::serving::InvocationReceipt,
        detail: Option<String>,
    ) -> Record {
        let mut record = self.unscreened(slot, receipt.outcome().as_str());
        let (source_generation, sequence) = receipt.snapshot_position();
        record.served = receipt.selected().map(|selected| super::draft::served(receipt, selected));
        record.snapshot = Some(lattice_core::execution_binding::serving::record::Snapshot {
            identity: receipt.snapshot_identity().to_owned(),
            source_generation,
            sequence,
            status: receipt.snapshot_status().as_str().to_owned(),
        });
        record.admission = admission(receipt.admission());
        record.outcome = lattice_core::execution_binding::serving::record::Outcome {
            class: class(receipt.outcome()),
            label: receipt.outcome().as_str().to_owned(),
            detail,
        };
        record.usage = usage(receipt.usage());
        record.receipt_identity = Some(receipt.identity().to_owned());
        record.evidence = self.record_evidence(receipt.outcome().is_dispatched());
        record.provider_receipt_identity = receipt.provider_receipt_identity().map(str::to_owned);
        debug_assert_eq!(record.request_id, receipt.request_id());
        debug_assert_eq!(record.outcome.class.is_dispatched(), receipt.outcome().is_dispatched());
        record
    }

    /// A refusal before any snapshot could be screened; no producer receipt exists.
    pub(super) fn unscreened(&self, slot: &Slot, label: &str) -> Record {
        let route_identity = match &self.admitted.model_routes {
            lattice_core::execution_binding::request::RouteDeclaration::Routes { routes } => {
                routes.iter().find(|route| route.name == slot.route_name).map(|route| route.route_identity.clone())
            }
            lattice_core::execution_binding::request::RouteDeclaration::NoModel {} => None,
        };
        Record {
            schema: lattice_core::execution_binding::serving::record::OBSERVATION_SCHEMA.to_owned(),
            binding_identity: self.admitted.binding_identity.clone(),
            profile_revision: self.admitted.profile_revision.clone(),
            worker_incarnation: self.worker_incarnation.clone(),
            attempt_id: self.admitted.attempt.attempt_id.clone(),
            fence: self.admitted.attempt.fence,
            route_name: slot.route_name.clone(),
            route_identity: route_identity.unwrap_or_default(),
            service_id: slot.service_id.clone(),
            request_id: slot.request_id.clone(),
            ordinal: slot.ordinal,
            served: None,
            snapshot: None,
            admission: lattice_core::execution_binding::serving::record::Admission {
                label: "not-requested".to_owned(),
                grant_ref: None,
                admission_epoch: None,
            },
            outcome: lattice_core::execution_binding::serving::record::Outcome {
                class: lattice_core::execution_binding::serving::record::OutcomeClass::NotDispatched,
                label: label.to_owned(),
                detail: None,
            },
            usage: lattice_core::execution_binding::serving::record::Usage::UNKNOWN,
            evidence: self.record_evidence(false),
            receipt_identity: None,
            provider_receipt_identity: None,
        }
    }
}

fn admission(
    record: &animus_adapters::serving::AdmissionRecord,
) -> lattice_core::execution_binding::serving::record::Admission {
    let (grant_ref, admission_epoch) = match record {
        animus_adapters::serving::AdmissionRecord::Granted {
            grant_ref,
            admission_epoch,
        } => (Some(grant_ref.as_str().to_owned()), Some(*admission_epoch)),
        animus_adapters::serving::AdmissionRecord::NotRequested
        | animus_adapters::serving::AdmissionRecord::Rejected(_)
        | animus_adapters::serving::AdmissionRecord::Unavailable
        | animus_adapters::serving::AdmissionRecord::PortIssue => (None, None),
    };
    lattice_core::execution_binding::serving::record::Admission {
        label: record.as_str().to_owned(),
        grant_ref,
        admission_epoch,
    }
}

const fn class(
    outcome: animus_adapters::serving::DispatchOutcome,
) -> lattice_core::execution_binding::serving::record::OutcomeClass {
    match outcome {
        animus_adapters::serving::DispatchOutcome::NotDispatched(_) => {
            lattice_core::execution_binding::serving::record::OutcomeClass::NotDispatched
        }
        animus_adapters::serving::DispatchOutcome::Completed => {
            lattice_core::execution_binding::serving::record::OutcomeClass::Completed
        }
        animus_adapters::serving::DispatchOutcome::ResponseRejected(_) => {
            lattice_core::execution_binding::serving::record::OutcomeClass::ResponseRejected
        }
        animus_adapters::serving::DispatchOutcome::Uncertain(_) => {
            lattice_core::execution_binding::serving::record::OutcomeClass::Uncertain
        }
    }
}

const fn usage(usage: animus_adapters::serving::Usage) -> lattice_core::execution_binding::serving::record::Usage {
    lattice_core::execution_binding::serving::record::Usage {
        input_tokens: counter(usage.input_tokens),
        output_tokens: counter(usage.output_tokens),
        origin: match usage.origin {
            animus_adapters::serving::UsageOrigin::ProviderReported => {
                lattice_core::execution_binding::serving::record::UsageOrigin::ProviderReported
            }
            animus_adapters::serving::UsageOrigin::Unknown => {
                lattice_core::execution_binding::serving::record::UsageOrigin::Unknown
            }
        },
    }
}

const fn counter(
    counter: animus_adapters::serving::Counter,
) -> lattice_core::execution_binding::serving::record::Counter {
    match counter {
        animus_adapters::serving::Counter::Known(value) => {
            lattice_core::execution_binding::serving::record::Counter::Known(value)
        }
        animus_adapters::serving::Counter::Unknown => {
            lattice_core::execution_binding::serving::record::Counter::Unknown
        }
    }
}

/// Screening exclusion label, with the reported readiness when not ready.
pub(super) fn exclusion_detail(exclusion: &animus_adapters::serving::Exclusion) -> String {
    let reason = exclusion.reason();
    if let animus_adapters::serving::ExclusionReason::NotReady(readiness) = reason {
        return format!("not-ready:{}", readiness.as_str());
    }
    reason.as_str().to_owned()
}
