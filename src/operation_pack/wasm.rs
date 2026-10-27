//! Pure import-free component descriptor and admission vocabulary.
//!
//! r[impl lattice.operation_packs_wasm.manifest]
//! r[impl lattice.operation_packs_wasm.functional_core]
//! r[impl lattice.operation_packs_wasm.world]
//! r[impl lattice.operation_packs_wasm.nonclaims]

use serde::Deserialize;
use serde::Serialize;

mod admission;
pub(crate) mod profile;
mod validation;

pub use admission::admit_dispatch;
pub use profile::runtime_profile_identity;
pub(crate) use validation::normalize;
pub(crate) use validation::validate_descriptor;
pub(crate) use validation::validate_method_closure;

pub const RUNTIME_KIND: &str = "wasm-component";
pub const WIT_PACKAGE: &str = "lattice:operation-pack@1.0.0";
pub const WORLD: &str = "operation";
pub const EXPORT_INTERFACE: &str = "lattice:operation-pack/methods@1.0.0";
pub const INVOKE: &str = "invoke";
pub const RUNTIME_REVISION: &str = "1d642fa5cba7ce8a6d8f127f28ec6d8505842378";
pub const VERIFIER_REVISION: &str = "dcf4b51b3210f25f08e96d4e7bc1653d2fbcd240";
pub const FIRST_WORLD_WIT: &str = "package lattice:operation-pack@1.0.0;\n\ninterface methods {\n  invoke: func(method: string, input: list<u8>) -> list<u8>;\n}\n\nworld operation {\n  export methods;\n}\n";
pub const RESULT_SCHEMA: &str = "lattice.operation-pack-wasm-result.v1";
pub const EFFECT_INTENT_SCHEMA: &str = "lattice.operation-pack-wasm-effect-intents.v1";
pub const MAXIMUM_EFFECT_INTENTS: usize = 64;
pub const REQUIRED_NON_CLAIMS: [&str; 7] = [
    "component evidence does not prove pack trust",
    "component evidence does not prove component correctness",
    "component evidence does not prove operation correctness",
    "component evidence does not prove sandbox completeness",
    "component success does not prove external effect success",
    "component evidence does not prove production readiness",
    "component evidence does not prove release eligibility",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Descriptor {
    pub wit_content_id: String,
    pub bundle_content_id: String,
    pub wit_package: String,
    pub world: String,
    pub export_interface: String,
    pub invoke: String,
    pub methods: Vec<MethodMapping>,
    pub verifier_revision: String,
    pub evidence_identities: std::collections::BTreeMap<super::model::EvidenceKind, String>,
    pub runtime_revision: String,
    pub runtime_profile_identity: String,
    pub resources: Resources,
    pub result_schema: String,
    pub effect_intent_schema: String,
    pub effect_intent_targets: Vec<EffectTarget>,
    pub maximum_effect_intents: usize,
    pub required_non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MethodMapping {
    pub type_id: String,
    pub method: String,
    pub guest_method: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectTarget {
    pub type_id: String,
    pub method: String,
}

/// All values are hard per-dispatch limits, including cumulative guest totals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub fuel: u64,
    pub memory_bytes: u64,
    pub stack_bytes: u64,
    pub table_elements: u64,
    pub instances: u32,
    pub tables: u32,
    pub memories: u32,
    pub maximum_input_bytes: u64,
    pub maximum_output_bytes: u64,
    pub maximum_component_bytes: u64,
    pub maximum_wit_bytes: u64,
    pub maximum_concurrency: u32,
    pub deadline_millis: u64,
}

/// Supplied by the materialized bundle verifier, never by guest output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleFacts {
    pub bundle_identity: String,
    pub component_identity: String,
    pub wit_identity: String,
    pub verifier_revision: String,
    pub valid: bool,
    pub current: bool,
    pub overclaimed: bool,
}

/// Supplied by actual component analysis, not an echo of manifest declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldFacts {
    pub wit_package: String,
    pub world: String,
    pub export_interface: String,
    pub invoke: String,
    pub imports: Vec<String>,
}

/// Host operation admission remains authoritative; a component cannot grant it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityFacts {
    pub pack_identity: String,
    pub lock_identity: String,
    pub type_id: String,
    pub type_revision: String,
    pub method: String,
    pub capabilities: Vec<crate::operation::CapabilityClass>,
    pub operation_admitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchFacts {
    pub platform: String,
    pub runtime_available: bool,
    pub runtime_revision: String,
    pub runtime_profile_identity: String,
    pub bundle: BundleFacts,
    pub world: WorldFacts,
    pub resources: Resources,
    pub input_bytes: u64,
    pub authority: AuthorityFacts,
}

/// Declaration order is diagnostic priority, independent of fact collection order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Blocker {
    Candidate,
    Lock,
    RuntimeDeclaration,
    RuntimeUnavailable,
    Platform,
    Bundle,
    VerifierRevision,
    RuntimeRevision,
    RuntimeProfile,
    World,
    Imports,
    Method,
    Resources,
    Authority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DispatchPlan {
    pub runtime_id: String,
    pub type_id: String,
    pub type_revision: String,
    pub method: String,
    pub guest_method: String,
    pub component_identity: String,
    pub wit_identity: String,
    pub bundle_identity: String,
    pub cohort_identity: String,
    pub descriptor: Descriptor,
}
