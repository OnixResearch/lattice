//! The per-request serving gate of one attempt: screen, owner admission,
//! recheck and `gate_dispatch`; only an admitted verdict with a durable intent
//! returns a [`Ticket`]. Every refusal and settled call leaves one record, and
//! nothing is retried, replayed or rerouted.

// r[impl lattice.serving_binding.request_admission]
// r[impl lattice.serving_binding.request_admission.scenario.not_reservation]
// r[impl lattice.serving_binding.request_admission.scenario.refused]
// r[impl lattice.serving_binding.request_observation.scenario.uncertain]
// r[impl lattice.serving_binding.request_observation.scenario.cancellation]

mod stop;

type Admitted = lattice_core::execution_binding::admission::bound::Admitted;
type Record = lattice_core::execution_binding::serving::record::Observation;

/// The broker's own checks, passed to the producer as the existing admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checks {
    pub is_exact_model: bool,
    pub is_within_budget: bool,
    pub is_current: bool,
}

/// What the upstream response reported; counters are provider reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseFacts {
    pub model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub provider_receipt: Option<String>,
}

/// How the admitted upstream call ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Upstream {
    Responded(ResponseFacts),
    TimedOut,
    /// Transport failure, or a response the broker withheld from the worker.
    TransportFailed,
    Cancelled,
}

/// Permission for exactly one upstream call, held until it is settled.
#[derive(Debug)]
pub struct Ticket {
    slot: super::receipt::Slot,
    screening: animus_adapters::serving::Screening,
    selected: animus_adapters::serving::EligibleRoute,
    query: animus_adapters::serving::AdmissionQuery,
    owner: animus_adapters::serving::OwnerReply,
    permit: animus_adapters::serving::DispatchPermit,
}

impl Ticket {
    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.slot.request_id
    }
}

#[derive(Debug)]
pub enum Decision {
    Dispatch(Box<Ticket>),
    Refused(Box<Record>),
}

struct Served {
    name: String,
    service_id: String,
    envelope_tokens: u64,
    candidate: animus_adapters::serving::RouteCandidate,
}

/// Gate state of one attempt's served routes.
pub struct Session<'a> {
    context: super::receipt::Context,
    profile: animus_adapters::serving::Profile,
    privacy: animus_adapters::router::PrivacyBand,
    routes: Vec<Served>,
    newest: Option<(u64, u64)>,
    ordinals: std::collections::BTreeMap<String, u64>,
    /// Known served usage against the declared envelopes, settled so far.
    overrun: lattice_core::execution_binding::serving::envelope::Overrun,
    /// At most one recorded refusal per route and stop label.
    stops: Vec<Record>,
    port: &'a mut dyn super::OwnerPort,
    sink: &'a mut dyn super::Sink,
}

impl<'a> Session<'a> {
    /// Build the producer profile and candidates of a binding's serving
    /// declaration. A session over retained evidence continues after the
    /// highest retained ordinal of each route, so a restarted session never
    /// reuses a request identity whose intent or record exists.
    // r[impl lattice.serving_binding.brokered_evidence.scenario.conflict]
    pub fn new(
        admitted: &Admitted,
        worker_incarnation: &str,
        port: &'a mut dyn super::OwnerPort,
        sink: &'a mut dyn super::Sink,
    ) -> Result<Self, String> {
        let declaration = admitted.serving.as_ref().ok_or("execution binding declares no served route")?;
        let context = super::receipt::Context::new(admitted, worker_incarnation, port.provenance())?;
        let mut routes = Vec::with_capacity(declaration.routes.len());
        let mut ordinals = std::collections::BTreeMap::new();
        for route in &declaration.routes {
            routes.push(Served {
                name: route.name.clone(),
                service_id: route.service_id.clone(),
                envelope_tokens: lattice_core::execution_binding::serving::envelope::reserved_tokens(route),
                candidate: super::draft::candidate(admitted, route)?,
            });
            ordinals.insert(route.name.clone(), sink.highest_ordinal(&route.name)?);
        }
        debug_assert_eq!(routes.len(), declaration.routes.len());
        debug_assert!(!routes.is_empty(), "a validated declaration serves a route");
        Ok(Self {
            context,
            profile: super::draft::profile(&declaration.profile)?,
            privacy: super::draft::privacy(declaration.privacy),
            routes,
            newest: None,
            ordinals,
            overrun: lattice_core::execution_binding::serving::envelope::Overrun::new(declaration.stop_on_overrun),
            stops: Vec::new(),
            port,
            sink,
        })
    }

    /// Whether `route_name` is served by this binding.
    #[must_use]
    pub fn serves(&self, route_name: &str) -> bool {
        self.routes.iter().any(|route| route.name == route_name)
    }

    /// Decide one request before any upstream effect.
    pub fn admit(&mut self, route_name: &str, checks: Checks) -> Result<Decision, String> {
        let index = self.routes.iter().position(|route| route.name == route_name).ok_or("model route is not served")?;
        // The stop policy runs before the gate: no port call, no new ordinal once stopped.
        if let Some(record) = self.overrun_stop(index)? {
            return Ok(Decision::Refused(Box::new(record)));
        }
        let slot = self.next_slot(index)?;
        let query = super::SnapshotQuery {
            attempt_id: self.context.attempt_id().to_owned(),
            route_name: slot.route_name.clone(),
            service_id: slot.service_id.clone(),
        };
        let (snapshot, now_tick) = match self.port.observe_snapshot(&query) {
            super::SnapshotReply::Observed { snapshot, now_tick } => (snapshot, now_tick),
            super::SnapshotReply::Unavailable => return self.refuse_unscreened(&slot, "snapshot-unavailable"),
            super::SnapshotReply::PortIssue => return self.refuse_unscreened(&slot, "snapshot-port-issue"),
        };
        let candidates = self.routes.get(index).map(|route| std::slice::from_ref(&route.candidate)).unwrap_or_default();
        let screening = animus_adapters::serving::screen_routes(&animus_adapters::serving::ScreenRequest {
            profile: &self.profile,
            snapshot: &snapshot,
            freshness: animus_adapters::serving::Freshness {
                now_tick,
                newest_accepted: self.newest,
            },
            privacy: self.privacy,
            candidates,
        })
        .map_err(|issue| format!("serving screen contract issue: {issue:?}"))?;
        if screening.status() == animus_adapters::serving::SnapshotStatus::Current {
            let position = (snapshot.source_generation(), snapshot.sequence());
            self.newest = Some(self.newest.map_or(position, |newest| newest.max(position)));
        }
        debug_assert_eq!(candidates.len(), 1, "one declared route is screened per request");
        let Some(selected) = screening.eligible().first().cloned() else {
            let outcome = animus_adapters::serving::NotDispatched::NoEligibleRoute;
            return self.refuse(&slot, (&screening, None, None), outcome);
        };
        self.gate(slot, screening, selected, checks)
    }

    fn gate(
        &mut self,
        slot: super::receipt::Slot,
        screening: animus_adapters::serving::Screening,
        selected: animus_adapters::serving::EligibleRoute,
        checks: Checks,
    ) -> Result<Decision, String> {
        let query = animus_adapters::serving::admission_query(&selected, self.context.caller(), &slot.request_id)
            .map_err(|issue| format!("serving admission query is invalid: {issue:?}"))?;
        let owner = self.port.admit_request(&query);
        let is_granted = matches!(owner, animus_adapters::serving::OwnerReply::Granted(_));
        let current = if is_granted {
            self.port.recheck(&query)
        } else {
            super::Current::Unavailable
        };
        let verdict = animus_adapters::serving::gate_dispatch(&animus_adapters::serving::GateInput {
            profile: &self.profile,
            query: &query,
            owner: &owner,
            current: super::draft::reading(&current),
            existing: super::draft::existing(checks),
        });
        let outcome = match verdict {
            animus_adapters::serving::GateVerdict::Admitted(permit) => {
                return self.dispatch(Ticket {
                    slot,
                    screening,
                    selected,
                    query,
                    owner,
                    permit,
                });
            }
            animus_adapters::serving::GateVerdict::Rejected(rejection) => {
                animus_adapters::serving::NotDispatched::Rejected(rejection)
            }
            animus_adapters::serving::GateVerdict::Blocked(block) => {
                animus_adapters::serving::NotDispatched::Blocked(block)
            }
        };
        debug_assert!(!matches!(owner, animus_adapters::serving::OwnerReply::Granted(_)) || is_granted);
        self.refuse(&slot, (&screening, Some(&selected), Some(&owner)), outcome)
    }

    /// Persist the intent first; only then may the caller contact upstream.
    fn dispatch(&mut self, ticket: Ticket) -> Result<Decision, String> {
        let intent = self.context.intent(&ticket.slot, &ticket.selected, &ticket.permit);
        // Create-only: an existing intent refuses the request before upstream.
        self.sink.create_intent(&intent)?;
        debug_assert_eq!(intent.request_id, ticket.slot.request_id);
        debug_assert!(intent.deployment_generation > 0, "producer generations are positive");
        Ok(Decision::Dispatch(Box::new(ticket)))
    }

    /// Record how the admitted call ended. Uncertain calls are never retried.
    pub fn settle(&mut self, ticket: Ticket, upstream: Upstream) -> Result<Record, String> {
        let (outcome, response, usage, provider) = match upstream {
            Upstream::Responded(facts) => super::draft::responded(&ticket.query, facts),
            Upstream::TimedOut => super::draft::uncertain(animus_adapters::serving::UncertainCause::TimedOut),
            Upstream::TransportFailed => {
                super::draft::uncertain(animus_adapters::serving::UncertainCause::TransportFailed)
            }
            Upstream::Cancelled => super::draft::uncertain(animus_adapters::serving::UncertainCause::Cancelled),
        };
        let admission = animus_adapters::serving::AdmissionRecord::from_reply(&ticket.owner);
        let receipt = animus_adapters::serving::project_receipt(&animus_adapters::serving::ReceiptInput {
            evidence: self.context.evidence(true),
            request_id: &ticket.slot.request_id,
            caller: self.context.caller(),
            screening: &ticket.screening,
            selected: Some(&ticket.selected),
            admission: &admission,
            permit: Some(&ticket.permit),
            outcome,
            response,
            usage,
            provider_receipt_identity: provider.as_deref(),
        })
        .map_err(|issue| format!("serving receipt is inconsistent: {issue:?}"))?;
        let record = self.context.record(&ticket.slot, &receipt, None);
        self.sink.write_record(&record)?;
        self.account(&record)?;
        debug_assert!(record.outcome.class.is_dispatched(), "a settled ticket was dispatched");
        Ok(record)
    }

    fn refuse(
        &mut self,
        slot: &super::receipt::Slot,
        seen: super::receipt::Seen<'_>,
        refusal: animus_adapters::serving::NotDispatched,
    ) -> Result<Decision, String> {
        let (screening, selected, owner) = seen;
        let admission = owner.map_or(
            animus_adapters::serving::AdmissionRecord::NotRequested,
            animus_adapters::serving::AdmissionRecord::from_reply,
        );
        let receipt = animus_adapters::serving::project_receipt(&animus_adapters::serving::ReceiptInput {
            evidence: self.context.evidence(false),
            request_id: &slot.request_id,
            caller: self.context.caller(),
            screening,
            selected,
            admission: &admission,
            permit: None,
            outcome: animus_adapters::serving::DispatchOutcome::NotDispatched(refusal),
            response: None,
            usage: animus_adapters::serving::Usage::UNKNOWN,
            provider_receipt_identity: None,
        })
        .map_err(|issue| format!("serving receipt is inconsistent: {issue:?}"))?;
        let detail = screening.exclusions().first().map(super::receipt::exclusion_detail);
        let record = self.context.record(slot, &receipt, detail);
        self.sink.write_record(&record)?;
        debug_assert!(!record.outcome.class.is_dispatched());
        Ok(Decision::Refused(Box::new(record)))
    }

    fn refuse_unscreened(&mut self, slot: &super::receipt::Slot, label: &str) -> Result<Decision, String> {
        let record = self.context.unscreened(slot, label);
        self.sink.write_record(&record)?;
        debug_assert!(record.receipt_identity.is_none(), "no snapshot means no producer receipt");
        Ok(Decision::Refused(Box::new(record)))
    }
}
