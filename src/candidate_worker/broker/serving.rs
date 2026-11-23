//! The serving-owner gate in the broker and upstream response facts.

// r[impl lattice.serving_binding.request_admission]

impl super::ModelRequestGate for super::OwnerGate<'_> {
    /// Only an admitted producer verdict with a durable intent dispatches.
    fn admit(&mut self, checks: crate::execution_binding::serving::gate::Checks, _: u32) -> Option<super::GateTicket> {
        match self.session.admit(&self.route_name, checks) {
            Ok(crate::execution_binding::serving::gate::Decision::Dispatch(ticket)) => {
                Some(super::GateTicket(super::Permit::Served(ticket)))
            }
            Ok(crate::execution_binding::serving::gate::Decision::Refused(_)) | Err(_) => None,
        }
    }

    /// A rejected response (substituted or missing model) is withheld.
    fn settle(
        &mut self,
        ticket: super::GateTicket,
        upstream: crate::execution_binding::serving::gate::Upstream,
    ) -> Result<bool, String> {
        let super::Permit::Served(ticket) = ticket.0 else {
            return Err("served route settled without a serving ticket".into());
        };
        let record = self.session.settle(*ticket, upstream)?;
        Ok(record.outcome.class == lattice_core::execution_binding::serving::record::OutcomeClass::Completed)
    }

    /// The broker's exhausted request budget is recorded once, without a port call.
    fn exhausted(&mut self) -> Result<(), String> {
        self.session.refuse_budget_exhausted(&self.route_name).map(|_| ())
    }
}

/// Model, provider usage and response identity reported by an OpenAI-style
/// chat-completion body or its server-sent event stream. Absent facts stay
/// absent; nothing is inferred from the request.
pub(crate) fn response_facts(body: &str) -> crate::execution_binding::serving::gate::ResponseFacts {
    let mut facts = crate::execution_binding::serving::gate::ResponseFacts {
        model: None,
        input_tokens: None,
        output_tokens: None,
        provider_receipt: None,
    };
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        absorb(&mut facts, &value);
        return facts;
    }
    for line in body.lines() {
        let Some(data) = line.strip_prefix("data:").map(str::trim) else {
            continue;
        };
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
            absorb(&mut facts, &value);
        }
    }
    debug_assert!(facts.input_tokens.is_none() || facts.output_tokens.is_some() || !body.is_empty());
    facts
}

/// The first model and response id win; the last reported usage wins.
fn absorb(facts: &mut crate::execution_binding::serving::gate::ResponseFacts, value: &serde_json::Value) {
    if facts.model.is_none() {
        facts.model = value.get("model").and_then(serde_json::Value::as_str).map(str::to_owned);
    }
    if facts.provider_receipt.is_none() {
        facts.provider_receipt = value.get("id").and_then(serde_json::Value::as_str).map(str::to_owned);
    }
    if let Some(usage) = value.get("usage").filter(|usage| usage.is_object()) {
        facts.input_tokens = usage.get("prompt_tokens").and_then(serde_json::Value::as_u64);
        facts.output_tokens = usage.get("completion_tokens").and_then(serde_json::Value::as_u64);
    }
}
