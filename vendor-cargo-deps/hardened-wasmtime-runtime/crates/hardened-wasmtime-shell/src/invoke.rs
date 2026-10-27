//! Bounded core-module invocation with exact consumer-owned linking.
// r[impl onix.wasmtime_runtime.resource_limits.output]
// r[impl onix.wasmtime_runtime.failure_phase]

/// Invalid wasm32 memory span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuestPointerError {
    /// Pointer and length overflow wasm32.
    Overflow,
    /// Span exceeds current memory.
    OutOfRange,
}

/// Checked wasm32 pointer and byte count.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GuestSpan {
    /// Start in linear memory.
    pub pointer: u32,
    /// Byte count.
    pub byte_len: u32,
}
impl GuestSpan {
    /// Validate arithmetic before constructing a span.
    pub fn new(pointer: u32, byte_len: u32) -> Result<Self, GuestPointerError> {
        pointer
            .checked_add(byte_len)
            .ok_or(GuestPointerError::Overflow)?;
        Ok(Self { pointer, byte_len })
    }
}

/// Validate both arithmetic and the current guest-memory boundary.
pub fn check_guest_range(span: GuestSpan, memory_bytes: u64) -> Result<(), GuestPointerError> {
    let end = span
        .pointer
        .checked_add(span.byte_len)
        .ok_or(GuestPointerError::Overflow)?;
    if u64::from(end) > memory_bytes {
        return Err(GuestPointerError::OutOfRange);
    }
    Ok(())
}

/// Prepared core module using the memory/run(ptr,len)->length exchange ABI.
pub struct BoundedInvocation {
    prepared: crate::preparation::Prepared,
    module: wasmtime::Module,
}
impl BoundedInvocation {
    /// Validate identity, bounds, class, epoch and exact import/export names.
    pub fn prepare(
        runtime: &crate::BoundedRuntime,
        options: crate::PrepareOptions<'_>,
    ) -> Result<Self, crate::PrepareError> {
        let prepared = crate::preparation::admit(
            runtime,
            &options,
            hardened_wasmtime_runtime::artifact::shape::Class::CoreModule,
        )?;
        let module = wasmtime::Module::new(&runtime.engine, options.artifact.bytes)
            .map_err(|_| crate::PrepareError::CompileFailed)?;
        crate::preparation::names_match(
            module
                .imports()
                .map(|import| crate::qualified_name(&[import.module(), import.name()])),
            &options.artifact.facts.imports,
            runtime.bounds.max_imports,
        )?;
        crate::preparation::names_match(
            module
                .exports()
                .map(|export| crate::qualified_name(&[export.name()])),
            &options.artifact.facts.exports,
            runtime.bounds.max_exports,
        )?;
        Ok(Self { prepared, module })
    }

    /// Run with an exact consumer binder. Host state and authorization remain consumer-owned.
    pub fn run<T: 'static, F>(&self, input: &[u8], host: T, mut bind: F) -> crate::InvocationOutcome
    where
        F: FnMut(
            &hardened_wasmtime_runtime::restricted::imports::Descriptor,
            &mut wasmtime::Linker<crate::State<T>>,
        ) -> Result<(), crate::HostDenied>,
    {
        let runtime = &self.prepared.runtime;
        if input.len() as u64 > runtime.bounds.max_input_bytes {
            return crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded,
            );
        }
        let Some(_permit) = runtime.enter() else {
            return crate::store::failure(
                hardened_wasmtime_runtime::outcomes::Class::ConcurrencyExhausted,
            );
        };
        let mut store = match crate::store::create(runtime, host, self.prepared.ticks) {
            Ok(store) => store,
            Err(class) => return crate::store::failure(class),
        };
        let mut linker = wasmtime::Linker::new(&runtime.engine);
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Linking;
        for descriptor in &self.prepared.descriptors {
            if bind(descriptor, &mut linker).is_err() {
                return crate::store::finish(
                    &store,
                    Err(hardened_wasmtime_runtime::outcomes::Class::HostFailed),
                );
            }
        }
        let result = self.execute(&mut store, &linker, input);
        crate::store::finish(&store, result)
    }

    fn execute<T: 'static>(
        &self,
        store: &mut wasmtime::Store<crate::State<T>>,
        linker: &wasmtime::Linker<crate::State<T>>,
        input: &[u8],
    ) -> Result<Vec<u8>, hardened_wasmtime_runtime::outcomes::Class> {
        store.data_mut().phase =
            hardened_wasmtime_runtime::outcomes::InvocationPhase::Instantiation;
        let instance = linker
            .instantiate(&mut *store, &self.module)
            .map_err(crate::store::instantiation_error)?;
        store.data_mut().phase = hardened_wasmtime_runtime::outcomes::InvocationPhase::Invocation;
        let memory = instance
            .get_memory(&mut *store, "memory")
            .ok_or(hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied)?;
        let len = u32::try_from(input.len())
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded)?;
        check_guest_range(
            GuestSpan::new(0, len)
                .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected)?,
            memory.data_size(&*store) as u64,
        )
        .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected)?;
        memory
            .write(&mut *store, 0, input)
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected)?;
        let run = instance
            .get_typed_func::<(i32, i32), i32>(&mut *store, "run")
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::ArtifactDenied)?;
        let input_len = i32::try_from(len)
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::InputLimitExceeded)?;
        let written = run
            .call(&mut *store, (0, input_len))
            .map_err(|error| crate::store::classify(&error))?;
        let len = u32::try_from(written)
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected)?;
        if u64::from(len) > self.prepared.runtime.bounds.max_output_bytes {
            return Err(hardened_wasmtime_runtime::outcomes::Class::OutputLimitExceeded);
        }
        check_guest_range(
            GuestSpan::new(0, len)
                .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected)?,
            memory.data_size(&*store) as u64,
        )
        .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::GuestPointerRejected)?;
        let end = usize::try_from(len)
            .map_err(|_| hardened_wasmtime_runtime::outcomes::Class::OutputLimitExceeded)?;
        Ok(memory.data(&*store)[..end].to_vec())
    }
}
