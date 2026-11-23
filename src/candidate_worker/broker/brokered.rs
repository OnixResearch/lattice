//! The recorded route of a binding without a serving declaration: the
//! broker's own checks decide, and every admitted request leaves a synced
//! intent before upstream contact and a terminal record after it.

// r[impl lattice.serving_binding.brokered_evidence]
// r[impl lattice.serving_binding.brokered_evidence.scenario.unattributed]

type Outcome = lattice_core::execution_binding::serving::record::Outcome;
type OutcomeClass = lattice_core::execution_binding::serving::record::OutcomeClass;
type Usage = lattice_core::execution_binding::serving::record::Usage;

impl super::ModelRequestGate for super::RecordedRoute<'_> {
    /// Only a request whose fresh intent is durably created reaches upstream.
    /// An index at or below one this route admitted, an intent that already
    /// exists (identical bytes included) or one that cannot be written
    /// refuses the request before any effect, so no index reaches upstream
    /// twice and identical records never hide a second contact.
    fn admit(
        &mut self,
        checks: crate::execution_binding::serving::gate::Checks,
        request_index: u32,
    ) -> Option<super::GateTicket> {
        let is_fresh = self.highest.is_none_or(|highest| request_index > highest);
        if !super::is_admitted(checks) || !is_fresh {
            return None;
        }
        let intent = self.context.intent(u64::from(request_index));
        self.evidence.create_intent(&intent).ok()?;
        self.highest = Some(request_index);
        debug_assert_eq!(intent.request_index, u64::from(request_index));
        Some(super::GateTicket(super::Permit::Recorded(Box::new(intent))))
    }

    /// The response reaches the worker exactly as it would without evidence;
    /// a record that cannot be retained fails the broker.
    fn settle(
        &mut self,
        ticket: super::GateTicket,
        upstream: crate::execution_binding::serving::gate::Upstream,
    ) -> Result<bool, String> {
        let super::Permit::Recorded(intent) = ticket.0 else {
            return Err("recorded route settled without its intent".into());
        };
        let (outcome, usage, provider_receipt) = terminal(upstream);
        let record = lattice_core::execution_binding::serving::brokered::Record::settled(
            &intent,
            outcome,
            usage,
            provider_receipt,
        );
        self.evidence.write_record(&record)?;
        debug_assert_eq!(record.request_id, intent.request_id);
        Ok(true)
    }

    /// A budget refusal happens before any intent: nothing is dispatched.
    fn exhausted(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// The outcome, provider-reported counters and provider receipt of one call.
/// Counters are known only when both were reported; a receipt that is not an
/// opaque producer token is dropped, never truncated.
fn terminal(upstream: crate::execution_binding::serving::gate::Upstream) -> (Outcome, Usage, Option<String>) {
    let uncertain = |label: &str| Outcome {
        class: OutcomeClass::Uncertain,
        label: label.to_owned(),
        detail: None,
    };
    match upstream {
        crate::execution_binding::serving::gate::Upstream::Responded(facts) => {
            let usage = match (facts.input_tokens, facts.output_tokens) {
                (Some(input), Some(output)) => Usage {
                    input_tokens: lattice_core::execution_binding::serving::record::Counter::Known(input),
                    output_tokens: lattice_core::execution_binding::serving::record::Counter::Known(output),
                    origin: lattice_core::execution_binding::serving::record::UsageOrigin::ProviderReported,
                },
                _ => Usage::UNKNOWN,
            };
            let receipt = facts
                .provider_receipt
                .filter(|receipt| lattice_core::execution_binding::serving::is_producer_token(receipt));
            let completed = Outcome {
                class: OutcomeClass::Completed,
                label: "completed".to_owned(),
                detail: None,
            };
            (completed, usage, receipt)
        }
        crate::execution_binding::serving::gate::Upstream::TimedOut => {
            (uncertain("uncertain-timed-out"), Usage::UNKNOWN, None)
        }
        crate::execution_binding::serving::gate::Upstream::TransportFailed => {
            (uncertain("uncertain-transport-failed"), Usage::UNKNOWN, None)
        }
        crate::execution_binding::serving::gate::Upstream::Cancelled => {
            (uncertain("uncertain-cancelled"), Usage::UNKNOWN, None)
        }
    }
}
