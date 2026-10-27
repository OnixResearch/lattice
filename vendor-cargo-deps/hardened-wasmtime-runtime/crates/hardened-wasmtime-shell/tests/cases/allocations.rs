//! Actual allocations distinguish independent per-memory and cumulative ceilings.
fn two_memory_component(action: &str) -> Vec<u8> {
    let bytes = String::from_utf8(super::support::component(
        "invoke",
        super::support::Body {
            action,
            cleanup: "",
        },
    ))
    .expect("WAT")
    .replace(
        "(core instance $i (instantiate $m))",
        "(core instance $extra (instantiate $m)) (core instance $i (instantiate $m))",
    );
    bytes.into_bytes()
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.memory]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn component_memories_share_one_invocation_memory_budget() {
    const TOTAL_PAGES: u32 = 1;
    let bytes = two_memory_component("");
    let mut bounds = super::support::profile().limits;
    bounds.max_total_memory_pages = TOTAL_PAGES;
    let result = super::resources::run_component(&bytes, bounds);
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded
    );
    assert!(result.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn two_component_memories_succeed_at_the_exact_aggregate_cap() {
    const PER_MEMORY_PAGES: u32 = 1;
    const TOTAL_PAGES: u32 = 2;
    let bytes = two_memory_component("");
    let mut bounds = super::support::profile().limits;
    bounds.max_memory_pages = PER_MEMORY_PAGES;
    bounds.max_total_memory_pages = TOTAL_PAGES;
    let result = super::resources::run_component(&bytes, bounds);
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(result.output, b"OK");
}

#[test]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn component_per_memory_cap_denies_growth_with_aggregate_capacity_remaining() {
    const PER_MEMORY_PAGES: u32 = 1;
    const TOTAL_PAGES: u32 = 3;
    let bytes = two_memory_component("i32.const 1 memory.grow drop");
    let mut bounds = super::support::profile().limits;
    bounds.max_memory_pages = PER_MEMORY_PAGES;
    bounds.max_total_memory_pages = TOTAL_PAGES;
    let result = super::resources::run_component(&bytes, bounds);
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded
    );
    assert!(result.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn zero_memory_caps_deny_engine_construction_independently() {
    let profile = super::support::profile();
    let mut zero_per_memory = profile.limits;
    zero_per_memory.max_memory_pages = 0;
    let mut zero_aggregate = profile.limits;
    zero_aggregate.max_total_memory_pages = 0;
    for bounds in [zero_per_memory, zero_aggregate] {
        assert!(matches!(
            hardened_wasmtime_shell::BoundedRuntime::build(
                &profile.policy,
                bounds,
                hardened_wasmtime_runtime::Strategy::Cranelift,
            ),
            Err(hardened_wasmtime_shell::BuildFailure::BoundsDenied)
        ));
    }
}

#[test]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn frozen_profile_retains_its_cumulative_memory_ceiling() {
    const HALF_PROFILE_PAGES: u32 = 128;
    let bounds = super::support::profile().limits;
    let component = |action| {
        String::from_utf8(two_memory_component(action))
            .expect("WAT")
            .replace(
                "(memory (export \"memory\") 1)",
                &format!("(memory (export \"memory\") {HALF_PROFILE_PAGES})"),
            )
            .into_bytes()
    };
    let allowed = super::resources::run_component(&component(""), bounds);
    assert_eq!(
        allowed.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(allowed.output, b"OK");
    let denied =
        super::resources::run_component(&component("i32.const 1 memory.grow drop"), bounds);
    assert_eq!(
        denied.class,
        hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded
    );
    assert!(denied.output.is_empty());
}
