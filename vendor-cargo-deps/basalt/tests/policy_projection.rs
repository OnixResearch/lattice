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
