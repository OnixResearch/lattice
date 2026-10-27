//! Metadata-only runtime-authority seal receipt projection.

#![allow(missing_docs)]

#[allow(unused_imports)]
use crate::prelude::*;

const FIELD_RECEIPT: &str = "receipt";
const SENSITIVE_FIELD_COUNT: usize = 9;

/// Project a metadata-only pre-effect receipt for one prepared unit.
///
/// # Errors
///
/// Returns issues when the draft contains sensitive data, overclaims, or an invalid state.
pub fn project_runtime_authority_seal_receipt(
    unit: &super::data::RuntimeAuthorityUnit,
    draft: super::data::RuntimeAuthorityReceiptDraft<'_>,
) -> Result<super::data::RuntimeAuthoritySealReceipt, super::data::RuntimeAuthoritySealError> {
    reject_sensitive(draft.sensitive)?;
    if !draft.semantic_claims.is_empty() {
        return Err(error(
            super::data::RuntimeAuthorityIssueCode::AuthorityOverclaim,
        ));
    }
    if unit.state() != super::data::RuntimeAuthorityUnitState::Preparing {
        return Err(error(
            super::data::RuntimeAuthorityIssueCode::InvalidStateTransition,
        ));
    }
    let non_claims = super::data::NON_CLAIMS
        .iter()
        .map(|value| String::from(*value))
        .collect::<Vec<_>>();
    let mut receipt = super::data::RuntimeAuthoritySealReceipt {
        schema: super::data::RUNTIME_AUTHORITY_SEAL_RECEIPT_SCHEMA,
        unit_identity: String::from(unit.unit_identity()),
        session_id: String::from(unit.session_id()),
        dispatch_id: String::from(unit.dispatch_id()),
        session_lineage_identity: String::from(unit.session_lineage_identity()),
        authority_profile_identity: String::from(unit.authority_profile_identity()),
        durable_binding_identity: String::from(unit.durable_binding_identity()),
        enforcement_observation_identity: String::from(unit.enforcement_observation_identity()),
        operation_set_identity: String::from(unit.operation_set_identity()),
        adapter_cohort_identity: String::from(unit.adapter_cohort_identity()),
        resource_profile_identity: String::from(unit.resource_profile_identity()),
        resource_budget_identity: String::from(unit.resource_budget_identity()),
        runtime_services_identity: String::from(unit.runtime_services_identity()),
        operation_identities: unit.operation_identities().to_vec(),
        resource_identities: unit.resource_identities().to_vec(),
        runtime_views: unit.runtime_views().to_vec(),
        state: unit.state(),
        commit_required_before_effect: true,
        redaction_marker: super::data::REDACTION_MARKER,
        non_claims,
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = super::core::receipt_identity(&receipt);
    debug_assert_eq!(receipt.receipt_identity.len(), super::data::HASH_HEX_BYTES);
    debug_assert!(receipt.commit_required_before_effect);
    Ok(receipt)
}

/// Validate receipt identity and exact binding to one prepared unit.
///
/// # Errors
///
/// Returns an issue when any receipt field or its BLAKE3 identity does not match.
pub fn validate_runtime_authority_seal_receipt(
    unit: &super::data::RuntimeAuthorityUnit,
    receipt: &super::data::RuntimeAuthoritySealReceipt,
) -> Result<(), super::data::RuntimeAuthoritySealError> {
    debug_assert!(!super::data::NON_CLAIMS.is_empty());
    debug_assert!(!super::data::REDACTION_MARKER.is_empty());
    let expected_non_claims = super::data::NON_CLAIMS
        .iter()
        .map(|value| String::from(*value))
        .collect::<Vec<_>>();
    let is_binding_match = receipt.schema == super::data::RUNTIME_AUTHORITY_SEAL_RECEIPT_SCHEMA
        && receipt.unit_identity == unit.unit_identity()
        && receipt.session_id == unit.session_id()
        && receipt.dispatch_id == unit.dispatch_id()
        && receipt.session_lineage_identity == unit.session_lineage_identity()
        && receipt.authority_profile_identity == unit.authority_profile_identity()
        && receipt.durable_binding_identity == unit.durable_binding_identity()
        && receipt.enforcement_observation_identity == unit.enforcement_observation_identity()
        && receipt.operation_set_identity == unit.operation_set_identity()
        && receipt.adapter_cohort_identity == unit.adapter_cohort_identity()
        && receipt.resource_profile_identity == unit.resource_profile_identity()
        && receipt.resource_budget_identity == unit.resource_budget_identity()
        && receipt.runtime_services_identity == unit.runtime_services_identity()
        && receipt.operation_identities == unit.operation_identities()
        && receipt.resource_identities == unit.resource_identities()
        && receipt.runtime_views == unit.runtime_views()
        && receipt.state == super::data::RuntimeAuthorityUnitState::Preparing
        && receipt.commit_required_before_effect
        && receipt.redaction_marker == super::data::REDACTION_MARKER
        && receipt.non_claims == expected_non_claims;
    let expected_identity = super::core::receipt_identity(receipt);
    if !is_binding_match || receipt.receipt_identity != expected_identity {
        return Err(error(
            super::data::RuntimeAuthorityIssueCode::ReceiptMismatch,
        ));
    }
    Ok(())
}

fn reject_sensitive(
    sensitive: super::data::RuntimeAuthoritySensitiveDraft<'_>,
) -> Result<(), super::data::RuntimeAuthoritySealError> {
    let values = [
        sensitive.raw_grant,
        sensitive.descriptor,
        sensitive.host_path,
        sensitive.credential,
        sensitive.prompt,
        sensitive.provider_payload,
        sensitive.tool_body,
        sensitive.model_output,
        sensitive.environment_value,
    ];
    let sensitive_count = values.iter().filter(|value| value.is_some()).count();
    debug_assert_eq!(values.len(), SENSITIVE_FIELD_COUNT);
    debug_assert!(sensitive_count <= SENSITIVE_FIELD_COUNT);
    if sensitive_count == 0 {
        Ok(())
    } else {
        Err(error(super::data::RuntimeAuthorityIssueCode::UnsafeReceipt))
    }
}

fn error(code: super::data::RuntimeAuthorityIssueCode) -> super::data::RuntimeAuthoritySealError {
    super::data::RuntimeAuthoritySealError::from_issues(vec![
        super::data::RuntimeAuthorityIssue::new(code, FIELD_RECEIPT, None),
    ])
}
