//! Metadata-only provider retry receipt projection.

const BLAKE3_HEX_CHARS: usize = super::data::BLAKE3_HEX_CHARS;
const MAX_ATTEMPT_COUNT: u32 = super::data::MAX_ATTEMPT_COUNT;
const MAX_REASON_COUNT: usize = super::data::MAX_REASON_COUNT;
const NON_CLAIM: &str = super::data::NON_CLAIM;
const PROVIDER_RETRY_RECEIPT_HASH_DOMAIN: &str = super::data::PROVIDER_RETRY_RECEIPT_HASH_DOMAIN;
const PROVIDER_RETRY_RECEIPT_SCHEMA: &str = super::data::PROVIDER_RETRY_RECEIPT_SCHEMA;
const REDACTION_MARKER: &str = super::data::REDACTION_MARKER;
const FIELD_RECEIPT: super::core::RetryField = super::core::RetryField("provider_retry_receipt");

type ProviderRetryError = super::data::ProviderRetryError;
type ProviderRetryIssue = super::data::ProviderRetryIssue;
type ProviderRetryReceipt = super::data::ProviderRetryReceipt;
type ProviderRetryReceiptInput<'a> = super::data::ProviderRetryReceiptInput<'a>;
type ProviderRetrySensitiveDraft<'a> = super::data::ProviderRetrySensitiveDraft<'a>;

/// Projects a bounded metadata-only retry receipt.
///
/// # Errors
///
/// Returns [`ProviderRetryError`] for malformed counts/hashes, empty reason
/// evidence, or any supplied private provider/request/tool material.
// r[impl agent_harness_runtime.tau_provider_retry.receipts]
pub fn project_provider_retry_receipt(
    input: ProviderRetryReceiptInput<'_>,
) -> Result<ProviderRetryReceipt, ProviderRetryError> {
    validate_receipt_input(&input)?;
    reject_sensitive(input.sensitive)?;
    let timer_label = input
        .timer_outcome
        .map_or("none", super::data::ProviderRetryTimerOutcome::as_str);
    let mut parts = Vec::with_capacity(receipt_hash_part_capacity(input.reason_classes.len()));
    parts.extend([
        PROVIDER_RETRY_RECEIPT_SCHEMA.to_string(),
        input.policy_hash.to_string(),
        input.exact_plan_hash.to_string(),
        input.completed_attempt_count.to_string(),
        timer_label.to_string(),
        input.status.as_str().to_string(),
        REDACTION_MARKER.to_string(),
        NON_CLAIM.to_string(),
    ]);
    parts.extend(
        input
            .reason_classes
            .iter()
            .map(|reason| reason.as_str().to_string()),
    );
    let receipt_hash =
        super::core::hash_owned_parts(PROVIDER_RETRY_RECEIPT_HASH_DOMAIN, parts.as_slice());
    debug_assert_eq!(
        parts.len(),
        receipt_hash_part_capacity(input.reason_classes.len())
    );
    debug_assert_eq!(receipt_hash.len(), BLAKE3_HEX_CHARS);
    Ok(ProviderRetryReceipt {
        schema: PROVIDER_RETRY_RECEIPT_SCHEMA,
        policy_hash: input.policy_hash.to_string(),
        exact_plan_hash: input.exact_plan_hash.to_string(),
        completed_attempt_count: input.completed_attempt_count,
        reason_classes: input.reason_classes.to_vec(),
        timer_outcome: input.timer_outcome,
        status: input.status,
        redaction_marker: REDACTION_MARKER,
        non_claim: NON_CLAIM,
        receipt_hash,
    })
}

fn validate_receipt_input(input: &ProviderRetryReceiptInput<'_>) -> Result<(), ProviderRetryError> {
    super::core::lowercase_blake3(input.policy_hash, FIELD_RECEIPT)?;
    super::core::lowercase_blake3(input.exact_plan_hash, FIELD_RECEIPT)?;
    let is_invalid_attempt_count = input.completed_attempt_count == u32::default()
        || input.completed_attempt_count > MAX_ATTEMPT_COUNT;
    if is_invalid_attempt_count {
        return Err(super::core::error(
            ProviderRetryIssue::InvalidInput,
            FIELD_RECEIPT,
        ));
    }
    if input.reason_classes.is_empty() {
        return Err(super::core::error(
            ProviderRetryIssue::InvalidInput,
            FIELD_RECEIPT,
        ));
    }
    if input.reason_classes.len() > MAX_REASON_COUNT {
        return Err(super::core::error(
            ProviderRetryIssue::InvalidInput,
            FIELD_RECEIPT,
        ));
    }
    debug_assert!(input.completed_attempt_count <= MAX_ATTEMPT_COUNT);
    debug_assert!(input.reason_classes.len() <= MAX_REASON_COUNT);
    Ok(())
}

fn reject_sensitive(sensitive: ProviderRetrySensitiveDraft<'_>) -> Result<(), ProviderRetryError> {
    let has_private_material = [
        sensitive.raw_request,
        sensitive.response_body,
        sensitive.response_headers,
        sensitive.endpoint,
        sensitive.credential,
        sensitive.exception_body,
        sensitive.prompt,
        sensitive.reasoning,
        sensitive.tool_body,
        sensitive.private_path,
    ]
    .into_iter()
    .any(|value| value.is_some());
    if has_private_material {
        return Err(super::core::error(
            ProviderRetryIssue::ReceiptLeak,
            FIELD_RECEIPT,
        ));
    }
    debug_assert!(!has_private_material);
    debug_assert!(sensitive.raw_request.is_none());
    Ok(())
}

fn receipt_hash_part_capacity(reason_count: usize) -> usize {
    const FIXED_RECEIPT_PART_COUNT: usize = 8;
    FIXED_RECEIPT_PART_COUNT.saturating_add(reason_count)
}
