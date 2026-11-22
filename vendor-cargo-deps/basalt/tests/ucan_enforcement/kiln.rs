use crate::support::*;
use basalt::authority_core::BasaltPolicyRef;
use basalt::kiln::{Denied, ImplementationFraming, Invocation, Operation, Request, Verifier};

const ALL_OPERATIONS: [Operation; 9] = [
    Operation::ClaimAcquire,
    Operation::ClaimRenew,
    Operation::Dispatch,
    Operation::CandidateSubmit,
    Operation::VerificationAdmit,
    Operation::AttemptCancel,
    Operation::PublicationPublish,
    Operation::IntegrationLand,
    Operation::LifecycleComplete,
];

fn request(operation: Operation) -> Request {
    Request {
        operation,
        subject: signer(HOLDER_SEED_BYTE).issuer().unwrap().as_str().into(),
        audience: "kiln-service-a".into(),
        project: "OnixResearch/kiln".into(),
        work: "kiln.implementation.work".into(),
        work_version: 3,
        attempt: "attempt-a".into(),
        fence: 5,
        generation: 7,
        request_digest: [41; 32],
        target: "candidate/approved".into(),
        candidate: if operation.requires_candidate() {
            [43; 32]
        } else {
            [0; 32]
        },
        framing: ImplementationFraming::V1,
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
        "kiln-dispatch",
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
        audience: "kiln-service-a",
        trusted_roots: roots,
        max_delegation_depth: 1,
        implementation_v1: basalt::kiln::LegacyFraming::Accept,
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
// r[verify basalt.kiln_authority.actions.scenario.distinct]
// r[verify basalt.kiln_authority.actions.scenario.substitution]
// r[verify basalt.kiln_authority.revalidation.scenario.nonconsuming]
fn kiln_signed_operations_are_distinct_and_revalidation_does_not_spend_replay_again() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    for operation in ALL_OPERATIONS {
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
        let wrong = if operation == Operation::LifecycleComplete {
            Operation::Dispatch
        } else {
            Operation::LifecycleComplete
        };
        assert_eq!(
            session.require(wrong, &request.target, request.request_digest),
            Err(Denied)
        );
        if operation.requires_candidate() {
            session.require_candidate(request.candidate).unwrap();
        }
        assert_eq!(session.require_candidate([44; 32]), Err(Denied));
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
// r[verify basalt.kiln_authority.binding.scenario.substitution]
// r[verify basalt.kiln_authority.ingress.scenario.denied]
fn kiln_denies_field_substitution_before_replay() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let original = request(Operation::CandidateSubmit);
    let (fixture, signature) = fixture(&verifier, &original);
    for field in 0..12 {
        let mut altered = original.clone();
        match field {
            0 => altered.subject.push('x'),
            1 => altered.audience.push('x'),
            2 => altered.project.push('x'),
            3 => altered.work.push('x'),
            4 => altered.work_version += 1,
            5 => altered.attempt.push('x'),
            6 => altered.fence += 1,
            7 => altered.generation += 1,
            8 => altered.request_digest[0] ^= 1,
            9 => altered.target.push('x'),
            10 => altered.candidate[0] ^= 1,
            _ => altered.candidate = [0; 32],
        }
        let replay = RecordingReplay::new();
        let bounds = boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS);
        let error = verifier
            .verify(altered, &invocation(&fixture, &signature), &bounds)
            .unwrap_err();
        assert_eq!(error.to_string(), "kiln-authority-denied");
        assert_eq!(replay.calls.get(), 0);
    }
}

#[test]
// r[verify basalt.kiln_authority.revalidation.scenario.stale]
fn kiln_live_session_denies_expired_revoked_stale_and_untrusted_issuers() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let request = request(Operation::Dispatch);
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
// r[verify basalt.kiln_authority.binding.scenario.candidate]
fn kiln_candidate_identity_is_required_exactly_where_a_mutation_binds_one() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);

    let mut zero_submit = request(Operation::CandidateSubmit);
    zero_submit.candidate = [0; 32];
    assert!(verifier.enforcement(&zero_submit).is_err());

    let mut candidate_dispatch = request(Operation::Dispatch);
    candidate_dispatch.candidate = [43; 32];
    assert!(verifier.enforcement(&candidate_dispatch).is_err());

    let mut zero_cancel = request(Operation::AttemptCancel);
    zero_cancel.candidate = [43; 32];
    assert!(verifier.enforcement(&zero_cancel).is_err());

    assert!(verifier.enforcement(&request(Operation::Dispatch)).is_ok());
    assert!(
        verifier
            .enforcement(&request(Operation::CandidateSubmit))
            .is_ok()
    );
}

#[test]
// r[verify basalt.kiln_authority.delegation.scenario.widen]
fn kiln_execution_grant_cannot_be_widened_to_publication_or_lifecycle() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let parent_request = request(Operation::Dispatch);
    let parent_exact = verifier.enforcement(&parent_request).unwrap();
    let issuer = signer(ISSUER_SEED_BYTE);
    let delegate = signer(DELEGATE_SEED_BYTE);
    for widened in [Operation::PublicationPublish, Operation::LifecycleComplete] {
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
        let child_request = request(widened);
        let child_exact = verifier.enforcement(&child_request).unwrap();
        let child = issue_token_with_signer(
            &delegate,
            &AudienceDid::from(holder.issuer().unwrap()),
            &CapabilitySet::new(vec![
                CapabilityDocument::new(
                    child_exact.resource().into(),
                    child_exact.ability().into(),
                )
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
            holder: signer(HOLDER_SEED_BYTE),
            keys: KeyResolutionContext::new(vec![
                issuer.verification_key().unwrap(),
                delegate.verification_key().unwrap(),
                signer(HOLDER_SEED_BYTE).verification_key().unwrap(),
            ]),
            proofs: ProofCollection::from_tokens(vec![parent]),
        };
        let binding = bind_enforcement_action(widened.action(), &child_exact).unwrap();
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
}

#[test]
// r[verify basalt.kiln_authority.delegation.scenario.exact]
fn kiln_root_delegation_reaches_exact_dispatch_operation() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let request = request(Operation::Dispatch);
    let exact = verifier.enforcement(&request).unwrap();
    let issuer = signer(ISSUER_SEED_BYTE);
    let delegate = signer(DELEGATE_SEED_BYTE);
    let holder = signer(HOLDER_SEED_BYTE);
    let parent = issue_token_with_signer(
        &issuer,
        &AudienceDid::from(delegate.issuer().unwrap()),
        &CapabilitySet::new(vec![
            CapabilityDocument::new("kiln://implementation/v1/".into(), exact.ability().into())
                .unwrap(),
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
    let binding = bind_enforcement_action(Operation::Dispatch.action(), &exact).unwrap();
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

#[test]
// r[verify basalt.kiln_authority.policy.scenario.missing]
fn kiln_policy_without_the_selected_contract_denies_before_replay() {
    let admitted = default_policy().unwrap();
    let reference = BasaltPolicyRef::new(
        format!(
            "blake3:{}",
            blake3::hash(basalt::DEFAULT_POLICY_JSON.as_bytes()).to_hex()
        ),
        basalt::SUPPORTED_SCHEMA_VERSION,
        "kiln-dispatch",
    );
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let admitted_verifier = verifier(&admitted, &reference, &roots);
    let request = request(Operation::Dispatch);
    let exact = admitted_verifier.enforcement(&request).unwrap();
    let fixture = InvocationFixture::direct(
        CapabilityDocument::new(exact.resource().into(), exact.ability().into()).unwrap(),
    );
    let binding = bind_enforcement_action(Operation::Dispatch.action(), &exact).unwrap();
    let signature = sign_operation(&fixture, &exact, nonce(), binding);
    let minimal = basalt::parse_policy_json(
        r#"{
          "schema_version": "ucan-nickel-contracts.policy.v1",
          "contracts": {
            "document-write": {
              "id": "document-write",
              "resource_prefixes": ["did:key:z6Mkresource/docs/"],
              "abilities": ["document/write"]
            }
          }
        }"#,
    )
    .expect("minimal policy must parse");
    let verifier = verifier(&minimal, &reference, &roots);
    let replay = RecordingReplay::new();
    let bounds = boundaries(&fixture, &replay, &NoRevocations, VERIFY_AT_SECONDS);
    assert!(
        verifier
            .verify(request, &invocation(&fixture, &signature), &bounds)
            .is_err()
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
// r[verify basalt.kiln_authority.attenuable.scenario.v1_stable]
fn kiln_v1_framing_bytes_and_serialized_requests_are_unchanged() {
    let policy = default_policy().unwrap();
    let reference = policy_ref();
    let roots = [signer(ISSUER_SEED_BYTE).issuer().unwrap().as_str().into()];
    let verifier = verifier(&policy, &reference, &roots);
    let mut fixed = request(Operation::CandidateSubmit);
    fixed.subject = "did:key:z6Mkholder".into();
    // Frozen bytes recorded from the pre-v2 framing at basalt 1c142e2.
    assert_eq!(
        verifier.enforcement(&fixed).unwrap().resource(),
        "kiln://implementation/v1/18/6469643a6b65793a7a364d6b686f6c646572/\
         14/6b696c6e2d736572766963652d61/17/4f6e697852657365617263682f6b696c6e/\
         24/6b696c6e2e696d706c656d656e746174696f6e2e776f726b/9/617474656d70742d61/\
         18/63616e6469646174652f617070726f766564/3/5/7/\
         2929292929292929292929292929292929292929292929292929292929292929/\
         2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"
    );
    let mut legacy = serde_json::to_value(&fixed).unwrap();
    legacy.as_object_mut().unwrap().remove("framing");
    let decoded: Request = serde_json::from_value(legacy).unwrap();
    assert!(decoded == fixed);
}
