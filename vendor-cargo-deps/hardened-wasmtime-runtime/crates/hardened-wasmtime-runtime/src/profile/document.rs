//! Admission of the checked Molten component-runtime profile document.
// r[impl onix.wasmtime_runtime.feature_policy]

/// Schema identifier this admission accepts.
pub const PROFILE_SCHEMA_ID: &str = "molten.wasm-component-profile.v1";

/// Admitted profile identifier.
pub const ADMITTED_DOCUMENT_ID: &str = "molten.wasm.component.v1";

/// Profile-document admission failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The document is not valid JSON or does not match the schema.
    MalformedDocument(String),
    /// `schema_id` does not name the admitted schema.
    UnknownSchema(String),
    /// `schema_version` is not the admitted major version.
    UnsupportedSchemaVersion(u32),
    /// `profile_id` does not name the admitted profile.
    UnknownProfile(String),
    /// The growth policy string is not `fixed`.
    UnknownGrowthPolicy(String),
    /// The host-input policy is not the admitted recorded-effects-only.
    UnknownHostInputs(String),
    /// The profile admits ambient imports or WASI interfaces; the base
    /// runtime is deny-by-default.
    AmbientImportsAdmitted,
    /// The runtime strategy is not the admitted portable-component one.
    UnknownRuntimeStrategy(String),
    /// The source language is not the admitted Nickel one.
    UnknownSourceLanguage(String),
    /// The pinned toolchain does not match the admitted revisions.
    UnknownToolchainPin(String),
    /// The evidence scope is not the admitted production scope.
    UnknownEvidenceScope(String),
    /// The profile carries no non-claims.
    EmptyNonClaims,
    /// A resource limit is zero; every limit must be positive.
    NonPositiveResourceLimit(String),
    /// Memory bytes are not a whole number of WebAssembly pages.
    MemoryNotPageAligned,
    /// The feature or determinism facts were rejected by policy admission.
    PolicyAdmission(String),
}

/// An admitted profile document: feature policy plus resource limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admitted {
    /// The admitted explicit feature policy.
    pub policy: crate::feature::policy::Admitted,
    /// The admitted cumulative resource limits.
    pub limits: crate::resource::admission::RuntimeBounds,
}

/// Admit a profile document and return its admitted feature policy and
/// resource limits.
///
/// Every feature fact is required; `deny_unknown_fields` turns unknown or
/// renamed facts into hard errors, so an older or drifted profile cannot
/// admit by omission.
/// Check document, profile, strategy, scope, language, WIT, and
/// non-claim identities.
pub fn check_identities(parsed: &super::types::Document) -> Result<(), Error> {
    assert!(!PROFILE_SCHEMA_ID.is_empty());
    assert!(!ADMITTED_DOCUMENT_ID.is_empty());
    assert_eq!(PROFILE_SCHEMA_ID, "molten.wasm-component-profile.v1");
    if parsed.schema_id != PROFILE_SCHEMA_ID {
        return Err(Error::UnknownSchema(parsed.schema_id.clone()));
    }
    if parsed.schema_version != 1 {
        return Err(Error::UnsupportedSchemaVersion(parsed.schema_version));
    }
    if parsed.source_language != "nickel" {
        return Err(Error::UnknownSourceLanguage(parsed.source_language.clone()));
    }
    if parsed.profile.profile_id != ADMITTED_DOCUMENT_ID {
        return Err(Error::UnknownProfile(parsed.profile.profile_id.clone()));
    }
    if parsed.profile.runtime_strategy != "portable-component" {
        return Err(Error::UnknownRuntimeStrategy(
            parsed.profile.runtime_strategy.clone(),
        ));
    }
    if parsed.profile.evidence_scope != "production" {
        return Err(Error::UnknownEvidenceScope(
            parsed.profile.evidence_scope.clone(),
        ));
    }
    if parsed.profile.non_claims.is_empty() {
        return Err(Error::EmptyNonClaims);
    }
    if parsed.profile.wit.package != "molten:component-runtime@1.0.0"
        || parsed.profile.wit.world != "actor"
        || !parsed.profile.wit.source_ref.starts_with("blake3:")
    {
        return Err(Error::UnknownProfile(parsed.profile.wit.package.clone()));
    }
    Ok(())
}

/// The base runtime admits no ambient imports or WASI surfaces.
pub fn check_ambient_imports(parsed: &super::types::Document) -> Result<(), Error> {
    if !parsed.profile.allowed_imports.is_empty()
        || !parsed.profile.allowed_wasi_interfaces.is_empty()
    {
        return Err(Error::AmbientImportsAdmitted);
    }
    if parsed.profile.determinism.host_inputs != "recorded-effects-only" {
        return Err(Error::UnknownHostInputs(
            parsed.profile.determinism.host_inputs.clone(),
        ));
    }
    Ok(())
}

/// Pin the toolchain revisions the extraction was reviewed against.
pub fn check_toolchain_pins(toolchain: &super::types::ToolchainFacts) -> Result<(), Error> {
    let pins = [
        ("wasmtime", toolchain.wasmtime.as_str(), "45.0.0"),
        ("wasmtime_wasi", toolchain.wasmtime_wasi.as_str(), "45.0.0"),
        ("wasmparser", toolchain.wasmparser.as_str(), "0.240.0"),
        ("wasm_tools", toolchain.wasm_tools.as_str(), "1.240.0"),
        ("wit_bindgen", toolchain.wit_bindgen.as_str(), "0.46.0"),
        ("wasi_package", toolchain.wasi_package.as_str(), "0.2.6"),
    ];
    for (name, actual, expected) in pins {
        if actual != expected {
            return Err(Error::UnknownToolchainPin(format!(
                "{name}={actual}, expected {expected}"
            )));
        }
    }
    Ok(())
}

/// Every resource limit must be positive.
pub fn check_resource_positivity(resources: &super::types::Resources) -> Result<(), Error> {
    assert!(resources.fuel > 0);
    assert!(resources.max_memory_bytes >= resources.fuel);
    let counted: [(&'static str, u32); 7] = [
        ("max_concurrency", resources.max_concurrency),
        ("max_exports", resources.max_exports),
        ("max_imports", resources.max_imports),
        ("max_instances", resources.max_instances),
        ("max_memories", resources.max_memories),
        ("max_table_elements", resources.max_table_elements),
        ("max_tables", resources.max_tables),
    ];
    for (name, value) in counted {
        if value == 0 {
            return Err(Error::NonPositiveResourceLimit(String::from(name)));
        }
    }
    let byte_limit_checks_bytes: [(&'static str, u64); 7] = [
        ("fuel", resources.fuel),
        ("max_component_bytes", resources.max_component_bytes),
        ("max_hostcall_bytes", resources.max_hostcall_bytes),
        ("max_memory_bytes", resources.max_memory_bytes),
        ("max_result_bytes", resources.max_result_bytes),
        ("max_stack_bytes", resources.max_stack_bytes),
        ("max_wit_bytes", resources.max_wit_bytes),
    ];
    for (name, value) in byte_limit_checks_bytes {
        if value == 0 {
            return Err(Error::NonPositiveResourceLimit(String::from(name)));
        }
    }
    Ok(())
}

/// Derive the shell's runtime limits from the admitted profile facts.
#[allow(clippy::too_many_lines)]
fn build_limits(
    features: crate::feature::policy::WasmtimeFeatures,
    determinism: crate::feature::policy::Determinism,
    resources: &super::types::Resources,
) -> Result<Admitted, Error> {
    assert!(resources.max_memory_bytes.is_multiple_of(65_536));
    assert!(resources.max_memory_bytes >= 65_536);
    let policy = crate::feature::policy::admit(features, determinism)
        .map_err(|error| Error::PolicyAdmission(format!("{error:?}")))?;
    // The frozen document's memory cap has always bounded both each memory
    // and cumulative invocation allocations; retain both compatibility limits.
    let memory_pages =
        u32::try_from(resources.max_memory_bytes / crate::resource::admission::WASM_PAGE_BYTES)
            .map_err(|_| Error::MemoryNotPageAligned)?;
    let admitted_runtime_bounds =
        crate::resource::admission::admit_limits(crate::resource::admission::RuntimeBounds {
            fuel: resources.fuel,
            max_stack_bytes: resources.max_stack_bytes,
            max_memory_pages: memory_pages,
            max_total_memory_pages: memory_pages,
            max_tables: resources.max_tables,
            max_table_elements: resources.max_table_elements,
            max_instances: resources.max_instances,
            max_memories: resources.max_memories,
            max_input_bytes: resources.max_hostcall_bytes,
            max_output_bytes: resources.max_result_bytes,
            max_component_bytes: resources.max_component_bytes,
            max_wit_bytes: resources.max_wit_bytes,
            max_imports: resources.max_imports,
            max_exports: resources.max_exports,
            max_concurrency: resources.max_concurrency,
        })
        .map_err(|error| Error::PolicyAdmission(format!("{error:?}")))?;
    Ok(Admitted {
        policy,
        limits: admitted_runtime_bounds,
    })
}

/// Const-compatible string equality.
const fn str_eq(a: &str, b: &[u8]) -> bool {
    let a = a.as_bytes();
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Map document feature and determinism facts to the policy inputs.
pub const fn policy_facts(
    parsed: &super::types::Document,
) -> (
    crate::feature::policy::WasmtimeFeatures,
    crate::feature::policy::Determinism,
) {
    let facts = &parsed.profile.features;
    let features = crate::feature::policy::WasmtimeFeatures {
        bulk_memory: facts.bulk_memory,
        component_model: facts.component_model,
        multi_value: facts.multi_value,
        reference_types: facts.reference_types,
        simd: facts.simd,
        tail_call: facts.tail_call,
        wide_arithmetic: facts.wide_arithmetic,
        memory64: facts.memory64,
        multi_memory: facts.multi_memory,
        threads: facts.threads,
        gc: facts.gc,
        exceptions: facts.exceptions,
        relaxed_simd: facts.relaxed_simd,
        custom_page_sizes: facts.custom_page_sizes,
        extended_const: facts.extended_const,
        function_references: facts.function_references,
        component_async: facts.component_async,
    };
    let determinism = crate::feature::policy::Determinism {
        fuel_interruption: parsed.profile.determinism.fuel_interruption,
        nan_canonicalization: parsed.profile.determinism.nan_canonicalization,
        relaxed_simd_deterministic: parsed.profile.determinism.relaxed_simd_deterministic,
        memory_growth_fixed: str_eq(parsed.profile.determinism.memory_growth.as_str(), b"fixed"),
        table_growth_fixed: str_eq(parsed.profile.determinism.table_growth.as_str(), b"fixed"),
    };
    assert!(features.component_model == facts.component_model);
    assert!(determinism.fuel_interruption == parsed.profile.determinism.fuel_interruption);
    (features, determinism)
}

/// Admit a profile document and return its admitted feature policy and
/// resource limits.
///
/// Every feature fact is required; `deny_unknown_fields` turns unknown or
/// renamed facts into hard errors, so an older or drifted profile cannot
/// admit by omission.
pub fn admit(document: &str) -> Result<Admitted, Error> {
    let parsed: super::types::Document = serde_json::from_str(document)
        .map_err(|error| Error::MalformedDocument(error.to_string()))?;
    check_identities(&parsed)?;
    check_ambient_imports(&parsed)?;
    check_toolchain_pins(&parsed.profile.toolchain)?;
    let resources = &parsed.profile.resources;
    check_resource_positivity(resources)?;
    if parsed.profile.determinism.memory_growth != "fixed"
        || parsed.profile.determinism.table_growth != "fixed"
    {
        return Err(Error::UnknownGrowthPolicy(String::from("unbounded")));
    }
    let (features, determinism) = policy_facts(&parsed);
    build_limits(features, determinism, resources)
}
