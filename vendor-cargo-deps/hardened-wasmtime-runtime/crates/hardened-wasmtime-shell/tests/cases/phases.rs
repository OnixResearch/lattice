struct Body<'a> {
    start: &'a str,
    action: &'a str,
}

fn component(body: Body<'_>) -> Vec<u8> {
    format!(
        include_str!("phasecomponent.wat"),
        start = body.start,
        action = body.action,
    )
    .into_bytes()
}

fn prepare(
    runtime: &hardened_wasmtime_shell::BoundedRuntime,
    bytes: &[u8],
) -> hardened_wasmtime_shell::ComponentInvocation {
    let service = runtime.epoch_service();
    let mut facts = super::support::facts(
        bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let descriptor = super::slicesupport::marker_descriptor(&facts);
    facts.imports.insert(descriptor.function.clone());
    let mut options = super::support::options(bytes, &facts, &service);
    options.descriptors = std::slice::from_ref(&descriptor);
    hardened_wasmtime_shell::ComponentInvocation::prepare(runtime, options)
        .expect("admitted component and host import")
}

fn bind_step(
    _: &hardened_wasmtime_runtime::restricted::imports::Descriptor,
    linker: &mut wasmtime::component::Linker<hardened_wasmtime_shell::State<()>>,
) -> Result<(), hardened_wasmtime_shell::HostDenied> {
    linker
        .root()
        .func_wrap("host-step", |_, (): ()| Ok(()))
        .map_err(|_| hardened_wasmtime_shell::HostDenied)
}

#[test]
// r[verify onix.wasmtime_runtime.failure_phase]
fn start_and_export_traps_remain_distinct() {
    let runtime = super::support::engine();
    for (start, action, phase) in [
        (
            "unreachable",
            "",
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation,
        ),
        (
            "",
            "unreachable",
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation,
        ),
    ] {
        let bytes = component(Body { start, action });
        let invocation = prepare(&runtime, &bytes);
        let outcome = invocation.run("invoke", b"", (), bind_step);
        assert_eq!(
            outcome.class,
            hardened_wasmtime_runtime::outcomes::Class::GuestTrapped
        );
        assert_eq!(outcome.phase, phase);
        assert!(outcome.output.is_empty());
        assert!(outcome.fuel_remaining < super::support::profile().limits.fuel);
    }
}

#[test]
// r[verify onix.wasmtime_runtime.failure_phase]
fn component_binding_denial_missing_binding_and_completion_keep_their_stages() {
    let runtime = super::support::engine();
    let bytes = component(Body {
        start: "",
        action: "",
    });
    let invocation = prepare(&runtime, &bytes);
    let denied = invocation.run("invoke", b"", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(
        denied.class,
        hardened_wasmtime_runtime::outcomes::Class::HostFailed
    );
    assert_eq!(
        denied.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Linking
    );
    assert!(denied.output.is_empty());
    assert_eq!(denied.fuel_remaining, super::support::profile().limits.fuel);

    let unbound = invocation.run("invoke", b"", (), |_, _| Ok(()));
    assert_eq!(
        unbound.class,
        hardened_wasmtime_runtime::outcomes::Class::InstantiateFailed
    );
    assert_eq!(
        unbound.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation
    );
    assert!(unbound.output.is_empty());

    let completed = invocation.run("invoke", b"", (), bind_step);
    assert_eq!(
        completed.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(
        completed.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
    );
    assert_eq!(completed.output, b"OK");
    assert!(completed.fuel_remaining < super::support::profile().limits.fuel);
}

#[test]
// r[verify onix.wasmtime_runtime.failure_phase]
fn component_input_admission_precedes_linking_and_missing_export_is_invocation() {
    const INPUT_LIMIT_BYTES: u64 = 1;
    let mut bounds = super::support::profile().limits;
    bounds.max_input_bytes = INPUT_LIMIT_BYTES;
    let runtime = super::support::engine_with_bounds(bounds);
    let bytes = component(Body {
        start: "",
        action: "",
    });
    let invocation = prepare(&runtime, &bytes);
    let outcome = invocation.run("invoke", b"too long", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(
        outcome.class,
        hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded
    );
    assert_eq!(
        outcome.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Admission
    );
    assert!(outcome.output.is_empty());

    let missing = invocation.run("absent", b"", (), bind_step);
    assert_eq!(
        missing.class,
        hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied
    );
    assert_eq!(
        missing.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
    );
    assert!(missing.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.failure_phase]
fn component_automatic_cleanup_failure_stays_in_invocation() {
    let bytes = super::support::component(
        "invoke",
        super::support::Body {
            action: "",
            cleanup: "unreachable",
        },
    );
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    let facts = super::support::facts(
        &bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let invocation = hardened_wasmtime_shell::ComponentInvocation::prepare(
        &runtime,
        super::support::options(&bytes, &facts, &service),
    )
    .expect("component with automatic cleanup");
    let outcome = invocation.run("invoke", b"", (), |_, _| {
        Err(hardened_wasmtime_shell::HostDenied)
    });
    assert_eq!(
        outcome.class,
        hardened_wasmtime_runtime::outcomes::Class::GuestTrapped
    );
    assert_eq!(
        outcome.phase,
        hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation
    );
    assert!(outcome.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.failure_phase]
fn module_failures_and_completion_retain_the_reached_stage() {
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    for (start, action, class, phase) in [
        (
            "unreachable",
            "i32.const 0",
            hardened_wasmtime_runtime::outcomes::Class::GuestTrapped,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation,
        ),
        (
            "",
            "unreachable",
            hardened_wasmtime_runtime::outcomes::Class::GuestTrapped,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation,
        ),
        (
            "",
            "i32.const 0",
            hardened_wasmtime_runtime::outcomes::Class::Completed,
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation,
        ),
    ] {
        let bytes = format!(
            r#"(module
          (memory (export "memory") 1)
          (func $start {start}) (start $start)
          (func (export "run") (param i32 i32) (result i32) {action}))"#
        )
        .into_bytes();
        let facts = super::support::module_facts(&bytes);
        let invocation = hardened_wasmtime_shell::BoundedInvocation::prepare(
            &runtime,
            super::support::options(&bytes, &facts, &service),
        )
        .expect("module");
        let outcome = invocation.run(b"", (), |_, _| Err(hardened_wasmtime_shell::HostDenied));
        assert_eq!(outcome.class, class);
        assert_eq!(outcome.phase, phase);
        assert!(outcome.output.is_empty());
    }
}
