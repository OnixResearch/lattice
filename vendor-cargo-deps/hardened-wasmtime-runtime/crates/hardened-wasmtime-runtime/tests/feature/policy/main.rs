//! Integration tests for feature::policy.

use hardened_wasmtime_runtime::feature::policy::*;

/// The admitted molten.wasm.component.v1 feature facts.
const fn molten_features() -> WasmtimeFeatures {
    WasmtimeFeatures {
        bulk_memory: true,
        component_model: true,
        multi_value: true,
        reference_types: true,
        simd: true,
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
    }
}

fn molten_determinism() -> Determinism {
    Determinism {
        fuel_interruption: true,
        nan_canonicalization: true,
        relaxed_simd_deterministic: true,
        memory_growth_fixed: true,
        table_growth_fixed: true,
    }
}

#[test]
// r[verify onix.wasmtime_runtime.feature_policy.valid]
fn complete_profile_admits_with_stable_identity() -> Result<(), Error> {
    let admitted = admit(molten_features(), molten_determinism())?;
    let again = admit(molten_features(), molten_determinism())?;
    assert_eq!(admitted.identity, again.identity);
    assert_ne!(admitted.identity, [0_u8; 32]);
    Ok(())
}

const DENIED_INSTRUCTION_SETS: &[(&str, WasmtimeFeatures)] = &{
    let base = molten_features();
    [
        (
            "tail_call",
            WasmtimeFeatures {
                tail_call: true,
                ..base
            },
        ),
        (
            "wide_arithmetic",
            WasmtimeFeatures {
                wide_arithmetic: true,
                ..base
            },
        ),
        (
            "extended_const",
            WasmtimeFeatures {
                extended_const: true,
                ..base
            },
        ),
        (
            "function_references",
            WasmtimeFeatures {
                function_references: true,
                ..base
            },
        ),
    ]
};

const DENIED_MEMORY_MODELS: &[(&str, WasmtimeFeatures)] = &{
    let base = molten_features();
    [
        (
            "memory64",
            WasmtimeFeatures {
                memory64: true,
                ..base
            },
        ),
        (
            "multi_memory",
            WasmtimeFeatures {
                multi_memory: true,
                ..base
            },
        ),
        ("gc", WasmtimeFeatures { gc: true, ..base }),
        (
            "custom_page_sizes",
            WasmtimeFeatures {
                custom_page_sizes: true,
                ..base
            },
        ),
    ]
};

const DENIED_EXECUTION_MODES: &[(&str, WasmtimeFeatures)] = &{
    let base = molten_features();
    [
        (
            "threads",
            WasmtimeFeatures {
                threads: true,
                ..base
            },
        ),
        (
            "exceptions",
            WasmtimeFeatures {
                exceptions: true,
                ..base
            },
        ),
        (
            "relaxed_simd",
            WasmtimeFeatures {
                relaxed_simd: true,
                ..base
            },
        ),
        (
            "component_async",
            WasmtimeFeatures {
                component_async: true,
                ..base
            },
        ),
    ]
};

#[test]
// r[verify onix.wasmtime_runtime.feature_policy.unsupported]
fn denied_instruction_sets_fail_by_name() {
    for &(name, features) in DENIED_INSTRUCTION_SETS {
        assert_eq!(
            admit(features, molten_determinism()),
            Err(Error::UnsupportedFeature(name)),
            "denied fact {name} must fail by name"
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.feature_policy.unsupported]
fn denied_memory_models_fail_by_name() {
    for &(name, features) in DENIED_MEMORY_MODELS {
        assert_eq!(
            admit(features, molten_determinism()),
            Err(Error::UnsupportedFeature(name)),
            "denied fact {name} must fail by name"
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.feature_policy.unsupported]
fn denied_execution_modes_fail_by_name() {
    for &(name, features) in DENIED_EXECUTION_MODES {
        assert_eq!(
            admit(features, molten_determinism()),
            Err(Error::UnsupportedFeature(name)),
            "denied fact {name} must fail by name"
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.feature_policy.valid]
fn determinism_facts_are_required() {
    let mut determinism = molten_determinism();
    determinism.fuel_interruption = false;
    assert_eq!(
        admit(molten_features(), determinism),
        Err(Error::FuelInterruptionRequired)
    );

    let mut determinism = molten_determinism();
    determinism.nan_canonicalization = false;
    assert_eq!(
        admit(molten_features(), determinism),
        Err(Error::NanCanonicalizationRequired)
    );

    let mut determinism = molten_determinism();
    determinism.relaxed_simd_deterministic = false;
    assert_eq!(
        admit(molten_features(), determinism),
        Err(Error::RelaxedSimdDeterministicRequired)
    );

    let mut determinism = molten_determinism();
    determinism.memory_growth_fixed = false;
    assert_eq!(
        admit(molten_features(), determinism),
        Err(Error::MemoryGrowthMustBeFixed)
    );

    let mut determinism = molten_determinism();
    determinism.table_growth_fixed = false;
    assert_eq!(
        admit(molten_features(), determinism),
        Err(Error::TableGrowthMustBeFixed)
    );
}

#[test]
fn identity_binds_explicit_facts() -> Result<(), Error> {
    let mut different = molten_features();
    different.bulk_memory = false;
    let admitted = admit(molten_features(), molten_determinism())?;
    let other = admit(different, molten_determinism())?;
    assert_ne!(admitted.identity, other.identity);
    Ok(())
}
