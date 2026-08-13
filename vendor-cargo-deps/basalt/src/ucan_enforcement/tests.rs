use ucan::OperationProfileAdmissionError;

use super::core::OPERATION_PROFILE_DOMAIN;
use super::core::blake3_reference_prefix;
use super::core::decode_blake3_reference;
use super::core::derive_request_binding;
use super::core::operation_profile_digest;
use super::error::profile_error_reason;
use super::receipt::BLAKE3_DIGEST_BYTES;
use super::receipt::ReceiptContext;
use super::receipt::build_receipt;
use super::receipt::receipt_schema;
use super::types::EnforcementAction;
use super::types::UcanEnforcementReason;
use crate::EnforcementRequest;

const CREDENTIAL_BYTE: u8 = 3;
const REQUEST_BYTE: u8 = 5;
const POLICY_BYTE: u8 = 7;
const HEX_CHARACTERS_PER_BYTE: usize = 2;
const EXPECTED_RECEIPT_BLAKE3: &str = "blake3:5f616b3417ee513cabba8dd8326bebe36b33e67e53be81093efe729870b1f640";

fn receipt_context() -> ReceiptContext {
    ReceiptContext {
        credential_bytes: Some(u64::try_from(BLAKE3_DIGEST_BYTES).expect("digest width must fit u64")),
        credential: Some([CREDENTIAL_BYTE; BLAKE3_DIGEST_BYTES]),
        request_binding: Some([REQUEST_BYTE; BLAKE3_DIGEST_BYTES]),
        operation_profile: operation_profile_digest(EnforcementAction::DocumentWrite),
        policy: Some([POLICY_BYTE; BLAKE3_DIGEST_BYTES]),
    }
}

#[test]
fn typed_actions_have_distinct_reviewed_bindings() {
    let read = operation_profile_digest(EnforcementAction::DocumentRead);
    let write = operation_profile_digest(EnforcementAction::DocumentWrite);
    let request = EnforcementRequest::new("document-write", "did:key:z6Mkresource/docs/report", "document/write");
    let first = derive_request_binding(EnforcementAction::DocumentWrite, &request).expect("bounded request must bind");
    let second =
        derive_request_binding(EnforcementAction::DocumentWrite, &request).expect("the same bounded request must bind");

    assert_ne!(read, write);
    assert_eq!(first, second);
    assert_eq!(EnforcementAction::DocumentRead.ability(), "document/read");
    assert_eq!(EnforcementAction::DocumentWrite.ability(), "document/write");
}

#[test]
fn operation_profile_hash_uses_unambiguous_length_prefixes() {
    let action = EnforcementAction::DocumentWrite;
    let fields = [
        action.as_str().as_bytes(),
        action.contract_id().as_bytes(),
        action.ability().as_bytes(),
    ];
    let mut framed = blake3::Hasher::new();
    framed.update(OPERATION_PROFILE_DOMAIN);
    for field in fields {
        let field_length = u64::try_from(field.len()).expect("test field length must fit u64");
        framed.update(&field_length.to_be_bytes());
        framed.update(field);
    }
    let mut ambiguous = blake3::Hasher::new();
    ambiguous.update(OPERATION_PROFILE_DOMAIN);
    for field in fields {
        ambiguous.update(field);
    }

    assert_eq!(operation_profile_digest(action), *framed.finalize().as_bytes());
    assert_ne!(operation_profile_digest(action), *ambiguous.finalize().as_bytes());
}

#[test]
fn profile_reason_mapping_is_stable() {
    assert_eq!(
        profile_error_reason(OperationProfileAdmissionError::UnknownProfile),
        UcanEnforcementReason::OperationMismatch
    );
    assert_eq!(
        profile_error_reason(OperationProfileAdmissionError::Backend),
        UcanEnforcementReason::BackendUnavailable
    );
}

#[test]
fn receipt_is_deterministic_and_redacted() {
    let first = build_receipt(UcanEnforcementReason::Allowed, true, receipt_context());
    let second = build_receipt(UcanEnforcementReason::Allowed, true, receipt_context());
    let json = serde_json::to_string(&first).expect("receipt must serialize");

    assert_eq!(first, second);
    assert_eq!(first.schema, receipt_schema());
    assert_eq!(first.receipt_blake3, EXPECTED_RECEIPT_BLAKE3);
    assert!(first.receipt_blake3.starts_with(blake3_reference_prefix()));
    assert!(!json.contains("did:key"));
    assert!(!json.contains("document/write"));
    assert!(!json.contains("header.payload.signature"));
    assert!(!json.contains("caveat"));
}

#[test]
fn oversized_receipt_omits_credential_identity() {
    let mut context = receipt_context();
    context.credential = None;
    let receipt = build_receipt(UcanEnforcementReason::OversizedCredential, false, context);

    assert!(receipt.credential_blake3.is_none());
    assert!(!receipt.is_allowed());
}

#[test]
fn blake3_reference_decoder_rejects_malformed_input() {
    let hex_length = BLAKE3_DIGEST_BYTES * HEX_CHARACTERS_PER_BYTE;
    let valid = format!("{}{}", blake3_reference_prefix(), "a".repeat(hex_length));

    assert!(decode_blake3_reference(&valid).is_some());
    assert!(decode_blake3_reference("blake3:short").is_none());
    assert!(
        decode_blake3_reference("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").is_none()
    );
}
