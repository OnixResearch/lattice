use std::cell::Cell;

use crate::support::*;

#[test]
fn caveat_observations_repeat_before_one_final_replay_commit() {
    let fixture = InvocationFixture::direct(caveated_document_capability());
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let caveats = CountingCaveatSet::new(true);
    let replay = RecordingReplay::after_caveats(&caveats.policy.calls, EXPECTED_CAVEAT_OBSERVATIONS);

    let live = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &caveats, &replay),
    )
    .expect("admitted caveat must pass both observations");

    assert!(live.receipt().is_allowed());
    assert_eq!(caveats.policy.calls.get(), EXPECTED_CAVEAT_OBSERVATIONS);
    assert_eq!(replay.calls.get(), FIRST_CALL);
    assert!(replay.committed.get());
}

#[test]
fn basalt_policy_denial_does_not_burn_nonce() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(PRIVATE_RESOURCE);
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
        UcanEnforcementReason::PolicyDenied,
    );

    assert_eq!(replay.calls.get(), 0);
    assert!(!replay.committed.get());
}

#[test]
fn final_verification_is_authoritative_and_drops_prospective_authority() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();
    let resolver = FailsAfterResolver {
        inner: &fixture.keys,
        allowed_calls: EXPECTED_PREFLIGHT_RESOLVER_CALLS,
        calls: Cell::new(0),
    };
    let services = TestServices {
        key_resolver: &resolver,
        proof_store: &fixture.proofs,
        revocations: &NoRevocations,
        caveats: &DenyUnknownCaveats,
        replay: &replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    };

    let receipt = assert_denied(
        evaluate(fixture.token.as_str(), &request, &signature, nonce(), services),
        UcanEnforcementReason::BackendUnavailable,
    );

    assert!(!receipt.is_allowed());
    assert_eq!(replay.calls.get(), 0);
    assert!(resolver.calls.get() > EXPECTED_PREFLIGHT_RESOLVER_CALLS);
}

#[test]
fn duplicate_nonce_fails_only_at_final_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    let first = evaluate(
        fixture.token.as_str(),
        &request,
        &signature,
        nonce(),
        services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
    );
    assert!(first.is_ok());
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::ReplayDenied,
    );

    assert_eq!(replay.calls.get(), EXPECTED_CAVEAT_OBSERVATIONS);
}

#[test]
fn successful_replay_branch_only_moves_prebuilt_values() {
    let source = include_str!("../../src/ucan_enforcement/shell.rs");
    let success_branch = source
        .split("match verify_holder_signed_invocation")
        .nth(1)
        .expect("final holder verification call must exist")
        .split("Err(error)")
        .next()
        .expect("success branch must precede failure branch");

    for forbidden in [
        "build_receipt",
        "evaluate_authority",
        "adapt_enforcement_request",
        "blake3::",
        "format!",
        "assert!",
        "String::",
        "Vec::",
        "Box::",
        ".collect",
        ".map_err",
        ".and_then",
        "failure(",
        "?",
        "to_owned",
        "clone()",
    ] {
        assert!(
            !success_branch.contains(forbidden),
            "post-replay success branch contains forbidden work: {forbidden}"
        );
    }
    assert!(success_branch.contains("Ok(LiveUcanEnforcement"));
    assert!(success_branch.contains("_authority_decision: prospective.decision"));
    assert!(success_branch.contains("receipt: prospective.receipt"));
}
