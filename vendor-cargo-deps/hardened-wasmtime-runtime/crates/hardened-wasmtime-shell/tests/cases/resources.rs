fn module(action: &str) -> Vec<u8> {
    format!(
        r#"(module (memory (export "memory") 1)
        (func (export "run") (param i32 i32) (result i32) {action}))"#
    )
    .into_bytes()
}
fn run_module(
    bytes: &[u8],
    bounds: hardened_wasmtime_runtime::resource::admission::RuntimeBounds,
    input: &[u8],
) -> hardened_wasmtime_shell::InvocationOutcome {
    let runtime = super::support::engine_with_bounds(bounds);
    let service = runtime.epoch_service();
    let facts = super::support::module_facts(bytes);
    let prepared = hardened_wasmtime_shell::BoundedInvocation::prepare(
        &runtime,
        super::support::options(bytes, &facts, &service),
    )
    .expect("module");
    prepared.run(input, (), |_, _| Err(hardened_wasmtime_shell::HostDenied))
}
pub(super) fn run_component(
    bytes: &[u8],
    bounds: hardened_wasmtime_runtime::resource::admission::RuntimeBounds,
) -> hardened_wasmtime_shell::InvocationOutcome {
    let runtime = super::support::engine_with_bounds(bounds);
    let service = runtime.epoch_service();
    let facts = super::support::facts(
        bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let prepared = hardened_wasmtime_shell::ComponentInvocation::prepare(
        &runtime,
        super::support::options(bytes, &facts, &service),
    )
    .expect("component");
    prepared.run("invoke", b"", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    })
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.fuel]
fn module_and_component_loops_exhaust_one_fuel_budget() {
    let bytes = module("(loop br 0) i32.const 0");
    let result = run_module(&bytes, super::support::profile().limits, b"");
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::FuelExhausted
    );
    let component = super::support::component(
        "invoke",
        super::support::Body {
            action: "(loop br 0)",
            cleanup: "",
        },
    );
    let result = run_component(&component, super::support::profile().limits);
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::FuelExhausted
    );
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.memory]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn guest_memory_growth_denial_has_a_stable_class() {
    let bytes = module("i32.const 1 memory.grow drop i32.const 0");
    let mut bounds = super::support::profile().limits;
    bounds.max_memory_pages = 1;
    let result = run_module(&bytes, bounds, b"");
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded
    );
    assert!(result.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.independent_memory_limits]
fn module_aggregate_cap_denies_growth_below_the_per_memory_cap() {
    const GROWN_PAGES: u32 = 2;
    const INITIAL_PAGES: u32 = 1;
    let bytes =
        module("i32.const 1 memory.grow drop i32.const 0 i32.const 19279 i32.store16 i32.const 2");
    let mut bounds = super::support::profile().limits;
    bounds.max_memory_pages = GROWN_PAGES;
    bounds.max_total_memory_pages = INITIAL_PAGES;
    let denied = run_module(&bytes, bounds, b"");
    assert_eq!(
        denied.class,
        hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded
    );
    assert!(denied.output.is_empty());
    bounds.max_total_memory_pages = GROWN_PAGES;
    let allowed = run_module(&bytes, bounds, b"");
    assert_eq!(
        allowed.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(allowed.output, b"OK");
}

#[test]
// r[verify onix.wasmtime_runtime.guest_memory.range]
fn output_length_cannot_escape_guest_memory() {
    let bytes = module("i32.const 65537");
    let mut bounds = super::support::profile().limits;
    bounds.max_output_bytes = 65537;
    let result = run_module(&bytes, bounds, b"");
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected
    );
    let negative = module("i32.const -1");
    let result = run_module(&negative, bounds, b"");
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected
    );
}

#[test]
// r[verify onix.wasmtime_runtime.guest_memory.overflow]
fn pointer_overflow_and_memory_edge_are_distinct() {
    assert_eq!(
        hardened_wasmtime_shell::GuestSpan::new(u32::MAX, 1),
        Err(hardened_wasmtime_shell::GuestPointerError::Overflow)
    );
    let edge = hardened_wasmtime_shell::GuestSpan::new(65536, 0).expect("empty edge");
    assert_eq!(
        hardened_wasmtime_shell::check_guest_range(edge, 65536),
        Ok(())
    );
    assert_eq!(
        hardened_wasmtime_shell::check_guest_range(edge, 65535),
        Err(hardened_wasmtime_shell::GuestPointerError::OutOfRange)
    );
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.output]
fn input_and_output_limits_apply_at_the_exact_boundary() {
    let mut bounds = super::support::profile().limits;
    bounds.max_input_bytes = 2;
    bounds.max_output_bytes = 2;
    let result = run_module(super::support::MODULE, bounds, b"in");
    assert_eq!(result.output, b"OK");
    let result = run_module(super::support::MODULE, bounds, b"big");
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded
    );
    bounds.max_output_bytes = 1;
    let result = run_module(super::support::MODULE, bounds, b"in");
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::OutputLimitExceeded
    );
    assert!(result.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.fuel]
fn component_post_return_shares_the_fuel_budget() {
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "",
            cleanup: "(loop br 0)",
        },
    );
    let result = run_component(&bytes, super::support::profile().limits);
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::FuelExhausted
    );
    assert!(result.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits.output]
fn component_output_is_owned_before_guest_cleanup() {
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "",
            cleanup: "i32.const 32 i32.const 0 i32.store16",
        },
    );
    let result = run_component(&bytes, super::support::profile().limits);
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(result.output, b"OK");
}
