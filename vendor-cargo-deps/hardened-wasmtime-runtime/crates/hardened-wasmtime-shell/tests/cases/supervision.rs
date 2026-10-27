#[test]
// r[verify onix.wasmtime_runtime.epoch_deadline.exhausted]
fn an_injected_tick_interrupts_active_guest_execution() {
    let fixture = super::imports::fixture_with_action(super::imports::Action {
        module: "consumer",
        wat: "(loop br 0)",
    });
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    let mut options = super::support::options(&fixture.bytes, &fixture.facts, &service);
    options.descriptors = std::slice::from_ref(&fixture.descriptor);
    options.deadline.budget_ticks = 1;
    let invocation =
        hardened_wasmtime_shell::BoundedInvocation::prepare(&runtime, options).expect("module");
    let driver = service.clone();
    let result = invocation.run(b"", (), |_, linker| {
        let driver = driver.clone();
        linker
            .func_wrap("consumer", "step", move || {
                driver.advance();
                19279i32
            })
            .map_err(|_| hardened_wasmtime_shell::HostDenied)?;
        Ok(())
    });
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::DeadlineExhausted
    );
    assert!(result.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.resource_limits]
fn shared_engine_concurrency_is_bounded_and_released() {
    let fixture = super::imports::fixture("consumer");
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "",
            cleanup: "",
        },
    );
    let facts = super::support::facts(
        &bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let cloned_runtime = runtime.clone();
    let component = hardened_wasmtime_shell::ComponentInvocation::prepare(
        &cloned_runtime,
        super::support::options(&bytes, &facts, &service),
    )
    .expect("component");
    let mut options = super::support::options(&fixture.bytes, &fixture.facts, &service);
    options.descriptors = std::slice::from_ref(&fixture.descriptor);
    let invocation =
        hardened_wasmtime_shell::BoundedInvocation::prepare(&runtime, options).expect("module");
    let result = invocation.run(b"", (), |_, linker| {
        let nested = component.run("invoke", b"", (), |_, _| {
            Err(hardened_wasmtime_shell::HostDenied)
        });
        assert_eq!(
            nested.class,
            hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted
        );
        linker
            .func_wrap("consumer", "step", || 19279i32)
            .map_err(|_| hardened_wasmtime_shell::HostDenied)?;
        Ok(())
    });
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    let denied = invocation.run(b"", (), |_, _| Err(hardened_wasmtime_shell::HostDenied));
    assert_eq!(
        denied.class,
        hardened_wasmtime_runtime::outcomes::Class::HostFailed
    );
    let after = component.run("invoke", b"", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(after.output, b"OK");
}
