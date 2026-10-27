use crate::support::*;
use basalt::authority_core::BasaltPolicyRef;
use basalt::mantle::{Denied, Invocation, Operation, Request, Verifier};

fn request(operation: Operation) -> Request {
    Request {
        operation,
        subject: signer(HOLDER_SEED_BYTE).issuer().unwrap().as_str().into(),
        account: "account-a".into(),
        project: "project-a".into(),
        endpoint: "private-gateway".into(),
        audience: "mantle-service-a".into(),
        generation: 7,
        request_digest: [41; 32],
        target: "/jobs/approved".into(),
    }
}
fn boundaries<'a>(
    fixture: &'a InvocationFixture,
    replay: &'a RecordingReplay<'_>,
    revocations: &'a dyn RevocationChecker,
    now: u64,
) -> UcanEnforcementBoundaries<'a> {
    UcanEnforcementBoundaries {
        verification_time: VerificationTime::from_unix_seconds(now),
        key_resolver: &fixture.keys,
        proof_store: &fixture.proofs,
        revocations,
        caveat_policies: &DenyUnknownCaveats,
        replay,
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    }
}
fn policy_ref() -> BasaltPolicyRef {
    BasaltPolicyRef::new(
        format!(
            "blake3:{}",
            blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex()
        ),
        basalt::SUPPORTED_SCHEMA_VERSION,
        "mantle-build-submit",
    )
}
const fn verifier<'a>(
    policy: &'a basalt::Policy,
    policy_ref: &'a BasaltPolicyRef,
    roots: &'a [String],
) -> Verifier<'a> {
    Verifier {
        policy,
        policy_ref,
        generation: 7,
        endpoint: "private-gateway",
        audience: "mantle-service-a",
        trusted_roots: roots,
    }
}
fn fixture(verifier: &Verifier<'_>, request: &Request) -> (InvocationFixture, SignerSignature) {
    let exact = verifier.enforcement(request).unwrap();
    let fixture = InvocationFixture::direct(
        CapabilityDocument::new(exact.resource().into(), exact.ability().into()).unwrap(),
    );
    let binding = bind_enforcement_action(request.operation.action(), &exact).unwrap();
    let signature = sign_operation(&fixture, &exact, nonce(), binding);
    (fixture, signature)
}
fn invocation<'a>(
    fixture: &'a InvocationFixture,
    signature: &'a SignerSignature,
) -> Invocation<'a> {
    Invocation {
        compact_token: fixture.token.as_str(),
        nonce: nonce(),
        holder_signature: signature,
    }
}

#[test]
fn mantle_signed_operations_are_distinct_and_revalidation_does_not_spend_replay_again() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    for operation in [
        Operation::BuildSubmit,
        Operation::InputUpload,
        Operation::StoreRead,
        Operation::ProjectDeclare,
        Operation::StatusRead,
        Operation::LogRead,
        Operation::EventRead,
        Operation::ResultRead,
        Operation::UsageRead,
        Operation::OwnedAttemptCancel,
        Operation::Publish,
        Operation::Admin,
    ] {
        let request = request(operation);
        let (fixture, signature) = fixture(&verifier, &request);
        let replay = RecordingReplay::new();
        let bounds = boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS);
        let session = verifier
            .verify(request.clone(), &invocation(&fixture, &signature), &bounds)
            .unwrap();
        verifier.revalidate(&session, &bounds).unwrap();
        session
            .require(operation, &request.target, request.request_digest)
            .unwrap();
        let wrong = match operation {
            Operation::Admin | Operation::ProjectDeclare => Operation::BuildSubmit,
            Operation::BuildSubmit => Operation::ProjectDeclare,
            _ => Operation::Admin,
        };
        assert_eq!(
            session.require(wrong, &request.target, request.request_digest),
            Err(Denied)
        );
        assert_eq!(replay.calls.get(), 1);
        assert_eq!(session.policy_identity(), reference.artifact_ref);
        let mut substituted = request.clone();
        substituted.operation = wrong;
        assert!(
            verifier
                .verify(substituted, &invocation(&fixture, &signature), &bounds)
                .is_err()
        );
    }
}

#[test]
fn mantle_denies_subject_project_endpoint_audience_generation_and_request_substitution_before_replay()
 {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let original = request(Operation::BuildSubmit);
    let (fixture, signature) = fixture(&verifier, &original);
    for field in 0..8 {
        let mut altered = original.clone();
        match field {
            0 => altered.subject.push('x'),
            1 => altered.project.push('x'),
            2 => altered.account.push('x'),
            3 => altered.endpoint.push('x'),
            4 => altered.audience.push('x'),
            5 => altered.generation += 1,
            6 => altered.request_digest[0] ^= 1,
            _ => altered.target.push('x'),
        }
        let replay = RecordingReplay::new();
        let bounds = boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS);
        let error = verifier
            .verify(altered, &invocation(&fixture, &signature), &bounds)
            .unwrap_err();
        assert_eq!(error.to_string(), "gateway-authority-denied");
        assert_eq!(replay.calls.get(), 0);
    }
}

#[test]
fn mantle_live_session_denies_expired_revoked_stale_and_untrusted_issuers() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let request = request(Operation::BuildSubmit);
    let (fixture, signature) = fixture(&verifier, &request);
    let replay = RecordingReplay::new();
    let bounds = boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS);
    let session = verifier
        .verify(request, &invocation(&fixture, &signature), &bounds)
        .unwrap();
    assert_eq!(
        verifier.revalidate(
            &session,
            &boundaries(&fixture, &replay, &NoRevocations, EXPIRED_VERIFY_AT_SECONDS)
        ),
        Err(Denied)
    );
    let revoked = RevocationList::from_references(vec![proof_reference(&fixture.token)]);
    assert_eq!(
        verifier.revalidate(
            &session,
            &boundaries(&fixture, &replay, &revoked, VERIFY_AT_SECONDS)
        ),
        Err(Denied)
    );
    let stale = Verifier {
        generation: 8,
        ..verifier
    };
    assert_eq!(stale.revalidate(&session, &bounds), Err(Denied));
    let no_roots = Verifier {
        generation: 7,
        trusted_roots: &[],
        ..stale
    };
    assert_eq!(no_roots.revalidate(&session, &bounds), Err(Denied));
    assert_eq!(replay.calls.get(), 1);
}

#[test]
fn mantle_revalidation_rejects_time_before_holder_admission_without_spending_replay() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let request = request(Operation::BuildSubmit);
    let (fixture, signature) = fixture(&verifier, &request);
    let replay = RecordingReplay::new();
    let bounds = boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS);
    let session = verifier
        .verify(request, &invocation(&fixture, &signature), &bounds)
        .unwrap();
    verifier.revalidate(&session, &bounds).unwrap();
    assert_eq!(
        verifier.revalidate(
            &session,
            &boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS - 1)
        ),
        Err(Denied)
    );
    assert_eq!(replay.calls.get(), 1);
}

#[test]
fn mantle_parent_grant_cannot_be_widened_to_administration() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let parent_request = request(Operation::BuildSubmit);
    let parent_exact = verifier.enforcement(&parent_request).unwrap();
    let issuer = signer(ISSUER_SEED_BYTE);
    let delegate = signer(DELEGATE_SEED_BYTE);
    let holder = signer(HOLDER_SEED_BYTE);
    let parent = issue_token_with_signer(
        &issuer,
        &AudienceDid::from(delegate.issuer().unwrap()),
        &CapabilitySet::new(vec![
            CapabilityDocument::new(
                parent_exact.resource().into(),
                parent_exact.ability().into(),
            )
            .unwrap(),
        ])
        .unwrap(),
        &ProofReferences::empty(),
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS).unwrap(),
    )
    .unwrap();
    let child_request = request(Operation::Admin);
    let child_exact = verifier.enforcement(&child_request).unwrap();
    let child = issue_token_with_signer(
        &delegate,
        &AudienceDid::from(holder.issuer().unwrap()),
        &CapabilitySet::new(vec![
            CapabilityDocument::new(child_exact.resource().into(), child_exact.ability().into())
                .unwrap(),
        ])
        .unwrap(),
        &ProofReferences::from_references(vec![proof_reference(&parent)]),
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS).unwrap(),
    )
    .unwrap();
    let fixture = InvocationFixture {
        token: child,
        audience: AudienceDid::from(holder.issuer().unwrap()),
        holder,
        keys: KeyResolutionContext::new(vec![
            issuer.verification_key().unwrap(),
            delegate.verification_key().unwrap(),
            signer(HOLDER_SEED_BYTE).verification_key().unwrap(),
        ]),
        proofs: ProofCollection::from_tokens(vec![parent]),
    };
    let binding = bind_enforcement_action(Operation::Admin.action(), &child_exact).unwrap();
    let signature = sign_operation(&fixture, &child_exact, nonce(), binding);
    let replay = RecordingReplay::new();
    assert!(
        verifier
            .verify(
                child_request,
                &invocation(&fixture, &signature),
                &boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS)
            )
            .is_err()
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn mantle_root_delegation_reaches_exact_build_operation() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let request = request(Operation::BuildSubmit);
    let exact = verifier.enforcement(&request).unwrap();
    let issuer = signer(ISSUER_SEED_BYTE);
    let delegate = signer(DELEGATE_SEED_BYTE);
    let holder = signer(HOLDER_SEED_BYTE);
    let parent = issue_token_with_signer(
        &issuer,
        &AudienceDid::from(delegate.issuer().unwrap()),
        &CapabilitySet::new(vec![
            CapabilityDocument::new("mantle://gateway/v1/".into(), exact.ability().into()).unwrap(),
        ])
        .unwrap(),
        &ProofReferences::empty(),
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS).unwrap(),
    )
    .unwrap();
    let child = issue_token_with_signer(
        &delegate,
        &AudienceDid::from(holder.issuer().unwrap()),
        &CapabilitySet::new(vec![
            CapabilityDocument::new(exact.resource().into(), exact.ability().into()).unwrap(),
        ])
        .unwrap(),
        &ProofReferences::from_references(vec![proof_reference(&parent)]),
        TokenTimeBounds::new(TOKEN_NOT_BEFORE_SECONDS, TOKEN_EXPIRES_SECONDS).unwrap(),
    )
    .unwrap();
    let fixture = InvocationFixture {
        token: child,
        audience: AudienceDid::from(holder.issuer().unwrap()),
        holder,
        keys: KeyResolutionContext::new(vec![
            issuer.verification_key().unwrap(),
            delegate.verification_key().unwrap(),
            signer(HOLDER_SEED_BYTE).verification_key().unwrap(),
        ]),
        proofs: ProofCollection::from_tokens(vec![parent]),
    };
    let binding = bind_enforcement_action(Operation::BuildSubmit.action(), &exact).unwrap();
    let signature = sign_operation(&fixture, &exact, nonce(), binding);
    let replay = RecordingReplay::new();
    let bounds = boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS);
    let session = verifier
        .verify(request, &invocation(&fixture, &signature), &bounds)
        .unwrap();
    verifier.revalidate(&session, &bounds).unwrap();
    assert_eq!(replay.calls.get(), 1);
    let revoked_parent =
        RevocationList::from_references(vec![proof_reference(&fixture.proofs.as_slice()[0])]);
    assert_eq!(
        verifier.revalidate(
            &session,
            &boundaries(&fixture, &replay, &revoked_parent, VERIFY_AT_SECONDS)
        ),
        Err(Denied)
    );
}

// r[verify ucan_token_flow.oidc.native_signing]
#[test]
fn canonical_signing_request_rejects_malformed_fields_and_aggregate_bounds() {
    let reference = policy_ref();
    let original = request(Operation::BuildSubmit);
    for target in [String::new(), "x".repeat(513), "invalid\nfield".into(), "nonascii-é".into()] {
        let mut altered = original.clone();
        altered.target = target;
        assert!(basalt::mantle::canonical_enforcement(&altered, &reference).is_err());
    }
    let mut zero_generation = original.clone();
    zero_generation.generation = 0;
    assert!(basalt::mantle::canonical_enforcement(&zero_generation, &reference).is_err());
    let mut oversized = original;
    oversized.subject = "s".repeat(400);
    oversized.account = "a".repeat(400);
    oversized.project = "p".repeat(400);
    oversized.endpoint = "e".repeat(400);
    oversized.audience = "u".repeat(400);
    oversized.target = "t".repeat(400);
    assert!(basalt::mantle::canonical_enforcement(&oversized, &reference).is_err());
}
