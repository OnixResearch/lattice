pub(super) struct Fixture {
    pub bytes: Vec<u8>,
    pub facts: hardened_wasmtime_runtime::artifact::shape::MaterializationFacts,
    pub descriptor: hardened_wasmtime_runtime::restricted::imports::Descriptor,
}
pub(super) fn fixture(module: &str) -> Fixture {
    fixture_with_action(Action { module, wat: "" })
}
pub(super) struct Action<'a> {
    pub module: &'a str,
    pub wat: &'a str,
}
fn module_bytes(input: &Action<'_>) -> Vec<u8> {
    format!(
        r#"(module (import "{module}" "step" (func $step (result i32)))
        (memory (export "memory") 1)
        (func (export "run") (param i32 i32) (result i32)
          i32.const 0 call $step i32.store16 {action} i32.const 2))"#,
        module = input.module,
        action = input.wat,
    )
    .into_bytes()
}
pub(super) fn fixture_with_action(input: Action<'_>) -> Fixture {
    let bytes = module_bytes(&input);
    let mut facts = super::support::module_facts(&bytes);
    let function = super::support::function(&[input.module, "step"]);
    facts.imports.insert(function.clone());
    let descriptor = hardened_wasmtime_runtime::restricted::imports::Descriptor {
        function,
        world: facts.world.clone(),
        capability_id: String::from("admitted:turn-1"),
        adapter_id: String::from("consumer:step-v1"),
    };
    Fixture {
        bytes,
        facts,
        descriptor,
    }
}

#[test]
// r[verify onix.wasmtime_runtime.restricted_imports.valid]
fn exact_consumer_binding_executes_without_ambient_access() {
    let fixture = fixture("consumer");
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    let mut options = super::support::options(&fixture.bytes, &fixture.facts, &service);
    options.descriptors = std::slice::from_ref(&fixture.descriptor);
    let invocation = hardened_wasmtime_shell::BoundedInvocation::prepare(&runtime, options)
        .expect("exact imports");
    let result = invocation.run(b"", (), |_, linker| {
        linker
            .func_wrap("consumer", "step", || 19279i32)
            .map_err(|_| hardened_wasmtime_shell::HostDenied)?;
        Ok(())
    });
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(result.output, b"OK");
    let missing = invocation.run(b"", (), |_, _| Ok(()));
    assert_eq!(
        missing.class,
        hardened_wasmtime_runtime::outcomes::Class::InstantiateFailed
    );
}

#[test]
// r[verify onix.wasmtime_runtime.normalized_outcomes.host]
fn consumer_denial_is_not_a_guest_trap() {
    let fixture = fixture("consumer");
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    let mut options = super::support::options(&fixture.bytes, &fixture.facts, &service);
    options.descriptors = std::slice::from_ref(&fixture.descriptor);
    let invocation = hardened_wasmtime_shell::BoundedInvocation::prepare(&runtime, options)
        .expect("exact imports");
    let result = invocation.run(b"", (), |_, linker| {
        linker
            .func_wrap("consumer", "step", || -> Result<i32, wasmtime::Error> {
                Err(wasmtime::Error::new(hardened_wasmtime_shell::HostDenied))
            })
            .map_err(|_| hardened_wasmtime_shell::HostDenied)?;
        Ok(())
    });
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::HostFailed
    );
    assert!(result.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.restricted_imports.undeclared]
// r[verify onix.wasmtime_runtime.restricted_imports.wasi]
fn undeclared_and_wasi_are_denied_before_binding() {
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    let fixture = fixture("consumer");
    assert!(matches!(
        hardened_wasmtime_shell::BoundedInvocation::prepare(
            &runtime,
            super::support::options(&fixture.bytes, &fixture.facts, &service)
        ),
        Err(hardened_wasmtime_shell::PrepareError::ImportMismatch)
    ));
    let wasi = self::fixture("wasi_snapshot_preview1");
    let mut options = super::support::options(&wasi.bytes, &wasi.facts, &service);
    options.descriptors = std::slice::from_ref(&wasi.descriptor);
    assert!(matches!(
        hardened_wasmtime_shell::BoundedInvocation::prepare(&runtime, options),
        Err(hardened_wasmtime_shell::PrepareError::ImportAdmission(
            hardened_wasmtime_runtime::restricted::imports::ImportError::WasiImport(_)
        ))
    ));
}
