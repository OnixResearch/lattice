#[path = "traceability_promotion/core.rs"]
mod core;

use core::CoverageReceipt;
use core::marker_is_attached_to_test;
use core::validate_promotion;
use std::collections::BTreeSet;

const BASELINE_COVERAGE: &str = include_str!("fixtures/traceability-core-credentials-current.json");
const CURRENT_COVERAGE: &str = include_str!("fixtures/traceability-core-credentials-promotion-current.json");
const CREDENTIAL_TEST_SOURCE: &str = include_str!("test_credentials.rs");
const INVENTORY_POLICY: &str = include_str!("../cairn-policy/traceability-inventory.ncl");
const BASELINE_MISSING_COUNT: usize = 353;
const CURRENT_MISSING_COUNT: usize = 341;
const PROMOTED_COUNT: usize = 12;
const UNRELATED_MISSING_ID: &str = "lattice.agent_orchestration.authority.scenario.ambient-denied";
const UNEXPECTED_ADDED_ID: &str = "lattice.example.unaccepted.scenario.added";
const DANGLING_ID: &str = "lattice.example.dangling.scenario.marker";
const CACHE_REFERENCE_ID: &str = concat!("n8n", "clone.credentials.cache_identity.reference");
const PROVIDER_MISSING_ID: &str = concat!("n8n", "clone.credentials.provider_interface.missing");
const PROVIDER_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.provider_interface.resolve");
const REGISTRY_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.provider_registry.resolve");
const REGISTRY_UNKNOWN_ID: &str = concat!("n8n", "clone.credentials.provider_registry.unknown");
const ENV_MISSING_ID: &str = concat!("n8n", "clone.credentials.providers.env.missing");
const ENV_RESOLVE_ID: &str = concat!("n8n", "clone.credentials.providers.env.resolve");
const REDACTION_OUTPUT_ID: &str = concat!("n8n", "clone.credentials.redaction.output");
const REDACTION_VALIDATION_ID: &str = concat!("n8n", "clone.credentials.redaction.validation");
const REJECT_PLAINTEXT_ID: &str = concat!("n8n", "clone.credentials.secret_refs.model.reject_plaintext");
const STORE_REFERENCE_ID: &str = concat!("n8n", "clone.credentials.secret_refs.model.store_ref");
const VALIDATION_MALFORMED_ID: &str = concat!("n8n", "clone.credentials.validation.malformed");
const PROMOTED_IDS: &[&str] = &[
    CACHE_REFERENCE_ID,
    PROVIDER_MISSING_ID,
    PROVIDER_RESOLVE_ID,
    REGISTRY_RESOLVE_ID,
    REGISTRY_UNKNOWN_ID,
    ENV_MISSING_ID,
    ENV_RESOLVE_ID,
    REDACTION_OUTPUT_ID,
    REDACTION_VALIDATION_ID,
    REJECT_PLAINTEXT_ID,
    STORE_REFERENCE_ID,
    VALIDATION_MALFORMED_ID,
];

struct Attachment {
    test_name: &'static str,
    requirement_id: &'static str,
}

const REQUIRED_ATTACHMENTS: &[Attachment] = &[
    Attachment {
        test_name: "cache_identity_uses_secret_reference_not_secret_value",
        requirement_id: CACHE_REFERENCE_ID,
    },
    Attachment {
        test_name: "missing_secret_fails_before_shell_execution",
        requirement_id: PROVIDER_MISSING_ID,
    },
    Attachment {
        test_name: "credential_provider_redacts_output_logs_history_json_and_persistence",
        requirement_id: PROVIDER_RESOLVE_ID,
    },
    Attachment {
        test_name: "registry_resolves_registered_provider_and_validation_rejects_unknown_provider",
        requirement_id: REGISTRY_RESOLVE_ID,
    },
    Attachment {
        test_name: "registry_resolves_registered_provider_and_validation_rejects_unknown_provider",
        requirement_id: REGISTRY_UNKNOWN_ID,
    },
    Attachment {
        test_name: "environment_provider_resolves_present_secret_and_rejects_missing_or_empty_secret",
        requirement_id: ENV_MISSING_ID,
    },
    Attachment {
        test_name: "environment_provider_resolves_present_secret_and_rejects_missing_or_empty_secret",
        requirement_id: ENV_RESOLVE_ID,
    },
    Attachment {
        test_name: "credential_provider_redacts_output_logs_history_json_and_persistence",
        requirement_id: REDACTION_OUTPUT_ID,
    },
    Attachment {
        test_name: "validation_text_and_json_preserve_reference_identity_without_plaintext_value",
        requirement_id: REDACTION_VALIDATION_ID,
    },
    Attachment {
        test_name: "nickel_import_rejects_all_non_null_plaintext_credential_shapes_with_safe_error",
        requirement_id: REJECT_PLAINTEXT_ID,
    },
    Attachment {
        test_name: "cli_plaintext_credential_rejection_is_safe_and_precedes_store_creation",
        requirement_id: REJECT_PLAINTEXT_ID,
    },
    Attachment {
        test_name: "resolved_secret_is_not_saved_or_exported_with_workflow",
        requirement_id: STORE_REFERENCE_ID,
    },
    Attachment {
        test_name: "malformed_secret_reference_is_a_fatal_validation_diagnostic",
        requirement_id: VALIDATION_MALFORMED_ID,
    },
];

fn decode_receipt(source: &str) -> CoverageReceipt {
    serde_json::from_str(source).expect("coverage fixture must decode")
}

fn issue_codes(issues: &[core::PromotionIssue]) -> BTreeSet<&str> {
    issues.iter().map(|issue| issue.code).collect()
}

// r[verify lattice.repository_validation.core_credential_promotion]
// r[verify lattice.repository_validation.core_credential_promotion.scenario.exact_delta]
#[test]
fn exact_twelve_id_promotion_preserves_every_unrelated_gap() {
    let baseline = decode_receipt(BASELINE_COVERAGE);
    let current = decode_receipt(CURRENT_COVERAGE);
    let issues = validate_promotion(&baseline, &current, PROMOTED_IDS);

    assert!(issues.is_empty(), "issues: {issues:?}");
    assert_eq!(baseline.missing.len(), BASELINE_MISSING_COUNT);
    assert_eq!(current.missing.len(), CURRENT_MISSING_COUNT);
    assert_eq!(PROMOTED_IDS.len(), PROMOTED_COUNT);
    assert!(baseline.dangling.is_empty());
    assert!(current.dangling.is_empty());
    assert!(!baseline.valid);
    assert!(!current.valid);
}

// r[verify lattice.repository_validation.core_credential_promotion.scenario.local_binding]
#[test]
fn promoted_markers_are_attached_to_exact_tests_and_leave_the_debt_policy() {
    for attachment in REQUIRED_ATTACHMENTS {
        assert!(
            marker_is_attached_to_test(CREDENTIAL_TEST_SOURCE, attachment.test_name, attachment.requirement_id,),
            "marker {} was not attached to {}",
            attachment.requirement_id,
            attachment.test_name
        );
    }
    for promoted in PROMOTED_IDS {
        assert!(!INVENTORY_POLICY.contains(promoted), "promoted ID remained in debt policy: {promoted}");
    }
}

// r[verify lattice.repository_validation.core_credential_promotion.scenario.fail_closed]
#[test]
fn duplicate_unexpected_dangling_and_successful_transitions_fail_closed() {
    let mut baseline = decode_receipt(BASELINE_COVERAGE);
    baseline.missing.push(baseline.missing[0].clone());
    let mut current = decode_receipt(CURRENT_COVERAGE);
    current.missing.retain(|identifier| identifier != UNRELATED_MISSING_ID);
    current.missing.push(PROMOTED_IDS[0].to_string());
    current.missing.push(PROMOTED_IDS[0].to_string());
    current.missing.push(UNEXPECTED_ADDED_ID.to_string());
    current.dangling.push(DANGLING_ID.to_string());
    current.valid = true;
    let mut allowlist = PROMOTED_IDS.to_vec();
    allowlist.push(PROMOTED_IDS[0]);

    let issues = validate_promotion(&baseline, &current, &allowlist);
    let codes = issue_codes(&issues);

    assert!(codes.contains("baseline.duplicate_missing"));
    assert!(codes.contains("baseline.unsorted"));
    assert!(codes.contains("current.duplicate_missing"));
    assert!(codes.contains("current.unsorted"));
    assert!(codes.contains("current.dangling"));
    assert!(codes.contains("current.expected_invalid"));
    assert!(codes.contains("allowlist.duplicate"));
    assert!(codes.contains("transition.expected_removal_missing"));
    assert!(codes.contains("transition.unexpected_removed"));
    assert!(codes.contains("transition.unexpected_added"));
}
