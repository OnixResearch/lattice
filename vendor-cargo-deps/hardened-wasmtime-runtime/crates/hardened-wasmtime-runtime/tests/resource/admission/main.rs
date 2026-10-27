//! Integration tests for resource::limits.

use hardened_wasmtime_runtime::resource::admission::*;

const MOLTEN_STACK_BYTES: u64 = 524_288;
const MOLTEN_PAYLOAD_BYTES: u64 = 65_536;
const MOLTEN_COMPONENT_BYTES: u64 = 4_194_304;
const MOLTEN_MEMORY_PAGES: u32 = 256;

/// The admitted molten.wasm.component.v1 resource limits.
const fn molten_limits() -> RuntimeBounds {
    RuntimeBounds {
        fuel: 100_000,
        max_stack_bytes: MOLTEN_STACK_BYTES,
        max_memory_pages: MOLTEN_MEMORY_PAGES,
        max_total_memory_pages: MOLTEN_MEMORY_PAGES,
        max_tables: 4,
        max_table_elements: 1_024,
        max_instances: 16,
        max_memories: 4,
        max_input_bytes: MOLTEN_PAYLOAD_BYTES,
        max_output_bytes: MOLTEN_PAYLOAD_BYTES,
        max_component_bytes: MOLTEN_COMPONENT_BYTES,
        max_wit_bytes: MOLTEN_PAYLOAD_BYTES,
        max_imports: 128,
        max_exports: 128,
        max_concurrency: 1,
    }
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.valid]
fn molten_limits_admit_exactly() {
    assert_eq!(admit_limits(molten_limits()), Ok(molten_limits()));
}

#[test]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn memory_caps_admit_independently_without_clamping() {
    const SMALL_CAP: u32 = 1;
    const LARGE_CAP: u32 = 2;
    for (max_memory_pages, max_total_memory_pages) in
        [(SMALL_CAP, LARGE_CAP), (LARGE_CAP, SMALL_CAP)]
    {
        let request = RuntimeBounds {
            max_memory_pages,
            max_total_memory_pages,
            ..molten_limits()
        };
        assert_eq!(admit_limits(request), Ok(request));
    }
}

const ZERO_EXECUTION_BUDGETS: &[(&str, RuntimeBounds)] = &{
    let base = molten_limits();
    [
        ("fuel", RuntimeBounds { fuel: 0, ..base }),
        (
            "max_stack_bytes",
            RuntimeBounds {
                max_stack_bytes: 0,
                ..base
            },
        ),
        (
            "max_concurrency",
            RuntimeBounds {
                max_concurrency: 0,
                ..base
            },
        ),
    ]
};

const ZERO_GUEST_ALLOCATIONS: &[(&str, RuntimeBounds)] = &{
    let base = molten_limits();
    [
        (
            "max_memory_pages",
            RuntimeBounds {
                max_memory_pages: 0,
                ..base
            },
        ),
        (
            "max_total_memory_pages",
            RuntimeBounds {
                max_total_memory_pages: 0,
                ..base
            },
        ),
        (
            "max_tables",
            RuntimeBounds {
                max_tables: 0,
                ..base
            },
        ),
        (
            "max_table_elements",
            RuntimeBounds {
                max_table_elements: 0,
                ..base
            },
        ),
        (
            "max_instances",
            RuntimeBounds {
                max_instances: 0,
                ..base
            },
        ),
        (
            "max_memories",
            RuntimeBounds {
                max_memories: 0,
                ..base
            },
        ),
    ]
};

const ZERO_PAYLOAD_BOUNDS: &[(&str, RuntimeBounds)] = &{
    let base = molten_limits();
    [
        (
            "max_input_bytes",
            RuntimeBounds {
                max_input_bytes: 0,
                ..base
            },
        ),
        (
            "max_output_bytes",
            RuntimeBounds {
                max_output_bytes: 0,
                ..base
            },
        ),
        (
            "max_component_bytes",
            RuntimeBounds {
                max_component_bytes: 0,
                ..base
            },
        ),
        (
            "max_wit_bytes",
            RuntimeBounds {
                max_wit_bytes: 0,
                ..base
            },
        ),
    ]
};

const ZERO_INTERFACE_BOUNDS: &[(&str, RuntimeBounds)] = &{
    let base = molten_limits();
    [
        (
            "max_imports",
            RuntimeBounds {
                max_imports: 0,
                ..base
            },
        ),
        (
            "max_exports",
            RuntimeBounds {
                max_exports: 0,
                ..base
            },
        ),
    ]
};

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.fuel]
fn zero_execution_budgets_fail_naming_the_field() {
    for &(name, limits) in ZERO_EXECUTION_BUDGETS {
        assert_eq!(
            admit_limits(limits),
            Err(BoundsError::ZeroLimit(name)),
            "zero {name} must fail by name"
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn zero_guest_allocations_fail_naming_the_field() {
    for &(name, limits) in ZERO_GUEST_ALLOCATIONS {
        assert_eq!(
            admit_limits(limits),
            Err(BoundsError::ZeroLimit(name)),
            "zero {name} must fail by name"
        );
    }
}

#[test]
fn zero_payload_bounds_fail_naming_the_field() {
    for &(name, limits) in ZERO_PAYLOAD_BOUNDS {
        assert_eq!(
            admit_limits(limits),
            Err(BoundsError::ZeroLimit(name)),
            "zero {name} must fail by name"
        );
    }
}

#[test]
fn zero_interface_bounds_fail_naming_the_field() {
    for &(name, limits) in ZERO_INTERFACE_BOUNDS {
        assert_eq!(
            admit_limits(limits),
            Err(BoundsError::ZeroLimit(name)),
            "zero {name} must fail by name"
        );
    }
}

#[test]
fn outcome_classes_are_distinct() {
    let classes = [
        INPUT_LIMIT_CLASS,
        OUTPUT_LIMIT_CLASS,
        MEMORY_LIMIT_CLASS,
        DEADLINE_CLASS,
    ];
    let mut remaining = classes.as_slice();
    while let Some((class, others)) = remaining.split_first() {
        for other in others {
            assert_ne!(class, other);
        }
        remaining = others;
    }
}
