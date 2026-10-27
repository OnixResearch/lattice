//! Deserialized shape of the checked Molten profile document.
//!
//! Field names mirror the document keys one to one; the field docs are
//! the JSON keys themselves.

#![allow(missing_docs)]

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub profile: Body,
    pub schema_id: String,
    pub schema_version: u32,
    pub source_language: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Body {
    pub allowed_imports: Vec<String>,
    pub allowed_wasi_interfaces: Vec<String>,
    pub determinism: DeterminismFacts,
    pub evidence_scope: String,
    pub features: FeatureFacts,
    pub non_claims: Vec<String>,
    pub profile_id: String,
    pub resources: Resources,
    pub runtime_strategy: String,
    pub toolchain: ToolchainFacts,
    pub wit: WitFacts,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureFacts {
    pub bulk_memory: bool,
    pub component_model: bool,
    pub multi_value: bool,
    pub reference_types: bool,
    pub simd: bool,
    pub tail_call: bool,
    pub wide_arithmetic: bool,
    pub memory64: bool,
    pub multi_memory: bool,
    pub threads: bool,
    pub gc: bool,
    pub exceptions: bool,
    pub relaxed_simd: bool,
    pub custom_page_sizes: bool,
    pub extended_const: bool,
    pub function_references: bool,
    pub component_async: bool,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeterminismFacts {
    pub fuel_interruption: bool,
    pub host_inputs: String,
    pub memory_growth: String,
    pub nan_canonicalization: bool,
    pub relaxed_simd_deterministic: bool,
    pub table_growth: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub fuel: u64,
    pub max_component_bytes: u64,
    pub max_concurrency: u32,
    pub max_exports: u32,
    pub max_hostcall_bytes: u64,
    pub max_imports: u32,
    pub max_instances: u32,
    pub max_memories: u32,
    pub max_memory_bytes: u64,
    pub max_result_bytes: u64,
    pub max_stack_bytes: u64,
    pub max_table_elements: u32,
    pub max_tables: u32,
    pub max_wit_bytes: u64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainFacts {
    pub wasi_package: String,
    pub wasm_tools: String,
    pub wasmparser: String,
    pub wasmtime: String,
    pub wasmtime_wasi: String,
    pub wit_bindgen: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WitFacts {
    pub package: String,
    pub source_ref: String,
    pub world: String,
}
