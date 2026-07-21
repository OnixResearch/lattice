const NODE_TEMPLATE_SPEC: &str = include_str!("../cairn/specs/node-templates/spec.md");
const REPOSITORY_VALIDATION_SPEC: &str = include_str!("../cairn/specs/repository-validation/spec.md");
const HEADLESS_SOURCE: &str = include_str!("../src/headless.rs");
const TEMPLATE_PARAMETER_TESTS: &str = include_str!("test_template_parameters.rs");
const TRIGGER_TESTS: &str = include_str!("test_triggers.rs");
const THIS_SOURCE: &str = include_str!("test_traceability_marker_alignment.rs");

const BOUNDS_REQUIREMENT: &str = "n8nclone.node_templates.parameters.invalid.bounds";
const RECONCILIATION_REQUIREMENT: &str =
    "lattice.repository_validation.traceability_sources.scenario.marker-reconciliation";
const RETIRED_MARKERS: &[&str] = &[
    "n8nclone.credentials.webhook_auth.provider",
    "n8nclone.credentials.webhook_auth.redacted",
    "n8nclone.scheduling.triggers.admission.enabled",
    "n8nclone.scheduling.triggers.model.persist",
    "n8nclone.scheduling.triggers.webhook.content_type",
    "n8nclone.scheduling.triggers.webhook.headless_run",
    "n8nclone.scheduling.triggers.webhook.method",
];

// r[verify lattice.repository_validation.traceability_sources.scenario.marker-reconciliation]
#[test]
fn accepted_marker_ids_have_matching_evidence() {
    assert!(NODE_TEMPLATE_SPEC.contains(&format!("r[{BOUNDS_REQUIREMENT}]")));
    assert!(TEMPLATE_PARAMETER_TESTS.contains(&format!("r[verify {BOUNDS_REQUIREMENT}]")));
    assert!(REPOSITORY_VALIDATION_SPEC.contains(&format!("r[{RECONCILIATION_REQUIREMENT}]")));
    assert!(THIS_SOURCE.contains(&format!("r[verify {RECONCILIATION_REQUIREMENT}]")));
}

#[test]
fn retired_dangling_marker_ids_are_absent() {
    for marker in RETIRED_MARKERS {
        assert!(!TRIGGER_TESTS.contains(marker), "retired test marker remained: {marker}");
        assert!(!HEADLESS_SOURCE.contains(marker), "retired source marker remained: {marker}");
    }
}
