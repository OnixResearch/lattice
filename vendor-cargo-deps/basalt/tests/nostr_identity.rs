use basalt::nostr_identity::{
    NostrIdentityPolicyError, nostr_identity_implementation_identity,
    nostr_identity_schema_identity, parse_nostr_identity_policy,
};

const GENERATED_POLICY: &str = include_str!("../contracts/nostr-identity/generated/policy.json");
const GENERATED_IDENTITIES: &str =
    include_str!("../contracts/nostr-identity/generated/identities.json");
const AMBIGUOUS_KEY: &str =
    include_str!("../contracts/nostr-identity/fixtures/negative/ambiguous-key.json");
const SECRET_FIELD: &str =
    include_str!("../contracts/nostr-identity/fixtures/negative/secret-field.json");
const WEAKENED_NONCLAIMS: &str =
    include_str!("../contracts/nostr-identity/fixtures/negative/weakened-nonclaims.json");

// r[verify nostr_identity_bindings.policy_contract]
// r[verify nostr_identity_bindings.final_validation]
#[test]
fn generated_nickel_policy_is_accepted_by_the_rust_core() {
    let policy = parse_nostr_identity_policy(GENERATED_POLICY).expect("generated policy");
    assert_eq!(policy.policy().subjects.len(), 1);
    assert_eq!(policy.policy().subjects[0].authenticators.len(), 2);
}

#[test]
fn generated_producer_identities_match_the_pure_core() {
    let identities: serde_json::Value =
        serde_json::from_str(GENERATED_IDENTITIES).expect("producer identities");
    let policy = parse_nostr_identity_policy(GENERATED_POLICY).expect("generated policy");
    assert_eq!(
        identities["schema"],
        "basalt.nostr-identity-producer-identities.v1"
    );
    assert_eq!(
        identities["ucan_source_revision"],
        basalt::nostr_identity::UCAN_SOURCE_REVISION
    );
    assert_eq!(
        identities["producer_schema_blake3"],
        nostr_identity_schema_identity().hex
    );
    assert_eq!(
        identities["producer_implementation_blake3"],
        nostr_identity_implementation_identity().hex
    );
    assert_eq!(identities["policy_blake3"], policy.policy_identity().hex);
}

#[test]
fn malformed_secret_ambiguous_and_weakened_fixtures_fail_closed() {
    assert!(matches!(
        parse_nostr_identity_policy(SECRET_FIELD),
        Err(NostrIdentityPolicyError::Decode(_))
    ));
    for fixture in [AMBIGUOUS_KEY, WEAKENED_NONCLAIMS] {
        assert!(matches!(
            parse_nostr_identity_policy(fixture),
            Err(NostrIdentityPolicyError::Invalid(_))
        ));
    }
}
