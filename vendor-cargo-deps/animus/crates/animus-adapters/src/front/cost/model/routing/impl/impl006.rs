fn reject_receipt_leaks(draft: &ModelRoutingReceiptDraft<'_>) -> Result<(), ModelRoutingError> {
    if draft.raw_prompt.is_some() {
        return Err(receipt_leak(ModelRoutingReceiptLeak::RawPrompt));
    }
    if draft.provider_payload.is_some() {
        return Err(receipt_leak(ModelRoutingReceiptLeak::ProviderPayload));
    }
    if draft.credential.is_some() {
        return Err(receipt_leak(ModelRoutingReceiptLeak::Credential));
    }
    if draft.pricing_store_dump.is_some() {
        return Err(receipt_leak(ModelRoutingReceiptLeak::PricingStoreDump));
    }
    if draft.hidden_reasoning.is_some() {
        return Err(receipt_leak(ModelRoutingReceiptLeak::HiddenReasoning));
    }
    Ok(())
}

const fn receipt_leak(leak: ModelRoutingReceiptLeak) -> ModelRoutingError {
    ModelRoutingError::ReceiptLeak { leak }
}

fn routing_receipt_hash(decision: &ModelRoutingDecision) -> crate::artifact::Hash {
    let mut canonical = String::from(CANONICAL_VERSION_PREFIX);
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(decision.status().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(decision.selected_role_label().unwrap_or("none"));
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(decision.selected_model_label().unwrap_or("none"));
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(decision.score_bucket().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(decision.budget_status().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(&decision.candidate_count().to_string());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(
        decision
            .fallback_class()
            .map_or("none", RoutingFallbackClass::as_str),
    );
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(decision.policy_hash().as_str());
    canonical.push_str(CANONICAL_SEPARATOR);
    canonical.push_str(if decision.provider_request_sent() {
        "provider-request-sent"
    } else {
        "no-provider-request"
    });
    let digest = blake3::hash(canonical.as_bytes());
    crate::artifact::Hash::from_bytes(digest.as_bytes())
}

/// Routing receipt leak class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRoutingReceiptLeak {
    /// Raw prompt text was present.
    RawPrompt,
    /// Provider payload was present.
    ProviderPayload,
    /// Credential-looking value was present.
    Credential,
    /// Full pricing-store dump was present.
    PricingStoreDump,
    /// Hidden reasoning text was present.
    HiddenReasoning,
}

/// Routing policy validation issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelRoutingPolicyIssue {
    /// Schema version was zero.
    ZeroSchemaVersion,
    /// No roles were configured.
    EmptyRoles,
    /// Duplicate role label appeared.
    DuplicateRole,
    /// Negative complexity weight appeared.
    NegativeWeight {
        /// Field containing the negative weight.
        field: &'static str,
    },
    /// Threshold ordering was invalid.
    UnorderedThresholds,
    /// Downgrade target did not reference a configured role.
    UnknownDowngradeTarget,
    /// Missing-price behavior was unspecified.
    MissingPriceBehaviorUnspecified,
    /// Hard budget was lower than soft budget.
    UnorderedBudgets,
}

/// Model routing error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelRoutingError {
    /// A safe text field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 hash failed validation.
    InvalidHash(crate::artifact::Error),
    /// Policy validation failed.
    Policy(ModelRoutingPolicyIssue),
    /// A bounded item list exceeded the allowed item count.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum supported items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// Receipt contained forbidden sensitive material.
    ReceiptLeak {
        /// Leak class.
        leak: ModelRoutingReceiptLeak,
    },
}

impl core::fmt::Display for ModelRoutingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidHash(error) => write!(formatter, "{error}"),
            Self::Policy(issue) => write!(formatter, "model routing policy issue: {issue:?}"),
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} supports at most {max_items} items; got {actual_items}"
            ),
            Self::ReceiptLeak { leak } => write!(formatter, "model routing receipt leak: {leak:?}"),
        }
    }
}

impl From<crate::AdapterContractError> for ModelRoutingError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

fn validate_item_count(
    field: &'static str,
    actual_items: usize,
    max_items: usize,
) -> Result<(), ModelRoutingError> {
    if actual_items > max_items {
        return Err(ModelRoutingError::TooManyItems {
            field,
            max_items,
            actual_items,
        });
    }
    Ok(())
}

fn normalize(
    field: crate::validation::FieldName,
    value: &str,
) -> Result<String, ModelRoutingError> {
    crate::validation::normalized_adapter_field(field, value).map_err(ModelRoutingError::Contract)
}

fn normalize_optional(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<String>, ModelRoutingError> {
    value.map(|inner| normalize(field, inner)).transpose()
}

