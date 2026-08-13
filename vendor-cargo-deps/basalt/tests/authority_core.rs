use basalt::CapabilityGrant;
use basalt::EnforcementRequest;
use basalt::authority_core::AUTHORITY_DECISION_SCHEMA;
use basalt::authority_core::AUTHORITY_FIXTURE_SCHEMA;
use basalt::authority_core::AuthorityAdapterError;
use basalt::authority_core::AuthorityDecision;
use basalt::authority_core::AuthorityDenyDiagnostic;
use basalt::authority_core::AuthorityEvidence;
use basalt::authority_core::AuthorityInput;
use basalt::authority_core::REQUIRED_AUTHORITY_NON_CLAIMS;
use basalt::authority_core::VerificationRefDisposition;
use basalt::authority_core::VerifiedGrantBinding;
use basalt::authority_core::adapt_enforcement_request;
use basalt::authority_core::default_authority_policy_ref;
use basalt::authority_core::evaluate_authority;
use basalt::default_policy;
use serde::Deserialize;

const DIRECT_MATCH: &str = include_str!("../contracts/authority-core/fixtures/generated/positive/direct-match.json");
const DELEGATED_MATCH: &str =
    include_str!("../contracts/authority-core/fixtures/generated/positive/delegated-match.json");
const MISSING_GRANT: &str = include_str!("../contracts/authority-core/fixtures/generated/negative/missing-grant.json");
const WRONG_RESOURCE: &str =
    include_str!("../contracts/authority-core/fixtures/generated/negative/wrong-resource.json");
const WRONG_ABILITY: &str = include_str!("../contracts/authority-core/fixtures/generated/negative/wrong-ability.json");
const STALE_VERIFICATION_REF: &str =
    include_str!("../contracts/authority-core/fixtures/generated/negative/stale-verification-ref.json");
const CAVEAT_DENIED: &str = include_str!("../contracts/authority-core/fixtures/generated/negative/caveat-denied.json");
const REPLAY_DENIED: &str = include_str!("../contracts/authority-core/fixtures/generated/negative/replay-denied.json");
const MISSING_POLICY_REF: &str =
    include_str!("../contracts/authority-core/fixtures/generated/negative/missing-policy-ref.json");
const MALFORMED_REQUEST_REF: &str =
    include_str!("../contracts/authority-core/fixtures/generated/negative/malformed-request-ref.json");
const AUTHORITY_OVERCLAIM: &str =
    include_str!("../contracts/authority-core/fixtures/generated/negative/authority-overclaim.json");
const MISSING_VERIFICATION_RECEIPT: &str =
    include_str!("../contracts/authority-core/fixtures/generated/negative/missing-verification-receipt.json");
const VERIFICATION_REQUEST_MISMATCH: &str =
    include_str!("../contracts/authority-core/fixtures/generated/negative/verification-request-mismatch.json");

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityFixture {
    schema: String,
    fixture_id: String,
    input: AuthorityInput,
    expected: ExpectedDecision,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedDecision {
    allowed: bool,
    diagnostic: String,
}

fn parse_fixture(input: &str) -> AuthorityFixture {
    serde_json::from_str(input).expect("generated authority fixture should parse")
}

fn evaluate_fixture(fixture: &AuthorityFixture) -> AuthorityDecision {
    evaluate_input(&fixture.input)
}

fn evaluate_input(input: &AuthorityInput) -> AuthorityDecision {
    let policy = default_policy().expect("embedded policy should parse");
    let policy_ref = default_authority_policy_ref(input.request.contract_id.as_str());
    evaluate_authority(&policy, &policy_ref, input)
}

fn actual_diagnostic(decision: &AuthorityDecision) -> &str {
    decision.diagnostic.map_or("none", AuthorityDenyDiagnostic::as_str)
}

#[test]
fn positive_fixtures_admit_only_the_exact_request() {
    for input in [DIRECT_MATCH, DELEGATED_MATCH] {
        let fixture = parse_fixture(input);
        let decision = evaluate_fixture(&fixture);

        assert_eq!(fixture.schema, AUTHORITY_FIXTURE_SCHEMA);
        assert!(fixture.expected.allowed);
        assert_eq!(fixture.expected.diagnostic, "none");
        assert!(decision.is_allowed(), "{}", fixture.fixture_id);
        assert_eq!(decision.schema, AUTHORITY_DECISION_SCHEMA);
        assert_eq!(decision.binding.as_ref(), Some(&fixture.input.request));
        assert_eq!(decision.request_ref, fixture.input.request.request_ref);
        assert_eq!(decision.supported_claim, "exact-request-only");
        assert_eq!(decision.non_claims, REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned));
    }
}

#[test]
fn negative_fixtures_fail_closed_with_stable_binding_diagnostics() {
    let cases = [
        MISSING_GRANT,
        WRONG_RESOURCE,
        WRONG_ABILITY,
        STALE_VERIFICATION_REF,
        CAVEAT_DENIED,
        REPLAY_DENIED,
        MISSING_POLICY_REF,
        MALFORMED_REQUEST_REF,
        AUTHORITY_OVERCLAIM,
        MISSING_VERIFICATION_RECEIPT,
        VERIFICATION_REQUEST_MISMATCH,
    ];

    for input in cases {
        let fixture = parse_fixture(input);
        let decision = evaluate_fixture(&fixture);

        assert!(!fixture.expected.allowed);
        assert!(decision.is_denied(), "{}", fixture.fixture_id);
        assert_eq!(decision.schema, AUTHORITY_DECISION_SCHEMA);
        assert!(decision.binding.is_none());
        assert_eq!(actual_diagnostic(&decision), fixture.expected.diagnostic);
        assert_eq!(decision.supported_claim, "no-authority-admitted");
        assert_eq!(decision.non_claims, REQUIRED_AUTHORITY_NON_CLAIMS.map(str::to_owned));
    }
}

#[test]
fn mutated_verified_fact_edges_fail_closed() {
    let base_input = parse_fixture(DIRECT_MATCH).input;
    let mut malformed_grant = base_input.clone();
    malformed_grant.verified_grants[0].grant_ref = "blake3:short".to_owned();
    let mut omitted_grant_binding = base_input.clone();
    omitted_grant_binding.verification_receipts[0].verified_grant_refs =
        vec!["blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned()];
    let mut rejected_receipt = base_input.clone();
    rejected_receipt.verification_receipts[0].disposition = VerificationRefDisposition::Rejected;
    let mut multiple_policies = base_input;
    multiple_policies.policy_refs.push(multiple_policies.policy_refs[0].clone());

    let cases = [
        ("malformed grant reference", AuthorityDenyDiagnostic::MalformedVerifiedGrant, malformed_grant),
        (
            "receipt omits selected grant reference",
            AuthorityDenyDiagnostic::VerificationGrantMismatch,
            omitted_grant_binding,
        ),
        ("rejected verification receipt", AuthorityDenyDiagnostic::RejectedVerificationRef, rejected_receipt),
        ("multiple policy references", AuthorityDenyDiagnostic::MultiplePolicyRefs, multiple_policies),
    ];

    for (name, expected, input) in cases {
        let decision = evaluate_input(&input);
        assert!(decision.is_denied(), "{name}");
        assert_eq!(decision.diagnostic, Some(expected), "{name}");
        assert!(decision.binding.is_none(), "{name}");
    }
}

#[test]
fn malformed_verified_fact_json_is_rejected_before_evaluation() {
    let mut fixture: serde_json::Value = serde_json::from_str(DIRECT_MATCH).expect("fixture JSON should parse");
    fixture["input"]["verified_grants"][0]
        .as_object_mut()
        .expect("verified grant should be an object")
        .remove("grant_ref");

    let error = serde_json::from_value::<AuthorityFixture>(fixture)
        .expect_err("missing grant reference must be rejected by the DTO boundary");

    assert!(error.to_string().contains("missing field `grant_ref`"));
}

#[test]
fn existing_enforcement_request_adapter_preserves_public_fields() {
    let fixture = parse_fixture(DIRECT_MATCH);
    let grant = &fixture.input.verified_grants[0];
    let request = EnforcementRequest::new(
        fixture.input.request.contract_id.clone(),
        fixture.input.request.resource.clone(),
        fixture.input.request.ability.clone(),
    )
    .with_capability(CapabilityGrant::new(grant.resource.clone(), grant.ability.clone()));
    let evidence = AuthorityEvidence {
        request_ref: fixture.input.request.request_ref.clone(),
        grant_bindings: vec![VerifiedGrantBinding {
            grant_ref: grant.grant_ref.clone(),
            verification_receipt_ref: grant.verification_receipt_ref.clone(),
            kind: grant.kind,
        }],
        verification_receipts: fixture.input.verification_receipts.clone(),
        policy_refs: fixture.input.policy_refs.clone(),
        caveat: fixture.input.caveat,
        replay: fixture.input.replay,
        claims: fixture.input.claims.clone(),
        non_claims: fixture.input.non_claims.clone(),
    };

    let adapted = adapt_enforcement_request(&request, evidence)
        .expect("one verified binding should adapt one existing capability");
    let policy = default_policy().expect("embedded policy should parse");
    let policy_ref = default_authority_policy_ref(request.contract.as_str());
    let decision = evaluate_authority(&policy, &policy_ref, &adapted);

    assert_eq!(adapted.request.contract_id, request.contract);
    assert_eq!(adapted.request.resource, request.resource);
    assert_eq!(adapted.request.ability, request.ability);
    assert_eq!(adapted.verified_grants.len(), request.capabilities.len());
    assert_eq!(adapted.verified_grants[0].resource, request.capabilities[0].resource);
    assert_eq!(adapted.verified_grants[0].ability, request.capabilities[0].ability);
    assert!(decision.is_allowed());
}

#[test]
fn adapter_rejects_unbound_existing_capabilities() {
    let fixture = parse_fixture(DIRECT_MATCH);
    let request = EnforcementRequest::new(
        fixture.input.request.contract_id.clone(),
        fixture.input.request.resource.clone(),
        fixture.input.request.ability.clone(),
    )
    .with_capability(CapabilityGrant::new(
        fixture.input.request.resource.clone(),
        fixture.input.request.ability.clone(),
    ));
    let evidence = AuthorityEvidence {
        request_ref: fixture.input.request.request_ref,
        grant_bindings: Vec::new(),
        verification_receipts: fixture.input.verification_receipts,
        policy_refs: fixture.input.policy_refs,
        caveat: fixture.input.caveat,
        replay: fixture.input.replay,
        claims: fixture.input.claims,
        non_claims: fixture.input.non_claims,
    };

    let error = adapt_enforcement_request(&request, evidence)
        .expect_err("capability without a verified reference binding must fail");

    assert_eq!(error, AuthorityAdapterError::GrantBindingCountMismatch);
}

#[test]
fn raw_token_material_is_not_an_authority_input_field() {
    let mut fixture: serde_json::Value = serde_json::from_str(DIRECT_MATCH).expect("fixture JSON should parse");
    fixture["input"]["raw_token"] = serde_json::Value::String("header.payload.signature".to_owned());

    let error = serde_json::from_value::<AuthorityFixture>(fixture)
        .expect_err("raw token material must be rejected at the pure DTO boundary");

    assert!(error.to_string().contains("unknown field `raw_token`"));
}

#[test]
fn trusted_policy_reference_mismatch_denies() {
    let fixture = parse_fixture(DIRECT_MATCH);
    let policy = default_policy().expect("embedded policy should parse");
    let mut trusted = default_authority_policy_ref(fixture.input.request.contract_id.as_str());
    trusted.artifact_ref = "blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned();

    let decision = evaluate_authority(&policy, &trusted, &fixture.input);

    assert!(decision.is_denied());
    assert_eq!(decision.diagnostic, Some(AuthorityDenyDiagnostic::PolicyRefMismatch));
}
