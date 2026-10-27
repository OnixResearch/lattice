use crate::support::*;

pub const POLICY_JSON: &str = include_str!("../../contracts/process-execution/policy.json");
pub const RESOURCE: &str = concat!(
    "onix://process-execution/",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
);

pub fn request() -> EnforcementRequest {
    let action = EnforcementAction::ProcessExecute;
    EnforcementRequest::new(action.contract_id(), RESOURCE, action.ability())
}

pub fn fixture(resource: &str) -> InvocationFixture {
    InvocationFixture::direct(
        CapabilityDocument::new(
            resource.to_owned(),
            EnforcementAction::ProcessExecute.ability().to_owned(),
        )
        .expect("test execution capability"),
    )
}

pub fn signature(fixture: &InvocationFixture, request: &EnforcementRequest) -> SignerSignature {
    let operation = bind_enforcement_action(EnforcementAction::ProcessExecute, request)
        .expect("exact process binding");
    sign_operation(fixture, request, nonce(), operation)
}

pub fn verify<'a>(
    policy_json: &str,
    fixture: &InvocationFixture,
    request: &'a EnforcementRequest,
    signature: &SignerSignature,
    services: &TestServices<'_>,
) -> Result<LiveUcanEnforcement<'a>, UcanEnforcementFailure> {
    let policy = basalt::parse_policy_json(policy_json).expect("test policy");
    let policy_ref = basalt::authority_core::BasaltPolicyRef::new(
        format!("blake3:{}", blake3::hash(policy_json.as_bytes()).to_hex()),
        &policy.schema_version,
        request.contract(),
    );
    let raw = UcanEnforcementRequest::new(
        request,
        fixture.token.as_str(),
        BasaltUcanProtocolProfile::OnixCompactEd25519V1,
        EnforcementAction::ProcessExecute,
        nonce(),
        signature,
    );
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
    verify_ucan_enforcement_request(&policy, &policy_ref, &raw, &boundaries)
}
