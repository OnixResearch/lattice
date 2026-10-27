use basalt::license_consumption::{
    LICENSE_CONSUMPTION_FIXTURE_SCHEMA, LICENSE_CONSUMPTION_RECEIPT_SCHEMA,
    LicenseConsumptionReceipt, LicenseReviewAction, REQUIRED_LICENSE_CONSUMPTION_NON_CLAIMS,
    validate_license_consumption,
};
use serde::Deserialize;

const ACKNOWLEDGED_AGPL: &str = include_str!(
    "../contracts/license-consumption/fixtures/generated/positive/acknowledged-agpl.json"
);
const MISSING_REVIEW: &str = include_str!(
    "../contracts/license-consumption/fixtures/generated/negative/missing-review.json"
);
const WRONG_ARTIFACT: &str = include_str!(
    "../contracts/license-consumption/fixtures/generated/negative/wrong-artifact-identity.json"
);
const STALE_METADATA: &str = include_str!(
    "../contracts/license-consumption/fixtures/generated/negative/stale-license-metadata.json"
);
const WEAKENED_NON_CLAIMS: &str = include_str!(
    "../contracts/license-consumption/fixtures/generated/negative/weakened-non-claims.json"
);
const WRONG_LICENSE: &str = include_str!(
    "../contracts/license-consumption/fixtures/generated/negative/wrong-license-expression.json"
);

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LicenseFixture {
    schema: String,
    fixture_id: String,
    input: LicenseConsumptionReceipt,
    expected: ExpectedVerdict,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedVerdict {
    accepted: bool,
    diagnostic: String,
}

fn parse_fixture(input: &str) -> LicenseFixture {
    serde_json::from_str(input).expect("generated license-consumption fixture should parse")
}

fn issue_codes(verdict: &basalt::license_consumption::LicenseConsumptionVerdict) -> Vec<String> {
    verdict
        .issues
        .iter()
        .map(|issue| issue.code.clone())
        .collect()
}

#[test]
fn positive_fixture_acknowledges_agpl_consumption() {
    let fixture = parse_fixture(ACKNOWLEDGED_AGPL);
    let verdict = validate_license_consumption(&fixture.input);

    assert_eq!(fixture.schema, LICENSE_CONSUMPTION_FIXTURE_SCHEMA);
    assert_eq!(fixture.input.schema, LICENSE_CONSUMPTION_RECEIPT_SCHEMA);
    assert!(fixture.expected.accepted);
    assert_eq!(fixture.expected.diagnostic, "none");
    assert!(verdict.accepted, "{}", fixture.fixture_id);
    assert!(
        verdict.issues.is_empty(),
        "unexpected issues: {:?}",
        verdict.issues
    );
    assert_eq!(
        verdict.supported_claim.as_deref(),
        Some("intentional-license-review")
    );
}

#[test]
fn negative_fixtures_fail_closed_with_stable_diagnostics() {
    let cases = [
        (MISSING_REVIEW, "missing-review"),
        (WRONG_ARTIFACT, "wrong-artifact-identity"),
        (STALE_METADATA, "stale-license-metadata"),
        (WEAKENED_NON_CLAIMS, "non-claims-weakened"),
        (WRONG_LICENSE, "wrong-license-expression"),
    ];

    for (input, expected_diagnostic) in cases {
        let fixture = parse_fixture(input);
        let verdict = validate_license_consumption(&fixture.input);

        assert!(!fixture.expected.accepted, "{}", fixture.fixture_id);
        assert_eq!(fixture.expected.diagnostic, expected_diagnostic);
        assert!(!verdict.accepted, "{}", fixture.fixture_id);
        assert!(!verdict.issues.is_empty(), "{}", fixture.fixture_id);
        assert!(verdict.supported_claim.is_none(), "{}", fixture.fixture_id);
    }
}

#[test]
fn missing_review_issue_code_is_stable() {
    let fixture = parse_fixture(MISSING_REVIEW);
    let verdict = validate_license_consumption(&fixture.input);

    assert!(issue_codes(&verdict).contains(&"license-consumption.review".to_owned()));
}

#[test]
fn non_claim_boundary_is_preserved() {
    assert_eq!(
        REQUIRED_LICENSE_CONSUMPTION_NON_CLAIMS,
        [
            "legal-advice",
            "license-exception",
            "compatibility-promise",
            "alternate-license-grant",
        ]
    );
}

#[test]
fn receipt_with_deferred_review_is_rejected() {
    let fixture = parse_fixture(ACKNOWLEDGED_AGPL);
    let mut input = fixture.input;
    input.review_action = LicenseReviewAction::Deferred;

    let verdict = validate_license_consumption(&input);

    assert!(!verdict.accepted);
    assert!(issue_codes(&verdict).contains(&"license-consumption.review".to_owned()));
}

#[test]
fn unsupported_schema_is_rejected_at_the_dto_boundary() {
    let fixture = parse_fixture(ACKNOWLEDGED_AGPL);
    let mut input = fixture.input;
    input.schema = "basalt.ucan-authority-input.v1".to_owned();

    let verdict = validate_license_consumption(&input);

    assert!(!verdict.accepted);
    assert!(issue_codes(&verdict).contains(&"license-consumption.schema".to_owned()));
}
