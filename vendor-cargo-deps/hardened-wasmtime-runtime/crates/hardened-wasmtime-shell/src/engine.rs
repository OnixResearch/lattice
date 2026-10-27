//! Engine construction from explicit, revalidated profile facts.
// r[impl onix.wasmtime_runtime.feature_policy]

/// Engine bound to one complete feature policy and cumulative resource budget.
#[derive(Clone)]
pub struct BoundedRuntime {
    pub(crate) engine: wasmtime::Engine,
    pub(crate) bounds: hardened_wasmtime_runtime::resource::admission::RuntimeBounds,
    pub(crate) store_limits: wasmtime::StoreLimits,
    pub(crate) total_memory_bytes: usize,
    pub(crate) transfer_bytes: usize,
    pub(crate) active: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

/// Engine construction failed before execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildFailure {
    /// Feature facts failed admission or their identity was forged.
    ProfileDenied,
    /// Resource limits failed admission or platform conversion.
    BoundsDenied,
    /// Wasmtime rejected the explicit configuration.
    ConfigurationRejected,
}

/// An injected epoch driver tied to the exact engine it advances.
/// The consumer must operate it; this crate creates no clocks or threads.
#[derive(Clone)]
pub struct EpochService {
    pub(crate) engine: wasmtime::Engine,
}

impl EpochService {
    /// Advance this engine's epoch by one tick.
    pub fn advance(&self) {
        self.engine.increment_epoch();
    }
}

impl BoundedRuntime {
    /// Construct an engine enforcing the complete admitted policy and bounds.
    pub fn build(
        policy: &hardened_wasmtime_runtime::feature::policy::Admitted,
        bounds: hardened_wasmtime_runtime::resource::admission::RuntimeBounds,
        strategy: hardened_wasmtime_runtime::Strategy,
    ) -> Result<Self, BuildFailure> {
        let admitted =
            hardened_wasmtime_runtime::feature::policy::admit(policy.features, policy.determinism)
                .map_err(|_| BuildFailure::ProfileDenied)?;
        if admitted != *policy {
            return Err(BuildFailure::ProfileDenied);
        }
        let bounds = hardened_wasmtime_runtime::resource::admission::admit_limits(bounds)
            .map_err(|_| BuildFailure::BoundsDenied)?;
        let memory_bytes = memory_bound(bounds.max_memory_pages)?;
        let total_memory_bytes = memory_bound(bounds.max_total_memory_pages)?;
        let mut config = configuration(policy, bounds.max_stack_bytes)?;
        config.strategy(match strategy {
            hardened_wasmtime_runtime::Strategy::Cranelift => wasmtime::Strategy::Cranelift,
            hardened_wasmtime_runtime::Strategy::Winch => wasmtime::Strategy::Winch,
        });
        let engine =
            wasmtime::Engine::new(&config).map_err(|_| BuildFailure::ConfigurationRejected)?;
        // r[impl onix.wasmtime_runtime.independent_memory_limits]
        let transfer_bytes = platform_bound(bounds.max_output_bytes)?;
        let store_caps = wasmtime::StoreLimitsBuilder::new()
            .memory_size(memory_bytes)
            .memories(platform_bound(u64::from(bounds.max_memories))?)
            .tables(platform_bound(u64::from(bounds.max_tables))?)
            .table_elements(platform_bound(u64::from(bounds.max_table_elements))?)
            .instances(platform_bound(u64::from(bounds.max_instances))?)
            .trap_on_grow_failure(true)
            .build();
        Ok(Self {
            engine,
            bounds,
            store_limits: store_caps,
            total_memory_bytes,
            transfer_bytes,
            active: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
        })
    }

    /// Obtain an epoch driver; the caller owns when and how it is advanced.
    #[must_use]
    pub fn epoch_service(&self) -> EpochService {
        EpochService {
            engine: self.engine.clone(),
        }
    }

    pub(crate) fn enter(&self) -> Option<crate::concurrency::Permit<'_>> {
        crate::concurrency::reserve(&self.active, self.bounds.max_concurrency)
            .then(|| crate::concurrency::Permit(&self.active))
    }

    pub(crate) fn enter_owned(&self) -> Option<crate::concurrency::OwnedPermit> {
        if !crate::concurrency::reserve(&self.active, self.bounds.max_concurrency) {
            return None;
        }
        Some(crate::concurrency::OwnedPermit(std::sync::Arc::clone(
            &self.active,
        )))
    }
}

fn platform_bound(value: u64) -> Result<usize, BuildFailure> {
    usize::try_from(value).map_err(|_| BuildFailure::BoundsDenied)
}

fn memory_bound(pages: u32) -> Result<usize, BuildFailure> {
    let bytes = u64::from(pages)
        .checked_mul(hardened_wasmtime_runtime::resource::admission::WASM_PAGE_BYTES)
        .ok_or(BuildFailure::BoundsDenied)?;
    platform_bound(bytes)
}

fn configuration(
    policy: &hardened_wasmtime_runtime::feature::policy::Admitted,
    stack_bytes: u64,
) -> Result<wasmtime::Config, BuildFailure> {
    let mut config = wasmtime::Config::new();
    configure_admitted(&mut config, policy.features);
    configure_denied(&mut config);
    config
        .wasm_component_model_error_context(false)
        .wasm_component_model_gc(false)
        .wasm_component_model_map(false)
        .wasm_component_model_fixed_length_lists(false)
        .consume_fuel(policy.determinism.fuel_interruption)
        .cranelift_nan_canonicalization(policy.determinism.nan_canonicalization)
        .relaxed_simd_deterministic(policy.determinism.relaxed_simd_deterministic)
        .epoch_interruption(true)
        .max_wasm_stack(platform_bound(stack_bytes)?);
    // Threads and component async are compiled out and denied by core admission.
    Ok(config)
}

fn configure_admitted(
    config: &mut wasmtime::Config,
    features: hardened_wasmtime_runtime::feature::policy::WasmtimeFeatures,
) {
    config
        .wasm_bulk_memory(features.bulk_memory)
        .wasm_component_model(features.component_model)
        .wasm_multi_value(features.multi_value)
        .wasm_reference_types(features.reference_types)
        .wasm_simd(features.simd);
}

fn configure_denied(config: &mut wasmtime::Config) {
    config
        .wasm_tail_call(false)
        .wasm_wide_arithmetic(false)
        .wasm_memory64(false)
        .wasm_multi_memory(false)
        .wasm_gc(false)
        .collector(wasmtime::Collector::Null)
        .wasm_exceptions(false)
        .wasm_relaxed_simd(false)
        .wasm_custom_page_sizes(false)
        .wasm_extended_const(false)
        .wasm_function_references(false);
}
