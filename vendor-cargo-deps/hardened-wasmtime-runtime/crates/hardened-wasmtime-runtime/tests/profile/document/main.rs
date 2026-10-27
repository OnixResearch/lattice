//! Integration tests for profile::document.

use hardened_wasmtime_runtime::profile::document::*;

/// The exact generated Aspen profile document (frozen copy).
const MOLTEN_PROFILE: &str = include_str!("../../../../../fixtures/aspen/molten-profile.json");

#[test]
// r[verify onix.wasmtime_runtime.feature_policy.missing]
fn a_missing_feature_fact_fails_across_the_boundary() {
    // Remove `simd` from the document entirely.
    let missing = MOLTEN_PROFILE.replace("\"simd\": true,", "");
    assert!(matches!(admit(&missing), Err(Error::MalformedDocument(_))));
}

#[test]
fn unknown_fields_or_schema_fail_closed() {
    let drifted = MOLTEN_PROFILE.replace(
        "\"schema_id\": \"molten.wasm-component-profile.v1\"",
        "\"schema_id\": \"molten.wasm-component-profile.v0\"",
    );
    assert_eq!(
        admit(&drifted),
        Err(Error::UnknownSchema(String::from(
            "molten.wasm-component-profile.v0"
        )))
    );
    let extra = MOLTEN_PROFILE.replace(
        "\"bulk_memory\": true,",
        "\"bulk_memory\": true, \"surprise_feature\": true,",
    );
    assert!(matches!(admit(&extra), Err(Error::MalformedDocument(_))));
}

#[test]
fn ambient_imports_are_denied() {
    let ambient = MOLTEN_PROFILE.replace(
        "\"allowed_imports\": []",
        "\"allowed_imports\": [\"wasi:filesystem/types@0.2.6\"]",
    );
    assert_eq!(admit(&ambient), Err(Error::AmbientImportsAdmitted));
}

#[test]
fn toolchain_pin_drift_fails_admission() {
    let drifted = MOLTEN_PROFILE.replace("\"wasmtime\": \"45.0.0\"", "\"wasmtime\": \"44.0.0\"");
    assert_eq!(
        admit(&drifted),
        Err(Error::UnknownToolchainPin(String::from(
            "wasmtime=44.0.0, expected 45.0.0"
        )))
    );
}

#[test]
fn nondeterministic_growth_fails_admission() {
    let grown = MOLTEN_PROFILE.replace(
        "\"memory_growth\": \"fixed\"",
        "\"memory_growth\": \"unbounded\"",
    );
    assert_eq!(
        admit(&grown),
        Err(Error::UnknownGrowthPolicy(String::from("unbounded")))
    );
}
