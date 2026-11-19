//! The attempt's own stop policy ahead of the serving gate (#13): the broker's
//! exhausted request budget and, when declared, an observed overrun of known
//! served usage beyond the declared token envelopes. Either refuses before any
//! port call. Each stop is recorded once per route as a not-dispatched record;
//! later refusals return that same record without a new ordinal or file.

// r[impl lattice.serving_binding.overrun]
// r[impl lattice.serving_binding.overrun.scenario.stop]
// r[impl lattice.serving_binding.overrun.scenario.budget_exhausted]

type Record = lattice_core::execution_binding::serving::record::Observation;

impl super::Session<'_> {
    /// Record the broker's exhausted request budget as `budget-exhausted`.
    /// The serving port is never called.
    pub fn refuse_budget_exhausted(&mut self, route_name: &str) -> Result<Record, String> {
        let index = self.routes.iter().position(|route| route.name == route_name).ok_or("model route is not served")?;
        self.stop(index, lattice_core::execution_binding::serving::envelope::BUDGET_EXHAUSTED)
    }

    /// Known usage and the declared envelopes so far; unknown usage is never headroom.
    #[must_use]
    pub const fn overrun(&self) -> &lattice_core::execution_binding::serving::envelope::Overrun {
        &self.overrun
    }

    /// With `stop_on_overrun`, an observed overrun refuses before the gate.
    pub(super) fn overrun_stop(&mut self, index: usize) -> Result<Option<Record>, String> {
        if !self.overrun.must_stop() {
            return Ok(None);
        }
        debug_assert!(self.overrun.overrun_tokens > 0, "a stop follows an observed overrun");
        self.stop(index, lattice_core::execution_binding::serving::envelope::OVERRUN_STOPPED).map(Some)
    }

    /// Account one settled dispatched request against its route's envelope.
    pub(super) fn account(&mut self, record: &Record) -> Result<(), String> {
        let route = self
            .routes
            .iter()
            .find(|route| route.name == record.route_name)
            .ok_or("model route is not served")?;
        debug_assert!(record.outcome.class.is_dispatched(), "only dispatched requests reserve an envelope");
        self.overrun
            .dispatched(route.envelope_tokens, record.usage)
            .map_err(|conflict| format!("served usage overflowed: {conflict:?}"))
    }

    fn stop(&mut self, index: usize, label: &str) -> Result<Record, String> {
        let route_name = self.routes.get(index).map(|route| route.name.clone()).ok_or("model route is not served")?;
        let existing =
            self.stops.iter().find(|record| record.route_name == route_name && record.outcome.label == label);
        if let Some(record) = existing {
            return Ok(record.clone());
        }
        let slot = self.next_slot(index)?;
        let record = self.context.unscreened(&slot, label);
        self.sink.write_record(&record)?;
        debug_assert!(!record.outcome.class.is_dispatched(), "a stop never dispatches");
        debug_assert!(record.receipt_identity.is_none(), "a stop screens no snapshot");
        self.stops.push(record.clone());
        Ok(record)
    }

    pub(super) fn next_slot(&mut self, index: usize) -> Result<super::super::receipt::Slot, String> {
        let route = self.routes.get(index).ok_or("model route is not served")?;
        let ordinal = self.ordinals.entry(route.name.clone()).or_insert(0);
        let next = ordinal.checked_add(1).ok_or("model request ordinal overflowed")?;
        *ordinal = next;
        let request_id = super::super::draft::request_id((self.context.binding_identity(), &route.name), next);
        debug_assert!(next > 0);
        debug_assert!(lattice_core::execution_binding::serving::is_producer_token(&request_id));
        Ok(super::super::receipt::Slot {
            route_name: route.name.clone(),
            service_id: route.service_id.clone(),
            request_id,
            ordinal: next,
        })
    }
}
