//! The published result or non-authoritative diagnostic document.
//!
//! Producer counters stay JSON numbers until this boundary and are published
//! as canonical decimal strings with their own names and units.

const COMPUTATION_DOMAIN: &str = "lattice.pure-inet-computation.v1";

/// The normalized evaluation label published in the document.
pub fn evaluation_label(outcome: crate::pure_inet_profile::PureEvaluationOutcome) -> &'static str {
    match outcome {
        crate::pure_inet_profile::PureEvaluationOutcome::Complete { .. } => "complete",
        crate::pure_inet_profile::PureEvaluationOutcome::Partial => "partial",
        crate::pure_inet_profile::PureEvaluationOutcome::Invalid => "invalid",
        crate::pure_inet_profile::PureEvaluationOutcome::Exhausted => "exhausted",
        crate::pure_inet_profile::PureEvaluationOutcome::Failed => "failed",
        crate::pure_inet_profile::PureEvaluationOutcome::Cancelled => "cancelled",
        crate::pure_inet_profile::PureEvaluationOutcome::Unknown => "unknown",
    }
}

/// Build the published document; only a complete evaluation is authoritative.
pub fn build(
    plan: &crate::operation::InvocationPlan,
    request: &crate::inet::route::arguments::Request,
    normalized: &crate::inet::report::Normalized,
) -> Result<serde_json::Value, crate::inet::Error> {
    let computation = computation_identity(plan, request)?;
    let mut binding = serde_json::json!({
        "invocation": plan.identity,
        "computation": computation,
        "attempt": request.attempt,
        "program_identity": request.program_identity,
        "inputs_identity": request.inputs_identity,
        "executable_identity": plan.runtime.executor_identity,
        "producer_revision": crate::inet::report::PRODUCER_SOURCE_REVISION,
        "net_contract": crate::inet::report::NET_CONTRACT[0].1,
        "rule_table": crate::inet::report::NET_CONTRACT[2].1,
        "output_contract_identity": plan.output_contract_identity,
        "profile_identity": plan.type_revision,
    });
    for (key, value) in [
        ("predecessor", &request.predecessor),
        ("consumer_binding", &request.consumer_binding),
    ] {
        if let Some(value) = value {
            binding[key] = serde_json::json!(value);
        }
    }
    let ceilings = request.ceilings;
    let mut document = serde_json::json!({
        "schema": crate::inet::route::RESULT_SCHEMA,
        "evaluation": evaluation_label(normalized.outcome),
        "reason": normalized.reason,
        "authoritative": normalized.completion.is_some(),
        "binding": binding,
        "limits": {
            "fuel_interactions": ceilings.fuel_interactions.to_string(),
            "max_input_bytes": ceilings.max_input_bytes.to_string(),
            "max_retained_bytes": ceilings.max_retained_bytes.to_string(),
            "max_output_bytes": ceilings.max_output_bytes.to_string(),
            "deadline_seconds": plan.bounds.timeout_seconds.to_string(),
        },
    });
    debug_assert_eq!(normalized.completion.is_some(), normalized.outcome.usable_result().is_some());
    if let Some(completion) = &normalized.completion {
        document["binding"]["graph_identity"] = serde_json::json!(completion.graph_identity);
        document["result"] = serde_json::json!(crate::pure_inet_profile::format_canonical_i64(completion.result));
        document["counters"] = counters(&completion.counters);
        document["semantic_interactions"] = serde_json::to_value(&completion.interactions)
            .map_err(|_| crate::inet::Error::Core(crate::pure_inet_profile::PureProfileError::InvalidSchema))?;
    }
    if let Some(exhaustion) = &normalized.exhaustion {
        document["exhaustion"] = serde_json::json!({
            "limit": exhaustion.limit,
            "cap": exhaustion.cap.to_string(),
            "observed": exhaustion.observed.to_string(),
        });
    }
    debug_assert!(document["authoritative"] == normalized.completion.is_some());
    Ok(document)
}

fn counters(counters: &crate::inet::report::Counters) -> serde_json::Value {
    serde_json::json!({
        "interactions": counters.interactions.to_string(),
        "expected_interactions": counters.expected_interactions.to_string(),
        "peak_live_agents": counters.peak_live_agents.to_string(),
        "final_wires": counters.final_wires.to_string(),
        "retained_bytes_estimate": counters.retained_bytes.to_string(),
        "retained_bytes_high_water": counters.retained_bytes_high_water.to_string(),
    })
}

/// The identity of the immutable computation: program, inputs, producer,
/// executable, contract, profile and output contract, but not the attempt
/// or the lowered ceilings.
pub fn computation_identity(
    plan: &crate::operation::InvocationPlan,
    request: &crate::inet::route::arguments::Request,
) -> Result<String, crate::inet::Error> {
    let material = serde_json::json!({
        "program_identity": request.program_identity,
        "inputs_identity": request.inputs_identity,
        "executable_identity": plan.runtime.executor_identity,
        "producer_revision": crate::inet::report::PRODUCER_SOURCE_REVISION,
        "net_contract": crate::inet::report::NET_CONTRACT,
        "profile_identity": plan.type_revision,
        "output_contract_identity": plan.output_contract_identity,
    });
    crate::operation::identity::hash_value(COMPUTATION_DOMAIN, &material)
        .map_err(|_| crate::inet::Error::Core(crate::pure_inet_profile::PureProfileError::InvalidSchema))
}
