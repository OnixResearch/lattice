//! Lattice-owned profile over the producer's explicit feature and limit contract.
use hardened_wasmtime_runtime::feature::policy::Determinism;
use hardened_wasmtime_runtime::feature::policy::WasmtimeFeatures;
use hardened_wasmtime_runtime::feature::policy::{self};
use hardened_wasmtime_runtime::resource::admission::RuntimeBounds;
use hardened_wasmtime_runtime::resource::admission::WASM_PAGE_BYTES;
use hardened_wasmtime_runtime::resource::admission::{self};

use crate::operation_pack::Error;

const PROFILE_DOMAIN: &str = "lattice.operation-pack-wasm-runtime-profile.v1";
const IMPORT_LIMIT: u32 = 1;
const EXPORT_LIMIT: u32 = 2; // named interface plus its single function

pub(crate) struct Profile {
    pub policy: policy::Admitted,
    pub bounds: RuntimeBounds,
    pub identity: String,
}

/// Compute the exact profile a materialized bundle must bind before execution.
/// This is Lattice policy, not a generic runtime policy or a source-trust claim.
pub fn runtime_profile_identity(descriptor: &super::Descriptor) -> Result<String, Error> {
    Ok(admit(descriptor)?.identity)
}

pub(crate) fn admit(descriptor: &super::Descriptor) -> Result<Profile, Error> {
    let policy = policy::admit(
        WasmtimeFeatures {
            bulk_memory: true,
            component_model: true,
            multi_value: true,
            reference_types: true,
            simd: false,
            tail_call: false,
            wide_arithmetic: false,
            memory64: false,
            multi_memory: false,
            threads: false,
            gc: false,
            exceptions: false,
            relaxed_simd: false,
            custom_page_sizes: false,
            extended_const: false,
            function_references: false,
            component_async: false,
        },
        Determinism {
            fuel_interruption: true,
            nan_canonicalization: true,
            relaxed_simd_deterministic: true,
            memory_growth_fixed: true,
            table_growth_fixed: true,
        },
    )
    .map_err(|_| Error::Invalid("component feature policy"))?;
    let resources = &descriptor.resources;
    if resources.memory_bytes % WASM_PAGE_BYTES != 0 || resources.deadline_millis == 0 {
        return Err(Error::Invalid("component memory or deadline bound"));
    }
    let memory_pages = u32::try_from(resources.memory_bytes / WASM_PAGE_BYTES)
        .map_err(|_| Error::Invalid("component memory bound"))?;
    let bounds = admission::admit_limits(RuntimeBounds {
        fuel: resources.fuel,
        max_stack_bytes: resources.stack_bytes,
        max_memory_pages: memory_pages,
        max_total_memory_pages: memory_pages,
        max_tables: resources.tables,
        max_table_elements: u32::try_from(resources.table_elements)
            .map_err(|_| Error::Invalid("component table bound"))?,
        max_instances: resources.instances,
        max_memories: resources.memories,
        max_input_bytes: resources.maximum_input_bytes,
        max_output_bytes: resources.maximum_output_bytes,
        max_component_bytes: resources.maximum_component_bytes,
        max_wit_bytes: resources.maximum_wit_bytes,
        max_imports: IMPORT_LIMIT,
        max_exports: EXPORT_LIMIT,
        max_concurrency: resources.maximum_concurrency,
    })
    .map_err(|_| Error::Invalid("component resource bounds"))?;
    let identity = crate::operation::identity::hash_value(
        PROFILE_DOMAIN,
        &(
            super::RUNTIME_REVISION,
            policy.identity,
            IMPORT_LIMIT,
            EXPORT_LIMIT,
            super::FIRST_WORLD_WIT,
            &descriptor.export_interface,
            &descriptor.invoke,
            resources,
        ),
    )
    .map_err(|_| Error::Encoding)?;
    Ok(Profile {
        policy,
        bounds,
        identity,
    })
}
