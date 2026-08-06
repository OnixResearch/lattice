#[cfg(test)]
mod tests;

/// Evaluates exec policy and returns a redacted sandbox-escalation event.
#[must_use]
pub fn evaluate_sandbox_escalation(request: animus_adapters::ExecPolicyRequest<'_>) -> crate::RuntimeEvent {
    let evaluation = animus_adapters::evaluate_exec_policy(request);
    sandbox_escalation_event(&evaluation)
}

/// Builds a redacted sandbox-escalation event from a policy evaluation.
#[must_use]
pub fn sandbox_escalation_event(evaluation: &animus_adapters::ExecPolicyEvaluation) -> crate::RuntimeEvent {
    crate::RuntimeEvent::SandboxEscalation {
        decision: evaluation.decision(),
        rule_ids: rule_ids(evaluation),
        command_fingerprints: command_fingerprints(evaluation),
        active_sandbox: String::from(evaluation.sandbox().active_sandbox()),
        requested_capability: String::from(evaluation.sandbox().requested_capability()),
        escalation_reason: String::from(evaluation.sandbox().escalation_reason()),
        redaction_marker: String::from(evaluation.redaction_marker()),
    }
}

fn rule_ids(evaluation: &animus_adapters::ExecPolicyEvaluation) -> Vec<String> {
    evaluation.matched_rules().iter().map(|evidence| String::from(evidence.rule_id())).collect()
}

fn command_fingerprints(evaluation: &animus_adapters::ExecPolicyEvaluation) -> Vec<String> {
    evaluation
        .command_fingerprints()
        .iter()
        .map(|fingerprint| String::from(fingerprint.as_str()))
        .collect()
}
