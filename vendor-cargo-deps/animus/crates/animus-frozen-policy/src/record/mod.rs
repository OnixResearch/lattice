//! Bounded decision receipts and their re-admission for replay without inference.
//!
//! A receipt's body binds the requested and admitted identities, the actual input and output,
//! the outcome, and deterministic cost units. Its identity is the domain-separated `b3:`
//! identity of the body's compact encoding, whose keys are declared in sorted order.

mod checks;

/// Schema of a decision receipt, and the identity domain of its body.
pub const RECEIPT_SCHEMA: &str = "animus.frozen-policy-receipt.v1";
/// Identity domain of a decision's input: the feature schema and the raw feature values.
pub const INPUT_DOMAIN: &str = "animus.frozen-policy-input.v1";
/// Identity domain of a decision's eligible set: the action schema and the sorted labels.
pub const ACTION_SET_DOMAIN: &str = "animus.frozen-policy-action-set.v1";
/// Largest receipt encoding [`verify_receipt`] admits, in bytes.
pub const MAX_RECEIPT_BYTES: usize = 16_384;

/// Deterministic cost units of one decision.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CostUnits {
    /// Whether the declared fallback answered: 1 or 0.
    pub fallback_decisions: u64,
    /// Weight-input products summed, or tree splits visited.
    pub inference_operations: u64,
    /// Feature values normalized.
    pub normalization_operations: u64,
}

/// The decided outcome a receipt records.
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReceiptOutcome {
    /// The policy selected an eligible action.
    Selected,
    /// The consumer's declared fallback answered.
    Fallback,
    /// No action answered.
    Refused,
}

/// The body of a decision receipt, keys in sorted order.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptBody {
    /// The answering action: the selection or the fallback.
    pub action: Option<alloc::string::String>,
    /// Identity of the eligible set under the bound action schema.
    pub action_set: alloc::string::String,
    /// The admitted artifact identity, if one was admitted.
    pub artifact: Option<alloc::string::String>,
    /// The artifact API of this runtime.
    pub artifact_api: u32,
    /// What the consumer's profile pinned; empty identities when the binding itself was unsafe.
    pub binding: crate::PolicyBinding,
    /// Deterministic cost units.
    pub cost: CostUnits,
    /// The eligible labels as requested; empty when the request's shape was refused.
    pub eligible: alloc::vec::Vec<alloc::string::String>,
    /// The failure code, if the policy did not select.
    pub failure: Option<alloc::string::String>,
    /// The failure's bounded detail.
    pub failure_detail: Option<alloc::string::String>,
    /// The raw features as requested; empty when the request's shape was refused.
    pub features: alloc::vec::Vec<i64>,
    /// Identity of the input under the bound feature schema.
    pub input: alloc::string::String,
    /// The admitted model identity (weights digest), if one was admitted.
    pub model: Option<alloc::string::String>,
    /// The decided outcome.
    pub outcome: ReceiptOutcome,
    /// The admitted policy kind, if one was admitted.
    pub policy_kind: Option<alloc::string::String>,
    /// The runtime crate that decided.
    pub runtime_crate: alloc::string::String,
    /// The runtime crate's version.
    pub runtime_version: alloc::string::String,
    /// The receipt schema.
    pub schema: alloc::string::String,
    /// The selected action's score, if selected.
    pub score: Option<i64>,
    /// One score per declared action, in declared order; empty unless the model ran.
    pub scores: alloc::vec::Vec<i64>,
}

/// A decision receipt: its body and the body's `b3:` identity.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionReceipt {
    /// What the decision bound and answered.
    pub body: ReceiptBody,
    /// The body's canonical `b3:` identity.
    pub identity: alloc::string::String,
}

impl DecisionReceipt {
    /// The receipt's compact canonical encoding, for durable retention.
    #[must_use]
    pub fn encode(&self) -> alloc::string::String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Why a retained receipt was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptError {
    /// The encoding is empty or exceeds [`MAX_RECEIPT_BYTES`].
    Size,
    /// The bytes are not one receipt document, or hold unknown fields.
    Malformed,
    /// Another receipt schema or runtime.
    Schema,
    /// A list exceeds its bound, or a label or identity is unsafe.
    Bounds,
    /// The outcome, action, score, and failure disagree.
    Inconsistent,
    /// The identity differs from the body's canonical identity.
    Identity,
}

/// Issues the receipt of one decision.
// r[impl agent_harness_runtime.frozen_policy.receipt]
pub(crate) fn issue(
    request: &crate::DecisionRequest,
    attempt: crate::selection::flow::Attempt,
    outcome: &crate::Outcome,
    is_shaped: bool,
) -> DecisionReceipt {
    let (kind, action, score, failure) = match outcome {
        crate::Outcome::Selected { action, score } => (
            ReceiptOutcome::Selected,
            Some(action.clone()),
            Some(*score),
            None,
        ),
        crate::Outcome::Fallback { action, failure } => (
            ReceiptOutcome::Fallback,
            Some(action.clone()),
            None,
            Some(*failure),
        ),
        crate::Outcome::Refused { failure } => {
            (ReceiptOutcome::Refused, None, None, Some(*failure))
        }
    };
    let mut cost = attempt.cost;
    cost.fallback_decisions = u64::from(kind == ReceiptOutcome::Fallback);
    let is_binding_unsafe =
        failure == Some(crate::Failure::MalformedInput(crate::InputFault::Binding));
    let (artifact, model, policy_kind) = attempt.policy.map_or((None, None, None), |facts| {
        (
            Some(facts.artifact),
            Some(facts.model),
            Some(alloc::string::String::from(facts.policy_kind)),
        )
    });
    let body = ReceiptBody {
        action,
        action_set: checks::action_set_identity(&request.binding.action_schema, &request.eligible),
        artifact,
        artifact_api: crate::ARTIFACT_API,
        binding: if is_binding_unsafe {
            crate::PolicyBinding::default()
        } else {
            request.binding.clone()
        },
        cost,
        eligible: if is_shaped {
            request.eligible.clone()
        } else {
            alloc::vec::Vec::new()
        },
        failure: failure.map(|failure| alloc::string::String::from(failure.code())),
        failure_detail: failure.and_then(crate::Failure::detail),
        features: if is_shaped {
            request.features.clone()
        } else {
            alloc::vec::Vec::new()
        },
        input: checks::input_identity(&request.binding.feature_schema, &request.features),
        model,
        outcome: kind,
        policy_kind,
        runtime_crate: alloc::string::String::from(crate::RUNTIME_CRATE),
        runtime_version: alloc::string::String::from(env!("CARGO_PKG_VERSION")),
        schema: alloc::string::String::from(RECEIPT_SCHEMA),
        score,
        scores: attempt.scores,
    };
    debug_assert!(checks::check_consistency(&body).is_ok());
    let identity = checks::body_identity(&body);
    DecisionReceipt { body, identity }
}

/// Re-admits a retained receipt and returns it, so replay uses the recorded outcome and never
/// re-runs inference.
///
/// # Errors
///
/// Returns the first [`ReceiptError`] that refuses the receipt.
// r[impl agent_harness_runtime.frozen_policy.receipt]
pub fn verify_receipt(bytes: &[u8]) -> Result<DecisionReceipt, ReceiptError> {
    if bytes.is_empty() || bytes.len() > MAX_RECEIPT_BYTES {
        return Err(ReceiptError::Size);
    }
    let receipt: DecisionReceipt =
        serde_json::from_slice(bytes).map_err(|_| ReceiptError::Malformed)?;
    let body = &receipt.body;
    let is_this_runtime = body.schema == RECEIPT_SCHEMA
        && body.runtime_crate == crate::RUNTIME_CRATE
        && body.artifact_api == crate::ARTIFACT_API;
    if !is_this_runtime {
        return Err(ReceiptError::Schema);
    }
    checks::check_bounds(body)?;
    checks::check_consistency(body)?;
    if checks::body_identity(body) != receipt.identity {
        return Err(ReceiptError::Identity);
    }
    debug_assert!(crate::digest::is_b3_identity(&receipt.identity));
    Ok(receipt)
}
