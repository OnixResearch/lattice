use basalt::policy_identity::build_policy_identity_indexes;
use basalt::{Policy, parse_policy_json};

const VALID_POLICY: &str =
    include_str!("../contracts/policy-identity/fixtures/positive/valid-policy.json");
const DUP_CONTRACT: &str =
    include_str!("../contracts/policy-identity/fixtures/negative/duplicate-contract-id.json");
const MISMATCH_KEY: &str =
    include_str!("../contracts/policy-identity/fixtures/negative/contract-key-id-mismatch.json");
const UNSUPPORTED_BACKEND: &str =
    include_str!("../contracts/policy-identity/fixtures/negative/unsupported-backend-id.json");

fn policy(input: &str) -> Policy {
    parse_policy_json(input).expect("fixture policy should parse")
}

#[test]
fn replays_positive_valid_policy_index() {
    let indexes = build_policy_identity_indexes(&policy(VALID_POLICY))
        .expect("positive fixture should build valid indexes");

    assert_eq!(indexes.contract_count(), 1);
    assert_eq!(indexes.backend_count(), 2);
    assert!(indexes.contract("document-write").is_some());
    assert!(indexes.backend("nickel").is_some());
    assert!(indexes.backend("steel").is_some());
}

#[test]
fn replays_negative_duplicate_contract_id() {
    let issues = build_policy_identity_indexes(&policy(DUP_CONTRACT))
        .expect_err("duplicate contract id fixture must fail closed");
    assert!(
        issues
            .iter()
            .any(|i| i.class == "policy-identity.contract.duplicate-id")
    );
}

#[test]
fn replays_negative_key_value_mismatch() {
    let issues = build_policy_identity_indexes(&policy(MISMATCH_KEY))
        .expect_err("key/value mismatch fixture must fail closed");
    assert!(
        issues
            .iter()
            .any(|i| i.class == "policy-identity.contract.key-value-id-mismatch")
    );
}

#[test]
fn replays_negative_unsupported_backend() {
    let issues = build_policy_identity_indexes(&policy(UNSUPPORTED_BACKEND))
        .expect_err("unsupported backend fixture must fail closed");
    assert!(
        issues
            .iter()
            .any(|i| i.class == "policy-identity.backend.unsupported")
    );
}
