//! Metadata-only authority-profile receipt projection.

#![allow(missing_docs)]

#[allow(unused_imports)]
use crate::prelude::*;

const FIELD_RECEIPT: &str = "receipt";
const RECEIPT_HASH_HEX_BYTES: usize = 64;
const SENSITIVE_FIELD_COUNT: usize = 9;

/// Projects one bounded authority-profile receipt.
///
/// # Errors
///
/// Returns [`super::data::SessionAuthorityError`] when identities are invalid or
/// sensitive material is supplied.
pub fn project_session_authority_receipt(
    draft: super::data::SessionAuthorityReceiptDraft<'_>,
) -> Result<super::data::SessionAuthorityReceipt, super::data::SessionAuthorityError> {
    reject_sensitive(draft.sensitive)?;
    super::core::validate_safe_label(draft.session_id)?;
    super::core::validate_safe_label(draft.branch_id)?;
    super::core::validate_hash(draft.head_hash)?;
    super::core::validate_hash(draft.comparison.current_profile_identity())?;
    if let Some(identity) = draft.comparison.committed_profile_identity() {
        super::core::validate_hash(identity)?;
    }
    if let Some(binding) = draft.binding {
        super::core::validate_hash(binding.binding_identity())?;
    }
    if let Some(enforcement) = draft.enforcement {
        super::core::validate_hash(enforcement.observation_identity())?;
    }
    let changed_components = draft
        .comparison
        .changed_components()
        .iter()
        .map(|class| String::from(class.as_str()))
        .collect::<Vec<_>>();
    let non_claims = super::data::NON_CLAIMS
        .iter()
        .map(|value| String::from(*value))
        .collect::<Vec<_>>();
    let receipt_identity =
        receipt_identity(draft, changed_components.as_slice(), non_claims.as_slice());
    let receipt = super::data::SessionAuthorityReceipt {
        schema: super::data::SESSION_AUTHORITY_RECEIPT_SCHEMA,
        status: draft.comparison.status(),
        session_id: String::from(draft.session_id),
        branch_id: String::from(draft.branch_id),
        head_hash: String::from(draft.head_hash),
        committed_profile_identity: draft
            .comparison
            .committed_profile_identity()
            .map(String::from),
        current_profile_identity: String::from(draft.comparison.current_profile_identity()),
        binding_identity: draft
            .binding
            .map(super::data::SessionAuthorityBinding::binding_identity)
            .map(String::from),
        enforcement_observation_identity: draft
            .enforcement
            .map(super::data::SessionAuthorityEnforcementObservation::observation_identity)
            .map(String::from),
        changed_components,
        redaction_marker: super::data::REDACTION_MARKER,
        non_claims,
        receipt_identity,
    };
    debug_assert_eq!(receipt.receipt_identity.len(), RECEIPT_HASH_HEX_BYTES);
    debug_assert_eq!(
        receipt.current_profile_identity,
        draft.comparison.current_profile_identity()
    );
    Ok(receipt)
}

fn reject_sensitive(
    sensitive: super::data::SessionAuthoritySensitiveDraft<'_>,
) -> Result<(), super::data::SessionAuthorityError> {
    let sensitive_values = [
        sensitive.raw_policy,
        sensitive.prompt,
        sensitive.tool_schema,
        sensitive.extension_body,
        sensitive.credential,
        sensitive.environment_value,
        sensitive.host_path,
        sensitive.provider_payload,
        sensitive.raw_diagnostic,
    ];
    let sensitive_count_items = sensitive_values
        .iter()
        .filter(|value| value.is_some())
        .count();
    let has_sensitive_data = sensitive_count_items > 0;
    debug_assert_eq!(sensitive_values.len(), SENSITIVE_FIELD_COUNT);
    debug_assert!(sensitive_count_items <= SENSITIVE_FIELD_COUNT);
    if has_sensitive_data {
        Err(super::core::error(
            super::data::SessionAuthorityIssue::UnsafeReceipt,
            FIELD_RECEIPT,
        ))
    } else {
        Ok(())
    }
}

fn receipt_identity(
    draft: super::data::SessionAuthorityReceiptDraft<'_>,
    changed_components: &[String],
    non_claims: &[String],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(super::data::SESSION_AUTHORITY_RECEIPT_HASH_DOMAIN.as_bytes());
    let owned = [
        super::data::SESSION_AUTHORITY_RECEIPT_SCHEMA.to_string(),
        draft.comparison.status().as_str().to_string(),
        draft.session_id.to_string(),
        draft.branch_id.to_string(),
        draft.head_hash.to_string(),
        draft
            .comparison
            .committed_profile_identity()
            .unwrap_or(super::data::ABSENT_IDENTITY)
            .to_string(),
        draft.comparison.current_profile_identity().to_string(),
        draft
            .binding
            .map_or(
                super::data::ABSENT_IDENTITY,
                super::data::SessionAuthorityBinding::binding_identity,
            )
            .to_string(),
        draft
            .enforcement
            .map_or(
                super::data::ABSENT_IDENTITY,
                super::data::SessionAuthorityEnforcementObservation::observation_identity,
            )
            .to_string(),
        super::data::REDACTION_MARKER.to_string(),
    ];
    for part in owned
        .iter()
        .chain(changed_components.iter())
        .chain(non_claims.iter())
    {
        hasher.update(super::data::HASH_SEPARATOR);
        hasher.update(part.as_bytes());
    }
    let identity = hasher.finalize().to_hex().to_string();
    debug_assert_eq!(identity.len(), RECEIPT_HASH_HEX_BYTES);
    debug_assert!(!owned.is_empty());
    identity
}
