use basalt::parse_policy_json;
use basalt::policy_projection::validate_projection;

const GENERATED_POLICY: &str = include_str!("../contracts/generated/policy.json");

const KEY_ID_MISMATCH: &str =
    include_str!("../contracts/policy-authoring/fixtures/external-json/key-id-mismatch.json");
const EMPTY_ALLOWLIST: &str =
    include_str!("../contracts/policy-authoring/fixtures/external-json/empty-allowlist.json");
const UNKNOWN_BACKEND: &str =
    include_str!("../contracts/policy-authoring/fixtures/external-json/unknown-backend.json");
const DUPLICATE_ENTRY: &str =
    include_str!("../contracts/policy-authoring/fixtures/external-json/duplicate-entry.json");
const MALFORMED_ABILITY: &str =
    include_str!("../contracts/policy-authoring/fixtures/external-json/malformed-ability.json");

fn projection_of(input: &str) -> basalt::policy_projection::ProjectionVerdict {
    let policy = parse_policy_json(input).expect("fixture JSON must parse into the policy DTO");
    validate_projection(&policy)
}

#[test]
fn generated_policy_projection_is_compatible() {
    let verdict = projection_of(GENERATED_POLICY);

    assert!(verdict.is_accepted(), "{:?}", verdict.issues);
    assert!(verdict.issues.is_empty());
}

#[test]
fn external_json_key_id_mismatch_fails_closed() {
    let verdict = projection_of(KEY_ID_MISMATCH);

    assert!(!verdict.is_accepted());
    assert!(
        verdict
            .issues
            .iter()
            .any(|i| i.class == "projection.contracts.key-id-mismatch")
    );
}

#[test]
fn external_json_empty_allowlist_fails_closed() {
    let verdict = projection_of(EMPTY_ALLOWLIST);

    assert!(!verdict.is_accepted());
    assert!(
        verdict
            .issues
            .iter()
            .any(|i| i.class == "projection.contracts.abilities")
    );
}

#[test]
fn external_json_unknown_backend_fails_closed() {
    let verdict = projection_of(UNKNOWN_BACKEND);

    assert!(!verdict.is_accepted());
    assert!(
        verdict
            .issues
            .iter()
            .any(|i| i.class == "projection.backends.unsupported")
    );
}

#[test]
fn external_json_duplicate_entry_fails_closed() {
    let verdict = projection_of(DUPLICATE_ENTRY);

    assert!(!verdict.is_accepted());
    assert!(
        verdict
            .issues
            .iter()
            .any(|i| i.class == "projection.contracts.abilities.duplicate")
    );
}

#[test]
fn external_json_malformed_ability_fails_closed() {
    let verdict = projection_of(MALFORMED_ABILITY);

    assert!(!verdict.is_accepted());
    assert!(
        verdict
            .issues
            .iter()
            .any(|i| i.class == "projection.lexical.ability.path")
    );
}

#[test]
fn unsupported_schema_is_rejected_at_parse() {
    let mut policy: serde_json::Value =
        serde_json::from_str(GENERATED_POLICY).expect("generated policy JSON should parse");
    policy["schema_version"] = serde_json::Value::String("wrong.schema.v1".to_owned());

    let error = parse_policy_json(&policy.to_string())
        .expect_err("unsupported schema must be rejected before projection");

    assert!(
        error
            .to_string()
            .contains("unsupported policy schema version")
    );
}

#[test]
fn empty_prefix_counterexample_fails_admission_and_enforcement() {
    let policy_json = r#"{
      "schema_version": "ucan-nickel-contracts.policy.v1",
      "contracts": {
        "document-write": {
          "id": "document-write",
          "resource_prefixes": [""],
          "abilities": ["document/write"]
        }
      }
    }"#;

    let policy = parse_policy_json(policy_json).expect("DTO decode remains available");
    assert!(!validate_projection(&policy).is_accepted());

    let admit_err =
        basalt::admit_policy_json(policy_json).expect_err("empty prefix must fail admission");
    assert!(matches!(
        admit_err,
        basalt::EnforcementError::InvalidPolicyProjection { ref issues }
            if issues
                .iter()
                .any(|issue| issue.class == "projection.contracts.resource-prefixes.bounded")
    ));

    let resource = "did:key:z6Mkresource/private/report";
    let request = basalt::EnforcementRequest::new("document-write", resource, "document/write")
        .with_capability(basalt::CapabilityGrant::new(resource, "document/write"));
    let enforce_err =
        basalt::enforce(&policy, &request).expect_err("empty prefix must not authorize");
    assert!(matches!(
        enforce_err,
        basalt::EnforcementError::InvalidPolicyProjection { ref issues }
            if issues
                .iter()
                .any(|issue| issue.class == "projection.contracts.resource-prefixes.bounded")
    ));
}

#[test]
fn external_json_negatives_fail_policy_admission_and_enforcement() {
    let cases = [
        (KEY_ID_MISMATCH, "projection.contracts.key-id-mismatch"),
        (EMPTY_ALLOWLIST, "projection.contracts.abilities"),
        (UNKNOWN_BACKEND, "projection.backends.unsupported"),
        (DUPLICATE_ENTRY, "projection.contracts.abilities.duplicate"),
        (MALFORMED_ABILITY, "projection.lexical.ability.path"),
    ];
    let request = basalt::EnforcementRequest::new(
        "document-write",
        "did:key:z6Mkresource/docs/report",
        "document/write",
    )
    .with_capability(basalt::CapabilityGrant::new(
        "did:key:z6Mkresource/docs/report",
        "document/write",
    ));

    for (input, expected_class) in cases {
        let admit_err = basalt::admit_policy_json(input)
            .expect_err("external JSON negative must fail admission");
        assert!(
            matches!(
                admit_err,
                basalt::EnforcementError::InvalidPolicyProjection { ref issues }
                    if issues.iter().any(|issue| issue.class == expected_class)
            ),
            "expected {expected_class}, got {admit_err}"
        );

        let policy = parse_policy_json(input).expect("fixture parses as raw DTO");
        let enforce_err = basalt::enforce(&policy, &request)
            .expect_err("external JSON negative must fail enforcement");
        assert!(
            matches!(
                enforce_err,
                basalt::EnforcementError::InvalidPolicyProjection { ref issues }
                    if issues.iter().any(|issue| issue.class == expected_class)
            ),
            "expected {expected_class}, got {enforce_err}"
        );
    }
}

#[test]
fn direct_construction_and_post_parse_mutation_cannot_bypass_admission() {
    let mut mutated = basalt::default_policy().expect("default policy parses");
    mutated
        .contracts
        .get_mut("document-write")
        .expect("document-write contract exists")
        .resource_prefixes = vec![String::new()];

    let direct = basalt::Policy {
        schema_version: basalt::SUPPORTED_SCHEMA_VERSION.to_owned(),
        contracts: std::collections::BTreeMap::from([(
            "document-write".to_owned(),
            basalt::ContractPolicy {
                id: "document-write".to_owned(),
                description: String::new(),
                resource_prefixes: vec![String::new()],
                abilities: vec!["document/write".to_owned()],
            },
        )]),
        backends: std::collections::BTreeMap::new(),
    };

    let resource = "did:key:z6Mkresource/private/report";
    let request = basalt::EnforcementRequest::new("document-write", resource, "document/write")
        .with_capability(basalt::CapabilityGrant::new(resource, "document/write"));

    for policy in [&mutated, &direct] {
        assert!(matches!(
            basalt::AdmittedPolicy::admit(policy.clone()),
            Err(basalt::EnforcementError::InvalidPolicyProjection { .. })
        ));
        assert!(matches!(
            basalt::enforce(policy, &request),
            Err(basalt::EnforcementError::InvalidPolicyProjection { .. })
        ));
    }

    let admitted = basalt::default_admitted_policy().expect("default policy admits");
    let valid_request = basalt::EnforcementRequest::new(
        "document-write",
        "did:key:z6Mkresource/docs/report",
        "document/write",
    )
    .with_capability(basalt::CapabilityGrant::new(
        "did:key:z6Mkresource/docs/report",
        "document/write",
    ));
    let receipt = basalt::enforce_admitted(&admitted, &valid_request)
        .expect("admitted policy authorizes valid request");
    assert!(receipt.is_allowed());
}

const LEXICAL_NEGATIVE_CASES: [(&str, &str, &str); 13] = [
    (
        "id-with-space",
        include_str!("../contracts/policy-authoring/fixtures/external-json/id-with-space.json"),
        "projection.contract-id.alphabet",
    ),
    (
        "ability-leading-space",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/ability-leading-space.json"
        ),
        "projection.lexical.ability.whitespace",
    ),
    (
        "ability-trailing-space",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/ability-trailing-space.json"
        ),
        "projection.lexical.ability.whitespace",
    ),
    (
        "ability-internal-space",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/ability-internal-space.json"
        ),
        "projection.lexical.ability.whitespace",
    ),
    (
        "ability-tab-control",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/ability-tab-control.json"
        ),
        "projection.lexical.ability.control",
    ),
    (
        "ability-newline-control",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/ability-newline-control.json"
        ),
        "projection.lexical.ability.control",
    ),
    (
        "ability-non-ascii-whitespace",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/ability-non-ascii-whitespace.json"
        ),
        "projection.lexical.ability.whitespace",
    ),
    (
        "resource-prefix-leading-space",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/resource-prefix-leading-space.json"
        ),
        "projection.lexical.resource-prefix.whitespace",
    ),
    (
        "resource-prefix-trailing-space",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/resource-prefix-trailing-space.json"
        ),
        "projection.lexical.resource-prefix.whitespace",
    ),
    (
        "resource-prefix-internal-space",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/resource-prefix-internal-space.json"
        ),
        "projection.lexical.resource-prefix.whitespace",
    ),
    (
        "resource-prefix-tab-control",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/resource-prefix-tab-control.json"
        ),
        "projection.lexical.resource-prefix.control",
    ),
    (
        "resource-prefix-newline-control",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/resource-prefix-newline-control.json"
        ),
        "projection.lexical.resource-prefix.control",
    ),
    (
        "resource-prefix-non-ascii-whitespace",
        include_str!(
            "../contracts/policy-authoring/fixtures/external-json/resource-prefix-non-ascii-whitespace.json"
        ),
        "projection.lexical.resource-prefix.whitespace",
    ),
];

#[test]
fn paired_lexical_counterexamples_and_whitespace_fixtures_fail_closed() {
    for (name, input, expected_class) in LEXICAL_NEGATIVE_CASES {
        let verdict = projection_of(input);
        assert!(
            !verdict.is_accepted(),
            "{name} must fail Rust projection validation"
        );
        assert!(
            verdict.issues.iter().any(|i| i.class == expected_class),
            "{name}: expected issue class {expected_class}, got {:?}",
            verdict.issues
        );
        let admit_err = basalt::admit_policy_json(input)
            .expect_err("invalid lexical fixture must fail admission");
        assert!(
            matches!(
                admit_err,
                basalt::EnforcementError::InvalidPolicyProjection { ref issues }
                    if issues.iter().any(|i| i.class == expected_class)
            ),
            "{name}: unexpected admission error {admit_err}"
        );
    }
}

#[test]
fn valid_normalized_id_and_lexical_controls_are_accepted() {
    let valid_json = r#"{
      "schema_version": "ucan-nickel-contracts.policy.v1",
      "contracts": {
        "Contract_v1.2-alpha": {
          "id": "Contract_v1.2-alpha",
          "description": "Valid normalized ID and lexical values.",
          "resource_prefixes": ["did:key:z6Mkresource/docs/", "onix://state-sync/v1/"],
          "abilities": ["document/write", "state_sync.publish"]
        }
      }
    }"#;

    let admitted =
        basalt::admit_policy_json(valid_json).expect("valid normalized policy must be admitted");
    assert!(validate_projection(admitted.as_policy()).is_accepted());
}
