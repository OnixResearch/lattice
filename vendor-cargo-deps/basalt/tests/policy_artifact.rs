use basalt::{
    CapabilityGrant, DEFAULT_POLICY_JSON, EnforcementRequest, default_policy, enforce,
    parse_policy_json,
};

#[test]
fn packaged_policy_json_is_parseable_and_usable() {
    let policy = default_policy().expect("packaged policy should parse");
    assert!(policy.contract("document-read").is_some());
    assert!(policy.contract("document-write").is_some());

    let request = EnforcementRequest::new(
        "document-read",
        "did:key:z6Mkresource/docs/report",
        "document/read",
    )
    .with_capability(CapabilityGrant::new(
        "did:key:z6Mkresource/docs/report",
        "document/read",
    ));

    let receipt = enforce(&policy, &request).expect("packaged policy should enforce");
    assert!(receipt.is_allowed());
}

#[test]
fn packaged_policy_json_matches_tracked_artifact() {
    let tracked_policy = include_str!("../contracts/generated/policy.json");
    assert_eq!(DEFAULT_POLICY_JSON, tracked_policy);
    parse_policy_json(tracked_policy).expect("tracked artifact should parse");
}

#[test]
// r[verify basalt.kiln_authority.policy.scenario.packaged]
fn packaged_policy_carries_every_kiln_contract() {
    let policy = default_policy().expect("packaged policy should parse");
    for (contract, ability) in [
        ("kiln-claim-acquire", "kiln/claim/acquire"),
        ("kiln-claim-renew", "kiln/claim/renew"),
        ("kiln-dispatch", "kiln/dispatch"),
        ("kiln-candidate-submit", "kiln/candidate/submit"),
        ("kiln-verification-admit", "kiln/verification/admit"),
        ("kiln-attempt-cancel", "kiln/attempt/cancel"),
        ("kiln-publication-publish", "kiln/publication/publish"),
        ("kiln-integration-land", "kiln/integration/land"),
        ("kiln-lifecycle-complete", "kiln/lifecycle/complete"),
    ] {
        let entry = policy.contract(contract).expect("Kiln contract must exist");
        assert!(entry.permits_resource("kiln://implementation/v1/"));
        assert!(entry.permits_resource("kiln://implementation/v2/"));
        assert!(!entry.permits_resource("kiln://source/v1/"));
        assert!(entry.permits_ability(ability));
    }
    for (contract, ability) in [
        ("kiln-source-discover", "kiln/source/discover"),
        ("kiln-source-read", "kiln/source/read"),
        ("kiln-source-history", "kiln/source/history"),
        ("kiln-source-prepare", "kiln/source/prepare"),
        ("kiln-source-modify", "kiln/source/modify"),
        ("kiln-source-create", "kiln/source/create"),
        ("kiln-source-delegate", "kiln/source/delegate"),
        ("kiln-source-result", "kiln/source/result"),
    ] {
        let entry = policy
            .contract(contract)
            .expect("Kiln source contract must exist");
        assert!(entry.permits_resource("kiln://source/v1/"));
        assert!(!entry.permits_resource("kiln://implementation/v2/"));
        assert!(entry.permits_ability(ability));
    }
    // No contract exists for the chain-only implementation delegate ability.
    assert!(policy.contract("kiln-delegate").is_none());
}
