use std::collections::BTreeMap;

use basalt::CallableContractDescriptor;
use basalt::CapabilityGrant;
use basalt::ContractEffectClass;
use basalt::ContractEnvelope;
use basalt::ContractPolicy;
use basalt::EnforcementRequest;
use basalt::Policy;
use basalt::SUPPORTED_SCHEMA_VERSION;
use basalt::SteelEvaluationReceipt;
use basalt::SteelEvaluationRequest;
use basalt::enforce;
use basalt::parse_policy_json;
use basalt::steel_receipt_hash;
use basalt::steel_request_hash;
use basalt::validate_callable_descriptor;
use basalt::validate_steel_evaluation_receipt;
use basalt::validate_steel_evaluation_request;

const CONTRACT_ID: &str = "document-write";
const PREFIX: &str = "did:key:z6Mkresource/docs/";
const ABILITY: &str = "document/write";

fn policy() -> Policy {
    Policy {
        schema_version: SUPPORTED_SCHEMA_VERSION.to_owned(),
        contracts: BTreeMap::from([(CONTRACT_ID.to_owned(), contract(CONTRACT_ID))]),
        backends: BTreeMap::new(),
    }
}

fn contract(id: &str) -> ContractPolicy {
    ContractPolicy {
        id: id.to_owned(),
        description: "generated fixture contract".to_owned(),
        resource_prefixes: vec![PREFIX.to_owned()],
        abilities: vec![ABILITY.to_owned()],
    }
}

fn request(resource: &str, ability: &str, capabilities: Vec<CapabilityGrant>) -> EnforcementRequest {
    EnforcementRequest::new(CONTRACT_ID, resource, ability).with_capabilities(capabilities)
}

fn steel_envelope(source_hash: &str) -> ContractEnvelope {
    ContractEnvelope::new(
        "steel",
        "document-write-predicate",
        "v1",
        source_hash,
        "ucan.contract.input.v1",
        "ucan.contract.output.v1",
        "ucan.contract.receipt.v1",
    )
}

fn callable() -> CallableContractDescriptor {
    CallableContractDescriptor {
        callable_id: "can-write-document".to_owned(),
        arity: 1,
        argument_contracts: vec!["document-request.v1".to_owned()],
        return_contract: "bool".to_owned(),
        required_capabilities: vec![CapabilityGrant::new("did:key:z6Mkresource/docs/report", ABILITY)],
        effect_class: ContractEffectClass::ReadsDeclaredCapabilities,
        content_hash: "blake3:callable-source".to_owned(),
        redaction_class: "safe-summary".to_owned(),
    }
}

#[test]
fn generated_enforcement_boundaries_fail_closed_or_allow_deterministically() {
    let policy = policy();
    let cases = [
        (
            "allowed exact grant",
            request("did:key:z6Mkresource/docs/report", ABILITY, vec![CapabilityGrant::new(
                "did:key:z6Mkresource/docs/report",
                ABILITY,
            )]),
            true,
            "allowed",
        ),
        (
            "prefix sibling is not inside contract",
            request("did:key:z6Mkresource/docs-private/report", ABILITY, vec![CapabilityGrant::new(
                "did:key:z6Mkresource/docs-private/report",
                ABILITY,
            )]),
            false,
            "outside contract",
        ),
        (
            "wrong ability denied before capability allow",
            request("did:key:z6Mkresource/docs/report", "document/delete", vec![CapabilityGrant::new(
                "did:key:z6Mkresource/docs/report",
                "document/delete",
            )]),
            false,
            "outside contract",
        ),
        (
            "missing exact capability denied",
            request("did:key:z6Mkresource/docs/report", ABILITY, vec![CapabilityGrant::new(
                "did:key:z6Mkresource/docs/other",
                ABILITY,
            )]),
            false,
            "ucan capability set",
        ),
        (
            "one of many grants can authorize",
            request("did:key:z6Mkresource/docs/report", ABILITY, vec![
                CapabilityGrant::new("did:key:z6Mkresource/docs/other", ABILITY),
                CapabilityGrant::new("did:key:z6Mkresource/docs/report", ABILITY),
            ]),
            true,
            "allowed",
        ),
    ];

    for (case_name, request, allowed, reason_fragment) in cases {
        let receipt = enforce(&policy, &request).unwrap_or_else(|error| panic!("{case_name}: {error}"));
        assert_eq!(receipt.is_allowed(), allowed, "{case_name}");
        assert!(receipt.reason().contains(reason_fragment), "{case_name}: {:?}", receipt.reason());
    }
}

#[test]
fn generated_policy_shape_errors_do_not_authorize() {
    let unsupported_schema = format!(r#"{{"schema_version":"{SUPPORTED_SCHEMA_VERSION}.next","contracts":{{}}}}"#);
    let malformed_contract = r#"{
        "schema_version":"ucan-nickel-contracts.policy.v1",
        "contracts":{"document-write":{"id":"document-write"}}
    }"#;
    let mismatched_policy = Policy {
        schema_version: SUPPORTED_SCHEMA_VERSION.to_owned(),
        contracts: BTreeMap::from([(CONTRACT_ID.to_owned(), contract("mismatched"))]),
        backends: BTreeMap::new(),
    };

    assert!(parse_policy_json(&unsupported_schema).is_err());
    assert!(parse_policy_json(malformed_contract).is_err());

    let receipt = enforce(
        &mismatched_policy,
        &request("did:key:z6Mkresource/docs/report", ABILITY, vec![CapabilityGrant::new(
            "did:key:z6Mkresource/docs/report",
            ABILITY,
        )]),
    )
    .expect("id mismatch returns a denial receipt");
    assert!(receipt.is_denied());
    assert!(receipt.reason().contains("does not match policy id"));
}

#[test]
fn generated_steel_dto_cases_fail_closed_and_hash_authority_fields() {
    let base_request = SteelEvaluationRequest {
        envelope: steel_envelope("blake3:steel-source"),
        input: serde_json::json!({"resource":"did:key:z6Mkresource/docs/report"}),
        max_input_bytes: 256,
        callable: Some(callable()),
        requested_host_capability: None,
    };
    assert!(validate_steel_evaluation_request(&base_request).is_accepted());

    let base_hash = steel_request_hash(&base_request);
    let mut changed_input = base_request.clone();
    changed_input.input = serde_json::json!({"resource":"did:key:z6Mkresource/docs/other"});
    assert_ne!(base_hash, steel_request_hash(&changed_input));

    let mut oversized = base_request.clone();
    oversized.max_input_bytes = 1;
    assert!(!validate_steel_evaluation_request(&oversized).is_accepted());

    let mut host_request = base_request;
    host_request.requested_host_capability = Some("filesystem.read".to_owned());
    assert!(!validate_steel_evaluation_request(&host_request).is_accepted());

    let mut descriptor = callable();
    descriptor.arity += 1;
    assert!(!validate_callable_descriptor(&descriptor).is_accepted());

    let capability = CapabilityGrant::new("did:key:z6Mkresource/docs/report", ABILITY);
    let mut receipt = SteelEvaluationReceipt {
        backend: "steel".to_owned(),
        contract_id: "document-write-predicate".to_owned(),
        normalized_source_hash: "blake3:steel-source".to_owned(),
        request_hash: base_hash,
        allowed: true,
        reason: "allowed".to_owned(),
        caveat_evidence: BTreeMap::from([("predicate".to_owned(), "passed".to_owned())]),
        effect_class: ContractEffectClass::ReadsDeclaredCapabilities,
        required_capabilities: vec![capability.clone()],
        checked_capabilities: vec![capability],
        receipt_hash: None,
    };
    let receipt_hash = steel_receipt_hash(&receipt);
    receipt.receipt_hash = Some(receipt_hash.clone());
    assert!(validate_steel_evaluation_receipt(&receipt).is_accepted());

    let mut drifted = receipt;
    drifted.reason = "changed".to_owned();
    assert!(!validate_steel_evaluation_receipt(&drifted).is_accepted());
    assert_ne!(receipt_hash, steel_receipt_hash(&drifted));
}
