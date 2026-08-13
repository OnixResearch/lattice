use super::*;

pub(crate) struct TestServices<'a> {
    pub(crate) key_resolver: &'a dyn KeyResolver,
    pub(crate) proof_store: &'a dyn ProofStore,
    pub(crate) revocations: &'a dyn RevocationChecker,
    pub(crate) caveats: &'a dyn CaveatPolicySet,
    pub(crate) replay: &'a dyn HolderInvocationReplayAdmission,
    pub(crate) verification_time: VerificationTime,
    pub(crate) verification_limits: VerificationLimits,
    pub(crate) holder_limits: HolderInvocationLimits,
}

pub(crate) fn sign_request(
    fixture: &InvocationFixture,
    request: &EnforcementRequest,
    request_nonce: HolderInvocationNonce,
) -> SignerSignature {
    let operation = bind_enforcement_action(EnforcementAction::DocumentWrite, request)
        .expect("test request must map to a typed operation");
    sign_operation(fixture, request, request_nonce, operation)
}

pub(crate) fn sign_operation(
    fixture: &InvocationFixture,
    request: &EnforcementRequest,
    request_nonce: HolderInvocationNonce,
    operation: OperationBinding,
) -> SignerSignature {
    sign_holder_invocation(
        &HolderInvocationSigningRequest::new(
            fixture.token.as_str(),
            &fixture.audience,
            request.resource(),
            request.ability(),
            operation,
            request_nonce,
        ),
        &fixture.holder,
        VerificationLimits::default(),
        HolderInvocationLimits::default(),
    )
    .expect("test holder request must sign")
}

pub(crate) fn services<'a>(
    fixture: &'a InvocationFixture,
    revocations: &'a dyn RevocationChecker,
    caveats: &'a dyn CaveatPolicySet,
    replay: &'a dyn HolderInvocationReplayAdmission,
) -> TestServices<'a> {
    TestServices {
        key_resolver: &fixture.keys,
        proof_store: &fixture.proofs,
        revocations,
        caveats,
        replay,
        verification_time: VerificationTime::from_unix_seconds(VERIFY_AT_SECONDS),
        verification_limits: VerificationLimits::default(),
        holder_limits: HolderInvocationLimits::default(),
    }
}

// Own the time and limit snapshot used by one adapter evaluation.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn evaluate<'request>(
    compact_token: &str,
    request: &'request EnforcementRequest,
    signature: &SignerSignature,
    request_nonce: HolderInvocationNonce,
    services: TestServices<'_>,
) -> Result<LiveUcanEnforcement<'request>, UcanEnforcementFailure> {
    evaluate_action(compact_token, request, EnforcementAction::DocumentWrite, signature, request_nonce, &services)
}

pub(crate) fn evaluate_action<'request>(
    compact_token: &str,
    request: &'request EnforcementRequest,
    action: EnforcementAction,
    signature: &SignerSignature,
    request_nonce: HolderInvocationNonce,
    services: &TestServices<'_>,
) -> Result<LiveUcanEnforcement<'request>, UcanEnforcementFailure> {
    let policy = default_policy().expect("embedded policy must parse");
    let policy_ref = default_authority_policy_ref(request.contract());
    let raw_request = UcanEnforcementRequest::new(request, compact_token, action, request_nonce, signature);
    let boundaries = UcanEnforcementBoundaries {
        verification_time: services.verification_time,
        key_resolver: services.key_resolver,
        proof_store: services.proof_store,
        revocations: services.revocations,
        caveat_policies: services.caveats,
        replay: services.replay,
        verification_limits: services.verification_limits,
        holder_limits: services.holder_limits,
    };
    verify_ucan_enforcement_request(&policy, &policy_ref, &raw_request, &boundaries)
}

pub(crate) fn assert_denied(
    result: Result<LiveUcanEnforcement<'_>, UcanEnforcementFailure>,
    expected: UcanEnforcementReason,
) -> UcanEnforcementReceipt {
    let error = result.expect_err("negative adapter case must deny");
    assert_eq!(error.reason(), expected);
    assert!(!error.receipt().is_allowed());
    error.receipt().clone()
}
