use basalt::secretspec_issuer::IssuerKeyDeclaration;
use basalt::secretspec_issuer::IssuerResolutionReason;
use basalt::secretspec_issuer::RotationWindow;
use basalt::secretspec_issuer::classify_resolution;
use basalt::secretspec_issuer::validate_declaration;
use basalt::secretspec_issuer::validate_rotation_window;

const POSITIVE_DECLARATION: &str =
    include_str!("../contracts/secretspec-issuer/fixtures/positive/issuer-key-declaration.json");
const POSITIVE_ROTATION: &str =
    include_str!("../contracts/secretspec-issuer/fixtures/positive/rotation-overlap-window.json");
const NEGATIVE_MISSING_REQUIRED: &str =
    include_str!("../contracts/secretspec-issuer/fixtures/negative/missing-required-key.json");
const NEGATIVE_MISSING_DECLARED: &str =
    include_str!("../contracts/secretspec-issuer/fixtures/negative/missing-declared-key.json");
const NEGATIVE_WRONG_SCOPE: &str = include_str!("../contracts/secretspec-issuer/fixtures/negative/wrong-scope.json");
const NEGATIVE_PROVIDER_FAILURE: &str =
    include_str!("../contracts/secretspec-issuer/fixtures/negative/provider-failure.json");

fn declaration() -> IssuerKeyDeclaration {
    serde_json::from_str(POSITIVE_DECLARATION).expect("positive issuer-key declaration fixture should parse")
}

#[test]
fn replays_positive_issuer_key_load_and_rotation_fixtures() {
    let declaration = declaration();
    assert!(validate_declaration(&declaration).is_empty());

    let loaded = classify_resolution(&declaration, &ok_envelope(), 256);
    assert!(loaded.is_allowed());
    assert_eq!(loaded.reason, IssuerResolutionReason::Loaded);

    let rotation: RotationWindow =
        serde_json::from_str(POSITIVE_ROTATION).expect("positive rotation fixture should parse");
    assert!(validate_rotation_window(&rotation).is_empty());
    assert!(rotation.admits("oidc-signing-key-2"));
}

#[test]
fn fails_closed_for_all_negative_resolution_fixtures() {
    let declaration = declaration();

    let cases = [
        (NEGATIVE_MISSING_REQUIRED, IssuerResolutionReason::Missing),
        (NEGATIVE_MISSING_DECLARED, IssuerResolutionReason::Missing),
        (NEGATIVE_WRONG_SCOPE, IssuerResolutionReason::WrongScope),
        (NEGATIVE_PROVIDER_FAILURE, IssuerResolutionReason::ProviderFailure),
    ];

    for (fixture, expected) in cases {
        let result = classify_resolution(&declaration, fixture, 256);
        assert!(!result.is_allowed());
        assert_eq!(result.reason, expected, "fixture should fail closed as {expected:?}");
    }
}

fn ok_envelope() -> String {
    serde_json::json!({
        "ok": true,
        "response": {
            "schema_version": 2,
            "provider": "env://",
            "profile": "production",
            "scope": "basalt-oidc-exchange",
            "secrets": {
                "oidc-signing-key": {
                    "value": "k",
                    "as_path": false,
                    "source": "provider"
                }
            },
            "missing_required": [],
            "missing_optional": []
        }
    })
    .to_string()
}
