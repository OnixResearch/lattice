//! Integration tests for artifact::shape.

struct InterfaceName<'a> {
    qualified: &'a str,
    schema: &'a str,
}

const INCOMING_FUNCTION: InterfaceName<'static> = InterfaceName {
    qualified: "molten:actor/operate@1.0.0.run",
    schema: "blake3-schema-1",
};
const OUTGOING_FUNCTION: InterfaceName<'static> = InterfaceName {
    qualified: "molten:actor/guest@1.0.0.run",
    schema: "blake3-schema-2",
};

fn interface(
    identity: InterfaceName<'_>,
) -> hardened_wasmtime_runtime::artifact::shape::InterfaceFunction {
    hardened_wasmtime_runtime::artifact::shape::InterfaceFunction {
        qualified_name: String::from(identity.qualified),
        schema_identity: String::from(identity.schema),
    }
}

fn facts() -> hardened_wasmtime_runtime::artifact::shape::MaterializationFacts {
    let incoming = interface(INCOMING_FUNCTION);
    let outgoing = interface(OUTGOING_FUNCTION);
    hardened_wasmtime_runtime::artifact::shape::MaterializationFacts {
        artifact_identity: String::from("blake3-artifact-1"),
        class: hardened_wasmtime_runtime::artifact::shape::Class::Component,
        world: hardened_wasmtime_runtime::artifact::shape::WorldIdentity {
            package: String::from("molten:component-runtime@1.0.0"),
            world: String::from("actor"),
        },
        imports: std::collections::BTreeSet::from([incoming]),
        exports: std::collections::BTreeSet::from([outgoing]),
    }
}

fn inspected() -> hardened_wasmtime_runtime::artifact::shape::Inspected {
    let declared = facts();
    hardened_wasmtime_runtime::artifact::shape::Inspected {
        artifact_identity: declared.artifact_identity.clone(),
        class: declared.class,
        world: declared.world.clone(),
        imports: declared.imports.clone(),
        exports: declared.exports.clone(),
    }
}

#[test]
// r[verify onix.wasmtime_runtime.artifact_shape.valid]
fn agreeing_facts_admit() {
    assert_eq!(
        hardened_wasmtime_runtime::artifact::shape::admit(&facts(), &inspected()),
        Ok(())
    );
}

#[test]
// r[verify onix.wasmtime_runtime.artifact_shape.import]
fn import_drift_denies_by_name() {
    let mut drifted = inspected();
    drifted.imports.clear();
    assert_eq!(
        hardened_wasmtime_runtime::artifact::shape::admit(&facts(), &drifted),
        Err(hardened_wasmtime_runtime::artifact::shape::AdmissionError::ImportDrift)
    );
    let mut extra = inspected();
    extra.imports.insert(interface(InterfaceName {
        qualified: "wasi:filesystem/types@0.2.6.read",
        schema: "blake3-schema-3",
    }));
    assert_eq!(
        hardened_wasmtime_runtime::artifact::shape::admit(&facts(), &extra),
        Err(hardened_wasmtime_runtime::artifact::shape::AdmissionError::ImportDrift)
    );
}

#[test]
// r[verify onix.wasmtime_runtime.artifact_shape.export]
fn export_drift_denies_by_name() {
    let mut drifted = inspected();
    drifted.exports.insert(interface(InterfaceName {
        qualified: "molten:actor/guest@1.0.0.extra",
        schema: "blake3-schema-4",
    }));
    assert_eq!(
        hardened_wasmtime_runtime::artifact::shape::admit(&facts(), &drifted),
        Err(hardened_wasmtime_runtime::artifact::shape::AdmissionError::ExportDrift)
    );
}

#[test]
fn identity_class_and_world_drift_each_deny() {
    let mut drifted = inspected();
    drifted.artifact_identity = String::from("blake3-artifact-2");
    assert_eq!(
        hardened_wasmtime_runtime::artifact::shape::admit(&facts(), &drifted),
        Err(hardened_wasmtime_runtime::artifact::shape::AdmissionError::IdentityMismatch)
    );
    let mut swapped = inspected();
    swapped.class = hardened_wasmtime_runtime::artifact::shape::Class::CoreModule;
    assert_eq!(
        hardened_wasmtime_runtime::artifact::shape::admit(&facts(), &swapped),
        Err(hardened_wasmtime_runtime::artifact::shape::AdmissionError::ClassMismatch)
    );
    let mut reworlded = inspected();
    reworlded.world.world = String::from("other");
    assert_eq!(
        hardened_wasmtime_runtime::artifact::shape::admit(&facts(), &reworlded),
        Err(hardened_wasmtime_runtime::artifact::shape::AdmissionError::WorldMismatch)
    );
}
