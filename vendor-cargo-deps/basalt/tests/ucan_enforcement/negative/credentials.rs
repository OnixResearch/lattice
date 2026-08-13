use crate::support::*;

#[test]
fn malformed_and_oversized_credentials_fail_closed() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let malformed = "not.enough";
    let oversized = "x".repeat(OVERSIZED_TOKEN_BYTES);
    let mut oversized_services = services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay);
    oversized_services.verification_limits = VerificationLimits::new(
        VerificationLimits::default().max_proof_depth(),
        VerificationLimits::default().max_proof_count(),
        OVERSIZED_TOKEN_LIMIT,
    );

    let malformed_receipt = assert_denied(
        evaluate(
            malformed,
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::MalformedCredential,
    );
    let oversized_receipt = assert_denied(
        evaluate(oversized.as_str(), &request, &signature, nonce(), oversized_services),
        UcanEnforcementReason::OversizedCredential,
    );

    assert!(malformed_receipt.credential_blake3.is_some());
    assert!(oversized_receipt.credential_blake3.is_none());
    assert_eq!(
        oversized_receipt.credential_bytes,
        Some(u64::try_from(OVERSIZED_TOKEN_BYTES).expect("test token length must fit u64"))
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn token_signature_and_holder_signature_failures_are_distinct() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let substituted_nonce_signature = sign_request(&fixture, &request, other_nonce());
    let replay = RecordingReplay::new();
    let wrong_issuer = signer(WRONG_ISSUER_SEED_BYTE);
    let wrong_key_resolver = WrongIssuerKeyResolver {
        public_key: wrong_issuer.public_key().expect("wrong issuer public key must derive"),
    };
    let wrong_key_services = TestServices {
        key_resolver: &wrong_key_resolver,
        proof_store: &fixture.proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };

    assert_denied(
        evaluate(fixture.token.as_str(), &request, &signature, nonce(), wrong_key_services),
        UcanEnforcementReason::SignatureDenied,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &substituted_nonce_signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::HolderDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn canonical_field_size_failure_has_stable_limit_reason() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let mut bounded_services = services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay);
    bounded_services.holder_limits = HolderInvocationLimits::try_new(
        HolderInvocationLimits::default().max_audience_bytes(),
        SHORT_CANONICAL_RESOURCE_LIMIT,
        HolderInvocationLimits::default().max_ability_bytes(),
    )
    .expect("short test canonical limit must validate");

    assert_denied(
        evaluate(fixture.token.as_str(), &request, &signature, nonce(), bounded_services),
        UcanEnforcementReason::LimitExceeded,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn operation_and_request_limits_fail_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let valid_request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &valid_request, nonce());
    let wrong_action_request = EnforcementRequest::new("document-write", DOCUMENT_RESOURCE, DOCUMENT_READ_ABILITY);
    let oversized_resource = "r".repeat(OVERSIZED_REQUEST_BYTES);
    let oversized_request = enforcement_request(oversized_resource.as_str());
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &wrong_action_request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::OperationMismatch,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &oversized_request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::LimitExceeded,
    );
    assert_eq!(replay.calls.get(), 0);
}
