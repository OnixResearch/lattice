//! Integration tests for restricted::imports.

fn world() -> hardened_wasmtime_runtime::artifact::shape::WorldIdentity {
    hardened_wasmtime_runtime::artifact::shape::WorldIdentity {
        package: String::from("molten:component-runtime@1.0.0"),
        world: String::from("actor"),
    }
}

fn descriptor(name: &str) -> hardened_wasmtime_runtime::restricted::imports::Descriptor {
    hardened_wasmtime_runtime::restricted::imports::Descriptor {
        function: hardened_wasmtime_runtime::artifact::shape::InterfaceFunction {
            qualified_name: String::from(name),
            schema_identity: String::from("blake3-schema-1"),
        },
        world: world(),
        capability_id: String::from("capability-1"),
        adapter_id: String::from("adapter-1"),
    }
}

#[test]
// r[verify onix.wasmtime_runtime.restricted_imports.valid]
fn exact_bound_descriptors_admit() {
    let expected = descriptor("molten:actor/operate@1.0.0.run");
    let admitted = hardened_wasmtime_runtime::restricted::imports::admit(
        std::slice::from_ref(&expected),
        &world(),
    );
    assert_eq!(
        admitted,
        Ok(std::collections::BTreeSet::from([expected.function]))
    );
}

#[test]
// r[verify onix.wasmtime_runtime.restricted_imports.wasi]
fn wasi_names_are_denied_by_name() {
    for name in [
        "wasi:filesystem/types@0.2.6.read",
        "wasi_snapshot_preview1.fd_read",
        "wasi_unstable.random_get",
    ] {
        assert_eq!(
            hardened_wasmtime_runtime::restricted::imports::admit(&[descriptor(name)], &world()),
            Err(
                hardened_wasmtime_runtime::restricted::imports::ImportError::WasiImport(
                    String::from(name)
                )
            )
        );
    }
}

#[test]
// r[verify onix.wasmtime_runtime.restricted_imports.capability]
fn unbound_or_cross_world_descriptors_are_denied() {
    let mut unbound = descriptor("molten:actor/operate@1.0.0.run");
    unbound.capability_id = String::new();
    assert_eq!(
        hardened_wasmtime_runtime::restricted::imports::admit(&[unbound], &world()),
        Err(hardened_wasmtime_runtime::restricted::imports::ImportError::UnboundDescriptor)
    );
    let mut other_world = descriptor("molten:actor/operate@1.0.0.run");
    other_world.world.world = String::from("other");
    assert_eq!(
        hardened_wasmtime_runtime::restricted::imports::admit(&[other_world], &world()),
        Err(hardened_wasmtime_runtime::restricted::imports::ImportError::WorldMismatch)
    );
}

#[test]
fn duplicate_bindings_fail_closed() {
    assert_eq!(
        hardened_wasmtime_runtime::restricted::imports::admit(
            &[
                descriptor("molten:actor/operate@1.0.0.run"),
                descriptor("molten:actor/operate@1.0.0.run"),
            ],
            &world()
        ),
        Err(
            hardened_wasmtime_runtime::restricted::imports::ImportError::DuplicateBinding(
                String::from("molten:actor/operate@1.0.0.run")
            )
        )
    );
}

#[test]
fn an_empty_descriptor_set_admits_nothing() {
    // Deny-by-default: an empty admitted set links nothing.
    assert_eq!(
        hardened_wasmtime_runtime::restricted::imports::admit(&[], &world()),
        Ok(std::collections::BTreeSet::new())
    );
}
