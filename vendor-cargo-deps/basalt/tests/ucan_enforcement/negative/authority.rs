use crate::support::*;

#[test]
fn caller_supplied_capability_facts_are_rejected_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE).with_capability(CapabilityGrant::new(
        DOCUMENT_RESOURCE,
        DOCUMENT_WRITE_ABILITY,
    ));
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
    let holder_only_keys = KeyResolutionContext::new(vec![
        direct
            .holder
            .verification_key()
            .expect("holder key must derive"),
    ]);
    let proof_services = TestServices {
        key_resolver: &delegated.keys,
        trusted_roots: &delegated.trusted_roots,
        proof_store: &empty_proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };

    assert_denied(
        evaluate(
            delegated.token.as_str(),
            &delegated_request,
            &delegated_signature,
            nonce(),
            proof_services,
        ),
        UcanEnforcementReason::ProofDenied,
    );
    let key_services = TestServices {
        key_resolver: &holder_only_keys,
        trusted_roots: &direct.trusted_roots,
        proof_store: &direct.proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };
    assert_denied(
        evaluate(
            direct.token.as_str(),
            &direct_request,
            &direct_signature,
            nonce(),
            key_services,
        ),
        UcanEnforcementReason::KeyResolutionDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn time_revocation_caveat_and_scope_failures_are_typed() {
    let direct = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let direct_signature = sign_request(&direct, &request, nonce());
    let expired = InvocationFixture::direct_with_expiration(
        document_capability(PARENT_RESOURCE),
        TOKEN_EXPIRES_SECONDS,
    );
    let expired_signature = sign_request(&expired, &request, nonce());
    let caveated = InvocationFixture::direct(caveated_document_capability());
    let caveated_signature = sign_request(&caveated, &request, nonce());
    let scoped = InvocationFixture::direct(document_capability(OTHER_RESOURCE));
    let scoped_signature = sign_request(&scoped, &request, nonce());
    let replay = RecordingReplay::new();
    let revoked = RevocationList::from_references(vec![proof_reference(&direct.token)]);
    let mut expired_services = services(&expired, &NoRevocations, &DenyUnknownCaveats, &replay);
    expired_services.verification_time =
        VerificationTime::from_unix_seconds(EXPIRED_VERIFY_AT_SECONDS);

    assert_denied(
        evaluate(
            expired.token.as_str(),
            &request,
            &expired_signature,
            nonce(),
            expired_services,
        ),
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

#[test]
fn untrusted_delegate_or_holder_cannot_mint_root_token_via_shared_key_resolver() {
    // Issue #4 PoC: DELEGATE (whose key is present in key_resolver) issues HOLDER
    // a proof-less root token on did:key:z6Mkresource/docs/. Only ISSUER is in
    // trusted_roots, so generic UCAN enforcement must fail closed before replay.
    let trusted_issuer = signer(ISSUER_SEED_BYTE);
    let delegate = signer(DELEGATE_SEED_BYTE);
    let holder = signer(HOLDER_SEED_BYTE);
    let holder_audience = AudienceDid::from(holder.issuer().expect("holder DID must derive"));
    let forged_root = issue_token_with_signer(
        &delegate,
        &holder_audience,
        &CapabilitySet::new(vec![document_capability("did:key:z6Mkresource/docs/")])
            .expect("capability must validate"),
        &ProofReferences::empty(),
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS)
            .expect("time bounds must validate"),
    )
    .expect("forged root token must issue");
    let keys = KeyResolutionContext::new(vec![
        trusted_issuer
            .verification_key()
            .expect("issuer key must derive"),
        delegate
            .verification_key()
            .expect("delegate key must derive"),
        holder.verification_key().expect("holder key must derive"),
    ]);
    let trusted_roots = vec![
        trusted_issuer
            .issuer()
            .expect("issuer DID must derive")
            .as_str()
            .to_owned(),
    ];
    let fixture = InvocationFixture {
        token: forged_root,
        holder,
        audience: holder_audience,
        keys,
        trusted_roots,
        proofs: ProofCollection::empty(),
    };
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::UntrustedRootIssuer,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn empty_trusted_roots_and_untrusted_delegation_chain_root_deny_before_replay() {
    let direct = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&direct, &request, nonce());
    let replay = RecordingReplay::new();
    let mut empty_roots_services = services(&direct, &NoRevocations, &DenyUnknownCaveats, &replay);
    empty_roots_services.trusted_roots = &[];

    assert_denied(
        evaluate(
            direct.token.as_str(),
            &request,
            &signature,
            nonce(),
            empty_roots_services,
        ),
        UcanEnforcementReason::UntrustedRootIssuer,
    );
    assert_eq!(replay.calls.get(), 0);

    // Delegated chain whose root proof is issued by an untrusted key (WRONG_ISSUER)
    // must also fail closed even when all keys are resolvable.
    let trusted_issuer = signer(ISSUER_SEED_BYTE);
    let wrong_issuer = signer(WRONG_ISSUER_SEED_BYTE);
    let delegate = signer(DELEGATE_SEED_BYTE);
    let holder = signer(HOLDER_SEED_BYTE);
    let delegate_audience = AudienceDid::from(delegate.issuer().expect("delegate DID must derive"));
    let holder_audience = AudienceDid::from(holder.issuer().expect("holder DID must derive"));
    let bounds =
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS).expect("time bounds");
    let untrusted_parent = issue_token_with_signer(
        &wrong_issuer,
        &delegate_audience,
        &CapabilitySet::new(vec![document_capability(PARENT_RESOURCE)]).expect("parent cap"),
        &ProofReferences::empty(),
        bounds,
    )
    .expect("parent token");
    let child = issue_token_with_signer(
        &delegate,
        &holder_audience,
        &CapabilitySet::new(vec![document_capability(DOCUMENT_RESOURCE)]).expect("child cap"),
        &ProofReferences::from_references(vec![proof_reference(&untrusted_parent)]),
        bounds,
    )
    .expect("child token");
    let fixture = InvocationFixture {
        token: child,
        holder,
        audience: holder_audience,
        keys: KeyResolutionContext::new(vec![
            trusted_issuer.verification_key().expect("trusted key"),
            wrong_issuer.verification_key().expect("wrong key"),
            delegate.verification_key().expect("delegate key"),
            signer(HOLDER_SEED_BYTE)
                .verification_key()
                .expect("holder key"),
        ]),
        trusted_roots: vec![
            trusted_issuer
                .issuer()
                .expect("trusted DID")
                .as_str()
                .to_owned(),
        ],
        proofs: ProofCollection::from_tokens(vec![untrusted_parent]),
    };
    let child_signature = sign_request(&fixture, &request, nonce());

    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &child_signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::UntrustedRootIssuer,
    );
    assert_eq!(replay.calls.get(), 0);
}
