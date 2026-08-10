use std::collections::BTreeSet;

use serde_json::Value;

const GENERATED_POLICY: &str = include_str!("../cairn-policy/generated/cairn-policy.json");
const INVALID_SOURCE_ROOT_FIXTURE: &str = include_str!("../cairn-policy/fixtures/invalid-traceability-source-root.ncl");
const DEFAULT_PROFILE_ID: &str = "cairn-default";
const UNSAFE_SOURCE_ROOT: &str = "../src";
const LEGACY_CRATE_ROOT: &str = "crates";
const LEGACY_TOOL_ROOT: &str = "tools";
const REQUIRED_MARKER_VERBS: [&str; 4] = ["depends", "impl", "related", "verify"];
const REQUIRED_SOURCES: [(&str, &str); 7] = [
    ("cairn-policy", "nickel_source"),
    ("examples", "nickel_source"),
    ("examples", "rust_source"),
    ("scripts", "tool_source"),
    ("src", "rust_source"),
    ("tests", "rust_source"),
    (".cairn/specs", "accepted_specs"),
];

fn policy() -> Value {
    serde_json::from_str(GENERATED_POLICY).expect("generated Cairn policy must be valid JSON")
}

fn default_profile(policy: &Value) -> &Value {
    let traceability = &policy["traceability_policy"];
    assert_eq!(traceability["default_profile"], DEFAULT_PROFILE_ID);
    traceability["profiles"]
        .as_array()
        .expect("traceability profiles")
        .iter()
        .find(|profile| profile["id"] == DEFAULT_PROFILE_ID)
        .expect("default traceability profile")
}

fn source_pairs(profile: &Value) -> BTreeSet<(&str, &str)> {
    let mut sources = BTreeSet::new();
    for field in ["requirement_sources", "evidence_sources"] {
        for source in profile[field].as_array().expect("traceability sources") {
            sources.insert((
                source["root"].as_str().expect("source root"),
                source["source_kind"].as_str().expect("source kind"),
            ));
        }
    }
    sources
}

// r[verify lattice.repository_validation.traceability_sources.scenario.discovery]
#[test]
fn default_traceability_profile_scans_lattice_owned_sources() {
    let policy = policy();
    let actual = source_pairs(default_profile(&policy));
    let expected = BTreeSet::from(REQUIRED_SOURCES);

    assert_eq!(actual, expected);
}

// r[verify lattice.repository_validation.traceability_sources.scenario.fail-closed]
// r[verify lattice.repository_validation.traceability_sources.scenario.unsafe-root]
#[test]
fn traceability_profile_rejects_fallback_and_unsafe_roots_without_relaxing_verbs() {
    let policy = policy();
    let profile = default_profile(&policy);
    let sources = source_pairs(profile);
    let actual_verbs = profile["marker_verbs"]
        .as_array()
        .expect("marker verbs")
        .iter()
        .map(|verb| verb.as_str().expect("marker verb"))
        .collect::<BTreeSet<_>>();
    let required_verbs = BTreeSet::from(REQUIRED_MARKER_VERBS);

    assert!(!sources.iter().any(|(root, _)| *root == LEGACY_CRATE_ROOT));
    assert!(!sources.iter().any(|(root, _)| *root == LEGACY_TOOL_ROOT));
    assert_eq!(actual_verbs, required_verbs);
    assert!(INVALID_SOURCE_ROOT_FIXTURE.contains(UNSAFE_SOURCE_ROOT));
}
