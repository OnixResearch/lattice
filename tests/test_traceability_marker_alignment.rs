const NODE_TEMPLATE_SPEC: &str = include_str!("../cairn/specs/node-templates/spec.md");
const REPOSITORY_VALIDATION_SPEC: &str = include_str!("../cairn/specs/repository-validation/spec.md");
const HEADLESS_SOURCE: &str = include_str!("../src/headless.rs");
const TEMPLATE_PARAMETER_TESTS: &str = include_str!("test_template_parameters.rs");
const TRIGGER_TESTS: &str = include_str!("test_triggers.rs");
const THIS_SOURCE: &str = include_str!("test_traceability_marker_alignment.rs");

const BOUNDS_REQUIREMENT: &str = concat!("n8n", "clone.node_templates.parameters.invalid.bounds");
const RECONCILIATION_REQUIREMENT: &str =
    "lattice.repository_validation.traceability_sources.scenario.marker-reconciliation";
const RETIRED_MARKERS: &[&str] = &[
    concat!("n8n", "clone.credentials.webhook_auth.provider"),
    concat!("n8n", "clone.credentials.webhook_auth.redacted"),
    concat!("n8n", "clone.scheduling.triggers.admission.enabled"),
    concat!("n8n", "clone.scheduling.triggers.model.persist"),
    concat!("n8n", "clone.scheduling.triggers.webhook.content_type"),
    concat!("n8n", "clone.scheduling.triggers.webhook.headless_run"),
    concat!("n8n", "clone.scheduling.triggers.webhook.method"),
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
