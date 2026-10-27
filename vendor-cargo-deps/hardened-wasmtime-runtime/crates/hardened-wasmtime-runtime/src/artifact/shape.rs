//! Exact artifact-shape admission.
//!
//! Compares the facts a consumer declares about one artifact (identity,
//! class, world, imports, exports) against the facts the artifact itself
//! carries. Every field must agree exactly; any drift denies compilation
//! and instantiation before either can start.
// r[impl onix.wasmtime_runtime.artifact_shape]

/// The two artifact classes the runtime admits.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    /// A WebAssembly component.
    Component,
    /// A core WebAssembly module.
    CoreModule,
}

/// Identity of the component world the artifact targets.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldIdentity {
    /// WIT package name, for example `molten:component-runtime@1.0.0`.
    pub package: String,
    /// World name, for example `actor`.
    pub world: String,
}

/// One declared import or export with its exact operation binding.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InterfaceFunction {
    /// Exact interface-qualified name, for example
    /// `molten:actor/operate@1.0.0.run`.
    pub qualified_name: String,
    /// Exact schema identity bound to the operation.
    pub schema_identity: String,
}

/// The facts a consumer declares about one artifact before admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializationFacts {
    /// BLAKE3 hex identity of the exact artifact bytes.
    pub artifact_identity: String,
    /// Declared artifact class.
    pub class: Class,
    /// Declared world identity.
    pub world: WorldIdentity,
    /// Declared import set.
    pub imports: std::collections::BTreeSet<InterfaceFunction>,
    /// Declared export set.
    pub exports: std::collections::BTreeSet<InterfaceFunction>,
}

/// The facts the artifact itself exposes after inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inspected {
    /// BLAKE3 hex identity computed over the exact artifact bytes.
    pub artifact_identity: String,
    /// Inspected artifact class.
    pub class: Class,
    /// Inspected world identity.
    pub world: WorldIdentity,
    /// Inspected import set.
    pub imports: std::collections::BTreeSet<InterfaceFunction>,
    /// Inspected export set.
    pub exports: std::collections::BTreeSet<InterfaceFunction>,
}

/// Artifact-shape admission failure. Each variant names the drifted field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmissionError {
    /// The artifact bytes differ from the declared identity.
    IdentityMismatch,
    /// The artifact class differs from the declared class.
    ClassMismatch,
    /// The world identity differs.
    WorldMismatch,
    /// The inspected imports differ from the declared imports.
    ImportDrift,
    /// The inspected exports differ from the declared exports.
    ExportDrift,
}

/// Admit one artifact only when every inspected fact matches the declared
/// facts exactly.
pub fn admit(declared: &MaterializationFacts, inspected: &Inspected) -> Result<(), AdmissionError> {
    if declared.artifact_identity != inspected.artifact_identity {
        return Err(AdmissionError::IdentityMismatch);
    }
    if declared.class != inspected.class {
        return Err(AdmissionError::ClassMismatch);
    }
    if declared.world != inspected.world {
        return Err(AdmissionError::WorldMismatch);
    }
    if declared.imports != inspected.imports {
        return Err(AdmissionError::ImportDrift);
    }
    if declared.exports != inspected.exports {
        return Err(AdmissionError::ExportDrift);
    }
    Ok(())
}
