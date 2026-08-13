use basalt::CapabilityGrant;
use basalt::DEFAULT_POLICY_JSON;
use basalt::EnforcementRequest;
use basalt::default_policy;
use basalt::enforce;
use basalt::parse_policy_json;

#[test]
fn packaged_policy_json_is_parseable_and_usable() {
    let policy = default_policy().expect("packaged policy should parse");
    assert!(policy.contract("document-read").is_some());
    assert!(policy.contract("document-write").is_some());

    let request = EnforcementRequest::new("document-read", "did:key:z6Mkresource/docs/report", "document/read")
        .with_capability(CapabilityGrant::new("did:key:z6Mkresource/docs/report", "document/read"));

    let receipt = enforce(&policy, &request).expect("packaged policy should enforce");
    assert!(receipt.is_allowed());
}

#[test]
fn packaged_policy_json_matches_tracked_artifact() {
    let tracked_policy = include_str!("../contracts/generated/policy.json");
    assert_eq!(DEFAULT_POLICY_JSON, tracked_policy);
    parse_policy_json(tracked_policy).expect("tracked artifact should parse");
}
