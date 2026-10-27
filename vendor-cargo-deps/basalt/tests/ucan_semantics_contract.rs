use basalt::ucan_profile::BasaltUcanProtocolProfile;
use basalt::ucan_semantics::{
    BASALT_POLICY_BLAKE3, REQUIRED_NON_CLAIMS, SemanticsReasonClass, UCAN_RUNTIME_REVISION,
    UCAN_SUPPORT_CATALOG_BLAKE3, UCAN_SUPPORT_CATALOG_REVISION, UcanSemanticsFixture,
    default_policy_blake3, replay_semantics_fixture, replay_semantics_fixture_json,
};

const ALLOW_FIXTURE: &str =
    include_str!("../contracts/ucan-semantics/fixtures/generated/positive/allow.json");
const CAPABILITY_DENY_FIXTURE: &str =
    include_str!("../contracts/ucan-semantics/fixtures/generated/positive/capability-deny.json");
const OVER_DELEGATION_FIXTURE: &str =
    include_str!("../contracts/ucan-semantics/fixtures/generated/negative/over-delegation.json");
const WILDCARD_REQUEST_FIXTURE: &str =
    include_str!("../contracts/ucan-semantics/fixtures/generated/negative/wildcard-request.json");
const MISSING_CAVEAT_POLICY_FIXTURE: &str = include_str!(
    "../contracts/ucan-semantics/fixtures/generated/negative/missing-caveat-policy.json"
);
const STALE_PROOF_FIXTURE: &str =
    include_str!("../contracts/ucan-semantics/fixtures/generated/negative/stale-proof.json");
const REPLAY_DENIED_FIXTURE: &str =
    include_str!("../contracts/ucan-semantics/fixtures/generated/negative/replay-denied.json");

fn parse_fixture(input: &str) -> UcanSemanticsFixture {
    serde_json::from_str(input).expect("generated UCAN semantics fixture should parse")
}

#[test]
fn replays_matching_policy_allow_and_capability_deny() {
    let allow = replay_semantics_fixture_json(ALLOW_FIXTURE);
    let deny = replay_semantics_fixture_json(CAPABILITY_DENY_FIXTURE);

    assert!(allow.is_accepted());
    assert!(allow.enforcement.is_allowed());
    assert_eq!(allow.reason_class, SemanticsReasonClass::Allowed);
    assert!(allow.issues.is_empty());

    assert!(deny.is_accepted());
    assert!(deny.enforcement.is_denied());
    assert_eq!(
        deny.reason_class,
        SemanticsReasonClass::UcanCapabilityDenied
    );
    assert!(deny.issues.is_empty());
}

#[test]
fn fails_closed_for_all_supplied_ucan_denial_facts() {
    let cases = [
        (
            OVER_DELEGATION_FIXTURE,
            SemanticsReasonClass::DelegationDenied,
        ),
        (
            WILDCARD_REQUEST_FIXTURE,
            SemanticsReasonClass::WildcardRequestDenied,
        ),
        (
            MISSING_CAVEAT_POLICY_FIXTURE,
            SemanticsReasonClass::CaveatDenied,
        ),
        (STALE_PROOF_FIXTURE, SemanticsReasonClass::ProofDenied),
        (REPLAY_DENIED_FIXTURE, SemanticsReasonClass::ReplayDenied),
    ];

    for (fixture, expected_reason) in cases {
        let report = replay_semantics_fixture_json(fixture);

        assert!(report.is_accepted());
        assert!(report.enforcement.is_denied());
        assert_eq!(report.reason_class, expected_reason);
        assert!(report.issues.is_empty());
    }
}

#[test]
fn rejects_malformed_json_with_deterministic_diagnostic() {
    let malformed = r#"{"schema":"basalt.ucan-semantics-fixture.v2"}"#;
    let first = replay_semantics_fixture_json(malformed);
    let second = replay_semantics_fixture_json(malformed);

    assert_eq!(first, second);
    assert!(!first.is_accepted());
    assert!(first.enforcement.is_denied());
    assert_eq!(first.reason_class, SemanticsReasonClass::FixtureInvalid);
    assert_eq!(first.issues.len(), 1);
    assert_eq!(first.issues[0].code, "semantics.fixture.json");
}

#[test]
fn rejects_missing_unknown_and_cross_profile_selection() {
    let value: serde_json::Value =
        serde_json::from_str(ALLOW_FIXTURE).expect("generated fixture must be JSON");
    let mut missing = value.clone();
    missing
        .as_object_mut()
        .expect("fixture must be an object")
        .remove("protocol_profile");
    let mut unknown = value;
    unknown["protocol_profile"] = serde_json::Value::String("unknown-profile".to_owned());

    for malformed in [missing, unknown] {
        let report = replay_semantics_fixture_json(
            &serde_json::to_string(&malformed).expect("fixture mutation must serialize"),
        );
        assert!(!report.is_accepted());
        assert_eq!(report.issues[0].code, "semantics.fixture.json");
    }

    let mut substituted = parse_fixture(ALLOW_FIXTURE);
    substituted.protocol_profile = BasaltUcanProtocolProfile::UcanWg100;
    let report = replay_semantics_fixture(&substituted);
    assert!(!report.is_accepted());
    assert_eq!(report.issues[0].code, "semantics.profile.mismatch");
}

#[test]
fn rejects_stale_identity_and_weakened_non_claims() {
    let mut fixture = parse_fixture(ALLOW_FIXTURE);
    fixture.support_catalog_blake3 = "stale".to_owned();
    fixture.non_claims.pop();

    let report = replay_semantics_fixture(&fixture);
    let issue_codes = report
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();

    assert!(!report.is_accepted());
    assert!(report.enforcement.is_denied());
    assert_eq!(report.reason_class, SemanticsReasonClass::FixtureInvalid);
    assert_eq!(
        issue_codes,
        ["semantics.identity.catalog-blake3", "semantics.non-claims"]
    );
}

#[test]
fn rejects_expected_result_incompatible_with_decision_facts() {
    let mut fixture = parse_fixture(OVER_DELEGATION_FIXTURE);
    fixture.expected.allowed = true;
    fixture.expected.reason_class = SemanticsReasonClass::Allowed;

    let report = replay_semantics_fixture(&fixture);

    assert!(!report.is_accepted());
    assert!(report.enforcement.is_denied());
    assert_eq!(report.reason_class, SemanticsReasonClass::FixtureInvalid);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, "semantics.expected.precondition");
}

#[test]
fn rejects_expected_result_incompatible_with_replay() {
    let mut fixture = parse_fixture(ALLOW_FIXTURE);
    fixture.expected.allowed = false;
    fixture.expected.reason_class = SemanticsReasonClass::PolicyDenied;

    let report = replay_semantics_fixture(&fixture);

    assert!(!report.is_accepted());
    assert!(report.enforcement.is_allowed());
    assert_eq!(report.reason_class, SemanticsReasonClass::Allowed);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].code, "semantics.expected.mismatch");
}

#[test]
fn binds_runtime_catalog_and_policy_identities() {
    let fixture = parse_fixture(ALLOW_FIXTURE);
    let required_non_claims = REQUIRED_NON_CLAIMS.map(str::to_owned);

    assert_eq!(fixture.runtime_ucan_revision, UCAN_RUNTIME_REVISION);
    assert_eq!(
        fixture.support_catalog_revision,
        UCAN_SUPPORT_CATALOG_REVISION
    );
    assert_eq!(fixture.support_catalog_blake3, UCAN_SUPPORT_CATALOG_BLAKE3);
    assert_eq!(fixture.basalt_policy_blake3, BASALT_POLICY_BLAKE3);
    assert_eq!(default_policy_blake3(), BASALT_POLICY_BLAKE3);
    assert_eq!(fixture.non_claims, required_non_claims);
}
