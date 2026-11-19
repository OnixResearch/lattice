//! Known served usage against the declared per-request token envelopes (#13).
//!
//! A served route's envelope (`input_tokens + output_tokens`) is reserved for
//! every dispatched request. Only requests whose provider reported both
//! counters add known tokens, and each is compared with its own envelope, so
//! neither an under-used request nor unknown usage is headroom that could
//! offset an overrun. While any usage is unknown the reservation is not an
//! exact ceiling. Provider counters are reports, not invoices; no pricing
//! identity exists.

// r[impl lattice.serving_binding.overrun]
// r[impl lattice.serving_binding.overrun.scenario.inexact]

/// Not-dispatched label of a request the broker's request budget refused.
pub const BUDGET_EXHAUSTED: &str = "budget-exhausted";
/// Not-dispatched label of a request refused after an observed overrun.
pub const OVERRUN_STOPPED: &str = "overrun-stopped";

type Conflict = super::usage::Conflict;

/// Reserved versus known tokens of one attempt's served requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overrun {
    /// The declaration refuses further requests once an overrun is observed.
    pub stop_on_overrun: bool,
    /// Declared envelopes of every dispatched request; a reservation, not usage.
    pub reserved_tokens: u64,
    /// Provider-reported tokens of the requests whose usage is known.
    pub known_tokens: u64,
    /// Known-usage requests that exceeded their own envelope.
    pub overrun_requests: u64,
    /// Known tokens beyond those requests' own envelopes.
    pub overrun_tokens: u64,
    /// Dispatched requests without known usage; their envelopes are never headroom.
    pub unknown_usage_requests: u64,
    /// Requests refused by the request budget or the stop policy, each stop recorded once.
    pub refused_requests: u64,
    /// Every dispatched request reported usage within its envelope. Never true
    /// while any usage is unknown.
    pub is_exact_ceiling: bool,
}

impl Overrun {
    #[must_use]
    pub const fn new(stop_on_overrun: bool) -> Self {
        Self {
            stop_on_overrun,
            reserved_tokens: 0,
            known_tokens: 0,
            overrun_requests: 0,
            overrun_tokens: 0,
            unknown_usage_requests: 0,
            refused_requests: 0,
            is_exact_ceiling: true,
        }
    }

    /// Account one dispatched request against its route's envelope.
    pub fn dispatched(&mut self, envelope_tokens: u64, usage: super::record::Usage) -> Result<(), Conflict> {
        increment(&mut self.reserved_tokens, envelope_tokens)?;
        match known_tokens(usage)? {
            Some(tokens) => {
                increment(&mut self.known_tokens, tokens)?;
                let beyond_tokens = tokens.saturating_sub(envelope_tokens);
                if beyond_tokens > 0 {
                    increment(&mut self.overrun_requests, 1)?;
                    increment(&mut self.overrun_tokens, beyond_tokens)?;
                }
            }
            None => increment(&mut self.unknown_usage_requests, 1)?,
        }
        self.is_exact_ceiling = self.unknown_usage_requests == 0 && self.overrun_requests == 0;
        debug_assert_eq!(self.overrun_requests == 0, self.overrun_tokens == 0);
        debug_assert!(self.unknown_usage_requests == 0 || !self.is_exact_ceiling);
        Ok(())
    }

    /// Account one recorded stop refusal.
    pub fn refused(&mut self) -> Result<(), Conflict> {
        increment(&mut self.refused_requests, 1)
    }

    #[must_use]
    pub const fn is_overrun(&self) -> bool {
        self.overrun_requests > 0
    }

    /// The stop policy refuses the next request.
    #[must_use]
    pub const fn must_stop(&self) -> bool {
        self.stop_on_overrun && self.is_overrun()
    }
}

/// The declared token envelope one request on `route` reserves.
#[must_use]
pub fn reserved_tokens(route: &super::Route) -> u64 {
    let requirements = route.requirements;
    u64::from(requirements.input_tokens).saturating_add(u64::from(requirements.output_tokens))
}

/// Whether a not-dispatched label is the attempt's own stop refusal.
#[must_use]
pub fn is_stop(label: &str) -> bool {
    label == BUDGET_EXHAUSTED || label == OVERRUN_STOPPED
}

/// The overrun block of a summary over its per-service tallies.
pub(super) fn summarize(
    declaration: &super::Declaration,
    services: &[super::usage::Tally],
) -> Result<Overrun, Conflict> {
    let mut overrun = Overrun::new(declaration.stop_on_overrun);
    for tally in services {
        let route = super::route(declaration, &tally.route_name);
        debug_assert!(route.is_some(), "every tally belongs to a served route");
        let envelope_tokens = route.map_or(0, reserved_tokens);
        for entry in &tally.requests {
            if entry.class != super::usage::EntryClass::NotDispatched {
                overrun.dispatched(envelope_tokens, entry.usage)?;
            } else if is_stop(&entry.outcome) {
                overrun.refused()?;
            }
        }
    }
    let not_dispatched =
        services.iter().fold(0_u64, |total, tally| total.saturating_add(tally.not_dispatched_requests));
    debug_assert!(overrun.refused_requests <= not_dispatched);
    Ok(overrun)
}

/// Both provider-reported counters, summed; `None` is unknown usage.
fn known_tokens(usage: super::record::Usage) -> Result<Option<u64>, Conflict> {
    let super::record::Usage {
        input_tokens: super::record::Counter::Known(input),
        output_tokens: super::record::Counter::Known(output),
        origin: super::record::UsageOrigin::ProviderReported,
    } = usage
    else {
        return Ok(None);
    };
    input.checked_add(output).map(Some).ok_or(Conflict::Overflow)
}

fn increment(total: &mut u64, amount: u64) -> Result<(), Conflict> {
    *total = total.checked_add(amount).ok_or(Conflict::Overflow)?;
    Ok(())
}
