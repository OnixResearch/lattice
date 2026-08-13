use crate::support::*;

#[test]
fn caller_supplied_capability_facts_are_rejected_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE)
        .with_capability(CapabilityGrant::new(DOCUMENT_RESOURCE, DOCUMENT_WRITE_ABILITY));
    let valid_request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &valid_request, nonce());
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::UnverifiedAuthorityFields,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn proof_and_key_failures_are_typed() {
    let delegated = InvocationFixture::delegated();
    let delegated_request = enforcement_request(DOCUMENT_RESOURCE);
    let delegated_signature = sign_request(&delegated, &delegated_request, nonce());
    let direct = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let direct_request = enforcement_request(DOCUMENT_RESOURCE);
    let direct_signature = sign_request(&direct, &direct_request, nonce());
    let replay = RecordingReplay::new();
    let empty_proofs = ProofCollection::empty();
    let holder_only_keys =
        KeyResolutionContext::new(vec![direct.holder.verification_key().expect("holder key must derive")]);
    let proof_services = TestServices {
        key_resolver: &delegated.keys,
        proof_store: &empty_proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };

    assert_denied(
        evaluate(delegated.token.as_str(), &delegated_request, &delegated_signature, nonce(), proof_services),
        UcanEnforcementReason::ProofDenied,
    );
    let key_services = TestServices {
        key_resolver: &holder_only_keys,
        proof_store: &direct.proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };
    assert_denied(
        evaluate(direct.token.as_str(), &direct_request, &direct_signature, nonce(), key_services),
        UcanEnforcementReason::KeyResolutionDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn time_revocation_caveat_and_scope_failures_are_typed() {
    let direct = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let direct_signature = sign_request(&direct, &request, nonce());
    let expired =
        InvocationFixture::direct_with_expiration(document_capability(PARENT_RESOURCE), TOKEN_EXPIRES_SECONDS);
    let expired_signature = sign_request(&expired, &request, nonce());
    let caveated = InvocationFixture::direct(caveated_document_capability());
    let caveated_signature = sign_request(&caveated, &request, nonce());
    let scoped = InvocationFixture::direct(document_capability(OTHER_RESOURCE));
    let scoped_signature = sign_request(&scoped, &request, nonce());
    let replay = RecordingReplay::new();
    let revoked = RevocationList::from_references(vec![proof_reference(&direct.token)]);
    let mut expired_services = services(&expired, &NoRevocations, &DenyUnknownCaveats, &replay);
    expired_services.verification_time = VerificationTime::from_unix_seconds(EXPIRED_VERIFY_AT_SECONDS);

    assert_denied(
        evaluate(expired.token.as_str(), &request, &expired_signature, nonce(), expired_services),
        UcanEnforcementReason::TimeDenied,
    );
    assert_denied(
        evaluate(
            direct.token.as_str(),
            &request,
            &direct_signature,
            nonce(),
            services(&direct, &revoked, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::Revoked,
    );
    assert_denied(
        evaluate(
            caveated.token.as_str(),
            &request,
            &caveated_signature,
            nonce(),
            services(&caveated, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::CaveatDenied,
    );
    assert_denied(
        evaluate(
            scoped.token.as_str(),
            &request,
            &scoped_signature,
            nonce(),
            services(&scoped, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::ScopeDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}
