const INTERFACE: &str = "consumer:bytes/methods@1.0.0";

fn component(body: super::support::Body<'_>) -> Vec<u8> {
    format!(
        r#"(component
      (core module $m
        (memory (export "memory") 1)
        (global $heap (mut i32) (i32.const 1024))
        (func (export "realloc") (param i32 i32 i32) (param $size i32) (result i32)
          (local $ptr i32)
          global.get $heap local.tee $ptr local.get $size i32.add global.set $heap
          local.get $ptr)
        (func (export "run") (param i32 i32 i32 i32) (result i32)
          {action}
          i32.const 0 local.get 2 i32.store
          i32.const 4 local.get 3 i32.store
          i32.const 0)
        (func (export "cleanup") (param i32)
          i32.const 1024 i32.const 0 i32.const 512 memory.fill {cleanup}))
      (core instance $i (instantiate $m))
      (alias core export $i "memory" (core memory $memory))
      (alias core export $i "realloc" (core func $realloc))
      (alias core export $i "run" (core func $run))
      (alias core export $i "cleanup" (core func $cleanup))
      (type $call (func (param "method" string) (param "input" (list u8)) (result (list u8))))
      (func $call (type $call) (canon lift (core func $run) (memory $memory)
        (realloc $realloc) (post-return $cleanup)))
      (instance $methods (export "invoke" (func $call)))
      (export "{INTERFACE}" (instance $methods)))"#,
        action = body.action,
        cleanup = body.cleanup,
    )
    .into_bytes()
}

fn prepare(
    bytes: &[u8],
    bounds: hardened_wasmtime_runtime::resource::admission::RuntimeBounds,
) -> hardened_wasmtime_shell::ComponentInvocation {
    let runtime = super::support::engine_with_bounds(bounds);
    let service = runtime.epoch_service();
    let mut facts = super::support::facts(
        bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::Component,
        "method",
        &[],
    );
    facts
        .exports
        .insert(super::support::function(&[INTERFACE, "invoke"]));
    hardened_wasmtime_shell::ComponentInvocation::prepare(
        &runtime,
        super::support::options(bytes, &facts, &service),
    )
    .expect("exact component")
}

#[test]
// r[verify onix.wasmtime_runtime.named_byte_method]
fn named_method_owns_exact_input_output_before_cleanup() {
    let bytes = component(super::support::Body {
        action: "local.get 0 i32.load i32.const 1869112165 i32.ne if unreachable end",
        cleanup: "",
    });
    let mut bounds = super::support::profile().limits;
    bounds.max_input_bytes = 11; // four method bytes and seven input bytes
    let method = prepare(&bytes, bounds)
        .into_byte_method(INTERFACE, "invoke")
        .expect("named carrier");
    let result = method.run("echo", b"payload");
    assert_eq!(
        result.class,
        hardened_wasmtime_runtime::outcomes::Class::Completed
    );
    assert_eq!(result.output, b"payload");
    let refused = method.run("echo", b"payload!");
    assert_eq!(
        refused.class,
        hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded
    );
    assert!(refused.output.is_empty());
}

#[test]
// r[verify onix.wasmtime_runtime.named_byte_method]
fn missing_or_incompatible_named_carrier_is_denied_before_start_trap() {
    let bytes = component(super::support::Body {
        action: "",
        cleanup: "",
    });
    let bounds = super::support::profile().limits;
    assert!(matches!(
        prepare(&bytes, bounds).into_byte_method("absent", "invoke"),
        Err(hardened_wasmtime_shell::PrepareError::ArtifactMismatch)
    ));
    assert!(matches!(
        prepare(&bytes, bounds).into_byte_method(INTERFACE, "absent"),
        Err(hardened_wasmtime_shell::PrepareError::ArtifactMismatch)
    ));
    let wrong = String::from_utf8(bytes).expect("WAT")
        .replace("(param \"method\" string)", "(param \"method\" (list u8))")
        .replace("(core instance $i", "(core module $start (func $trap unreachable) (start $trap)) (core instance (instantiate $start)) (core instance $i");
    assert!(matches!(
        prepare(wrong.as_bytes(), bounds).into_byte_method(INTERFACE, "invoke"),
        Err(hardened_wasmtime_shell::PrepareError::ArtifactMismatch)
    ));
}

#[test]
// r[verify onix.wasmtime_runtime.named_byte_method]
fn named_method_output_fuel_and_automatic_cleanup_failures_discard_output() {
    let cases = [
        (
            "",
            "",
            Some(2),
            hardened_wasmtime_runtime::outcomes::Class::OutputLimitExceeded,
        ),
        (
            "loop br 0 end",
            "",
            None,
            hardened_wasmtime_runtime::outcomes::Class::FuelExhausted,
        ),
        (
            "unreachable",
            "",
            None,
            hardened_wasmtime_runtime::outcomes::Class::GuestTrapped,
        ),
        (
            "",
            "unreachable",
            None,
            hardened_wasmtime_runtime::outcomes::Class::GuestTrapped,
        ),
    ];
    for (action, cleanup, output_limit, expected) in cases {
        let bytes = component(super::support::Body { action, cleanup });
        let mut bounds = super::support::profile().limits;
        if let Some(limit) = output_limit {
            bounds.max_output_bytes = limit;
        }
        let method = prepare(&bytes, bounds)
            .into_byte_method(INTERFACE, "invoke")
            .expect("carrier");
        let outcome = method.run("echo", b"payload");
        assert_eq!(outcome.class, expected);
        assert!(outcome.output.is_empty());
    }
}
