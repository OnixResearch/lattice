//! Generated Steel skill-module policy parity tests.

use serde::Deserialize;

const GENERATED_POLICY: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../config/generated/steel-skill-module-policy.json"
));
const POLICY_SCHEMA: &str = "animus.steel-skill-module-policy.v1";
const SOURCE_CHARS_MAX: u32 = 1_048_576;
const EVALUATION_FUEL_MAX: u64 = 16_777_216;
const MODULES_MAX: u32 = 64;
const EXPORTS_MAX: u32 = 64;
const DEPENDENCIES_MAX: u32 = 64;
const CAPABILITY_REQUESTS_MAX: u32 = 32;

#[allow(
    clippy::struct_excessive_bools,
    reason = "closed policy parity fixture"
)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    schema: String,
    language_cohort: String,
    trust_states: Vec<String>,
    source_forms: Vec<String>,
    denied_source_forms: Vec<String>,
    automatic_activation: bool,
    ambient_discovery: bool,
    native_modules: bool,
    dynamic_libraries: bool,
    package_manager: bool,
    capability_intersection: Vec<String>,
    bounds: Bounds,
    non_claims: Vec<String>,
}

#[allow(clippy::struct_field_names, reason = "JSON field names are policy API")]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bounds {
    source_chars_max: u32,
    evaluation_fuel_max: u64,
    modules_max: u32,
    exports_per_module_max: u32,
    dependencies_per_module_max: u32,
    capability_requests_per_module_max: u32,
}

#[test]
fn generated_skill_module_policy_is_closed_and_matches_rust_bounds() {
    // r[verify agent_harness_runtime.steel_skill_modules.admission]
    let policy: Policy = serde_json::from_str(GENERATED_POLICY).expect("generated policy");
    assert_eq!(policy.schema, POLICY_SCHEMA);
    assert_eq!(
        policy.language_cohort,
        animus_adapter_rlm_kernel::STEEL_SKILL_MODULE_COHORT
    );
    assert_eq!(policy.trust_states, ["trusted"]);
    assert_eq!(policy.source_forms, ["complete-steel-source-unit"]);
    assert!(
        policy
            .denied_source_forms
            .contains(&String::from("native-module"))
    );
    assert!(
        policy
            .denied_source_forms
            .contains(&String::from("arbitrary-path"))
    );
    assert!(!policy.automatic_activation);
    assert!(!policy.ambient_discovery);
    assert!(!policy.native_modules);
    assert!(!policy.dynamic_libraries);
    assert!(!policy.package_manager);
    assert_eq!(
        policy.capability_intersection,
        ["session", "model-profile", "skill", "current-host"]
    );
    assert_eq!(policy.bounds.source_chars_max, SOURCE_CHARS_MAX);
    assert_eq!(policy.bounds.evaluation_fuel_max, EVALUATION_FUEL_MAX);
    assert_eq!(policy.bounds.modules_max, MODULES_MAX);
    assert_eq!(policy.bounds.exports_per_module_max, EXPORTS_MAX);
    assert_eq!(policy.bounds.dependencies_per_module_max, DEPENDENCIES_MAX);
    assert_eq!(
        policy.bounds.capability_requests_per_module_max,
        CAPABILITY_REQUESTS_MAX
    );
    assert!(
        policy
            .non_claims
            .contains(&String::from("skill-correctness"))
    );
    assert!(
        policy
            .non_claims
            .contains(&String::from("production-readiness"))
    );
}

#[test]
fn generated_skill_module_policy_rejects_unknown_fields() {
    let malformed = GENERATED_POLICY.replacen("{\n", "{\n  \"unknown_authority\": true,\n", 1);
    assert!(serde_json::from_str::<Policy>(malformed.as_str()).is_err());
}
