const GUEST: &[u8] = br#"(component
  (import "host-step" (func $host (result u32)))
  (core func $lower (canon lower (func $host)))
  (core instance $host (export "step" (func $lower)))
  (core module $m
    (import "host" "step" (func $step (result i32)))
    (memory (export "memory") 1)
    (func (export "realloc") (param i32 i32 i32 i32) (result i32) i32.const 1024)
    (func (export "run") (param i32 i32) (result i32)
      i32.const 32 call $step i32.store16
      i32.const 0 i32.const 0 i32.store
      i32.const 4 i32.const 32 i32.store
      i32.const 8 i32.const 2 i32.store i32.const 0))
  (core instance $i (instantiate $m (with "host" (instance $host))))
  (alias core export $i "memory" (core memory $memory))
  (alias core export $i "realloc" (core func $realloc))
  (alias core export $i "run" (core func $run))
  (func (export "invoke") (param "input" (list u8)) (result (result (list u8) (error string)))
    (canon lift (core func $run) (memory $memory) (realloc $realloc))))"#;

#[test]
// r[verify onix.wasmtime_runtime.restricted_imports.valid]
// r[verify onix.wasmtime_runtime.normalized_outcomes.unknown]
fn component_bindings_return_exact_bytes_and_unknown_errors_fail_closed() {
    let runtime = super::support::engine();
    let service = runtime.epoch_service();
    let mut facts = super::support::facts(
        GUEST,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "actor",
        &["invoke"],
    );
    let function = super::support::function(&["host-step"]);
    facts.imports.insert(function.clone());
    let descriptor = hardened_wasmtime_runtime::restricted::imports::Descriptor {
        function,
        world: facts.world.clone(),
        capability_id: String::from("observed:turn-1"),
        adapter_id: String::from("consumer:step-v1"),
    };
    let mut options = super::support::options(GUEST, &facts, &service);
    options.descriptors = std::slice::from_ref(&descriptor);
    let invocation = hardened_wasmtime_shell::ComponentInvocation::prepare(&runtime, options)
        .expect("exact imports");
    let result = invocation.run("invoke", b"", (), |_, linker| {
        linker
            .root()
            .func_wrap("host-step", |_, (): ()| Ok((19279u32,)))
            .map_err(|_| hardened_wasmtime_shell::HostDenied)?;
        Ok(())
    });
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(result.output, b"OK");
    let unknown = invocation.run("invoke", b"", (), |_, linker| {
        linker
            .root()
            .func_wrap(
                "host-step",
                |_, (): ()| -> Result<(u32,), wasmtime::Error> {
                    Err(wasmtime::Error::msg("opaque consumer failure"))
                },
            )
            .map_err(|_| hardened_wasmtime_shell::HostDenied)?;
        Ok(())
    });
    assert_eq!(
        unknown.class,
        hardened_wasmtime_runtime::outcomes::Class::RuntimeFailed
    );
    assert!(unknown.output.is_empty());
}
