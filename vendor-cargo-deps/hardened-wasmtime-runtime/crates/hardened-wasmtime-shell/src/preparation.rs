//! Bounded artifact, exact import/export and injected-deadline admission.

/// Bytes and trusted-by-the-consumer materialization facts for one artifact.
pub struct Artifact<'a> {
    /// Raw artifact bytes, measured before compilation.
    pub bytes: &'a [u8],
    /// Expected exact identity, class, world and interface facts.
    pub facts: &'a hardened_wasmtime_runtime::artifact::shape::MaterializationFacts,
}

/// Explicit facts required before every module or component invocation.
pub struct PrepareOptions<'a> {
    /// Artifact bytes and expected materialization facts.
    pub artifact: Artifact<'a>,
    /// Exact consumer-admitted import descriptors; empty means no imports.
    pub descriptors: &'a [hardened_wasmtime_runtime::restricted::imports::Descriptor],
    /// Positive finite epoch budget for the whole invocation.
    pub deadline: hardened_wasmtime_runtime::epoch::deadline::Budget,
    /// Injected driver for the exact engine; its operation remains consumer-owned.
    pub epoch_service: Option<&'a crate::EpochService>,
}

/// Preparation is effect-free with respect to consumer bindings and guest code.
#[derive(Debug, PartialEq, Eq)]
pub enum PrepareError {
    /// Compilation input exceeds the admitted byte budget.
    ArtifactTooLarge,
    /// Artifact identity, class, world or import/export names differ.
    ArtifactMismatch,
    /// A descriptor is malformed, unbound, duplicated or WASI-bearing.
    ImportAdmission(hardened_wasmtime_runtime::restricted::imports::ImportError),
    /// Descriptor names differ from the artifact's exact function imports.
    ImportMismatch,
    /// Epoch budget is zero, non-finite, absent, or tied to another engine.
    DeadlineDenied,
    /// Artifact compilation failed under the explicit engine policy.
    CompileFailed,
}

pub(crate) struct Prepared {
    pub runtime: crate::BoundedRuntime,
    pub descriptors: Vec<hardened_wasmtime_runtime::restricted::imports::Descriptor>,
    pub ticks: u64,
}

/// Unambiguous function path, shared by observed names and consumer descriptors.
/// Each UTF-8 segment is prefixed with its byte length and a colon.
#[must_use]
pub fn qualified_name(parts: &[&str]) -> String {
    let mut name = String::new();
    for part in parts {
        name = append_qualified(name, part);
    }
    name
}

pub(crate) fn append_qualified(mut name: String, part: &str) -> String {
    use std::fmt::Write as _;
    assert!(
        write!(&mut name, "{}:{part}", part.len()).is_ok(),
        "formatting into String is infallible"
    );
    name
}

pub(crate) fn admit(
    runtime: &crate::BoundedRuntime,
    options: &PrepareOptions<'_>,
    class: hardened_wasmtime_runtime::artifact::shape::Class,
) -> Result<Prepared, PrepareError> {
    admit_artifact(&options.artifact, class, runtime.bounds.max_component_bytes)?;
    admit_interface_counts(runtime, options)?;
    let ticks = admit_deadline(runtime, options)?;
    admit_import_set(options)?;
    Ok(Prepared {
        runtime: runtime.clone(),
        descriptors: options.descriptors.to_vec(),
        ticks,
    })
}

fn admit_artifact(
    artifact: &Artifact<'_>,
    class: hardened_wasmtime_runtime::artifact::shape::Class,
    max_bytes: u64,
) -> Result<(), PrepareError> {
    if artifact.bytes.len() as u64 > max_bytes {
        return Err(PrepareError::ArtifactTooLarge);
    }
    admit_world(artifact.facts, class)?;
    if blake3::hash(artifact.bytes).to_hex().as_str() != artifact.facts.artifact_identity {
        return Err(PrepareError::ArtifactMismatch);
    }
    Ok(())
}

const fn admit_world(
    facts: &hardened_wasmtime_runtime::artifact::shape::MaterializationFacts,
    class: hardened_wasmtime_runtime::artifact::shape::Class,
) -> Result<(), PrepareError> {
    if !is_same_class((facts.class, class)) {
        return Err(PrepareError::ArtifactMismatch);
    }
    if facts.world.package.is_empty() {
        return Err(PrepareError::ArtifactMismatch);
    }
    if facts.world.world.is_empty() {
        return Err(PrepareError::ArtifactMismatch);
    }
    Ok(())
}

const fn is_same_class(
    classes: (
        hardened_wasmtime_runtime::artifact::shape::Class,
        hardened_wasmtime_runtime::artifact::shape::Class,
    ),
) -> bool {
    matches!(
        classes,
        (
            hardened_wasmtime_runtime::artifact::shape::Class::CoreModule,
            hardened_wasmtime_runtime::artifact::shape::Class::CoreModule
        ) | (
            hardened_wasmtime_runtime::artifact::shape::Class::Component,
            hardened_wasmtime_runtime::artifact::shape::Class::Component
        )
    )
}

fn admit_interface_counts(
    runtime: &crate::BoundedRuntime,
    options: &PrepareOptions<'_>,
) -> Result<(), PrepareError> {
    let facts = options.artifact.facts;
    admit_count(facts.imports.len(), runtime.bounds.max_imports)?;
    admit_count(facts.exports.len(), runtime.bounds.max_exports)?;
    admit_count(options.descriptors.len(), runtime.bounds.max_imports)
}

const fn admit_count(count: usize, limit: u32) -> Result<(), PrepareError> {
    if count as u64 > limit as u64 {
        return Err(PrepareError::ArtifactTooLarge);
    }
    Ok(())
}

fn admit_deadline(
    runtime: &crate::BoundedRuntime,
    options: &PrepareOptions<'_>,
) -> Result<u64, PrepareError> {
    let service = options.epoch_service.ok_or(PrepareError::DeadlineDenied)?;
    if !wasmtime::Engine::same(&runtime.engine, &service.engine) {
        return Err(PrepareError::DeadlineDenied);
    }
    if options.deadline.budget_ticks == u64::MAX {
        return Err(PrepareError::DeadlineDenied);
    }
    hardened_wasmtime_runtime::epoch::deadline::admit(options.deadline)
        .map(|deadline| deadline.budget_ticks)
        .map_err(|_| PrepareError::DeadlineDenied)
}

fn admit_import_set(options: &PrepareOptions<'_>) -> Result<(), PrepareError> {
    let facts = options.artifact.facts;
    let bound_functions =
        hardened_wasmtime_runtime::restricted::imports::admit(options.descriptors, &facts.world)
            .map_err(PrepareError::ImportAdmission)?;
    if bound_functions != facts.imports {
        return Err(PrepareError::ImportMismatch);
    }
    Ok(())
}

pub(crate) fn names_match(
    observed: impl IntoIterator<Item = String>,
    expected: &std::collections::BTreeSet<
        hardened_wasmtime_runtime::artifact::shape::InterfaceFunction,
    >,
    limit: u32,
) -> Result<(), PrepareError> {
    let mut names = std::collections::BTreeSet::new();
    for name in observed {
        if names.len() as u64 >= u64::from(limit) || !names.insert(name) {
            return Err(PrepareError::ArtifactMismatch);
        }
    }
    if names.len() != expected.len()
        || !expected.iter().all(|function| {
            !function.schema_identity.is_empty() && names.contains(&function.qualified_name)
        })
    {
        return Err(PrepareError::ArtifactMismatch);
    }
    Ok(())
}
