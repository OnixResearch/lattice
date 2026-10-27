use crate::support::*;

#[test]
fn typed_action_and_profile_substitutions_fail_before_replay() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let valid_signature = sign_request(&fixture, &request, nonce());
    let exact_operation = bind_enforcement_action(EnforcementAction::DocumentWrite, &request)
        .expect("test request must bind");
    let substituted_operation = OperationBinding::new(
        OperationProfileId::from_bytes([UNKNOWN_PROFILE_BYTE; HOLDER_INVOCATION_DIGEST_BYTES]),
        exact_operation.request_binding(),
    );
    let substituted_profile_signature =
        sign_operation(&fixture, &request, nonce(), substituted_operation);
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate_action(
            fixture.token.as_str(),
            &request,
            EnforcementAction::DocumentRead,
            &valid_signature,
            nonce(),
            &services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::OperationMismatch,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &request,
            &substituted_profile_signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::HolderDenied,
    );
    assert_denied(
        evaluate_action_with_profile(
            fixture.token.as_str(),
            &request,
            BasaltUcanProtocolProfile::UcanWg100,
            EnforcementAction::DocumentWrite,
            &valid_signature,
            nonce(),
            &services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::ProtocolProfileDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}

#[test]
fn token_and_request_substitutions_invalidate_holder_signature() {
    let fixture = InvocationFixture::direct(document_capability(PARENT_RESOURCE));
    let substituted_token_fixture =
        InvocationFixture::direct(document_capability(DOCUMENT_RESOURCE));
    let request = enforcement_request(DOCUMENT_RESOURCE);
    let substituted_request = enforcement_request(SUBSTITUTED_REQUEST_RESOURCE);
    let signature = sign_request(&fixture, &request, nonce());
    let replay = RecordingReplay::new();

    assert_denied(
        evaluate(
            substituted_token_fixture.token.as_str(),
            &request,
            &signature,
            nonce(),
            services(
                &substituted_token_fixture,
                &NoRevocations,
                &DenyUnknownCaveats,
                &replay,
            ),
        ),
        UcanEnforcementReason::HolderDenied,
    );
    assert_denied(
        evaluate(
            fixture.token.as_str(),
            &substituted_request,
            &signature,
            nonce(),
            services(&fixture, &NoRevocations, &DenyUnknownCaveats, &replay),
        ),
        UcanEnforcementReason::HolderDenied,
    );
    assert_eq!(replay.calls.get(), 0);
}
