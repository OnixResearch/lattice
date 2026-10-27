//! Explicit Wasmtime feature-policy admission.
//!
//! Every reviewed Wasmtime proposal and deterministic code-generation
//! option is a required field. There are no defaults: a missing fact is a
//! type error in Rust, an omitted JSON fact is an admission error, and a
//! denied feature that is enabled fails admission before any engine
//! exists.
// r[impl onix.wasmtime_runtime.feature_policy]

/// Stable identity context for admitted feature policies.
pub const IDENTITY_CONTEXT: &[u8] = b"hardened_wasmtime.feature_policy.v1\0";

/// Every reviewed Wasmtime proposal and code-generation option, with an
/// explicit reviewed state. Omission is impossible in Rust; the JSON
/// admission path enforces the same exhaustiveness across the boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WasmtimeFeatures {
    /// Bulk-memory operations proposal state.
    pub bulk_memory: bool,
    /// Component-model proposal state.
    pub component_model: bool,
    /// Multi-value proposal state.
    pub multi_value: bool,
    /// Reference-types proposal state.
    pub reference_types: bool,
    /// Fixed-width SIMD proposal state.
    pub simd: bool,
    /// Tail-call proposal state.
    pub tail_call: bool,
    /// Wide-arithmetic proposal state.
    pub wide_arithmetic: bool,
    /// 64-bit memory proposal state.
    pub memory64: bool,
    /// Multi-memory proposal state.
    pub multi_memory: bool,
    /// Threads proposal state.
    pub threads: bool,
    /// GC proposal state.
    pub gc: bool,
    /// Exception-handling proposal state.
    pub exceptions: bool,
    /// Relaxed-SIMD proposal state.
    pub relaxed_simd: bool,
    /// Custom page sizes proposal state.
    pub custom_page_sizes: bool,
    /// Extended-const proposal state.
    pub extended_const: bool,
    /// Function-references proposal state.
    pub function_references: bool,
    /// Component-model async state.
    pub component_async: bool,
}

/// Features the hardened profile denies outright. Enabling any of these
/// fails admission regardless of the profile's other facts.
pub const DENIED_FEATURES: [&str; 12] = [
    "tail_call",
    "wide_arithmetic",
    "memory64",
    "multi_memory",
    "threads",
    "gc",
    "exceptions",
    "relaxed_simd",
    "custom_page_sizes",
    "extended_const",
    "function_references",
    "component_async",
];

/// Deterministic code-generation and execution determinism controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Determinism {
    /// Metered fuel interruption is enabled.
    pub fuel_interruption: bool,
    /// NaN values are canonicalized for determinism.
    pub nan_canonicalization: bool,
    /// Relaxed SIMD uses its deterministic form.
    pub relaxed_simd_deterministic: bool,
    /// Memory growth beyond admitted limits is fixed off.
    pub memory_growth_fixed: bool,
    /// Table growth beyond admitted limits is fixed off.
    pub table_growth_fixed: bool,
}

/// An admitted feature policy with its deterministic identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admitted {
    /// The admitted feature facts.
    pub features: WasmtimeFeatures,
    /// The admitted determinism facts.
    pub determinism: Determinism,
    /// BLAKE3 identity over the admitted facts in field order.
    pub identity: [u8; 32],
}

/// Feature-policy admission failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// A denied feature was enabled; names the feature.
    UnsupportedFeature(&'static str),
    /// Fuel interruption must be on for deadline enforcement to exist.
    FuelInterruptionRequired,
    /// Non-canonical NaNs break deterministic runs.
    NanCanonicalizationRequired,
    /// Relaxed SIMD must be pinned to its deterministic form.
    RelaxedSimdDeterministicRequired,
    /// Fixed memory growth is the only admitted policy.
    MemoryGrowthMustBeFixed,
    /// Fixed table growth is the only admitted policy.
    TableGrowthMustBeFixed,
}

/// Name the first denied feature the profile enables, if any.
pub fn first_denied(features: &WasmtimeFeatures) -> Option<Error> {
    assert_eq!(DENIED_FEATURES.len(), 12);
    let enabled = [
        (features.tail_call, "tail_call"),
        (features.wide_arithmetic, "wide_arithmetic"),
        (features.memory64, "memory64"),
        (features.multi_memory, "multi_memory"),
        (features.threads, "threads"),
        (features.gc, "gc"),
        (features.exceptions, "exceptions"),
        (features.relaxed_simd, "relaxed_simd"),
        (features.custom_page_sizes, "custom_page_sizes"),
        (features.extended_const, "extended_const"),
        (features.function_references, "function_references"),
        (features.component_async, "component_async"),
    ];
    assert_eq!(enabled.len(), DENIED_FEATURES.len());
    enabled
        .into_iter()
        .find(|(enabled, _)| *enabled)
        .map(|(_, name)| Error::UnsupportedFeature(name))
}

/// Determinism controls are mandatory for bounded execution.
fn check_determinism(determinism: &Determinism) -> Result<(), Error> {
    if !determinism.fuel_interruption {
        return Err(Error::FuelInterruptionRequired);
    }
    if !determinism.nan_canonicalization {
        return Err(Error::NanCanonicalizationRequired);
    }
    if !determinism.relaxed_simd_deterministic {
        return Err(Error::RelaxedSimdDeterministicRequired);
    }
    if !determinism.memory_growth_fixed {
        return Err(Error::MemoryGrowthMustBeFixed);
    }
    if !determinism.table_growth_fixed {
        return Err(Error::TableGrowthMustBeFixed);
    }
    Ok(())
}

/// Admit a complete explicit feature policy or name the violated fact.
pub fn admit(features: WasmtimeFeatures, determinism: Determinism) -> Result<Admitted, Error> {
    assert_eq!(DENIED_FEATURES.len(), 12);
    if let Some(error) = first_denied(&features) {
        return Err(error);
    }
    check_determinism(&determinism)?;

    let mut input = Vec::new();
    input.extend_from_slice(IDENTITY_CONTEXT);
    input.extend_from_slice(&[u8::from(features.bulk_memory)]);
    input.extend_from_slice(&[u8::from(features.component_model)]);
    input.extend_from_slice(&[u8::from(features.multi_value)]);
    input.extend_from_slice(&[u8::from(features.reference_types)]);
    input.extend_from_slice(&[u8::from(features.simd)]);
    input.extend_from_slice(&[u8::from(determinism.fuel_interruption)]);
    input.extend_from_slice(&[u8::from(determinism.nan_canonicalization)]);
    input.extend_from_slice(&[u8::from(determinism.relaxed_simd_deterministic)]);
    input.extend_from_slice(&[u8::from(determinism.memory_growth_fixed)]);
    input.extend_from_slice(&[u8::from(determinism.table_growth_fixed)]);
    let identity = *blake3::hash(&input).as_bytes();
    assert_eq!(identity.len(), 32);

    Ok(Admitted {
        features,
        determinism,
        identity,
    })
}
