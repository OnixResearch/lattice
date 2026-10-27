use basalt::{
    CapabilityGrant, EnforcementError, EnforcementReceipt, EnforcementRequest,
    SUPPORTED_SCHEMA_VERSION, enforce, parse_policy_json,
};

const POLICY_JSON: &str = r#"{
  "schema_version": "ucan-nickel-contracts.policy.v1",
  "contracts": {
    "document-write": {
      "id": "document-write",
      "description": "Write bounded document resources.",
      "resource_prefixes": ["did:key:z6Mkresource/docs/"],
      "abilities": ["document/write"]
    }
  }
}"#;

fn authorized_request() -> EnforcementRequest {
    EnforcementRequest::new(
        "document-write",
        "did:key:z6Mkresource/docs/report",
        "document/write",
    )
    .with_capability(CapabilityGrant::new(
        "did:key:z6Mkresource/docs/report",
        "document/write",
    ))
}

#[test]
fn public_api_allows_authorized_request() {
    let policy = parse_policy_json(POLICY_JSON).expect("policy should parse");
    assert_eq!(policy.schema_version(), SUPPORTED_SCHEMA_VERSION);
    assert!(
        policy
            .contract("document-write")
            .expect("contract should exist")
            .permits_ability("document/write")
    );

    let request = authorized_request();
    assert_eq!(request.contract(), "document-write");
    assert_eq!(request.capabilities()[0].ability(), "document/write");

    let receipt = enforce(&policy, &request).expect("enforcement should succeed");

    assert_eq!(receipt, EnforcementReceipt::allowed());
    assert!(receipt.is_allowed());
    assert_eq!(receipt.reason(), "allowed");
}

#[test]
fn public_api_denies_policy_mismatch_without_error() {
    let policy = parse_policy_json(POLICY_JSON).expect("policy should parse");
    let request = EnforcementRequest::new(
        "document-write",
        "did:key:z6Mkresource/private/report",
        "document/write",
    )
    .with_capability(CapabilityGrant::new(
        "did:key:z6Mkresource/private/report",
        "document/write",
    ));

    let receipt = enforce(&policy, &request).expect("policy denies should be receipts");

    assert!(receipt.is_denied());
    assert!(receipt.reason().contains("outside contract"));
}

#[test]
fn public_api_reports_malformed_policy_json() {
    let error = parse_policy_json("not json").expect_err("malformed JSON should fail");

    assert!(matches!(error, EnforcementError::PolicyJson { .. }));
}

#[test]
fn public_api_reports_unsupported_policy_schema() {
    let error =
        parse_policy_json(r#"{"schema_version":"ucan-nickel-contracts.policy.v2","contracts":{}}"#)
            .expect_err("unsupported schema should fail");

    assert!(matches!(
        error,
        EnforcementError::UnsupportedSchema { version } if version == "ucan-nickel-contracts.policy.v2"
    ));
}

#[test]
fn public_api_reports_invalid_capability() {
    let policy = parse_policy_json(POLICY_JSON).expect("policy should parse");
    let request = EnforcementRequest::new(
        "document-write",
        "did:key:z6Mkresource/docs/report",
        "document/write",
    )
    .with_capability(CapabilityGrant::new("", "document/write"));

    let error = enforce(&policy, &request).expect_err("invalid capability should fail");

    assert!(matches!(error, EnforcementError::InvalidCapability { .. }));
}

#[test]
fn public_api_dtos_serialize_and_expose_accessors() {
    let grant = CapabilityGrant::new("did:key:z6Mkresource/docs/report", "document/write");
    assert_eq!(grant.resource(), "did:key:z6Mkresource/docs/report");
    assert_eq!(grant.ability(), "document/write");

    let request = EnforcementRequest::new(
        "document-write",
        "did:key:z6Mkresource/docs/report",
        "document/write",
    )
    .with_capability(grant.clone());
    assert_eq!(request.contract(), "document-write");
    assert_eq!(request.resource(), "did:key:z6Mkresource/docs/report");
    assert_eq!(request.ability(), "document/write");
    assert_eq!(request.capabilities(), &[grant]);

    let json = serde_json::to_string(&request).expect("request should serialize");
    let roundtrip: EnforcementRequest =
        serde_json::from_str(&json).expect("request should deserialize");
    assert_eq!(roundtrip, request);
}
