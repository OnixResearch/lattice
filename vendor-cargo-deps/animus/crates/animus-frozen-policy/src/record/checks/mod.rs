//! A receipt body's bounds, internal consistency, and identities.

/// Every list within its bound, and every label and identity safe.
pub(super) fn check_bounds(body: &super::ReceiptBody) -> Result<(), super::ReceiptError> {
    let is_optional_digest = |value: &Option<alloc::string::String>| {
        value.as_deref().is_none_or(crate::digest::is_b3_identity)
    };
    let is_binding_safe = [
        &body.binding.artifact,
        &body.binding.feature_schema,
        &body.binding.action_schema,
    ]
    .iter()
    .all(|identity| identity.is_empty() || crate::digest::is_safe_identity(identity));
    let is_known_kind = body
        .policy_kind
        .as_deref()
        .is_none_or(|kind| [crate::TREE_POLICY_KIND, crate::MLP_POLICY_KIND].contains(&kind));
    let is_bounded = body.features.len() <= crate::MAX_FEATURES
        && body.eligible.len() <= crate::MAX_ACTIONS
        && body.scores.len() <= crate::MAX_ACTIONS
        && crate::digest::is_unique_label_set(
            body.eligible.iter().map(alloc::string::String::as_str),
        )
        && crate::digest::is_b3_identity(&body.input)
        && crate::digest::is_b3_identity(&body.action_set)
        && is_optional_digest(&body.artifact)
        && is_optional_digest(&body.model)
        && is_binding_safe
        && is_known_kind
        && body.runtime_version.len() <= crate::MAX_LABEL_BYTES
        && body
            .failure_detail
            .as_deref()
            .is_none_or(|detail| detail.len() <= crate::MAX_LABEL_BYTES);
    if !is_bounded {
        return Err(super::ReceiptError::Bounds);
    }
    debug_assert!(body.features.len() <= crate::MAX_FEATURES);
    debug_assert!(body.eligible.len() <= crate::MAX_ACTIONS);
    Ok(())
}

/// The outcome, answering action, score, failure, and fallback count agree.
pub(super) fn check_consistency(body: &super::ReceiptBody) -> Result<(), super::ReceiptError> {
    let is_eligible = body
        .action
        .as_ref()
        .is_some_and(|action| body.eligible.contains(action));
    let has_known_failure = body
        .failure
        .as_deref()
        .is_some_and(|code| crate::Failure::CODES.contains(&code));
    let is_consistent = match body.outcome {
        super::ReceiptOutcome::Selected => {
            is_eligible
                && body.score.is_some()
                && body.failure.is_none()
                && body.failure_detail.is_none()
                && body.artifact.is_some()
                && !body.scores.is_empty()
                && body.cost.fallback_decisions == 0
        }
        super::ReceiptOutcome::Fallback => {
            is_eligible
                && body.score.is_none()
                && has_known_failure
                && body.cost.fallback_decisions == 1
        }
        super::ReceiptOutcome::Refused => {
            body.action.is_none()
                && body.score.is_none()
                && has_known_failure
                && body.cost.fallback_decisions == 0
        }
    };
    if !is_consistent {
        return Err(super::ReceiptError::Inconsistent);
    }
    debug_assert!(body.action.is_some() || body.outcome == super::ReceiptOutcome::Refused);
    debug_assert!(body.failure.is_some() != (body.outcome == super::ReceiptOutcome::Selected));
    Ok(())
}

/// The canonical identity of `body` under the receipt schema.
pub(super) fn body_identity(body: &super::ReceiptBody) -> alloc::string::String {
    let canonical = serde_json::to_string(body).unwrap_or_default();
    crate::digest::domain_identity(super::RECEIPT_SCHEMA, &[&canonical])
}

/// The identity of `features` under `feature_schema`.
pub(super) fn input_identity(feature_schema: &str, features: &[i64]) -> alloc::string::String {
    let encoded = serde_json::to_string(features).unwrap_or_default();
    crate::digest::domain_identity(super::INPUT_DOMAIN, &[feature_schema, &encoded])
}

/// The identity of the eligible set under `action_schema`; label order does not change it.
pub(super) fn action_set_identity(
    action_schema: &str,
    eligible: &[alloc::string::String],
) -> alloc::string::String {
    let mut sorted: alloc::vec::Vec<&str> =
        eligible.iter().map(alloc::string::String::as_str).collect();
    sorted.sort_unstable();
    let encoded = serde_json::to_string(&sorted).unwrap_or_default();
    crate::digest::domain_identity(super::ACTION_SET_DOMAIN, &[action_schema, &encoded])
}
