//! Cumulative runtime resource-limit admission.
//!
//! Every limit is independent and positive; reaching any one limit during
//! execution produces its own normalized outcome class. Limits are derived
//! from an admitted profile document so the shell can enforce exactly what
//! the profile admitted.
// r[impl onix.wasmtime_runtime.resource_limits]
// r[impl onix.wasmtime_runtime.independent_memory_limits]

/// Bytes in one WebAssembly page.
pub const WASM_PAGE_BYTES: u64 = 65_536;

/// The named cumulative limits for one admitted runtime profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeBounds {
    /// Metered fuel budget for the whole execution.
    pub fuel: u64,
    /// Maximum guest stack size in bytes.
    pub max_stack_bytes: u64,
    /// Maximum size of each guest linear memory in 64 KiB pages.
    pub max_memory_pages: u32,
    /// Maximum cumulative invocation memory allocations in 64 KiB pages.
    pub max_total_memory_pages: u32,
    /// Maximum number of tables.
    pub max_tables: u32,
    /// Maximum elements per table.
    pub max_table_elements: u32,
    /// Maximum number of live instances.
    pub max_instances: u32,
    /// Maximum number of memories.
    pub max_memories: u32,
    /// Maximum bytes the host accepts as input for one invocation.
    pub max_input_bytes: u64,
    /// Maximum bytes the host accepts as output from one invocation.
    pub max_output_bytes: u64,
    /// Maximum artifact size in bytes.
    pub max_component_bytes: u64,
    /// Maximum WIT document size in bytes.
    pub max_wit_bytes: u64,
    /// Maximum number of declared imports.
    pub max_imports: u32,
    /// Maximum number of declared exports.
    pub max_exports: u32,
    /// Maximum concurrent invocations.
    pub max_concurrency: u32,
}

/// Resource-limit admission failure. Names the violated limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundsError {
    /// A limit that must be positive was zero; names the limit.
    ZeroLimit(&'static str),
    /// Memory bytes were not a whole number of WebAssembly pages.
    MemoryNotPageAligned,
}

/// Admit positive runtime bounds. Memory is expressed in whole Wasm pages.
pub fn admit_limits(limits: RuntimeBounds) -> Result<RuntimeBounds, BoundsError> {
    for (name, value) in named_limits(limits) {
        if value == 0 {
            return Err(BoundsError::ZeroLimit(name));
        }
    }
    Ok(limits)
}

fn named_limits(limits: RuntimeBounds) -> [(&'static str, u64); 15] {
    [
        ("fuel", limits.fuel),
        ("max_stack_bytes", limits.max_stack_bytes),
        ("max_memory_pages", u64::from(limits.max_memory_pages)),
        (
            "max_total_memory_pages",
            u64::from(limits.max_total_memory_pages),
        ),
        ("max_tables", u64::from(limits.max_tables)),
        ("max_table_elements", u64::from(limits.max_table_elements)),
        ("max_instances", u64::from(limits.max_instances)),
        ("max_memories", u64::from(limits.max_memories)),
        ("max_input_bytes", limits.max_input_bytes),
        ("max_output_bytes", limits.max_output_bytes),
        ("max_component_bytes", limits.max_component_bytes),
        ("max_wit_bytes", limits.max_wit_bytes),
        ("max_imports", u64::from(limits.max_imports)),
        ("max_exports", u64::from(limits.max_exports)),
        ("max_concurrency", u64::from(limits.max_concurrency)),
    ]
}

/// Outcome class produced when a guest exceeds its admitted input bound.
pub const INPUT_LIMIT_CLASS: &str = "input-limit-exceeded";
/// Outcome class produced when a guest exceeds its admitted output bound.
pub const OUTPUT_LIMIT_CLASS: &str = "output-limit-exceeded";
/// Outcome class produced when a guest exceeds the memory bound.
pub const MEMORY_LIMIT_CLASS: &str = "memory-limit-exceeded";
/// Outcome class produced when the epoch deadline passes.
pub const DEADLINE_CLASS: &str = "deadline-exhausted";
