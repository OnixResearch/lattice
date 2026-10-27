#[test]
// r[verify onix.wasmtime_runtime.slice_limits]
fn memory_growth_after_suspension_cannot_escape_the_invocation_budget() {
    let action = format!(
        "{} i32.const 1 memory.grow drop",
        super::slicesupport::FINITE_WORK
    );
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: &action,
            cleanup: "",
        },
    );
    let mut resource_bounds = super::slicesupport::bounds();
    resource_bounds.max_total_memory_pages = 1;
    let (prepared, _) =
        super::slicesupport::prepare(&bytes, resource_bounds, super::slicesupport::DEADLINE_TICKS);
    let mut invocation = super::slicesupport::start(&prepared);
    let (outcome, yields) = super::slicesupport::finish(&mut invocation);
    assert!(yields >= super::slicesupport::MIN_YIELDS);
    assert_eq!(
        outcome.class,
        hardened_wasmtime_runtime::outcomes::Class::MemoryLimitExceeded
    );
    assert_eq!(
        outcome.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
    );
    assert!(outcome.output.is_empty());
    assert!(outcome.fuel_remaining < super::slicesupport::TOTAL_FUEL);
}

#[test]
// r[verify onix.wasmtime_runtime.slice_limits]
fn oversized_owned_input_is_denied_before_retaining_a_future() {
    const INPUT_LIMIT_BYTES: u64 = 1;
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "",
            cleanup: "",
        },
    );
    let mut resource_bounds = super::slicesupport::bounds();
    resource_bounds.max_input_bytes = INPUT_LIMIT_BYTES;
    let (prepared, _) =
        super::slicesupport::prepare(&bytes, resource_bounds, super::slicesupport::DEADLINE_TICKS);
    let mut request = super::slicesupport::options(super::slicesupport::QUANTUM_FUEL);
    request.input = b"too long".to_vec();
    let failure = std::sync::Arc::clone(&prepared)
        .start_sliced(request, (), |_, _| Err(hardened_wasmtime_shell::HostDenied))
        .err()
        .expect("oversized input denied");
    assert_eq!(
        failure.class,
        hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded
    );
    assert_eq!(
        failure.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Admission
    );
    assert!(failure.output.is_empty());
    let mut accepted = super::slicesupport::start(&prepared);
    let (outcome, _) = super::slicesupport::finish(&mut accepted);
    assert_eq!(
        outcome.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(outcome.output, b"OK");
}
