use basalt::CapabilityGrant;
use basalt::EnforcementError;
use basalt::EnforcementRequest;
use basalt::default_policy;
use basalt::enforce;
use basalt::parse_policy_json;

fn authorized_document_write() -> EnforcementRequest {
    EnforcementRequest::new("document-write", "did:key:z6Mkresource/docs/report", "document/write")
        .with_capability(CapabilityGrant::new("did:key:z6Mkresource/docs/report", "document/write"))
}

#[test]
fn downstream_app_uses_packaged_policy_and_public_api() {
    let policy = default_policy().expect("packaged policy should parse");
    let receipt = enforce(&policy, &authorized_document_write()).expect("enforcement should run");

    assert!(receipt.is_allowed());
    assert_eq!(receipt.reason(), "allowed");
}

#[test]
fn downstream_app_uses_supplied_policy_json_and_public_api() {
    let policy_json = r#"{
      "schema_version": "ucan-nickel-contracts.policy.v1",
      "contracts": {
        "notes-read": {
          "id": "notes-read",
          "description": "Read note resources.",
          "resource_prefixes": ["did:key:z6Mkresource/notes/"],
          "abilities": ["note/read"]
        }
      }
    }"#;
    let policy = parse_policy_json(policy_json).expect("consumer policy should parse");
    let request = EnforcementRequest::new("notes-read", "did:key:z6Mkresource/notes/today", "note/read")
        .with_capability(CapabilityGrant::new("did:key:z6Mkresource/notes/today", "note/read"));

    let receipt = enforce(&policy, &request).expect("enforcement should run");

    assert!(receipt.is_allowed());
}

#[test]
fn downstream_malformed_policy_fails_closed() {
    let error = parse_policy_json("{ not-json").expect_err("malformed policy must fail");

    assert!(matches!(error, EnforcementError::PolicyJson { .. }));
}

#[test]
fn downstream_unsupported_schema_fails_closed() {
    let error = parse_policy_json(r#"{"schema_version":"ucan-nickel-contracts.policy.v2","contracts":{}}"#)
        .expect_err("unsupported schema must fail");

    assert!(matches!(error, EnforcementError::UnsupportedSchema { .. }));
}

#[test]
fn downstream_wrong_resource_is_denied_receipt() {
    let policy = default_policy().expect("packaged policy should parse");
    let request = EnforcementRequest::new("document-write", "did:key:z6Mkresource/private/report", "document/write")
        .with_capability(CapabilityGrant::new("did:key:z6Mkresource/private/report", "document/write"));

    let receipt = enforce(&policy, &request).expect("policy denial should be a receipt");

    assert!(receipt.is_denied());
    assert!(receipt.reason().contains("outside contract"));
}

#[test]
fn downstream_wrong_ability_is_denied_receipt() {
    let policy = default_policy().expect("packaged policy should parse");
    let request = EnforcementRequest::new("document-write", "did:key:z6Mkresource/docs/report", "document/delete")
        .with_capability(CapabilityGrant::new("did:key:z6Mkresource/docs/report", "document/delete"));

    let receipt = enforce(&policy, &request).expect("policy denial should be a receipt");

    assert!(receipt.is_denied());
    assert!(receipt.reason().contains("outside contract"));
}

#[test]
fn downstream_missing_capability_is_denied_receipt() {
    let policy = default_policy().expect("packaged policy should parse");
    let request = EnforcementRequest::new("document-write", "did:key:z6Mkresource/docs/report", "document/write");

    let receipt = enforce(&policy, &request).expect("missing capability should deny");

    assert!(receipt.is_denied());
    assert!(receipt.reason().contains("ucan capability set"));
}
