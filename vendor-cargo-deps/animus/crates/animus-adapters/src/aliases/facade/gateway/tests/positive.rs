use super::support::*;

#[test]
fn authorized_prompt_admission_records_safe_receipt_without_prompt_body() {
    let request = request_with(Some(credential(VALID_CREDENTIAL)), Some(HASH_B)).expect("request should validate");

    let decision = super::super::admit(&request).expect("decision should evaluate");
    let receipt_text = decision.receipt().render_safe_text();

    assert!(decision.is_allowed());
    assert_eq!(decision.verdict(), super::super::AdmissionVerdict::Allow);
    assert_eq!(decision.reason(), None);
    assert_eq!(decision.receipt().credential_hash().map(crate::ArtifactHash::as_str), Some(HASH_A));
    assert!(!receipt_text.contains(ACTOR));
    assert!(!receipt_text.contains(AUDIENCE));
    assert!(!receipt_text.contains(PROMPT_BODY));
    super::super::validate_safe_text(receipt_text.as_str()).expect("safe receipt should validate");
}

#[test]
fn host_owned_boundaries_are_supplied_hashes_only() {
    let request = request_with(Some(credential(VALID_CREDENTIAL)), Some(HASH_B)).expect("request should validate");
    let credential = request.credential().expect("credential should exist");

    assert_eq!(credential.credential_hash().as_str(), HASH_A);
    assert_eq!(request.policy_hash().map(crate::ArtifactHash::as_str), Some(HASH_B));
}

#[test]
fn pinned_https_remote_server_admission_records_safe_receipt() {
    // r[verify agent_harness_runtime.goose_remote_server_admission.descriptors.pinned]
    // r[verify agent_harness_runtime.goose_remote_server_admission.admission.pinned_success]
    // r[verify agent_harness_runtime.goose_remote_server_admission.receipts.rejects_leaks]
    let descriptor = remote_descriptor().expect("descriptor should validate");
    let handshake = remote_handshake().expect("handshake should validate");
    let policy = remote_policy().expect("policy should validate");

    let decision = remote_decision(&descriptor, Some(&handshake), &policy).expect("admission should evaluate");
    let receipt_text = decision.receipt().render_safe_text();

    assert!(decision.is_admitted());
    assert_eq!(decision.outcome(), super::super::RemoteServerAdmissionOutcome::Admitted);
    assert_eq!(decision.reason(), None);
    assert_eq!(decision.receipt().endpoint_identity_hash().as_str(), HASH_C);
    assert_eq!(decision.receipt().policy_hash().as_str(), HASH_B);
    assert!(!receipt_text.contains(RAW_ENDPOINT));
    assert!(!receipt_text.contains(SECRET_SOURCE));
    assert!(!receipt_text.contains(AUDIENCE));
    super::super::validate_remote_server_admission_receipt_text(receipt_text.as_str())
        .expect("safe remote server receipt should validate");
}

#[test]
fn approved_tofu_remote_server_admission_is_explicit() {
    // r[verify agent_harness_runtime.goose_remote_server_admission.verification]
    let descriptor = remote_descriptor_with(
        super::super::RemoteServerTransportKind::Https,
        super::super::RemoteServerFingerprintPolicy::TrustOnFirstUsePending,
        super::super::RemoteServerProtocolRole::HeadlessControl,
        HASH_A,
    )
    .expect("descriptor should validate");
    let handshake = remote_handshake_with(RemoteHandshakeFixture {
        fingerprint_status: super::super::RemoteServerFingerprintStatus::TofuPending,
        ..RemoteHandshakeFixture::pinned()
    })
    .expect("handshake should validate");
    let policy = remote_policy_with(RemotePolicyFixture {
        tofu_approved: true,
        ..RemotePolicyFixture::default()
    })
    .expect("policy should validate");

    let decision = remote_decision(&descriptor, Some(&handshake), &policy).expect("admission should evaluate");

    assert!(decision.is_admitted());
    assert_eq!(decision.receipt().fingerprint_status(), super::super::RemoteServerFingerprintStatus::TofuPending);
}
