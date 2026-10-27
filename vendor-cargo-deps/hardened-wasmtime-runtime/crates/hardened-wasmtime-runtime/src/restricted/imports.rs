//! Deny-by-default restricted import admission.
//!
//! The base runtime links nothing. A consumer adapter supplies an exact
//! import descriptor set; every descriptor binds an interface-qualified
//! function name to one world, operation, schema, capability, and adapter
//! identity. WASI names, empty bindings, and cross-request capability
//! bindings are denied.
// r[impl onix.wasmtime_runtime.restricted_imports]

/// A fully bound import descriptor supplied by a consumer adapter.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Descriptor {
    /// Exact interface-qualified function name the guest may import.
    pub function: crate::artifact::shape::InterfaceFunction,
    /// The world this binding is valid for.
    pub world: crate::artifact::shape::WorldIdentity,
    /// Opaque capability observation bound to one request or operation.
    pub capability_id: String,
    /// Identifier of the adapter supplying the host function.
    pub adapter_id: String,
}

/// Import-descriptor admission failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// A descriptor is bound to a different world.
    WorldMismatch,
    /// A descriptor names a WASI interface; WASI is never linked.
    WasiImport(String),
    /// A descriptor is missing its capability, schema, or adapter binding.
    UnboundDescriptor,
    /// Two descriptors bind the same function name.
    DuplicateBinding(String),
}

/// Prefixes that identify WASI interfaces. A base-profile guest that
/// imports any of these is denied at instantiation.
pub const WASI_INTERFACE_PREFIXES: [&str; 4] =
    ["wasi:", "wasi_unstable", "wasi_snapshot_preview1", "wasi_"];

/// Decide whether a function name belongs to a WASI interface.
#[must_use]
pub fn is_wasi_name(qualified_name: &str) -> bool {
    if has_wasi_prefix(qualified_name) {
        return true;
    }
    let mut remaining = qualified_name;
    while let Some((segment, tail)) = next_segment(remaining) {
        if has_wasi_prefix(segment) {
            return true;
        }
        remaining = tail;
    }
    false
}

fn has_wasi_prefix(name: &str) -> bool {
    WASI_INTERFACE_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn next_segment(encoded: &str) -> Option<(&str, &str)> {
    let (length, tail) = encoded.split_once(':')?;
    let length_bytes = length.parse::<usize>().ok()?;
    Some((tail.get(..length_bytes)?, tail.get(length_bytes..)?))
}

/// Admit an exact import-descriptor set for one world. Only admitted
/// names may be linked; duplicates and unbound descriptors fail closed.
pub fn admit(
    descriptors: &[Descriptor],
    world: &crate::artifact::shape::WorldIdentity,
) -> Result<std::collections::BTreeSet<crate::artifact::shape::InterfaceFunction>, ImportError> {
    if world.package.is_empty() || world.world.is_empty() {
        return Err(ImportError::WorldMismatch);
    }
    let mut admitted = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for descriptor in descriptors {
        descriptor.validate(world)?;
        if !names.insert(descriptor.function.qualified_name.as_str()) {
            return Err(ImportError::DuplicateBinding(
                descriptor.function.qualified_name.clone(),
            ));
        }
        admitted.insert(descriptor.function.clone());
    }
    Ok(admitted)
}

impl Descriptor {
    fn validate(&self, world: &crate::artifact::shape::WorldIdentity) -> Result<(), ImportError> {
        if self.world != *world {
            return Err(ImportError::WorldMismatch);
        }
        if is_wasi_name(&self.function.qualified_name) {
            return Err(ImportError::WasiImport(String::from(
                self.function.qualified_name.as_str(),
            )));
        }
        if !self.is_bound() {
            return Err(ImportError::UnboundDescriptor);
        }
        Ok(())
    }

    fn is_bound(&self) -> bool {
        let bindings = [
            self.function.qualified_name.as_str(),
            self.capability_id.as_str(),
            self.function.schema_identity.as_str(),
            self.adapter_id.as_str(),
        ];
        bindings.iter().all(|binding| !binding.is_empty())
    }
}
