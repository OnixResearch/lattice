pub const MODULE: &[u8] = br#"(module
  (memory (export "memory") 1)
  (func (export "run") (param i32 i32) (result i32)
    i32.const 0 i32.const 19279 i32.store16 i32.const 2))"#;

pub fn profile() -> hardened_wasmtime_runtime::profile::document::Admitted {
    hardened_wasmtime_runtime::profile::document::admit(include_str!(
        "../../../../fixtures/aspen/molten-profile.json"
    ))
    .expect("frozen profile")
}
pub fn engine() -> hardened_wasmtime_shell::BoundedRuntime {
    engine_with_bounds(profile().limits)
}
pub fn engine_with_bounds(
    bounds: hardened_wasmtime_runtime::resource::admission::RuntimeBounds,
) -> hardened_wasmtime_shell::BoundedRuntime {
    hardened_wasmtime_shell::BoundedRuntime::build(
        &profile().policy,
        bounds,
        hardened_wasmtime_runtime::Strategy::Cranelift,
    )
    .expect("admitted engine")
}
pub fn module_facts(
    bytes: &[u8],
) -> hardened_wasmtime_runtime::artifact::shape::MaterializationFacts {
    facts(
        bytes,
        hardened_wasmtime_runtime::artifact::shape::Class::CoreModule,
        "actor",
        &["memory", "run"],
    )
}
pub fn facts(
    bytes: &[u8],
    class: hardened_wasmtime_runtime::artifact::shape::Class,
    world: &str,
    exports: &[&str],
) -> hardened_wasmtime_runtime::artifact::shape::MaterializationFacts {
    hardened_wasmtime_runtime::artifact::shape::MaterializationFacts {
        artifact_identity: blake3::hash(bytes).to_hex().to_string(),
        class,
        world: hardened_wasmtime_runtime::artifact::shape::WorldIdentity {
            package: format!("consumer:{world}@1.0.0"),
            world: world.to_owned(),
        },
        imports: std::collections::BTreeSet::new(),
        exports: exports.iter().map(|name| function(&[*name])).collect(),
    }
}
pub fn function(parts: &[&str]) -> hardened_wasmtime_runtime::artifact::shape::InterfaceFunction {
    hardened_wasmtime_runtime::artifact::shape::InterfaceFunction {
        qualified_name: hardened_wasmtime_shell::qualified_name(parts),
        schema_identity: String::from("reviewed:bytes-v1"),
    }
}
pub fn options<'a>(
    bytes: &'a [u8],
    facts: &'a hardened_wasmtime_runtime::artifact::shape::MaterializationFacts,
    service: &'a hardened_wasmtime_shell::EpochService,
) -> hardened_wasmtime_shell::PrepareOptions<'a> {
    hardened_wasmtime_shell::PrepareOptions {
        artifact: hardened_wasmtime_shell::Artifact { bytes, facts },
        descriptors: &[],
        deadline: hardened_wasmtime_runtime::epoch::deadline::Budget { budget_ticks: 10 },
        epoch_service: Some(service),
    }
}
pub struct Body<'a> {
    pub action: &'a str,
    pub cleanup: &'a str,
}
pub fn component(export: &str, body: Body<'_>) -> Vec<u8> {
    format!(
        r#"(component
      (core module $m
        (memory (export "memory") 1)
        (func (export "cleanup") (param i32) {cleanup})
        (func (export "realloc") (param i32 i32 i32 i32) (result i32) i32.const 1024)
        (func (export "run") (param i32 i32) (result i32)
          {action}
          i32.const 0 i32.const 0 i32.store
          i32.const 4 i32.const 32 i32.store
          i32.const 8 i32.const 2 i32.store
          i32.const 32 i32.const 19279 i32.store16
          i32.const 0))
      (core instance $i (instantiate $m))
      (alias core export $i "memory" (core memory $memory))
      (alias core export $i "realloc" (core func $realloc))
      (alias core export $i "run" (core func $run))
      (alias core export $i "cleanup" (core func $cleanup))
      (type $call (func (param "input" (list u8)) (result (result (list u8) (error string)))))
      (func $call (type $call) (canon lift (core func $run) (memory $memory) (realloc $realloc)
        (post-return $cleanup)))
      (export "{export}" (func $call)))"#,
        action = body.action,
        cleanup = body.cleanup,
    )
    .into_bytes()
}
